use super::*;

#[derive(Clone, Copy)]
pub struct MinesPalette {
    pub board_bg: Color,
    pub cell_unrevealed: Color,
    pub cell_hovered: Color,
    pub cell_revealed: Color,
    pub cell_exploded: Color,
    pub cell_unrevealed_line: Color,
    pub cell_revealed_outline: Color,
    pub mine_normal: Color,
    pub mine_exploded: Color,
    pub flag: Color,
    pub wrong_flag: Color,
    pub num1: Color,
    pub num2: Color,
    pub num3: Color,
    pub num4: Color,
    pub num5: Color,
}

impl MinesPalette {
    pub fn for_scheme(scheme: gamevex_core::ColorScheme) -> Self {
        match scheme {
            gamevex_core::ColorScheme::Cyan => Self {
                board_bg: Color::srgb(0.01, 0.03, 0.047),
                cell_unrevealed: Color::srgb(0.027, 0.08, 0.115),
                cell_hovered: Color::srgb(0.05, 0.15, 0.20),
                cell_revealed: Color::srgb(0.018, 0.045, 0.065),
                cell_exploded: Color::srgb(0.34, 0.025, 0.08),
                cell_unrevealed_line: Color::srgb(0.12, 0.34, 0.41),
                cell_revealed_outline: Color::srgb(0.04, 0.12, 0.16),
                mine_normal: Color::srgb(1.0, 0.0, 0.67),
                mine_exploded: Color::srgb(1.0, 0.2, 0.4),
                flag: Color::srgb(1.0, 0.67, 0.0),
                wrong_flag: Color::srgb(1.0, 0.2, 0.4),
                num1: Color::srgb(0.0, 0.83, 1.0),
                num2: Color::srgb(0.0, 1.0, 0.72),
                num3: Color::srgb(1.0, 0.67, 0.0),
                num4: Color::srgb(1.0, 0.0, 0.67),
                num5: Color::srgb(1.0, 0.2, 0.4),
            },
            gamevex_core::ColorScheme::Amber => Self {
                board_bg: Color::srgb(0.038, 0.026, 0.016),
                cell_unrevealed: Color::srgb(0.085, 0.058, 0.032),
                cell_hovered: Color::srgb(0.15, 0.10, 0.05),
                cell_revealed: Color::srgb(0.05, 0.035, 0.022),
                cell_exploded: Color::srgb(0.36, 0.05, 0.06),
                cell_unrevealed_line: Color::srgb(0.32, 0.22, 0.12),
                cell_revealed_outline: Color::srgb(0.12, 0.08, 0.045),
                mine_normal: Color::srgb(1.0, 0.45, 0.30),
                mine_exploded: Color::srgb(1.0, 0.25, 0.20),
                flag: Color::srgb(1.0, 0.75, 0.20),
                wrong_flag: Color::srgb(1.0, 0.25, 0.20),
                num1: Color::srgb(1.0, 0.76, 0.20),
                num2: Color::srgb(0.28, 0.88, 0.58),
                num3: Color::srgb(1.0, 0.40, 0.25),
                num4: Color::srgb(0.95, 0.92, 0.82),
                num5: Color::srgb(1.0, 0.22, 0.32),
            },
            gamevex_core::ColorScheme::Emerald => Self {
                board_bg: Color::srgb(0.014, 0.038, 0.025),
                cell_unrevealed: Color::srgb(0.035, 0.095, 0.065),
                cell_hovered: Color::srgb(0.055, 0.15, 0.10),
                cell_revealed: Color::srgb(0.022, 0.055, 0.038),
                cell_exploded: Color::srgb(0.32, 0.06, 0.08),
                cell_unrevealed_line: Color::srgb(0.18, 0.45, 0.32),
                cell_revealed_outline: Color::srgb(0.045, 0.11, 0.075),
                mine_normal: Color::srgb(1.0, 0.48, 0.36),
                mine_exploded: Color::srgb(1.0, 0.25, 0.25),
                flag: Color::srgb(0.40, 0.95, 0.65),
                wrong_flag: Color::srgb(1.0, 0.30, 0.30),
                num1: Color::srgb(0.35, 0.95, 0.70),
                num2: Color::srgb(0.20, 0.90, 0.50),
                num3: Color::srgb(1.0, 0.52, 0.38),
                num4: Color::srgb(0.85, 0.98, 0.90),
                num5: Color::srgb(1.0, 0.30, 0.30),
            },
            gamevex_core::ColorScheme::Violet => Self {
                board_bg: Color::srgb(0.036, 0.020, 0.048),
                cell_unrevealed: Color::srgb(0.085, 0.052, 0.115),
                cell_hovered: Color::srgb(0.14, 0.085, 0.19),
                cell_revealed: Color::srgb(0.052, 0.030, 0.070),
                cell_exploded: Color::srgb(0.34, 0.04, 0.15),
                cell_unrevealed_line: Color::srgb(0.35, 0.22, 0.45),
                cell_revealed_outline: Color::srgb(0.11, 0.065, 0.14),
                mine_normal: Color::srgb(0.82, 0.55, 1.0),
                mine_exploded: Color::srgb(1.0, 0.22, 0.45),
                flag: Color::srgb(0.25, 0.95, 0.88),
                wrong_flag: Color::srgb(1.0, 0.25, 0.45),
                num1: Color::srgb(0.25, 0.95, 0.88),
                num2: Color::srgb(0.82, 0.55, 1.0),
                num3: Color::srgb(0.98, 0.55, 0.85),
                num4: Color::srgb(0.92, 0.88, 0.98),
                num5: Color::srgb(1.0, 0.30, 0.55),
            },
            gamevex_core::ColorScheme::Ice => Self {
                board_bg: Color::srgb(0.015, 0.028, 0.052),
                cell_unrevealed: Color::srgb(0.045, 0.075, 0.125),
                cell_hovered: Color::srgb(0.08, 0.13, 0.20),
                cell_revealed: Color::srgb(0.028, 0.048, 0.080),
                cell_exploded: Color::srgb(0.30, 0.05, 0.12),
                cell_unrevealed_line: Color::srgb(0.20, 0.35, 0.52),
                cell_revealed_outline: Color::srgb(0.06, 0.10, 0.16),
                mine_normal: Color::srgb(1.0, 0.45, 0.62),
                mine_exploded: Color::srgb(1.0, 0.25, 0.35),
                flag: Color::srgb(0.40, 0.85, 1.0),
                wrong_flag: Color::srgb(1.0, 0.30, 0.45),
                num1: Color::srgb(0.40, 0.82, 1.0),
                num2: Color::srgb(0.45, 0.92, 0.80),
                num3: Color::srgb(1.0, 0.48, 0.62),
                num4: Color::srgb(0.90, 0.95, 1.0),
                num5: Color::srgb(1.0, 0.30, 0.45),
            },
            gamevex_core::ColorScheme::Paper => Self {
                board_bg: Color::srgb(0.93, 0.91, 0.86),
                cell_unrevealed: Color::srgb(0.90, 0.88, 0.83),
                cell_hovered: Color::srgb(0.85, 0.83, 0.77),
                cell_revealed: Color::srgb(0.96, 0.95, 0.92),
                cell_exploded: Color::srgb(0.92, 0.72, 0.75),
                cell_unrevealed_line: Color::srgb(0.75, 0.72, 0.66),
                cell_revealed_outline: Color::srgb(0.80, 0.78, 0.73),
                mine_normal: Color::srgb(0.18, 0.18, 0.20),
                mine_exploded: Color::srgb(0.72, 0.08, 0.15),
                flag: Color::srgb(0.68, 0.08, 0.18),
                wrong_flag: Color::srgb(0.75, 0.12, 0.16),
                num1: Color::srgb(0.08, 0.32, 0.68),
                num2: Color::srgb(0.08, 0.45, 0.20),
                num3: Color::srgb(0.68, 0.08, 0.16),
                num4: Color::srgb(0.14, 0.15, 0.18),
                num5: Color::srgb(0.58, 0.20, 0.06),
            },
        }
    }
}

pub(crate) fn mines(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let palette = MinesPalette::for_scheme(f.palette);
    let t = f.theme();
    let narrow = w < 700.0;
    f.text(
        Vec2::new(m, 100.0),
        "Minesweeper",
        if narrow { 31.0 } else { 42.0 },
        t.ink,
    );
    let y = 158.0;
    let difficulty = s.save.minesweeper.game.difficulty;
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
    let status = match s.save.minesweeper.game.status {
        Status::Ready => "READY",
        Status::Playing => "PLAYING",
        Status::Won => "CLEARED",
        Status::Lost => "MINE HIT",
    };
    let status_color = match s.save.minesweeper.game.status {
        Status::Won => t.success,
        Status::Lost => t.danger,
        _ => t.accent,
    };
    let values = [
        ("Field", status.to_string(), status_color),
        (
            "Time",
            format_time(s.save.minesweeper.game.elapsed_ms),
            t.ink,
        ),
        (
            "Mines left",
            format!("{:02}", s.save.minesweeper.game.remaining_mines()),
            t.gold,
        ),
        (
            "Best",
            s.save
                .minesweeper
                .records
                .best(difficulty)
                .map(format_time)
                .unwrap_or("--".into()),
            t.secondary,
        ),
    ];
    for (i, (label, value, color)) in values.into_iter().enumerate() {
        let x = m + i as f32 * w / 4.0;
        f.text(Vec2::new(x, rail_y), label, 12.0, t.muted);
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
        palette.board_bg,
    );
    f.outline(Vec2::new(m, top), Vec2::new(w, bottom - top), 1.0, t.edge);
    let cell = if difficulty == Difficulty::Beginner && w > 500.0 {
        38.0
    } else {
        32.0
    };
    let board_size = Vec2::new(
        s.save.minesweeper.game.cols() as f32,
        s.save.minesweeper.game.rows() as f32,
    ) * cell;
    let max_pan = (board_size - clip.size()).max(Vec2::ZERO);
    s.pan = s.pan.clamp(Vec2::ZERO, max_pan);
    let origin = clip.min + (clip.size() - board_size).max(Vec2::ZERO) * 0.5 - s.pan;
    let focused = s.keyboard;
    let board_focus = s.focus == l.hits.len();
    let board_layout = BoardLayout { origin, clip, cell };
    for (i, c) in s.save.minesweeper.game.cells.iter().enumerate() {
        let p = origin
            + Vec2::new(
                (i % s.save.minesweeper.game.cols()) as f32,
                (i / s.save.minesweeper.game.cols()) as f32,
            ) * cell;
        let r = rect(p.x + 1.5, p.y + 1.5, cell - 3.0, cell - 3.0);
        let visible = r.intersect(clip);
        if visible.is_empty() {
            continue;
        }
        let hovered = s.cursor.is_some_and(|v| r.contains(v) && clip.contains(v));
        let fill = if s.save.minesweeper.game.exploded == Some(i) {
            palette.cell_exploded
        } else if c.revealed {
            palette.cell_revealed
        } else if hovered {
            palette.cell_hovered
        } else {
            palette.cell_unrevealed
        };
        f.rect(visible.min, visible.size(), fill);
        f.outline(
            visible.min,
            visible.size(),
            1.0,
            if c.revealed {
                palette.cell_revealed_outline
            } else {
                t.edge
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
                if s.save.minesweeper.game.exploded == Some(i) {
                    palette.mine_exploded
                } else {
                    palette.mine_normal
                },
            );
        } else if c.flagged {
            flag(f, r.center(), cell * 0.28, palette.flag);
        } else if c.revealed && c.adjacent > 0 {
            let color = match c.adjacent {
                1 => palette.num1,
                2 => palette.num2,
                3 => palette.num3,
                4 => palette.num4,
                _ => palette.num5,
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
                palette.cell_unrevealed_line,
            );
        }
        if s.save.minesweeper.game.status == Status::Lost && c.flagged && !c.mine {
            f.line(
                r.min + Vec2::splat(6.0),
                r.max - Vec2::splat(6.0),
                2.0,
                palette.wrong_flag,
            );
            f.line(
                r.min + Vec2::new(6.0, r.height() - 6.0),
                r.min + Vec2::new(r.width() - 6.0, 6.0),
                2.0,
                palette.wrong_flag,
            );
        }
        if board_focus && s.cell == i && (s.keyboard || focused) {
            f.outline(
                r.min - Vec2::splat(1.0),
                r.size() + Vec2::splat(2.0),
                2.0,
                t.gold,
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
            t.gold,
        );
        if max_pan.x > 0.0 {
            let bar = w * (clip.width() / board_size.x).min(1.0);
            let x = m + (w - bar) * s.pan.x / max_pan.x;
            f.rect(Vec2::new(x, bottom - 3.0), Vec2::new(bar, 3.0), t.accent);
        }
    } else {
        f.text(
            Vec2::new(m, hint_y),
            compact(&s.message, (w / 7.2) as usize),
            12.0,
            if s.save.minesweeper.game.status == Status::Lost {
                t.danger
            } else {
                t.muted
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
        t.ink,
    );
    f.text(
        Vec2::new(m, f.size.y - 53.0),
        if narrow {
            "Enter: chord / Tab: controls / Esc: lobby"
        } else {
            "Right click: flag   Click an open number: chord   Esc: lobby   Timer pauses away"
        },
        12.0,
        t.muted,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamevex_core::ColorScheme;

    #[test]
    fn mines_palette_supports_all_schemes_and_paper_is_light() {
        for scheme in [
            ColorScheme::Cyan,
            ColorScheme::Amber,
            ColorScheme::Emerald,
            ColorScheme::Violet,
            ColorScheme::Ice,
            ColorScheme::Paper,
        ] {
            let pal = MinesPalette::for_scheme(scheme);
            let bg = pal.board_bg.to_srgba();
            let unrev = pal.cell_unrevealed.to_srgba();
            let rev = pal.cell_revealed.to_srgba();
            assert!(bg.red.is_finite() && bg.green.is_finite() && bg.blue.is_finite());
            assert!(unrev.red.is_finite() && unrev.green.is_finite() && unrev.blue.is_finite());
            assert!(rev.red.is_finite() && rev.green.is_finite() && rev.blue.is_finite());
        }

        let paper = MinesPalette::for_scheme(ColorScheme::Paper);
        let bg = paper.board_bg.to_srgba();
        let rev = paper.cell_revealed.to_srgba();
        let unrev = paper.cell_unrevealed.to_srgba();
        assert!(bg.red > 0.85 && bg.green > 0.85 && bg.blue > 0.80);
        assert!(rev.red > 0.85 && rev.green > 0.85 && rev.blue > 0.85);
        assert!(unrev.red > 0.85 && unrev.green > 0.85 && unrev.blue > 0.80);

        // Mines numbers on Paper must avoid faint near-white colors: each must be dark and readable
        for (i, num) in [paper.num1, paper.num2, paper.num3, paper.num4, paper.num5]
            .iter()
            .enumerate()
        {
            let n = num.to_srgba();
            let lum = n.red * 0.2126 + n.green * 0.7152 + n.blue * 0.0722;
            assert!(
                lum < 0.40,
                "Paper mines num{} must be dark and readable, got lum={lum}",
                i + 1
            );
        }
    }
}
