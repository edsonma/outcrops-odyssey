mod game;
mod world;

use game::*;
use macroquad::prelude::*;
use std::collections::HashMap;
use world::*;

const TICK: f32 = 1.0 / 60.0;
const PANEL: (f32, f32, f32, f32) = (190.0, 60.0, 900.0, 600.0); // x, y(bottom), w, h

fn window_conf() -> Conf {
    Conf {
        window_title: "Outcrops Odyssey".to_owned(),
        window_width: SCREEN_W as i32,
        window_height: SCREEN_H as i32,
        window_resizable: true,
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

fn render(g: &Game, a: &Assets) {
    clear_background(col(20, 26, 18, 255));
    render_world(g, a);
    render_move_target_marker(g);
    render_player(g, a);
    render_popups(g, a);
    render_night_overlay(g);
    render_ui(g, a);
    match g.ui_mode {
        UiMode::Map => render_map_overlay(g, a),
        UiMode::Cards => render_cards_overlay(g, a),
        UiMode::None => {}
    }
}

fn render_world(g: &Game, a: &Assets) {
    let (cam_x, cam_y) = g.cam();
    let (pcx, pcy) = world_to_chunk(g.player.x, g.player.y);
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
            for t in &chunk.trees {
                sprite(a, "tree_shadow", t.x - cam_x - 60.0, t.y - cam_y - 20.0, 120.0, 46.0, 255, false);
                sprite(a, "tree", t.x - cam_x - 70.0, t.y - cam_y - 10.0, 140.0, 160.0, 255, false);
            }
            for o in &chunk.outcrops {
                let visited = g.visited.contains(&o.id);
                let (ox, oy) = (o.x - cam_x - 36.0, o.y - cam_y - 28.0);
                sprite(a, if visited { "outcrop_visited" } else { "outcrop_raw" }, ox, oy, 72.0, 56.0, 255, false);
                if !visited {
                    let bob = ((g.tick_count as f32 / 20.0) + o.x).sin() * 3.0;
                    sprite(a, "sparkle", ox + 22.0, oy + 44.0 + bob, 28.0, 28.0, 200, false);
                }
            }
        }
    }
}

fn render_player(g: &Game, a: &Assets) {
    let sx = SCREEN_W / 2.0 - PLAYER_W / 2.0;
    let sy = SCREEN_H / 2.0 - PLAYER_H / 2.0;
    sprite(a, "player_shadow", sx - 10.0, sy - 26.0, PLAYER_W + 20.0, 22.0, 255, false);
    sprite(a, "player", sx, sy, PLAYER_W, PLAYER_H, 255, g.player.facing < 0);
    if g.nearest_outcrop().is_some() && g.ui_mode == UiMode::None {
        label(a, "SPACE to break outcrop", SCREEN_W / 2.0, 120.0, 1, true, col(255, 255, 255, 220), false);
    }
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
    label(a, "Yellow = raw   Green = visited   Red = you   .   Press M to close",
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

fn render_ui(g: &Game, a: &Assets) {
    label(a, &format!("Outcrop Points: {}", g.outcrop_points), 24.0, SCREEN_H - 18.0, 2, false, col(255, 255, 255, 230), true);
    label(a, &format!("XP: {}   Inventory: {}/{}", g.xp, g.inventory_count, INVENTORY_CAP), 24.0, SCREEN_H - 42.0, 0, false, col(230, 230, 230, 220), false);
    label(a, &format!("{}% night", (g.night_amount() * 100.0) as i32), 24.0, SCREEN_H - 64.0, -1, false, col(200, 200, 220, 200), false);
    label(a, "WASD/Arrows/Click to move  .  SPACE break  .  M map  .  C cards", 24.0, 40.0, -1, false, col(220, 220, 220, 180), false);
}

/// Debug-builds only: OO_DEMO=map|cards|play scripts a short session for screenshots.
#[cfg(debug_assertions)]
fn debug_demo(g: &mut Game, frame: u32, pending: &mut Input) {
    let Ok(mode) = std::env::var("OO_DEMO") else { return };
    if frame == 5 {
        for _ in 0..14 {
            g.ensure_nearby_chunks_generated();
            let target = g.chunks.values().flat_map(|c| c.outcrops.iter()).find(|o| !g.visited.contains(&o.id)).cloned();
            if let Some(o) = target {
                g.player.x = o.x;
                g.player.y = o.y;
                g.break_nearest_outcrop();
            }
        }
        g.player.x += 300.0;
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

    loop {
        // --- gather input (edges are latched until a tick consumes them) ---
        let (scale, off_x, off_y) = fit();
        pending.map_pressed |= is_key_pressed(KeyCode::M);
        pending.cards_pressed |= is_key_pressed(KeyCode::C);
        pending.space_pressed |= is_key_pressed(KeyCode::Space);
        pending.page_down_pressed |= is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        pending.page_up_pressed |= is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (vx, vy) = ((mx - off_x) / scale, (my - off_y) / scale);
            if (0.0..=SCREEN_W).contains(&vx) && (0.0..=SCREEN_H).contains(&vy) {
                pending.click = Some((vx, SCREEN_H - vy));
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
            game.tick(&input);
            pending = Input::default();
            acc -= TICK;
        }

        // --- draw to the virtual screen, then blit letterboxed ---
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_W, SCREEN_H));
        cam.render_target = Some(target.clone());
        set_camera(&cam);
        render(&game, &assets);

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &target.texture,
            off_x,
            off_y,
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(SCREEN_W * scale, SCREEN_H * scale)), flip_y: true, ..Default::default() },
        );
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
