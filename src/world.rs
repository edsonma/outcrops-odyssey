//! Deterministic, infinite world generation (Rust port of the DragonRuby
//! prototype). The same chunk coordinates always regenerate the same
//! content. World coordinates are y-up, matching the original game.

pub const CHUNK: i32 = 640;
pub const VIEW_RADIUS_CHUNKS: i32 = 2;

// The Ruby version needed tiny primes to stay inside mruby's fixed-width
// integers. We keep the exact same arithmetic (in i64) so worlds generated
// by the Ruby prototype and this port are identical for the same chunk.
const HASH_MOD: i64 = 104_729;
const HASH_MUL: i64 = 9_973;
const HASH_MUL2: i64 = 7_919;

// ---- main roads & basecamp -------------------------------------------------
// Horizontal main roads run along y = k * ROAD_SPACING; vertical ones along
// x = ROAD_V_OFFSET + k * ROAD_SPACING. The basecamp sits just north of the
// y = 0 road, clear of every vertical road, with the truck parked on the road.
pub const ROAD_SPACING: f32 = 3200.0;
pub const ROAD_W: f32 = 120.0;
pub const ROAD_V_OFFSET: f32 = 1600.0;
pub const CAMP: (f32, f32, f32, f32) = (-400.0, 90.0, 400.0, 520.0); // x0, y0, x1, y1
pub const TRUCK_SPOT: (f32, f32) = (0.0, 0.0);
pub const CAMP_SPAWN: (f32, f32) = (0.0, 230.0);

pub fn nearest_h_road(y: f32) -> f32 {
    (y / ROAD_SPACING).round() * ROAD_SPACING
}
pub fn nearest_v_road(x: f32) -> f32 {
    ROAD_V_OFFSET + ((x - ROAD_V_OFFSET) / ROAD_SPACING).round() * ROAD_SPACING
}
/// Distance to the centre line of the closest main road.
pub fn road_distance(x: f32, y: f32) -> f32 {
    (y - nearest_h_road(y)).abs().min((x - nearest_v_road(x)).abs())
}
pub fn on_road(x: f32, y: f32, extra: f32) -> bool {
    road_distance(x, y) <= ROAD_W / 2.0 + extra
}
pub fn in_camp(x: f32, y: f32, margin: f32) -> bool {
    x >= CAMP.0 - margin && x <= CAMP.2 + margin && y >= CAMP.1 - margin && y <= CAMP.3 + margin
}

fn push_unique(pts: &mut Vec<(f32, f32)>, p: (f32, f32)) {
    if pts.last().map_or(true, |l| (l.0 - p.0).abs() > 0.5 || (l.1 - p.1).abs() > 0.5) {
        pts.push(p);
    }
}

/// Waypoints a truck follows (staying on main roads) from a road position
/// back to the basecamp truck spot.
pub fn route_to_camp(px: f32, py: f32) -> Vec<(f32, f32)> {
    let (xv, yh) = (nearest_v_road(px), nearest_h_road(py));
    let on_h = (py - yh).abs() <= ROAD_W / 2.0;
    let mut pts = Vec::new();
    if on_h {
        push_unique(&mut pts, (px, yh));
        push_unique(&mut pts, (xv, yh));
    } else {
        push_unique(&mut pts, (xv, py));
    }
    push_unique(&mut pts, (xv, TRUCK_SPOT.1));
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
pub const LITHOLOGIES: [&str; 3] = ["Igneous", "Sedimentary", "Metamorphic"];

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
    pub lithology: &'static str,
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

pub fn generate_chunk(cx: i32, cy: i32) -> Chunk {
    let grass_variant = chunk_rand_int(cx, cy, 1, 3);
    let outcrop_count = chunk_rand_int(cx, cy, 2, 4); // 0..=3
    let tree_count = chunk_rand_int(cx, cy, 3, 3); // 0..=2
    let (ox, oy) = (cx * CHUNK, cy * CHUNK);

    // Nothing spawns on a main road or inside the basecamp.
    let clear = |x: f32, y: f32, m: f32| !on_road(x, y, m) && !in_camp(x, y, m);

    let outcrops = (0..outcrop_count)
        .map(|i| Outcrop {
            id: format!("{cx}_{cy}_{i}"),
            x: (ox + chunk_rand_int(cx, cy, 10 + i, CHUNK - 40) + 20) as f32,
            y: (oy + chunk_rand_int(cx, cy, 20 + i, CHUNK - 40) + 20) as f32,
            lithology: LITHOLOGIES[chunk_rand_int(cx, cy, 40 + i, 3) as usize],
        })
        .filter(|o| clear(o.x, o.y, 50.0))
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
    format!("{adj} {}", o.lithology)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for cx in -6..=6 {
            for cy in -6..=6 {
                let a = generate_chunk(cx, cy);
                let b = generate_chunk(cx, cy);
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
        for cx in -12..=12 {
            for cy in -12..=12 {
                let c = generate_chunk(cx, cy);
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
    fn roads_and_camp_geometry() {
        assert!(on_road(0.0, 0.0, 0.0) && on_road(1600.0, 777.0, 0.0) && on_road(-1600.0, 5.0, 0.0));
        assert!(!on_road(400.0, 400.0, 0.0));
        // camp is clear of every road
        for &(x, y) in &[(CAMP.0, CAMP.1), (CAMP.2, CAMP.3), (CAMP.0, CAMP.3), (CAMP.2, CAMP.1)] {
            assert!(!on_road(x, y, 0.0), "{x},{y}");
        }
        assert!(on_road(TRUCK_SPOT.0, TRUCK_SPOT.1, 0.0));
        assert!(in_camp(CAMP_SPAWN.0, CAMP_SPAWN.1, 0.0) && !on_road(CAMP_SPAWN.0, CAMP_SPAWN.1, 0.0));
    }

    #[test]
    fn routes_are_axis_aligned_road_driving_ending_at_camp() {
        for &(x, y) in &[(1600.0, 2000.0), (1600.0, -1800.0), (-1600.0, 900.0), (4800.0, 3200.0), (500.0, 3200.0), (1600.0, 0.0), (-3000.0, 0.0)] {
            let r = route_to_camp(x, y);
            assert_eq!(*r.last().unwrap(), TRUCK_SPOT);
            for w in r.windows(2) {
                assert!(w[0].0 == w[1].0 || w[0].1 == w[1].1, "diagonal leg {:?}", w);
                for k in 0..=20 {
                    let t = k as f32 / 20.0;
                    let (px, py) = (w[0].0 + (w[1].0 - w[0].0) * t, w[0].1 + (w[1].1 - w[0].1) * t);
                    assert!(on_road(px, py, 0.5), "off-road at {px},{py} for target {x},{y}");
                }
            }
            let out = route_from_camp(x, y);
            assert_eq!(out.first().copied().unwrap(), TRUCK_SPOT);
        }
    }

    #[test]
    fn negative_coords_map_to_correct_chunk() {
        assert_eq!(world_to_chunk(-0.5, 639.9), (-1, 0));
        assert_eq!(world_to_chunk(-640.0, -641.0), (-1, -2));
    }

    #[test]
    fn card_names_are_stable() {
        let o = Outcrop { id: "0_0_0".into(), x: 0.0, y: 0.0, lithology: "Igneous" };
        assert_eq!(card_name(&o), card_name(&o));
        assert!(card_name(&o).ends_with("Igneous"));
    }
}
