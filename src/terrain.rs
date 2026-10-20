//! Deterministic terrain: elevation (for contour lines and slopes), water,
//! fenced farms, dense forest and cliffs. Pure logic, no rendering.
//!
//! Everything is a function of the world position and the terrain's seed, so
//! the same place always looks the same. Main roads and the basecamp sit on a
//! flat, obstacle-free corridor so the truck routes and the drop points stay
//! valid whatever the surrounding relief is.

use crate::world::{road_distance, CAMP, ROAD_W};
#[cfg(test)]
use crate::world::{h_road_y, v_road_x};

/// Elevation range in metres (1 world unit = 1 m for slope maths).
pub const ELEV_MAX: f32 = 300.0;
pub const WATER_LEVEL: f32 = 58.0;
pub const PEAK_LEVEL: f32 = 228.0;
/// Steeper than this (rise/run) is a cliff: impassable on foot.
pub const CLIFF_GRADE: f32 = 0.62;
/// Elevation of the flat corridor around roads and camp.
pub const BASE_ELEV: f32 = 130.0;
pub const CONTOUR_INTERVAL: f32 = 15.0;
/// Every Nth contour is drawn heavier (an "index contour").
pub const INDEX_EVERY: i32 = 5;
/// No obstacle of any kind spawns this close to a road edge or the camp.
pub const CLEAR_RADIUS: f32 = 260.0;
pub const FARM_CELL: f32 = 1200.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ground {
    Open,
    Lake,
    /// Shallow water: can be waded (slowly) and refills the bottle.
    River,
    Farm,
    Forest,
    Cliff,
    /// Outside the surveyed map rectangle.
    Edge,
}

impl Ground {
    pub fn blocked(self) -> bool {
        matches!(self, Ground::Lake | Ground::Farm | Ground::Forest | Ground::Cliff | Ground::Edge)
    }
    pub fn is_water(self) -> bool {
        matches!(self, Ground::Lake | Ground::River)
    }
    /// Speed multiplier from the surface itself (before slope).
    pub fn surface_speed(self) -> f32 {
        if self == Ground::River { 0.55 } else { 1.0 }
    }
    pub fn block_message(self) -> &'static str {
        match self {
            Ground::Lake => "Lake - too deep to cross",
            Ground::Farm => "Fenced farm - private property",
            Ground::Forest => "Dense forest - impassable",
            Ground::Cliff => "Too steep to climb",
            Ground::Edge => "Edge of the survey area",
            _ => "",
        }
    }
}

/// Speed multiplier for walking along a slope. `grade` is rise/run along the
/// direction of travel: positive = uphill.
pub fn speed_factor(grade: f32) -> f32 {
    if grade >= 0.0 {
        (1.0 / (1.0 + 2.4 * grade)).max(0.3)
    } else {
        (1.0 + 0.8 * -grade).min(1.35)
    }
}

/// Energy cost multiplier per unit of effort on a slope.
pub fn effort_factor(grade: f32) -> f32 {
    if grade >= 0.0 {
        (1.0 + 4.0 * grade).min(3.0)
    } else {
        (1.0 + 0.9 * grade).max(0.5)
    }
}

#[derive(Clone, Debug)]
pub struct Terrain {
    pub seed: u32,
    /// A flat, obstacle-free world (used by tests and the basecamp tutorial).
    pub flat: bool,
    /// Relief multiplier: 0.5 gentle plateau .. 1.5 rugged mountains.
    pub relief: f32,
    pub water_level: f32,
    /// Percent chance that a farm cell actually holds a farm.
    pub farm_pct: u32,
    /// Forest noise threshold; lower means more dense forest.
    pub forest_threshold: f32,
    /// Half-width of the river band in noise units (bigger = wider rivers).
    pub river_band: f32,
    /// Playable rectangle (x0, y0, x1, y1); beyond it is `Ground::Edge`.
    pub bounds: Option<(f32, f32, f32, f32)>,
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain { seed: 0x0DDC_0FFE, flat: false, relief: 1.0, water_level: WATER_LEVEL, farm_pct: 90, forest_threshold: 0.60, river_band: 0.010, bounds: None }
    }
}

// ---- value noise ---------------------------------------------------------

fn lattice(seed: u32, ix: i32, iy: i32) -> f32 {
    let mut h = seed ^ (ix as u32).wrapping_mul(0x85EB_CA6B) ^ (iy as u32).wrapping_mul(0xC2B2_AE35);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0xFFFF) as f32 / 65535.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(seed: u32, x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (smooth(x - x0), smooth(y - y0));
    let (ix, iy) = (x0 as i32, y0 as i32);
    let a = lattice(seed, ix, iy);
    let b = lattice(seed, ix + 1, iy);
    let c = lattice(seed, ix, iy + 1);
    let d = lattice(seed, ix + 1, iy + 1);
    let top = a + (b - a) * fx;
    let bot = c + (d - c) * fx;
    top + (bot - top) * fy
}

/// Fractal noise in 0..1 (centred on 0.5).
pub fn fbm(seed: u32, x: f32, y: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut freq) = (0.0, 1.0, 0.0, 1.0);
    for o in 0..octaves {
        sum += amp * value_noise(seed.wrapping_add(o * 977), x * freq, y * freq);
        norm += amp;
        amp *= 0.4;
        freq *= 2.0;
    }
    sum / norm
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn camp_distance(x: f32, y: f32) -> f32 {
    let dx = (CAMP.0 - x).max(0.0).max(x - CAMP.2);
    let dy = (CAMP.1 - y).max(0.0).max(y - CAMP.3);
    (dx * dx + dy * dy).sqrt()
}

/// Distance to the nearest road edge or the camp (0 when on either).
pub fn clear_distance(x: f32, y: f32) -> f32 {
    (road_distance(x, y) - ROAD_W / 2.0).min(camp_distance(x, y)).max(0.0)
}

impl Terrain {
    pub fn flat() -> Self {
        Terrain { flat: true, ..Terrain::default() }
    }

    /// Elevation in metres, 0..=ELEV_MAX. Flattened towards BASE_ELEV near
    /// roads and the camp.
    pub fn elevation(&self, x: f32, y: f32) -> f32 {
        if self.flat {
            return BASE_ELEV;
        }
        let n = fbm(self.seed, x / 3000.0, y / 3000.0, 4);
        // soft saturation keeps summits and basins from becoming flat slabs
        let raw = (0.5 + 0.5 * ((n - 0.5) * self.relief * 6.0).tanh()) * ELEV_MAX;
        // Relief is damped (not removed) near roads and camp, so the corridor
        // rolls gently. The distance is domain-warped so the damping zone has
        // organic outlines instead of rings that follow the road grid.
        let wx = x + (fbm(self.seed ^ 0x5EED_0001, x / 900.0, y / 900.0, 2) - 0.5) * 1400.0;
        let wy = y + (fbm(self.seed ^ 0x5EED_0002, x / 900.0, y / 900.0, 2) - 0.5) * 1400.0;
        let amp = 0.2 + 0.8 * smoothstep(CLEAR_RADIUS * 0.5, CLEAR_RADIUS * 4.5, clear_distance(wx, wy));
        BASE_ELEV + (raw - BASE_ELEV) * amp
    }

    /// Terrain gradient magnitude (rise/run).
    pub fn steepness(&self, x: f32, y: f32) -> f32 {
        if self.flat {
            return 0.0;
        }
        let h = 10.0;
        let gx = (self.elevation(x + h, y) - self.elevation(x - h, y)) / (2.0 * h);
        let gy = (self.elevation(x, y + h) - self.elevation(x, y - h)) / (2.0 * h);
        (gx * gx + gy * gy).sqrt()
    }

    /// Rise/run when heading along the unit vector (ux, uy): positive = uphill.
    pub fn grade_toward(&self, x: f32, y: f32, ux: f32, uy: f32) -> f32 {
        if self.flat {
            return 0.0;
        }
        let h = 8.0;
        (self.elevation(x + ux * h, y + uy * h) - self.elevation(x - ux * h, y - uy * h)) / (2.0 * h)
    }

    fn river_at(&self, x: f32, y: f32) -> bool {
        let n = fbm(self.seed ^ 0xA11C_E5ED, x / 1300.0, y / 1300.0, 2);
        (n - 0.5).abs() < self.river_band
    }

    fn forest_noise(&self, x: f32, y: f32) -> f32 {
        fbm(self.seed ^ 0x0F0E_57ED, x / 700.0, y / 700.0, 3)
    }

    /// The fenced farm rectangle (x0, y0, x1, y1) in a farm cell, if any.
    pub fn farm_rect(&self, ci: i32, cj: i32) -> Option<(f32, f32, f32, f32)> {
        if self.flat {
            return None;
        }
        let r = |salt: u32| lattice(self.seed ^ 0xFA43_0000 ^ salt, ci, cj);
        if (r(1) * 100.0) as u32 >= self.farm_pct {
            return None;
        }
        let w = 460.0 + r(2) * 360.0;
        let h = 380.0 + r(3) * 340.0;
        let x0 = ci as f32 * FARM_CELL + 90.0 + r(4) * (FARM_CELL - 180.0 - w);
        let y0 = cj as f32 * FARM_CELL + 90.0 + r(5) * (FARM_CELL - 180.0 - h);
        let (x1, y1) = (x0 + w, y0 + h);
        // keep away from roads and camp: test the rectangle's centre and corners
        let pts = [((x0 + x1) / 2.0, (y0 + y1) / 2.0), (x0, y0), (x1, y0), (x0, y1), (x1, y1)];
        if pts.iter().any(|&(px, py)| clear_distance(px, py) < CLEAR_RADIUS) {
            return None;
        }
        // farms need gentle, dry ground across the whole plot
        let (mx, my) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let probes = [(mx, my), (x0, y0), (x1, y0), (x0, y1), (x1, y1), (mx, y0), (mx, y1), (x0, my), (x1, my)];
        for &(px, py) in &probes {
            let e = self.elevation(px, py);
            if e < self.water_level + 15.0 || e > PEAK_LEVEL - 70.0 || self.steepness(px, py) > 0.26 {
                return None;
            }
        }
        Some((x0, y0, x1, y1))
    }

    fn in_farm(&self, x: f32, y: f32) -> bool {
        let (ci, cj) = ((x / FARM_CELL).floor() as i32, (y / FARM_CELL).floor() as i32);
        self.farm_rect(ci, cj).is_some_and(|(x0, y0, x1, y1)| x >= x0 && x <= x1 && y >= y0 && y <= y1)
    }

    /// True beyond the playable rectangle (always false for unbounded terrain).
    pub fn outside(&self, x: f32, y: f32) -> bool {
        self.bounds.is_some_and(|(x0, y0, x1, y1)| x < x0 || x > x1 || y < y0 || y > y1)
    }

    pub fn ground(&self, x: f32, y: f32) -> Ground {
        if self.outside(x, y) {
            return Ground::Edge;
        }
        if self.flat || clear_distance(x, y) < CLEAR_RADIUS {
            return Ground::Open;
        }
        let e = self.elevation(x, y);
        let steep = if e < self.water_level || e > PEAK_LEVEL { 0.0 } else { self.steepness(x, y) };
        self.classify(x, y, e, steep)
    }

    /// Ground type from an already-known elevation and steepness at (x, y).
    /// Lets the renderer classify a whole grid from one elevation sample each.
    pub fn classify(&self, x: f32, y: f32, e: f32, steep: f32) -> Ground {
        if self.outside(x, y) {
            return Ground::Edge;
        }
        if self.flat || clear_distance(x, y) < CLEAR_RADIUS {
            return Ground::Open;
        }
        if e < self.water_level {
            return Ground::Lake;
        }
        if e > PEAK_LEVEL || steep > CLIFF_GRADE {
            return Ground::Cliff;
        }
        if self.in_farm(x, y) {
            return Ground::Farm;
        }
        if self.river_at(x, y) && e < PEAK_LEVEL - 60.0 {
            return Ground::River;
        }
        if self.forest_noise(x, y) > self.forest_threshold {
            return Ground::Forest;
        }
        Ground::Open
    }

    pub fn blocked(&self, x: f32, y: f32) -> bool {
        self.ground(x, y).blocked()
    }
}

/// Marching squares for one contour level inside a unit cell. `v` are the
/// corner values in the order bottom-left, bottom-right, top-right, top-left.
/// Returns up to two segments in cell coordinates (0..1).
pub fn contour_segments(v: [f32; 4], level: f32) -> Vec<((f32, f32), (f32, f32))> {
    let idx = (v[0] >= level) as u8 | ((v[1] >= level) as u8) << 1 | ((v[2] >= level) as u8) << 2 | ((v[3] >= level) as u8) << 3;
    if idx == 0 || idx == 15 {
        return Vec::new();
    }
    let t = |a: f32, b: f32| if (b - a).abs() < 1e-6 { 0.5 } else { ((level - a) / (b - a)).clamp(0.0, 1.0) };
    let bottom = (t(v[0], v[1]), 0.0);
    let right = (1.0, t(v[1], v[2]));
    let top = (t(v[3], v[2]), 1.0);
    let left = (0.0, t(v[0], v[3]));
    match idx {
        1 | 14 => vec![(left, bottom)],
        2 | 13 => vec![(bottom, right)],
        3 | 12 => vec![(left, right)],
        4 | 11 => vec![(right, top)],
        6 | 9 => vec![(bottom, top)],
        7 | 8 => vec![(left, top)],
        5 => vec![(left, bottom), (right, top)],
        10 => vec![(bottom, right), (left, top)],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{in_camp, on_road, TRUCK_SPOT};

    fn sample_grid(t: &Terrain, half: i32, step: f32) -> Vec<(f32, f32)> {
        let mut pts = Vec::new();
        for i in -half..=half {
            for j in -half..=half {
                pts.push((i as f32 * step, j as f32 * step));
            }
        }
        pts
    }

    #[test]
    fn elevation_is_deterministic_and_bounded() {
        let t = Terrain::default();
        for (x, y) in sample_grid(&t, 40, 300.0) {
            let e = t.elevation(x, y);
            assert_eq!(e, t.elevation(x, y));
            assert!((0.0..=ELEV_MAX).contains(&e), "{e} at {x},{y}");
        }
    }

    #[test]
    fn roads_and_camp_are_flat_and_obstacle_free() {
        let t = Terrain::default();
        for &(x, y) in &[(0.0, 0.0), (v_road_x(0, 900.0), 900.0), (v_road_x(0, -2200.0), -2200.0), (v_road_x(-1, 3200.0), 3200.0), (0.0, 300.0), (-350.0, 450.0)] {
            assert!(!t.blocked(x, y), "{x},{y}");
            assert!(t.steepness(x, y) < 0.15, "{x},{y} steep {}", t.steepness(x, y));
        }
        // sample along roads and the camp
        for k in -40..=40 {
            let s = k as f32 * 150.0;
            assert!(!t.blocked(s, h_road_y(0, s)) && !t.blocked(v_road_x(0, s), s) && !t.blocked(v_road_x(-1, s), s) && !t.blocked(s, h_road_y(1, s)));
        }
        assert!(!t.blocked(TRUCK_SPOT.0, TRUCK_SPOT.1));
    }

    #[test]
    fn every_obstacle_kind_exists_and_open_ground_dominates() {
        let t = Terrain::default();
        let mut counts = std::collections::HashMap::new();
        let mut n = 0u32;
        for i in -60..=60 {
            for j in -60..=60 {
                let (x, y) = (i as f32 * 130.0, j as f32 * 130.0);
                *counts.entry(t.ground(x, y)).or_insert(0u32) += 1;
                n += 1;
            }
        }
        for g in [Ground::Open, Ground::Lake, Ground::River, Ground::Farm, Ground::Forest, Ground::Cliff] {
            assert!(counts.get(&g).copied().unwrap_or(0) > 0, "no {:?} in {:?}", g, counts);
        }
        let open = counts[&Ground::Open] as f32 / n as f32;
        let blocked = (counts[&Ground::Lake] + counts[&Ground::Farm] + counts[&Ground::Forest] + counts[&Ground::Cliff]) as f32 / n as f32;
        assert!(open > 0.45, "open {open} {:?}", counts);
        assert!((0.05..0.5).contains(&blocked), "blocked {blocked} {:?}", counts);
    }

    #[test]
    fn farms_are_rectangles_clear_of_roads_and_camp() {
        let t = Terrain::default();
        let mut found = 0;
        for ci in -12..=12 {
            for cj in -12..=12 {
                if let Some((x0, y0, x1, y1)) = t.farm_rect(ci, cj) {
                    found += 1;
                    assert!(x1 > x0 && y1 > y0);
                    for &(x, y) in &[(x0, y0), (x1, y1), (x0, y1), (x1, y0), ((x0 + x1) / 2.0, (y0 + y1) / 2.0)] {
                        assert!(!on_road(x, y, 0.0) && !in_camp(x, y, 0.0));
                    }
                    assert_eq!(t.ground((x0 + x1) / 2.0, (y0 + y1) / 2.0), Ground::Farm);
                }
            }
        }
        assert!(found >= 3, "only {found} farms");
    }

    #[test]
    fn slope_changes_speed_and_energy_in_the_right_direction() {
        assert!(speed_factor(0.3) < 1.0 && speed_factor(0.3) < speed_factor(0.1));
        assert!(speed_factor(-0.3) > 1.0 && speed_factor(-0.3) <= 1.35);
        assert_eq!(speed_factor(0.0), 1.0);
        assert!(speed_factor(5.0) >= 0.3);
        assert!(effort_factor(0.3) > 1.0 && effort_factor(-0.3) < 1.0);
        assert!(effort_factor(-5.0) >= 0.5 && effort_factor(5.0) <= 3.0);
        // climbing costs more energy per metre than descending
        assert!(effort_factor(0.3) > effort_factor(-0.3));
    }

    #[test]
    fn grade_sign_matches_elevation_change() {
        let t = Terrain::default();
        let mut checked = 0;
        for (x, y) in sample_grid(&t, 12, 410.0) {
            let up = t.grade_toward(x, y, 1.0, 0.0);
            let dn = t.grade_toward(x, y, -1.0, 0.0);
            assert!((up + dn).abs() < 1e-3, "grade must flip sign when reversing");
            if up.abs() > 0.05 {
                checked += 1;
                let e0 = t.elevation(x, y);
                let e1 = t.elevation(x + 20.0, y);
                assert_eq!(up > 0.0, e1 > e0);
            }
        }
        assert!(checked > 10);
    }

    #[test]
    fn contours_follow_the_level() {
        // a plane rising to the right crosses level 0.5 halfway
        let segs = contour_segments([0.0, 1.0, 1.0, 0.0], 0.5);
        assert_eq!(segs.len(), 1);
        let ((ax, _), (bx, _)) = segs[0];
        assert!((ax - 0.5).abs() < 1e-5 && (bx - 0.5).abs() < 1e-5);
        assert!(contour_segments([0.0; 4], 0.5).is_empty());
        assert!(contour_segments([1.0; 4], 0.5).is_empty());
        // saddles yield two segments
        assert_eq!(contour_segments([1.0, 0.0, 1.0, 0.0], 0.5).len(), 2);
        assert_eq!(contour_segments([0.0, 1.0, 0.0, 1.0], 0.5).len(), 2);
        // every segment endpoint lies on the cell border
        for mask in 1..15u8 {
            let v = [(mask & 1) as f32, ((mask >> 1) & 1) as f32, ((mask >> 2) & 1) as f32, ((mask >> 3) & 1) as f32];
            for ((ax, ay), (bx, by)) in contour_segments(v, 0.5) {
                for (x, y) in [(ax, ay), (bx, by)] {
                    // endpoints sit on the cell border
                    assert!(x == 0.0 || x == 1.0 || y == 0.0 || y == 1.0, "{mask}: {x},{y}");
                }
            }
        }
    }

    #[test]
    fn flat_terrain_has_no_relief_or_obstacles() {
        let t = Terrain::flat();
        for (x, y) in sample_grid(&t, 20, 333.0) {
            assert_eq!(t.ground(x, y), Ground::Open);
            assert_eq!(t.grade_toward(x, y, 1.0, 0.0), 0.0);
        }
    }

    /// Run with `cargo test terrain_distribution -- --ignored --nocapture`
    /// to tune the constants.
    #[test]
    #[ignore]
    fn terrain_distribution() {
        let t = Terrain::default();
        let mut counts = std::collections::BTreeMap::new();
        let (mut emin, mut emax, mut esum, mut n) = (f32::MAX, 0.0f32, 0.0f32, 0u32);
        let mut grades = Vec::new();
        for i in -150..=150 {
            for j in -150..=150 {
                let (x, y) = (i as f32 * 90.0, j as f32 * 90.0);
                *counts.entry(format!("{:?}", t.ground(x, y))).or_insert(0u32) += 1;
                let e = t.elevation(x, y);
                emin = emin.min(e);
                emax = emax.max(e);
                esum += e;
                grades.push(t.steepness(x, y));
                n += 1;
            }
        }
        grades.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("elev min {emin:.0} max {emax:.0} mean {:.0}", esum / n as f32);
        println!("grade p50 {:.3} p90 {:.3} p99 {:.3}", grades[grades.len() / 2], grades[grades.len() * 9 / 10], grades[grades.len() * 99 / 100]);
        for (k, v) in counts {
            println!("{k}: {:.1}%", v as f32 * 100.0 / n as f32);
        }
    }

    /// `OO_MAP_OUT=/path.ppm cargo test dump_terrain_map -- --ignored` writes a
    /// top-down picture of the default terrain (roads in grey) for eyeballing.
    #[test]
    #[ignore]
    fn dump_terrain_map() {
        let Ok(path) = std::env::var("OO_MAP_OUT") else { return };
        let t = Terrain::default();
        let (size, step) = (900usize, 12.0f32);
        let (x0, y0) = (-3200.0f32, -3200.0f32);
        let mut img = vec![0u8; size * size * 3];
        for py in 0..size {
            for px in 0..size {
                let (x, y) = (x0 + px as f32 * step, y0 + (size - 1 - py) as f32 * step);
                let e = t.elevation(x, y);
                let shade = (60.0 + e / ELEV_MAX * 150.0) as u8;
                let mut c = [shade / 2, shade, shade / 2];
                match t.ground(x, y) {
                    Ground::Lake => c = [40, 100, 190],
                    Ground::River => c = [110, 170, 225],
                    Ground::Farm => c = [200, 180, 80],
                    Ground::Forest => c = [20, 70, 30],
                    Ground::Cliff => c = [130, 120, 110],
                    Ground::Edge => c = [10, 10, 10],
                    Ground::Open => {}
                }
                let k = (e / CONTOUR_INTERVAL).floor();
                if (t.elevation(x + step, y) / CONTOUR_INTERVAL).floor() != k || (t.elevation(x, y + step) / CONTOUR_INTERVAL).floor() != k {
                    if t.ground(x, y) != Ground::Lake {
                        c = [90, 60, 30];
                    }
                }
                if on_road(x, y, 0.0) {
                    c = [190, 190, 195];
                }
                if in_camp(x, y, 0.0) {
                    c = [240, 120, 40];
                }
                img[(py * size + px) * 3..][..3].copy_from_slice(&c);
            }
        }
        let mut out = format!("P6\n{size} {size}\n255\n").into_bytes();
        out.extend(img);
        std::fs::write(path, out).unwrap();
    }
}
