//! Textured materials and hand-built models: the vintage pickup truck and the geologist.
//!
//! Everything is assembled from primitives plus the procedural PBR maps in `assets/textures`
//! (see `tools/make_procedural_textures.py`). Swap any of it for a Blender `.glb` later and
//! nothing else changes: the rest of the app only talks to the `TruckRig` / `PlayerRig` entities.

use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

// ------------------------------------------------------------------ components
#[derive(Component)]
pub struct PlayerRig;
#[derive(Component)]
pub struct TruckRig;
/// Swinging limbs of the geologist; `sign` is the phase (+1 / -1) of the walk cycle.
#[derive(Component)]
pub struct Limb {
    pub sign: f32,
    pub arm: bool,
}
#[derive(Component)]
pub struct Torso;
#[derive(Component)]
pub struct WheelSpin;

// ------------------------------------------------------------------- textures
fn repeat() -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    })
}

pub fn tex(assets: &AssetServer, path: String, srgb: bool) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(move |s: &mut ImageLoaderSettings| {
            s.is_srgb = srgb;
            s.sampler = repeat();
        })
        .load(path)
}

/// albedo (sRGB) + normal map (linear) from `textures/<name>_{albedo.jpg,normal.png}`.
pub fn textured(assets: &AssetServer, name: &str, tint: Color, uv_scale: f32, rough: f32, metallic: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: tint,
        base_color_texture: Some(tex(assets, format!("textures/{name}_albedo.jpg"), true)),
        normal_map_texture: Some(tex(assets, format!("textures/{name}_normal.png"), false)),
        perceptual_roughness: rough,
        metallic,
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(uv_scale)),
        ..default()
    }
}

pub fn plain(color: Color, rough: f32, metallic: f32) -> StandardMaterial {
    StandardMaterial { base_color: color, perceptual_roughness: rough, metallic, ..default() }
}

/// Every material the scenery uses.
pub struct Surfaces {
    pub grass: Handle<StandardMaterial>,
    pub strata: Handle<StandardMaterial>,
    pub road: Handle<StandardMaterial>,
    pub dash: Handle<StandardMaterial>,
    pub yard: Handle<StandardMaterial>,
    pub crop: Handle<StandardMaterial>,
    pub fence: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub roof: Handle<StandardMaterial>,
    pub canvas: Handle<StandardMaterial>,
    pub crate_wood: Handle<StandardMaterial>,
    pub log: Handle<StandardMaterial>,
    pub ember: Handle<StandardMaterial>,
    // truck
    pub paint: Handle<StandardMaterial>,
    pub white: Handle<StandardMaterial>,
    pub yellow: Handle<StandardMaterial>,
    pub bed_wood: Handle<StandardMaterial>,
    pub wicker: Handle<StandardMaterial>,
    pub chrome: Handle<StandardMaterial>,
    pub dull_metal: Handle<StandardMaterial>,
    pub glass: Handle<StandardMaterial>,
    pub rubber: Handle<StandardMaterial>,
    pub headlight: Handle<StandardMaterial>,
    // geologist
    pub skin: Handle<StandardMaterial>,
    pub beard: Handle<StandardMaterial>,
    pub hair: Handle<StandardMaterial>,
    pub shirt: Handle<StandardMaterial>,
    pub vest: Handle<StandardMaterial>,
    pub reflective: Handle<StandardMaterial>,
    pub pants: Handle<StandardMaterial>,
    pub boots: Handle<StandardMaterial>,
    pub pack: Handle<StandardMaterial>,
    pub hat: Handle<StandardMaterial>,
    pub dark: Handle<StandardMaterial>,
    pub white_eye: Handle<StandardMaterial>,
    pub notebook: Handle<StandardMaterial>,
    pub brass: Handle<StandardMaterial>,
}

impl Surfaces {
    pub fn load(assets: &AssetServer, m: &mut Assets<StandardMaterial>) -> Self {
        let fabric = tex(assets, "textures/fabric_normal.png".into(), false);
        let mut cloth = |c: Color, rough: f32| {
            m.add(StandardMaterial { base_color: c, perceptual_roughness: rough, normal_map_texture: Some(fabric.clone()), uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(6.0)), ..default() })
        };
        let shirt = cloth(Color::srgb(0.80, 0.77, 0.68), 0.9);
        let vest = cloth(Color::srgb(0.96, 0.46, 0.10), 0.85);
        let pants = cloth(Color::srgb(0.44, 0.40, 0.29), 0.95);
        let pack = cloth(Color::srgb(0.20, 0.24, 0.31), 0.9);
        let wicker_mat = m.add(textured(assets, "wicker", Color::srgb(0.85, 0.6, 0.5), 1.0, 0.95, 0.0));
        Surfaces {
            grass: m.add(StandardMaterial {
                perceptual_roughness: 1.0,
                ..textured(assets, "grass", Color::srgb(0.85, 1.12, 0.70), 0.25, 1.0, 0.0)
            }),
            // cliffs show banded sedimentary layers (see terrain_view for the dipping UVs)
            strata: m.add(textured(assets, "strata", Color::srgb(1.0, 0.95, 0.84), 0.3, 0.95, 0.0)),
            road: m.add(textured(assets, "asphalt", Color::srgb(0.9, 0.9, 0.95), 0.35, 0.88, 0.0)),
            dash: m.add(plain(Color::srgb(0.92, 0.78, 0.25), 0.7, 0.0)),
            yard: m.add(textured(assets, "gravel", Color::srgb(1.0, 0.96, 0.9), 0.4, 1.0, 0.0)),
            crop: m.add(textured(assets, "crop", Color::WHITE, 0.5, 0.95, 0.0)),
            fence: m.add(textured(assets, "wood", Color::srgb(0.75, 0.65, 0.55), 1.0, 0.95, 0.0)),
            wall: m.add(textured(assets, "wood", Color::srgb(0.95, 0.82, 0.70), 1.0, 0.92, 0.0)),
            roof: m.add(textured(assets, "shingles", Color::WHITE, 1.0, 0.8, 0.0)),
            canvas: m.add(textured(assets, "canvas", Color::srgb(1.0, 0.95, 0.85), 1.0, 0.95, 0.0)),
            crate_wood: m.add(textured(assets, "wood", Color::srgb(1.1, 0.95, 0.75), 1.0, 0.95, 0.0)),
            log: m.add(textured(assets, "bark", Color::srgb(0.7, 0.55, 0.45), 1.0, 1.0, 0.0)),
            ember: m.add(StandardMaterial { base_color: Color::srgb(1.0, 0.45, 0.1), emissive: LinearRgba::new(6.0, 2.0, 0.3, 1.0), ..default() }),

            paint: m.add(textured(assets, "truck_paint", Color::WHITE, 0.7, 0.45, 0.15)),
            white: m.add(plain(Color::srgb(0.93, 0.93, 0.9), 0.55, 0.1)),
            yellow: m.add(textured(assets, "truck_yellow", Color::WHITE, 0.8, 0.7, 0.0)),
            bed_wood: m.add(textured(assets, "wood", Color::srgb(0.55, 0.45, 0.38), 1.0, 0.95, 0.0)),
            wicker: wicker_mat,
            chrome: m.add(plain(Color::srgb(0.78, 0.8, 0.82), 0.3, 0.8)),
            dull_metal: m.add(plain(Color::srgb(0.55, 0.56, 0.55), 0.75, 0.5)),
            glass: m.add(StandardMaterial { base_color: Color::srgba(0.55, 0.75, 0.88, 0.85), perceptual_roughness: 0.08, metallic: 0.2, alpha_mode: AlphaMode::Blend, ..default() }),
            rubber: m.add(plain(Color::srgb(0.06, 0.06, 0.07), 0.95, 0.0)),
            headlight: m.add(StandardMaterial { base_color: Color::srgb(0.95, 0.95, 0.85), emissive: LinearRgba::new(0.6, 0.6, 0.45, 1.0), perceptual_roughness: 0.2, ..default() }),

            skin: m.add(plain(Color::srgb(0.88, 0.64, 0.50), 0.65, 0.0)),
            beard: m.add(plain(Color::srgb(0.20, 0.17, 0.15), 0.95, 0.0)),
            hair: m.add(plain(Color::srgb(0.42, 0.40, 0.38), 0.95, 0.0)),
            shirt,
            vest,
            reflective: m.add(plain(Color::srgb(0.85, 0.87, 0.8), 0.4, 0.3)),
            pants,
            boots: m.add(plain(Color::srgb(0.20, 0.13, 0.08), 0.8, 0.0)),
            pack,
            hat: m.add(plain(Color::srgb(0.95, 0.46, 0.14), 0.35, 0.0)),
            dark: m.add(plain(Color::srgb(0.05, 0.05, 0.06), 0.4, 0.2)),
            white_eye: m.add(plain(Color::srgb(0.96, 0.96, 0.94), 0.3, 0.0)),
            notebook: m.add(plain(Color::srgb(0.45, 0.50, 0.58), 0.8, 0.0)),
            brass: m.add(plain(Color::srgb(0.85, 0.7, 0.3), 0.35, 0.8)),
        }
    }
}

// ---------------------------------------------------------------------- meshes
/// Planar UVs in metres from the dominant normal axis, so a texture tiles at a constant scale
/// whatever the box size (`tile` = metres per texture repeat).
pub fn project_uvs(mut mesh: Mesh, tile: f32) -> Mesh {
    let (Some(VertexAttributeValues::Float32x3(pos)), Some(VertexAttributeValues::Float32x3(nor))) =
        (mesh.attribute(Mesh::ATTRIBUTE_POSITION).cloned(), mesh.attribute(Mesh::ATTRIBUTE_NORMAL).cloned())
    else {
        return mesh;
    };
    let uv: Vec<[f32; 2]> = pos
        .iter()
        .zip(&nor)
        .map(|(p, n)| {
            let (ax, ay, az) = (n[0].abs(), n[1].abs(), n[2].abs());
            if ay >= ax && ay >= az {
                [p[0] / tile, p[2] / tile]
            } else if ax >= az {
                [p[2] / tile, p[1] / tile]
            } else {
                [p[0] / tile, p[1] / tile]
            }
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    mesh
}

pub fn tex_box(meshes: &mut Assets<Mesh>, w: f32, h: f32, d: f32, tile: f32) -> Handle<Mesh> {
    meshes.add(project_uvs(Cuboid::new(w, h, d).mesh().build(), tile))
}

/// Upper half of a sphere with a flat base: hard-hat dome.
pub fn dome(radius: f32) -> Mesh {
    let mut mesh = Sphere::new(radius).mesh().uv(32, 16);
    if let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        for p in pos.iter_mut() {
            p[1] = p[1].max(0.0);
        }
    }
    mesh
}

// ----------------------------------------------------------------------- truck
/// A vintage pale-green pickup with a white roof, chrome bumper, yellow bed sides and a woven
/// cargo rail. Local +X is forward; the origin is on the ground under the middle of the truck.
pub fn spawn_truck(commands: &mut Commands, meshes: &mut Assets<Mesh>, s: &Surfaces) -> Entity {
    let mut mesh_of = |m: Mesh| meshes.add(m);
    let root = commands.spawn((TruckRig, Transform::default(), Visibility::default())).id();
    commands.entity(root).with_children(|t| {
        let mut part = |mesh: Handle<Mesh>, mat: &Handle<StandardMaterial>, at: Vec3, rot: Quat, scale: Vec3| {
            t.spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), Transform { translation: at, rotation: rot, scale }));
        };
        let id = Quat::IDENTITY;
        let one = Vec3::ONE;

        // chassis and running boards
        part(tex_box_free(&mut mesh_of, 4.1, 0.16, 1.2, 1.0), &s.dull_metal, Vec3::new(0.0, 0.5, 0.0), id, one);
        for z in [-0.78f32, 0.78] {
            part(tex_box_free(&mut mesh_of, 0.9, 0.05, 0.28, 1.0), &s.dull_metal, Vec3::new(0.25, 0.55, z), id, one);
        }

        // hood + bulbous front fenders (ellipsoids give the rounded 1940s look)
        part(tex_box_free(&mut mesh_of, 1.15, 0.5, 1.15, 1.5), &s.paint, Vec3::new(1.45, 1.0, 0.0), id, one);
        for z in [-0.7f32, 0.7] {
            part(mesh_of(Mesh::from(Sphere::new(0.5).mesh().uv(24, 12))), &s.paint, Vec3::new(1.35, 0.86, z), id, Vec3::new(1.15, 0.78, 0.5));
            part(mesh_of(Mesh::from(Sphere::new(0.5).mesh().uv(24, 12))), &s.paint, Vec3::new(-1.25, 0.84, z), id, Vec3::new(1.0, 0.7, 0.42));
        }
        part(mesh_of(Mesh::from(Sphere::new(0.5).mesh().uv(24, 12))), &s.paint, Vec3::new(1.95, 0.9, 0.0), id, Vec3::new(0.5, 0.62, 1.1));
        // grille, headlights, bumper
        part(tex_box_free(&mut mesh_of, 0.05, 0.42, 0.72, 1.0), &s.chrome, Vec3::new(2.14, 0.86, 0.0), id, one);
        for k in 0..5 {
            part(tex_box_free(&mut mesh_of, 0.06, 0.4, 0.025, 1.0), &s.dark, Vec3::new(2.165, 0.86, -0.27 + k as f32 * 0.135), id, one);
        }
        for z in [-0.55f32, 0.55] {
            part(mesh_of(Mesh::from(Sphere::new(0.13).mesh().uv(16, 8))), &s.headlight, Vec3::new(2.06, 1.0, z), id, one);
        }
        part(tex_box_free(&mut mesh_of, 0.16, 0.13, 1.7, 1.0), &s.dull_metal, Vec3::new(2.22, 0.5, 0.0), id, one);

        // cab: green walls, white roof, glass
        part(tex_box_free(&mut mesh_of, 1.0, 0.95, 1.36, 1.5), &s.paint, Vec3::new(0.35, 1.46, 0.0), id, one);
        part(tex_box_free(&mut mesh_of, 1.12, 0.08, 1.5, 1.0), &s.white, Vec3::new(0.34, 1.97, 0.0), id, one);
        part(tex_box_free(&mut mesh_of, 0.05, 0.6, 1.18, 1.0), &s.glass, Vec3::new(0.9, 1.58, 0.0), Quat::from_rotation_z(0.28), one);
        part(tex_box_free(&mut mesh_of, 0.05, 0.5, 1.1, 1.0), &s.glass, Vec3::new(-0.16, 1.6, 0.0), id, one);
        for z in [-0.69f32, 0.69] {
            part(tex_box_free(&mut mesh_of, 0.62, 0.48, 0.04, 1.0), &s.glass, Vec3::new(0.38, 1.62, z), id, one);
            // mirror on a short arm
            part(tex_box_free(&mut mesh_of, 0.04, 0.2, 0.14, 1.0), &s.chrome, Vec3::new(0.82, 1.62, z * 1.28), id, one);
            part(tex_box_free(&mut mesh_of, 0.04, 0.04, 0.22, 1.0), &s.chrome, Vec3::new(0.82, 1.55, z * 1.14), id, one);
            // door handle
            part(tex_box_free(&mut mesh_of, 0.1, 0.03, 0.03, 1.0), &s.chrome, Vec3::new(0.12, 1.4, z * 1.02), id, one);
        }

        // bed: wooden floor, yellow side boards, woven rails on stakes, tailgate
        part(tex_box_free(&mut mesh_of, 1.85, 0.08, 1.4, 1.0), &s.bed_wood, Vec3::new(-1.2, 0.98, 0.0), id, one);
        for z in [-0.7f32, 0.7] {
            part(tex_box_free(&mut mesh_of, 1.85, 0.4, 0.06, 0.8), &s.yellow, Vec3::new(-1.2, 1.2, z), id, one);
            part(tex_box_free(&mut mesh_of, 1.85, 0.5, 0.05, 0.5), &s.wicker, Vec3::new(-1.2, 1.64, z), id, one);
        }
        part(tex_box_free(&mut mesh_of, 0.06, 0.4, 1.4, 0.8), &s.yellow, Vec3::new(-2.13, 1.2, 0.0), id, one);
        part(tex_box_free(&mut mesh_of, 0.05, 0.5, 1.4, 0.5), &s.wicker, Vec3::new(-0.3, 1.64, 0.0), id, one);
        part(tex_box_free(&mut mesh_of, 0.05, 0.5, 1.4, 0.5), &s.wicker, Vec3::new(-2.13, 1.64, 0.0), id, one);
        for x in [-0.3f32, -1.2, -2.12] {
            for z in [-0.7f32, 0.7] {
                part(tex_box_free(&mut mesh_of, 0.07, 0.95, 0.07, 1.0), &s.bed_wood, Vec3::new(x, 1.42, z), id, one);
            }
        }
        // a gray tank slung under the bed, and a rusty spare
        part(mesh_of(Mesh::from(Cylinder::new(0.2, 0.8))), &s.dull_metal, Vec3::new(-0.4, 0.62, 0.62), Quat::from_rotation_z(FRAC_PI_2), one);

        // wheels: tyre, silver hubcap
        for (x, z) in [(1.4f32, 0.82f32), (1.4, -0.82), (-1.25, 0.82), (-1.25, -0.82)] {
            let side = z.signum();
            t.spawn((WheelSpin, Mesh3d(mesh_of(Mesh::from(Cylinder::new(0.42, 0.3)))), MeshMaterial3d(s.rubber.clone()), Transform::from_xyz(x, 0.42, z).with_rotation(Quat::from_rotation_x(FRAC_PI_2))))
                .with_children(|w| {
                    w.spawn((Mesh3d(mesh_of(Mesh::from(Cylinder::new(0.26, 0.04)))), MeshMaterial3d(s.chrome.clone()), Transform::from_xyz(0.0, side * 0.16 * z.signum(), 0.0)));
                });
        }
    });
    root
}

/// Box mesh helper that works while `part` holds the commands borrow.
fn tex_box_free(mesh_of: &mut impl FnMut(Mesh) -> Handle<Mesh>, w: f32, h: f32, d: f32, tile: f32) -> Handle<Mesh> {
    mesh_of(Mesh::from(project_uvs(Cuboid::new(w, h, d).mesh().build(), tile)))
}

// ------------------------------------------------------------------- geologist
/// The geologist: orange hard hat, glasses, a big grin and grey beard, an orange vest over a
/// beige shirt, a backpack, a notebook in one hand and a rock in the other. Faces local +Z.
pub fn spawn_geologist(commands: &mut Commands, meshes: &mut Assets<Mesh>, s: &Surfaces) -> Entity {
    let root = commands.spawn((PlayerRig, Transform::default(), Visibility::default())).id();
    let mut add = |m: Mesh| meshes.add(m);
    commands.entity(root).with_children(|p| {
        let id = Quat::IDENTITY;
        let part = |parent: &mut ChildSpawnerCommands, mesh: Handle<Mesh>, mat: &Handle<StandardMaterial>, at: Vec3, rot: Quat, scale: Vec3| {
            parent.spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), Transform { translation: at, rotation: rot, scale }));
        };

        // legs: pivot at the hip so they can swing
        for (x, sign) in [(-0.14f32, 1.0f32), (0.14, -1.0)] {
            p.spawn((Limb { sign, arm: false }, Transform::from_xyz(x, 0.80, 0.0), Visibility::default())).with_children(|l| {
                part(l, add(Mesh::from(Capsule3d::new(0.125, 0.42))), &s.pants, Vec3::new(0.0, -0.30, 0.0), id, Vec3::ONE);
                part(l, add(Mesh::from(Cuboid::new(0.24, 0.17, 0.40))), &s.boots, Vec3::new(0.0, -0.66, 0.07), id, Vec3::ONE);
            });
        }

        // torso, belt, vest, backpack
        p.spawn((Torso, Transform::default(), Visibility::default())).with_children(|b| {
            part(b, add(Mesh::from(Capsule3d::new(0.29, 0.30))), &s.shirt, Vec3::new(0.0, 1.10, 0.0), id, Vec3::new(1.2, 1.0, 0.86));
            part(b, add(Mesh::from(Cylinder::new(0.32, 0.07))), &s.dark, Vec3::new(0.0, 0.86, 0.0), id, Vec3::new(1.12, 1.0, 0.86));
            part(b, add(Mesh::from(Cuboid::new(0.07, 0.06, 0.03))), &s.brass, Vec3::new(0.0, 0.86, 0.28), id, Vec3::ONE);
            // orange vest: two front panels with reflective stripes, a back panel, shoulder straps
            for x in [-0.20f32, 0.20] {
                part(b, add(Mesh::from(Cuboid::new(0.26, 0.56, 0.09))), &s.vest, Vec3::new(x, 1.10, 0.235), Quat::from_rotation_y(-x.signum() * 0.18), Vec3::ONE);
                part(b, add(Mesh::from(Cuboid::new(0.27, 0.035, 0.095))), &s.reflective, Vec3::new(x, 0.98, 0.24), Quat::from_rotation_y(-x.signum() * 0.18), Vec3::ONE);
                part(b, add(Mesh::from(Cuboid::new(0.16, 0.12, 0.06))), &s.vest, Vec3::new(x * 1.05, 0.98, 0.30), id, Vec3::new(1.0, 1.0, 0.7));
                part(b, add(Mesh::from(Cuboid::new(0.12, 0.04, 0.34))), &s.vest, Vec3::new(x * 0.9, 1.38, 0.0), id, Vec3::ONE);
            }
            part(b, add(Mesh::from(Cuboid::new(0.60, 0.56, 0.08))), &s.vest, Vec3::new(0.0, 1.10, -0.23), id, Vec3::ONE);
            // backpack and its straps
            part(b, add(Mesh::from(Cuboid::new(0.52, 0.64, 0.24))), &s.pack, Vec3::new(0.0, 1.12, -0.38), id, Vec3::ONE);
            part(b, add(Mesh::from(Cuboid::new(0.34, 0.26, 0.10))), &s.vest, Vec3::new(0.0, 0.98, -0.53), id, Vec3::ONE);
            for x in [-0.19f32, 0.19] {
                part(b, add(Mesh::from(Cuboid::new(0.07, 0.07, 0.62))), &s.pack, Vec3::new(x, 1.40, -0.08), id, Vec3::ONE);
                part(b, add(Mesh::from(Cuboid::new(0.07, 0.42, 0.04))), &s.pack, Vec3::new(x, 1.14, 0.29), id, Vec3::ONE);
            }
        });

        // arms: pivot at the shoulder; right hand holds a notebook, left a rock
        for (x, sign) in [(-0.40f32, -1.0f32), (0.40, 1.0)] {
            p.spawn((Limb { sign, arm: true }, Transform::from_xyz(x, 1.30, 0.0), Visibility::default())).with_children(|a| {
                part(a, add(Mesh::from(Capsule3d::new(0.105, 0.16))), &s.shirt, Vec3::new(0.0, -0.13, 0.0), id, Vec3::ONE);
                part(a, add(Mesh::from(Capsule3d::new(0.085, 0.22))), &s.skin, Vec3::new(0.0, -0.40, 0.04), Quat::from_rotation_x(-0.3), Vec3::ONE);
                part(a, add(Mesh::from(Sphere::new(0.095))), &s.skin, Vec3::new(0.0, -0.58, 0.12), id, Vec3::ONE);
                if sign > 0.0 {
                    part(a, add(Mesh::from(Cuboid::new(0.20, 0.27, 0.035))), &s.notebook, Vec3::new(0.04, -0.60, 0.20), Quat::from_rotation_x(-0.5) * Quat::from_rotation_z(0.2), Vec3::ONE);
                } else {
                    part(a, add(Mesh::from(Sphere::new(0.12).mesh().ico(2).unwrap())), &s.dull_metal, Vec3::new(0.0, -0.62, 0.18), id, Vec3::new(1.2, 0.8, 1.0));
                }
            });
        }

        // head (big and friendly)
        let head = Vec3::new(0.0, 1.62, 0.02);
        part(p, add(Mesh::from(Sphere::new(0.34).mesh().uv(32, 16))), &s.skin, head, id, Vec3::new(1.0, 1.05, 0.98));
        part(p, add(Mesh::from(Sphere::new(0.06))), &s.skin, head + Vec3::new(0.0, -0.04, 0.34), id, Vec3::new(1.0, 0.9, 1.0)); // nose
        part(p, add(Mesh::from(Sphere::new(0.05))), &s.skin, head + Vec3::new(-0.33, -0.04, 0.0), id, Vec3::new(0.5, 1.0, 1.0)); // ears
        part(p, add(Mesh::from(Sphere::new(0.05))), &s.skin, head + Vec3::new(0.33, -0.04, 0.0), id, Vec3::new(0.5, 1.0, 1.0));
        // beard, moustache, grin
        part(p, add(Mesh::from(Sphere::new(0.32).mesh().uv(24, 12))), &s.beard, head + Vec3::new(0.0, -0.20, 0.035), id, Vec3::new(1.0, 0.72, 0.95));
        part(p, add(Mesh::from(Cuboid::new(0.24, 0.045, 0.05))), &s.beard, head + Vec3::new(0.0, -0.075, 0.33), Quat::from_rotation_z(0.0), Vec3::ONE);
        part(p, add(Mesh::from(Cuboid::new(0.17, 0.055, 0.04))), &s.white_eye, head + Vec3::new(0.0, -0.145, 0.315), id, Vec3::ONE);
        // eyes, brows and round glasses
        for x in [-0.115f32, 0.115] {
            part(p, add(Mesh::from(Sphere::new(0.055))), &s.white_eye, head + Vec3::new(x, 0.045, 0.285), id, Vec3::new(1.0, 1.0, 0.7));
            part(p, add(Mesh::from(Sphere::new(0.03))), &s.dark, head + Vec3::new(x, 0.045, 0.318), id, Vec3::ONE);
            part(p, add(Mesh::from(Cuboid::new(0.12, 0.024, 0.03))), &s.hair, head + Vec3::new(x, 0.125, 0.30), Quat::from_rotation_z(-x.signum() * 0.12), Vec3::ONE);
            part(p, add(Mesh::from(Torus { minor_radius: 0.012, major_radius: 0.088 })), &s.dark, head + Vec3::new(x, 0.045, 0.325), Quat::from_rotation_x(FRAC_PI_2), Vec3::ONE);
            part(p, add(Mesh::from(Cuboid::new(0.025, 0.025, 0.30))), &s.dark, head + Vec3::new(x * 2.7, 0.05, 0.17), id, Vec3::ONE);
        }
        part(p, add(Mesh::from(Cuboid::new(0.07, 0.018, 0.02))), &s.dark, head + Vec3::new(0.0, 0.055, 0.335), id, Vec3::ONE);
        // grey hair at the sides and back
        for (x, z) in [(-0.31f32, -0.02f32), (0.31, -0.02), (0.0, -0.30)] {
            part(p, add(Mesh::from(Sphere::new(0.10))), &s.hair, head + Vec3::new(x, 0.08, z), id, Vec3::new(if z < -0.2 { 2.6 } else { 1.0 }, 1.3, 1.0));
        }
        // hard hat: dome + brim + front ridge
        part(p, add(Mesh::from(dome(0.375))), &s.hat, head + Vec3::new(0.0, 0.135, 0.0), id, Vec3::new(1.0, 0.95, 1.05));
        part(p, add(Mesh::from(Cylinder::new(0.43, 0.035))), &s.hat, head + Vec3::new(0.0, 0.14, 0.05), id, Vec3::new(1.0, 1.0, 1.12));
        part(p, add(Mesh::from(Cuboid::new(0.07, 0.05, 0.70))), &s.hat, head + Vec3::new(0.0, 0.50, 0.0), Quat::from_rotation_x(0.02), Vec3::ONE);
        part(p, add(Mesh::from(Cuboid::new(0.05, 0.035, 0.12))), &s.hat, head + Vec3::new(0.0, 0.43, 0.34), id, Vec3::ONE);
    });
    root
}

/// Swing arms and legs while the player walks; breathe a little when standing.
pub fn animate_geologist(time: Res<Time>, anim: Res<WalkAmount>, mut limbs: Query<(&Limb, &mut Transform)>, mut torso: Query<&mut Transform, (With<Torso>, Without<Limb>)>) {
    let t = time.elapsed_secs();
    let w = anim.0;
    for (limb, mut tf) in &mut limbs {
        let swing = (t * 11.0).sin() * limb.sign;
        let (amp, idle) = if limb.arm { (0.75, (t * 1.6).sin() * 0.04) } else { (0.65, 0.0) };
        tf.rotation = Quat::from_rotation_x(swing * amp * w + idle * (1.0 - w));
    }
    if let Ok(mut tf) = torso.single_mut() {
        tf.scale = Vec3::splat(1.0 + (t * 1.6).sin() * 0.008 * (1.0 - w));
        tf.rotation = Quat::from_rotation_z((t * 11.0).sin() * 0.04 * w);
    }
}

/// 0 = standing, 1 = walking (smoothed); written by `sync_actors`.
#[derive(Resource, Default)]
pub struct WalkAmount(pub f32);

/// Roll the wheels in proportion to how fast the truck moves.
pub fn spin_wheels(time: Res<Time>, speed: Res<TruckSpeed>, mut wheels: Query<&mut Transform, With<WheelSpin>>) {
    let a = speed.0 * time.delta_secs() / 0.42;
    for mut t in &mut wheels {
        // the wheel is a cylinder rotated onto its side; spin it about its own (local Y) axis
        t.rotate_local_y(a);
    }
}

/// Truck speed in metres/second; written by `sync_actors`.
#[derive(Resource, Default)]
pub struct TruckSpeed(pub f32);

#[allow(dead_code)]
const _UNUSED: f32 = PI;
