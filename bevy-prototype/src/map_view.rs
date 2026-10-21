//! The 2D geological map for the Bevy build: the same colours as the Macroquad map (unit colours
//! with relief shading, lakes, rivers, forest, cliffs, farms, roads), painted into one small
//! RGBA image that the UI shows. Unexplored ground stays dark.

use crate::game::Game;
use crate::terrain::Ground;
use crate::world::{in_camp, on_road, world_to_chunk, CHUNK};

pub const MAP_PX: usize = 192;

pub struct MapPixels {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

fn lerp(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}

/// World rectangle to show: the whole survey area on bounded maps, else the explored chunks.
pub fn world_rect(g: &Game) -> (f32, f32, f32, f32) {
    if let Some((x0, y0, x1, y1)) = g.terrain.bounds {
        return (x0, y0, x1 - x0, y1 - y0);
    }
    if g.chunks.is_empty() {
        return (-640.0, -640.0, 1280.0, 1280.0);
    }
    let (mut ax, mut ay, mut bx, mut by) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for &(cx, cy) in g.chunks.keys() {
        ax = ax.min(cx);
        ay = ay.min(cy);
        bx = bx.max(cx);
        by = by.max(cy);
    }
    let (x0, y0) = ((ax * CHUNK) as f32, (ay * CHUNK) as f32);
    (x0, y0, (((bx + 1) * CHUNK) as f32 - x0).max(1.0), (((by + 1) * CHUNK) as f32 - y0).max(1.0))
}

pub fn build(g: &Game) -> MapPixels {
    let (x0, y0, ww, wh) = world_rect(g);
    let cell = ww.max(wh) / MAP_PX as f32;
    let (nx, ny) = ((ww / cell).ceil() as usize, (wh / cell).ceil() as usize);
    let t = &g.terrain;
    let gw = nx + 1;
    let mut elev = vec![0.0f32; gw * (ny + 1)];
    for j in 0..=ny {
        for i in 0..=nx {
            elev[j * gw + i] = t.elevation(x0 + i as f32 * cell, y0 + j as f32 * cell);
        }
    }
    let at = |i: usize, j: usize| elev[j * gw + i];
    let mut rgba = vec![0u8; nx * ny * 4];
    for j in 0..ny {
        for i in 0..nx {
            let (mx, my) = (x0 + (i as f32 + 0.5) * cell, y0 + (j as f32 + 0.5) * cell);
            let row = ny - 1 - j; // image row 0 is the top (north)
            let px = (row * nx + i) * 4;
            let mut put = |c: (u8, u8, u8), a: u8| {
                rgba[px] = c.0;
                rgba[px + 1] = c.1;
                rgba[px + 2] = c.2;
                rgba[px + 3] = a;
            };
            if !g.chunks.contains_key(&world_to_chunk(mx, my)) {
                put((14, 16, 12), 255);
                continue;
            }
            let (e00, e10, e11, e01) = (at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1));
            let avg = (e00 + e10 + e11 + e01) / 4.0;
            let steep = (((e10 + e11 - e00 - e01) / 2.0) / cell).hypot(((e01 + e11 - e00 - e10) / 2.0) / cell);
            let ground = t.classify(mx, my, avg, steep);
            let unit_col = || g.region.units[g.region.unit_at_elev(t, mx, my, avg)].color;
            let c = match ground {
                Ground::Open | Ground::Farm => {
                    let (ur, ug, ub) = unit_col();
                    let dzdx = ((e10 + e11) - (e00 + e01)) / 2.0 / cell;
                    let dzdy = ((e01 + e11) - (e00 + e10)) / 2.0 / cell;
                    let k = 1.0 + ((-dzdx + dzdy) * 1.4).clamp(-0.35, 0.35);
                    let f = |c: u8| (c as f32 * k).clamp(0.0, 255.0) as u8;
                    let base = (f(ur), f(ug), f(ub));
                    if ground == Ground::Farm { lerp(base, (230, 215, 120), 0.45) } else { base }
                }
                Ground::Lake => (58, 118, 188),
                Ground::River => (110, 172, 222),
                Ground::Forest => lerp(unit_col(), (24, 78, 40), 0.72),
                Ground::Cliff => (126, 118, 110),
                Ground::Edge => (14, 16, 12),
            };
            if on_road(mx, my, 40.0) && !ground.is_water() {
                put((170, 170, 178), 255);
            } else if in_camp(mx, my, 0.0) {
                put((236, 210, 150), 255);
            } else {
                put(c, 255);
            }
        }
    }
    MapPixels { w: nx, h: ny, rgba }
}
