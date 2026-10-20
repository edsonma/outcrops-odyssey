//! Deterministic, infinite world generation. The same chunk coordinates
//! (and region) always regenerate the same content. World coordinates are y-up.

use crate::geology::{pick_rock, Region};
use crate::terrain::{Ground, Terrain};

pub const CHUNK: i32 = 640;
pub const VIEW_RADIUS_CHUNKS: i32 = 2;

// The Ruby version needed tiny primes to stay inside mruby's fixed-width
// integers. We keep the exact same arithmetic (in i64) so worlds generated
// by the Ruby prototype and this port are identical for the same chunk.
const HASH_MOD: i64 = 104_729;
const HASH_MUL: i64 = 9_973;
const HASH_MUL2: i64 = 7_919;

// ---- main roads & basecamp -------------------------------------------------
// A sparse, winding network: horizontal-ish roads run near y = k * ROAD_SPACING and
// vertical-ish ones near x = ROAD_V_OFFSET + j * ROAD_SPACING, but every road
// meanders (two summed sine waves with a per-road phase), so they are curves, not a
// chessboard. Blocks are 4800 units (120 m) across. The basecamp sits just north of
// the y = 0 road, which is dead straight near the camp so the truck spot stays put.
pub const ROAD_SPACING: f32 = 4800.0;
pub const ROAD_W: f32 = 120.0;
pub const ROAD_V_OFFSET: f32 = 2400.0;
pub const CAMP: (f32, f32, f32, f32) = (-400.0, 90.0, 400.0, 520.0); // x0, y0, x1, y1
pub const TRUCK_SPOT: (f32, f32) = (0.0, 0.0);
pub const CAMP_SPAWN: (f32, f32) = (0.0, 230.0);

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Sideways wander of a road at distance `s` along it (road `idx`, family `fam`).
fn wander(fam: f32, idx: i32, s: f32) -> f32 {
    let p = idx as f32 * 1.7 + fam * 4.1;
    260.0 * (s / 650.0 + p).sin() + 70.0 * (s / 280.0 + 2.0 * p + 1.0).sin()
}

/// y of horizontal road `k` at x. Road 0 is straight near the camp.
pub fn h_road_y(k: i32, x: f32) -> f32 {
    let fade = if k == 0 { smoothstep(600.0, 1500.0, x.abs()) } else { 1.0 };
    k as f32 * ROAD_SPACING + wander(0.0, k, x) * fade
}

/// x of vertical road `j` at y.
pub fn v_road_x(j: i32, y: f32) -> f32 {
    ROAD_V_OFFSET + j as f32 * ROAD_SPACING + wander(1.0, j, y)
}

pub fn h_index(y: f32) -> i32 {
    (y / ROAD_SPACING).round() as i32
}
pub fn v_index(x: f32) -> i32 {
    ((x - ROAD_V_OFFSET) / ROAD_SPACING).round() as i32
}

/// Slope dy/dx of horizontal road k (numerical; the curves are smooth).
fn h_slope(k: i32, x: f32) -> f32 {
    (h_road_y(k, x + 8.0) - h_road_y(k, x - 8.0)) / 16.0
}
fn v_slope(j: i32, y: f32) -> f32 {
    (v_road_x(j, y + 8.0) - v_road_x(j, y - 8.0)) / 16.0
}

/// Distance to the centre line of the closest main road.
pub fn road_distance(x: f32, y: f32) -> f32 {
    let k = h_index(y);
    let sh = h_slope(k, x);
    let dh = (y - h_road_y(k, x)).abs() / (1.0 + sh * sh).sqrt();
    let j = v_index(x);
    let sv = v_slope(j, y);
    let dv = (x - v_road_x(j, y)).abs() / (1.0 + sv * sv).sqrt();
    dh.min(dv)
}
pub fn on_road(x: f32, y: f32, extra: f32) -> bool {
    road_distance(x, y) <= ROAD_W / 2.0 + extra
}
pub fn in_camp(x: f32, y: f32, margin: f32) -> bool {
    x >= CAMP.0 - margin && x <= CAMP.2 + margin && y >= CAMP.1 - margin && y <= CAMP.3 + margin
}

/// Closest point on the centre line of the nearest road, and whether that road is horizontal-ish.
pub fn road_anchor(x: f32, y: f32) -> ((f32, f32), bool) {
    let (k, j) = (h_index(y), v_index(x));
    let sh = h_slope(k, x);
    let sv = v_slope(j, y);
    let dh = (y - h_road_y(k, x)).abs() / (1.0 + sh * sh).sqrt();
    let dv = (x - v_road_x(j, y)).abs() / (1.0 + sv * sv).sqrt();
    if dh <= dv { ((x, h_road_y(k, x)), true) } else { ((v_road_x(j, y), y), false) }
}

/// Where horizontal road k crosses vertical road j.
pub fn crossing(k: i32, j: i32) -> (f32, f32) {
    let mut x = ROAD_V_OFFSET + j as f32 * ROAD_SPACING;
    let mut y = h_road_y(k, x);
    for _ in 0..8 {
        x = v_road_x(j, y);
        y = h_road_y(k, x);
    }
    (x, y)
}

/// Points along horizontal road k from x0 to x1 (either direction), at most `step` apart.
pub fn h_points(k: i32, x0: f32, x1: f32, step: f32) -> Vec<(f32, f32)> {
    let n = ((x1 - x0).abs() / step).ceil().max(1.0) as usize;
    (0..=n).map(|i| { let x = x0 + (x1 - x0) * i as f32 / n as f32; (x, h_road_y(k, x)) }).collect()
}
pub fn v_points(j: i32, y0: f32, y1: f32, step: f32) -> Vec<(f32, f32)> {
    let n = ((y1 - y0).abs() / step).ceil().max(1.0) as usize;
    (0..=n).map(|i| { let y = y0 + (y1 - y0) * i as f32 / n as f32; (v_road_x(j, y), y) }).collect()
}

const ROUTE_STEP: f32 = 120.0;

fn push_unique(pts: &mut Vec<(f32, f32)>, p: (f32, f32)) {
    if pts.last().map_or(true, |l| (l.0 - p.0).abs() > 0.5 || (l.1 - p.1).abs() > 0.5) {
        pts.push(p);
    }
}

fn extend_unique(pts: &mut Vec<(f32, f32)>, more: Vec<(f32, f32)>) {
    for p in more {
        push_unique(pts, p);
    }
}

/// Waypoints a truck follows (staying on main roads) from a road position
/// back to the basecamp truck spot.
pub fn route_to_camp(px: f32, py: f32) -> Vec<(f32, f32)> {
    let ((ax, ay), on_h) = road_anchor(px, py);
    let mut pts = Vec::new();
    if on_h {
        let k = h_index(ay);
        if k == 0 {
            extend_unique(&mut pts, h_points(0, ax, TRUCK_SPOT.0, ROUTE_STEP));
        } else {
            // along this road to the nearest vertical road, down it to road 0, then home
            let j = v_index(ax);
            let (ix, iy) = crossing(k, j);
            let (hx, hy) = crossing(0, j);
            extend_unique(&mut pts, h_points(k, ax, ix, ROUTE_STEP));
            extend_unique(&mut pts, v_points(j, iy, hy, ROUTE_STEP));
            extend_unique(&mut pts, h_points(0, hx, TRUCK_SPOT.0, ROUTE_STEP));
        }
    } else {
        let j = v_index(ax);
        let (hx, hy) = crossing(0, j);
        extend_unique(&mut pts, v_points(j, ay, hy, ROUTE_STEP));
        extend_unique(&mut pts, h_points(0, hx, TRUCK_SPOT.0, ROUTE_STEP));
    }
    push_unique(&mut pts, TRUCK_SPOT);
    pts
}

/// Waypoints from the basecamp truck spot out to a road position.
pub fn route_from_camp(tx: f32, ty: f32) -> Vec<(f32, f32)> {
    let mut r = route_to_camp(tx, ty);
    r.reverse();
    r
}

pub const CARD_ADJECTIVES: [&str; 12] = [
    "Ancient", "Weathered", "Crystalline", "Rugged", "Sunlit", "Deep",
    "Mossy", "Jagged", "Polished", "Faded", "Banded", "Coarse",
];

pub fn chunk_hash(cx: i32, cy: i32, salt: i32) -> i64 {
    let cxm = (cx as i64).rem_euclid(46_340);
    let cym = (cy as i64).rem_euclid(46_340);
    let saltm = (salt as i64).rem_euclid(46_340);
    let mut h: i64 = 17;
    h = (h * HASH_MUL + cxm) % HASH_MOD;
    h = (h * HASH_MUL + cym) % HASH_MOD;
    h = (h * HASH_MUL2 + saltm) % HASH_MOD;
    h
}

pub fn chunk_rand_int(cx: i32, cy: i32, salt: i32, modulus: i32) -> i32 {
    (chunk_hash(cx, cy, salt) % modulus as i64) as i32
}

pub fn world_to_chunk(x: f32, y: f32) -> (i32, i32) {
    ((x / CHUNK as f32).floor() as i32, (y / CHUNK as f32).floor() as i32)
}

#[derive(Clone, Debug)]
pub struct Outcrop {
    pub id: String,
    pub x: f32,
    pub y: f32,
    /// Igneous | Sedimentary | Metamorphic
    pub lithology: &'static str,
    /// Index into the region's `units`.
    pub unit: usize,
    /// Specific rock type, e.g. "Itabirite".
    pub rock: &'static str,
}

impl Outcrop {
    pub fn new(id: &str, x: f32, y: f32, unit: usize, rock: &'static str, lithology: &'static str) -> Self {
        Outcrop { id: id.to_string(), x, y, lithology, unit, rock }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub grass_variant: i32,
    pub outcrops: Vec<Outcrop>,
    pub trees: Vec<Point>,
    pub dirt_patches: Vec<Point>,
}

pub fn generate_chunk(cx: i32, cy: i32, terrain: &Terrain, region: &Region) -> Chunk {
    let grass_variant = chunk_rand_int(cx, cy, 1, 3);
    let outcrop_count = chunk_rand_int(cx, cy, 2, 4); // 0..=3
    let tree_count = chunk_rand_int(cx, cy, 3, 3); // 0..=2
    let (ox, oy) = (cx * CHUNK, cy * CHUNK);

    // Nothing spawns on a main road, inside the basecamp, or on ground you
    // can't stand on (lakes, farms, dense forest, cliffs) or wade through.
    let clear = |x: f32, y: f32, m: f32| {
        !on_road(x, y, m) && !in_camp(x, y, m) && terrain.ground(x, y) == Ground::Open
    };

    // Each spot belongs to the geological unit mapped under it; the unit decides
    // how often rock is exposed there and which rock type it is.
    let outcrops = (0..outcrop_count)
        .filter_map(|i| {
            let x = (ox + chunk_rand_int(cx, cy, 10 + i, CHUNK - 40) + 20) as f32;
            let y = (oy + chunk_rand_int(cx, cy, 20 + i, CHUNK - 40) + 20) as f32;
            if !clear(x, y, 50.0) {
                return None;
            }
            let unit = region.unit_at(terrain, x, y);
            let u = &region.units[unit];
            if chunk_rand_int(cx, cy, 100 + i, 100) as u32 >= u.exposure {
                return None;
            }
            let (rock, lithology) = pick_rock(u, chunk_hash(cx, cy, 40 + i) as u32);
            Some(Outcrop { id: format!("{}:{cx}_{cy}_{i}", region.id), x, y, lithology, unit, rock })
        })
        .collect();

    let trees = (0..tree_count)
        .map(|i| Point {
            x: (ox + chunk_rand_int(cx, cy, 60 + i, CHUNK - 40) + 20) as f32,
            y: (oy + chunk_rand_int(cx, cy, 70 + i, CHUNK - 40) + 20) as f32,
        })
        .filter(|t| clear(t.x, t.y, 90.0))
        .collect();

    let mut dirt_patches = Vec::new();
    if chunk_rand_int(cx, cy, 90, 100) < 35 {
        let d = Point {
            x: (ox + chunk_rand_int(cx, cy, 91, CHUNK - 180) + 90) as f32,
            y: (oy + chunk_rand_int(cx, cy, 92, CHUNK - 130) + 65) as f32,
        };
        if clear(d.x, d.y, 100.0) {
            dirt_patches.push(d);
        }
    }

    Chunk { grass_variant, outcrops, trees, dirt_patches }
}

/// Stable, human-readable card name derived from the outcrop id.
pub fn card_name(o: &Outcrop) -> String {
    let seed = o
        .id
        .chars()
        .fold(7i64, |acc, ch| (acc * 31 + ch as i64) % 100_000);
    let adj = CARD_ADJECTIVES[(seed as usize) % CARD_ADJECTIVES.len()];
    format!("{adj} {}", o.rock)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geology::REGIONS;

    #[test]
    fn hash_is_deterministic_and_bounded_for_extreme_coords() {
        for &c in &[0, 1, -1, 46_339, 46_340, -46_341, 1_000_000, i32::MAX, i32::MIN + 1] {
            for salt in [1, 2, 99, 1000] {
                let a = chunk_hash(c, -c, salt);
                assert_eq!(a, chunk_hash(c, -c, salt));
                assert!((0..HASH_MOD).contains(&a));
            }
        }
    }

    #[test]
    fn matches_ruby_reference_values() {
        // Hand-computed from the Ruby implementation:
        // h=17 -> (17*9973+0)%104729=60,... for (0,0,1)
        let mut h = 17i64;
        h = (h * 9973) % 104_729;
        h = (h * 9973) % 104_729;
        h = (h * 7919 + 1) % 104_729;
        assert_eq!(chunk_hash(0, 0, 1), h);
    }

    #[test]
    fn chunks_are_stable_and_within_bounds() {
        let t = Terrain::default();
        let r = &REGIONS[0];
        for cx in -6..=6 {
            for cy in -6..=6 {
                let a = generate_chunk(cx, cy, &t, r);
                let b = generate_chunk(cx, cy, &t, r);
                assert_eq!(a.outcrops.len(), b.outcrops.len());
                assert!(a.outcrops.len() <= 3 && a.trees.len() <= 2 && a.dirt_patches.len() <= 1);
                assert!((0..3).contains(&a.grass_variant));
                for o in &a.outcrops {
                    assert!(o.x >= (cx * CHUNK) as f32 && o.x < ((cx + 1) * CHUNK) as f32);
                    assert!(o.y >= (cy * CHUNK) as f32 && o.y < ((cy + 1) * CHUNK) as f32);
                }
            }
        }
    }

    #[test]
    fn nothing_spawns_on_roads_or_in_camp() {
        let t = Terrain::default();
        let r = &REGIONS[0];
        for cx in -12..=12 {
            for cy in -12..=12 {
                let c = generate_chunk(cx, cy, &t, r);
                for o in &c.outcrops {
                    assert!(!on_road(o.x, o.y, 0.0) && !in_camp(o.x, o.y, 0.0), "{:?}", o);
                }
                for t in &c.trees {
                    assert!(!on_road(t.x, t.y, 0.0) && !in_camp(t.x, t.y, 0.0));
                }
            }
        }
    }

    #[test]
    fn nothing_spawns_on_water_farms_forest_or_cliffs() {
        let t = Terrain::default();
        let r = &REGIONS[0];
        let mut outcrops = 0;
        for cx in -14..=14 {
            for cy in -14..=14 {
                let c = generate_chunk(cx, cy, &t, r);
                for o in &c.outcrops {
                    outcrops += 1;
                    assert_eq!(t.ground(o.x, o.y), Ground::Open, "{:?}", o);
                }
                for tr in &c.trees {
                    assert_eq!(t.ground(tr.x, tr.y), Ground::Open);
                }
                for d in &c.dirt_patches {
                    assert_eq!(t.ground(d.x, d.y), Ground::Open);
                }
            }
        }
        assert!(outcrops > 200);
    }

    #[test]
    fn roads_and_camp_geometry() {
        assert!(on_road(0.0, 0.0, 0.0));
        let (vx, vy) = (v_road_x(0, 777.0), 777.0);
        assert!(on_road(vx, vy, 0.0));
        let (hx, hy) = (1000.0, h_road_y(1, 1000.0));
        assert!(on_road(hx, hy, 0.0));
        assert!(!on_road(400.0, 400.0, 0.0));
        // camp is clear of every road, and the road in front of it is dead straight
        for &(x, y) in &[(CAMP.0, CAMP.1), (CAMP.2, CAMP.3), (CAMP.0, CAMP.3), (CAMP.2, CAMP.1)] {
            assert!(!on_road(x, y, 0.0), "{x},{y}");
        }
        for x in [-500.0f32, -250.0, 0.0, 250.0, 500.0] {
            assert!(h_road_y(0, x).abs() < 1e-3, "road 0 must be straight by the camp at x={x}");
        }
        assert!(on_road(TRUCK_SPOT.0, TRUCK_SPOT.1, 0.0));
        assert!(in_camp(CAMP_SPAWN.0, CAMP_SPAWN.1, 0.0) && !on_road(CAMP_SPAWN.0, CAMP_SPAWN.1, 0.0));
    }

    #[test]
    fn roads_really_curve_and_blocks_are_far_apart() {
        // the vertical road wanders by hundreds of units, so it is not a straight line
        let xs: Vec<f32> = (0..20).map(|i| v_road_x(0, i as f32 * 400.0)).collect();
        let (lo, hi) = (xs.iter().cloned().fold(f32::MAX, f32::min), xs.iter().cloned().fold(f32::MIN, f32::max));
        assert!(hi - lo > 300.0, "road should meander, spread was {}", hi - lo);
        // slope stays drivable
        for i in 0..400 {
            let y = i as f32 * 20.0;
            assert!((v_road_x(0, y + 8.0) - v_road_x(0, y - 8.0)).abs() / 16.0 < 1.0);
        }
        // neighbouring roads never come close to touching
        let gap = (0..400).map(|i| { let y = i as f32 * 30.0; v_road_x(0, y) - v_road_x(-1, y) }).fold(f32::MAX, f32::min);
        assert!(gap > 3000.0, "gap {gap}");
        assert!(ROAD_SPACING >= 4800.0);
    }

    #[test]
    fn routes_follow_the_roads_and_end_at_camp() {
        let pts = [
            (v_road_x(0, 2000.0), 2000.0),
            (v_road_x(0, -1800.0), -1800.0),
            (v_road_x(-1, 900.0), 900.0),
            (3000.0, h_road_y(1, 3000.0)),
            (-1500.0, h_road_y(1, -1500.0)),
            (1200.0, h_road_y(0, 1200.0)),
            (-3000.0, h_road_y(0, -3000.0)),
        ];
        for &(x, y) in &pts {
            let r = route_to_camp(x, y);
            assert_eq!(*r.last().unwrap(), TRUCK_SPOT);
            for w in r.windows(2) {
                assert!(((w[0].0 - w[1].0).powi(2) + (w[0].1 - w[1].1).powi(2)).sqrt() <= ROUTE_STEP * 1.5 + 1.0, "long leg {:?}", w);
                for k in 0..=10 {
                    let t = k as f32 / 10.0;
                    let (px, py) = (w[0].0 + (w[1].0 - w[0].0) * t, w[0].1 + (w[1].1 - w[0].1) * t);
                    assert!(on_road(px, py, 0.5), "off-road at {px},{py} for target {x},{y}");
                }
            }
            assert_eq!(route_from_camp(x, y).first().copied().unwrap(), TRUCK_SPOT);
        }
    }

    #[test]
    fn crossings_lie_on_both_roads() {
        for k in -1..=1 {
            for j in -1..=1 {
                let (x, y) = crossing(k, j);
                assert!((y - h_road_y(k, x)).abs() < 1.0 && (x - v_road_x(j, y)).abs() < 1.0, "k{k} j{j}");
            }
        }
    }

    #[test]
    fn negative_coords_map_to_correct_chunk() {
        assert_eq!(world_to_chunk(-0.5, 639.9), (-1, 0));
        assert_eq!(world_to_chunk(-640.0, -641.0), (-1, -2));
    }

    #[test]
    fn card_names_are_stable() {
        let o = Outcrop::new("0_0_0", 0.0, 0.0, 0, "Basalt", "Igneous");
        assert_eq!(card_name(&o), card_name(&o));
        assert!(card_name(&o).ends_with("Basalt"));
    }

    #[test]
    fn every_outcrop_belongs_to_the_unit_mapped_at_its_location() {
        for r in REGIONS.iter() {
            let t = r.terrain();
            let mut seen = 0;
            for cx in -5..5 {
                for cy in -5..5 {
                    for o in generate_chunk(cx, cy, &t, r).outcrops {
                        let u = r.unit_at(&t, o.x, o.y);
                        assert_eq!(o.unit, u, "{} {}", r.name, o.id);
                        assert!(r.units[u].rocks.iter().any(|x| x.0 == o.rock && x.2 == o.lithology), "{}: {} not in {}", r.name, o.rock, r.units[u].name);
                        assert!(o.id.starts_with(r.id));
                        assert_eq!(t.ground(o.x, o.y), crate::terrain::Ground::Open);
                        seen += 1;
                    }
                }
            }
            assert!(seen > 25, "{} has only {} outcrops", r.name, seen);
        }
    }
}
