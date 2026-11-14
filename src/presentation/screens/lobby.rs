use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LobbyCategory {
    Puzzle,
    HighLimit,
    Solitaire,
    CardAndBoard,
}

impl LobbyCategory {
    pub(crate) const ALL: [Self; 4] = [
        Self::Puzzle,
        Self::HighLimit,
        Self::Solitaire,
        Self::CardAndBoard,
    ];

    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Puzzle => "puzzle",
            Self::HighLimit => "highLimit",
            Self::Solitaire => "solitaire",
            Self::CardAndBoard => "cardAndBoard",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Puzzle => "Puzzle",
            Self::HighLimit => "High Limit",
            Self::Solitaire => "Solitaire",
            Self::CardAndBoard => "Card & Board",
        }
    }

    pub(crate) fn accent_for(self, theme: &crate::theme::Theme) -> Color {
        match self {
            Self::Puzzle => theme.accent,
            Self::HighLimit => theme.gold,
            Self::Solitaire => theme.secondary,
            Self::CardAndBoard => theme.success,
        }
    }

    fn includes(self, screen: Screen) -> bool {
        match self {
            Self::Puzzle => matches!(screen, Screen::Mines | Screen::Sudoku | Screen::Mahjong),
            Self::HighLimit => matches!(
                screen,
                Screen::Blackjack | Screen::Baccarat | Screen::Holdem | Screen::Roulette
            ),
            Self::Solitaire => matches!(
                screen,
                Screen::Solitaire | Screen::FreeCell | Screen::Spider
            ),
            Self::CardAndBoard => matches!(
                screen,
                Screen::Hearts
                    | Screen::Cribbage
                    | Screen::Backgammon
                    | Screen::Connect4
                    | Screen::TicTacToe
                    | Screen::Cards
            ),
        }
    }
}

#[derive(Clone, Copy)]
enum EntryRole {
    Accent,
    Secondary,
    Gold,
    Success,
    Danger,
}

impl EntryRole {
    fn color(self, theme: &crate::theme::Theme) -> Color {
        match self {
            Self::Accent => theme.accent,
            Self::Secondary => theme.secondary,
            Self::Gold => theme.gold,
            Self::Success => theme.success,
            Self::Danger => theme.danger,
        }
    }
}

const ENTRIES: [(&str, &str, &str, EntryRole, Screen); 16] = [
    (
        "MINESWEEPER",
        "Read the field. Trust the numbers.",
        "PLAY",
        EntryRole::Accent,
        Screen::Mines,
    ),
    (
        "CARD LAB",
        "Deal, flip, drag. An interaction sandbox.",
        "OPEN",
        EntryRole::Secondary,
        Screen::Cards,
    ),
    (
        "BLACKJACK",
        "Five hands. One shoe. Your call.",
        "DEAL",
        EntryRole::Gold,
        Screen::Blackjack,
    ),
    (
        "BACCARAT",
        "Player. Banker. Closest to nine.",
        "DEAL",
        EntryRole::Success,
        Screen::Baccarat,
    ),
    (
        "SUDOKU",
        "Every row. Every column. Every box.",
        "PLAY",
        EntryRole::Accent,
        Screen::Sudoku,
    ),
    (
        "MAHJONG",
        "Match free tiles. Clear the stack.",
        "PLAY",
        EntryRole::Success,
        Screen::Mahjong,
    ),
    (
        "SOLITAIRE",
        "Build down in alternating colors.",
        "PLAY",
        EntryRole::Gold,
        Screen::Solitaire,
    ),
    (
        "FREECELL",
        "Four free cells. Plan every move.",
        "PLAY",
        EntryRole::Accent,
        Screen::FreeCell,
    ),
    (
        "SPIDER",
        "Build suited sequences. Clear the tableau.",
        "PLAY",
        EntryRole::Secondary,
        Screen::Spider,
    ),
    (
        "HEARTS",
        "Pass the danger. Keep your score low.",
        "PLAY",
        EntryRole::Danger,
        Screen::Hearts,
    ),
    (
        "CRIBBAGE",
        "Make fifteen. Race to 121.",
        "PLAY",
        EntryRole::Accent,
        Screen::Cribbage,
    ),
    (
        "TEXAS HOLD'EM",
        "Read the table. Make your move.",
        "DEAL",
        EntryRole::Gold,
        Screen::Holdem,
    ),
    (
        "ROULETTE",
        "One wheel. Pick your pocket.",
        "SPIN",
        EntryRole::Accent,
        Screen::Roulette,
    ),
    (
        "TIC-TAC-TOE",
        "Three in a row. Make your move.",
        "PLAY",
        EntryRole::Accent,
        Screen::TicTacToe,
    ),
    (
        "CONNECT 4",
        "Drop your discs. Connect four to win.",
        "PLAY",
        EntryRole::Danger,
        Screen::Connect4,
    ),
    (
        "BACKGAMMON",
        "Roll the dice. Race your checkers home.",
        "PLAY",
        EntryRole::Secondary,
        Screen::Backgammon,
    ),
];

pub(crate) fn lobby(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    if s.lobby_category.is_none() {
        category_grid(f, s, l, m, w);
        if platform::install_available() {
            super::button(
                f,
                s,
                l,
                rect(m, f.size.y - 40.0, 140.0, 32.0),
                "Install game",
                Action::Install,
                false,
            );
        }
        return;
    }
    let t = f.theme();
    let category = s.lobby_category.unwrap();
    let entries: Vec<_> = ENTRIES
        .into_iter()
        .filter(|entry| category.includes(entry.4))
        .collect();
    let small = w < 600.0;
    f.text(
        Vec2::new(m, 105.0),
        category.label(),
        if small { 27.0 } else { 44.0 },
        category.accent_for(t),
    );
    super::button(
        f,
        s,
        l,
        rect(
            m + w - 110.0,
            if small { 154.0 } else { 112.0 },
            110.0,
            36.0,
        ),
        "Categories",
        Action::LobbyCategory(None),
        false,
    );
    f.text(
        Vec2::new(m, 164.0),
        format!("{} games", entries.len()),
        13.0,
        t.muted,
    );
    let clip = rect(m, 210.0, w, (f.size.y - 250.0).max(1.0));
    const ROW_H: f32 = 76.0;
    const GAP: f32 = 10.0;
    let count = entries.len();
    let content_h = count as f32 * ROW_H + (count - 1) as f32 * GAP;
    l.lobby_clip = Some(clip);
    l.lobby_max_scroll = (content_h - clip.height()).max(0.0);
    if s.keyboard
        && let Some(row) = s.focus.checked_sub(l.hits.len()).filter(|i| *i < count)
    {
        let top = row as f32 * (ROW_H + GAP);
        if top < s.lobby_scroll {
            s.lobby_scroll = top;
        } else if top + ROW_H > s.lobby_scroll + clip.height() {
            s.lobby_scroll = top + ROW_H - clip.height();
        }
    }
    s.lobby_scroll = s.lobby_scroll.clamp(0.0, l.lobby_max_scroll);
    let scroll = s.lobby_scroll;
    for (i, (title, desc, verb, role, screen)) in entries.into_iter().enumerate() {
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
            role.color(t),
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
        f.rect(track.min, track.size(), t.edge);
        let thumb_h = (track.height() * clip.height() / content_h)
            .max(22.0)
            .min(track.height());
        let thumb_y = track.min.y + (track.height() - thumb_h) * (scroll / l.lobby_max_scroll);
        f.rect(
            Vec2::new(track.min.x, thumb_y),
            Vec2::new(track.width(), thumb_h),
            t.accent,
        );
    }
}

fn category_grid(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let t = f.theme();
    let small = w < 600.0;
    f.text(
        Vec2::new(m, 104.0),
        "Choose a category",
        if small { 27.0 } else { 48.0 },
        t.ink,
    );
    let top = if small { 163.0 } else { 196.0 };
    let gap = if small { 10.0 } else { 16.0 };
    let width = (w - gap) * 0.5;
    let height = ((f.size.y - top - 45.0 - gap) * 0.5).clamp(86.0, 182.0);
    for (i, category) in LobbyCategory::ALL.into_iter().enumerate() {
        let tile = rect(
            m + (i % 2) as f32 * (width + gap),
            top + (i / 2) as f32 * (height + gap),
            width,
            height,
        );
        let accent = category.accent_for(t);
        let focused = s.keyboard && s.focus == l.hits.len();
        let hot = focused || s.cursor.is_some_and(|p| tile.contains(p));
        let hover_bg = t.panel_hover;
        f.rounded_rect(tile, 8.0, if hot { hover_bg } else { t.panel });
        f.rounded_outline(
            tile.min,
            tile.size(),
            8.0,
            if hot { 2.0 } else { 1.0 },
            if hot { accent } else { t.edge },
        );
        let inset = if small { 13.0 } else { 24.0 };
        f.rect(
            tile.min + Vec2::new(inset, 14.0),
            Vec2::new(if small { 26.0 } else { 44.0 }, 3.0),
            accent,
        );
        category_mark(
            f,
            category,
            tile.max
                - Vec2::new(
                    if small { 26.0 } else { 48.0 },
                    if small { 30.0 } else { 48.0 },
                ),
            if small { 12.0 } else { 23.0 },
        );
        f.text(
            tile.min + Vec2::new(inset, if small { 29.0 } else { 43.0 }),
            category.label(),
            if small { 17.0 } else { 31.0 },
            t.ink,
        );
        let count = ENTRIES
            .iter()
            .filter(|entry| category.includes(entry.4))
            .count();
        f.text(
            tile.min + Vec2::new(inset, if small { 57.0 } else { 91.0 }),
            format!("{count} games"),
            if small { 11.0 } else { 15.0 },
            accent,
        );
        l.hits.push(Hit {
            rect: tile,
            action: Action::LobbyCategory(Some(category)),
        });
    }
}

fn category_mark(f: &mut Frame, category: LobbyCategory, p: Vec2, r: f32) {
    let color = category.accent_for(f.theme());
    match category {
        LobbyCategory::Puzzle => {
            for (x, y) in [(-1.0, -1.0), (0.15, -1.0), (-1.0, 0.15)] {
                f.outline(p + Vec2::new(x, y) * r, Vec2::splat(r * 0.85), 1.4, color);
            }
            f.rect(p + Vec2::splat(r * 0.15), Vec2::splat(r * 0.85), color);
        }
        LobbyCategory::HighLimit => {
            super::circle(f, p, r, 1.5, color);
            super::circle(f, p, r * 0.66, 1.2, color);
            for offset in [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y] {
                f.line(p + offset * r * 0.68, p + offset * r, 2.0, color);
            }
        }
        LobbyCategory::Solitaire => {
            for x in [-1.0, -0.38, 0.24] {
                let card = rect(p.x + x * r, p.y - r, r * 0.82, r * 1.85);
                f.rect(card.min, card.size(), f.theme().panel);
                f.rounded_outline(card.min, card.size(), 2.0, 1.4, color);
            }
        }
        LobbyCategory::CardAndBoard => {
            f.rounded_outline(p - Vec2::splat(r), Vec2::splat(2.0 * r), 3.0, 1.4, color);
            for offset in [Vec2::ZERO, Vec2::splat(-0.47), Vec2::splat(0.47)] {
                f.rect(p + offset * r - Vec2::splat(1.5), Vec2::splat(3.0), color);
            }
        }
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
    let t = f.theme();
    let visible = intersect(r, clip);
    let index = l.hits.len();
    let hot = s.cursor.is_some_and(|p| r.contains(p) && clip.contains(p))
        || (s.keyboard && s.focus == index);
    let hover_bg = t.panel_hover;
    f.rect(
        visible.min,
        visible.size(),
        if hot { hover_bg } else { t.panel },
    );
    // Draw only clipped border segments so scrolling rows cannot paint into the page chrome.
    let edge = if hot { color } else { t.edge };
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
        f.text(title_pos, title, title_size, t.ink);
    }
    if desc_pos.y >= clip.min.y && desc_pos.y + desc_size <= clip.max.y {
        let available = (verb_x - desc_pos.x - 8.0).max(0.0);
        let desc = compact(desc, (available / (desc_size * 0.58)).floor() as usize);
        f.text(desc_pos, desc, desc_size, t.muted);
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
