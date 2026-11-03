use super::*;

pub(crate) fn mines(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let narrow = w < 700.0;
    f.text(
        Vec2::new(m, 100.0),
        "Minesweeper",
        if narrow { 31.0 } else { 42.0 },
        INK,
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
        Status::Won => GREEN,
        Status::Lost => RED,
        _ => CYAN,
    };
    let values = [
        ("Field", status.to_string(), status_color),
        ("Time", format_time(s.save.minesweeper.game.elapsed_ms), INK),
        (
            "Mines left",
            format!("{:02}", s.save.minesweeper.game.remaining_mines()),
            GOLD,
        ),
        (
            "Best",
            s.save
                .minesweeper
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
                if s.save.minesweeper.game.exploded == Some(i) {
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
        if s.save.minesweeper.game.status == Status::Lost && c.flagged && !c.mine {
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
            if s.save.minesweeper.game.status == Status::Lost {
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
