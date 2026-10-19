//! Game state and rules. No rendering or windowing code lives here, so the
//! whole loop is unit-testable headlessly.

use crate::world::*;
use std::collections::{HashMap, HashSet};

pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;
pub const PLAYER_W: f32 = 60.0;
pub const PLAYER_H: f32 = 80.0;
pub const PLAYER_SPEED: f32 = 4.2;
pub const BREAK_RANGE: f32 = 60.0;
pub const DAY_LENGTH_TICKS: u32 = 5400;
pub const CARDS_PER_PAGE: usize = 8;
pub const INVENTORY_CAP: u32 = 10;
pub const POPUP_LIFE: i32 = 70;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiMode {
    None,
    Map,
    Cards,
}

/// One tick's worth of input. Edge-triggered fields (`*_pressed`, `click`)
/// are true only on the tick they happen.
#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub map_pressed: bool,
    pub cards_pressed: bool,
    pub space_pressed: bool,
    pub page_down_pressed: bool,
    pub page_up_pressed: bool,
    /// Click position in 1280x720 virtual screen space, y-UP.
    pub click: Option<(f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub lithology: &'static str,
    pub mineral: bool,
    pub fossil: bool,
    pub bonus_rock: bool,
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub life: i32,
    pub color: (u8, u8, u8),
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub facing: i32,
}

/// Tiny xorshift RNG so game logic stays dependency-free and seedable in tests.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    /// Uniform-ish integer in 0..n.
    pub fn below(&mut self, n: u32) -> u32 {
        self.next_u32() % n
    }
}

pub struct Game {
    pub player: Player,
    pub outcrop_points: u32,
    pub xp: u32,
    pub inventory_count: u32,
    pub popups: Vec<Popup>,
    pub cards: Vec<Card>,
    pub ui_mode: UiMode,
    pub card_page: usize,
    pub move_target: Option<(f32, f32)>,
    pub tick_count: u32,
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub visited: HashSet<String>,
    pub rng: Rng,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        Game {
            player: Player { x: 0.0, y: 0.0, facing: 1 },
            outcrop_points: 0,
            xp: 10,
            inventory_count: 0,
            popups: Vec::new(),
            cards: Vec::new(),
            ui_mode: UiMode::None,
            card_page: 0,
            move_target: None,
            tick_count: 0,
            chunks: HashMap::new(),
            visited: HashSet::new(),
            rng: Rng::new(seed),
        }
    }

    pub fn cam(&self) -> (f32, f32) {
        (self.player.x - SCREEN_W / 2.0, self.player.y - SCREEN_H / 2.0)
    }

    pub fn total_card_pages(&self) -> usize {
        self.cards.len().div_ceil(CARDS_PER_PAGE).max(1)
    }

    pub fn tick(&mut self, input: &Input) {
        self.handle_input(input);
        self.tick_count += 1;
        self.ensure_nearby_chunks_generated();
        for p in &mut self.popups {
            p.y += 0.6;
            p.life -= 1;
        }
        self.popups.retain(|p| p.life > 0);
    }

    fn handle_input(&mut self, input: &Input) {
        if input.map_pressed {
            self.ui_mode = if self.ui_mode == UiMode::Map { UiMode::None } else { UiMode::Map };
        }
        if input.cards_pressed {
            self.ui_mode = if self.ui_mode == UiMode::Cards { UiMode::None } else { UiMode::Cards };
        }
        match self.ui_mode {
            UiMode::Cards => {
                let pages = self.total_card_pages();
                if input.page_down_pressed {
                    self.card_page = (self.card_page + 1) % pages;
                }
                if input.page_up_pressed {
                    self.card_page = (self.card_page + pages - 1) % pages;
                }
                return;
            }
            UiMode::Map => return,
            UiMode::None => {}
        }

        self.handle_movement(input);
        if let Some((mx, my)) = input.click {
            let (cx, cy) = self.cam();
            self.move_target = Some((mx + cx, my + cy));
        }
        if input.space_pressed {
            self.break_nearest_outcrop();
        }
    }

    fn handle_movement(&mut self, input: &Input) {
        let dx = input.right as i32 - input.left as i32;
        let dy = input.up as i32 - input.down as i32;
        if dx != 0 || dy != 0 {
            self.move_target = None; // manual movement cancels click-to-move
            let mag = ((dx * dx + dy * dy) as f32).sqrt();
            self.player.x += dx as f32 / mag * PLAYER_SPEED;
            self.player.y += dy as f32 / mag * PLAYER_SPEED;
            if dx != 0 {
                self.player.facing = if dx < 0 { -1 } else { 1 };
            }
        } else if let Some((tx, ty)) = self.move_target {
            let (vx, vy) = (tx - self.player.x, ty - self.player.y);
            let dist = (vx * vx + vy * vy).sqrt();
            if dist < PLAYER_SPEED {
                self.player.x = tx;
                self.player.y = ty;
                self.move_target = None;
            } else {
                self.player.x += vx / dist * PLAYER_SPEED;
                self.player.y += vy / dist * PLAYER_SPEED;
                self.player.facing = if vx < 0.0 { -1 } else { 1 };
            }
        }
    }

    pub fn ensure_nearby_chunks_generated(&mut self) {
        let (pcx, pcy) = world_to_chunk(self.player.x, self.player.y);
        for dx in -VIEW_RADIUS_CHUNKS..=VIEW_RADIUS_CHUNKS {
            for dy in -VIEW_RADIUS_CHUNKS..=VIEW_RADIUS_CHUNKS {
                let key = (pcx + dx, pcy + dy);
                self.chunks.entry(key).or_insert_with(|| generate_chunk(key.0, key.1));
            }
        }
    }

    pub fn nearest_outcrop(&self) -> Option<&Outcrop> {
        let (pcx, pcy) = world_to_chunk(self.player.x, self.player.y);
        let mut best: Option<(&Outcrop, f32)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(chunk) = self.chunks.get(&(pcx + dx, pcy + dy)) {
                    for o in &chunk.outcrops {
                        let d = ((o.x - self.player.x).powi(2) + (o.y - self.player.y).powi(2)).sqrt();
                        if best.map_or(true, |(_, bd)| d < bd) {
                            best = Some((o, d));
                        }
                    }
                }
            }
        }
        best.filter(|(_, d)| *d <= BREAK_RANGE).map(|(o, _)| o)
    }

    pub fn break_nearest_outcrop(&mut self) {
        let Some(o) = self.nearest_outcrop().cloned() else { return };

        if !self.visited.insert(o.id.clone()) {
            self.spawn_popup(o.x, o.y, "Already surveyed", (130, 130, 130));
            return;
        }

        self.outcrop_points += 10;
        self.xp += 20;

        // GDD: flat 5% independent drop chance per category.
        let mineral = self.rng.below(100) < 5;
        let fossil = self.rng.below(100) < 5;
        let bonus_roll = self.rng.below(100) < 5;
        let bonus_rock = bonus_roll && self.inventory_count < INVENTORY_CAP;

        if mineral {
            self.xp += 10;
        }
        if fossil {
            self.xp += 10;
        }
        if bonus_rock {
            self.inventory_count += 1;
        }

        let name = card_name(&o);
        self.cards.push(Card {
            id: o.id.clone(),
            name: name.clone(),
            lithology: o.lithology,
            mineral,
            fossil,
            bonus_rock,
        });

        self.spawn_popup(o.x, o.y + 20.0, &format!("{name}! +10 pts / +20 xp"), (255, 255, 255));
        if mineral {
            self.spawn_popup(o.x, o.y + 46.0, "Rare mineral found! +10 xp", (240, 210, 90));
        }
        if fossil {
            self.spawn_popup(o.x, o.y + 72.0, "Fossil found! +10 xp", (190, 220, 255));
        }
        if bonus_rock {
            self.spawn_popup(o.x, o.y + 98.0, "Bonus rock collected", (200, 255, 200));
        }
    }

    fn spawn_popup(&mut self, x: f32, y: f32, text: &str, color: (u8, u8, u8)) {
        self.popups.push(Popup { x, y, text: text.to_string(), life: POPUP_LIFE, color });
    }

    /// 0.0 (full daylight) .. 1.0 (deepest night), smooth sine cycle.
    pub fn night_amount(&self) -> f32 {
        let phase = (self.tick_count % DAY_LENGTH_TICKS) as f32 / DAY_LENGTH_TICKS as f32;
        (((phase * std::f32::consts::TAU) - std::f32::consts::FRAC_PI_2).sin() + 1.0) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idle() -> Input {
        Input::default()
    }

    #[test]
    fn starts_with_gdd_defaults() {
        let g = Game::new(1);
        assert_eq!((g.outcrop_points, g.xp, g.inventory_count), (0, 10, 0));
    }

    #[test]
    fn walking_generates_chunks_and_normalizes_diagonals() {
        let mut g = Game::new(1);
        g.tick(&Input { right: true, up: true, ..idle() });
        let step = (g.player.x.powi(2) + g.player.y.powi(2)).sqrt();
        assert!((step - PLAYER_SPEED).abs() < 1e-4);
        assert_eq!(g.chunks.len(), 25);
        for _ in 0..2000 {
            g.tick(&Input { right: true, ..idle() });
        }
        assert!(g.chunks.len() > 25);
    }

    #[test]
    fn click_to_move_arrives_and_manual_input_cancels() {
        let mut g = Game::new(1);
        // click 200px right of screen center (y-up): world target (200, 0)
        g.tick(&Input { click: Some((SCREEN_W / 2.0 + 200.0, SCREEN_H / 2.0)), ..idle() });
        assert!(g.move_target.is_some());
        for _ in 0..80 {
            g.tick(&idle());
        }
        assert!((g.player.x - 200.0).abs() < 0.01 && g.move_target.is_none());

        g.tick(&Input { click: Some((0.0, 0.0)), ..idle() });
        assert!(g.move_target.is_some());
        g.tick(&Input { up: true, ..idle() });
        assert!(g.move_target.is_none());
    }

    fn walk_to_first_outcrop(g: &mut Game) -> Outcrop {
        g.ensure_nearby_chunks_generated();
        let o = g.chunks.values().flat_map(|c| c.outcrops.iter()).next().unwrap().clone();
        g.player.x = o.x;
        g.player.y = o.y;
        o
    }

    #[test]
    fn breaking_awards_once_then_marks_visited() {
        let mut g = Game::new(7);
        let o = walk_to_first_outcrop(&mut g);
        g.tick(&Input { space_pressed: true, ..idle() });
        assert_eq!(g.outcrop_points, 10);
        assert!(g.xp >= 30);
        assert_eq!(g.cards.len(), 1);
        assert!(g.visited.contains(&o.id));
        g.tick(&Input { space_pressed: true, ..idle() });
        assert_eq!((g.outcrop_points, g.cards.len()), (10, 1));
    }

    #[test]
    fn inventory_never_exceeds_cap_and_drop_rate_is_about_5_percent() {
        let mut g = Game::new(42);
        let mut minerals = 0;
        let n = 20_000;
        for i in 0..n {
            g.visited.clear();
            let o = Outcrop { id: format!("t{i}"), x: 0.0, y: 0.0, lithology: "Igneous" };
            g.chunks.insert((0, 0), Chunk { grass_variant: 0, outcrops: vec![o], trees: vec![], dirt_patches: vec![] });
            g.break_nearest_outcrop();
            minerals += g.cards.last().unwrap().mineral as u32;
            assert!(g.inventory_count <= INVENTORY_CAP);
        }
        let rate = minerals as f32 / n as f32;
        assert!((0.04..0.06).contains(&rate), "rate {rate}");
        assert_eq!(g.inventory_count, INVENTORY_CAP);
    }

    #[test]
    fn overlays_toggle_and_suspend_movement_and_page_wraps() {
        let mut g = Game::new(1);
        g.tick(&Input { map_pressed: true, ..idle() });
        assert_eq!(g.ui_mode, UiMode::Map);
        g.tick(&Input { right: true, ..idle() });
        assert_eq!(g.player.x, 0.0);
        g.tick(&Input { map_pressed: true, ..idle() });
        assert_eq!(g.ui_mode, UiMode::None);

        g.tick(&Input { cards_pressed: true, ..idle() });
        g.tick(&Input { page_up_pressed: true, ..idle() });
        assert_eq!(g.card_page, 0); // single page wraps onto itself
    }

    #[test]
    fn popups_expire_and_night_cycles() {
        let mut g = Game::new(1);
        g.spawn_popup(0.0, 0.0, "x", (0, 0, 0));
        for _ in 0..POPUP_LIFE {
            g.tick(&idle());
        }
        assert!(g.popups.is_empty());
        let mut g2 = Game::new(1);
        assert!(g2.night_amount() < 0.01);
        g2.tick_count = DAY_LENGTH_TICKS / 2;
        assert!(g2.night_amount() > 0.99);
    }
}
