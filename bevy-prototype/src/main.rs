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
#[path = "../../src/terrain.rs"]
#[allow(dead_code)]
mod terrain;
#[path = "../../src/geology.rs"]
#[allow(dead_code)]
mod geology;
#[path = "../../src/kit.rs"]
#[allow(dead_code)]
mod kit;

mod terrain_view;
mod kit_ui;
mod scenery;

use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::light::{CascadeShadowConfigBuilder, GlobalAmbientLight};
use bevy::mesh::VertexAttributeValues;
use bevy::pbr::DistanceFog;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use game::*;
use scenery::{PlayerRig, Surfaces, TruckRig, TruckSpeed, WalkAmount};
use std::collections::HashMap;
use terrain_view::{ground_y, S};
use world::*;

const SPAWN_RADIUS: i32 = 2;
/// Chunk meshes built per frame (see `sync_chunks`).
const CHUNKS_PER_FRAME: usize = 2;

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

/// Farm plots are bigger than chunks, so they are streamed on their own grid (value None = no farm there).
#[derive(Resource, Default)]
struct FarmEntities(HashMap<(i32, i32), Option<Entity>>);

/// The camp is rebuilt when the map (region) changes, because the relief under it does.
#[derive(Resource)]
struct CampState {
    entity: Entity,
    level: usize,
}

#[derive(Resource, PartialEq, Eq, Clone, Copy)]
enum CamMode {
    Tilted,
    Side,
}

#[derive(Resource)]
struct Look {
    grass: Handle<StandardMaterial>,
    cliff: Handle<StandardMaterial>,
    water: Handle<StandardMaterial>,
    edge: Handle<StandardMaterial>,
    road: Handle<StandardMaterial>,
    dash: Handle<StandardMaterial>,
    contour: Handle<StandardMaterial>,
    crop: Handle<StandardMaterial>,
    fence: Handle<StandardMaterial>,
    surf: Surfaces,
    rock_raw: Handle<StandardMaterial>,
    rock_visited: Handle<StandardMaterial>,
    rocks: Vec<Handle<Mesh>>,
    tree: Handle<WorldAsset>,
}

// --------------------------------------------------------------- components
#[derive(Component)]
struct Sun;
#[derive(Component)]
struct MainCamera;
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
        .insert_resource(bevy::light::DirectionalLightShadowMap { size: if cfg!(target_arch = "wasm32") { 1024 } else { 2048 } })
        .insert_resource(GameRes(Game::new(0x5eed_1234)))
        .init_resource::<PendingInput>()
        .init_resource::<ChunkEntities>()
        .init_resource::<FarmEntities>()
        .init_resource::<WalkAmount>()
        .init_resource::<TruckSpeed>()
        .insert_resource(CamMode::Tilted)
        .insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.9)))
        .add_systems(Startup, (setup_scene, setup_camp, setup_hud, kit_ui::setup_kit_ui).chain())
        .add_systems(FixedUpdate, tick_game)
        .add_systems(
            Update,
            (
                kit_ui::button_input,
                gather_input,
                sync_level,
                sync_chunks,
                sync_outcrops,
                sync_actors,
                scenery::animate_geologist,
                scenery::spin_wheels,
                follow_camera,
                update_environment,
                update_hud,
                kit_ui::update_kit_ui,
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
    let surf = Surfaces::load(&assets, &mut mats);
    let look = Look {
        // terrain UVs are in metres, so uv_scale 0.25 repeats the texture every 4 m, seamlessly across chunks
        grass: surf.grass.clone(),
        cliff: surf.strata.clone(),
        water: mats.add(StandardMaterial {
            base_color: Color::srgba(0.10, 0.34, 0.55, 0.78),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.06,
            reflectance: 0.6,
            ..default()
        }),
        edge: mats.add(plain(Color::srgb(0.02, 0.025, 0.02), 1.0)),
        road: surf.road.clone(),
        dash: surf.dash.clone(),
        contour: mats.add(StandardMaterial { base_color: Color::WHITE, unlit: true, ..default() }),
        crop: surf.crop.clone(),
        fence: surf.fence.clone(),
        rock_raw: mats.add(pbr(&assets, "rock", Color::WHITE, 1.5, LinearRgba::new(0.10, 0.075, 0.01, 1.0))),
        rock_visited: mats.add(pbr(&assets, "moss", Color::srgb(0.75, 0.9, 0.7), 1.5, LinearRgba::BLACK)),
        rocks: (0..3).map(|i| meshes.add(make_rock(i as f32 * 2.7 + 0.5))).collect(),
        // glTF pipeline: swap tree.glb for any Blender export and nothing else changes
        tree: assets.load("models/tree.glb#Scene0"),
        surf,
    };
    // ---- sun (updated every frame by update_environment) + soft ambient light
    commands.spawn((
        Sun,
        DirectionalLight { illuminance: 11_000.0, shadow_maps_enabled: true, ..default() },
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            first_cascade_far_bound: 10.0,
            maximum_distance: 48.0,
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
        // phones: skip multisampling, the textured ground hides most aliasing
        #[cfg(target_arch = "wasm32")]
        bevy::render::view::Msaa::Off,
        Transform::from_xyz(0.0, 11.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        DistanceFog {
            color: Color::srgb(0.55, 0.72, 0.9),
            falloff: bevy::pbr::FogFalloff::Linear { start: 28.0, end: 80.0 },
            ..default()
        },
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
    ));

    // ---- the geologist and the vintage pickup (see scenery.rs)
    scenery::spawn_geologist(&mut commands, &mut meshes, &look.surf);
    scenery::spawn_truck(&mut commands, &mut meshes, &look.surf);
    commands.insert_resource(look);
}

/// The basecamp is a fixed landmark: dirt yard, three cabins, a tent, crates and a campfire,
/// all standing on whatever relief the current map has under it.
fn setup_camp(mut commands: Commands, game: Res<GameRes>, look: Res<Look>, mut meshes: ResMut<Assets<Mesh>>) {
    let entity = spawn_camp(&mut commands, &game.0.terrain, &mut meshes, &look.surf);
    commands.insert_resource(CampState { entity, level: game.0.level });
}

fn spawn_camp(commands: &mut Commands, t: &terrain::Terrain, meshes: &mut Assets<Mesh>, m: &Surfaces) -> Entity {
    let (yard, wall, roof, canvas, crate_m, log_m, ember) = (m.yard.clone(), m.wall.clone(), m.roof.clone(), m.canvas.clone(), m.crate_wood.clone(), m.log.clone(), m.ember.clone());

    // a point on the ground at game position (x, y)
    let at = |x: f32, y: f32| terrain_view::surface(t, x, y);
    let root = commands.spawn((Transform::default(), Visibility::default())).id();
    let mut c = commands.entity(root);
    c.with_children(|c| {
        c.spawn((
            Mesh3d(meshes.add(terrain_view::drape_rect(t, CAMP, 0.03, [1.0; 4]).build())),
            MeshMaterial3d(yard),
        ));

        for (cx, cy) in [(-240.0, 400.0), (-60.0, 415.0), (260.0, 410.0)] {
            let p = at(cx, cy);
            c.spawn((Mesh3d(scenery::tex_box(meshes, 3.0, 1.9, 1.7, 1.2)), MeshMaterial3d(wall.clone()), Transform::from_translation(p + Vec3::Y * 0.8)));
            for sign in [-1.0f32, 1.0] {
                c.spawn((
                    Mesh3d(scenery::tex_box(meshes, 3.4, 0.14, 1.15, 1.0)),
                    MeshMaterial3d(roof.clone()),
                    Transform::from_translation(p + Vec3::new(0.0, 2.0, sign * 0.52)).with_rotation(Quat::from_rotation_x(sign * 0.55)),
                ));
            }
        }
        // tent: a 4-sided pyramid
        c.spawn((
            Mesh3d(meshes.add(Cone { radius: 1.3, height: 1.7 }.mesh().resolution(4))),
            MeshMaterial3d(canvas),
            Transform::from_translation(at(340.0, 285.0) + Vec3::Y * 0.85).with_rotation(Quat::from_rotation_y(0.78)),
        ));
        for (cx, cy, s) in [(255.0, 205.0, 0.7), (285.0, 210.0, 0.55), (265.0, 238.0, 0.6)] {
            c.spawn((Mesh3d(scenery::tex_box(meshes, s, s, s, 0.7)), MeshMaterial3d(crate_m.clone()), Transform::from_translation(at(cx, cy) + Vec3::Y * s / 2.0)));
        }
        // campfire: logs + embers + a flickering light
        let fire = at(-30.0, 240.0);
        for a in [0.0f32, 1.05, 2.1] {
            c.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.07, 0.8))),
                MeshMaterial3d(log_m.clone()),
                Transform::from_translation(fire + Vec3::Y * 0.1).with_rotation(Quat::from_rotation_y(a) * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ));
        }
        c.spawn((Mesh3d(meshes.add(Sphere::new(0.18))), MeshMaterial3d(ember), Transform::from_translation(fire + Vec3::Y * 0.2)));
        c.spawn((
            Campfire,
            PointLight { color: Color::srgb(1.0, 0.55, 0.2), intensity: 60_000.0, range: 14.0, shadow_maps_enabled: false, ..default() },
            Transform::from_translation(fire + Vec3::Y * 0.8),
        ));
    });
    root
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
    ui_buttons: Query<&Interaction, With<Button>>,
    game: Res<GameRes>,
    mut pend: ResMut<PendingInput>,
    mut mode: ResMut<CamMode>,
) {
    let p = &mut pend.0;
    p.map_pressed |= keys.just_pressed(KeyCode::KeyM);
    p.cards_pressed |= keys.just_pressed(KeyCode::KeyC);
    p.pack_pressed |= keys.just_pressed(KeyCode::KeyB);
    p.use_pressed |= keys.just_pressed(KeyCode::KeyF);
    p.space_pressed |= keys.just_pressed(KeyCode::Space);
    p.interact_pressed |= keys.just_pressed(KeyCode::KeyE);
    p.restart_pressed |= keys.just_pressed(KeyCode::Enter);
    p.page_down_pressed |= keys.just_pressed(KeyCode::ArrowDown);
    p.page_up_pressed |= keys.just_pressed(KeyCode::ArrowUp);
    if keys.just_pressed(KeyCode::KeyV) {
        *mode = if *mode == CamMode::Tilted { CamMode::Side } else { CamMode::Tilted };
    }

    // a press that lands on an on-screen button is the button's, not the world's
    if ui_buttons.iter().any(|i| *i != Interaction::None) {
        return;
    }
    let screen_pos = if mouse.just_pressed(MouseButton::Left) {
        windows.single().ok().and_then(|w| w.cursor_position())
    } else {
        touches.iter_just_pressed().next().map(|t| t.position())
    };
    let Some(pos) = screen_pos else { return };
    match game.0.ui_mode {
        // menus use the game's 1280x720 y-up virtual screen
        UiMode::Backpack => {
            if let Ok(w) = windows.single() {
                p.click = Some((pos.x / w.width() * 1280.0, 720.0 - pos.y / w.height() * 720.0));
            }
        }
        UiMode::None => {
            // click / tap -> ray from the camera -> terrain -> game coordinates
            if let Ok((cam, gt)) = camera.single() {
                if let Ok(ray) = cam.viewport_to_world(gt, pos) {
                    if let Some(hit) = terrain_view::raycast(&game.0.terrain, ray.origin, *ray.direction, 120.0) {
                        p.world_click = Some((hit.x / S, -hit.z / S));
                    }
                }
            }
        }
        _ => {}
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

/// When the player moves on to another map the relief changes: throw the old world away.
fn sync_level(
    mut commands: Commands,
    game: Res<GameRes>,
    mut live: ResMut<ChunkEntities>,
    mut farms: ResMut<FarmEntities>,
    mut camp: ResMut<CampState>,
    look: Res<Look>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let g = &game.0;
    if camp.level == g.level {
        return;
    }
    for (_, e) in live.0.drain() {
        commands.entity(e).despawn();
    }
    for (_, e) in farms.0.drain() {
        if let Some(e) = e {
            commands.entity(e).despawn();
        }
    }
    commands.entity(camp.entity).despawn();
    camp.entity = spawn_camp(&mut commands, &g.terrain, &mut meshes, &look.surf);
    camp.level = g.level;
}

fn sync_chunks(
    mut commands: Commands,
    game: Res<GameRes>,
    look: Res<Look>,
    mut live: ResMut<ChunkEntities>,
    mut farms: ResMut<FarmEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let g = &game.0;
    let t = &g.terrain;
    let (fx, fy) = g.focus();
    let (pcx, pcy) = world_to_chunk(fx, fy);

    // despawn chunks that fell out of range
    let stale: Vec<_> = live.0.keys().copied().filter(|(x, y)| (x - pcx).abs() > SPAWN_RADIUS + 1 || (y - pcy).abs() > SPAWN_RADIUS + 1).collect();
    for k in stale {
        if let Some(e) = live.0.remove(&k) {
            commands.entity(e).despawn();
        }
    }

    // building a chunk (mesh + trees + roads) is the heaviest thing the game does, so at most
    // CHUNKS_PER_FRAME are made per frame, nearest first: no hitch when walking into new ground
    let mut wanted: Vec<(i32, i32)> = (-SPAWN_RADIUS..=SPAWN_RADIUS)
        .flat_map(|dx| (-SPAWN_RADIUS..=SPAWN_RADIUS).map(move |dy| (pcx + dx, pcy + dy)))
        .filter(|k| !live.0.contains_key(k))
        .collect();
    wanted.sort_by_key(|(x, y)| (x - pcx).pow(2) + (y - pcy).pow(2));
    wanted.truncate(CHUNKS_PER_FRAME);
    {
        for key in wanted {
            let Some(chunk) = g.chunks.get(&key) else { continue };
            let (cx, cy) = key;
            let (ox, oy) = ((cx * CHUNK) as f32, (cy * CHUNK) as f32);
            let center = (ox + CHUNK as f32 / 2.0, oy + CHUNK as f32 / 2.0);
            // chunk-local position standing on the terrain
            let local = |x: f32, y: f32| Vec3::new((x - center.0) * S, ground_y(t, x, y), -(y - center.1) * S);

            let geo = terrain_view::build_chunk(t, cx, cy);
            let road_mat = look.road.clone();
            let dash_mat = look.dash.clone();
            let entity = commands
                .spawn((Transform::from_translation(to_world(center.0, center.1)), Visibility::default()))
                .with_children(|c| {
                    // ground
                    c.spawn((Mesh3d(meshes.add(geo.grass)), MeshMaterial3d(look.grass.clone())));
                    if let Some(m) = geo.cliff {
                        c.spawn((Mesh3d(meshes.add(m)), MeshMaterial3d(look.cliff.clone())));
                    }
                    if let Some(m) = geo.edge {
                        c.spawn((Mesh3d(meshes.add(m)), MeshMaterial3d(look.edge.clone())));
                    }
                    if let Some(m) = geo.contours {
                        c.spawn((Mesh3d(meshes.add(m)), MeshMaterial3d(look.contour.clone()), bevy::light::NotShadowCaster));
                    }
                    if let Some(m) = geo.water {
                        c.spawn((Mesh3d(meshes.add(m)), MeshMaterial3d(look.water.clone()), bevy::light::NotShadowCaster));
                    }

                    // trees: the glTF scene, varied by a stable per-position hash. Loose trees on
                    // open ground, plus the dense forest that fills the impassable forest cells.
                    let loose = chunk.trees.iter().map(|p| (p.x, p.y, 0.8f32));
                    let dense = geo.forest.iter().map(|&(x, y)| (x, y, 0.8f32));
                    for (i, (x, y, size)) in loose.chain(dense).enumerate() {
                        let h = hash01(&format!("{cx}_{cy}_t{i}"));
                        c.spawn((
                            WorldAssetRoot(look.tree.clone()),
                            Transform::from_translation(local(x, y))
                                .with_rotation(Quat::from_rotation_y(h * 6.28))
                                .with_scale(Vec3::splat((0.85 + h * 0.5) * size)),
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

                    // main roads: winding ribbons draped over the relief. A chunk draws the x (or y)
                    // slice of every road whose centre line passes through it at mid-chunk.
                    let (w, mid) = (CHUNK as f32, CHUNK as f32 / 2.0);
                    for k in h_index(oy - 700.0)..=h_index(oy + w + 700.0) {
                        let ym = h_road_y(k, ox + mid);
                        if ym < oy || ym >= oy + w {
                            continue;
                        }
                        let pos = move |x: f32| (x, h_road_y(k, x));
                        let road = terrain_view::drape_curve(t, &pos, ox, ox + w, 20.0, ROAD_W, 0.06, center, [1.0; 4]);
                        c.spawn((Mesh3d(meshes.add(road.build())), MeshMaterial3d(road_mat.clone())));
                        let dashes = (0..8)
                            .map(|i| {
                                let x = ox + i as f32 * 80.0 + 22.0;
                                terrain_view::drape_curve(t, &pos, x, x + 44.0, 22.0, 6.0, 0.09, center, [1.0; 4])
                            })
                            .collect();
                        c.spawn((Mesh3d(meshes.add(terrain_view::merge(dashes).build())), MeshMaterial3d(dash_mat.clone())));
                    }
                    for j in v_index(ox - 700.0)..=v_index(ox + w + 700.0) {
                        let xm = v_road_x(j, oy + mid);
                        if xm < ox || xm >= ox + w {
                            continue;
                        }
                        let pos = move |y: f32| (v_road_x(j, y), y);
                        let road = terrain_view::drape_curve(t, &pos, oy, oy + w, 20.0, ROAD_W, 0.065, center, [1.0; 4]);
                        c.spawn((Mesh3d(meshes.add(road.build())), MeshMaterial3d(road_mat.clone())));
                        let dashes = (0..8)
                            .map(|i| {
                                let y = oy + i as f32 * 80.0 + 22.0;
                                terrain_view::drape_curve(t, &pos, y, y + 44.0, 22.0, 6.0, 0.095, center, [1.0; 4])
                            })
                            .collect();
                        c.spawn((Mesh3d(meshes.add(terrain_view::merge(dashes).build())), MeshMaterial3d(dash_mat.clone())));
                    }
                })
                .id();
            live.0.insert(key, entity);
        }
    }

    // farm plots live on their own (larger) grid
    let (fcx, fcy) = ((fx / terrain::FARM_CELL).floor() as i32, (fy / terrain::FARM_CELL).floor() as i32);
    let stale: Vec<_> = farms.0.keys().copied().filter(|(a, b)| (a - fcx).abs() > 3 || (b - fcy).abs() > 3).collect();
    for k in stale {
        if let Some(Some(e)) = farms.0.remove(&k) {
            commands.entity(e).despawn();
        }
    }
    for ci in fcx - 2..=fcx + 2 {
        for cj in fcy - 2..=fcy + 2 {
            if farms.0.contains_key(&(ci, cj)) {
                continue;
            }
            let ent = t.farm_rect(ci, cj).map(|r| {
                let (crops, fence) = terrain_view::build_farm(t, r);
                commands
                    .spawn((Transform::default(), Visibility::default()))
                    .with_children(|f| {
                        f.spawn((Mesh3d(meshes.add(crops)), MeshMaterial3d(look.crop.clone()), bevy::light::NotShadowCaster));
                        f.spawn((Mesh3d(meshes.add(fence)), MeshMaterial3d(look.fence.clone())));
                    })
                    .id()
            });
            farms.0.insert((ci, cj), ent);
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
    mut walk: ResMut<WalkAmount>,
    mut speed: ResMut<TruckSpeed>,
    mut last_truck: Local<Option<Vec3>>,
) {
    let g = &game.0;
    if let Ok(mut t) = player.single_mut() {
        let pos = terrain_view::surface(&g.terrain, g.player.x, g.player.y);
        let moved = last.map(|l| pos - l).unwrap_or(Vec3::ZERO);
        let walking = moved.length_squared() > 1e-8;
        if walking {
            let target = Quat::from_rotation_y(moved.x.atan2(moved.z));
            t.rotation = t.rotation.slerp(target, 0.25);
        }
        let bob = if walking { (time.elapsed_secs() * 12.0).sin().abs() * 0.08 } else { 0.0 };
        t.translation = pos + Vec3::Y * bob;
        *last = Some(pos);
        let k = 1.0 - (-10.0 * time.delta_secs()).exp();
        walk.0 += ((if walking { 1.0 } else { 0.0 }) - walk.0) * k;
    }
    if let Ok(mut v) = vis.single_mut() {
        *v = if g.player_visible() { Visibility::Inherited } else { Visibility::Hidden };
    }
    if let Ok(mut t) = truck.single_mut() {
        let pos = terrain_view::surface(&g.terrain, g.truck.x, g.truck.y);
        let dt = time.delta_secs().max(1e-4);
        speed.0 = last_truck.map(|l| (pos - l).length() / dt).unwrap_or(0.0).min(40.0);
        *last_truck = Some(pos);
        t.translation = pos;
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
    let g = &game.0;
    let (fx, fy) = g.focus();
    let target = terrain_view::surface(&g.terrain, fx, fy) + Vec3::Y * 1.0;
    // Tilted = angled top-down-ish (the Macroquad view, in 3D); Side = low, side-on "2.5D" view.
    let offset = match *mode {
        CamMode::Tilted => Vec3::new(0.0, 13.0, 10.5),
        CamMode::Side => Vec3::new(0.0, 3.2, 15.0),
    };
    let mut desired = target + offset;
    // never dip under a hill that rises behind the player
    desired.y = desired.y.max(ground_y(&g.terrain, desired.x / S, -desired.z / S) + 1.8);
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
        light.illuminance = 15_000.0 * (1.0 - dark * 0.9) * (0.7 + 0.3 * elevation.sin());
        light.color = Color::srgb(sun_color.x, sun_color.y, sun_color.z);
    }
    ambient.brightness = 260.0 + 440.0 * (1.0 - (night / 0.85).clamp(0.0, 1.0));
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
/// Only touch a `Text` when it really changed: re-assigning identical text re-lays-out the glyphs.
fn set_text(t: &mut Text, new: String) {
    if t.0 != new {
        t.0 = new;
    }
}

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
        let mut lines = vec![
            format!("Outcrop Points: {}", g.outcrop_points),
            format!("XP: {}   Pack: {}/{}", g.xp, g.backpack_used(), kit::BACKPACK_SLOTS),
            format!("Day {}   {:02}:{:02}   (sunset 18:00)", g.day, m / 60, m % 60),
        ];
        if matches!(g.phase, Phase::Exploring | Phase::Basecamp) {
            let pk = &g.pickaxe;
            lines.push(format!("Pickaxe {} {}/{}   Food: {} apple, {} sandwich", pk.name(), pk.durability, pk.max_durability(), g.apples, g.sandwiches));
            let q = g.region.quota;
            lines.push(format!("{}  (map {}/{})   Survey {}/{}", g.region.name, g.level + 1, geology::REGIONS.len(), g.surveyed.min(q), q));
        }
        if g.phase == Phase::Exploring {
            let slope = (g.move_grade * 100.0).round() as i32;
            let wading = if g.ground_here() == terrain::Ground::River { "   wading" } else { "" };
            lines.push(format!("Elev {} m   Slope {:+}%{}", g.elevation_here().round() as i32, slope, wading));
            let u = g.unit_here();
            lines.push(format!("Unit: {} ({})", u.name, u.age));
        }
        if g.water <= 0.0 {
            lines.push("Water - EMPTY".into());
        }
        lines.push(format!("[V] camera: {}   [B] backpack", if *mode == CamMode::Tilted { "tilted" } else { "side" }));
        set_text(&mut t, if g.ui_mode == UiMode::Backpack { String::new() } else { ascii(&lines.join("\n")) });
    }
    if let Ok(mut t) = hint.single_mut() {
        let msg = match g.phase {
            Phase::Basecamp if g.near_truck() => "Press E to board the truck".into(),
            Phase::Basecamp => "Walk to the truck on the road below camp (WASD / click)".into(),
            Phase::Exploring if g.player_at_camp() => "Press E (TRUCK) to rest at camp and end the day".into(),
            Phase::Exploring if g.player_on_road() => "Press E to call the truck back to basecamp".into(),
            Phase::Exploring if g.nearest_outcrop().is_some() => "SPACE (or click the rock) to break it".into(),
            Phase::Exploring if g.pickaxe.broken() => "Pickaxe broken - press B for a new one".into(),
            Phase::Exploring if g.near_water() && g.water < kit::WATER_MAX - 0.5 => "Press F to fill your bottle".into(),
            Phase::Exploring if g.near_tree() && g.apples + g.sandwiches > 0 && g.energy <= kit::NOT_HUNGRY_ABOVE => "Shade here - press F to eat".into(),
            Phase::Exploring if g.minutes_to_sunset() <= 120 => format!("Sunset in {} min - reach a main road!", g.minutes_to_sunset()),
            Phase::GameOver => "GAME OVER - night fell off the road. Press ENTER to start over".into(),
            Phase::Sleeping => format!("Day {} complete - resting...", g.day),
            _ => String::new(),
        };
        set_text(&mut t, msg);
    }
    if let Ok(mut t) = banner.single_mut() {
        set_text(&mut t, g.banner.as_ref().map(|(s, _)| s.clone()).unwrap_or_default());
    }
    if let Ok(mut t) = log.single_mut() {
        set_text(&mut t, g.popups.iter().rev().take(4).map(|p| p.text.clone()).collect::<Vec<_>>().join("\n"));
    }
}

// ================================================================ DEV HOOKS
// OO_DEMO=camp|field|noon|sunset|night|road  scripts a scene;  OO_SHOT=/path.png [OO_SHOT_FRAMES=n]
// saves a screenshot after n frames and exits. Handy for headless visual checks; harmless otherwise.
fn dev_hooks(mut commands: Commands, mut game: ResMut<GameRes>, mut frame: Local<u32>, mut cam: ResMut<CamMode>) {
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
            // terrain scenes: stand on open ground near the kind of terrain we want to look at
            let want: Option<fn(terrain::Ground) -> bool> = match scene.as_str() {
                "farm" => Some(|k| k == terrain::Ground::Farm),
                "lake" | "pack" => Some(|k| k == terrain::Ground::Lake),
                "river" => Some(|k| k == terrain::Ground::River),
                "forest" => Some(|k| k == terrain::Ground::Forest),
                "cliff" => Some(|k| k == terrain::Ground::Cliff),
                _ => None,
            };
            if let Some(want) = want {
                if let Some((x, y)) = find_spot(&g.terrain, want) {
                    g.player.x = x;
                    g.player.y = y;
                }
                if scene == "pack" {
                    g.water = 40.0;
                    g.pickaxe.durability = 4;
                    g.outcrop_points = 150;
                    g.apples = 2;
                    g.ui_mode = UiMode::Backpack;
                }
            }
            // OO_CLOCK=<hour> sets the time of day, OO_CAM=side picks the side-on camera
            if let Some(h) = std::env::var("OO_CLOCK").ok().and_then(|v| v.parse::<u32>().ok()) {
                g.clock_ticks = (h.max(5) - 5) * 60 * TICKS_PER_MINUTE;
            }
            if std::env::var("OO_CAM").as_deref() == Ok("side") {
                *cam = CamMode::Side;
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

/// Nearest open, off-road spot (spiralling out from the camp) with `want` ground within ~250 units.
fn find_spot(t: &terrain::Terrain, want: fn(terrain::Ground) -> bool) -> Option<(f32, f32)> {
    for ring in 1..60 {
        let r = ring as f32 * 120.0;
        for k in 0..(ring * 8) {
            let a = k as f32 / (ring * 8) as f32 * std::f32::consts::TAU;
            let (x, y) = (TRUCK_SPOT.0 + a.cos() * r, TRUCK_SPOT.1 + a.sin() * r);
            if t.ground(x, y) != terrain::Ground::Open || terrain::clear_distance(x, y) < 350.0 {
                continue;
            }
            if (0..8).any(|d| {
                let b = d as f32 * std::f32::consts::FRAC_PI_4;
                want(t.ground(x + b.cos() * 260.0, y + b.sin() * 260.0))
            }) {
                return Some((x, y));
            }
        }
    }
    None
}

/// Bevy's built-in font has no accented letters; fold them so "Bacia do Parana" doesn't show boxes.
fn ascii(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' => 'e',
            'í' | 'ì' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ç' => 'c',
            'Á' | 'Â' | 'Ã' => 'A',
            'É' | 'Ê' => 'E',
            'Í' => 'I',
            'Ó' | 'Ô' | 'Õ' => 'O',
            'Ú' => 'U',
            'Ç' => 'C',
            c => c,
        })
        .collect()
}
