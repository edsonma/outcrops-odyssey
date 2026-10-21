//! Kit and backpack UI: energy/water bars, the backpack & camp-store panel, and big
//! touch-friendly action buttons. All state lives in `game.rs` / `kit.rs`; this only draws it
//! and turns presses into the same `Input` flags the keyboard uses.

use crate::game::*;
use crate::kit;
use crate::{GameRes, PendingInput};
use bevy::prelude::*;

#[derive(Component)]
pub struct PackRoot;
#[derive(Component)]
pub struct BarFill(BarKind);
#[derive(Component)]
pub struct PackRowBg(usize);
#[derive(Component)]
pub struct ActionBtn(Act);
#[derive(Component)]
pub struct BtnLabel;

#[derive(Clone, Copy)]
enum BarKind {
    Energy,
    Water,
}

#[derive(Clone, Copy)]
enum Act {
    Break,
    Use,
    Truck,
    Pack,
}

#[derive(Component)]
pub enum PackText {
    Title,
    Status,
    Msg,
    Footer,
    Label(usize),
    Detail(usize),
}

fn font(size: f32) -> TextFont {
    TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }
}

/// A rectangle given in the game's 1280x720 y-up virtual screen, as a percentage-positioned node,
/// so taps map onto it exactly like in the Macroquad front-end.
fn virt(x: f32, y_bottom: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(x / 1280.0 * 100.0),
        bottom: Val::Percent(y_bottom / 720.0 * 100.0),
        width: Val::Percent(w / 1280.0 * 100.0),
        height: Val::Percent(h / 720.0 * 100.0),
        ..default()
    }
}

pub fn setup_kit_ui(mut commands: Commands) {
    // ---- bars (bottom-left)
    for (i, (kind, color, label)) in [
        (BarKind::Energy, Color::srgb(0.43, 0.78, 0.43), "Energy"),
        (BarKind::Water, Color::srgb(0.35, 0.63, 0.90), "Water"),
    ]
    .into_iter()
    .enumerate()
    {
        commands
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                bottom: Val::Px(14.0 + i as f32 * 26.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|p| {
                p.spawn((Node { width: Val::Px(200.0), height: Val::Px(14.0), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55))))
                    .with_children(|b| {
                        b.spawn((BarFill(kind), Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(color)));
                    });
                p.spawn((Text::new(label), font(16.0), TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8))));
            });
    }

    // ---- action buttons (bottom-right): the touch-friendly twins of SPACE / F / E / B
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(14.0),
            bottom: Val::Px(14.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            align_items: AlignItems::FlexEnd,
            ..default()
        })
        .with_children(|col| {
            for chunk in [[(Act::Pack, "PACK  [B]"), (Act::Truck, "TRUCK  [E]")], [(Act::Use, "FILL / EAT  [F]"), (Act::Break, "BREAK  [SPACE]")]] {
                col.spawn(Node { column_gap: Val::Px(8.0), ..default() }).with_children(|row| {
                    for (act, text) in chunk {
                        row.spawn((
                            Button,
                            ActionBtn(act),
                            Node {
                                min_width: Val::Px(120.0),
                                height: Val::Px(54.0),
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(0.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.12, 0.10, 0.07, 0.72)),
                        ))
                        .with_children(|b| {
                            b.spawn((BtnLabel, Text::new(text), font(18.0), TextColor(Color::srgb(0.96, 0.92, 0.80))));
                        });
                    }
                });
            }
        });

    // ---- backpack panel (hidden until the game's UiMode::Backpack)
    commands
        .spawn((
            PackRoot,
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, ..default() },
        ))
        .with_children(|root| {
            let (px, py, pw, ph) = PANEL;
            root.spawn((virt(px, py, pw, ph), BackgroundColor(Color::srgba(0.07, 0.06, 0.04, 0.93))));
            let text = |root: &mut ChildSpawnerCommands, what: PackText, size: f32, x: f32, y_bottom: f32, w: f32| {
                let mut n = virt(x, y_bottom, w, size * 1.6);
                n.align_items = AlignItems::Center;
                root.spawn((what, n, Text::new(""), font(size), TextColor(Color::WHITE)));
            };
            text(root, PackText::Title, 30.0, px + 28.0, py + ph - 62.0, pw - 56.0);
            text(root, PackText::Status, 18.0, px + 500.0, py + ph - 56.0, pw - 520.0); // same line as the title, clear of the rows
            for i in 0..kit::ACTIONS.len() {
                let (rx, ry, rw, rh) = pack_row_rect(i);
                root.spawn((PackRowBg(i), virt(rx, ry, rw, rh), BackgroundColor(Color::NONE)));
                text(root, PackText::Label(i), 22.0, rx + 16.0, ry + rh * 0.48, rw - 32.0);
                text(root, PackText::Detail(i), 15.0, rx + 16.0, ry + 3.0, rw - 32.0);
            }
            text(root, PackText::Msg, 22.0, px + 28.0, py + 70.0, pw - 56.0);
            text(root, PackText::Footer, 16.0, px + 28.0, py + 24.0, pw - 56.0);
        });
}

pub fn update_kit_ui(
    game: Res<GameRes>,
    mut root: Query<&mut Node, With<PackRoot>>,
    mut bars: Query<(&mut Node, &mut BackgroundColor, &BarFill), Without<PackRoot>>,
    mut rows: Query<(&mut BackgroundColor, &PackRowBg), Without<BarFill>>,
    mut texts: Query<(&mut Text, &mut TextColor, &PackText)>,
) {
    let g = &game.0;
    let open = g.ui_mode == UiMode::Backpack;
    if let Ok(mut n) = root.single_mut() {
        n.display = if open { Display::Flex } else { Display::None };
    }

    for (mut node, mut bg, bar) in &mut bars {
        let (f, ok, low) = match bar.0 {
            BarKind::Energy => ((g.energy / ENERGY_MAX).clamp(0.0, 1.0), Color::srgb(0.43, 0.78, 0.43), Color::srgb(0.86, 0.35, 0.27)),
            BarKind::Water => ((g.water / kit::WATER_MAX).clamp(0.0, 1.0), Color::srgb(0.35, 0.63, 0.90), Color::srgb(0.90, 0.58, 0.27)),
        };
        node.width = Val::Percent(f * 100.0);
        bg.0 = if f > 0.25 { ok } else { low };
    }

    if !open {
        return;
    }
    let pack_rows = g.pack_rows();
    for (mut bg, row) in &mut rows {
        let selected = row.0 == g.pack_sel;
        let on = pack_rows.get(row.0).is_some_and(|r| r.enabled);
        bg.0 = match (selected, row.0 % 2) {
            (true, _) => Color::srgba(0.35, 0.30, 0.15, 0.85),
            (false, 0) => Color::srgba(0.16, 0.14, 0.11, if on { 0.6 } else { 0.35 }),
            (false, _) => Color::srgba(0.16, 0.14, 0.11, if on { 0.35 } else { 0.2 }),
        };
    }
    let at_camp = g.phase == Phase::Basecamp;
    for (mut text, mut color, kind) in &mut texts {
        match kind {
            PackText::Title => {
                text.0 = if at_camp { "BACKPACK & CAMP STORE".into() } else { "BACKPACK".into() };
                color.0 = Color::srgb(0.97, 0.88, 0.60);
            }
            PackText::Status => {
                text.0 = format!("Points: {}    Pack {}/{} slots", g.outcrop_points, g.backpack_used(), kit::BACKPACK_SLOTS);
                color.0 = Color::srgb(1.0, 0.94, 0.78);
            }
            PackText::Msg => {
                text.0 = g.pack_msg.as_ref().map(|(m, _)| m.clone()).unwrap_or_default();
                color.0 = Color::WHITE;
            }
            PackText::Footer => {
                text.0 = "Up/Down select  .  Enter / click to use  .  B to close".into();
                color.0 = Color::srgba(0.86, 0.86, 0.86, 0.8);
            }
            PackText::Label(i) | PackText::Detail(i) => {
                let Some(row) = pack_rows.get(*i) else { continue };
                if matches!(kind, PackText::Label(_)) {
                    text.0 = row.label.clone();
                    color.0 = if row.enabled { Color::srgb(0.96, 0.94, 0.88) } else { Color::srgba(0.65, 0.63, 0.59, 0.8) };
                } else {
                    text.0 = row.detail.clone();
                    color.0 = if row.enabled { Color::srgb(0.75, 0.88, 0.67) } else { Color::srgba(0.59, 0.57, 0.53, 0.75) };
                }
            }
        }
    }
}

/// Button presses become the same one-tick input flags as the keys.
pub fn button_input(q: Query<(&Interaction, &ActionBtn), Changed<Interaction>>, mut pend: ResMut<PendingInput>) {
    for (i, b) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        match b.0 {
            Act::Break => pend.0.space_pressed = true,
            Act::Use => pend.0.use_pressed = true,
            Act::Truck => pend.0.interact_pressed = true,
            Act::Pack => pend.0.pack_pressed = true,
        }
    }
}

// ------------------------------------------------------------------ virtual stick
/// A floating thumb-stick: touch the lower-left of the screen and drag. The same four booleans
/// as the keyboard come out of it (8-way), so walking feels identical on every device.
#[derive(Resource, Default)]
pub struct StickState {
    id: Option<u64>,
    base: Vec2,
    pos: Vec2,
}

const STICK_R: f32 = 62.0; // logical pixels
const STICK_DEAD: f32 = 0.28;

impl StickState {
    pub fn owns(&self, id: u64) -> bool {
        self.id == Some(id)
    }
    /// (left, right, up, down) like the arrow keys; screen-up is "up".
    pub fn dirs(&self) -> (bool, bool, bool, bool) {
        if self.id.is_none() {
            return (false, false, false, false);
        }
        let d = (self.pos - self.base) / STICK_R;
        if d.length() < STICK_DEAD {
            return (false, false, false, false);
        }
        let n = d.normalize_or_zero();
        (n.x < -0.38, n.x > 0.38, n.y < -0.38, n.y > 0.38)
    }
    fn knob(&self) -> Vec2 {
        let d = self.pos - self.base;
        self.base + d.clamp_length_max(STICK_R)
    }
}

#[derive(Component)]
pub struct StickBase;
#[derive(Component)]
pub struct StickKnob;

pub fn setup_stick_ui(mut commands: Commands) {
    let ring = |size: f32| Node {
        position_type: PositionType::Absolute,
        width: Val::Px(size),
        height: Val::Px(size),
        display: Display::None,
        border_radius: BorderRadius::MAX,
        ..default()
    };
    commands.spawn((StickBase, ring(STICK_R * 2.0), BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.10)), BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.45))));
    commands.spawn((StickKnob, ring(STICK_R * 0.9), BackgroundColor(Color::srgba(1.0, 0.85, 0.5, 0.55))));
}

/// Start / drag / release the stick from touches that begin in the lower-left of the screen.
pub fn touch_stick(
    touches: Res<Touches>,
    windows: Query<&Window>,
    game: Res<GameRes>,
    ui_buttons: Query<&Interaction, With<Button>>,
    mut stick: ResMut<StickState>,
    mut base: Query<&mut Node, (With<StickBase>, Without<StickKnob>)>,
    mut knob: Query<&mut Node, (With<StickKnob>, Without<StickBase>)>,
) {
    let Ok(w) = windows.single() else { return };
    let (sw, sh) = (w.width(), w.height());
    let usable = game.0.ui_mode == UiMode::None && game.0.player_visible();

    if stick.id.is_none() && usable && !ui_buttons.iter().any(|i| *i != Interaction::None) {
        for t in touches.iter_just_pressed() {
            let p = t.position();
            // lower-left 55% x 62% of the screen; the action buttons live on the right
            if p.x < sw * 0.55 && p.y > sh * 0.38 {
                stick.id = Some(t.id());
                stick.base = p;
                stick.pos = p;
                break;
            }
        }
    }
    if let Some(id) = stick.id {
        match touches.get_pressed(id) {
            Some(t) if usable => stick.pos = t.position(),
            _ => stick.id = None, // released, cancelled, or a menu opened
        }
    }

    let active = stick.id.is_some();
    let place = |n: &mut Node, centre: Vec2, size: f32| {
        n.display = if active { Display::Flex } else { Display::None };
        n.left = Val::Px(centre.x - size / 2.0);
        n.top = Val::Px(centre.y - size / 2.0);
    };
    if let Ok(mut n) = base.single_mut() {
        place(&mut n, stick.base, STICK_R * 2.0);
        n.border = UiRect::all(Val::Px(2.0));
    }
    if let Ok(mut n) = knob.single_mut() {
        place(&mut n, stick.knob(), STICK_R * 0.9);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(dx: f32, dy: f32) -> StickState {
        StickState { id: Some(1), base: Vec2::new(100.0, 100.0), pos: Vec2::new(100.0 + dx, 100.0 + dy) }
    }

    #[test]
    fn stick_has_a_deadzone_and_eight_directions() {
        assert_eq!(at(5.0, 5.0).dirs(), (false, false, false, false));
        assert_eq!(at(60.0, 0.0).dirs(), (false, true, false, false));
        assert_eq!(at(-60.0, 0.0).dirs(), (true, false, false, false));
        // screen y grows downward, so dragging up is negative y -> "up"
        assert_eq!(at(0.0, -60.0).dirs(), (false, false, true, false));
        assert_eq!(at(0.0, 60.0).dirs(), (false, false, false, true));
        assert_eq!(at(45.0, -45.0).dirs(), (false, true, true, false));
        assert_eq!(StickState::default().dirs(), (false, false, false, false));
    }
}
