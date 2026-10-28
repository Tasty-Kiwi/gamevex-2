use crate::{
    Arcade, Pending, platform,
    rendering::{Frame, RenderOptions},
};
use bevy::{prelude::*, window::PrimaryWindow};
use gamevex_core::{cards::Suit, *};

pub const BG: Color = Color::srgb(0.008, 0.016, 0.031);
pub const PANEL: Color = Color::srgb(0.025, 0.055, 0.085);
pub const CYAN: Color = Color::srgb(0.0, 0.83, 1.0);
pub const MAGENTA: Color = Color::srgb(1.0, 0.0, 0.67);
pub const GOLD: Color = Color::srgb(1.0, 0.67, 0.0);
pub const INK: Color = Color::srgb(0.87, 0.94, 1.0);
pub const MUTED: Color = Color::srgb(0.39, 0.58, 0.68);
pub const EDGE: Color = Color::srgb(0.08, 0.22, 0.29);
pub const GREEN: Color = Color::srgb(0.0, 1.0, 0.72);
pub const RED: Color = Color::srgb(1.0, 0.2, 0.4);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Lobby,
    Mines,
    Cards,
    Settings,
}
#[derive(Clone, Copy)]
pub enum Action {
    Open(Screen),
    Back,
    New(Difficulty, bool),
    Confirm,
    Cancel,
    FlagMode,
    Board,
    Preset,
    Bloom(f32),
    Scanlines(f32),
    Volume(f32),
    Crt,
    Motion,
    Mute,
    Retry,
    Recover,
    Flip,
    Deal,
    ReturnCard,
}
pub struct Hit {
    pub rect: Rect,
    pub action: Action,
}
pub struct BoardLayout {
    pub origin: Vec2,
    pub clip: Rect,
    pub cell: f32,
}
impl BoardLayout {
    pub fn index(&self, point: Vec2, cols: usize, rows: usize) -> Option<usize> {
        if !self.clip.contains(point) || point.x >= self.clip.max.x || point.y >= self.clip.max.y {
            return None;
        }
        let local = (point - self.origin) / self.cell;
        if local.x < 0.0 || local.y < 0.0 {
            return None;
        }
        let col = local.x.floor() as usize;
        let row = local.y.floor() as usize;
        (col < cols && row < rows).then_some(row * cols + col)
    }
}
#[derive(Resource, Default)]
pub struct ViewLayout {
    pub hits: Vec<Hit>,
    pub board: Option<BoardLayout>,
    pub cards: [Rect; 5],
    pub slots: [Rect; 3],
    pub card_order: Vec<usize>,
}

pub fn paint(
    mut frame: ResMut<Frame>,
    mut options: ResMut<RenderOptions>,
    mut state: ResMut<Arcade>,
    mut layout: ResMut<ViewLayout>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let metrics = platform::canvas_metrics();
    let size = metrics
        .map(|[w, h, _]| Vec2::new(w, h))
        .unwrap_or(Vec2::new(window.width(), window.height()));
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }
    frame.clear(size);
    layout.hits.clear();
    layout.board = None;
    layout.card_order.clear();
    options.scale_factor = metrics.map(|[_, _, d]| d).unwrap_or(window.scale_factor());
    options.pixel = state.save.settings.display_preset == DisplayPreset::PixelCrt;
    frame.min_stroke = if options.pixel { 2.0 } else { 0.0 };
    options.bloom = state.save.settings.bloom;
    options.scanlines = state.save.settings.scanlines;
    options.crt = state.save.settings.crt;
    frame.rect(Vec2::ZERO, size, BG);
    let margin = (size.x * 0.04).clamp(16.0, 52.0);
    let w = size.x - 2.0 * margin;
    // A quiet cabinet grid surrounds the flat play surfaces.
    for x in (0..size.x as usize).step_by(64) {
        frame.line(
            Vec2::new(x as f32, 0.0),
            Vec2::new(x as f32, size.y),
            0.5,
            Color::srgb(0.015, 0.045, 0.065),
        );
    }
    for y in (0..size.y as usize).step_by(64) {
        frame.line(
            Vec2::new(0.0, y as f32),
            Vec2::new(size.x, y as f32),
            0.5,
            Color::srgb(0.015, 0.045, 0.065),
        );
    }
    if state.modal.is_some() {
        dialog(&mut frame, &state, &mut layout, margin);
        platform::publish_board(None);
        return;
    }
    header(&mut frame, &state, &mut layout, margin);
    match state.screen {
        Screen::Lobby => lobby(&mut frame, &state, &mut layout, margin, w),
        Screen::Mines => mines(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Cards => cards(
            &mut frame,
            &mut state,
            &mut layout,
            margin,
            w,
            time.delta_secs(),
        ),
        Screen::Settings => settings(&mut frame, &state, &mut layout, margin, w),
    }
    let status = if state.protected_save || state.persistence.starts_with("Playing") {
        GOLD
    } else {
        MUTED
    };
    let footer = compact(&state.persistence, ((size.x - 2.0 * margin) / 7.5) as usize);
    frame.line(
        Vec2::new(margin, size.y - 33.0),
        Vec2::new(size.x - margin, size.y - 33.0),
        1.0,
        EDGE,
    );
    frame.text(Vec2::new(margin, size.y - 25.0), footer, 12.0, status);
    if let Some(board) = &layout.board {
        platform::publish_board(Some([
            board.clip.min.x,
            board.clip.min.y,
            board.clip.width(),
            board.clip.height(),
        ]));
    } else {
        platform::publish_board(None);
    }
}

fn header(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32) {
    f.text(Vec2::new(m, 24.0), "GAMEVEX", 25.0, CYAN);
    let width = f.size.x;
    if width > 720.0 {
        f.text(
            Vec2::new(m + 166.0, 32.0),
            "LOCAL ARCADE / RUST DEMO",
            12.0,
            MUTED,
        );
    }
    let mut x = width - m;
    if s.screen != Screen::Settings {
        x -= 100.0;
        button(
            f,
            s,
            l,
            rect(x, 24.0, 100.0, 34.0),
            "Settings",
            Action::Open(Screen::Settings),
            false,
        );
        x -= 12.0;
    }
    if s.screen != Screen::Lobby {
        x -= 84.0;
        button(
            f,
            s,
            l,
            rect(x, 24.0, 84.0, 34.0),
            "Lobby",
            Action::Open(Screen::Lobby),
            false,
        );
    }
    f.line(Vec2::new(m, 76.0), Vec2::new(width - m, 76.0), 1.0, CYAN);
    f.rect(Vec2::new(m, 74.0), Vec2::new(78.0, 4.0), GOLD);
}

fn lobby(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let wide = w > 800.0;
    let title_size = (w / 7.2).clamp(46.0, 118.0);
    f.text(Vec2::new(m - 3.0, 106.0), "GAMEVEX", title_size, INK);
    let y = 124.0 + title_size;
    f.line(Vec2::new(m, y), Vec2::new(m + w, y), 2.0, MAGENTA);
    f.text(
        Vec2::new(m, y + 17.0),
        "One more round.",
        if wide { 32.0 } else { 25.0 },
        CYAN,
    );
    if wide {
        f.text(
            Vec2::new(m + w - 330.0, y + 23.0),
            "No account. No network. Just play.",
            15.0,
            MUTED,
        );
    }
    let roster_y = y + 76.0;
    let row_h = if f.size.y < 720.0 { 82.0 } else { 105.0 };
    let a = rect(m, roster_y, w, row_h);
    cabinet(
        f,
        s,
        l,
        a,
        "MINESWEEPER",
        "Read the field. Trust the numbers.",
        "PLAY",
        CYAN,
        Action::Open(Screen::Mines),
    );
    let b = rect(m, roster_y + row_h + 16.0, w, row_h);
    cabinet(
        f,
        s,
        l,
        b,
        "CARD LAB",
        "Deal, flip, drag. An interaction sandbox.",
        "OPEN",
        MAGENTA,
        Action::Open(Screen::Cards),
    );
    let records_y = roster_y + row_h * 2.0 + 55.0;
    if records_y + 75.0 < f.size.y - 40.0 {
        f.text(
            Vec2::new(m, records_y),
            "Personal bests / Minesweeper",
            15.0,
            INK,
        );
        for (i, d) in Difficulty::ALL.into_iter().enumerate() {
            let x = m + i as f32 * w / 3.0;
            let name = difficulty_name(d);
            let best = s
                .save
                .records
                .best(d)
                .map(format_time)
                .unwrap_or("--".into());
            f.text(
                Vec2::new(x, records_y + 30.0),
                format!("{name}  {best}"),
                if wide { 17.0 } else { 12.0 },
                GOLD,
            );
        }
        if records_y + 115.0 < f.size.y - 40.0 {
            f.text(
                Vec2::new(m, records_y + 82.0),
                compact(&s.backend, 80),
                12.0,
                MUTED,
            );
            f.text(
                Vec2::new(m + w - 135.0, records_y + 82.0),
                format!("{:.0} fps / local", s.fps),
                12.0,
                MUTED,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cabinet(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    title: &str,
    desc: &str,
    verb: &str,
    color: Color,
    action: Action,
) {
    let index = l.hits.len();
    let hot = s.cursor.is_some_and(|p| r.contains(p)) || (s.keyboard && s.focus == index);
    f.rect(
        r.min,
        r.size(),
        if hot {
            Color::srgb(0.04, 0.1, 0.14)
        } else {
            PANEL
        },
    );
    f.outline(
        r.min,
        r.size(),
        if hot { 2.0 } else { 1.0 },
        if hot { color } else { EDGE },
    );
    f.rect(r.min, Vec2::new(5.0, r.height()), color);
    let small = r.width() < 600.0;
    f.text(
        r.min + Vec2::new(23.0, 17.0),
        title,
        if small { 22.0 } else { 30.0 },
        INK,
    );
    if !small {
        f.text(r.min + Vec2::new(24.0, 61.0), desc, 15.0, MUTED);
    } else {
        f.text(
            r.min + Vec2::new(24.0, 53.0),
            if title == "MINESWEEPER" {
                "A safe first move. A new field."
            } else {
                "Card interaction sandbox."
            },
            12.0,
            MUTED,
        );
    }
    f.text(
        Vec2::new(r.max.x - if small { 64.0 } else { 112.0 }, r.min.y + 26.0),
        verb,
        if small { 15.0 } else { 20.0 },
        color,
    );
    l.hits.push(Hit { rect: r, action });
}

fn mines(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let narrow = w < 700.0;
    f.text(
        Vec2::new(m, 100.0),
        "Minesweeper",
        if narrow { 31.0 } else { 42.0 },
        INK,
    );
    if !narrow {
        f.text(
            Vec2::new(m + w - 280.0, 119.0),
            "Clear the field. Leave no doubt.",
            14.0,
            MUTED,
        );
    }
    let y = 158.0;
    let difficulty = s.save.game.difficulty;
    for (i, d) in Difficulty::ALL.into_iter().enumerate() {
        button(
            f,
            s,
            l,
            rect(
                m + i as f32 * (w.min(540.0) / 3.0),
                y,
                w.min(540.0) / 3.0 - 8.0,
                34.0,
            ),
            difficulty_name(d),
            Action::New(d, false),
            difficulty == d,
        );
    }
    let y = 202.0;
    let button_width = ((w - 16.0) / 3.0).min(164.0);
    button(
        f,
        s,
        l,
        rect(m, y, button_width, 34.0),
        "Restart",
        Action::New(difficulty, true),
        false,
    );
    button(
        f,
        s,
        l,
        rect(m + button_width + 8.0, y, button_width, 34.0),
        "New field",
        Action::New(difficulty, false),
        false,
    );
    button(
        f,
        s,
        l,
        rect(m + 2.0 * (button_width + 8.0), y, button_width, 34.0),
        if s.flag_mode {
            if narrow { "Flag: ON" } else { "Flag mode: ON" }
        } else {
            if narrow {
                "Flag: off"
            } else {
                "Flag mode: off"
            }
        },
        Action::FlagMode,
        s.flag_mode,
    );
    let rail_y = 252.0;
    let status = match s.save.game.status {
        Status::Ready => "READY",
        Status::Playing => "PLAYING",
        Status::Won => "CLEARED",
        Status::Lost => "MINE HIT",
    };
    let status_color = match s.save.game.status {
        Status::Won => GREEN,
        Status::Lost => RED,
        _ => CYAN,
    };
    let values = [
        ("Field", status.to_string(), status_color),
        ("Time", format_time(s.save.game.elapsed_ms), INK),
        (
            "Mines left",
            format!("{:02}", s.save.game.remaining_mines()),
            GOLD,
        ),
        (
            "Best",
            s.save
                .records
                .best(difficulty)
                .map(format_time)
                .unwrap_or("--".into()),
            MAGENTA,
        ),
    ];
    for (i, (label, value, color)) in values.into_iter().enumerate() {
        let x = m + i as f32 * w / 4.0;
        f.text(Vec2::new(x, rail_y), label, 12.0, MUTED);
        f.text(
            Vec2::new(x, rail_y + 19.0),
            value,
            if narrow { 18.0 } else { 25.0 },
            color,
        );
    }
    let top = rail_y + 59.0;
    let bottom = (f.size.y - 107.0).max(top + 55.0);
    let clip = rect(m + 8.0, top + 8.0, w - 16.0, bottom - top - 16.0);
    f.rect(
        Vec2::new(m, top),
        Vec2::new(w, bottom - top),
        Color::srgb(0.01, 0.03, 0.047),
    );
    f.outline(Vec2::new(m, top), Vec2::new(w, bottom - top), 1.0, EDGE);
    let cell = if difficulty == Difficulty::Beginner && w > 500.0 {
        38.0
    } else {
        32.0
    };
    let board_size = Vec2::new(s.save.game.cols() as f32, s.save.game.rows() as f32) * cell;
    let max_pan = (board_size - clip.size()).max(Vec2::ZERO);
    s.pan = s.pan.clamp(Vec2::ZERO, max_pan);
    let origin = clip.min + (clip.size() - board_size).max(Vec2::ZERO) * 0.5 - s.pan;
    let focused = s.keyboard;
    let board_focus = s.focus == l.hits.len();
    let board_layout = BoardLayout { origin, clip, cell };
    for (i, c) in s.save.game.cells.iter().enumerate() {
        let p = origin
            + Vec2::new(
                (i % s.save.game.cols()) as f32,
                (i / s.save.game.cols()) as f32,
            ) * cell;
        let r = rect(p.x + 1.5, p.y + 1.5, cell - 3.0, cell - 3.0);
        let visible = r.intersect(clip);
        if visible.is_empty() {
            continue;
        }
        let hovered = s.cursor.is_some_and(|v| r.contains(v) && clip.contains(v));
        let fill = if s.save.game.exploded == Some(i) {
            Color::srgb(0.34, 0.025, 0.08)
        } else if c.revealed {
            Color::srgb(0.018, 0.045, 0.065)
        } else if hovered {
            Color::srgb(0.05, 0.15, 0.20)
        } else {
            Color::srgb(0.027, 0.08, 0.115)
        };
        f.rect(visible.min, visible.size(), fill);
        f.outline(
            visible.min,
            visible.size(),
            1.0,
            if c.revealed {
                Color::srgb(0.04, 0.12, 0.16)
            } else {
                EDGE
            },
        );
        if visible.width() < r.width() - 0.1 || visible.height() < r.height() - 0.1 {
            continue;
        }
        if c.revealed && c.mine {
            mine_symbol(
                f,
                r.center(),
                cell * 0.2,
                if s.save.game.exploded == Some(i) {
                    RED
                } else {
                    MAGENTA
                },
            );
        } else if c.flagged {
            flag(f, r.center(), cell * 0.28, GOLD);
        } else if c.revealed && c.adjacent > 0 {
            let color = match c.adjacent {
                1 => CYAN,
                2 => GREEN,
                3 => GOLD,
                4 => MAGENTA,
                _ => RED,
            };
            f.text(
                r.min + Vec2::new(cell * 0.25, cell * 0.10),
                c.adjacent.to_string(),
                cell * 0.64,
                color,
            );
        } else if !c.revealed {
            f.line(
                r.min + Vec2::splat(4.0),
                r.min + Vec2::new(9.0, 4.0),
                1.0,
                Color::srgb(0.12, 0.34, 0.41),
            );
        }
        if s.save.game.status == Status::Lost && c.flagged && !c.mine {
            f.line(r.min + Vec2::splat(6.0), r.max - Vec2::splat(6.0), 2.0, RED);
            f.line(
                r.min + Vec2::new(6.0, r.height() - 6.0),
                r.min + Vec2::new(r.width() - 6.0, 6.0),
                2.0,
                RED,
            );
        }
        if board_focus && s.cell == i && (s.keyboard || focused) {
            f.outline(
                r.min - Vec2::splat(1.0),
                r.size() + Vec2::splat(2.0),
                2.0,
                GOLD,
            );
        }
    }
    l.hits.push(Hit {
        rect: clip,
        action: Action::Board,
    });
    l.board = Some(board_layout);
    let hint_y = f.size.y - 96.0;
    if max_pan.x > 0.0 || max_pan.y > 0.0 {
        f.text(
            Vec2::new(m, hint_y),
            if narrow {
                "Scroll to pan / Shift+scroll sideways"
            } else {
                "Scroll to pan / Shift+scroll sideways / arrows follow focus"
            },
            12.0,
            GOLD,
        );
        if max_pan.x > 0.0 {
            let bar = w * (clip.width() / board_size.x).min(1.0);
            let x = m + (w - bar) * s.pan.x / max_pan.x;
            f.rect(Vec2::new(x, bottom - 3.0), Vec2::new(bar, 3.0), CYAN);
        }
    } else {
        f.text(
            Vec2::new(m, hint_y),
            compact(&s.message, (w / 7.2) as usize),
            12.0,
            if s.save.game.status == Status::Lost {
                RED
            } else {
                MUTED
            },
        );
    }
    f.text(
        Vec2::new(m, f.size.y - 72.0),
        if narrow {
            "Arrows: move / Space: reveal / F: flag"
        } else {
            "Arrows: move   Space: reveal   F: flag   Enter: chord   Tab: controls"
        },
        12.0,
        INK,
    );
    f.text(
        Vec2::new(m, f.size.y - 53.0),
        if narrow {
            "Enter: chord / Tab: controls / Esc: lobby"
        } else {
            "Right click: flag   Click an open number: chord   Esc: lobby   Timer pauses away"
        },
        12.0,
        MUTED,
    );
}

fn cards(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32, dt: f32) {
    let small = w < 620.0;
    f.text(
        Vec2::new(m, 102.0),
        "Card Lab",
        if small { 34.0 } else { 44.0 },
        INK,
    );
    f.text(
        Vec2::new(m, 154.0),
        "An interaction sandbox. No stakes, just a hand of cards.",
        if small { 11.0 } else { 15.0 },
        MUTED,
    );
    let controls_y = 190.0;
    button(
        f,
        s,
        l,
        rect(m, controls_y, 110.0, 34.0),
        "Deal again",
        Action::Deal,
        false,
    );
    button(
        f,
        s,
        l,
        rect(m + 118.0, controls_y, 90.0, 34.0),
        "Flip / F",
        Action::Flip,
        false,
    );
    button(
        f,
        s,
        l,
        rect(
            m + 216.0,
            controls_y,
            if small { 100.0 } else { 132.0 },
            34.0,
        ),
        "Return card",
        Action::ReturnCard,
        false,
    );
    let card_h =
        (((w - 36.0) / 3.0).clamp(82.0, 132.0) * 1.4).min(((f.size.y - 400.0) / 2.0).max(80.0));
    let card_w = card_h / 1.4;
    let slots_y = 256.0;
    let gap = if small { 12.0 } else { 34.0 };
    let start = m + (w - (card_w * 3.0 + gap * 2.0)) * 0.5;
    for i in 0..3 {
        let r = rect(start + i as f32 * (card_w + gap), slots_y, card_w, card_h);
        l.slots[i] = r;
        let hovered = s.lab.drag.is_some() && s.cursor.is_some_and(|p| r.contains(p));
        f.rect(r.min, r.size(), PANEL);
        f.outline(
            r.min,
            r.size(),
            if hovered { 2.0 } else { 1.0 },
            if hovered { GOLD } else { EDGE },
        );
        if s.lab.slots[i].is_none() {
            f.text(
                r.min + Vec2::new(card_w * 0.35, card_h * 0.38),
                format!("{}", i + 1),
                28.0,
                EDGE,
            );
        }
        f.text(
            r.min - Vec2::new(0.0, 23.0),
            format!("Slot {}", i + 1),
            12.0,
            MUTED,
        );
    }
    let hand_y = (f.size.y - card_h - 105.0).max(slots_y + card_h + 36.0);
    let spread = (card_w * 0.65).min((w - card_w) / 4.0);
    let hand_x = m + (w - (card_w + spread * 4.0)) * 0.5;
    let hovered = (0..5).rev().find(|&i| {
        s.lab.slot_of(i).is_none()
            && s.cursor.is_some_and(|p| {
                rect(s.card_positions[i].x, s.card_positions[i].y, card_w, card_h).contains(p)
            })
    });
    let mut order: Vec<usize> = (0..5).collect();
    if let Some(i) = s.lab.selected {
        order.retain(|&v| v != i);
        order.push(i);
    }
    if let Some(i) = s.lab.drag {
        order.retain(|&v| v != i);
        order.push(i);
    }
    for i in order.iter().copied() {
        let mut target = if let Some(slot) = s.lab.slot_of(i) {
            l.slots[slot].min
        } else {
            Vec2::new(hand_x + i as f32 * spread, hand_y)
        };
        if s.lab.slot_of(i).is_none() && (s.lab.selected == Some(i) || hovered == Some(i)) {
            target.y -= 18.0;
        }
        if s.lab.drag == Some(i)
            && let Some(p) = s.cursor
        {
            target = p - s.drag_offset;
        }
        let reduced = s.save.settings.reduced_motion;
        if s.card_positions[i].x < -900.0 {
            s.card_positions[i] = if reduced {
                target
            } else {
                Vec2::new(hand_x - 80.0, hand_y + 70.0)
            };
        }
        let ready = reduced || s.motion_time > i as f32 * 0.045;
        if ready {
            let t = if reduced || s.lab.drag == Some(i) {
                1.0
            } else {
                1.0 - (-dt.min(0.1) * 22.0).exp()
            };
            let current = s.card_positions[i];
            s.card_positions[i] = current.lerp(target, t);
        }
        let r = rect(s.card_positions[i].x, s.card_positions[i].y, card_w, card_h);
        l.cards[i] = r;
    }
    for (stack, &i) in order.iter().enumerate() {
        let rank_center = l.cards[i].min + Vec2::new(24.0, 24.0);
        let rank_visible = !order[stack + 1..]
            .iter()
            .any(|&later| l.cards[later].contains(rank_center));
        card_face(
            f,
            l.cards[i],
            s.lab.cards[i],
            s.lab.selected == Some(i) || hovered == Some(i),
            rank_visible,
        );
    }
    l.card_order = order;
    f.text(
        Vec2::new(m, f.size.y - 79.0),
        if small {
            "Drag to slots / invalid drops snap back"
        } else {
            "Drag to a slot. Invalid drops snap back. Click a card to select it."
        },
        12.0,
        INK,
    );
    f.text(
        Vec2::new(m, f.size.y - 57.0),
        if small {
            "Left/Right select / F flip / 1 / 2 / 3 place"
        } else {
            "Left/Right select   F flip   1 / 2 / 3 place in slot   Backspace returns card   Esc lobby"
        },
        12.0,
        MUTED,
    );
}

fn card_face(
    f: &mut Frame,
    r: Rect,
    card: gamevex_core::cards::Card,
    selected: bool,
    rank_visible: bool,
) {
    f.rect(
        r.min + Vec2::new(4.0, 7.0),
        r.size(),
        Color::srgb(0.0, 0.01, 0.025),
    );
    f.rect(r.min, r.size(), Color::srgb(0.025, 0.055, 0.095));
    f.outline(
        r.min,
        r.size(),
        if selected { 2.5 } else { 1.5 },
        if selected { GOLD } else { CYAN },
    );
    let inset = r.min + Vec2::splat(7.0);
    f.outline(inset, r.size() - Vec2::splat(14.0), 0.75, EDGE);
    if card.face_up {
        let color = if card.suit.is_red() { MAGENTA } else { CYAN };
        // Rank stays on the exposed left edge of an overlapping hand.
        if rank_visible {
            f.text(
                r.min + Vec2::new(12.0, 11.0),
                card.rank_label(),
                22.0,
                color,
            );
        }
        suit(f, r.min + Vec2::new(20.0, 49.0), 8.0, card.suit, color);
        // The large central suit uses geometry, so later cards cover it naturally.
        suit(
            f,
            r.center() + Vec2::new(5.0, 10.0),
            r.width() * 0.22,
            card.suit,
            color,
        );
    } else {
        for i in 0..5 {
            let d = 8.0 + i as f32 * 8.0;
            let p = r.min + Vec2::new(d, d);
            let size = r.size() - Vec2::splat(d * 2.0);
            if size.min_element() > 0.0 {
                f.outline(p, size, 1.0, if i % 2 == 0 { MAGENTA } else { EDGE });
            }
        }
    }
}

fn settings(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    f.text(
        Vec2::new(m, 102.0),
        "Tune the cabinet",
        if w < 600.0 { 29.0 } else { 42.0 },
        INK,
    );
    f.text(
        Vec2::new(m, 154.0),
        "Settings save automatically on this device.",
        13.0,
        MUTED,
    );
    let row_h = if f.size.y < 700.0 { 42.0 } else { 53.0 };
    let label_width = if w < 600.0 { w * 0.43 } else { 230.0 };
    let right = m + label_width;
    let usable = (w - label_width).min(440.0);
    let labels = [
        "Display",
        "Bloom",
        "Scanlines",
        "CRT effects",
        "Reduced motion",
        "Volume",
        "Sound",
    ];
    for (i, label) in labels.iter().enumerate() {
        let y = 189.0 + i as f32 * row_h;
        f.text(
            Vec2::new(m, y + 9.0),
            *label,
            if w < 400.0 { 13.0 } else { 16.0 },
            INK,
        );
        f.line(
            Vec2::new(m, y + row_h - 5.0),
            Vec2::new(m + w, y + row_h - 5.0),
            0.75,
            EDGE,
        );
        match i {
            0 => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.display_preset == DisplayPreset::VectorCrt {
                    "Vector CRT"
                } else {
                    "Pixel CRT"
                },
                Action::Preset,
                true,
            ),
            1 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.bloom,
                Action::Bloom(-0.1),
                Action::Bloom(0.1),
            ),
            2 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.scanlines,
                Action::Scanlines(-0.1),
                Action::Scanlines(0.1),
            ),
            3 => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.crt {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Action::Crt,
                s.save.settings.crt,
            ),
            4 => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.reduced_motion {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Action::Motion,
                s.save.settings.reduced_motion,
            ),
            5 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.volume,
                Action::Volume(-0.1),
                Action::Volume(0.1),
            ),
            _ => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.muted { "Muted" } else { "On" },
                Action::Mute,
                !s.save.settings.muted,
            ),
        }
    }
    let y = 189.0 + 7.0 * row_h + 10.0;
    button(f, s, l, rect(m, y, 94.0, 34.0), "Back", Action::Back, false);
    button(
        f,
        s,
        l,
        rect(m + 102.0, y, 110.0, 34.0),
        "Retry save",
        Action::Retry,
        false,
    );
    if s.protected_save {
        button(
            f,
            s,
            l,
            rect(m, y + 44.0, w.min(340.0), 34.0),
            "Reset damaged save",
            Action::Recover,
            true,
        );
    } else if y + 70.0 < f.size.y - 40.0 {
        f.text(
            Vec2::new(m, y + 53.0),
            compact(&s.backend, (w / 7.4) as usize),
            12.0,
            MUTED,
        );
        if y + 100.0 < f.size.y - 40.0 {
            f.text(
                Vec2::new(m, y + 76.0),
                compact(&platform::description(), (w / 7.4) as usize),
                12.0,
                MUTED,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stepper(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    x: f32,
    y: f32,
    w: f32,
    value: f32,
    down: Action,
    up: Action,
) {
    button(f, s, l, rect(x, y, 34.0, 31.0), "-", down, false);
    f.text(
        Vec2::new(x + w * 0.5 - 20.0, y + 6.0),
        format!("{:.0}%", value * 100.0),
        15.0,
        CYAN,
    );
    button(f, s, l, rect(x + w - 34.0, y, 34.0, 31.0), "+", up, false);
}

fn dialog(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32) {
    let width = (f.size.x - 2.0 * m).min(590.0);
    let r = rect(
        (f.size.x - width) * 0.5,
        (f.size.y - 254.0) * 0.5,
        width,
        254.0,
    );
    f.outline(
        r.min - Vec2::splat(6.0),
        r.size() + Vec2::splat(12.0),
        1.0,
        EDGE,
    );
    f.rect(r.min, r.size(), PANEL);
    f.rounded_outline(r.min, r.size(), 8.0, 2.0, GOLD);
    let recover = matches!(s.modal, Some(Pending::Recover));
    f.text(
        r.min + Vec2::new(22.0, 24.0),
        if recover {
            "Reset the damaged save?"
        } else {
            "Replace this run?"
        },
        if width < 400.0 { 23.0 } else { 29.0 },
        INK,
    );
    f.text(
        r.min + Vec2::new(22.0, 83.0),
        if recover {
            "Saved settings, records, and progress\nwill be replaced with a fresh session."
        } else {
            "Your unfinished field will be replaced.\nCancel to keep playing this run."
        },
        if width < 400.0 { 12.0 } else { 16.0 },
        MUTED,
    );
    let bw = (width - 58.0) * 0.5;
    button(
        f,
        s,
        l,
        rect(r.min.x + 22.0, r.max.y - 65.0, bw, 40.0),
        "Cancel",
        Action::Cancel,
        false,
    );
    button(
        f,
        s,
        l,
        rect(r.min.x + 36.0 + bw, r.max.y - 65.0, bw, 40.0),
        if recover { "Reset save" } else { "Replace run" },
        Action::Confirm,
        true,
    );
}

fn button(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    label: &str,
    action: Action,
    active: bool,
) {
    let idx = l.hits.len();
    let focused = s.keyboard && s.focus == idx;
    let hovered = s.cursor.is_some_and(|p| r.contains(p));
    f.rect(
        r.min,
        r.size(),
        if hovered || focused {
            Color::srgb(0.055, 0.14, 0.18)
        } else {
            PANEL
        },
    );
    let color = if focused {
        GOLD
    } else if active {
        CYAN
    } else if hovered {
        INK
    } else {
        EDGE
    };
    f.outline(r.min, r.size(), if focused { 2.0 } else { 1.0 }, color);
    if active {
        f.rect(r.min, Vec2::new(3.0, r.height()), CYAN);
    }
    let font = if r.width() < 150.0 { 13.0 } else { 14.0 };
    let tx = r.min.x + (r.width() - label.chars().count() as f32 * font * 0.6) * 0.5;
    f.text(
        Vec2::new(
            tx.max(r.min.x + 5.0),
            r.min.y + (r.height() - font * 1.2) * 0.5,
        ),
        label,
        font,
        if active || focused {
            INK
        } else {
            Color::srgb(0.6, 0.78, 0.85)
        },
    );
    l.hits.push(Hit { rect: r, action });
}

fn mine_symbol(f: &mut Frame, c: Vec2, r: f32, color: Color) {
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let d = Vec2::new(a.cos(), a.sin());
        f.line(c + d * r * 0.42, c + d * r * 1.25, 1.8, color);
    }
    circle(f, c, r * 0.60, 1.5, color);
    f.rect(c - Vec2::splat(2.0), Vec2::splat(4.0), INK);
}
fn flag(f: &mut Frame, c: Vec2, r: f32, color: Color) {
    let a = c + Vec2::new(-r * 0.45, -r);
    f.line(a, c + Vec2::new(-r * 0.45, r), 1.7, color);
    polyline(
        f,
        &[
            a,
            c + Vec2::new(r * 0.6, -r * 0.4),
            c + Vec2::new(-r * 0.45, 0.1 * r),
        ],
        1.7,
        color,
    );
    f.line(
        c + Vec2::new(-r * 0.85, r),
        c + Vec2::new(r * 0.35, r),
        1.7,
        color,
    );
}
fn circle(f: &mut Frame, c: Vec2, r: f32, width: f32, color: Color) {
    let points = (0..=24)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 24.0;
            c + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect::<Vec<_>>();
    polyline(f, &points, width, color);
}
fn polyline(f: &mut Frame, p: &[Vec2], width: f32, color: Color) {
    for points in p.windows(2) {
        f.line(points[0], points[1], width, color);
    }
}
fn suit(f: &mut Frame, c: Vec2, r: f32, suit: Suit, color: Color) {
    match suit {
        Suit::Diamonds => polyline(
            f,
            &[
                c + Vec2::new(0.0, -r),
                c + Vec2::new(r * 0.65, 0.0),
                c + Vec2::new(0.0, r),
                c + Vec2::new(-r * 0.65, 0.0),
                c + Vec2::new(0.0, -r),
            ],
            1.8,
            color,
        ),
        Suit::Clubs => {
            for d in [
                Vec2::new(0.0, -0.4),
                Vec2::new(-0.42, 0.12),
                Vec2::new(0.42, 0.12),
            ] {
                circle(f, c + d * r, r * 0.4, 1.8, color);
            }
            f.line(c, c + Vec2::new(0.0, r), 2.0, color);
            f.line(
                c + Vec2::new(-r * 0.35, r),
                c + Vec2::new(r * 0.35, r),
                1.8,
                color,
            );
        }
        Suit::Hearts | Suit::Spades => {
            let sign = if suit == Suit::Spades { -1.0 } else { 1.0 };
            let points = (0..=40)
                .map(|i| {
                    let t = i as f32 * std::f32::consts::TAU / 40.0;
                    let x = 16.0 * t.sin().powi(3) / 18.0;
                    let y = -(13.0 * t.cos()
                        - 5.0 * (2.0 * t).cos()
                        - 2.0 * (3.0 * t).cos()
                        - (4.0 * t).cos())
                        / 18.0;
                    c + Vec2::new(x, y * sign) * r
                })
                .collect::<Vec<_>>();
            polyline(f, &points, 1.8, color);
            if suit == Suit::Spades {
                f.line(c, c + Vec2::new(0.0, r), 2.0, color);
                f.line(
                    c + Vec2::new(-r * 0.35, r),
                    c + Vec2::new(r * 0.35, r),
                    1.8,
                    color,
                );
            }
        }
    }
}
fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_corners(Vec2::new(x, y), Vec2::new(x + w, y + h))
}
pub fn difficulty_name(d: Difficulty) -> &'static str {
    match d {
        Difficulty::Beginner => "Beginner",
        Difficulty::Intermediate => "Intermediate",
        Difficulty::Expert => "Expert",
    }
}
pub fn format_time(ms: u64) -> String {
    let seconds = ms / 1000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
fn compact(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}...",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn board_picking_respects_pan_and_clipped_edges() {
        let b = BoardLayout {
            origin: Vec2::new(-12.0, 50.0),
            clip: rect(20.0, 50.0, 200.0, 200.0),
            cell: 32.0,
        };
        assert_eq!(b.index(Vec2::new(20.0, 51.0), 30, 16), Some(1));
        assert_eq!(b.index(Vec2::new(19.0, 51.0), 30, 16), None);
        assert_eq!(b.index(Vec2::new(219.0, 81.0), 30, 16), Some(7));
        assert_eq!(b.index(Vec2::new(220.0, 81.0), 30, 16), None);
    }
}
