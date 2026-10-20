//! Outcrops Odyssey — Bevy 3D prototype.
//!
//! The rules, world generation, roads and truck routing are the *same code* as the
//! Macroquad game (`../src/game.rs`, `../src/world.rs`, pulled in with #[path]); this crate
//! only renders that state in 3D: PBR ground/rocks, a glTF tree, sun + shadows, fog and a
//! day/night cycle driven by the game clock.
//!
//! Coordinates: game units are y-up on a plane; Bevy is Y-up 3D. We map
//! game (x, y) -> Bevy (x*S, 0, -y*S), so "north" (game +y) is Bevy -Z.

#[path = "../../src/game.rs"]
#[allow(dead_code)]
mod game;
#[path = "../../src/world.rs"]
#[allow(dead_code)]
mod world;

use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::light::{CascadeShadowConfigBuilder, GlobalAmbientLight};
use bevy::mesh::VertexAttributeValues;
use bevy::pbr::DistanceFog;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use game::*;
use std::collections::HashMap;
use world::*;

/// metres per game unit: a 640-unit chunk is 16 m, the 80-unit player ~2 m tall.
const S: f32 = 0.025;
const SPAWN_RADIUS: i32 = 2;

fn to_world(x: f32, y: f32) -> Vec3 {
    Vec3::new(x * S, 0.0, -y * S)
}

// ---------------------------------------------------------------- resources
#[derive(Resource)]
struct GameRes(Game);

#[derive(Resource, Default)]
struct PendingInput(Input);

#[derive(Resource, Default)]
struct ChunkEntities(HashMap<(i32, i32), Entity>);

#[derive(Resource, PartialEq, Eq, Clone, Copy)]
enum CamMode {
    Tilted,
    Side,
}

#[derive(Resource)]
struct Look {
    grass: [Handle<StandardMaterial>; 3],
    dirt: Handle<StandardMaterial>,
    road: Handle<StandardMaterial>,
    dash: Handle<StandardMaterial>,
    rock_raw: Handle<StandardMaterial>,
    rock_visited: Handle<StandardMaterial>,
    rocks: Vec<Handle<Mesh>>,
    ground: Handle<Mesh>,
    tree: Handle<WorldAsset>,
}

// --------------------------------------------------------------- components
#[derive(Component)]
struct Sun;
#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct PlayerRig;
#[derive(Component)]
struct TruckRig;
#[derive(Component)]
struct Campfire;
#[derive(Component)]
struct OutcropViz(String);
#[derive(Component)]
struct HudText;
#[derive(Component)]
struct HintText;
#[derive(Component)]
struct BannerText;
#[derive(Component)]
struct LogText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // static hosts have no .meta files; don't request them
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "Outcrops Odyssey (Bevy prototype)".into(),
                resolution: WindowResolution::new(1280, 720),
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(GameRes(Game::new(0x5eed_1234)))
        .init_resource::<PendingInput>()
        .init_resource::<ChunkEntities>()
        .insert_resource(CamMode::Tilted)
        .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.9)))
        .add_systems(Startup, (setup_scene, setup_camp, setup_hud).chain())
        .add_systems(FixedUpdate, tick_game)
        .add_systems(
            Update,
            (
                gather_input,
                sync_chunks,
                sync_outcrops,
                sync_actors,
                follow_camera,
                update_environment,
                update_hud,
                dev_hooks,
            )
                .chain(),
        )
        .run();
}

// ======================================================================= assets
fn repeat_sampler() -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    })
}

/// Load a texture with repeat wrapping; colour maps are sRGB, normal/roughness maps are linear.
fn tex(assets: &AssetServer, path: &'static str, srgb: bool) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(move |s: &mut ImageLoaderSettings| {
            s.is_srgb = srgb;
            s.sampler = repeat_sampler();
        })
        .load(path)
}

fn pbr(
    assets: &AssetServer,
    name: &str,
    tint: Color,
    uv_scale: f32,
    emissive: LinearRgba,
) -> StandardMaterial {
    let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
    StandardMaterial {
        base_color: tint,
        base_color_texture: Some(tex(assets, leak(format!("textures/{name}_albedo.jpg")), true)),
        normal_map_texture: Some(tex(assets, leak(format!("textures/{name}_normal.png")), false)),
        metallic_roughness_texture: Some(tex(assets, leak(format!("textures/{name}_mr.jpg")), false)),
        perceptual_roughness: 1.0, // the roughness map (green channel) does the work
        metallic: 0.0,
        emissive,
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(uv_scale)),
        ..default()
    }
}

fn plain(color: Color, rough: f32) -> StandardMaterial {
    StandardMaterial { base_color: color, perceptual_roughness: rough, metallic: 0.0, ..default() }
}

/// A displaced icosphere: cheap procedural boulder. Smooth low-frequency noise keeps it chunky.
fn make_rock(seed: f32) -> Mesh {
    let mut mesh = Sphere::new(1.0).mesh().ico(3).expect("ico sphere");
    if let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        for p in pos.iter_mut() {
            let n = (p[0] * 2.3 + seed).sin() * (p[1] * 2.9 + seed * 1.7).sin() * 0.5
                + (p[2] * 3.7 + seed * 0.6).sin() * (p[0] * 1.9 - seed).cos() * 0.35
                + (p[1] * 6.1 + p[2] * 5.3 + seed).sin() * 0.12;
            let r = 1.0 + 0.28 * n;
            *p = [p[0] * r * 1.1, p[1] * r * 0.72, p[2] * r * 1.0];
        }
    }
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();
    mesh
}

fn setup_scene(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let grass_tints = [Color::srgb(0.85, 0.95, 0.75), Color::srgb(0.95, 0.95, 0.8), Color::srgb(0.78, 0.9, 0.7)];
    let look = Look {
        grass: grass_tints.map(|t| mats.add(pbr(&assets, "grass", t, 4.0, LinearRgba::BLACK))),
        dirt: mats.add(plain(Color::srgb(0.30, 0.22, 0.14), 1.0)),
        road: mats.add(plain(Color::srgb(0.13, 0.13, 0.145), 0.85)),
        dash: mats.add(plain(Color::srgb(0.92, 0.78, 0.25), 0.7)),
        rock_raw: mats.add(pbr(&assets, "rock", Color::WHITE, 1.5, LinearRgba::new(0.10, 0.075, 0.01, 1.0))),
        rock_visited: mats.add(pbr(&assets, "moss", Color::srgb(0.75, 0.9, 0.7), 1.5, LinearRgba::BLACK)),
        rocks: (0..3)
            .map(|i| meshes.add(make_rock(i as f32 * 2.7 + 0.5)))
            .collect(),
        ground: meshes.add(Plane3d::default().mesh().size(CHUNK as f32 * S, CHUNK as f32 * S)),
        // glTF pipeline: swap tree.glb for any Blender export and nothing else changes
        tree: assets.load("models/tree.glb#Scene0"),
    };
    commands.insert_resource(look);

    // ---- sun (updated every frame by update_environment) + soft ambient light
    commands.spawn((
        Sun,
        DirectionalLight { illuminance: 11_000.0, shadow_maps_enabled: true, ..default() },
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            first_cascade_far_bound: 10.0,
            maximum_distance: 70.0,
            ..default()
        }
        .build(),
        Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::new(-0.4, 0.0, -0.3), Vec3::Y),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.75, 0.82, 1.0),
        brightness: 450.0,
        ..default()
    });

    // ---- camera
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 11.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        DistanceFog {
            color: Color::srgb(0.55, 0.72, 0.9),
            falloff: bevy::pbr::FogFalloff::Linear { start: 28.0, end: 80.0 },
            ..default()
        },
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
    ));

    // ---- player: a small articulated-looking geologist built from primitives
    let skin = mats.add(plain(Color::srgb(0.87, 0.68, 0.55), 0.8));
    let vest = mats.add(plain(Color::srgb(0.55, 0.38, 0.22), 0.9));
    let hat = mats.add(plain(Color::srgb(0.95, 0.85, 0.2), 0.5));
    commands
        .spawn((PlayerRig, Transform::default(), Visibility::default()))
        .with_children(|p| {
            p.spawn((Mesh3d(meshes.add(Capsule3d::new(0.32, 0.85))), MeshMaterial3d(vest), Transform::from_xyz(0.0, 0.85, 0.0)));
            p.spawn((Mesh3d(meshes.add(Sphere::new(0.27))), MeshMaterial3d(skin), Transform::from_xyz(0.0, 1.62, 0.0)));
            p.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.32, 0.12))),
                MeshMaterial3d(hat),
                Transform::from_xyz(0.0, 1.86, 0.0),
            ));
        });

    // ---- truck (red cab, green box, 4 wheels); local +X is "forward"
    let red = mats.add(plain(Color::srgb(0.75, 0.16, 0.12), 0.45));
    let green = mats.add(plain(Color::srgb(0.27, 0.42, 0.22), 0.7));
    let glass = mats.add(StandardMaterial { base_color: Color::srgb(0.55, 0.75, 0.9), perceptual_roughness: 0.1, metallic: 0.2, ..default() });
    let rubber = mats.add(plain(Color::srgb(0.05, 0.05, 0.06), 0.95));
    commands
        .spawn((TruckRig, Transform::default(), Visibility::default()))
        .with_children(|t| {
            t.spawn((Mesh3d(meshes.add(Cuboid::new(2.1, 1.35, 1.55))), MeshMaterial3d(green), Transform::from_xyz(-0.55, 1.05, 0.0)));
            t.spawn((Mesh3d(meshes.add(Cuboid::new(1.1, 1.05, 1.5))), MeshMaterial3d(red), Transform::from_xyz(0.95, 0.85, 0.0)));
            t.spawn((Mesh3d(meshes.add(Cuboid::new(0.08, 0.5, 1.2))), MeshMaterial3d(glass), Transform::from_xyz(1.52, 1.02, 0.0)));
            for (x, z) in [(-1.0, 0.82), (-1.0, -0.82), (0.95, 0.82), (0.95, -0.82)] {
                t.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.36, 0.22))),
                    MeshMaterial3d(rubber.clone()),
                    Transform::from_xyz(x, 0.36, z).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ));
            }
        });
}

/// The basecamp is a fixed landmark: dirt yard, three cabins, a tent, crates and a campfire.
fn setup_camp(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let yard = mats.add(plain(Color::srgb(0.36, 0.28, 0.19), 1.0));
    let wall = mats.add(plain(Color::srgb(0.45, 0.30, 0.18), 0.9));
    let roof = mats.add(plain(Color::srgb(0.58, 0.22, 0.17), 0.8));
    let canvas = mats.add(plain(Color::srgb(0.85, 0.70, 0.28), 0.9));
    let crate_m = mats.add(plain(Color::srgb(0.55, 0.40, 0.22), 0.95));
    let log_m = mats.add(plain(Color::srgb(0.18, 0.11, 0.07), 1.0));
    let ember = mats.add(StandardMaterial { base_color: Color::srgb(1.0, 0.45, 0.1), emissive: LinearRgba::new(6.0, 2.0, 0.3, 1.0), ..default() });

    let (x0, y0, x1, y1) = CAMP;
    let center = to_world((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size((x1 - x0) * S, (y1 - y0) * S))),
        MeshMaterial3d(yard),
        Transform::from_translation(center + Vec3::Y * 0.02),
    ));

    for (cx, cy) in [(-240.0, 400.0), (-60.0, 415.0), (260.0, 410.0)] {
        let p = to_world(cx, cy);
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(3.0, 1.9, 1.7))), MeshMaterial3d(wall.clone()), Transform::from_translation(p + Vec3::Y * 0.95)));
        for sign in [-1.0f32, 1.0] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(3.4, 0.14, 1.15))),
                MeshMaterial3d(roof.clone()),
                Transform::from_translation(p + Vec3::new(0.0, 2.15, sign * 0.52))
                    .with_rotation(Quat::from_rotation_x(sign * 0.55)),
            ));
        }
    }
    // tent: a 4-sided pyramid
    commands.spawn((
        Mesh3d(meshes.add(Cone { radius: 1.3, height: 1.7 }.mesh().resolution(4))),
        MeshMaterial3d(canvas),
        Transform::from_translation(to_world(340.0, 285.0) + Vec3::Y * 0.85).with_rotation(Quat::from_rotation_y(0.78)),
    ));
    for (cx, cy, s) in [(255.0, 205.0, 0.7), (285.0, 210.0, 0.55), (265.0, 238.0, 0.6)] {
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(s, s, s))), MeshMaterial3d(crate_m.clone()), Transform::from_translation(to_world(cx, cy) + Vec3::Y * s / 2.0)));
    }
    // campfire: logs + embers + a flickering light
    let fire = to_world(-30.0, 240.0);
    for a in [0.0f32, 1.05, 2.1] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.07, 0.8))),
            MeshMaterial3d(log_m.clone()),
            Transform::from_translation(fire + Vec3::Y * 0.1).with_rotation(Quat::from_rotation_y(a) * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ));
    }
    commands.spawn((Mesh3d(meshes.add(Sphere::new(0.18))), MeshMaterial3d(ember), Transform::from_translation(fire + Vec3::Y * 0.2)));
    commands.spawn((
        Campfire,
        PointLight { color: Color::srgb(1.0, 0.55, 0.2), intensity: 60_000.0, range: 14.0, shadow_maps_enabled: false, ..default() },
        Transform::from_translation(fire + Vec3::Y * 0.8),
    ));
}

// ============================================================================ HUD
fn setup_hud(mut commands: Commands) {
    let text = |size: f32, color: Color| (Text::new(""), TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }, TextColor(color));
    commands.spawn((
        HudText,
        text(22.0, Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(14.0), ..default() },
    ));
    commands.spawn((
        HintText,
        text(24.0, Color::srgb(1.0, 0.97, 0.8)),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(26.0), left: Val::Percent(30.0), ..default() },
    ));
    commands.spawn((
        BannerText,
        text(30.0, Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(70.0), left: Val::Percent(22.0), ..default() },
    ));
    commands.spawn((
        LogText,
        text(18.0, Color::srgb(0.9, 0.95, 0.85)),
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), right: Val::Px(14.0), ..default() },
    ));
}

// ========================================================================== INPUT
fn gather_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut pend: ResMut<PendingInput>,
    mut mode: ResMut<CamMode>,
) {
    let p = &mut pend.0;
    p.map_pressed |= keys.just_pressed(KeyCode::KeyM);
    p.cards_pressed |= keys.just_pressed(KeyCode::KeyC);
    p.space_pressed |= keys.just_pressed(KeyCode::Space);
    p.interact_pressed |= keys.just_pressed(KeyCode::KeyE);
    p.restart_pressed |= keys.just_pressed(KeyCode::Enter);
    p.page_down_pressed |= keys.just_pressed(KeyCode::ArrowDown);
    p.page_up_pressed |= keys.just_pressed(KeyCode::ArrowUp);
    if keys.just_pressed(KeyCode::KeyV) {
        *mode = if *mode == CamMode::Tilted { CamMode::Side } else { CamMode::Tilted };
    }

    // click / tap -> ray from the camera -> ground plane -> game coordinates
    let screen_pos = if mouse.just_pressed(MouseButton::Left) {
        windows.single().ok().and_then(|w| w.cursor_position())
    } else {
        touches.iter_just_pressed().next().map(|t| t.position())
    };
    if let (Some(pos), Ok((cam, gt))) = (screen_pos, camera.single()) {
        if let Ok(ray) = cam.viewport_to_world(gt, pos) {
            if let Some(d) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) {
                let hit = ray.get_point(d);
                p.world_click = Some((hit.x / S, -hit.z / S));
            }
        }
    }
}

fn tick_game(keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<GameRes>, mut pend: ResMut<PendingInput>) {
    let mut input = std::mem::take(&mut pend.0);
    input.left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    input.right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);
    input.up = keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW);
    input.down = keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS);
    game.0.tick(&input);
}

// ===================================================================== WORLD SYNC
fn hash01(s: &str) -> f32 {
    let h = s.bytes().fold(7u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
    (h % 10_000) as f32 / 10_000.0
}

fn sync_chunks(mut commands: Commands, game: Res<GameRes>, look: Res<Look>, mut live: ResMut<ChunkEntities>, mut meshes: ResMut<Assets<Mesh>>) {
    let (fx, fy) = game.0.focus();
    let (pcx, pcy) = world_to_chunk(fx, fy);

    // despawn chunks that fell out of range
    let stale: Vec<_> = live.0.keys().copied().filter(|(x, y)| (x - pcx).abs() > SPAWN_RADIUS + 1 || (y - pcy).abs() > SPAWN_RADIUS + 1).collect();
    for k in stale {
        if let Some(e) = live.0.remove(&k) {
            commands.entity(e).despawn();
        }
    }

    for dx in -SPAWN_RADIUS..=SPAWN_RADIUS {
        for dy in -SPAWN_RADIUS..=SPAWN_RADIUS {
            let key = (pcx + dx, pcy + dy);
            if live.0.contains_key(&key) {
                continue;
            }
            let Some(chunk) = game.0.chunks.get(&key) else { continue };
            let (cx, cy) = key;
            let center = ((cx * CHUNK) as f32 + CHUNK as f32 / 2.0, (cy * CHUNK) as f32 + CHUNK as f32 / 2.0);
            let local = |x: f32, y: f32| to_world(x - center.0, y - center.1);

            let road_mat = look.road.clone();
            let dash_mat = look.dash.clone();
            let entity = commands
                .spawn((Transform::from_translation(to_world(center.0, center.1)), Visibility::default()))
                .with_children(|c| {
                    // ground
                    c.spawn((Mesh3d(look.ground.clone()), MeshMaterial3d(look.grass[chunk.grass_variant as usize % 3].clone())));

                    // dirt patches
                    for d in &chunk.dirt_patches {
                        c.spawn((
                            Mesh3d(meshes.add(Circle::new(2.0))),
                            MeshMaterial3d(look.dirt.clone()),
                            Transform::from_translation(local(d.x, d.y) + Vec3::Y * 0.012)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                                .with_scale(Vec3::new(1.2, 0.85, 1.0)),
                        ));
                    }

                    // trees: the glTF scene, varied by a stable per-position hash
                    for (i, t) in chunk.trees.iter().enumerate() {
                        let h = hash01(&format!("{cx}_{cy}_t{i}"));
                        c.spawn((
                            WorldAssetRoot(look.tree.clone()),
                            Transform::from_translation(local(t.x, t.y))
                                .with_rotation(Quat::from_rotation_y(h * 6.28))
                                .with_scale(Vec3::splat(0.85 + h * 0.5)),
                        ));
                    }

                    // outcrops
                    for o in &chunk.outcrops {
                        let h = hash01(&o.id);
                        let s = 0.75 + h * 0.5;
                        c.spawn((
                            OutcropViz(o.id.clone()),
                            Mesh3d(look.rocks[(h * 3.0) as usize % 3].clone()),
                            MeshMaterial3d(look.rock_raw.clone()),
                            Transform::from_translation(local(o.x, o.y) + Vec3::Y * 0.28 * s)
                                .with_rotation(Quat::from_rotation_y(h * 6.28))
                                .with_scale(Vec3::new(s * 1.1, s, s)),
                        ));
                    }

                    // main roads (each owned by the chunk containing its centre line)
                    let yh = nearest_h_road(center.1);
                    if ((yh / CHUNK as f32).floor() as i32) == cy {
                        let at = local(center.0, yh);
                        c.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(CHUNK as f32 * S, ROAD_W * S))), MeshMaterial3d(road_mat.clone()), Transform::from_translation(at + Vec3::Y * 0.02)));
                        for i in 0..8 {
                            let x = (cx * CHUNK) as f32 + i as f32 * 80.0 + 22.0;
                            c.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(44.0 * S, 6.0 * S))), MeshMaterial3d(dash_mat.clone()), Transform::from_translation(local(x, yh) + Vec3::Y * 0.03)));
                        }
                    }
                    let xv = nearest_v_road(center.0);
                    if ((xv / CHUNK as f32).floor() as i32) == cx {
                        let at = local(xv, center.1);
                        c.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(ROAD_W * S, CHUNK as f32 * S))), MeshMaterial3d(road_mat.clone()), Transform::from_translation(at + Vec3::Y * 0.025)));
                        for i in 0..8 {
                            let y = (cy * CHUNK) as f32 + i as f32 * 80.0 + 22.0;
                            c.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(6.0 * S, 44.0 * S))), MeshMaterial3d(dash_mat.clone()), Transform::from_translation(local(xv, y) + Vec3::Y * 0.035)));
                        }
                    }
                })
                .id();
            live.0.insert(key, entity);
        }
    }
}

/// Raw outcrops glow faintly and use bare rock; visited ones turn mossy.
fn sync_outcrops(game: Res<GameRes>, look: Res<Look>, mut q: Query<(&OutcropViz, &mut MeshMaterial3d<StandardMaterial>)>) {
    for (viz, mut m) in &mut q {
        let want = if game.0.visited.contains(&viz.0) { &look.rock_visited } else { &look.rock_raw };
        if m.0 != *want {
            m.0 = want.clone();
        }
    }
}

// ====================================================================== ACTORS
fn sync_actors(
    time: Res<Time>,
    game: Res<GameRes>,
    mut player: Query<&mut Transform, (With<PlayerRig>, Without<TruckRig>)>,
    mut truck: Query<&mut Transform, (With<TruckRig>, Without<PlayerRig>)>,
    mut vis: Query<&mut Visibility, With<PlayerRig>>,
    mut last: Local<Option<Vec3>>,
    mut fire: Query<&mut PointLight, With<Campfire>>,
) {
    let g = &game.0;
    if let Ok(mut t) = player.single_mut() {
        let pos = to_world(g.player.x, g.player.y);
        let moved = last.map(|l| pos - l).unwrap_or(Vec3::ZERO);
        let walking = moved.length_squared() > 1e-8;
        if walking {
            let target = Quat::from_rotation_y(moved.x.atan2(moved.z));
            t.rotation = t.rotation.slerp(target, 0.25);
        }
        let bob = if walking { (time.elapsed_secs() * 12.0).sin().abs() * 0.08 } else { 0.0 };
        t.translation = pos + Vec3::Y * bob;
        *last = Some(pos);
    }
    if let Ok(mut v) = vis.single_mut() {
        *v = if g.player_visible() { Visibility::Inherited } else { Visibility::Hidden };
    }
    if let Ok(mut t) = truck.single_mut() {
        t.translation = to_world(g.truck.x, g.truck.y);
        // local +X -> heading (dx, 0, -dy)  =>  yaw = atan2(dy, dx)
        t.rotation = Quat::from_rotation_y(g.truck.dy.atan2(g.truck.dx));
    }
    if let Ok(mut l) = fire.single_mut() {
        l.intensity = 55_000.0 + (time.elapsed_secs() * 17.0).sin() * 9_000.0 + (time.elapsed_secs() * 7.3).sin() * 6_000.0;
    }
}

fn follow_camera(
    time: Res<Time>,
    game: Res<GameRes>,
    mode: Res<CamMode>,
    mut cam: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut t) = cam.single_mut() else { return };
    let (fx, fy) = game.0.focus();
    let target = to_world(fx, fy) + Vec3::Y * 1.0;
    // Tilted = angled top-down-ish (the Macroquad view, in 3D); Side = low, side-on "2.5D" view.
    let offset = match *mode {
        CamMode::Tilted => Vec3::new(0.0, 13.0, 10.5),
        CamMode::Side => Vec3::new(0.0, 3.2, 15.0),
    };
    let desired = target + offset;
    let k = 1.0 - (-7.0 * time.delta_secs()).exp();
    t.translation = t.translation.lerp(desired, k);
    let look_at = target;
    *t = t.looking_at(look_at, Vec3::Y);
}

// ================================================================== ENVIRONMENT
fn lerp_color(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn update_environment(
    game: Res<GameRes>,
    mut sun: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut fog: Query<&mut DistanceFog>,
    mut clear: ResMut<ClearColor>,
) {
    let g = &game.0;
    let night = g.night_amount();
    // 05:00 -> 18:00 maps to 0..1; the sun rises in the east (+x), climbs, and sets in the west
    let m = g.minutes_of_day() as f32;
    let t = ((m - 5.0 * 60.0) / (13.0 * 60.0)).clamp(0.0, 1.0);
    let elevation = (8.0 + 62.0 * (t * std::f32::consts::PI).sin()).to_radians();
    let azimuth = (80.0 - 160.0 * t).to_radians();
    let to_sun = Vec3::new(azimuth.sin() * elevation.cos(), elevation.sin(), -azimuth.cos() * elevation.cos());

    let low = 1.0 - (elevation.sin()).clamp(0.0, 1.0); // 1 near the horizon
    let sun_color = lerp_color(Vec3::new(1.0, 0.97, 0.9), Vec3::new(1.0, 0.62, 0.35), low * 0.9);
    let day_sky = lerp_color(Vec3::new(0.55, 0.74, 0.95), Vec3::new(0.95, 0.60, 0.40), low * 0.8);
    let night_sky = Vec3::new(0.025, 0.035, 0.09);
    let sky = lerp_color(day_sky, night_sky, night / 0.85);

    if let Ok((mut tf, mut light)) = sun.single_mut() {
        *tf = Transform::from_translation(to_sun * 50.0).looking_at(Vec3::ZERO, Vec3::Y);
        // the light fades less than the sky does, so dawn/dusk stay readable
        let dark = (night / 0.85).clamp(0.0, 1.0);
        light.illuminance = 13_000.0 * (1.0 - dark * 0.9) * (0.55 + 0.45 * elevation.sin());
        light.color = Color::srgb(sun_color.x, sun_color.y, sun_color.z);
    }
    ambient.brightness = 120.0 + 480.0 * (1.0 - (night / 0.85).clamp(0.0, 1.0));
    ambient.color = {
        let c = lerp_color(Vec3::new(0.75, 0.83, 1.0), Vec3::new(0.35, 0.42, 0.8), night / 0.85);
        Color::srgb(c.x, c.y, c.z)
    };
    let sky_c = Color::srgb(sky.x, sky.y, sky.z);
    clear.0 = sky_c;
    for mut f in &mut fog {
        f.color = sky_c;
    }
}

// ========================================================================== HUD
fn update_hud(
    game: Res<GameRes>,
    mode: Res<CamMode>,
    mut hud: Query<&mut Text, (With<HudText>, Without<HintText>, Without<BannerText>, Without<LogText>)>,
    mut hint: Query<&mut Text, (With<HintText>, Without<HudText>, Without<BannerText>, Without<LogText>)>,
    mut banner: Query<&mut Text, (With<BannerText>, Without<HudText>, Without<HintText>, Without<LogText>)>,
    mut log: Query<&mut Text, (With<LogText>, Without<HudText>, Without<HintText>, Without<BannerText>)>,
) {
    let g = &game.0;
    let m = g.minutes_of_day();
    if let Ok(mut t) = hud.single_mut() {
        t.0 = format!(
            "Outcrop Points: {}\nXP: {}   Bag: {}/{}\nEnergy: {:.0}%\nDay {}   {:02}:{:02}   (sunset 18:00)\n[V] camera: {}",
            g.outcrop_points,
            g.xp,
            g.inventory_count,
            INVENTORY_CAP,
            g.energy,
            g.day,
            m / 60,
            m % 60,
            if *mode == CamMode::Tilted { "tilted" } else { "side" },
        );
    }
    if let Ok(mut t) = hint.single_mut() {
        t.0 = match g.phase {
            Phase::Basecamp if g.near_truck() => "Press E to board the truck".into(),
            Phase::Basecamp => "Walk to the truck on the road below camp (WASD / click)".into(),
            Phase::Exploring if g.player_on_road() => "Press E to call the truck back to basecamp".into(),
            Phase::Exploring if g.nearest_outcrop().is_some() => "SPACE (or click the rock) to break it".into(),
            Phase::Exploring if g.minutes_to_sunset() <= 120 => format!("Sunset in {} min - reach a main road!", g.minutes_to_sunset()),
            Phase::GameOver => "GAME OVER - night fell off the road. Press ENTER to start over".into(),
            Phase::Sleeping => format!("Day {} complete - resting...", g.day),
            _ => String::new(),
        };
    }
    if let Ok(mut t) = banner.single_mut() {
        t.0 = g.banner.as_ref().map(|(s, _)| s.clone()).unwrap_or_default();
    }
    if let Ok(mut t) = log.single_mut() {
        t.0 = g.popups.iter().rev().take(4).map(|p| p.text.clone()).collect::<Vec<_>>().join("\n");
    }
}

// ================================================================ DEV HOOKS
// OO_DEMO=camp|field|noon|sunset|night|road  scripts a scene;  OO_SHOT=/path.png [OO_SHOT_FRAMES=n]
// saves a screenshot after n frames and exits. Handy for headless visual checks; harmless otherwise.
fn dev_hooks(mut commands: Commands, mut game: ResMut<GameRes>, mut frame: Local<u32>) {
    use bevy::render::view::screenshot::{save_to_disk, Screenshot};
    *frame += 1;
    let g = &mut game.0;
    if *frame == 3 {
        if let Ok(scene) = std::env::var("OO_DEMO") {
            if scene != "camp" {
                g.player.x = TRUCK_SPOT.0;
                g.player.y = TRUCK_SPOT.1;
                g.tick(&Input { interact_pressed: true, ..Default::default() });
                while g.phase == Phase::TruckOut {
                    g.tick(&Input::default());
                }
                g.player.x += 380.0;
                g.player.y += 60.0;
                for _ in 0..8 {
                    g.ensure_nearby_chunks_generated();
                    let t = g.chunks.values().flat_map(|c| c.outcrops.iter()).find(|o| !g.visited.contains(&o.id)).cloned();
                    if let Some(o) = t {
                        let (px, py) = (g.player.x, g.player.y);
                        g.player.x = o.x;
                        g.player.y = o.y;
                        g.break_nearest_outcrop();
                        g.player.x = px;
                        g.player.y = py;
                    }
                }
            }
            match scene.as_str() {
                "noon" => g.clock_ticks = 7 * 60 * TICKS_PER_MINUTE,
                "sunset" => g.clock_ticks = (12 * 60 + 35) * TICKS_PER_MINUTE,
                "night" => g.phase = Phase::Sleeping,
                "road" => g.player.x -= 400.0,
                _ => {}
            }
        }
    }
    if let Ok(path) = std::env::var("OO_SHOT") {
        let n: u32 = std::env::var("OO_SHOT_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(120);
        if *frame == n {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        }
        if *frame == n + 40 {
            std::process::exit(0);
        }
    }
}
