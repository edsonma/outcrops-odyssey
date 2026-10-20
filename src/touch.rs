//! Touch controls: on-screen buttons and a floating virtual stick. Pure logic
//! (no windowing), so it is unit-tested headlessly. All coordinates are in the
//! 1280x720 virtual screen with y pointing UP.

use crate::game::*;
use crate::kit::{NOT_HUNGRY_ABOVE, WATER_MAX};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BtnId {
    Break,
    Act,
    Map,
    Cards,
    Pack,
    Use,
    Close,
    Prev,
    Next,
    Restart,
}

#[derive(Clone, Debug)]
pub struct Button {
    pub id: BtnId,
    /// bottom-left corner + size
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub label: String,
    /// highlighted when the action is currently useful
    pub hot: bool,
}

const STICK_RADIUS: f32 = 90.0;
const STICK_DEADZONE: f32 = 0.30;
/// Touches that start in this lower-left area steer the stick instead of tapping.
const ZONE: (f32, f32) = (560.0, 380.0);

pub fn buttons(g: &Game) -> Vec<Button> {
    let b = |id, x, y, w, h, label: &str, hot| Button { id, x, y, w, h, label: label.to_string(), hot };
    let mut v = Vec::new();
    if g.phase == Phase::GameOver {
        v.push(b(BtnId::Restart, SCREEN_W / 2.0 - 160.0, 180.0, 320.0, 80.0, "TAP TO RESTART", true));
        return v;
    }
    match g.ui_mode {
        UiMode::Map | UiMode::Backpack => v.push(b(BtnId::Close, 1000.0, 580.0, 80.0, 70.0, "X", true)),
        UiMode::Cards => {
            v.push(b(BtnId::Close, 1000.0, 580.0, 80.0, 70.0, "X", true));
            v.push(b(BtnId::Prev, 70.0, 330.0, 100.0, 90.0, "\u{25B2}", false));
            v.push(b(BtnId::Next, 1110.0, 330.0, 100.0, 90.0, "\u{25BC}", false));
        }
        UiMode::None => {
            v.push(b(BtnId::Map, 1180.0, 470.0, 80.0, 70.0, "MAP", false));
            v.push(b(BtnId::Cards, 1180.0, 390.0, 80.0, 70.0, "CARDS", false));
            if g.player_visible() {
                v.push(b(BtnId::Pack, 1180.0, 310.0, 80.0, 70.0, "PACK", g.pickaxe.broken()));
            }
            if g.player_visible() {
                let act = match g.phase {
                    Phase::Basecamp => ("BOARD", g.near_truck()),
                    Phase::Exploring => ("TRUCK", g.player_on_road() || g.player_at_camp()),
                    _ => ("E", false),
                };
                v.push(b(BtnId::Act, 930.0, 40.0, 130.0, 110.0, act.0, act.1));
                if g.phase == Phase::Exploring {
                    v.push(b(BtnId::Break, 1090.0, 40.0, 170.0, 170.0, "BREAK", g.nearest_outcrop().is_some()));
                    // context button: fill the bottle at water, eat under a tree
                    let thirsty = g.water < WATER_MAX - 0.5;
                    let (text, hot) = if g.near_water() && thirsty {
                        ("FILL", true)
                    } else if g.near_tree() && g.apples + g.sandwiches > 0 && g.energy <= NOT_HUNGRY_ABOVE {
                        ("EAT", true)
                    } else {
                        ("USE", false)
                    };
                    v.push(b(BtnId::Use, 930.0, 160.0, 130.0, 90.0, text, hot));
                }
            }
        }
    }
    v
}

pub fn hit(bs: &[Button], x: f32, y: f32) -> Option<BtnId> {
    const PAD: f32 = 10.0;
    bs.iter()
        .find(|b| x >= b.x - PAD && x <= b.x + b.w + PAD && y >= b.y - PAD && y <= b.y + b.h + PAD)
        .map(|b| b.id)
}

/// Translate a pressed button into this tick's input flags.
pub fn apply(id: BtnId, input: &mut Input, ui: UiMode) {
    match id {
        BtnId::Break => input.space_pressed = true,
        BtnId::Act => input.interact_pressed = true,
        BtnId::Map => input.map_pressed = true,
        BtnId::Cards => input.cards_pressed = true,
        BtnId::Pack => input.pack_pressed = true,
        BtnId::Use => input.use_pressed = true,
        BtnId::Close => {
            // closing is a toggle of whichever overlay is open
            match ui {
                UiMode::Cards => input.cards_pressed = true,
                UiMode::Backpack => input.pack_pressed = true,
                _ => input.map_pressed = true,
            }
        }
        BtnId::Prev => input.page_up_pressed = true,
        BtnId::Next => input.page_down_pressed = true,
        BtnId::Restart => input.restart_pressed = true,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stick {
    pub id: u64,
    pub base: (f32, f32),
    pub pos: (f32, f32),
}

impl Stick {
    pub fn vector(&self) -> (f32, f32) {
        let (dx, dy) = (self.pos.0 - self.base.0, self.pos.1 - self.base.1);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return (0.0, 0.0);
        }
        let k = (len.min(STICK_RADIUS)) / STICK_RADIUS / len;
        (dx * k, dy * k)
    }

    /// (left, right, up, down) with a dead zone; diagonals come out as two flags.
    pub fn dirs(&self) -> (bool, bool, bool, bool) {
        let (x, y) = self.vector();
        (x < -STICK_DEADZONE, x > STICK_DEADZONE, y > STICK_DEADZONE, y < -STICK_DEADZONE)
    }

    /// Where to draw the knob.
    pub fn knob(&self) -> (f32, f32) {
        let (x, y) = self.vector();
        (self.base.0 + x * STICK_RADIUS, self.base.1 + y * STICK_RADIUS)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tap {
    Button(BtnId),
    StickStarted,
    /// A tap on the world: click-to-move.
    World(f32, f32),
}

#[derive(Default)]
pub struct TouchState {
    /// becomes true on the first real touch; shows the on-screen controls
    pub active: bool,
    pub stick: Option<Stick>,
}

impl TouchState {
    pub fn in_stick_zone(x: f32, y: f32) -> bool {
        x < ZONE.0 && y < ZONE.1
    }

    pub fn down(&mut self, id: u64, x: f32, y: f32, g: &Game) -> Tap {
        self.active = true;
        let bs = buttons(g);
        if let Some(b) = hit(&bs, x, y) {
            return Tap::Button(b);
        }
        let on_foot_world = g.ui_mode == UiMode::None && g.player_visible();
        if on_foot_world && self.stick.is_none() && Self::in_stick_zone(x, y) {
            self.stick = Some(Stick { id, base: (x, y), pos: (x, y) });
            return Tap::StickStarted;
        }
        Tap::World(x, y)
    }

    pub fn moved(&mut self, id: u64, x: f32, y: f32) {
        if let Some(s) = &mut self.stick {
            if s.id == id {
                s.pos = (x, y);
            }
        }
    }

    pub fn up(&mut self, id: u64) {
        if self.stick.map_or(false, |s| s.id == id) {
            self.stick = None;
        }
    }

    pub fn dirs(&self) -> (bool, bool, bool, bool) {
        self.stick.map_or((false, false, false, false), |s| s.dirs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field_game() -> Game {
        let mut g = Game::new(1);
        g.phase = Phase::Exploring;
        g
    }

    #[test]
    fn buttons_follow_phase_and_overlay() {
        let mut g = Game::new(1);
        let ids = |g: &Game| buttons(g).iter().map(|b| b.id).collect::<Vec<_>>();
        assert!(ids(&g).contains(&BtnId::Act) && !ids(&g).contains(&BtnId::Break));
        g.phase = Phase::Exploring;
        assert!(ids(&g).contains(&BtnId::Break));
        g.ui_mode = UiMode::Cards;
        assert_eq!(ids(&g), vec![BtnId::Close, BtnId::Prev, BtnId::Next]);
        g.ui_mode = UiMode::None;
        g.phase = Phase::TruckOut;
        assert_eq!(ids(&g), vec![BtnId::Map, BtnId::Cards]);
        g.ui_mode = UiMode::Backpack;
        g.phase = Phase::Exploring;
        assert_eq!(ids(&g), vec![BtnId::Close]);
        g.ui_mode = UiMode::None;
        g.phase = Phase::GameOver;
        assert_eq!(ids(&g), vec![BtnId::Restart]);
    }

    #[test]
    fn buttons_stay_on_screen_and_do_not_overlap_the_stick_zone() {
        for phase in [Phase::Basecamp, Phase::Exploring, Phase::TruckOut, Phase::GameOver] {
            for ui in [UiMode::None, UiMode::Map, UiMode::Cards, UiMode::Backpack] {
                let mut g = Game::new(1);
                g.phase = phase;
                g.ui_mode = ui;
                for b in buttons(&g) {
                    assert!(b.x >= 0.0 && b.y >= 0.0 && b.x + b.w <= SCREEN_W && b.y + b.h <= SCREEN_H, "{:?}", b);
                    if ui == UiMode::None && phase != Phase::GameOver {
                        assert!(b.x > ZONE.0, "{:?} intrudes on the stick zone", b);
                    }
                }
            }
        }
    }

    #[test]
    fn taps_route_to_button_stick_or_world() {
        let g = field_game();
        let mut t = TouchState::default();
        assert!(!t.active);
        assert_eq!(t.down(1, 1200.0, 100.0, &g), Tap::Button(BtnId::Break));
        assert!(t.active);
        assert_eq!(t.down(2, 700.0, 500.0, &g), Tap::World(700.0, 500.0));
        assert_eq!(t.down(3, 200.0, 150.0, &g), Tap::StickStarted);
        // a second finger in the zone does not steal the stick
        assert_eq!(t.down(4, 300.0, 200.0, &g), Tap::World(300.0, 200.0));
    }

    #[test]
    fn stick_gives_directions_with_deadzone_and_releases() {
        let g = field_game();
        let mut t = TouchState::default();
        t.down(7, 200.0, 200.0, &g);
        assert_eq!(t.dirs(), (false, false, false, false));
        t.moved(7, 200.0 + 50.0, 200.0 + 5.0);
        assert_eq!(t.dirs(), (false, true, false, false));
        t.moved(7, 200.0 - 80.0, 200.0 - 80.0);
        assert_eq!(t.dirs(), (true, false, false, true));
        t.moved(9, 500.0, 500.0); // other finger ignored
        assert_eq!(t.dirs(), (true, false, false, true));
        let (kx, ky) = t.stick.unwrap().knob();
        assert!(((kx - 200.0).powi(2) + (ky - 200.0).powi(2)).sqrt() <= STICK_RADIUS + 0.01);
        t.up(7);
        assert!(t.stick.is_none() && t.dirs() == (false, false, false, false));
    }

    #[test]
    fn no_stick_inside_overlays_or_rides() {
        let mut g = field_game();
        g.ui_mode = UiMode::Map;
        let mut t = TouchState::default();
        assert!(matches!(t.down(1, 200.0, 150.0, &g), Tap::World(..)));
        g.ui_mode = UiMode::None;
        g.phase = Phase::TruckBack;
        assert!(matches!(t.down(2, 200.0, 150.0, &g), Tap::World(..)));
    }

    #[test]
    fn pack_and_use_buttons_work_in_the_field() {
        let mut g = field_game();
        let ids = |g: &Game| buttons(g).iter().map(|b| b.id).collect::<Vec<_>>();
        assert!(ids(&g).contains(&BtnId::Pack) && ids(&g).contains(&BtnId::Use));
        // PACK opens the backpack and its X closes it again
        let mut inp = Input::default();
        apply(BtnId::Pack, &mut inp, UiMode::None);
        g.tick(&inp);
        assert_eq!(g.ui_mode, UiMode::Backpack);
        let mut inp = Input::default();
        apply(BtnId::Close, &mut inp, UiMode::Backpack);
        g.tick(&inp);
        assert_eq!(g.ui_mode, UiMode::None);
        // USE is the F key: it flags use_pressed
        let mut inp = Input::default();
        apply(BtnId::Use, &mut inp, UiMode::None);
        assert!(inp.use_pressed);
    }

    #[test]
    fn pressing_buttons_drives_the_game() {
        let mut g = Game::new(1);
        let mut inp = Input::default();
        apply(BtnId::Map, &mut inp, UiMode::None);
        g.tick(&inp);
        assert_eq!(g.ui_mode, UiMode::Map);
        let mut inp = Input::default();
        apply(BtnId::Close, &mut inp, UiMode::Map);
        g.tick(&inp);
        assert_eq!(g.ui_mode, UiMode::None);
    }
}
