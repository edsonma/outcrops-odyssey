//! Touch-first UI: energy/water bars, round icon buttons, the backpack / map / cards panels and the
//! floating thumb-stick. All state lives in `game.rs` / `kit.rs`; this only draws it and turns
//! presses into the same `Input` flags the keyboard uses.

use crate::game::*;
use crate::kit;
use crate::map_view;
use crate::{CamMode, GameRes, PendingInput, Quality};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// ----------------------------------------------------------------------- helpers
/// Nominal font size at the 720-px-tall reference screen; `scale_fonts` shrinks it on small screens.
#[derive(Component)]
pub struct BaseFont(pub f32);

fn txt(size: f32, s: impl Into<String>, color: Color) -> impl Bundle {
    (
        Text::new(s.into()),
        TextFont { font_size: bevy::text::FontSize::Px(size), ..default() },
        TextColor(color),
        BaseFont(size),
    )
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

/// Text size follows the window height (1.0 at 720 px and above, down to 0.5): phones get small type.
pub fn scale_fonts(windows: Query<&Window>, quality: Res<Quality>, mut q: Query<(&BaseFont, &mut TextFont)>) {
    let Ok(w) = windows.single() else { return };
    let mut k = (w.height() / 720.0).clamp(0.5, 1.0);
    if quality.mobile {
        k *= 0.9;
    }
    for (base, mut tf) in &mut q {
        let want = (base.0 * k).max(9.0);
        let same = matches!(tf.font_size, bevy::text::FontSize::Px(v) if (v - want).abs() < 0.01);
        if !same {
            tf.font_size = bevy::text::FontSize::Px(want);
        }
    }
}

// ----------------------------------------------------------------------- components
#[derive(Component)]
pub struct PackRoot;
#[derive(Component)]
pub struct MapRoot;
#[derive(Component)]
pub struct CardsRoot;
#[derive(Component)]
pub struct BarFill(BarKind);
#[derive(Component)]
pub struct PackRowBg(usize);
#[derive(Component)]
pub struct ActionBtn(Act);
#[derive(Component)]
pub struct MapImage;
#[derive(Component)]
pub struct MapMarker(Marker);
#[derive(Component)]
pub struct LegendSwatch(usize);
#[derive(Component)]
pub struct CardRowBg(usize);

#[derive(Clone, Copy)]
enum Marker {
    Player,
    Truck,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BarKind {
    Energy,
    Water,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Act {
    Break,
    Use,
    Truck,
    Pack,
    Map,
    Cards,
    Cam,
    Close,
    Prev,
    Next,
}

impl Act {
    fn icon(self) -> &'static str {
        match self {
            Act::Break => "break",
            Act::Use => "use",
            Act::Truck => "truck",
            Act::Pack => "pack",
            Act::Map => "map",
            Act::Cards => "cards",
            Act::Cam => "cam",
            Act::Close => "close",
            Act::Prev => "prev",
            Act::Next => "next",
        }
    }
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

#[derive(Component)]
pub enum PanelText {
    MapTitle,
    CardsTitle,
    CardsFooter,
    CardName(usize),
    CardInfo(usize),
    Legend(usize),
}

#[derive(Resource)]
pub struct MapHandle(Handle<Image>);

const PANEL_BG: Color = Color::srgba(0.07, 0.06, 0.04, 0.93);
const CARD_ROWS: usize = CARDS_PER_PAGE;
const LEGEND_ROWS: usize = 14;

fn card_row_rect(i: usize) -> (f32, f32, f32, f32) {
    let (px, py, pw, ph) = PANEL;
    (px + 24.0, py + ph - 130.0 - 56.0 * i as f32, pw - 100.0, 52.0)
}

/// Where the map picture sits inside the panel (virtual screen).
const MAP_BOX: (f32, f32, f32, f32) = (PANEL.0 + 24.0, PANEL.1 + 26.0, 560.0, 500.0);

pub fn setup_kit_ui(mut commands: Commands, assets: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
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
                left: Val::Px(12.0),
                bottom: Val::Px(10.0 + i as f32 * 22.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|p| {
                p.spawn((Node { width: Val::Px(150.0), height: Val::Px(11.0), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55))))
                    .with_children(|b| {
                        b.spawn((BarFill(kind), Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(color)));
                    });
                p.spawn(txt(13.0, label, Color::srgba(1.0, 1.0, 1.0, 0.8)));
            });
    }

    // ---- round icon buttons. Gameplay ones sit bottom-right (a 2x2 pad) and top-right (map/cards/view).
    let icon_btn = |commands: &mut Commands, act: Act, node: Node| {
        let h: Handle<Image> = assets.load(format!("ui/icon_{}.png", act.icon()));
        commands
            .spawn((
                Button,
                ActionBtn(act),
                Node {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::MAX,
                    min_width: Val::Px(40.0),
                    min_height: Val::Px(40.0),
                    ..node
                },
                BackgroundColor(Color::srgba(0.12, 0.10, 0.07, 0.72)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.25)),
            ))
            .with_children(|b| {
                b.spawn((ImageNode::new(h), Node { width: Val::Percent(68.0), height: Val::Percent(68.0), ..default() }));
            });
    };
    let sq = |v: f32| (Val::Vh(v), Val::Vh(v));
    let place = |right: f32, bottom: f32, size: f32| {
        let (w, h) = sq(size);
        Node { right: Val::Vh(right), bottom: Val::Vh(bottom), width: w, height: h, ..default() }
    };
    let b = 11.0; // big button size, vh
    let gap = 1.6;
    icon_btn(&mut commands, Act::Break, place(2.0, 3.0, b));
    icon_btn(&mut commands, Act::Use, place(2.0 + b + gap, 3.0, b));
    icon_btn(&mut commands, Act::Truck, place(2.0, 3.0 + b + gap, b));
    icon_btn(&mut commands, Act::Pack, place(2.0 + b + gap, 3.0 + b + gap, b));
    let top = |right: f32| {
        let (w, h) = sq(8.0);
        Node { right: Val::Vh(right), top: Val::Vh(1.6), width: w, height: h, ..default() }
    };
    icon_btn(&mut commands, Act::Cam, top(1.6));
    icon_btn(&mut commands, Act::Cards, top(1.6 + 8.0 + 1.2));
    icon_btn(&mut commands, Act::Map, top(1.6 + 2.0 * (8.0 + 1.2)));

    // panel buttons live on the panel itself (virtual-screen coordinates)
    let (px, py, pw, ph) = PANEL;
    for (act, x, y) in [(Act::Close, px + pw - 66.0, py + ph - 66.0), (Act::Prev, px + pw - 66.0, py + ph - 150.0), (Act::Next, px + pw - 66.0, py + ph - 214.0)] {
        let v = virt(x, y, 54.0, 54.0);
        icon_btn(&mut commands, act, Node { left: v.left, bottom: v.bottom, width: v.width, height: v.height, min_width: Val::Px(34.0), min_height: Val::Px(34.0), ..default() });
    }

    // ---- backpack panel (hidden until the game's UiMode::Backpack)
    commands
        .spawn((PackRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, ..default() }))
        .with_children(|root| {
            root.spawn((virt(px, py, pw, ph), BackgroundColor(PANEL_BG)));
            let text = |root: &mut ChildSpawnerCommands, what: PackText, size: f32, x: f32, y_bottom: f32, w: f32| {
                let mut n = virt(x, y_bottom, w, size * 1.6);
                n.align_items = AlignItems::Center;
                root.spawn((what, n, txt(size, "", Color::WHITE)));
            };
            text(root, PackText::Title, 30.0, px + 28.0, py + ph - 62.0, pw - 300.0);
            text(root, PackText::Status, 18.0, px + 28.0, py + ph - 96.0, pw - 120.0);
            for i in 0..kit::ACTIONS.len() {
                let (rx, ry, rw, rh) = pack_row_rect(i);
                root.spawn((PackRowBg(i), virt(rx, ry, rw, rh), BackgroundColor(Color::NONE)));
                text(root, PackText::Label(i), 22.0, rx + 16.0, ry + rh * 0.48, rw - 32.0);
                text(root, PackText::Detail(i), 15.0, rx + 16.0, ry + 3.0, rw - 32.0);
            }
            text(root, PackText::Msg, 22.0, px + 28.0, py + 70.0, pw - 56.0);
            text(root, PackText::Footer, 16.0, px + 28.0, py + 24.0, pw - 56.0);
        });

    // ---- map panel
    let img = images.add(Image::new_fill(
        Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[14, 16, 12, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    commands.insert_resource(MapHandle(img.clone()));
    commands
        .spawn((MapRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, ..default() }))
        .with_children(|root| {
            root.spawn((virt(px, py, pw, ph), BackgroundColor(PANEL_BG)));
            let mut t = virt(px + 28.0, py + ph - 62.0, pw - 120.0, 48.0);
            t.align_items = AlignItems::Center;
            root.spawn((PanelText::MapTitle, t, txt(28.0, "", Color::srgb(0.97, 0.88, 0.60))));
            // the picture, centred in its box; markers are its children so they use its own percentages
            let (mx, my, mw, mh) = MAP_BOX;
            let mut boxn = virt(mx, my, mw, mh);
            boxn.justify_content = JustifyContent::Center;
            boxn.align_items = AlignItems::Center;
            root.spawn((boxn, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)))).with_children(|bx| {
                bx.spawn((MapImage, ImageNode::new(img), Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }))
                    .with_children(|m| {
                        for (kind, color) in [(Marker::Truck, Color::srgb(0.95, 0.9, 0.3)), (Marker::Player, Color::srgb(0.95, 0.35, 0.1))] {
                            m.spawn((
                                MapMarker(kind),
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(11.0),
                                    height: Val::Px(11.0),
                                    margin: UiRect { left: Val::Px(-5.5), bottom: Val::Px(-5.5), ..default() },
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(color),
                                BorderColor::all(Color::WHITE),
                            ));
                        }
                    });
            });
            // legend
            for i in 0..LEGEND_ROWS {
                let y = py + ph - 120.0 - 30.0 * i as f32;
                root.spawn((LegendSwatch(i), virt(px + 610.0, y + 4.0, 16.0, 16.0), BackgroundColor(Color::NONE)));
                let mut n = virt(px + 634.0, y - 2.0, pw - 700.0, 26.0);
                n.align_items = AlignItems::Center;
                root.spawn((PanelText::Legend(i), n, txt(15.0, "", Color::srgb(0.92, 0.9, 0.82))));
            }
        });

    // ---- cards panel
    commands
        .spawn((CardsRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, ..default() }))
        .with_children(|root| {
            root.spawn((virt(px, py, pw, ph), BackgroundColor(PANEL_BG)));
            let mut t = virt(px + 28.0, py + ph - 62.0, pw - 120.0, 48.0);
            t.align_items = AlignItems::Center;
            root.spawn((PanelText::CardsTitle, t, txt(28.0, "", Color::srgb(0.97, 0.88, 0.60))));
            for i in 0..CARD_ROWS {
                let (rx, ry, rw, rh) = card_row_rect(i);
                root.spawn((CardRowBg(i), virt(rx, ry, rw, rh), BackgroundColor(Color::NONE)));
                let mut a = virt(rx + 12.0, ry + rh * 0.5, rw - 24.0, 26.0);
                a.align_items = AlignItems::Center;
                root.spawn((PanelText::CardName(i), a, txt(19.0, "", Color::srgb(0.95, 0.92, 0.85))));
                let mut b = virt(rx + 12.0, ry + 2.0, rw - 24.0, 22.0);
                b.align_items = AlignItems::Center;
                root.spawn((PanelText::CardInfo(i), b, txt(13.0, "", Color::srgb(0.70, 0.78, 0.66))));
            }
            let mut f = virt(px + 28.0, py + 20.0, pw - 120.0, 30.0);
            f.align_items = AlignItems::Center;
            root.spawn((PanelText::CardsFooter, f, txt(15.0, "", Color::srgba(0.86, 0.86, 0.86, 0.85))));
        });
}

// ---------------------------------------------------------------------- updates
pub fn update_bars(game: Res<GameRes>, mut bars: Query<(&mut Node, &mut BackgroundColor, &BarFill)>) {
    let g = &game.0;
    for (mut node, mut bg, bar) in &mut bars {
        let (f, ok, low) = match bar.0 {
            BarKind::Energy => ((g.energy / ENERGY_MAX).clamp(0.0, 1.0), Color::srgb(0.43, 0.78, 0.43), Color::srgb(0.86, 0.35, 0.27)),
            BarKind::Water => ((g.water / kit::WATER_MAX).clamp(0.0, 1.0), Color::srgb(0.35, 0.63, 0.90), Color::srgb(0.90, 0.58, 0.27)),
        };
        node.width = Val::Percent(f * 100.0);
        bg.0 = if f > 0.25 { ok } else { low };
    }
}

/// Show only the buttons that make sense right now, and glow the one that is useful.
pub fn update_buttons(game: Res<GameRes>, mut btns: Query<(&ActionBtn, &mut Node, &mut BorderColor, &mut BackgroundColor)>) {
    let g = &game.0;
    let playing = matches!(g.phase, Phase::Exploring | Phase::Basecamp);
    let on_foot = g.phase == Phase::Exploring && g.player_visible();
    for (b, mut node, mut border, mut bg) in &mut btns {
        let (visible, hot) = match (g.ui_mode, b.0) {
            (UiMode::None, Act::Map | Act::Cards | Act::Cam) => (playing, false),
            (UiMode::None, Act::Truck) => (playing || g.phase == Phase::GameOver, g.near_truck() || g.player_at_camp() || g.player_on_road() || g.phase == Phase::GameOver),
            (UiMode::None, Act::Pack) => (playing, g.pickaxe.broken()),
            (UiMode::None, Act::Break) => (on_foot, g.nearest_outcrop().is_some()),
            (UiMode::None, Act::Use) => (
                on_foot,
                (g.near_water() && g.water < kit::WATER_MAX - 0.5) || (g.near_tree() && g.apples + g.sandwiches > 0 && g.energy <= kit::NOT_HUNGRY_ABOVE),
            ),
            (UiMode::None, _) => (false, false),
            (UiMode::Cards, Act::Close | Act::Prev | Act::Next) => (true, false),
            (_, Act::Close) => (true, true),
            _ => (false, false),
        };
        let want = if visible { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
        border.set_all(if hot { Color::srgb(1.0, 0.82, 0.35) } else { Color::srgba(1.0, 1.0, 1.0, 0.25) });
        bg.0 = if hot { Color::srgba(0.42, 0.30, 0.10, 0.85) } else { Color::srgba(0.12, 0.10, 0.07, 0.72) };
    }
}

pub fn update_pack(
    game: Res<GameRes>,
    mut root: Query<&mut Node, With<PackRoot>>,
    mut rows: Query<(&mut BackgroundColor, &PackRowBg)>,
    mut texts: Query<(&mut Text, &mut TextColor, &PackText)>,
) {
    let g = &game.0;
    let open = g.ui_mode == UiMode::Backpack;
    if let Ok(mut n) = root.single_mut() {
        n.display = if open { Display::Flex } else { Display::None };
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
        let (new, c): (String, Option<Color>) = match kind {
            PackText::Title => (if at_camp { "BACKPACK & CAMP STORE".into() } else { "BACKPACK".into() }, Some(Color::srgb(0.97, 0.88, 0.60))),
            PackText::Status => (format!("Points: {}    Pack {}/{} slots", g.outcrop_points, g.backpack_used(), kit::BACKPACK_SLOTS), Some(Color::srgb(1.0, 0.94, 0.78))),
            PackText::Msg => (g.pack_msg.as_ref().map(|(m, _)| m.clone()).unwrap_or_default(), Some(Color::WHITE)),
            PackText::Footer => ("Tap a row to use it  .  X to close".into(), Some(Color::srgba(0.86, 0.86, 0.86, 0.8))),
            PackText::Label(i) | PackText::Detail(i) => {
                let Some(row) = pack_rows.get(*i) else { continue };
                if matches!(kind, PackText::Label(_)) {
                    (row.label.clone(), Some(if row.enabled { Color::srgb(0.96, 0.94, 0.88) } else { Color::srgba(0.65, 0.63, 0.59, 0.8) }))
                } else {
                    (row.detail.clone(), Some(if row.enabled { Color::srgb(0.75, 0.88, 0.67) } else { Color::srgba(0.59, 0.57, 0.53, 0.75) }))
                }
            }
        };
        if text.0 != new {
            text.0 = new;
        }
        if let Some(c) = c {
            if color.0 != c {
                color.0 = c;
            }
        }
    }
}

pub fn update_map(
    game: Res<GameRes>,
    handle: Res<MapHandle>,
    mut images: ResMut<Assets<Image>>,
    mut root: Query<&mut Node, (With<MapRoot>, Without<MapMarker>, Without<MapImage>)>,
    mut markers: Query<(&MapMarker, &mut Node), (Without<MapRoot>, Without<MapImage>)>,
    mut texts: Query<(&mut Text, &PanelText)>,
    mut swatches: Query<(&LegendSwatch, &mut BackgroundColor)>,
    mut frame: Local<u32>,
    mut was_open: Local<bool>,
) {
    let g = &game.0;
    let open = g.ui_mode == UiMode::Map;
    if let Ok(mut n) = root.single_mut() {
        n.display = if open { Display::Flex } else { Display::None };
    }
    if !open {
        *was_open = false;
        return;
    }
    *frame += 1;
    // repaint on opening and about once a second afterwards (new chunks, moving truck)
    let (x0, y0, ww, wh) = map_view::world_rect(g);
    if !*was_open || *frame % 60 == 0 {
        let m = map_view::build(g);
        if let Some(mut img) = images.get_mut(&handle.0) {
            *img = Image::new(
                Extent3d { width: m.w as u32, height: m.h as u32, depth_or_array_layers: 1 },
                TextureDimension::D2,
                m.rgba,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
        }
    }
    *was_open = true;
    // markers: percentages of the picture
    let at = |x: f32, y: f32| (Val::Percent(((x - x0) / ww * 100.0).clamp(0.0, 100.0)), Val::Percent(((y - y0) / wh * 100.0).clamp(0.0, 100.0)));
    for (m, mut n) in &mut markers {
        let (px, py) = match m.0 {
            Marker::Player => (g.player.x, g.player.y),
            Marker::Truck => (g.truck.x, g.truck.y),
        };
        let (l, b) = at(px, py);
        n.left = l;
        n.bottom = b;
    }
    // legend
    let mut rows: Vec<(String, Option<(u8, u8, u8)>)> = vec![
        (g.region.name.to_string(), None),
        (format!("{}  -  level {}/{}", g.region.place, g.level + 1, crate::geology::REGIONS.len()), None),
    ];
    for u in g.region.units {
        rows.push((u.name.to_string(), Some(u.color)));
    }
    for (c, t) in [((58u8, 118u8, 188u8), "Water"), ((24, 78, 40), "Dense forest"), ((126, 118, 110), "Cliffs"), ((230, 215, 120), "Farm"), ((170, 170, 178), "Main road")] {
        rows.push((t.to_string(), Some(c)));
    }
    rows.truncate(LEGEND_ROWS);
    for (mut t, kind) in &mut texts {
        match kind {
            PanelText::MapTitle => t.0 = crate::ascii(&format!("MAP - {}", g.region.name)),
            PanelText::Legend(i) => t.0 = rows.get(*i).map(|r| crate::ascii(&r.0)).unwrap_or_default(),
            _ => {}
        }
    }
    for (s, mut bg) in &mut swatches {
        bg.0 = match rows.get(s.0).and_then(|r| r.1) {
            Some((r, gr, b)) => Color::srgb_u8(r, gr, b),
            None => Color::NONE,
        };
    }
}

/// Keep the map picture's shape: fit the world's aspect inside its box.
pub fn fit_map_image(game: Res<GameRes>, mut img: Query<&mut Node, With<MapImage>>) {
    if game.0.ui_mode != UiMode::Map {
        return;
    }
    let (_, _, ww, wh) = map_view::world_rect(&game.0);
    let box_aspect = MAP_BOX.2 / MAP_BOX.3;
    let a = ww / wh;
    let (w, h) = if a >= box_aspect { (100.0, 100.0 * box_aspect / a) } else { (100.0 * a / box_aspect, 100.0) };
    if let Ok(mut n) = img.single_mut() {
        n.width = Val::Percent(w);
        n.height = Val::Percent(h);
    }
}

pub fn update_cards(
    game: Res<GameRes>,
    mut root: Query<&mut Node, With<CardsRoot>>,
    mut rows: Query<(&mut BackgroundColor, &CardRowBg)>,
    mut texts: Query<(&mut Text, &PanelText)>,
) {
    let g = &game.0;
    let open = g.ui_mode == UiMode::Cards;
    if let Ok(mut n) = root.single_mut() {
        n.display = if open { Display::Flex } else { Display::None };
    }
    if !open {
        return;
    }
    let start = g.card_page * CARDS_PER_PAGE;
    let page: Vec<&Card> = g.cards.iter().skip(start).take(CARDS_PER_PAGE).collect();
    for (mut bg, r) in &mut rows {
        bg.0 = if r.0 < page.len() { Color::srgba(0.16, 0.14, 0.11, if r.0 % 2 == 0 { 0.6 } else { 0.35 }) } else { Color::NONE };
    }
    for (mut t, kind) in &mut texts {
        let new = match kind {
            PanelText::CardsTitle => format!("CARDS ({} collected)", g.cards.len()),
            PanelText::CardsFooter => {
                if g.cards.is_empty() {
                    "No cards yet - go break some outcrops!".to_string()
                } else {
                    format!("Page {}/{}   (use the arrows)", g.card_page + 1, g.total_card_pages())
                }
            }
            PanelText::CardName(i) => page
                .get(*i)
                .map(|c| {
                    let mut badges: Vec<String> = Vec::new();
                    if let Some(m) = c.mineral {
                        badges.push(m.to_string());
                    }
                    if let Some(f) = c.fossil {
                        badges.push(f.to_string());
                    }
                    if c.bonus_rock {
                        badges.push("Bonus rock".to_string());
                    }
                    let b = if badges.is_empty() { String::new() } else { format!("   [{}]", badges.join(", ")) };
                    format!("{}. {}{}", start + i + 1, c.name, b)
                })
                .unwrap_or_default(),
            PanelText::CardInfo(i) => page.get(*i).map(|c| format!("{} - {} ({}) - {}", c.lithology, c.unit, c.parent, c.age)).unwrap_or_default(),
            _ => continue,
        };
        let new = crate::ascii(&new);
        if t.0 != new {
            t.0 = new;
        }
    }
}

/// Button presses become the same one-tick input flags as the keys.
pub fn button_input(
    q: Query<(&Interaction, &ActionBtn), Changed<Interaction>>,
    game: Res<GameRes>,
    mut mode: ResMut<CamMode>,
    mut pend: ResMut<PendingInput>,
) {
    for (i, b) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        match b.0 {
            Act::Break => pend.0.space_pressed = true,
            Act::Use => pend.0.use_pressed = true,
            Act::Truck => {
                pend.0.interact_pressed = true;
                pend.0.restart_pressed |= game.0.phase == Phase::GameOver;
            }
            Act::Pack => pend.0.pack_pressed = true,
            Act::Map => pend.0.map_pressed = true,
            Act::Cards => pend.0.cards_pressed = true,
            Act::Cam => *mode = if *mode == CamMode::Tilted { CamMode::Side } else { CamMode::Tilted },
            Act::Close => match game.0.ui_mode {
                UiMode::Map => pend.0.map_pressed = true,
                UiMode::Cards => pend.0.cards_pressed = true,
                UiMode::Backpack => pend.0.pack_pressed = true,
                UiMode::None => {}
            },
            Act::Prev => pend.0.page_up_pressed = true,
            Act::Next => pend.0.page_down_pressed = true,
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
