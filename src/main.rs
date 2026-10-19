mod game;
mod touch;
mod world;

use game::*;
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};
use touch::{Tap, TouchState};
use world::*;

const TICK: f32 = 1.0 / 60.0;
const PANEL: (f32, f32, f32, f32) = (190.0, 60.0, 900.0, 600.0); // x, y(bottom), w, h

fn window_conf() -> Conf {
    Conf {
        window_title: "Outcrops Odyssey".to_owned(),
        window_width: SCREEN_W as i32,
        window_height: SCREEN_H as i32,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

struct Assets {
    tex: HashMap<&'static str, Texture2D>,
    font: Font,
    font_bold: Font,
}

const SPRITES: [&str; 12] = [
    "grass_0", "grass_1", "grass_2", "dirt_patch", "tree", "tree_shadow",
    "outcrop_raw", "outcrop_visited", "sparkle", "player", "player_shadow", "card_popup",
];

async fn load_assets() -> Assets {
    let mut tex = HashMap::new();
    for name in SPRITES {
        let t = load_texture(&format!("assets/sprites/{name}.png")).await.unwrap_or_else(|e| panic!("{name}: {e}"));
        t.set_filter(FilterMode::Linear);
        tex.insert(name, t);
    }
    let font = load_ttf_font("assets/fonts/DejaVuSans.ttf").await.expect("font");
    let font_bold = load_ttf_font("assets/fonts/DejaVuSans-Bold.ttf").await.expect("bold font");
    Assets { tex, font, font_bold }
}

// ---- drawing helpers: all take y-UP virtual coordinates (origin bottom-left) ----

fn col(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

fn solid(x: f32, y: f32, w: f32, h: f32, c: Color) {
    draw_rectangle(x, SCREEN_H - y - h, w, h, c);
}

fn border(x: f32, y: f32, w: f32, h: f32, c: Color) {
    draw_rectangle_lines(x, SCREEN_H - y - h, w, h, 2.0, c);
}

fn sprite(a: &Assets, name: &str, x: f32, y: f32, w: f32, h: f32, alpha: u8, flip_x: bool) {
    draw_texture_ex(
        &a.tex[name],
        x,
        SCREEN_H - y - h,
        col(255, 255, 255, alpha),
        DrawTextureParams { dest_size: Some(vec2(w, h)), flip_x, ..Default::default() },
    );
}

/// `size_enum` mirrors the original DragonRuby label sizes (0 = 22px, +/-2px per step).
fn label(a: &Assets, text: &str, x: f32, y: f32, size_enum: i32, center: bool, c: Color, bold: bool) {
    let size = (22 + 2 * size_enum) as u16;
    let font = if bold { &a.font_bold } else { &a.font };
    let dims = measure_text(text, Some(font), size, 1.0);
    let x = if center { x - dims.width / 2.0 } else { x };
    draw_text_ex(text, x, SCREEN_H - y + size as f32 * 0.85, TextParams { font: Some(font), font_size: size, color: c, ..Default::default() });
}

fn render(g: &Game, a: &Assets, touch: bool) {
    clear_background(col(20, 26, 18, 255));
    render_world(g, a);
    render_truck(g);
    render_player(g, a);
    render_move_target_marker(g);
    render_popups(g, a);
    render_night_overlay(g);
    render_ui(g, a, touch);
    match g.ui_mode {
        UiMode::Map => render_map_overlay(g, a),
        UiMode::Cards => render_cards_overlay(g, a),
        UiMode::None => {}
    }
    render_phase_overlays(g, a);
}

// ---- world-space helpers (y-up world coords -> y-up virtual screen) ----
fn wsolid(g: &Game, x: f32, y: f32, w: f32, h: f32, c: Color) {
    let (cx, cy) = g.cam();
    solid(x - cx, y - cy, w, h, c);
}

fn wlabel(g: &Game, a: &Assets, text: &str, x: f32, y: f32, size_enum: i32, c: Color) {
    let (cx, cy) = g.cam();
    label(a, text, x - cx, y - cy, size_enum, true, c, true);
}

fn render_world(g: &Game, a: &Assets) {
    let (cam_x, cam_y) = g.cam();
    let (fx, fy) = g.focus();
    let (pcx, pcy) = world_to_chunk(fx, fy);

    // pass 1: ground
    for dx in -VIEW_RADIUS_CHUNKS..=VIEW_RADIUS_CHUNKS {
        for dy in -VIEW_RADIUS_CHUNKS..=VIEW_RADIUS_CHUNKS {
            let (cx, cy) = (pcx + dx, pcy + dy);
            let Some(chunk) = g.chunks.get(&(cx, cy)) else { continue };
            let sx = (cx * CHUNK) as f32 - cam_x;
            let sy = (cy * CHUNK) as f32 - cam_y;
            let c = CHUNK as f32;
            if sx > SCREEN_W || sy > SCREEN_H || sx < -c || sy < -c {
                continue;
            }
            sprite(a, &format!("grass_{}", chunk.grass_variant), sx, sy, c, c, 255, false);
            for dp in &chunk.dirt_patches {
                sprite(a, "dirt_patch", dp.x - cam_x - 90.0, dp.y - cam_y - 65.0, 180.0, 130.0, 255, false);
            }
        }
    }

    render_roads(g);
    render_camp(g, a);

    // pass 2: standing objects, drawn back-to-front (higher y first)
    let mut trees: Vec<&Point> = Vec::new();
    let mut outcrops: Vec<&Outcrop> = Vec::new();
    for dx in -VIEW_RADIUS_CHUNKS..=VIEW_RADIUS_CHUNKS {
        for dy in -VIEW_RADIUS_CHUNKS..=VIEW_RADIUS_CHUNKS {
            if let Some(chunk) = g.chunks.get(&(pcx + dx, pcy + dy)) {
                trees.extend(chunk.trees.iter());
                outcrops.extend(chunk.outcrops.iter());
            }
        }
    }
    trees.sort_by(|p, q| q.y.partial_cmp(&p.y).unwrap());
    for t in trees {
        sprite(a, "tree_shadow", t.x - cam_x - 60.0, t.y - cam_y - 20.0, 120.0, 46.0, 255, false);
        sprite(a, "tree", t.x - cam_x - 70.0, t.y - cam_y - 10.0, 140.0, 160.0, 255, false);
    }
    for o in outcrops {
        let visited = g.visited.contains(&o.id);
        let (ox, oy) = (o.x - cam_x - 36.0, o.y - cam_y - 28.0);
        sprite(a, if visited { "outcrop_visited" } else { "outcrop_raw" }, ox, oy, 72.0, 56.0, 255, false);
        if !visited {
            let bob = ((g.tick_count as f32 / 20.0) + o.x).sin() * 3.0;
            sprite(a, "sparkle", ox + 22.0, oy + 44.0 + bob, 28.0, 28.0, 200, false);
        }
    }
}

fn render_roads(g: &Game) {
    let (cam_x, cam_y) = g.cam();
    let asphalt = col(62, 62, 68, 255);
    let edge = col(200, 200, 190, 255);
    let dash = col(235, 200, 70, 255);
    let half = ROAD_W / 2.0;

    // horizontal roads
    let j0 = ((cam_y - ROAD_W) / ROAD_SPACING).floor() as i32;
    let j1 = ((cam_y + SCREEN_H + ROAD_W) / ROAD_SPACING).ceil() as i32;
    for j in j0..=j1 {
        let y = j as f32 * ROAD_SPACING - cam_y;
        if y + half < 0.0 || y - half > SCREEN_H {
            continue;
        }
        solid(0.0, y - half, SCREEN_W, ROAD_W, asphalt);
        solid(0.0, y + half - 7.0, SCREEN_W, 3.0, edge);
        solid(0.0, y - half + 4.0, SCREEN_W, 3.0, edge);
        let start = (cam_x / 80.0).floor() * 80.0;
        let mut x = start;
        while x < cam_x + SCREEN_W + 80.0 {
            solid(x - cam_x, y - 3.0, 44.0, 6.0, dash);
            x += 80.0;
        }
    }
    // vertical roads
    let k0 = ((cam_x - ROAD_V_OFFSET - ROAD_W) / ROAD_SPACING).floor() as i32;
    let k1 = ((cam_x + SCREEN_W - ROAD_V_OFFSET + ROAD_W) / ROAD_SPACING).ceil() as i32;
    for k in k0..=k1 {
        let x = ROAD_V_OFFSET + k as f32 * ROAD_SPACING - cam_x;
        if x + half < 0.0 || x - half > SCREEN_W {
            continue;
        }
        solid(x - half, 0.0, ROAD_W, SCREEN_H, asphalt);
        solid(x + half - 7.0, 0.0, 3.0, SCREEN_H, edge);
        solid(x - half + 4.0, 0.0, 3.0, SCREEN_H, edge);
        let start = (cam_y / 80.0).floor() * 80.0;
        let mut y = start;
        while y < cam_y + SCREEN_H + 80.0 {
            solid(x - 3.0, y - cam_y, 6.0, 44.0, dash);
            y += 80.0;
        }
    }
}

fn render_camp(g: &Game, a: &Assets) {
    let (x0, y0, x1, y1) = CAMP;
    let (cam_x, cam_y) = g.cam();
    if x1 - cam_x < -50.0 || x0 - cam_x > SCREEN_W + 50.0 || y1 - cam_y < -50.0 || y0 - cam_y > SCREEN_H + 50.0 {
        return;
    }
    // packed-earth ground with a darker rim, plus a gravel path down to the road
    wsolid(g, x0 - 8.0, y0 - 8.0, x1 - x0 + 16.0, y1 - y0 + 16.0, col(70, 55, 38, 255));
    wsolid(g, x0, y0, x1 - x0, y1 - y0, col(128, 104, 72, 255));
    wsolid(g, -40.0, ROAD_W / 2.0, 80.0, y0 - ROAD_W / 2.0, col(150, 140, 120, 255));

    // cabins
    for &(cx, cy) in &[(-300.0f32, 380.0f32), (-120.0, 400.0), (200.0, 395.0)] {
        wsolid(g, cx - 4.0, cy - 54.0, 128.0, 8.0, col(0, 0, 0, 70)); // shadow
        wsolid(g, cx, cy - 46.0, 120.0, 50.0, col(112, 76, 46, 255)); // front wall
        wsolid(g, cx + 46.0, cy - 46.0, 28.0, 34.0, col(60, 38, 22, 255)); // door
        wsolid(g, cx + 10.0, cy - 30.0, 22.0, 16.0, col(170, 210, 230, 255)); // window
        wsolid(g, cx - 8.0, cy + 4.0, 136.0, 60.0, col(150, 58, 44, 255)); // roof
        wsolid(g, cx - 8.0, cy + 32.0, 136.0, 5.0, col(105, 38, 30, 255)); // ridge
    }
    // tent
    let (cam_x, cam_y) = (cam_x, cam_y);
    let tp = |x: f32, y: f32| vec2(x - cam_x, SCREEN_H - (y - cam_y));
    draw_triangle(tp(300.0, 250.0), tp(380.0, 250.0), tp(340.0, 320.0), col(214, 178, 70, 255));
    draw_triangle(tp(330.0, 250.0), tp(350.0, 250.0), tp(340.0, 292.0), col(70, 50, 30, 255));
    // crates and supplies
    for &(cx, cy) in &[(250.0f32, 190.0f32), (284.0, 196.0), (262.0, 226.0)] {
        wsolid(g, cx, cy, 28.0, 28.0, col(150, 108, 56, 255));
        wsolid(g, cx + 2.0, cy + 2.0, 24.0, 24.0, col(176, 132, 74, 255));
    }
    // campfire
    let flick = (g.tick_count as f32 / 4.0).sin() * 3.0;
    let c = tp(-30.0, 240.0);
    draw_circle(c.x, c.y, 24.0, col(60, 50, 44, 255));
    draw_circle(c.x, c.y, 16.0, col(240, 120, 30, 255));
    draw_circle(c.x, c.y, 9.0 + flick * 0.3, col(255, 214, 80, 255));
    // sign
    wsolid(g, -150.0, 140.0, 300.0, 36.0, col(86, 58, 34, 255));
    wlabel(g, a, "BASECAMP", 0.0, 168.0, 0, col(250, 240, 210, 255));
}

fn render_truck(g: &Game) {
    let t = &g.truck;
    let (dx, dy) = (t.dx, t.dy);
    let part = |u0: f32, u1: f32, v0: f32, v1: f32, c: Color| {
        let (ax, ay) = (t.x + u0 * dx - v0 * dy, t.y + u0 * dy + v0 * dx);
        let (bx, by) = (t.x + u1 * dx - v1 * dy, t.y + u1 * dy + v1 * dx);
        wsolid(g, ax.min(bx), ay.min(by), (ax - bx).abs(), (ay - by).abs(), c);
    };
    part(-74.0, 56.0, -30.0, 30.0, col(0, 0, 0, 60)); // shadow
    for &u in &[-52.0f32, 28.0] {
        part(u - 12.0, u + 12.0, -31.0, -23.0, col(25, 25, 28, 255));
        part(u - 12.0, u + 12.0, 23.0, 31.0, col(25, 25, 28, 255));
    }
    part(-72.0, 8.0, -25.0, 25.0, col(74, 112, 64, 255)); // cargo box
    part(-68.0, 4.0, -20.0, 20.0, col(92, 134, 80, 255));
    part(8.0, 52.0, -24.0, 24.0, col(196, 62, 50, 255)); // cab
    part(34.0, 46.0, -18.0, 18.0, col(170, 210, 230, 255)); // windshield
    part(50.0, 54.0, -22.0, -14.0, col(255, 240, 160, 255)); // headlights
    part(50.0, 54.0, 14.0, 22.0, col(255, 240, 160, 255));
}

fn render_player(g: &Game, a: &Assets) {
    if !g.player_visible() {
        return;
    }
    let (cam_x, cam_y) = g.cam();
    let sx = g.player.x - cam_x - PLAYER_W / 2.0;
    let sy = g.player.y - cam_y - PLAYER_H / 2.0;
    sprite(a, "player_shadow", sx - 10.0, sy - 26.0, PLAYER_W + 20.0, 22.0, 255, false);
    sprite(a, "player", sx, sy, PLAYER_W, PLAYER_H, 255, g.player.facing < 0);
}

fn render_popups(g: &Game, a: &Assets) {
    let (cam_x, cam_y) = g.cam();
    for p in &g.popups {
        let alpha = (p.life * 255 / POPUP_LIFE).clamp(0, 255) as u8;
        label(a, &p.text, p.x - cam_x, p.y - cam_y, 0, true, col(p.color.0, p.color.1, p.color.2, alpha), false);
    }
}

fn render_night_overlay(g: &Game) {
    let n = g.night_amount();
    if n >= 0.05 {
        solid(0.0, 0.0, SCREEN_W, SCREEN_H, col(15, 20, 55, (n * 165.0) as u8));
    }
}

fn render_move_target_marker(g: &Game) {
    if let Some((tx, ty)) = g.move_target {
        let (cam_x, cam_y) = g.cam();
        let pulse = 6.0 + (g.tick_count as f32 / 8.0).sin() * 2.0;
        border(tx - cam_x - pulse, ty - cam_y - pulse, pulse * 2.0, pulse * 2.0, col(255, 255, 255, 180));
    }
}

fn panel_frame(a: &Assets, title: &str) {
    let (px, py, pw, ph) = PANEL;
    solid(0.0, 0.0, SCREEN_W, SCREEN_H, col(10, 12, 10, 210));
    solid(px, py, pw, ph, col(30, 26, 20, 235));
    border(px, py, pw, ph, col(200, 180, 130, 255));
    label(a, title, px + pw / 2.0, py + ph + 34.0, 4, true, col(255, 255, 255, 230), true);
}

fn render_map_overlay(g: &Game, a: &Assets) {
    let (px, py, pw, ph) = PANEL;
    panel_frame(a, "MAP");
    if !g.chunks.is_empty() {
        let min_cx = g.chunks.keys().map(|k| k.0).min().unwrap();
        let max_cx = g.chunks.keys().map(|k| k.0).max().unwrap();
        let min_cy = g.chunks.keys().map(|k| k.1).min().unwrap();
        let max_cy = g.chunks.keys().map(|k| k.1).max().unwrap();
        let (wmin_x, wmin_y) = ((min_cx * CHUNK) as f32, (min_cy * CHUNK) as f32);
        let world_w = (((max_cx + 1) * CHUNK) as f32 - wmin_x).max(1.0);
        let world_h = (((max_cy + 1) * CHUNK) as f32 - wmin_y).max(1.0);
        let margin = 30.0;
        let scale = ((pw - margin * 2.0) / world_w).min((ph - margin * 2.0) / world_h);

        // main roads and the basecamp
        let (span_w, span_h) = (world_w * scale, world_h * scale);
        let j0 = (wmin_y / ROAD_SPACING).ceil() as i32;
        let j1 = ((wmin_y + world_h) / ROAD_SPACING).floor() as i32;
        for j in j0..=j1 {
            let yy = py + margin + (j as f32 * ROAD_SPACING - wmin_y) * scale;
            solid(px + margin, yy - 2.0, span_w, 4.0, col(150, 150, 160, 255));
        }
        let k0 = ((wmin_x - ROAD_V_OFFSET) / ROAD_SPACING).ceil() as i32;
        let k1 = ((wmin_x + world_w - ROAD_V_OFFSET) / ROAD_SPACING).floor() as i32;
        for k in k0..=k1 {
            let xx = px + margin + (ROAD_V_OFFSET + k as f32 * ROAD_SPACING - wmin_x) * scale;
            solid(xx - 2.0, py + margin, 4.0, span_h, col(150, 150, 160, 255));
        }
        let cxm = px + margin + (0.0 - wmin_x) * scale;
        let cym = py + margin + (CAMP.1 - wmin_y) * scale;
        solid(cxm - 7.0, cym - 7.0, 14.0, 14.0, col(90, 150, 235, 255));
        label(a, "Basecamp", cxm + 12.0, cym + 4.0, -2, false, col(160, 200, 255, 255), false);

        for chunk in g.chunks.values() {
            for o in &chunk.outcrops {
                let mx = px + margin + (o.x - wmin_x) * scale;
                let my = py + margin + (o.y - wmin_y) * scale;
                let c = if g.visited.contains(&o.id) { col(120, 210, 120, 255) } else { col(220, 210, 90, 255) };
                solid(mx - 3.0, my - 3.0, 6.0, 6.0, c);
            }
        }
        let ppx = px + margin + (g.player.x - wmin_x) * scale;
        let ppy = py + margin + (g.player.y - wmin_y) * scale;
        solid(ppx - 5.0, ppy - 5.0, 10.0, 10.0, col(235, 60, 60, 255));
    }
    label(a, "Yellow = raw   Green = visited   Red = you   Blue = basecamp   Gray = main roads   .   M to close",
        px + pw / 2.0, py - 14.0, -1, true, col(220, 220, 220, 200), false);
}

fn render_cards_overlay(g: &Game, a: &Assets) {
    let (px, py, pw, ph) = PANEL;
    panel_frame(a, &format!("CARDS ({} collected)", g.cards.len()));
    let start = g.card_page * CARDS_PER_PAGE;
    let page: Vec<&Card> = g.cards.iter().skip(start).take(CARDS_PER_PAGE).collect();
    if page.is_empty() {
        label(a, "No cards yet -- go break some outcrops!", px + pw / 2.0, py + ph / 2.0, 1, true, col(210, 205, 190, 220), false);
    }
    let mut row_y = py + ph - 60.0;
    for (i, c) in page.iter().enumerate() {
        let mut badges = Vec::new();
        if c.mineral { badges.push("MINERAL"); }
        if c.fossil { badges.push("FOSSIL"); }
        if c.bonus_rock { badges.push("ROCK"); }
        let badge_text = if badges.is_empty() { String::new() } else { format!("  [{}]", badges.join(", ")) };
        solid(px + 24.0, row_y - 12.0, pw - 48.0, 50.0, col(40, 35, 27, if i % 2 == 0 { 130 } else { 60 }));
        label(a, &format!("{}. {}", start + i + 1, c.name), px + 40.0, row_y + 34.0, 1, false, col(240, 235, 218, 235), false);
        label(a, &format!("{}{}", c.lithology, badge_text), px + 40.0, row_y + 8.0, -1, false, col(190, 195, 175, 210), false);
        row_y -= 58.0;
    }
    label(a, &format!("Page {}/{}   .   Up/Down to page   .   Press C to close", g.card_page + 1, g.total_card_pages()),
        px + pw / 2.0, py - 14.0, -1, true, col(220, 220, 220, 200), false);
}

fn render_ui(g: &Game, a: &Assets, touch: bool) {
    label(a, &format!("Outcrop Points: {}", g.outcrop_points), 24.0, SCREEN_H - 18.0, 2, false, col(255, 255, 255, 230), true);
    label(a, &format!("XP: {}   Inventory: {}/{}", g.xp, g.inventory_count, INVENTORY_CAP), 24.0, SCREEN_H - 42.0, 0, false, col(230, 230, 230, 220), false);

    // energy bar
    let (bx, by, bw, bh) = (24.0, SCREEN_H - 92.0, 200.0, 14.0);
    solid(bx, by, bw, bh, col(0, 0, 0, 150));
    let f = (g.energy / ENERGY_MAX).clamp(0.0, 1.0);
    solid(bx, by, bw * f, bh, if f > 0.25 { col(110, 200, 110, 230) } else { col(220, 90, 70, 230) });
    border(bx, by, bw, bh, col(255, 255, 255, 120));
    label(a, "Energy", bx + bw + 10.0, by + 14.0, -1, false, col(220, 220, 220, 200), false);

    // clock
    let m = g.minutes_of_day();
    label(a, &format!("Day {}   {:02}:{:02}", g.day, m / 60, m % 60), SCREEN_W - 24.0 - 190.0, SCREEN_H - 18.0, 2, false, col(255, 240, 200, 235), true);
    label(a, "Sunset 18:00", SCREEN_W - 24.0 - 190.0, SCREEN_H - 44.0, -1, false, col(230, 200, 160, 200), false);

    // contextual hints
    let mut hint: Option<(String, Color)> = None;
    match g.phase {
        Phase::Basecamp if g.near_truck() => hint = Some((if touch { "Tap BOARD to climb in" } else { "Press E to board the truck" }.into(), col(255, 255, 255, 235))),
        Phase::Basecamp => hint = Some(("Walk to the truck on the road below camp".into(), col(230, 230, 210, 210))),
        Phase::Exploring if g.player_on_road() => hint = Some((if touch { "Tap TRUCK to call the truck back to basecamp" } else { "Press E to call the truck back to basecamp" }.into(), col(190, 240, 190, 240))),
        Phase::Exploring if g.nearest_outcrop().is_some() => hint = Some((if touch { "Tap BREAK to break the outcrop" } else { "SPACE to break outcrop" }.into(), col(255, 255, 255, 220))),
        _ => {}
    }
    if let Some((text, c)) = hint {
        label(a, &text, SCREEN_W / 2.0, 120.0, 1, true, c, false);
    }

    if g.phase == Phase::Exploring {
        let left = g.minutes_to_sunset();
        if left <= 120 && !g.player_on_road() {
            let c = if left <= 45 { col(255, 110, 90, 245) } else { col(255, 210, 120, 235) };
            label(a, &format!("Sunset in {} min - reach a main road!", left), SCREEN_W / 2.0, SCREEN_H - 24.0, 1, true, c, true);
        }
        render_road_compass(g, a);
    }

    if let Some((text, life)) = &g.banner {
        let alpha = ((*life as f32 / 40.0).min(1.0) * 255.0) as u8;
        solid(0.0, SCREEN_H - 150.0, SCREEN_W, 44.0, col(0, 0, 0, alpha / 3));
        label(a, text, SCREEN_W / 2.0, SCREEN_H - 112.0, 2, true, col(255, 255, 255, alpha), true);
    }

    if !touch {
        label(a, "WASD/Arrows/Click move  .  SPACE break  .  E interact  .  M map  .  C cards", 24.0, 40.0, -1, false, col(220, 220, 220, 180), false);
    }
}

fn render_touch_ui(g: &Game, a: &Assets, t: &TouchState) {
    if !t.active {
        return;
    }
    for b in touch::buttons(g) {
        let (fill, line) = if b.hot { (col(255, 235, 150, 120), col(255, 245, 190, 235)) } else { (col(20, 20, 20, 110), col(255, 255, 255, 120)) };
        solid(b.x, b.y, b.w, b.h, fill);
        border(b.x, b.y, b.w, b.h, line);
        let size = if b.label.chars().count() > 5 { 0 } else { 1 };
        label(a, &b.label, b.x + b.w / 2.0, b.y + b.h / 2.0 + 10.0, size, true, col(255, 255, 255, 245), true);
    }
    // virtual stick: faint hint ring when idle, base + knob while held
    if g.ui_mode == UiMode::None && g.player_visible() {
        let (bx, by, kx, ky, alpha) = match t.stick {
            Some(s) => {
                let (kx, ky) = s.knob();
                (s.base.0, s.base.1, kx, ky, 150u8)
            }
            None => (150.0, 150.0, 150.0, 150.0, 55u8),
        };
        draw_circle_lines(bx, SCREEN_H - by, 90.0, 3.0, col(255, 255, 255, alpha));
        draw_circle(kx, SCREEN_H - ky, 34.0, col(255, 255, 255, alpha));
    }
}

/// A small arrow near the player pointing at the closest main road.
fn render_road_compass(g: &Game, a: &Assets) {
    if g.player_on_road() {
        return;
    }
    let (px, py) = (g.player.x, g.player.y);
    let (dy_road, dx_road) = (nearest_h_road(py) - py, nearest_v_road(px) - px);
    let (vx, vy, dist) = if dy_road.abs() <= dx_road.abs() { (0.0, dy_road.signum(), dy_road.abs()) } else { (dx_road.signum(), 0.0, dx_road.abs()) };
    let dist = (dist - ROAD_W / 2.0).max(0.0);
    let (cx, cy) = (SCREEN_W / 2.0 + vx * 110.0, SCREEN_H / 2.0 + vy * 110.0 + 20.0);
    // y-up -> y-down for the triangle
    let (sx, sy) = (cx, SCREEN_H - cy);
    let (fx, fy) = (vx, -vy);
    let (nx, ny) = (-fy, fx);
    draw_triangle(
        vec2(sx + fx * 16.0, sy + fy * 16.0),
        vec2(sx - fx * 8.0 + nx * 11.0, sy - fy * 8.0 + ny * 11.0),
        vec2(sx - fx * 8.0 - nx * 11.0, sy - fy * 8.0 - ny * 11.0),
        col(255, 240, 150, 220),
    );
    label(a, &format!("road {}m", dist as i32 / 10), cx, cy - 20.0 + vy * -26.0 - 4.0, -2, true, col(255, 240, 180, 220), false);
}

fn render_phase_overlays(g: &Game, a: &Assets) {
    match g.phase {
        Phase::Sleeping => {
            let t = g.sleep_timer as f32;
            let env = (t / 50.0).min((SLEEP_TICKS as f32 - t) / 50.0).clamp(0.0, 1.0);
            solid(0.0, 0.0, SCREEN_W, SCREEN_H, col(4, 6, 18, (env * 245.0) as u8));
            let alpha = (env * 255.0) as u8;
            label(a, &format!("Day {} complete", g.day), SCREEN_W / 2.0, SCREEN_H / 2.0 + 50.0, 6, true, col(255, 255, 255, alpha), true);
            label(a, &format!("{} points   {} cards   Resting to restore energy...", g.outcrop_points, g.cards.len()),
                SCREEN_W / 2.0, SCREEN_H / 2.0 - 10.0, 1, true, col(210, 215, 235, alpha), false);
        }
        Phase::GameOver => {
            solid(0.0, 0.0, SCREEN_W, SCREEN_H, col(5, 5, 14, 225));
            label(a, "GAME OVER", SCREEN_W / 2.0, SCREEN_H / 2.0 + 110.0, 10, true, col(235, 80, 70, 255), true);
            label(a, "Night fell and you never reached a main road.", SCREEN_W / 2.0, SCREEN_H / 2.0 + 40.0, 2, true, col(235, 235, 235, 240), false);
            if let Some((day, pts, cards)) = g.game_over_stats {
                label(a, &format!("Survived {} day(s)   {} points   {} cards", day, pts, cards),
                    SCREEN_W / 2.0, SCREEN_H / 2.0 - 10.0, 1, true, col(210, 210, 220, 230), false);
            }
            label(a, "Press ENTER to start over", SCREEN_W / 2.0, SCREEN_H / 2.0 - 80.0, 2, true, col(255, 240, 160, 255), true);
        }
        _ => {}
    }
}

/// Debug-builds only: OO_DEMO=<scene> scripts a state for screenshots.
/// scenes: camp | ride | field | road | map | cards | sunset | over | sleep
#[cfg(debug_assertions)]
fn debug_demo(g: &mut Game, frame: u32, pending: &mut Input) {
    let Ok(mode) = std::env::var("OO_DEMO") else { return };
    let to_field = |g: &mut Game| {
        g.player.x = TRUCK_SPOT.0;
        g.player.y = TRUCK_SPOT.1;
        g.tick(&Input { interact_pressed: true, ..Default::default() });
        while matches!(g.phase, Phase::TruckOut) {
            g.tick(&Input::default());
        }
    };
    let break_some = |g: &mut Game| {
        for _ in 0..14 {
            g.ensure_nearby_chunks_generated();
            let target = g.chunks.values().flat_map(|c| c.outcrops.iter()).find(|o| !g.visited.contains(&o.id)).cloned();
            if let Some(o) = target {
                g.player.x = o.x;
                g.player.y = o.y;
                g.break_nearest_outcrop();
            }
        }
    };
    if frame == 5 {
        match mode.as_str() {
            "ride" => {
                g.player.x = TRUCK_SPOT.0;
                g.player.y = TRUCK_SPOT.1;
                g.tick(&Input { interact_pressed: true, ..Default::default() });
                for _ in 0..70 {
                    g.tick(&Input::default());
                }
            }
            "field" => {
                to_field(g);
                g.player.x += 300.0;
                g.player.y += 40.0;
            }
            "road" => {
                to_field(g);
                g.clock_ticks = 11 * 60 * TICKS_PER_MINUTE;
                g.player.x -= 600.0;
            }
            "sunset" => {
                to_field(g);
                g.clock_ticks = (12 * 60 + 40) * TICKS_PER_MINUTE;
                g.player.x += 500.0;
            }
            "map" | "cards" => {
                to_field(g);
                break_some(g);
                g.player.x += 300.0;
            }
            "over" => {
                to_field(g);
                g.clock_ticks = (SUNSET_MIN - DAY_START_MIN) * TICKS_PER_MINUTE;
                g.player.x += 500.0;
            }
            "sleep" => {
                g.phase = Phase::Sleeping;
                g.sleep_timer = 110;
                g.outcrop_points = 120;
            }
            _ => {}
        }
    }
    if frame == 20 {
        match mode.as_str() {
            "map" => pending.map_pressed = true,
            "cards" => pending.cards_pressed = true,
            _ => {}
        }
    }
}

/// Letterbox the 1280x720 virtual screen into the real window.
fn fit() -> (f32, f32, f32) {
    let s = (screen_width() / SCREEN_W).min(screen_height() / SCREEN_H);
    (s, (screen_width() - SCREEN_W * s) / 2.0, (screen_height() - SCREEN_H * s) / 2.0)
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = load_assets().await;
    let target = render_target(SCREEN_W as u32, SCREEN_H as u32);
    target.texture.set_filter(FilterMode::Linear);

    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64 ^ 0x9E37_79B9_7F4A_7C15;
    let mut game = Game::new(seed);
    game.ensure_nearby_chunks_generated();

    let mut pending = Input::default(); // edge-triggered input collected between ticks
    let mut acc = 0.0f32;
    let mut frame = 0u32;
    let mut touch_state = TouchState::default();
    let mut known_touches: HashSet<u64> = HashSet::new();
    simulate_mouse_with_touch(false);
    #[cfg(debug_assertions)]
    if std::env::var("OO_TOUCH").is_ok() {
        touch_state.active = true;
    }

    loop {
        // --- gather input (edges are latched until a tick consumes them) ---
        let (scale, off_x, off_y) = fit();
        pending.map_pressed |= is_key_pressed(KeyCode::M);
        pending.cards_pressed |= is_key_pressed(KeyCode::C);
        pending.space_pressed |= is_key_pressed(KeyCode::Space);
        pending.interact_pressed |= is_key_pressed(KeyCode::E);
        pending.restart_pressed |= is_key_pressed(KeyCode::Enter);
        pending.page_down_pressed |= is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        pending.page_up_pressed |= is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        let to_virtual = |px: f32, py: f32| ((px - off_x) / scale, SCREEN_H - (py - off_y) / scale);
        let on_screen = |vx: f32, vy: f32| (0.0..=SCREEN_W).contains(&vx) && (0.0..=SCREEN_H).contains(&vy);
        let route = |tap: Tap, pending: &mut Input, ui: UiMode| match tap {
            Tap::Button(id) => touch::apply(id, pending, ui),
            Tap::World(x, y) => pending.click = Some((x, y)),
            Tap::StickStarted => {}
        };

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (vx, vy) = to_virtual(mx, my);
            if on_screen(vx, vy) {
                // mouse clicks may press on-screen buttons once they are visible, else walk
                let bs = touch::buttons(&game);
                match (touch_state.active, touch::hit(&bs, vx, vy)) {
                    (true, Some(id)) => touch::apply(id, &mut pending, game.ui_mode),
                    _ => pending.click = Some((vx, vy)),
                }
            }
        }

        // touches: positions are physical pixels, so divide by the DPI scale first
        let dpi = screen_dpi_scale();
        let mut seen_now = HashSet::new();
        for t in touches() {
            let (vx, vy) = to_virtual(t.position.x / dpi, t.position.y / dpi);
            seen_now.insert(t.id);
            match t.phase {
                TouchPhase::Started if !known_touches.contains(&t.id) => {
                    known_touches.insert(t.id);
                    let tap = touch_state.down(t.id, vx, vy, &game);
                    route(tap, &mut pending, game.ui_mode);
                }
                TouchPhase::Moved | TouchPhase::Stationary => touch_state.moved(t.id, vx, vy),
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    touch_state.up(t.id);
                    known_touches.remove(&t.id);
                }
                _ => {}
            }
        }
        // a touch that vanished without an Ended event must not leave the stick stuck
        known_touches.retain(|id| seen_now.contains(id));
        if let Some(st) = touch_state.stick {
            if !known_touches.contains(&st.id) {
                touch_state.up(st.id);
            }
        }

        #[cfg(debug_assertions)]
        debug_demo(&mut game, frame, &mut pending);

        // --- fixed 60Hz simulation, independent of display refresh rate ---
        acc = (acc + get_frame_time()).min(0.25);
        while acc >= TICK {
            let mut input = pending;
            input.left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
            input.right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
            input.up = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
            input.down = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
            let (sl, sr, su, sd) = touch_state.dirs();
            input.left |= sl;
            input.right |= sr;
            input.up |= su;
            input.down |= sd;
            game.tick(&input);
            pending = Input::default();
            acc -= TICK;
        }

        // --- draw to the virtual screen, then blit letterboxed ---
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_W, SCREEN_H));
        cam.render_target = Some(target.clone());
        set_camera(&cam);
        render(&game, &assets, touch_state.active);
        render_touch_ui(&game, &assets, &touch_state);

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &target.texture,
            off_x,
            off_y,
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(SCREEN_W * scale, SCREEN_H * scale)), flip_y: true, ..Default::default() },
        );
        if screen_height() > screen_width() * 1.05 {
            // portrait: the game is landscape-only; nudge the player to rotate
            let (w, h) = (screen_width(), screen_height());
            draw_rectangle(0.0, h * 0.5 - 70.0, w, 140.0, Color::from_rgba(0, 0, 0, 200));
            let msg = "Rotate your device to landscape";
            let d = measure_text(msg, Some(&assets.font_bold), 28, 1.0);
            draw_text_ex(msg, (w - d.width) / 2.0, h * 0.5 + 10.0, TextParams { font: Some(&assets.font_bold), font_size: 28, color: WHITE, ..Default::default() });
        }
        // dev aid: OO_SHOT=/path.png saves a screenshot after OO_SHOT_FRAMES frames and exits
        frame += 1;
        if let Ok(path) = std::env::var("OO_SHOT") {
            let n: u32 = std::env::var("OO_SHOT_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(90);
            if frame == n {
                get_screen_data().export_png(&path);
                return;
            }
        }
        next_frame().await;
    }
}
