//! Game state and rules. No rendering or windowing code lives here, so the
//! whole loop is unit-testable headlessly.
//!
//! Daily loop (GDD): wake at basecamp -> board truck -> truck drives along
//! main roads to the drop point -> explore and break outcrops -> reach a main
//! road before sunset (or lose) -> truck drives back -> sleep -> next morning.

use crate::geology::{Region, REGIONS};
use crate::kit::*;
use crate::terrain::{effort_factor, speed_factor, Ground, Terrain};
use crate::world::*;
use std::collections::{HashMap, HashSet};

pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;
pub const PLAYER_W: f32 = 60.0;
pub const PLAYER_H: f32 = 80.0;
pub const PLAYER_SPEED: f32 = 4.2;
pub const BREAK_RANGE: f32 = 60.0;
pub const CARDS_PER_PAGE: usize = 8;
pub const POPUP_LIFE: i32 = 70;

// Time: the field day runs 05:00 -> 18:00 (sunset). 25 ticks = 1 game minute,
// so a full day of exploring is ~5.4 real minutes at 60 Hz.
pub const TICKS_PER_MINUTE: u32 = 25;
pub const DAY_START_MIN: u32 = 5 * 60;
pub const SUNSET_MIN: u32 = 18 * 60;
/// Lunch time: the cue to find a tree and eat something.
pub const SUPPLY_MIN: u32 = 12 * 60;

pub const TRUCK_SPEED: f32 = 16.0;
pub const BOARD_RANGE: f32 = 150.0;
pub const SLEEP_TICKS: u32 = 220;
pub const ENERGY_MAX: f32 = 100.0;
pub const ENERGY_DRAIN_PER_TICK: f32 = 0.008;
pub const DROP_SIDE_OFFSET: f32 = 140.0;

/// An action queued by tapping something far away: walk there, then do it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Pending {
    None,
    Break(String),
    Board,
    /// Walk to water, then fill the bottle.
    Fill,
    /// Walk to a tree, then eat in its shade.
    Eat,
}

pub const TAP_PICK_RADIUS: f32 = 55.0;

/// Overlay panel (x, y-bottom, w, h) in the 1280x720 y-up virtual screen.
pub const PANEL: (f32, f32, f32, f32) = (190.0, 60.0, 900.0, 600.0);
pub const PACK_ROW_H: f32 = 50.0;
/// Bottom-left rect of backpack menu row `i`.
pub fn pack_row_rect(i: usize) -> (f32, f32, f32, f32) {
    let (px, py, pw, ph) = PANEL;
    (px + 24.0, py + ph - 120.0 - 56.0 * i as f32, pw - 48.0, PACK_ROW_H)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiMode {
    None,
    Map,
    Cards,
    Backpack,
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
    pub pack_pressed: bool,
    /// Context action: fill the bottle at water, eat under a tree.
    pub use_pressed: bool,
    pub space_pressed: bool,
    pub interact_pressed: bool,
    pub restart_pressed: bool,
    pub page_down_pressed: bool,
    pub page_up_pressed: bool,
    /// Click position in 1280x720 virtual screen space, y-UP.
    pub click: Option<(f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct Card {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    #[allow(dead_code)]
    pub lithology: &'static str,
    #[allow(dead_code)]
    pub rock: &'static str,
    /// Lithostratigraphic unit it came from, and that unit's age.
    pub unit: &'static str,
    pub parent: &'static str,
    pub age: &'static str,
    #[allow(dead_code)]
    pub region: &'static str,
    /// Name of the mineral / fossil found with it, if any.
    pub mineral: Option<&'static str>,
    pub fossil: Option<&'static str>,
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
    /// Lunch-time cue already shown today.
    pub supply_given: bool,
    pub route: Vec<(f32, f32)>,
    pub route_idx: usize,
    pub sleep_timer: u32,
    pub banner: Option<(String, i32)>,
    pub outcrop_points: u32,
    pub xp: u32,
    /// Bonus rocks carried (they share the backpack's slots with food).
    pub inventory_count: u32,
    pub water: f32,
    pub water_warned: bool,
    pub pickaxe: Pickaxe,
    pub apples: u32,
    pub sandwiches: u32,
    pub pack_sel: usize,
    pub pack_msg: Option<(String, i32)>,
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
    pub terrain: Terrain,
    /// Current map: index into `REGIONS` and the region itself.
    pub level: usize,
    pub region: &'static Region,
    /// The survey quota is met; the next morning moves on to the next map.
    pub level_done: bool,
    /// Outcrops surveyed on this map.
    pub surveyed: u32,
    /// Grade (rise/run) of the last step taken; 0 when standing still.
    pub move_grade: f32,
    block_cooldown: u32,
    pub game_over_stats: Option<(u32, u32, usize)>, // day, points, cards
}

impl Game {
    /// A new game on the first map.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(seed: u64) -> Self {
        Self::with_level(seed, 0)
    }

    pub fn with_level(seed: u64, level: usize) -> Self {
        let level = level.min(REGIONS.len() - 1);
        Self::build(seed, level, REGIONS[level].terrain())
    }

    /// First map's geology on a custom terrain (tests, experiments).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_terrain(seed: u64, terrain: Terrain) -> Self {
        Self::build(seed, 0, terrain)
    }

    fn build(seed: u64, level: usize, terrain: Terrain) -> Self {
        let mut g = Game {
            level,
            region: &REGIONS[level],
            level_done: false,
            surveyed: 0,
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
            water: WATER_MAX,
            water_warned: false,
            pickaxe: Pickaxe::new(0),
            apples: 0,
            sandwiches: 0,
            pack_sel: 0,
            pack_msg: None,
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
            terrain,
            move_grade: 0.0,
            block_cooldown: 0,
            game_over_stats: None,
        };
        g.issue_daily_kit();
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
                let terrain = self.terrain.clone();
                *self = Game::build(seed, self.level, terrain);
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
        if let Some((_, life)) = &mut self.pack_msg {
            *life -= 1;
            if *life <= 0 {
                self.pack_msg = None;
            }
        }
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
        if self.level_done {
            self.advance_level();
        }
        self.clock_ticks = 0;
        self.energy = ENERGY_MAX;
        self.supply_given = false;
        self.issue_daily_kit();
        self.player.x = CAMP_SPAWN.0;
        self.player.y = CAMP_SPAWN.1;
        self.move_target = None;
        self.phase = Phase::Basecamp;
        let banner = if self.surveyed == 0 && self.level > 0 {
            format!("Day {} - New field area: {} ({})", self.day, self.region.name, self.region.place)
        } else {
            format!("Day {} - Basecamp", self.day)
        };
        self.set_banner(banner);
    }

    fn update_exploring(&mut self) {
        self.clock_ticks += 1;

        if !self.supply_given && self.minutes_of_day() >= SUPPLY_MIN {
            self.supply_given = true;
            self.set_banner("12:00 - Lunch break: find a tree and eat something".to_string());
        }

        // the bottle slowly empties out in the field
        let before = self.water;
        self.water = (self.water - WATER_DRAIN_IDLE).max(0.0);
        self.warn_if_thirsty(before);

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
        let ty = match self.terrain.bounds {
            // bounded map: anywhere along the road inside the map, but not right by the camp
            Some((_, by0, _, by1)) => {
                let (lo, hi) = (by0 + 200.0, by1 - 200.0);
                let (neg, pos) = ((-900.0 - lo).max(0.0), (hi - 900.0).max(0.0));
                let r = self.rng.below((neg + pos) as u32) as f32;
                if r < neg { -900.0 - r } else { 900.0 + (r - neg) }
            }
            None => {
                let mag = 900.0 + self.rng.below(1600) as f32;
                if self.rng.below(2) == 0 { mag } else { -mag }
            }
        };
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
        if input.pack_pressed && (self.ui_mode == UiMode::Backpack || (self.ui_mode == UiMode::None && self.player_visible())) {
            self.ui_mode = if self.ui_mode == UiMode::Backpack { UiMode::None } else { UiMode::Backpack };
            self.pack_sel = 0;
            self.pack_msg = None;
        }
        match self.ui_mode {
            UiMode::Backpack => {
                self.handle_backpack_input(input);
                return;
            }
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
        self.resolve_pending();
        if input.interact_pressed {
            self.interact();
        }
        if input.use_pressed {
            self.use_context();
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
        if self.phase == Phase::Exploring {
            if self.terrain.ground(wx, wy).is_water() {
                if self.near_water() {
                    self.fill_bottle();
                } else {
                    self.move_target = Some((wx, wy));
                    self.pending = Pending::Fill;
                }
                return;
            }
            if let Some((tx, ty)) = self.tree_at(wx, wy) {
                if self.near_tree() {
                    self.eat_best();
                } else {
                    let (vx, vy) = (self.player.x - tx, self.player.y - ty);
                    let len = (vx * vx + vy * vy).sqrt().max(1.0);
                    self.move_target = Some((tx + vx / len * 60.0, ty + vy / len * 60.0));
                    self.pending = Pending::Eat;
                }
                return;
            }
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
            Pending::Fill => {
                if self.near_water() {
                    self.pending = Pending::None;
                    self.fill_bottle();
                } else if self.move_target.is_none() {
                    self.pending = Pending::None;
                }
            }
            Pending::Eat => {
                if self.near_tree() {
                    self.pending = Pending::None;
                    self.eat_best();
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
        let mut s = PLAYER_SPEED;
        if self.energy <= 0.0 {
            s *= 0.5;
        }
        if self.phase == Phase::Exploring && self.water <= 0.0 {
            s *= DEHYDRATED_SPEED;
        }
        s
    }

    /// Elevation (m) under the player.
    pub fn elevation_here(&self) -> f32 {
        self.terrain.elevation(self.player.x, self.player.y)
    }

    pub fn ground_here(&self) -> Ground {
        self.terrain.ground(self.player.x, self.player.y)
    }

    /// Move from (px, py) to (nx, ny), sliding along obstacles. Returns the
    /// reached position and the ground that stopped us, if any.
    fn collide(&self, px: f32, py: f32, nx: f32, ny: f32) -> ((f32, f32), Option<Ground>) {
        let g = self.terrain.ground(nx, ny);
        if !g.blocked() {
            return ((nx, ny), None);
        }
        // slide along the axis that moves most first
        let order = if (nx - px).abs() >= (ny - py).abs() { [(nx, py), (px, ny)] } else { [(px, ny), (nx, py)] };
        for (sx, sy) in order {
            if (sx, sy) != (px, py) && !self.terrain.blocked(sx, sy) {
                return ((sx, sy), None);
            }
        }
        ((px, py), Some(g))
    }

    fn handle_movement(&mut self, input: &Input) {
        self.block_cooldown = self.block_cooldown.saturating_sub(1);
        self.move_grade = 0.0;
        let base = self.current_speed();
        let (ox, oy) = (self.player.x, self.player.y);
        let dx = input.right as i32 - input.left as i32;
        let dy = input.up as i32 - input.down as i32;

        // desired unit direction and, for click-to-move, the distance left
        let mut want: Option<(f32, f32, Option<f32>)> = None;
        if dx != 0 || dy != 0 {
            self.move_target = None; // manual movement cancels click-to-move
            self.pending = Pending::None;
            let mag = ((dx * dx + dy * dy) as f32).sqrt();
            want = Some((dx as f32 / mag, dy as f32 / mag, None));
            if dx != 0 {
                self.player.facing = if dx < 0 { -1 } else { 1 };
            }
        } else if let Some((tx, ty)) = self.move_target {
            let (vx, vy) = (tx - ox, ty - oy);
            let dist = (vx * vx + vy * vy).sqrt();
            if dist <= 1e-4 {
                self.move_target = None;
            } else {
                want = Some((vx / dist, vy / dist, Some(dist)));
                self.player.facing = if vx < 0.0 { -1 } else { 1 };
            }
        }

        let on_foot_in_field = self.phase == Phase::Exploring;
        let mut effort = 1.0;
        if let Some((ux, uy, left)) = want {
            let (mut factor, mut grade) = (1.0, 0.0);
            if on_foot_in_field {
                grade = self.terrain.grade_toward(ox, oy, ux, uy);
                factor = speed_factor(grade) * self.ground_here().surface_speed();
                effort = effort_factor(grade);
            }
            let step = (base * factor).min(left.unwrap_or(f32::MAX));
            let (nx, ny) = (ox + ux * step, oy + uy * step);
            let ((fx, fy), stopped_by) = if on_foot_in_field { self.collide(ox, oy, nx, ny) } else { ((nx, ny), None) };
            self.player.x = fx;
            self.player.y = fy;
            self.move_grade = grade;
            if let Some(g) = stopped_by {
                // fully blocked: give up the queued walk and say why
                self.move_target = None;
                self.pending = Pending::None;
                if self.block_cooldown == 0 {
                    self.block_cooldown = 90;
                    self.spawn_popup(ox, oy + 60.0, g.block_message(), (255, 190, 120));
                }
            } else if left.is_some_and(|d| step >= d - 1e-4) && (fx, fy) == (nx, ny) {
                self.move_target = None; // arrived
            }
        }

        if self.phase == Phase::Basecamp {
            // on foot at camp you can wander the camp and the road in front of it
            self.player.x = self.player.x.clamp(CAMP.0 - 120.0, CAMP.2 + 120.0);
            self.player.y = self.player.y.clamp(-ROAD_W / 2.0 - 40.0, CAMP.3 + 60.0);
        } else if on_foot_in_field {
            let moved = ((self.player.x - ox).powi(2) + (self.player.y - oy).powi(2)).sqrt();
            let thirst = if self.water <= 0.0 { DEHYDRATED_EFFORT } else { 1.0 };
            self.energy = (self.energy - moved * ENERGY_DRAIN_PER_TICK / PLAYER_SPEED * effort * thirst).max(0.0);
            let before = self.water;
            self.water = (self.water - moved / PLAYER_SPEED * WATER_DRAIN_WALK * effort).max(0.0);
            self.warn_if_thirsty(before);
        }
    }

    pub fn ensure_nearby_chunks_generated(&mut self) {
        let (fx, fy) = self.focus();
        let (pcx, pcy) = world_to_chunk(fx, fy);
        for dx in -VIEW_RADIUS_CHUNKS..=VIEW_RADIUS_CHUNKS {
            for dy in -VIEW_RADIUS_CHUNKS..=VIEW_RADIUS_CHUNKS {
                let key = (pcx + dx, pcy + dy);
                self.chunks.entry(key).or_insert_with(|| generate_chunk(key.0, key.1, &self.terrain, self.region));
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
        if self.visited.contains(&o.id) {
            self.spawn_popup(o.x, o.y, "Already surveyed", (130, 130, 130));
            return;
        }
        if self.pickaxe.broken() {
            self.spawn_popup(o.x, o.y + 20.0, "Pickaxe broken! Open the pack (B) for a new one", (255, 170, 120));
            return;
        }
        self.visited.insert(o.id.clone());

        self.outcrop_points += 10;
        self.xp += 20;

        // What turns up depends on the unit the outcrop belongs to: ore-bearing
        // units give minerals more often, only fossiliferous units give fossils.
        // The bonus rock keeps the flat 5% chance.
        let region = self.region;
        let unit = &region.units[o.unit.min(region.units.len() - 1)];
        let mineral_roll = self.rng.below(100) < unit.mineral_pct;
        let fossil_roll = self.rng.below(100) < unit.fossil_pct;
        let bonus_roll = self.rng.below(100) < 5;
        let mineral = if mineral_roll && !unit.minerals.is_empty() {
            Some(unit.minerals[self.rng.below(unit.minerals.len() as u32) as usize])
        } else {
            None
        };
        let fossil = if fossil_roll && !unit.fossils.is_empty() {
            Some(unit.fossils[self.rng.below(unit.fossils.len() as u32) as usize])
        } else {
            None
        };
        let bonus_rock = bonus_roll && self.backpack_used() < BACKPACK_SLOTS;

        if mineral.is_some() {
            self.xp += 10;
        }
        if fossil.is_some() {
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
            rock: o.rock,
            unit: unit.name,
            parent: unit.parent,
            age: unit.age,
            region: region.name,
            mineral,
            fossil,
            bonus_rock,
        });

        self.spawn_popup(o.x, o.y + 20.0, &format!("{name}! +10 pts / +20 xp"), (255, 255, 255));
        self.spawn_popup(o.x, o.y - 6.0, &format!("{} - {}", unit.name, unit.age), (200, 215, 235));
        if let Some(m) = mineral {
            self.spawn_popup(o.x, o.y + 46.0, &format!("{m} found! +10 xp"), (240, 210, 90));
        }
        if let Some(f) = fossil {
            self.spawn_popup(o.x, o.y + 72.0, &format!("{f}! +10 xp"), (190, 220, 255));
        }
        if bonus_rock {
            self.spawn_popup(o.x, o.y + 98.0, "Bonus rock collected", (200, 255, 200));
        }
        if self.pickaxe.wear(rock_hardness(o.lithology)) {
            self.spawn_popup(o.x, o.y + 124.0, "Your pickaxe broke!", (255, 150, 110));
        }

        self.surveyed += 1;
        if self.surveyed == region.quota && !self.level_done {
            self.level_done = true;
            let next = if self.level + 1 < REGIONS.len() { format!("Next field area: {}", REGIONS[self.level + 1].name) } else { "That was the last map!".to_string() };
            self.set_banner(format!("Survey of {} complete! {}", region.name, next));
        }
    }

    /// The mapped geological unit under the player.
    pub fn unit_here(&self) -> &'static crate::geology::Unit {
        let i = self.region.unit_at(&self.terrain, self.player.x, self.player.y);
        &self.region.units[i]
    }

    /// Move on to the next map (called at the start of a new day).
    fn advance_level(&mut self) {
        if self.level + 1 >= REGIONS.len() {
            return;
        }
        self.level += 1;
        self.region = &REGIONS[self.level];
        self.terrain = self.region.terrain();
        self.chunks.clear();
        self.level_done = false;
        self.surveyed = 0;
    }

    // ------------------------------------------------------------------- kit
    /// Morning issue: a full bottle, one piece of food (if there is room) and,
    /// if yesterday's pickaxe broke, a fresh basic one so nobody is stuck.
    fn issue_daily_kit(&mut self) {
        self.water = WATER_MAX;
        self.water_warned = false;
        if self.pickaxe.broken() {
            self.pickaxe = Pickaxe::new(0);
        }
        if self.backpack_used() < BACKPACK_SLOTS {
            if self.rng.below(2) == 0 {
                self.apples += 1;
            } else {
                self.sandwiches += 1;
            }
        }
    }

    /// Slots in use: bonus rocks and food share the backpack.
    pub fn backpack_used(&self) -> u32 {
        self.inventory_count + self.apples + self.sandwiches
    }

    pub fn dehydrated(&self) -> bool {
        self.water <= 0.0
    }

    fn warn_if_thirsty(&mut self, before: f32) {
        if before > 0.0 && self.water <= 0.0 {
            self.spawn_popup(self.player.x, self.player.y + 60.0, "Bottle empty - find a lake or river!", (150, 200, 255));
        } else if !self.water_warned && self.water <= 20.0 {
            self.water_warned = true;
            self.set_banner("Water is running low - look for a lake or river".to_string());
        }
    }

    /// Is a lake or river within reach of the bottle?
    pub fn near_water(&self) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        if self.terrain.ground(px, py).is_water() {
            return true;
        }
        for r in [30.0, 55.0, WATER_REACH] {
            for k in 0..12 {
                let a = k as f32 * std::f32::consts::TAU / 12.0;
                if self.terrain.ground(px + a.cos() * r, py + a.sin() * r).is_water() {
                    return true;
                }
            }
        }
        false
    }

    /// A tree (or the edge of a dense forest) close enough to sit in its shade.
    pub fn near_tree(&self) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        let (cx, cy) = world_to_chunk(px, py);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(c) = self.chunks.get(&(cx + dx, cy + dy)) {
                    if c.trees.iter().any(|t| ((t.x - px).powi(2) + (t.y - py).powi(2)).sqrt() <= TREE_REACH) {
                        return true;
                    }
                }
            }
        }
        (0..8).any(|k| {
            let a = k as f32 * std::f32::consts::TAU / 8.0;
            self.terrain.ground(px + a.cos() * 70.0, py + a.sin() * 70.0) == Ground::Forest
        })
    }

    /// The standalone tree under a tap, if any (the crown sits above the base).
    fn tree_at(&self, wx: f32, wy: f32) -> Option<(f32, f32)> {
        let (cx, cy) = world_to_chunk(wx, wy);
        let mut best: Option<((f32, f32), f32)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(c) = self.chunks.get(&(cx + dx, cy + dy)) {
                    for t in &c.trees {
                        let d = ((t.x - wx).powi(2) + (t.y + 45.0 - wy).powi(2)).sqrt();
                        if d <= 75.0 && best.map_or(true, |(_, bd)| d < bd) {
                            best = Some(((t.x, t.y), d));
                        }
                    }
                }
            }
        }
        best.map(|(p, _)| p)
    }

    fn can_eat_here(&self) -> bool {
        self.phase == Phase::Basecamp || self.near_tree()
    }

    pub fn fill_bottle(&mut self) {
        if self.water >= WATER_MAX - 0.5 {
            self.spawn_popup(self.player.x, self.player.y + 60.0, "Bottle already full", (200, 220, 240));
        } else {
            self.water = WATER_MAX;
            self.water_warned = false;
            self.spawn_popup(self.player.x, self.player.y + 60.0, "Bottle filled", (150, 210, 255));
        }
    }

    fn eat(&mut self, sandwich: bool) {
        let gain = if sandwich {
            self.sandwiches -= 1;
            SANDWICH_ENERGY
        } else {
            self.apples -= 1;
            APPLE_ENERGY
        };
        let before = self.energy;
        self.energy = (self.energy + gain).min(ENERGY_MAX);
        let text = format!("{} - +{} energy", if sandwich { "Sandwich" } else { "Apple" }, (self.energy - before).round() as i32);
        self.spawn_popup(self.player.x, self.player.y + 60.0, &text, (190, 240, 160));
    }

    /// Eat whatever suits how tired you are, if you are in the shade.
    pub fn eat_best(&mut self) {
        let (px, py) = (self.player.x, self.player.y + 60.0);
        if !self.can_eat_here() {
            self.spawn_popup(px, py, "Find a tree to sit and eat in the shade", (255, 210, 150));
        } else if self.apples + self.sandwiches == 0 {
            self.spawn_popup(px, py, "No food left", (255, 210, 150));
        } else if self.energy > NOT_HUNGRY_ABOVE {
            self.spawn_popup(px, py, "Not hungry yet", (220, 220, 220));
        } else {
            let missing = ENERGY_MAX - self.energy;
            let sandwich = self.sandwiches > 0 && (self.apples == 0 || missing >= 35.0);
            self.eat(sandwich);
        }
    }

    /// The F / USE button: fill the bottle at water, otherwise eat under a tree.
    pub fn use_context(&mut self) {
        if self.near_water() && self.water < WATER_MAX - 0.5 {
            self.fill_bottle();
        } else if self.phase == Phase::Basecamp && self.water < WATER_MAX - 0.5 {
            self.fill_bottle();
        } else if self.near_water() {
            self.fill_bottle(); // full: says so
        } else {
            self.eat_best();
        }
    }

    // ----------------------------------------------------------- backpack menu
    pub fn pack_rows(&self) -> Vec<PackRow> {
        let camp = self.phase == Phase::Basecamp;
        let pts = self.outcrop_points;
        let full = self.backpack_used() >= BACKPACK_SLOTS;
        let row = |action, label: String, detail: &str, enabled| PackRow { action, label, detail: detail.to_string(), enabled };
        let mut rows = Vec::new();

        // water
        let (d, ok) = if self.water >= WATER_MAX - 0.5 {
            ("Bottle is full", false)
        } else if camp {
            ("Refill from the camp tank (free)", true)
        } else if self.near_water() {
            ("Fill from the water", true)
        } else {
            ("Find a lake or river", false)
        };
        rows.push(row(Action::RefillWater, format!("Water bottle   {:.0}/{:.0}", self.water, WATER_MAX), d, ok));

        // pickaxe
        let pick = format!("Pickaxe: {}   {}/{}", self.pickaxe.name(), self.pickaxe.durability, self.pickaxe.max_durability());
        if self.pickaxe.broken() {
            let ok = pts >= PICK_REPLACE_COST;
            rows.push(row(Action::ReplacePickaxe, format!("{pick}  BROKEN"), &format!("Buy a new basic pickaxe - {PICK_REPLACE_COST} pts{}", if ok { "" } else { " (not enough points)" }), ok));
        } else {
            rows.push(row(Action::ReplacePickaxe, pick, "In working order", false));
        }
        match self.pickaxe.next_tier() {
            Some((name, cost)) => {
                let (d, ok) = if !camp {
                    (format!("{cost} pts - upgrades are done at basecamp"), false)
                } else if pts < cost {
                    (format!("{cost} pts (not enough points)"), false)
                } else {
                    (format!("{cost} pts - a tougher pickaxe, fully repaired"), true)
                };
                rows.push(row(Action::UpgradePickaxe, format!("Upgrade to {name}"), &d, ok));
            }
            None => rows.push(row(Action::UpgradePickaxe, "Pickaxe fully upgraded".to_string(), "Best tool available", false)),
        }

        // food
        let eat_state = |count: u32, gain: f32| -> (String, bool) {
            if count == 0 {
                ("None left".to_string(), false)
            } else if !self.can_eat_here() {
                ("Find a tree to eat in the shade".to_string(), false)
            } else if self.energy > NOT_HUNGRY_ABOVE {
                ("Not hungry yet".to_string(), false)
            } else {
                (format!("Restores {gain:.0} energy"), true)
            }
        };
        let (d, ok) = eat_state(self.apples, APPLE_ENERGY);
        rows.push(row(Action::EatApple, format!("Eat apple   x{}", self.apples), &d, ok));
        let (d, ok) = eat_state(self.sandwiches, SANDWICH_ENERGY);
        rows.push(row(Action::EatSandwich, format!("Eat sandwich   x{}", self.sandwiches), &d, ok));

        // store
        let buy_state = |price: u32| -> (String, bool) {
            if !camp {
                (format!("{price} pts - sold at basecamp"), false)
            } else if full {
                ("Backpack is full".to_string(), false)
            } else if pts < price {
                (format!("{price} pts (not enough points)"), false)
            } else {
                (format!("{price} pts"), true)
            }
        };
        let (d, ok) = buy_state(APPLE_PRICE);
        rows.push(row(Action::BuyApple, "Buy an apple".to_string(), &d, ok));
        let (d, ok) = buy_state(SANDWICH_PRICE);
        rows.push(row(Action::BuySandwich, "Buy a sandwich".to_string(), &d, ok));
        rows
    }

    fn handle_backpack_input(&mut self, input: &Input) {
        let n = ACTIONS.len();
        if input.page_down_pressed {
            self.pack_sel = (self.pack_sel + 1) % n;
        }
        if input.page_up_pressed {
            self.pack_sel = (self.pack_sel + n - 1) % n;
        }
        let mut activate = input.interact_pressed || input.space_pressed || input.restart_pressed;
        if let Some((mx, my)) = input.click {
            for i in 0..n {
                let (x, y, w, h) = pack_row_rect(i);
                if mx >= x && mx <= x + w && my >= y && my <= y + h {
                    self.pack_sel = i;
                    activate = true;
                }
            }
        }
        if activate {
            self.run_pack_action(self.pack_sel);
        }
    }

    fn pack_notice(&mut self, text: &str) {
        self.pack_msg = Some((text.to_string(), 150));
    }

    pub fn run_pack_action(&mut self, idx: usize) {
        let rows = self.pack_rows();
        let Some(row) = rows.get(idx).cloned() else { return };
        if !row.enabled {
            self.pack_notice(&row.detail);
            return;
        }
        match row.action {
            Action::RefillWater => {
                self.water = WATER_MAX;
                self.water_warned = false;
                self.pack_notice("Bottle filled");
            }
            Action::ReplacePickaxe => {
                self.outcrop_points -= PICK_REPLACE_COST;
                self.pickaxe = Pickaxe::new(0);
                self.pack_notice(&format!("New pickaxe! -{PICK_REPLACE_COST} pts"));
            }
            Action::UpgradePickaxe => {
                if let Some((name, cost)) = self.pickaxe.next_tier() {
                    self.outcrop_points -= cost;
                    self.pickaxe = Pickaxe::new(self.pickaxe.tier + 1);
                    self.pack_notice(&format!("Upgraded to {name}! -{cost} pts"));
                }
            }
            Action::EatApple | Action::EatSandwich => {
                let sandwich = row.action == Action::EatSandwich;
                let before = self.energy;
                self.eat(sandwich);
                self.pack_notice(&format!("+{} energy", (self.energy - before).round() as i32));
            }
            Action::BuyApple => {
                self.outcrop_points -= APPLE_PRICE;
                self.apples += 1;
                self.pack_notice(&format!("Bought an apple -{APPLE_PRICE} pts"));
            }
            Action::BuySandwich => {
                self.outcrop_points -= SANDWICH_PRICE;
                self.sandwiches += 1;
                self.pack_notice(&format!("Bought a sandwich -{SANDWICH_PRICE} pts"));
            }
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

    /// Old rules were written for a flat world: keep those tests on flat ground.
    fn new_game(seed: u64) -> Game {
        Game::with_terrain(seed, Terrain::flat())
    }

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
        let g = new_game(1);
        assert_eq!((g.outcrop_points, g.xp, g.inventory_count), (0, 10, 0));
        assert_eq!((g.phase, g.day), (Phase::Basecamp, 1));
        assert!(in_camp(g.player.x, g.player.y, 0.0));
        assert_eq!(g.minutes_of_day(), 5 * 60);
    }

    #[test]
    fn cannot_board_when_far_from_truck_or_break_rocks_at_camp() {
        let mut g = new_game(1);
        g.tick(&press_e()); // spawn point is ~230 away from the truck
        assert_eq!(g.phase, Phase::Basecamp);
        g.tick(&Input { space_pressed: true, ..idle() });
        assert_eq!(g.cards.len(), 0);
    }

    #[test]
    fn full_day_loop_with_pickup() {
        let mut g = new_game(5);
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
        let mut g = new_game(9);
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
        let mut g = new_game(9);
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
    fn lunch_is_announced_once_but_gives_no_free_energy() {
        let mut g = new_game(3);
        exploring_at(&mut g, 700.0, 700.0);
        g.energy = 10.0;
        g.clock_ticks = (SUPPLY_MIN - DAY_START_MIN) * TICKS_PER_MINUTE - 1;
        g.tick(&idle());
        g.tick(&idle());
        assert!(g.supply_given);
        assert!(g.banner.as_ref().is_some_and(|(t, _)| t.contains("Lunch")));
        assert!((g.energy - 10.0).abs() < 0.01, "{}", g.energy);
    }

    #[test]
    fn exhausted_player_moves_at_half_speed() {
        let mut g = new_game(3);
        exploring_at(&mut g, 700.0, 700.0);
        g.energy = 0.0;
        g.tick(&Input { right: true, ..idle() });
        assert!((g.player.x - (700.0 + PLAYER_SPEED / 2.0)).abs() < 1e-3);
    }

    #[test]
    fn drop_points_vary_but_always_land_next_to_a_road() {
        let mut seen = HashSet::new();
        for seed in 1..40u64 {
            let mut g = new_game(seed * 7919);
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
        let mut g = new_game(1);
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
        let mut g = new_game(1);
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
        let mut g = new_game(7);
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
    fn inventory_never_exceeds_cap_and_drop_rate_follows_the_unit() {
        let mut g = new_game(42);
        g.player.x = 0.0;
        g.player.y = 0.0;
        let mut minerals = 0;
        let n = 20_000;
        for i in 0..n {
            g.visited.clear();
            g.pickaxe.durability = 99; // this test is about drop rates, not tool wear
            let o = Outcrop::new(&format!("t{i}"), 0.0, 0.0, 3, "Basalt", "Igneous");
            g.chunks.insert((0, 0), Chunk { grass_variant: 0, outcrops: vec![o], trees: vec![], dirt_patches: vec![] });
            g.break_nearest_outcrop();
            minerals += g.cards.last().unwrap().mineral.is_some() as u32;
            assert!(g.backpack_used() <= BACKPACK_SLOTS);
        }
        let rate = minerals as f32 / n as f32;
        let expect = g.region.units[3].mineral_pct as f32 / 100.0;
        assert!((expect - 0.015..expect + 0.015).contains(&rate), "rate {rate}, expected about {expect}");
        assert_eq!(g.backpack_used(), BACKPACK_SLOTS, "bonus rocks fill whatever the food leaves free");
    }

    #[test]
    fn overlays_toggle_and_suspend_movement_and_page_wraps() {
        let mut g = new_game(1);
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
        let mut g = new_game(1);
        for _ in 0..3000 {
            g.tick(&Input { left: true, down: true, ..idle() });
        }
        assert!(g.player.x >= CAMP.0 - 120.0 - 0.01 && g.player.y >= -ROAD_W / 2.0 - 40.0 - 0.01);
    }

    #[test]
    fn popups_expire_and_night_follows_the_field_clock() {
        let mut g = new_game(1);
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
        let mut g = new_game(7);
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
        let mut g = new_game(7);
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
        let mut g = new_game(7);
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
        let mut g = new_game(11);
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
        let mut g = new_game(1);
        exploring_at(&mut g, 700.0, 700.0);
        // a spot with no outcrop within pick radius
        g.chunks.values_mut().for_each(|c| c.outcrops.clear());
        g.tick(&Input { click: Some((SCREEN_W / 2.0 + 120.0, SCREEN_H / 2.0)), ..idle() });
        assert_eq!(g.pending, Pending::None);
        assert!(g.move_target.is_some());
    }

    // ------------------------------------------------------------- terrain
    fn find_spot(t: &Terrain, ok: impl Fn(&Terrain, f32, f32) -> bool) -> (f32, f32) {
        for i in -150..150 {
            for j in -150..150 {
                let (x, y) = (i as f32 * 41.0, j as f32 * 43.0);
                if ok(t, x, y) {
                    return (x, y);
                }
            }
        }
        panic!("no matching spot on the default terrain");
    }

    #[test]
    fn climbing_is_slower_and_costlier_than_descending() {
        let mut g = Game::new(1);
        let (x, y) = find_spot(&g.terrain, |t, x, y| {
            t.ground(x, y) == Ground::Open
                && t.grade_toward(x, y, 1.0, 0.0) > 0.2
                && t.grade_toward(x, y, 1.0, 0.0) < 0.5
                && !t.blocked(x - 12.0, y)
                && !t.blocked(x + 12.0, y)
        });
        let measure = |g: &mut Game, dir_right: bool| {
            exploring_at(g, x, y);
            g.energy = ENERGY_MAX;
            let input = if dir_right { Input { right: true, ..idle() } } else { Input { left: true, ..idle() } };
            g.tick(&input);
            ((g.player.x - x).abs(), ENERGY_MAX - g.energy, g.move_grade)
        };
        let (up_step, up_cost, up_grade) = measure(&mut g, true);
        let (dn_step, dn_cost, dn_grade) = measure(&mut g, false);
        assert!(up_grade > 0.0 && dn_grade < 0.0, "{up_grade} {dn_grade}");
        assert!(up_step < PLAYER_SPEED && dn_step > PLAYER_SPEED, "{up_step} {dn_step}");
        assert!(up_cost / up_step > dn_cost / dn_step * 1.5, "per-metre cost {} vs {}", up_cost / up_step, dn_cost / dn_step);
    }

    #[test]
    fn flat_ground_keeps_the_original_speed_and_energy_rate() {
        let mut g = new_game(1);
        exploring_at(&mut g, 700.0, 700.0);
        let e0 = g.energy;
        g.tick(&Input { right: true, ..idle() });
        assert!((g.player.x - 704.2).abs() < 1e-3);
        assert!((e0 - g.energy - ENERGY_DRAIN_PER_TICK).abs() < 1e-5);
    }

    #[test]
    fn obstacles_cannot_be_walked_into() {
        let mut g = Game::new(1);
        // an open spot with blocked ground a short way to the east
        let (x, y) = find_spot(&g.terrain, |t, x, y| {
            t.ground(x, y) == Ground::Open && t.blocked(x + 60.0, y) && t.blocked(x + 60.0, y + 30.0) && t.blocked(x + 60.0, y - 30.0)
        });
        exploring_at(&mut g, x, y);
        for _ in 0..200 {
            g.tick(&Input { right: true, ..idle() });
            assert!(!g.terrain.blocked(g.player.x, g.player.y), "walked into {:?}", g.ground_here());
        }
        assert!(g.player.x < x + 60.0);
        assert!(g.popups.iter().any(|p| !p.text.is_empty()), "no explanation shown");
    }

    #[test]
    fn click_to_move_stops_at_the_edge_of_an_obstacle() {
        let mut g = Game::new(1);
        let (x, y) = find_spot(&g.terrain, |t, x, y| {
            t.ground(x, y) == Ground::Open && (1..=8).all(|k| t.blocked(x + 40.0 + k as f32 * 30.0, y))
        });
        exploring_at(&mut g, x, y);
        // tap a point inside the obstacle (screen coords are relative to the camera)
        let (cx, cy) = g.cam();
        g.tick(&Input { click: Some((x + 200.0 - cx, y - cy)), ..idle() });
        for _ in 0..300 {
            g.tick(&idle());
            assert!(!g.terrain.blocked(g.player.x, g.player.y));
        }
        assert!(g.move_target.is_none());
    }

    #[test]
    fn sliding_along_an_obstacle_keeps_you_moving() {
        let mut g = Game::new(1);
        // blocked to the east, free to the north
        let (x, y) = find_spot(&g.terrain, |t, x, y| {
            t.ground(x, y) == Ground::Open
                && t.blocked(x + 8.0, y)
                && t.blocked(x + 8.0, y + 40.0)
                && !t.blocked(x, y + 40.0)
                && !t.blocked(x, y + 10.0)
        });
        exploring_at(&mut g, x, y);
        g.tick(&Input { right: true, up: true, ..idle() });
        assert!(g.player.y > y, "should slide north");
        assert!(!g.terrain.blocked(g.player.x, g.player.y));
    }

    #[test]
    fn basecamp_and_truck_ride_ignore_terrain() {
        let mut g = Game::new(5);
        g.player.x = g.truck.x;
        g.player.y = g.truck.y + 60.0;
        g.tick(&press_e());
        assert_eq!(g.phase, Phase::TruckOut);
        run_ride(&mut g);
        assert_eq!(g.phase, Phase::Exploring);
        assert!(!g.terrain.blocked(g.player.x, g.player.y), "dropped into an obstacle");
    }

    #[test]
    fn restart_keeps_the_same_terrain() {
        let mut g = Game::with_terrain(9, Terrain { relief: 1.25, ..Terrain::default() });
        exploring_at(&mut g, 700.0, 700.0);
        g.phase = Phase::GameOver;
        g.tick(&Input { restart_pressed: true, ..idle() });
        assert_eq!(g.terrain.relief, 1.25);
    }

    // ----------------------------------------------------------------- kit
    fn lone_outcrop(g: &mut Game, id: &str, lithology: &'static str) -> Outcrop {
        let o = Outcrop::new(id, 700.0, 700.0, 0, "Sandstone", lithology);
        g.player.x = 700.0;
        g.player.y = 700.0;
        g.chunks.insert((1, 1), Chunk { grass_variant: 0, outcrops: vec![o.clone()], trees: vec![], dirt_patches: vec![] });
        o
    }

    fn press(g: &mut Game, f: impl Fn(&mut Input)) {
        let mut i = idle();
        f(&mut i);
        g.tick(&i);
    }

    #[test]
    fn each_day_starts_with_a_full_bottle_one_food_and_a_pickaxe() {
        let mut g = new_game(4);
        assert_eq!(g.water, WATER_MAX);
        assert_eq!(g.apples + g.sandwiches, 1);
        assert_eq!((g.pickaxe.tier, g.pickaxe.durability), (0, 15));
        // spend things, sleep, and the kit is reissued
        g.water = 3.0;
        g.pickaxe.durability = 0;
        g.phase = Phase::Sleeping;
        g.sleep_timer = SLEEP_TICKS;
        g.tick(&idle());
        assert_eq!(g.day, 2);
        assert_eq!(g.water, WATER_MAX);
        assert_eq!(g.pickaxe, Pickaxe::new(0), "a broken pickaxe is replaced free in the morning");
        assert_eq!(g.apples + g.sandwiches, 2, "yesterday's unused food is kept");
    }

    #[test]
    fn the_daily_ration_needs_room_in_the_pack() {
        let mut g = new_game(4);
        g.apples = 0;
        g.sandwiches = 0;
        g.inventory_count = BACKPACK_SLOTS;
        g.phase = Phase::Sleeping;
        g.sleep_timer = SLEEP_TICKS;
        g.tick(&idle());
        assert_eq!(g.backpack_used(), BACKPACK_SLOTS);
    }

    #[test]
    fn walking_empties_the_bottle_faster_than_standing() {
        let mut g = new_game(1);
        exploring_at(&mut g, 700.0, 700.0);
        for _ in 0..600 {
            g.tick(&idle());
        }
        let idle_loss = WATER_MAX - g.water;
        assert!(idle_loss > 0.0);
        let mut g = new_game(1);
        exploring_at(&mut g, 700.0, 700.0);
        for _ in 0..600 {
            g.tick(&Input { right: true, ..idle() });
        }
        assert!(WATER_MAX - g.water > idle_loss * 2.0);
    }

    #[test]
    fn a_full_day_of_walking_needs_more_than_one_bottle() {
        // 13 field hours, walking the whole time on the flat
        let ticks = ((SUNSET_MIN - DAY_START_MIN) * TICKS_PER_MINUTE) as f32;
        let per_tick = WATER_DRAIN_IDLE + WATER_DRAIN_WALK;
        assert!(ticks * per_tick > WATER_MAX, "one bottle should not last a hard day");
        assert!(ticks * WATER_DRAIN_IDLE < WATER_MAX, "but resting it would");
    }

    #[test]
    fn an_empty_bottle_slows_you_and_burns_more_energy() {
        let mut a = new_game(1);
        exploring_at(&mut a, 700.0, 700.0);
        a.tick(&Input { right: true, ..idle() });
        let (step_ok, cost_ok) = (a.player.x - 700.0, ENERGY_MAX - a.energy);

        let mut b = new_game(1);
        exploring_at(&mut b, 700.0, 700.0);
        b.water = 0.0;
        b.tick(&Input { right: true, ..idle() });
        let (step_dry, cost_dry) = (b.player.x - 700.0, ENERGY_MAX - b.energy);
        assert!(b.dehydrated());
        assert!((step_dry - step_ok * DEHYDRATED_SPEED).abs() < 1e-3);
        assert!(cost_dry / step_dry > cost_ok / step_ok * 1.4);
    }

    fn spot_beside_water(t: &Terrain) -> (f32, f32) {
        for i in -150..150 {
            for j in -150..150 {
                let (x, y) = (i as f32 * 41.0, j as f32 * 43.0);
                if t.ground(x, y) == Ground::Open && (0..12).any(|k| {
                    let a = k as f32 * std::f32::consts::TAU / 12.0;
                    t.ground(x + a.cos() * 50.0, y + a.sin() * 50.0).is_water()
                }) {
                    return (x, y);
                }
            }
        }
        panic!("no shoreline found");
    }

    #[test]
    fn the_use_key_fills_the_bottle_at_water_only() {
        let mut g = Game::new(2);
        let (x, y) = spot_beside_water(&g.terrain);
        exploring_at(&mut g, 700.0, 700.0);
        g.water = 10.0;
        assert!(!g.near_water());
        press(&mut g, |i| i.use_pressed = true);
        assert!(g.water < 11.0, "no water here, nothing filled");
        exploring_at(&mut g, x, y);
        assert!(g.near_water());
        press(&mut g, |i| i.use_pressed = true);
        assert!(g.water > WATER_MAX - 0.1, "{}", g.water); // topped up (then a tick of drain)
        assert!(g.popups.iter().any(|p| p.text.contains("filled")));
    }

    #[test]
    fn tapping_water_walks_to_the_shore_and_fills() {
        let mut g = Game::new(2);
        let t = g.terrain.clone();
        // an open spot with a lake straight to the east, and open ground between
        let (x, y) = find_spot(&t, |t, x, y| {
            t.ground(x, y) == Ground::Open && (1..=5).all(|k| t.ground(x + k as f32 * 20.0, y) == Ground::Open) && t.ground(x + 300.0, y) == Ground::Lake
                && !(0..12).any(|k| {
                    let a = k as f32 * std::f32::consts::TAU / 12.0;
                    t.ground(x + a.cos() * 80.0, y + a.sin() * 80.0).is_water()
                })
        });
        exploring_at(&mut g, x, y);
        g.water = 20.0;
        let (cx, cy) = g.cam();
        g.tick(&Input { click: Some((x + 300.0 - cx, y - cy)), ..idle() });
        assert_eq!(g.pending, Pending::Fill);
        for _ in 0..200 {
            g.tick(&idle());
        }
        assert!(g.water > 95.0, "{}", g.water);
        assert_eq!(g.pending, Pending::None);
        assert!(!g.terrain.blocked(g.player.x, g.player.y));
    }

    fn tree_beside(g: &mut Game) {
        let (px, py) = (g.player.x, g.player.y);
        let (cx, cy) = world_to_chunk(px, py);
        g.chunks.insert((cx, cy), Chunk { grass_variant: 0, outcrops: vec![], trees: vec![Point { x: px + 60.0, y: py }], dirt_patches: vec![] });
    }

    #[test]
    fn eating_needs_a_tree_and_restores_energy_partially() {
        let mut g = new_game(6);
        exploring_at(&mut g, 700.0, 700.0);
        g.chunks.values_mut().for_each(|c| c.trees.clear());
        g.apples = 1;
        g.sandwiches = 0;
        g.energy = 40.0;
        press(&mut g, |i| i.use_pressed = true);
        assert_eq!((g.apples, g.energy), (1, 40.0), "no shade, no lunch");
        tree_beside(&mut g);
        press(&mut g, |i| i.use_pressed = true);
        assert_eq!(g.apples, 0);
        assert!((g.energy - (40.0 + APPLE_ENERGY)).abs() < 0.5, "{}", g.energy);
        assert!(g.energy < ENERGY_MAX, "partial restore");
        press(&mut g, |i| i.use_pressed = true);
        assert!(g.popups.iter().any(|p| p.text.contains("No food")));
    }

    #[test]
    fn you_are_not_hungry_when_nearly_full_and_energy_never_overflows() {
        let mut g = new_game(6);
        exploring_at(&mut g, 700.0, 700.0);
        tree_beside(&mut g);
        g.apples = 1;
        g.energy = 95.0;
        press(&mut g, |i| i.use_pressed = true);
        assert_eq!(g.apples, 1);
        g.energy = 90.0;
        press(&mut g, |i| i.use_pressed = true);
        assert_eq!((g.apples, g.energy), (0, ENERGY_MAX));
    }

    #[test]
    fn a_very_tired_geologist_gets_the_sandwich() {
        let mut g = new_game(6);
        exploring_at(&mut g, 700.0, 700.0);
        tree_beside(&mut g);
        g.apples = 1;
        g.sandwiches = 1;
        g.energy = 20.0;
        press(&mut g, |i| i.use_pressed = true);
        assert_eq!((g.apples, g.sandwiches), (1, 0));
        assert!((g.energy - 60.0).abs() < 0.5);
    }

    #[test]
    fn tapping_a_tree_walks_there_and_eats() {
        let mut g = new_game(6);
        exploring_at(&mut g, 700.0, 700.0);
        g.chunks.values_mut().for_each(|c| c.trees.clear());
        g.chunks.insert((1, 1), Chunk { grass_variant: 0, outcrops: vec![], trees: vec![Point { x: 1000.0, y: 700.0 }], dirt_patches: vec![] });
        g.apples = 1;
        g.sandwiches = 0;
        g.energy = 30.0;
        let (cx, cy) = g.cam();
        g.tick(&Input { click: Some((1000.0 - cx, 700.0 + 45.0 - cy)), ..idle() });
        assert_eq!(g.pending, Pending::Eat);
        for _ in 0..200 {
            g.tick(&idle());
        }
        assert_eq!(g.apples, 0);
        assert!(g.energy > 30.0);
    }

    #[test]
    fn the_edge_of_a_dense_forest_gives_shade_too() {
        let mut g = Game::new(3);
        let (x, y) = find_spot(&g.terrain, |t, x, y| {
            t.ground(x, y) == Ground::Open && t.ground(x + 50.0, y) == Ground::Forest
        });
        exploring_at(&mut g, x, y);
        g.chunks.values_mut().for_each(|c| c.trees.clear());
        assert!(g.near_tree());
    }

    #[test]
    fn breaking_wears_the_pickaxe_by_rock_hardness() {
        let mut g = new_game(7);
        exploring_at(&mut g, 700.0, 700.0);
        let o = lone_outcrop(&mut g, "a", "Igneous");
        g.break_outcrop(&o);
        assert_eq!(g.pickaxe.durability, 15 - 2);
        let o = lone_outcrop(&mut g, "b", "Sedimentary");
        g.break_outcrop(&o);
        assert_eq!(g.pickaxe.durability, 15 - 2 - 1);
        // an outcrop you already surveyed costs nothing
        g.break_outcrop(&o);
        assert_eq!(g.pickaxe.durability, 15 - 3);
    }

    #[test]
    fn a_broken_pickaxe_cannot_break_rock_until_replaced() {
        let mut g = new_game(7);
        exploring_at(&mut g, 700.0, 700.0);
        g.pickaxe.durability = 1;
        let o = lone_outcrop(&mut g, "a", "Igneous");
        g.break_outcrop(&o);
        assert_eq!(g.cards.len(), 1, "the breaking blow still counts");
        assert!(g.pickaxe.broken());
        assert!(g.popups.iter().any(|p| p.text.contains("pickaxe broke")));

        let o2 = lone_outcrop(&mut g, "b", "Sedimentary");
        g.break_outcrop(&o2);
        assert_eq!(g.cards.len(), 1);
        assert!(!g.visited.contains("b"), "still raw: it was not broken");

        // not enough points: replacing is refused
        g.outcrop_points = PICK_REPLACE_COST - 1;
        g.ui_mode = UiMode::Backpack;
        g.run_pack_action(1);
        assert!(g.pickaxe.broken());
        assert_eq!(g.outcrop_points, PICK_REPLACE_COST - 1);

        // with the points, a new one is bought on the spot, in the field
        g.outcrop_points = PICK_REPLACE_COST + 5;
        g.run_pack_action(1);
        assert_eq!(g.pickaxe, Pickaxe::new(0));
        assert_eq!(g.outcrop_points, 5);
        g.ui_mode = UiMode::None;
        g.break_outcrop(&o2);
        assert_eq!(g.cards.len(), 2);
    }

    #[test]
    fn upgrades_are_sold_at_basecamp_only_and_deduct_points() {
        let mut g = new_game(7);
        g.outcrop_points = 500;
        g.pickaxe.durability = 3;
        g.phase = Phase::Exploring;
        g.run_pack_action(2);
        assert_eq!((g.pickaxe.tier, g.outcrop_points), (0, 500), "not at basecamp");
        g.phase = Phase::Basecamp;
        g.run_pack_action(2);
        assert_eq!(g.pickaxe, Pickaxe::new(1), "tougher and fully repaired");
        assert_eq!(g.outcrop_points, 500 - PICK_TIERS[1].cost);
        g.run_pack_action(2);
        g.run_pack_action(2);
        assert_eq!(g.pickaxe.tier, PICK_TIERS.len() - 1);
        let rows = g.pack_rows();
        assert!(!rows[2].enabled && rows[2].label.contains("fully upgraded"));
        let spent: u32 = PICK_TIERS.iter().map(|t| t.cost).sum();
        assert_eq!(g.outcrop_points, 500 - spent);
    }

    #[test]
    fn upgrades_need_enough_points() {
        let mut g = new_game(7);
        g.outcrop_points = PICK_TIERS[1].cost - 1;
        g.run_pack_action(2);
        assert_eq!(g.pickaxe.tier, 0);
        assert!(g.pack_msg.is_some());
    }

    #[test]
    fn food_is_bought_at_camp_and_respects_points_and_space() {
        let mut g = new_game(8);
        g.apples = 0;
        g.sandwiches = 0;
        g.outcrop_points = 50;
        g.run_pack_action(5); // apple
        g.run_pack_action(6); // sandwich
        assert_eq!((g.apples, g.sandwiches, g.outcrop_points), (1, 1, 50 - APPLE_PRICE - SANDWICH_PRICE));
        g.run_pack_action(6); // only 5 points left
        assert_eq!(g.sandwiches, 1);
        g.outcrop_points = 1000;
        g.inventory_count = BACKPACK_SLOTS - g.apples - g.sandwiches;
        g.run_pack_action(5);
        assert_eq!(g.backpack_used(), BACKPACK_SLOTS, "backpack is full");
        assert_eq!(g.outcrop_points, 1000);
        // not for sale in the field
        g.inventory_count = 0;
        g.phase = Phase::Exploring;
        g.run_pack_action(5);
        assert_eq!(g.outcrop_points, 1000);
    }

    #[test]
    fn bonus_rocks_cannot_take_the_last_slot_from_a_full_pack() {
        let mut g = new_game(42);
        g.apples = 0;
        g.sandwiches = 0;
        g.inventory_count = BACKPACK_SLOTS;
        g.player.x = 0.0;
        g.player.y = 0.0;
        for i in 0..400 {
            g.pickaxe.durability = 99;
            let o = Outcrop::new(&format!("r{i}"), 0.0, 0.0, 3, "Basalt", "Igneous");
            g.chunks.insert((0, 0), Chunk { grass_variant: 0, outcrops: vec![o], trees: vec![], dirt_patches: vec![] });
            g.break_nearest_outcrop();
        }
        assert_eq!(g.backpack_used(), BACKPACK_SLOTS);
    }

    #[test]
    fn the_backpack_opens_on_foot_only_and_navigates_by_key_and_tap() {
        let mut g = new_game(5);
        press(&mut g, |i| i.pack_pressed = true);
        assert_eq!(g.ui_mode, UiMode::Backpack);
        // movement is suspended while it is open
        let x0 = g.player.x;
        g.tick(&Input { right: true, ..idle() });
        assert_eq!(g.player.x, x0);
        // keyboard selection wraps
        press(&mut g, |i| i.page_up_pressed = true);
        assert_eq!(g.pack_sel, ACTIONS.len() - 1);
        press(&mut g, |i| i.page_down_pressed = true);
        assert_eq!(g.pack_sel, 0);
        // tapping a row selects and activates it (apple costs points: none yet)
        g.outcrop_points = 100;
        g.apples = 0;
        let (rx, ry, rw, rh) = pack_row_rect(5);
        g.tick(&Input { click: Some((rx + rw / 2.0, ry + rh / 2.0)), ..idle() });
        assert_eq!((g.pack_sel, g.apples), (5, 1));
        press(&mut g, |i| i.pack_pressed = true);
        assert_eq!(g.ui_mode, UiMode::None);

        // not while riding the truck
        g.phase = Phase::TruckOut;
        press(&mut g, |i| i.pack_pressed = true);
        assert_eq!(g.ui_mode, UiMode::None);
    }

    #[test]
    fn disabled_rows_explain_themselves_instead_of_acting() {
        let mut g = new_game(5);
        g.phase = Phase::Exploring;
        let rows = g.pack_rows();
        assert_eq!(rows.len(), ACTIONS.len());
        assert!(rows.iter().all(|r| !r.detail.is_empty()));
        assert!(!rows[0].enabled && rows[0].detail.contains("full"));
        g.run_pack_action(5); // buying in the field
        assert!(g.pack_msg.as_ref().is_some_and(|(t, _)| t.contains("basecamp")));
    }

    #[test]
    fn restart_after_game_over_gives_a_fresh_kit() {
        let mut g = new_game(9);
        g.water = 1.0;
        g.pickaxe.durability = 0;
        g.outcrop_points = 77;
        g.phase = Phase::GameOver;
        g.tick(&Input { restart_pressed: true, ..idle() });
        assert_eq!((g.water, g.pickaxe, g.outcrop_points), (WATER_MAX, Pickaxe::new(0), 0));
    }

    // ------------------------------------------------------------ level maps
    fn outcrop_here(g: &mut Game, id: &str) -> Outcrop {
        let o = Outcrop::new(id, 0.0, 0.0, 0, "Sandstone", "Sedimentary");
        g.player.x = 0.0;
        g.player.y = 0.0;
        g.pickaxe.durability = 99;
        g.chunks.insert((0, 0), Chunk { grass_variant: 0, outcrops: vec![o.clone()], trees: vec![], dirt_patches: vec![] });
        o
    }

    #[test]
    fn levels_are_bounded_regional_maps() {
        for l in 0..REGIONS.len() {
            let g = Game::with_level(7, l);
            assert_eq!(g.level, l);
            assert_eq!(g.region.id, REGIONS[l].id);
            let (x0, y0, x1, y1) = g.terrain.bounds.expect("each level map is bounded");
            assert_eq!(g.terrain.ground(x1 + 500.0, 0.0), Ground::Edge);
            assert_eq!(g.terrain.ground(0.0, y0 - 500.0), Ground::Edge);
            assert!(x0 < 0.0 && y0 < 0.0 && x1 > 0.0 && y1 > 0.0);
        }
    }

    #[test]
    fn meeting_the_quota_unlocks_the_next_map_the_following_morning() {
        let mut g = new_game(3);
        g.phase = Phase::Exploring;
        let quota = g.region.quota;
        g.surveyed = quota - 1;
        let o = outcrop_here(&mut g, "paraná:q");
        g.break_outcrop(&o);
        assert!(g.level_done && g.surveyed == quota);
        assert_eq!(g.level, 0, "the map does not change mid-day");
        g.start_next_day();
        assert_eq!((g.level, g.surveyed, g.level_done), (1, 0, false));
        assert_eq!(g.region.id, REGIONS[1].id);
        assert!(g.chunks.is_empty(), "old map's chunks are dropped");
        assert_eq!(g.phase, Phase::Basecamp);
    }

    #[test]
    fn without_a_quota_the_same_map_continues() {
        let mut g = new_game(3);
        g.surveyed = g.region.quota - 1;
        g.start_next_day();
        assert_eq!((g.level, g.surveyed), (0, g.region.quota - 1));
    }

    #[test]
    fn the_last_map_is_the_last() {
        let mut g = Game::with_level(3, REGIONS.len() - 1);
        g.level_done = true;
        g.start_next_day();
        assert_eq!(g.level, REGIONS.len() - 1);
    }

    #[test]
    fn finds_come_from_the_outcrops_own_unit() {
        let mut g = Game::with_level(11, 1); // Araripe: the Crato Formation is the fossil unit
        g.phase = Phase::Exploring;
        let crato = g.region.units.iter().position(|u| u.name == "Crato Formation").unwrap();
        let (mut fossils, n) = (0, 6000);
        for i in 0..n {
            g.visited.clear();
            let mut o = outcrop_here(&mut g, &format!("c{i}"));
            o.unit = crato;
            g.chunks.insert((0, 0), Chunk { grass_variant: 0, outcrops: vec![o], trees: vec![], dirt_patches: vec![] });
            g.break_nearest_outcrop();
            let c = g.cards.last().unwrap();
            assert_eq!(c.unit, "Crato Formation");
            if let Some(f) = c.fossil {
                assert!(g.region.units[crato].fossils.contains(&f));
                fossils += 1;
            }
            if let Some(m) = c.mineral {
                assert!(g.region.units[crato].minerals.contains(&m));
            }
            g.cards.clear();
            g.inventory_count = 0;
        }
        let expect = g.region.units[crato].fossil_pct as f32 / 100.0;
        let rate = fossils as f32 / n as f32;
        assert!((rate - expect).abs() < 0.03, "fossil rate {rate}, expected {expect}");
    }
}
