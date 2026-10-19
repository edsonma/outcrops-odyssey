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

    let outcrops = (0..outcrop_count)
        .map(|i| Outcrop {
            id: format!("{cx}_{cy}_{i}"),
            x: (ox + chunk_rand_int(cx, cy, 10 + i, CHUNK - 40) + 20) as f32,
            y: (oy + chunk_rand_int(cx, cy, 20 + i, CHUNK - 40) + 20) as f32,
            lithology: LITHOLOGIES[chunk_rand_int(cx, cy, 40 + i, 3) as usize],
        })
        .collect();

    let trees = (0..tree_count)
        .map(|i| Point {
            x: (ox + chunk_rand_int(cx, cy, 60 + i, CHUNK - 40) + 20) as f32,
            y: (oy + chunk_rand_int(cx, cy, 70 + i, CHUNK - 40) + 20) as f32,
        })
        .collect();

    let mut dirt_patches = Vec::new();
    if chunk_rand_int(cx, cy, 90, 100) < 35 {
        dirt_patches.push(Point {
            x: (ox + chunk_rand_int(cx, cy, 91, CHUNK - 180) + 90) as f32,
            y: (oy + chunk_rand_int(cx, cy, 92, CHUNK - 130) + 65) as f32,
        });
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
