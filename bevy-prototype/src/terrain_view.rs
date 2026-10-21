//! 3D view of the game's terrain.
//!
//! The game logic (`../../src/terrain.rs`) is a pure function of position: elevation, lakes,
//! rivers, cliffs, forest and farms. This module turns it into meshes:
//!
//! * one chunk = a 40x40 grid of 16-unit cells sampled from `Terrain::elevation`;
//! * each cell is classified exactly like the Macroquad renderer does (`Terrain::classify`) and
//!   goes into one of four meshes: grass (open ground, farm, forest floor, lake bed), cliff rock,
//!   water, and the dark "outside the survey" border;
//! * roads, the camp yard and farm plots are *draped* over the same height function, so they
//!   follow the relief instead of floating or sinking.
//!
//! Heights use `VS = 1.25 * S`: the vertical scale is the horizontal scale plus a little drama.

use crate::terrain::{contour_segments, Ground, Terrain, BASE_ELEV, CONTOUR_INTERVAL, ELEV_MAX, INDEX_EVERY, WATER_LEVEL};
use crate::world::CHUNK;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// metres per game unit (horizontal).
pub const S: f32 = 0.025;
/// metres per metre-of-elevation (vertical).
pub const VS: f32 = S * 1.25;
const STEP: f32 = 16.0;
const N: usize = 40; // CHUNK / STEP
const _: () = assert!(N as i32 * 16 == CHUNK);

/// Elevation (game metres) -> Bevy height. Linear at VS, plus extra lift above 198 m so the
/// summits (the impassable "cliff" massifs, which the game logic reduces to a flat-topped patch)
/// stand up as real steep, layered mountains instead of a coloured sticker on the grass.
pub fn height_of(e: f32) -> f32 {
    let t = ((e - 198.0) / (ELEV_MAX - 198.0)).clamp(0.0, 1.0);
    (e - BASE_ELEV) * VS + 7.0 * t.powf(1.4)
}

/// World height (Bevy metres) of the ground at game position (x, y).
pub fn ground_y(t: &Terrain, x: f32, y: f32) -> f32 {
    height_of(t.elevation(x, y))
}

pub fn water_y(t: &Terrain) -> f32 {
    height_of(t.water_level)
}

/// Game position -> Bevy position on the terrain surface.
pub fn surface(t: &Terrain, x: f32, y: f32) -> Vec3 {
    Vec3::new(x * S, ground_y(t, x, y), -y * S)
}

// ------------------------------------------------------------------ mesh buffer
#[derive(Default)]
pub struct Buf {
    pos: Vec<[f32; 3]>,
    nor: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    col: Vec<[f32; 4]>,
    idx: Vec<u32>,
}

impl Buf {
    pub fn is_empty(&self) -> bool {
        self.idx.is_empty()
    }

    fn push(&mut self, p: Vec3, n: Vec3, uv: [f32; 2], c: [f32; 4]) -> u32 {
        self.pos.push(p.to_array());
        self.nor.push(n.to_array());
        self.uv.push(uv);
        self.col.push(c);
        (self.pos.len() - 1) as u32
    }

    /// Quad with corners in order a, b, c, d (a->b->c counter-clockwise seen from the normal side).
    fn quad(&mut self, p: [Vec3; 4], n: [Vec3; 4], uv: [[f32; 2]; 4], c: [[f32; 4]; 4]) {
        let b = self.pos.len() as u32;
        for k in 0..4 {
            self.push(p[k], n[k], uv[k], c[k]);
        }
        self.idx.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }

    /// Quad that always faces up, whichever way the corners were given.
    fn quad_up(&mut self, p: [Vec3; 4], c: [f32; 4]) {
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        let (p, n) = if n.y >= 0.0 { (p, Vec3::Y) } else { ([p[0], p[3], p[2], p[1]], Vec3::Y) };
        let uv = p.map(|v| [v.x, v.z]);
        self.quad(p, [n; 4], uv, [c; 4]);
    }

    /// An oriented box from a to b (centre line), `w` wide and `h` tall. Flat-shaded.
    fn beam(&mut self, a: Vec3, b: Vec3, w: f32, h: f32, c: [f32; 4]) {
        let d = (b - a).normalize_or_zero();
        if d == Vec3::ZERO {
            return;
        }
        let side = if d.y.abs() > 0.95 { Vec3::X } else { Vec3::Y.cross(d).normalize() };
        let up = d.cross(side).normalize();
        let (s, u) = (side * w / 2.0, up * h / 2.0);
        let corner = |o: Vec3, sx: f32, sy: f32| o + s * sx + u * sy;
        let faces: [([Vec3; 4], Vec3); 6] = [
            ([corner(a, -1., 1.), corner(a, 1., 1.), corner(b, 1., 1.), corner(b, -1., 1.)], up),
            ([corner(a, 1., -1.), corner(a, -1., -1.), corner(b, -1., -1.), corner(b, 1., -1.)], -up),
            ([corner(a, 1., 1.), corner(a, 1., -1.), corner(b, 1., -1.), corner(b, 1., 1.)], side),
            ([corner(a, -1., -1.), corner(a, -1., 1.), corner(b, -1., 1.), corner(b, -1., -1.)], -side),
            ([corner(b, -1., 1.), corner(b, 1., 1.), corner(b, 1., -1.), corner(b, -1., -1.)], d),
            ([corner(a, 1., 1.), corner(a, -1., 1.), corner(a, -1., -1.), corner(a, 1., -1.)], -d),
        ];
        for (p, n) in faces {
            // make sure the winding agrees with the stated normal
            let g = (p[1] - p[0]).cross(p[2] - p[0]);
            let p = if g.dot(n) >= 0.0 { p } else { [p[0], p[3], p[2], p[1]] };
            self.quad(p, [n; 4], [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], [c; 4]);
        }
    }

    pub fn build(self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.pos)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.nor)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uv)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.col)
            .with_inserted_indices(Indices::U32(self.idx))
    }
}

// ------------------------------------------------------------------- colours
/// Hypsometric tint like the 2D map (low ground dark green, uplands olive, peaks pale), turned
/// into a gentle *multiplier* for the grass texture rather than a flat colour.
fn open_tint(e: f32) -> [f32; 4] {
    const STOPS: [(f32, [f32; 3]); 5] = [
        (0.0, [40.0, 90.0, 62.0]),
        (0.35, [88.0, 132.0, 72.0]),
        (0.60, [145.0, 141.0, 117.0]),
        (0.80, [178.0, 168.0, 142.0]),
        (1.0, [236.0, 236.0, 232.0]),
    ];
    let t = (e / ELEV_MAX).clamp(0.0, 1.0);
    let mut rgb = STOPS[4].1;
    for w in STOPS.windows(2) {
        if t <= w[1].0 {
            let k = (t - w[0].0) / (w[1].0 - w[0].0);
            rgb = [0, 1, 2].map(|i| w[0].1[i] + (w[1].1[i] - w[0].1[i]) * k);
            break;
        }
    }
    let m = rgb.map(|c| (1.0 + 0.55 * (c / 255.0 * 2.0 - 1.0)).clamp(0.5, 1.4));
    [m[0], m[1], m[2], 1.0]
}

const FOREST_TINT: [f32; 4] = [0.42, 0.62, 0.40, 1.0];
const FARM_TINT: [f32; 4] = [1.25, 1.2, 0.70, 1.0];
const BED_TINT: [f32; 4] = [0.45, 0.5, 0.42, 1.0];

fn hash2(a: i32, b: i32) -> f32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B1) ^ (b as u32).wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 13;
    (h & 0xFFFF) as f32 / 65535.0
}

// -------------------------------------------------------------------- chunks
pub struct ChunkGeo {
    pub grass: Mesh,
    pub cliff: Option<Mesh>,
    pub water: Option<Mesh>,
    pub edge: Option<Mesh>,
    /// Thin ribbons laid on the relief along every 15 m contour (index contours are heavier).
    pub contours: Option<Mesh>,
    /// Game positions where a forest tree should stand.
    pub forest: Vec<(f32, f32)>,
}

/// Build the terrain meshes of chunk (cx, cy). Vertices are relative to the chunk centre in x/z
/// (put the entity at `to_world(centre)`) and absolute in y.
pub fn build_chunk(t: &Terrain, cx: i32, cy: i32) -> ChunkGeo {
    let (ox, oy) = ((cx * CHUNK) as f32, (cy * CHUNK) as f32);
    let w = N + 3;
    // elevations with a one-sample border, so normals agree across chunk seams
    let mut e = vec![0.0f32; w * w];
    for j in 0..w {
        for i in 0..w {
            e[j * w + i] = t.elevation(ox + (i as f32 - 1.0) * STEP, oy + (j as f32 - 1.0) * STEP);
        }
    }
    let el = |i: i32, j: i32| e[(j + 1) as usize * w + (i + 1) as usize];
    let half = CHUNK as f32 / 2.0;
    let vert = |i: i32, j: i32| {
        let p = Vec3::new((i as f32 * STEP - half) * S, height_of(el(i, j)), -(j as f32 * STEP - half) * S);
        // slopes from the (exaggerated) heights: dh/dx_b = dh / (2 STEP S);  dh/dz_b = -dh/dy
        let k = 1.0 / (2.0 * STEP * S);
        let n = Vec3::new(-(height_of(el(i + 1, j)) - height_of(el(i - 1, j))) * k, 1.0, (height_of(el(i, j + 1)) - height_of(el(i, j - 1))) * k).normalize();
        let uv = [(ox + i as f32 * STEP) * S, -(oy + j as f32 * STEP) * S];
        (p, n, uv)
    };

    let (mut grass, mut cliff, mut water, mut edge, mut lines) = (Buf::default(), Buf::default(), Buf::default(), Buf::default(), Buf::default());
    let mut forest = Vec::new();
    let wy = water_y(t);

    for j in 0..N as i32 {
        for i in 0..N as i32 {
            let (e00, e10, e11, e01) = (el(i, j), el(i + 1, j), el(i + 1, j + 1), el(i, j + 1));
            let avg = (e00 + e10 + e11 + e01) / 4.0;
            let steep = (((e10 + e11 - e00 - e01) / 2.0) / STEP).hypot(((e01 + e11 - e00 - e10) / 2.0) / STEP);
            let (gcx, gcy) = (ox + (i as f32 + 0.5) * STEP, oy + (j as f32 + 0.5) * STEP);
            let ground = t.classify(gcx, gcy, avg, steep);

            let vs = [vert(i, j), vert(i + 1, j), vert(i + 1, j + 1), vert(i, j + 1)];
            let es = [e00, e10, e11, e01];
            let p = vs.map(|v| v.0);
            let n = vs.map(|v| v.1);
            let uv = vs.map(|v| v.2);

            match ground {
                Ground::Open | Ground::Farm | Ground::Forest | Ground::Lake | Ground::River => {
                    let c = match ground {
                        Ground::Open => [0, 1, 2, 3].map(|k| open_tint(es[k])),
                        Ground::Farm => [FARM_TINT; 4],
                        Ground::Forest => [FOREST_TINT; 4],
                        _ => [BED_TINT; 4],
                    };
                    grass.quad(p, n, uv, c);
                    if ground == Ground::Forest {
                        let (wi, wj) = (cx * N as i32 + i, cy * N as i32 + j);
                        if wi.rem_euclid(8) == 0 && wj.rem_euclid(8) == 0 {
                            let jx = (hash2(wi, wj) - 0.5) * 90.0;
                            let jy = (hash2(wj, wi) - 0.5) * 90.0;
                            forest.push((gcx + jx, gcy + jy));
                        }
                    }
                    if ground == Ground::Lake {
                        let flat = p.map(|v| Vec3::new(v.x, wy, v.z));
                        water.quad_up(flat, [1.0; 4]);
                    } else if ground == Ground::River {
                        let lifted = p.map(|v| v + Vec3::Y * 0.05);
                        water.quad_up(lifted, [1.0; 4]);
                    }
                }
                Ground::Cliff => {
                    // bedding planes: stripes run along strike and tilt with a gentle dip, so the
                    // rock reads as layered strata whichever way the face points
                    let uv = p.map(|v| {
                        let (wx, wz) = (v.x + ox * S + half * S, v.z - oy * S - half * S);
                        [wx * 0.8 - wz * 0.6, v.y * 2.0 + 0.3 * (wx * 0.6 + wz * 0.8)]
                    });
                    cliff.quad(p, n, uv, [[1.0; 4]; 4])
                }
                Ground::Edge => edge.quad(p, n, uv, [[1.0; 4]; 4]),
            }
            if ground != Ground::Edge {
                contour_cell(&mut lines, es, p, avg);
            }
        }
    }
    let opt = |b: Buf| if b.is_empty() { None } else { Some(b.build()) };
    ChunkGeo { grass: grass.build(), cliff: opt(cliff), water: opt(water), edge: opt(edge), contours: opt(lines), forest }
}

/// 3D contour lines for one cell: marching squares at every 15 m of elevation, drawn as flat
/// ribbons just above the ground. `es` are the corner elevations (bl, br, tr, tl), `p` the corner
/// positions in the same order.
fn contour_cell(out: &mut Buf, es: [f32; 4], p: [Vec3; 4], avg: f32) {
    let (lo, hi) = es.iter().fold((f32::MAX, f32::MIN), |(a, b), &e| (a.min(e), b.max(e)));
    if hi < WATER_LEVEL || avg < WATER_LEVEL {
        return; // lake beds have no contours
    }
    let k0 = (lo / CONTOUR_INTERVAL).ceil() as i32;
    let k1 = (hi / CONTOUR_INTERVAL).floor() as i32;
    for k in k0..=k1 {
        let level = k as f32 * CONTOUR_INTERVAL;
        let index = k % INDEX_EVERY == 0;
        let (w, colour) = if index { (0.085, [0.16, 0.09, 0.04, 1.0]) } else { (0.04, [0.30, 0.19, 0.10, 1.0]) };
        let y = height_of(level) + 0.045;
        // cell coordinates (u along +x, v along +y of the game) onto the corner positions
        let at = |(u, v): (f32, f32)| {
            let x = p[0].x + (p[1].x - p[0].x) * u;
            let z = p[0].z + (p[3].z - p[0].z) * v;
            Vec3::new(x, y, z)
        };
        for (a, b) in contour_segments(es, level) {
            let (a, b) = (at(a), at(b));
            let d = (b - a).with_y(0.0);
            if d.length_squared() < 1e-8 {
                continue;
            }
            let side = Vec3::new(-d.z, 0.0, d.x).normalize() * w / 2.0;
            // round-ish caps: extend a touch along the line so joins between cells overlap
            let ext = d.normalize() * w * 0.4;
            out.quad_up([a - ext - side, b + ext - side, b + ext + side, a - ext + side], colour);
        }
    }
}

// ------------------------------------------------------------ draped surfaces
/// A strip along a straight line, draped over the terrain. Positions are relative to `origin`
/// (game units) in x/z and absolute in y.
pub fn drape_strip(t: &Terrain, from: (f32, f32), to: (f32, f32), width: f32, lift: f32, origin: (f32, f32), colour: [f32; 4]) -> Buf {
    let mut b = Buf::default();
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let len = dx.hypot(dy).max(1.0);
    let (px, py) = (-dy / len * width / 2.0, dx / len * width / 2.0);
    let n = (len / 20.0).ceil().max(1.0) as usize;
    let at = |x: f32, y: f32| Vec3::new((x - origin.0) * S, ground_y(t, x, y) + lift, -(y - origin.1) * S);
    for k in 0..n {
        let (f0, f1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
        let (x0, y0, x1, y1) = (from.0 + dx * f0, from.1 + dy * f0, from.0 + dx * f1, from.1 + dy * f1);
        b.quad_up([at(x0 - px, y0 - py), at(x1 - px, y1 - py), at(x1 + px, y1 + py), at(x0 + px, y0 + py)], colour);
    }
    b
}

/// A winding road ribbon draped over the terrain. `pos(t)` is the centre line in game units;
/// the tangent comes from the curve itself, so neighbouring slices join without cracks.
pub fn drape_curve(t: &Terrain, pos: &dyn Fn(f32) -> (f32, f32), t0: f32, t1: f32, step: f32, width: f32, lift: f32, origin: (f32, f32), colour: [f32; 4]) -> Buf {
    let mut b = Buf::default();
    let n = ((t1 - t0).abs() / step).ceil().max(1.0) as usize;
    let at = |x: f32, y: f32| Vec3::new((x - origin.0) * S, ground_y(t, x, y) + lift, -(y - origin.1) * S);
    let edge = |u: f32| {
        let (cx, cy) = pos(u);
        let (ax, ay) = pos(u - 2.0);
        let (bx, by) = pos(u + 2.0);
        let (dx, dy) = (bx - ax, by - ay);
        let len = dx.hypot(dy).max(1e-3);
        let (nx, ny) = (-dy / len * width / 2.0, dx / len * width / 2.0);
        (at(cx - nx, cy - ny), at(cx + nx, cy + ny))
    };
    let mut prev = edge(t0);
    for i in 1..=n {
        let cur = edge(t0 + (t1 - t0) * i as f32 / n as f32);
        b.quad_up([prev.0, cur.0, cur.1, prev.1], colour);
        prev = cur;
    }
    b
}

/// A rectangle draped over the terrain (camp yard), absolute world coordinates.
pub fn drape_rect(t: &Terrain, r: (f32, f32, f32, f32), lift: f32, colour: [f32; 4]) -> Buf {
    let mut b = Buf::default();
    let (x0, y0, x1, y1) = r;
    let (nx, ny) = (((x1 - x0) / STEP).ceil() as usize, ((y1 - y0) / STEP).ceil() as usize);
    let at = |x: f32, y: f32| Vec3::new(x * S, ground_y(t, x, y) + lift, -y * S);
    for j in 0..ny {
        for i in 0..nx {
            let xa = x0 + (x1 - x0) * i as f32 / nx as f32;
            let xb = x0 + (x1 - x0) * (i + 1) as f32 / nx as f32;
            let ya = y0 + (y1 - y0) * j as f32 / ny as f32;
            let yb = y0 + (y1 - y0) * (j + 1) as f32 / ny as f32;
            b.quad_up([at(xa, ya), at(xb, ya), at(xb, yb), at(xa, yb)], colour);
        }
    }
    b
}

/// A fenced farm: crop rows plus fence posts and rails, in absolute world coordinates.
pub fn build_farm(t: &Terrain, r: (f32, f32, f32, f32)) -> (Mesh, Mesh) {
    let (x0, y0, x1, y1) = r;
    let mut b = Buf::default();
    let mut fence = Buf::default();
    let rows = [[0.50, 0.60, 0.20, 1.0], [0.62, 0.54, 0.22, 1.0]];
    // crop rows
    let (mut y, mut k) = (y0 + 10.0, 0usize);
    while y < y1 - 22.0 {
        let strip = drape_strip(t, (x0 + 10.0, y + 6.5), (x1 - 10.0, y + 6.5), 13.0, 0.07, (0.0, 0.0), rows[k % 2]);
        append(&mut b, strip);
        y += 27.0;
        k += 1;
    }
    // fence
    let wood = [0.62, 0.55, 0.50, 1.0];
    let rail = [0.78, 0.68, 0.58, 1.0];
    let at = |x: f32, y: f32, h: f32| surface(t, x, y) + Vec3::Y * h;
    let mut side = |a: (f32, f32), z: (f32, f32)| {
        let (dx, dy) = (z.0 - a.0, z.1 - a.1);
        let n = (dx.hypot(dy) / 46.0).ceil().max(1.0) as usize;
        for s in 0..=n {
            let f = s as f32 / n as f32;
            let (x, y) = (a.0 + dx * f, a.1 + dy * f);
            fence.beam(at(x, y, -0.05), at(x, y, 1.15), 0.14, 0.14, wood);
            if s < n {
                let g = (s + 1) as f32 / n as f32;
                let (x2, y2) = (a.0 + dx * g, a.1 + dy * g);
                for h in [0.55, 0.95] {
                    fence.beam(at(x, y, h), at(x2, y2, h), 0.05, 0.10, rail);
                }
            }
        }
    };
    side((x0, y0), (x1, y0));
    side((x1, y0), (x1, y1));
    side((x1, y1), (x0, y1));
    side((x0, y1), (x0, y0));
    (b.build(), fence.build())
}

fn append(dst: &mut Buf, src: Buf) {
    let base = dst.pos.len() as u32;
    dst.pos.extend(src.pos);
    dst.nor.extend(src.nor);
    dst.uv.extend(src.uv);
    dst.col.extend(src.col);
    dst.idx.extend(src.idx.into_iter().map(|i| i + base));
}

pub fn merge(parts: Vec<Buf>) -> Buf {
    let mut out = Buf::default();
    for p in parts {
        append(&mut out, p);
    }
    out
}

/// Ray vs terrain: first point where the ray drops below the ground (marching + bisection).
pub fn raycast(t: &Terrain, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<Vec3> {
    let h = |p: Vec3| p.y - ground_y(t, p.x / S, -p.z / S);
    let mut prev = 0.0f32;
    let mut d = 0.0f32;
    while d < max_dist {
        d += 0.4;
        if h(origin + dir * d) <= 0.0 {
            let (mut lo, mut hi) = (prev, d);
            for _ in 0..16 {
                let mid = (lo + hi) / 2.0;
                if h(origin + dir * mid) > 0.0 { lo = mid } else { hi = mid }
            }
            return Some(origin + dir * hi);
        }
        prev = d;
    }
    None
}
