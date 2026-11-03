use super::*;

mod cards;
mod dialog;
mod lobby;
mod mines;
mod settings;

pub(super) use cards::{card_face, card_face_with_emphasis, card_hit_bounds, cards, rects_overlap};
pub(super) use dialog::dialog;
pub(crate) use lobby::LobbyCategory;
pub(super) use lobby::lobby;
pub(super) use mines::mines;
pub(super) use settings::settings;

pub(crate) fn button(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    label: &str,
    action: Action,
    active: bool,
) {
    let t = f.theme();
    let idx = l.hits.len();
    let focused = s.keyboard && s.focus == idx;
    let hovered = s.cursor.is_some_and(|p| r.contains(p));
    let fill = if hovered || focused {
        t.panel_hover
    } else {
        t.panel
    };
    f.rect(r.min, r.size(), fill);
    let color = if focused {
        t.gold
    } else if active {
        t.accent
    } else if hovered {
        t.ink
    } else {
        t.edge
    };
    f.outline(r.min, r.size(), if focused { 2.0 } else { 1.0 }, color);
    if active {
        f.rect(r.min, Vec2::new(3.0, r.height()), t.accent);
    }
    let font = if r.width() < 150.0 { 13.0 } else { 14.0 };
    f.centered_text(
        Vec2::new(r.center().x, r.min.y + (r.height() - font * 1.2) * 0.5),
        label,
        font,
        if active || focused {
            t.ink
        } else {
            t.control_text
        },
    );
    l.hits.push(Hit { rect: r, action });
}

pub(crate) fn mine_symbol(f: &mut Frame, c: Vec2, r: f32, color: Color) {
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let d = Vec2::new(a.cos(), a.sin());
        f.line(c + d * r * 0.42, c + d * r * 1.25, 1.8, color);
    }
    circle(f, c, r * 0.60, 1.5, color);
    f.rect(c - Vec2::splat(2.0), Vec2::splat(4.0), f.theme().ink);
}
pub(crate) fn flag(f: &mut Frame, c: Vec2, r: f32, color: Color) {
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
pub(crate) fn circle(f: &mut Frame, c: Vec2, r: f32, width: f32, color: Color) {
    let points = (0..=24)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 24.0;
            c + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect::<Vec<_>>();
    polyline(f, &points, width, color);
}
pub(crate) fn polyline(f: &mut Frame, p: &[Vec2], width: f32, color: Color) {
    for points in p.windows(2) {
        f.line(points[0], points[1], width, color);
    }
}
pub(crate) fn suit(f: &mut Frame, c: Vec2, r: f32, suit: Suit, color: Color) {
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

pub(crate) fn difficulty_label(level: u8) -> &'static str {
    match level {
        1 => "Easy",
        2 => "Casual",
        3 => "Normal",
        4 => "Hard",
        5 => "Expert",
        _ => "Normal",
    }
}

pub(crate) fn difficulty_buttons(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    current: u8,
    action: Action,
) {
    let width = r.width() / 5.0;
    for level in 1..=5 {
        let select = match action {
            Action::Connect4Difficulty(_) => Action::Connect4Difficulty(level),
            Action::TicTacToeDifficulty(_) => Action::TicTacToeDifficulty(level),
            _ => return,
        };
        button(
            f,
            s,
            l,
            rect(
                r.min.x + (level - 1) as f32 * width,
                r.min.y,
                width - 8.0,
                34.0,
            ),
            difficulty_label(level),
            select,
            current == level,
        );
    }
}
