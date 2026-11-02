use super::*;

pub(crate) fn lobby(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let wide = w > 800.0;
    let title_size = if f.size.y < 640.0 {
        (w / 13.0).clamp(28.0, 50.0)
    } else if f.size.y < 720.0 {
        (w / 10.0).clamp(34.0, 70.0)
    } else {
        (w / 12.0).clamp(48.0, 96.0)
    };
    f.text(Vec2::new(m - 3.0, 106.0), "GAMEVEX", title_size, INK);
    let y = 124.0 + title_size;
    f.line(Vec2::new(m, y), Vec2::new(m + w, y), 2.0, MAGENTA);
    f.text(
        Vec2::new(m, y + 17.0),
        "One more round.",
        if f.size.y < 640.0 {
            24.0
        } else if wide {
            32.0
        } else {
            25.0
        },
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

    // Keep the page chrome fixed. The cabinet entries form one scrollable roster.
    let compact_layout = f.size.y < 640.0;
    let roster_top = y + if compact_layout { 50.0 } else { 76.0 };
    let roster_bottom = (f.size.y - 40.0).max(roster_top + 1.0);
    let clip = rect(m, roster_top, w, roster_bottom - roster_top);
    const ROW_H: f32 = 76.0;
    const GAP: f32 = 10.0;
    const COUNT: usize = 12;
    let content_h = COUNT as f32 * ROW_H + (COUNT - 1) as f32 * GAP;
    l.lobby_clip = Some(clip);
    l.lobby_max_scroll = (content_h - clip.height()).max(0.0);
    let roster_hit_base = l.hits.len();
    if s.keyboard
        && l.lobby_max_scroll > 0.0
        && let Some(row) = s.focus.checked_sub(roster_hit_base).filter(|i| *i < COUNT)
    {
        let row_top = row as f32 * (ROW_H + GAP);
        if row_top < s.lobby_scroll {
            s.lobby_scroll = row_top;
        } else if row_top + ROW_H > s.lobby_scroll + clip.height() {
            s.lobby_scroll = row_top + ROW_H - clip.height();
        }
    }
    s.lobby_scroll = s.lobby_scroll.clamp(0.0, l.lobby_max_scroll);
    let scroll = s.lobby_scroll;
    f.rect(clip.min, clip.size(), BG);

    let entries = [
        (
            "MINESWEEPER",
            "Read the field. Trust the numbers.",
            "PLAY",
            CYAN,
            Screen::Mines,
        ),
        (
            "CARD LAB",
            "Deal, flip, drag. An interaction sandbox.",
            "OPEN",
            MAGENTA,
            Screen::Cards,
        ),
        (
            "BLACKJACK",
            "Five hands. One shoe. Your call.",
            "DEAL",
            GOLD,
            Screen::Blackjack,
        ),
        (
            "BACCARAT",
            "Player. Banker. Closest to nine.",
            "DEAL",
            GREEN,
            Screen::Baccarat,
        ),
        (
            "SUDOKU",
            "Every row. Every column. Every box.",
            "PLAY",
            CYAN,
            Screen::Sudoku,
        ),
        (
            "MAHJONG",
            "Match free tiles. Clear the stack.",
            "PLAY",
            GREEN,
            Screen::Mahjong,
        ),
        (
            "SOLITAIRE",
            "Build down in alternating colors.",
            "PLAY",
            GOLD,
            Screen::Solitaire,
        ),
        (
            "FREECELL",
            "Four free cells. Plan every move.",
            "PLAY",
            CYAN,
            Screen::FreeCell,
        ),
        (
            "SPIDER",
            "Build suited sequences. Clear the tableau.",
            "PLAY",
            MAGENTA,
            Screen::Spider,
        ),
        (
            "HEARTS",
            "Pass the danger. Keep your score low.",
            "PLAY",
            RED,
            Screen::Hearts,
        ),
        (
            "CRIBBAGE",
            "Make fifteen. Race to 121.",
            "PLAY",
            CYAN,
            Screen::Cribbage,
        ),
        (
            "TEXAS HOLD'EM",
            "Read the table. Make your move.",
            "DEAL",
            GOLD,
            Screen::Holdem,
        ),
    ];
    for (i, (title, desc, verb, color, screen)) in entries.into_iter().enumerate() {
        let row = rect(m, clip.min.y + i as f32 * (ROW_H + GAP) - scroll, w, ROW_H);
        cabinet(
            f,
            s,
            l,
            row,
            clip,
            title,
            desc,
            verb,
            color,
            Action::Open(screen),
        );
    }
    if l.lobby_max_scroll > 0.0 {
        let track = rect(
            clip.max.x - 4.0,
            clip.min.y + 2.0,
            3.0,
            (clip.height() - 4.0).max(1.0),
        );
        f.rect(track.min, track.size(), EDGE);
        let thumb_h = (track.height() * clip.height() / content_h)
            .max(22.0)
            .min(track.height());
        let thumb_y = track.min.y + (track.height() - thumb_h) * (scroll / l.lobby_max_scroll);
        f.rect(
            Vec2::new(track.min.x, thumb_y),
            Vec2::new(track.width(), thumb_h),
            CYAN,
        );
    }
}

fn intersect(a: Rect, b: Rect) -> Rect {
    Rect {
        min: a.min.max(b.min),
        max: a.max.min(b.max),
    }
}

fn offscreen_hit() -> Rect {
    let p = Vec2::splat(-10_000.0);
    Rect { min: p, max: p }
}

#[allow(clippy::too_many_arguments)]
fn cabinet(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    clip: Rect,
    title: &str,
    desc: &str,
    verb: &str,
    color: Color,
    action: Action,
) {
    let visible = intersect(r, clip);
    let index = l.hits.len();
    let hot = s.cursor.is_some_and(|p| r.contains(p) && clip.contains(p))
        || (s.keyboard && s.focus == index);
    f.rect(
        visible.min,
        visible.size(),
        if hot {
            Color::srgb(0.04, 0.1, 0.14)
        } else {
            PANEL
        },
    );
    // Draw only clipped border segments so scrolling rows cannot paint into the page chrome.
    let edge = if hot { color } else { EDGE };
    clipped_line(
        f,
        Vec2::new(r.min.x, r.min.y),
        Vec2::new(r.max.x, r.min.y),
        clip,
        edge,
    );
    clipped_line(
        f,
        Vec2::new(r.min.x, r.max.y),
        Vec2::new(r.max.x, r.max.y),
        clip,
        edge,
    );
    clipped_line(
        f,
        Vec2::new(r.min.x, r.min.y),
        Vec2::new(r.min.x, r.max.y),
        clip,
        edge,
    );
    clipped_line(
        f,
        Vec2::new(r.max.x, r.min.y),
        Vec2::new(r.max.x, r.max.y),
        clip,
        edge,
    );
    let accent = intersect(rect(r.min.x, r.min.y, 5.0, r.height()), clip);
    f.rect(accent.min, accent.size(), color);
    let small = r.width() < 600.0;
    let title_size = if small { 22.0 } else { 27.0 };
    let desc_size = if small { 12.0 } else { 14.0 };
    let verb_size = if small { 15.0 } else { 20.0 };
    let verb_x = r.max.x - if small { 64.0 } else { 112.0 };
    let title_pos = r.min + Vec2::new(23.0, 11.0);
    let desc_pos = r.min + Vec2::new(24.0, 48.0);
    let verb_pos = Vec2::new(verb_x, r.min.y + 26.0);
    if title_pos.y >= clip.min.y && title_pos.y + title_size <= clip.max.y {
        f.text(title_pos, title, title_size, INK);
    }
    if desc_pos.y >= clip.min.y && desc_pos.y + desc_size <= clip.max.y {
        let available = (verb_x - desc_pos.x - 8.0).max(0.0);
        let desc = compact(desc, (available / (desc_size * 0.58)).floor() as usize);
        f.text(desc_pos, desc, desc_size, MUTED);
    }
    if verb_pos.y >= clip.min.y && verb_pos.y + verb_size <= clip.max.y {
        f.text(verb_pos, verb, verb_size, color);
    }
    let hit = intersect(r, clip);
    let hit = if hit.width() > 0.0 && hit.height() > 0.0 {
        hit
    } else {
        offscreen_hit()
    };
    l.hits.push(Hit { rect: hit, action });
}

fn clipped_line(f: &mut Frame, a: Vec2, b: Vec2, clip: Rect, color: Color) {
    let d = b - a;
    let mut lo: f32 = 0.0;
    let mut hi: f32 = 1.0;
    for (p, q) in [
        (-d.x, a.x - clip.min.x),
        (d.x, clip.max.x - a.x),
        (-d.y, a.y - clip.min.y),
        (d.y, clip.max.y - a.y),
    ] {
        if p.abs() <= f32::EPSILON {
            if q < 0.0 {
                return;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                lo = lo.max(t);
            } else {
                hi = hi.min(t);
            }
            if lo > hi {
                return;
            }
        }
    }
    f.line(a + d * lo, a + d * hi, 1.0, color);
}
