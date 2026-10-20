//! Game state and rules. No rendering or windowing code lives here, so the
//! whole loop is unit-testable headlessly.
//!
//! Daily loop (GDD): wake at basecamp -> board truck -> truck drives along
//! main roads to the drop point -> explore and break outcrops -> reach a main
//! road before sunset (or lose) -> truck drives back -> sleep -> next morning.

use crate::world::*;
use std::collections::{HashMap, HashSet};

pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;
pub const PLAYER_W: f32 = 60.0;
pub const PLAYER_H: f32 = 80.0;
pub const PLAYER_SPEED: f32 = 4.2;
pub const BREAK_RANGE: f32 = 60.0;
pub const CARDS_PER_PAGE: usize = 8;
pub const INVENTORY_CAP: u32 = 10;
pub const POPUP_LIFE: i32 = 70;

// Time: the field day runs 05:00 -> 18:00 (sunset). 25 ticks = 1 game minute,
// so a full day of exploring is ~5.4 real minutes at 60 Hz.
pub const TICKS_PER_MINUTE: u32 = 25;
pub const DAY_START_MIN: u32 = 5 * 60;
pub const SUNSET_MIN: u32 = 18 * 60;
pub const SUPPLY_MIN: u32 = 12 * 60;

pub const TRUCK_SPEED: f32 = 16.0;
pub const BOARD_RANGE: f32 = 150.0;
pub const SLEEP_TICKS: u32 = 220;
pub const ENERGY_MAX: f32 = 100.0;
pub const ENERGY_DRAIN_PER_TICK: f32 = 0.008;
pub const SUPPLY_ENERGY: f32 = 50.0;
pub const DROP_SIDE_OFFSET: f32 = 140.0;

/// An action queued by tapping something far away: walk there, then do it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Pending {
    None,
    Break(String),
    Board,
}

pub const TAP_PICK_RADIUS: f32 = 55.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiMode {
    None,
    Map,
    Cards,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Morning at camp; walk to the truck and board it.
    Basecamp,
    /// Truck animation: basecamp -> drop point.
    TruckOut,
    /// Free exploration until sunset.
    Exploring,
    /// Truck animation: pickup point -> basecamp.
    TruckBack,
    /// Night at camp; energy restored, then the next day starts.
    Sleeping,
    /// Missed the main road before sunset.
    GameOver,
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
    pub interact_pressed: bool,
    pub restart_pressed: bool,
    pub page_down_pressed: bool,
    pub page_up_pressed: bool,
    /// Click position in 1280x720 virtual screen space, y-UP.
    pub click: Option<(f32, f32)>,
    /// Tap/click already resolved to a WORLD position (game units, y-up). Used by
    /// front-ends whose camera is not the fixed top-down one (e.g. the Bevy 3D prototype).
    pub world_click: Option<(f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct Card {
    #[allow(dead_code)]
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

pub struct Truck {
    pub x: f32,
    pub y: f32,
    /// Unit heading, axis-aligned: (1,0) east, (0,1) north (y-up) ...
    pub dx: f32,
    pub dy: f32,
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
    pub truck: Truck,
    pub phase: Phase,
    pub day: u32,
    /// Ticks of field time since 05:00 (only advances while exploring).
    pub clock_ticks: u32,
    pub energy: f32,
    pub supply_given: bool,
    pub route: Vec<(f32, f32)>,
    pub route_idx: usize,
    pub sleep_timer: u32,
    pub banner: Option<(String, i32)>,
    pub outcrop_points: u32,
    pub xp: u32,
    pub inventory_count: u32,
    pub popups: Vec<Popup>,
    pub cards: Vec<Card>,
    pub ui_mode: UiMode,
    pub card_page: usize,
    pub move_target: Option<(f32, f32)>,
    pub pending: Pending,
    pub tick_count: u32,
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub visited: HashSet<String>,
    pub rng: Rng,
    pub game_over_stats: Option<(u32, u32, usize)>, // day, points, cards
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut g = Game {
            player: Player { x: CAMP_SPAWN.0, y: CAMP_SPAWN.1, facing: 1 },
            truck: Truck { x: TRUCK_SPOT.0, y: TRUCK_SPOT.1, dx: 1.0, dy: 0.0 },
            phase: Phase::Basecamp,
            day: 1,
            clock_ticks: 0,
            energy: ENERGY_MAX,
            supply_given: false,
            route: Vec::new(),
            route_idx: 0,
            sleep_timer: 0,
            banner: None,
            outcrop_points: 0,
            xp: 10,
            inventory_count: 0,
            popups: Vec::new(),
            cards: Vec::new(),
            ui_mode: UiMode::None,
            card_page: 0,
            move_target: None,
            pending: Pending::None,
            tick_count: 0,
            chunks: HashMap::new(),
            visited: HashSet::new(),
            rng: Rng::new(seed),
            game_over_stats: None,
        };
        g.set_banner("Day 1 - Basecamp".to_string());
        g
    }

    fn set_banner(&mut self, text: String) {
        self.banner = Some((text, 180));
    }

    /// What the camera follows: the player on foot, the truck while riding.
    pub fn focus(&self) -> (f32, f32) {
        match self.phase {
            Phase::TruckOut | Phase::TruckBack => (self.truck.x, self.truck.y),
            _ => (self.player.x, self.player.y),
        }
    }

    pub fn cam(&self) -> (f32, f32) {
        let (fx, fy) = self.focus();
        (fx - SCREEN_W / 2.0, fy - SCREEN_H / 2.0)
    }

    pub fn player_visible(&self) -> bool {
        matches!(self.phase, Phase::Basecamp | Phase::Exploring)
    }

    pub fn total_card_pages(&self) -> usize {
        self.cards.len().div_ceil(CARDS_PER_PAGE).max(1)
    }

    pub fn minutes_of_day(&self) -> u32 {
        DAY_START_MIN + self.clock_ticks / TICKS_PER_MINUTE
    }

    pub fn minutes_to_sunset(&self) -> u32 {
        SUNSET_MIN.saturating_sub(self.minutes_of_day())
    }

    pub fn player_on_road(&self) -> bool {
        on_road(self.player.x, self.player.y, 0.0)
    }

    pub fn near_truck(&self) -> bool {
        let d = ((self.player.x - self.truck.x).powi(2) + (self.player.y - self.truck.y).powi(2)).sqrt();
        d <= BOARD_RANGE
    }

    pub fn tick(&mut self, input: &Input) {
        if self.phase == Phase::GameOver {
            if input.restart_pressed {
                let seed = self.rng.next_u32() as u64 ^ ((self.tick_count as u64) << 20);
                *self = Game::new(seed);
            }
            self.tick_count += 1;
            return;
        }

        self.handle_input(input);
        self.tick_count += 1;
        self.update_phase();
        self.ensure_nearby_chunks_generated();
        for p in &mut self.popups {
            p.y += 0.6;
            p.life -= 1;
        }
        self.popups.retain(|p| p.life > 0);
        if let Some((_, life)) = &mut self.banner {
            *life -= 1;
            if *life <= 0 {
                self.banner = None;
            }
        }
    }

    // ---------------------------------------------------------------- phases
    fn update_phase(&mut self) {
        match self.phase {
            Phase::TruckOut | Phase::TruckBack => self.drive_truck(),
            Phase::Exploring => self.update_exploring(),
            Phase::Sleeping => {
                self.sleep_timer += 1;
                if self.sleep_timer >= SLEEP_TICKS {
                    self.start_next_day();
                }
            }
            Phase::Basecamp | Phase::GameOver => {}
        }
    }

    fn drive_truck(&mut self) {
        let mut budget = TRUCK_SPEED;
        while budget > 0.0 && self.route_idx < self.route.len() {
            let (wx, wy) = self.route[self.route_idx];
            let (vx, vy) = (wx - self.truck.x, wy - self.truck.y);
            let dist = (vx * vx + vy * vy).sqrt();
            if dist > 0.01 {
                // roads are axis-aligned, so the heading is too
                if vx.abs() >= vy.abs() {
                    self.truck.dx = vx.signum();
                    self.truck.dy = 0.0;
                } else {
                    self.truck.dx = 0.0;
                    self.truck.dy = vy.signum();
                }
            }
            if dist <= budget {
                self.truck.x = wx;
                self.truck.y = wy;
                budget -= dist;
                self.route_idx += 1;
            } else {
                self.truck.x += vx / dist * budget;
                self.truck.y += vy / dist * budget;
                budget = 0.0;
            }
        }
        if self.route_idx >= self.route.len() {
            match self.phase {
                Phase::TruckOut => self.arrive_at_drop_point(),
                _ => self.arrive_at_camp(),
            }
        }
    }

    fn arrive_at_drop_point(&mut self) {
        self.phase = Phase::Exploring;
        self.player.x = self.truck.x + DROP_SIDE_OFFSET;
        self.player.y = self.truck.y;
        self.player.facing = 1;
        self.move_target = None;
        self.set_banner("Dropped off - explore, and be back on a main road before sunset".to_string());
    }

    fn arrive_at_camp(&mut self) {
        self.phase = Phase::Sleeping;
        self.sleep_timer = 0;
        self.truck.x = TRUCK_SPOT.0;
        self.truck.y = TRUCK_SPOT.1;
        self.truck.dx = 1.0;
        self.truck.dy = 0.0;
    }

    fn start_next_day(&mut self) {
        self.day += 1;
        self.clock_ticks = 0;
        self.energy = ENERGY_MAX;
        self.supply_given = false;
        self.player.x = CAMP_SPAWN.0;
        self.player.y = CAMP_SPAWN.1;
        self.move_target = None;
        self.phase = Phase::Basecamp;
        self.set_banner(format!("Day {} - Basecamp", self.day));
    }

    fn update_exploring(&mut self) {
        self.clock_ticks += 1;

        if !self.supply_given && self.minutes_of_day() >= SUPPLY_MIN {
            self.supply_given = true;
            self.energy = (self.energy + SUPPLY_ENERGY).min(ENERGY_MAX);
            self.set_banner(format!("12:00 - Supplies delivered! +{} energy", SUPPLY_ENERGY as i32));
        }

        if self.minutes_of_day() >= SUNSET_MIN {
            if self.player_on_road() {
                self.set_banner("Sunset - the truck picked you up just in time".to_string());
                self.start_truck_back();
            } else {
                self.phase = Phase::GameOver;
                self.game_over_stats = Some((self.day, self.outcrop_points, self.cards.len()));
                self.move_target = None;
            }
        }
    }

    fn start_truck_back(&mut self) {
        // the truck meets the player on the road centre line
        let (xv, yh) = (nearest_v_road(self.player.x), nearest_h_road(self.player.y));
        let on_h = (self.player.y - yh).abs() <= ROAD_W / 2.0;
        let (tx, ty) = if on_h { (self.player.x, yh) } else { (xv, self.player.y) };
        self.truck.x = tx;
        self.truck.y = ty;
        self.route = route_to_camp(tx, ty);
        self.route_idx = 0;
        self.move_target = None;
        self.phase = Phase::TruckBack;
    }

    fn start_truck_out(&mut self) {
        // Pick today's drop point: somewhere on a vertical main road, clear of the camp.
        let road_k = self.rng.below(3) as i32 - 1; // vertical road index -1..=1 around x=1600
        let xv = ROAD_V_OFFSET + road_k as f32 * ROAD_SPACING;
        let mag = 900.0 + self.rng.below(1600) as f32;
        let ty = if self.rng.below(2) == 0 { mag } else { -mag };
        self.route = route_from_camp(xv, ty);
        self.route_idx = 0;
        self.truck.x = TRUCK_SPOT.0;
        self.truck.y = TRUCK_SPOT.1;
        self.player.x = self.truck.x;
        self.player.y = self.truck.y;
        self.move_target = None;
        self.phase = Phase::TruckOut;
        self.set_banner("All aboard! Heading to the field...".to_string());
    }

    // ----------------------------------------------------------------- input
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

        // On foot only: basecamp and exploring.
        if !self.player_visible() {
            return;
        }

        self.handle_movement(input);
        if let Some((mx, my)) = input.click {
            let (cx, cy) = self.cam();
            self.on_tap(mx + cx, my + cy);
        }
        if let Some((wx, wy)) = input.world_click {
            self.on_tap(wx, wy);
        }
        self.resolve_pending();
        if input.interact_pressed {
            self.interact();
        }
        if input.space_pressed && self.phase == Phase::Exploring {
            self.break_nearest_outcrop();
        }
    }

    /// Minecraft-style tap: tapping an outcrop breaks it (walking over first if
    /// needed), tapping the truck boards it, tapping bare ground walks there.
    fn on_tap(&mut self, wx: f32, wy: f32) {
        self.pending = Pending::None;
        let dist = |ax: f32, ay: f32, bx: f32, by: f32| ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();

        if self.phase == Phase::Exploring {
            if let Some(o) = self.outcrop_at(wx, wy) {
                if dist(o.x, o.y, self.player.x, self.player.y) <= BREAK_RANGE {
                    self.break_outcrop(&o);
                } else {
                    // stand just in front of it (on the player's side), then break
                    let (vx, vy) = (self.player.x - o.x, self.player.y - o.y);
                    let len = (vx * vx + vy * vy).sqrt().max(1.0);
                    self.move_target = Some((o.x + vx / len * 35.0, o.y + vy / len * 35.0));
                    self.pending = Pending::Break(o.id);
                }
                return;
            }
        }
        if self.phase == Phase::Basecamp && dist(self.truck.x, self.truck.y, wx, wy) <= 100.0 {
            if self.near_truck() {
                self.start_truck_out();
            } else {
                self.move_target = Some((self.truck.x, self.truck.y + 70.0));
                self.pending = Pending::Board;
            }
            return;
        }
        self.move_target = Some((wx, wy));
    }

    fn resolve_pending(&mut self) {
        match self.pending.clone() {
            Pending::None => {}
            Pending::Break(id) => {
                let target = self.find_outcrop(&id).cloned();
                match target {
                    Some(o) if ((o.x - self.player.x).powi(2) + (o.y - self.player.y).powi(2)).sqrt() <= BREAK_RANGE => {
                        self.pending = Pending::None;
                        self.break_outcrop(&o);
                    }
                    _ if self.move_target.is_none() => self.pending = Pending::None, // arrived/cancelled without reaching it
                    _ => {}
                }
            }
            Pending::Board => {
                if self.near_truck() {
                    self.pending = Pending::None;
                    self.start_truck_out();
                } else if self.move_target.is_none() {
                    self.pending = Pending::None;
                }
            }
        }
    }

    fn outcrop_at(&self, wx: f32, wy: f32) -> Option<Outcrop> {
        let (cx, cy) = world_to_chunk(wx, wy);
        let mut best: Option<(Outcrop, f32)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(chunk) = self.chunks.get(&(cx + dx, cy + dy)) {
                    for o in &chunk.outcrops {
                        let d = ((o.x - wx).powi(2) + (o.y - wy).powi(2)).sqrt();
                        if d <= TAP_PICK_RADIUS && best.as_ref().map_or(true, |(_, bd)| d < *bd) {
                            best = Some((o.clone(), d));
                        }
                    }
                }
            }
        }
        best.map(|(o, _)| o)
    }

    fn find_outcrop(&self, id: &str) -> Option<&Outcrop> {
        self.chunks.values().flat_map(|c| c.outcrops.iter()).find(|o| o.id == id)
    }

    fn interact(&mut self) {
        match self.phase {
            Phase::Basecamp if self.near_truck() => self.start_truck_out(),
            Phase::Exploring if self.player_on_road() => {
                self.set_banner("Truck called - heading back to basecamp".to_string());
                self.start_truck_back();
            }
            _ => {}
        }
    }

    fn current_speed(&self) -> f32 {
        if self.energy <= 0.0 { PLAYER_SPEED * 0.5 } else { PLAYER_SPEED }
    }

    fn handle_movement(&mut self, input: &Input) {
        let speed = self.current_speed();
        let (ox, oy) = (self.player.x, self.player.y);
        let dx = input.right as i32 - input.left as i32;
        let dy = input.up as i32 - input.down as i32;
        if dx != 0 || dy != 0 {
            self.move_target = None; // manual movement cancels click-to-move
            self.pending = Pending::None;
            let mag = ((dx * dx + dy * dy) as f32).sqrt();
            self.player.x += dx as f32 / mag * speed;
            self.player.y += dy as f32 / mag * speed;
            if dx != 0 {
                self.player.facing = if dx < 0 { -1 } else { 1 };
            }
        } else if let Some((tx, ty)) = self.move_target {
            let (vx, vy) = (tx - self.player.x, ty - self.player.y);
            let dist = (vx * vx + vy * vy).sqrt();
            if dist < speed {
                self.player.x = tx;
                self.player.y = ty;
                self.move_target = None;
            } else {
                self.player.x += vx / dist * speed;
                self.player.y += vy / dist * speed;
                self.player.facing = if vx < 0.0 { -1 } else { 1 };
            }
        }

        if self.phase == Phase::Basecamp {
            // on foot at camp you can wander the camp and the road in front of it
            self.player.x = self.player.x.clamp(CAMP.0 - 120.0, CAMP.2 + 120.0);
            self.player.y = self.player.y.clamp(-ROAD_W / 2.0 - 40.0, CAMP.3 + 60.0);
        } else if self.phase == Phase::Exploring {
            let moved = ((self.player.x - ox).powi(2) + (self.player.y - oy).powi(2)).sqrt();
            self.energy = (self.energy - moved * ENERGY_DRAIN_PER_TICK / PLAYER_SPEED).max(0.0);
        }
    }

    pub fn ensure_nearby_chunks_generated(&mut self) {
        let (fx, fy) = self.focus();
        let (pcx, pcy) = world_to_chunk(fx, fy);
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
        self.break_outcrop(&o);
    }

    pub fn break_outcrop(&mut self, o: &Outcrop) {
        let o = o.clone();
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

    /// 0.0 (full daylight) .. 1.0 (deepest night), from the field clock:
    /// dim dawn around 05:00, full day 07:00-16:30, dusk darkening to sunset.
    pub fn night_amount(&self) -> f32 {
        let m = self.minutes_of_day() as f32;
        let n = if m < 7.0 * 60.0 {
            0.55 * (1.0 - (m - DAY_START_MIN as f32) / 120.0)
        } else if m < 16.5 * 60.0 {
            0.0
        } else {
            0.6 * ((m - 16.5 * 60.0) / 90.0)
        };
        if self.phase == Phase::Sleeping { 0.85 } else { n.clamp(0.0, 0.85) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idle() -> Input {
        Input::default()
    }
    fn press_e() -> Input {
        Input { interact_pressed: true, ..idle() }
    }

    /// Put the game into the exploring phase at a given spot (skipping the ride).
    fn exploring_at(g: &mut Game, x: f32, y: f32) {
        g.phase = Phase::Exploring;
        g.player.x = x;
        g.player.y = y;
        g.ensure_nearby_chunks_generated();
    }

    fn run_ride(g: &mut Game) -> u32 {
        let mut n = 0;
        while matches!(g.phase, Phase::TruckOut | Phase::TruckBack) {
            g.tick(&idle());
            assert!(on_road(g.truck.x, g.truck.y, 0.5), "truck left the road at {},{}", g.truck.x, g.truck.y);
            n += 1;
            assert!(n < 5000, "ride never finished");
        }
        n
    }

    #[test]
    fn starts_at_basecamp_with_gdd_defaults() {
        let g = Game::new(1);
        assert_eq!((g.outcrop_points, g.xp, g.inventory_count), (0, 10, 0));
        assert_eq!((g.phase, g.day), (Phase::Basecamp, 1));
        assert!(in_camp(g.player.x, g.player.y, 0.0));
        assert_eq!(g.minutes_of_day(), 5 * 60);
    }

    #[test]
    fn cannot_board_when_far_from_truck_or_break_rocks_at_camp() {
        let mut g = Game::new(1);
        g.tick(&press_e()); // spawn point is ~230 away from the truck
        assert_eq!(g.phase, Phase::Basecamp);
        g.tick(&Input { space_pressed: true, ..idle() });
        assert_eq!(g.cards.len(), 0);
    }

    #[test]
    fn full_day_loop_with_pickup() {
        let mut g = Game::new(5);
        // walk to the truck then board
        g.player.x = g.truck.x;
        g.player.y = g.truck.y + 60.0;
        g.tick(&press_e());
        assert_eq!(g.phase, Phase::TruckOut);
        assert!(!g.player_visible());
        let ride = run_ride(&mut g);
        assert!(ride > 100);
        assert_eq!(g.phase, Phase::Exploring);
        assert!(!g.player_on_road()); // dropped beside the road, not on it
        assert!(road_distance(g.player.x, g.player.y) < 200.0);
        assert!(g.chunks.len() >= 25);

        // explore a bit: clock advances, energy drains while walking
        let e0 = g.energy;
        for _ in 0..200 {
            g.tick(&Input { right: true, ..idle() });
        }
        assert!(g.clock_ticks >= 200 && g.energy < e0);

        // can't call the truck from off-road
        exploring_at(&mut g, 700.0, 700.0);
        g.tick(&press_e());
        assert_eq!(g.phase, Phase::Exploring);

        // walk onto the road and call the truck
        exploring_at(&mut g, 1600.0, 900.0);
        g.tick(&press_e());
        assert_eq!(g.phase, Phase::TruckBack);
        run_ride(&mut g);
        assert_eq!(g.phase, Phase::Sleeping);
        assert_eq!((g.truck.x, g.truck.y), TRUCK_SPOT);

        g.energy = 3.0;
        for _ in 0..SLEEP_TICKS + 2 {
            g.tick(&idle());
        }
        assert_eq!((g.phase, g.day), (Phase::Basecamp, 2));
        assert_eq!(g.energy, ENERGY_MAX);
        assert_eq!(g.minutes_of_day(), 5 * 60);
        assert!(in_camp(g.player.x, g.player.y, 0.0));
    }

    #[test]
    fn missing_the_road_at_sunset_is_game_over_then_restart() {
        let mut g = Game::new(9);
        exploring_at(&mut g, 700.0, 700.0);
        g.outcrop_points = 40;
        let total = (SUNSET_MIN - DAY_START_MIN) * TICKS_PER_MINUTE;
        for _ in 0..total + 5 {
            g.tick(&idle());
        }
        assert_eq!(g.phase, Phase::GameOver);
        assert_eq!(g.game_over_stats.map(|s| s.1), Some(40));

        // ignores everything but restart
        g.tick(&Input { right: true, ..idle() });
        assert_eq!(g.phase, Phase::GameOver);
        g.tick(&Input { restart_pressed: true, ..idle() });
        assert_eq!((g.phase, g.day, g.outcrop_points), (Phase::Basecamp, 1, 0));
    }

    #[test]
    fn being_on_the_road_at_sunset_gets_you_picked_up() {
        let mut g = Game::new(9);
        exploring_at(&mut g, 1600.0 + 30.0, 1200.0);
        g.clock_ticks = (SUNSET_MIN - DAY_START_MIN) * TICKS_PER_MINUTE - 3;
        for _ in 0..6 {
            g.tick(&idle());
        }
        assert_eq!(g.phase, Phase::TruckBack);
        run_ride(&mut g);
        assert_eq!(g.phase, Phase::Sleeping);
    }

    #[test]
    fn noon_supply_restores_energy_once() {
        let mut g = Game::new(3);
        exploring_at(&mut g, 700.0, 700.0);
        g.energy = 10.0;
        g.clock_ticks = (SUPPLY_MIN - DAY_START_MIN) * TICKS_PER_MINUTE - 1;
        g.tick(&idle());
        g.tick(&idle());
        assert!((g.energy - 60.0).abs() < 0.5, "{}", g.energy);
        let e = g.energy;
        for _ in 0..500 {
            g.tick(&idle());
        }
        assert!((g.energy - e).abs() < 0.01);
    }

    #[test]
    fn exhausted_player_moves_at_half_speed() {
        let mut g = Game::new(3);
        exploring_at(&mut g, 700.0, 700.0);
        g.energy = 0.0;
        g.tick(&Input { right: true, ..idle() });
        assert!((g.player.x - (700.0 + PLAYER_SPEED / 2.0)).abs() < 1e-3);
    }

    #[test]
    fn drop_points_vary_but_always_land_next_to_a_road() {
        let mut seen = HashSet::new();
        for seed in 1..40u64 {
            let mut g = Game::new(seed * 7919);
            g.player.x = g.truck.x;
            g.player.y = g.truck.y;
            g.tick(&press_e());
            run_ride(&mut g);
            assert_eq!(g.phase, Phase::Exploring);
            assert!(road_distance(g.player.x, g.player.y) <= DROP_SIDE_OFFSET + 1.0);
            seen.insert((g.truck.x as i32, g.truck.y as i32));
        }
        assert!(seen.len() > 3);
    }

    #[test]
    fn walking_generates_chunks_and_normalizes_diagonals() {
        let mut g = Game::new(1);
        exploring_at(&mut g, 700.0, 700.0);
        let (x0, y0) = (g.player.x, g.player.y);
        g.tick(&Input { right: true, up: true, ..idle() });
        let step = ((g.player.x - x0).powi(2) + (g.player.y - y0).powi(2)).sqrt();
        assert!((step - PLAYER_SPEED).abs() < 1e-4);
        let n = g.chunks.len();
        for _ in 0..2000 {
            g.tick(&Input { right: true, ..idle() });
        }
        assert!(g.chunks.len() > n);
    }

    #[test]
    fn click_to_move_arrives_and_manual_input_cancels() {
        let mut g = Game::new(1);
        exploring_at(&mut g, 700.0, 700.0);
        g.tick(&Input { click: Some((SCREEN_W / 2.0 + 200.0, SCREEN_H / 2.0)), ..idle() });
        assert!(g.move_target.is_some());
        for _ in 0..80 {
            g.tick(&idle());
        }
        assert!((g.player.x - 900.0).abs() < 0.01 && g.move_target.is_none());

        g.tick(&Input { click: Some((0.0, 0.0)), ..idle() });
        assert!(g.move_target.is_some());
        g.tick(&Input { up: true, ..idle() });
        assert!(g.move_target.is_none());
    }

    fn walk_to_first_outcrop(g: &mut Game) -> Outcrop {
        exploring_at(g, 700.0, 700.0);
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
        g.player.x = 0.0;
        g.player.y = 0.0;
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
        let x0 = g.player.x;
        g.tick(&Input { map_pressed: true, ..idle() });
        assert_eq!(g.ui_mode, UiMode::Map);
        g.tick(&Input { right: true, ..idle() });
        assert_eq!(g.player.x, x0);
        g.tick(&Input { map_pressed: true, ..idle() });
        assert_eq!(g.ui_mode, UiMode::None);

        g.tick(&Input { cards_pressed: true, ..idle() });
        g.tick(&Input { page_up_pressed: true, ..idle() });
        assert_eq!(g.card_page, 0);
    }

    #[test]
    fn camp_walk_is_bounded() {
        let mut g = Game::new(1);
        for _ in 0..3000 {
            g.tick(&Input { left: true, down: true, ..idle() });
        }
        assert!(g.player.x >= CAMP.0 - 120.0 - 0.01 && g.player.y >= -ROAD_W / 2.0 - 40.0 - 0.01);
    }

    #[test]
    fn popups_expire_and_night_follows_the_field_clock() {
        let mut g = Game::new(1);
        g.spawn_popup(0.0, 0.0, "x", (0, 0, 0));
        for _ in 0..POPUP_LIFE {
            g.tick(&idle());
        }
        assert!(g.popups.is_empty());
        g.phase = Phase::Exploring;
        g.clock_ticks = 0;
        let dawn = g.night_amount();
        g.clock_ticks = 4 * 60 * TICKS_PER_MINUTE; // 09:00
        assert!(dawn > 0.5 && g.night_amount() == 0.0);
        g.clock_ticks = (SUNSET_MIN - DAY_START_MIN) * TICKS_PER_MINUTE - 1;
        assert!(g.night_amount() > 0.5);
    }

    fn outcrop_screen_click(g: &Game, o: &Outcrop) -> (f32, f32) {
        let (cx, cy) = g.cam();
        (o.x - cx, o.y - cy)
    }

    #[test]
    fn tapping_a_nearby_outcrop_breaks_it_without_moving() {
        let mut g = Game::new(7);
        let o = walk_to_first_outcrop(&mut g);
        g.player.x = o.x + 20.0;
        let click = outcrop_screen_click(&g, &o);
        let x0 = g.player.x;
        g.tick(&Input { click: Some(click), ..idle() });
        assert_eq!(g.cards.len(), 1);
        assert_eq!(g.player.x, x0);
        assert!(g.move_target.is_none());
    }

    #[test]
    fn tapping_a_far_outcrop_walks_there_then_breaks() {
        let mut g = Game::new(7);
        let o = walk_to_first_outcrop(&mut g);
        g.player.x = o.x - 300.0;
        g.player.y = o.y;
        g.ensure_nearby_chunks_generated();
        let click = outcrop_screen_click(&g, &o);
        g.tick(&Input { click: Some(click), ..idle() });
        assert_eq!(g.pending, Pending::Break(o.id.clone()));
        assert_eq!(g.cards.len(), 0);
        for _ in 0..120 {
            g.tick(&idle());
        }
        assert_eq!(g.cards.len(), 1);
        assert_eq!(g.pending, Pending::None);
    }

    #[test]
    fn manual_movement_cancels_a_queued_break() {
        let mut g = Game::new(7);
        let o = walk_to_first_outcrop(&mut g);
        g.player.x = o.x - 300.0;
        g.player.y = o.y;
        g.ensure_nearby_chunks_generated();
        let click = outcrop_screen_click(&g, &o);
        g.tick(&Input { click: Some(click), ..idle() });
        g.tick(&Input { down: true, ..idle() });
        assert_eq!(g.pending, Pending::None);
        for _ in 0..200 {
            g.tick(&idle());
        }
        assert_eq!(g.cards.len(), 0);
    }

    #[test]
    fn tapping_the_truck_walks_over_and_boards() {
        let mut g = Game::new(11);
        let (cx, cy) = g.cam();
        g.tick(&Input { click: Some((g.truck.x - cx, g.truck.y - cy)), ..idle() });
        assert_eq!(g.pending, Pending::Board);
        for _ in 0..120 {
            g.tick(&idle());
            if g.phase == Phase::TruckOut {
                break;
            }
        }
        assert_eq!(g.phase, Phase::TruckOut);
        assert_eq!(g.pending, Pending::None);
    }

    #[test]
    fn tapping_bare_ground_still_just_walks() {
        let mut g = Game::new(1);
        exploring_at(&mut g, 700.0, 700.0);
        // a spot with no outcrop within pick radius
        g.chunks.values_mut().for_each(|c| c.outcrops.clear());
        g.tick(&Input { click: Some((SCREEN_W / 2.0 + 120.0, SCREEN_H / 2.0)), ..idle() });
        assert_eq!(g.pending, Pending::None);
        assert!(g.move_target.is_some());
    }

    #[test]
    fn world_click_behaves_like_a_screen_click_at_the_same_spot() {
        let mut a = Game::new(7);
        let o = walk_to_first_outcrop(&mut a);
        a.player.x = o.x + 20.0;
        a.tick(&Input { world_click: Some((o.x, o.y)), ..idle() });
        assert_eq!(a.cards.len(), 1);

        let mut b = Game::new(1);
        exploring_at(&mut b, 700.0, 700.0);
        b.chunks.values_mut().for_each(|c| c.outcrops.clear());
        b.tick(&Input { world_click: Some((900.0, 700.0)), ..idle() });
        assert_eq!(b.move_target, Some((900.0, 700.0)));
    }
}
