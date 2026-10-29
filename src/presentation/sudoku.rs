use super::screens::button;
use super::*;
use gamevex_core::sudoku::{Command, Difficulty};

const BOARD_BG: Color = Color::srgb(0.012, 0.035, 0.052);
const CLUE: Color = Color::srgb(0.35, 0.89, 1.0);
const ENTRY: Color = Color::srgb(1.0, 0.74, 0.26);
const PEER: Color = Color::srgb(0.035, 0.12, 0.16);
const SAME: Color = Color::srgb(0.12, 0.22, 0.24);
const ERROR_BG: Color = Color::srgb(0.32, 0.055, 0.12);

pub(super) fn sudoku(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let game = &s.save.sudoku.game;
    let n = usize::from(game.size);
    if n == 0 || game.board.len() != n * n {
        return;
    }
    let mobile = w < 620.0;
    let short = f.size.y < 620.0;
    let (box_rows, box_cols) = game.box_dims();
    let selected = s.sudoku_ui.selected.min(n * n - 1);
    let sr = selected / n;
    let sc = selected % n;
    let selected_value = game.board[selected];
    let top = if short { 96.0 } else { 98.0 };
    f.text(
        Vec2::new(m, top),
        "SUDOKU",
        if mobile { 26.0 } else { 33.0 },
        INK,
    );
    let info_y = top + if mobile { 28.0 } else { 34.0 };
    f.text(
        Vec2::new(m, info_y),
        format!("{} x {}   /   {}", n, n, game.difficulty.label()),
        12.0,
        CYAN,
    );
    f.text(
        Vec2::new(m + if mobile { 170.0 } else { 190.0 }, info_y),
        format_time(game.elapsed_ms),
        12.0,
        GOLD,
    );
    let best = s.save.sudoku.records.best(game.size, game.difficulty);
    f.text(
        Vec2::new(m, info_y + 18.0),
        format!(
            "BEST  {}",
            best.map(format_time).unwrap_or_else(|| "--".into())
        ),
        10.0,
        MUTED,
    );
    f.text(
        Vec2::new(m + if mobile { 170.0 } else { 190.0 }, info_y + 18.0),
        if game.assisted {
            "ASSISTED RUN"
        } else {
            "UNASSISTED"
        },
        10.0,
        if game.assisted { GOLD } else { MUTED },
    );

    let board_top = info_y + 55.0;
    let footer_top = f.size.y - 34.0;
    let board_bottom = if mobile {
        footer_top - if short { 230.0 } else { 278.0 }
    } else {
        footer_top - 8.0
    };
    let panel_gap = 20.0;
    let panel_w = if mobile {
        0.0
    } else {
        (w * 0.34).clamp(270.0, 390.0)
    };
    let board_w = if mobile { w } else { w - panel_w - panel_gap };
    let available = rect(m, board_top, board_w, board_bottom - board_top);
    let cell = (available.width() / n as f32)
        .min(available.height() / n as f32)
        .max(32.0);
    let board_size = cell * n as f32;
    let clip_size = Vec2::new(
        board_size.min(available.width()),
        board_size.min(available.height()),
    );
    let clip = rect(
        available.min.x + (available.width() - clip_size.x) * 0.5,
        available.min.y,
        clip_size.x,
        clip_size.y,
    );
    let max_x = (board_size - clip.width()).max(0.0);
    let max_y = (board_size - clip.height()).max(0.0);
    let mut pan = s.sudoku_ui.pan;
    if s.sudoku_ui.follow {
        let desired_x = (sc as f32 * cell + cell * 0.5 - clip.width() * 0.5).clamp(0.0, max_x);
        let desired_y = (sr as f32 * cell + cell * 0.5 - clip.height() * 0.5).clamp(0.0, max_y);
        pan = Vec2::new(desired_x, desired_y);
    }
    pan.x = pan.x.clamp(0.0, max_x);
    pan.y = pan.y.clamp(0.0, max_y);
    s.sudoku_ui.pan = pan;
    l.sudoku_max_pan = Vec2::new(max_x, max_y);
    let origin = clip.min - pan;
    l.sudoku = Some(BoardLayout { origin, clip, cell });
    l.hits.push(Hit {
        rect: clip,
        action: Action::SudokuCell,
    });

    f.rect(clip.min, clip.size(), BOARD_BG);
    // Draw only visible cell surfaces and marks. Text is placed only when the whole cell is visible.
    for row in 0..n {
        for col in 0..n {
            let index = row * n + col;
            let cell_rect = rect(
                origin.x + col as f32 * cell,
                origin.y + row as f32 * cell,
                cell,
                cell,
            );
            if cell_rect.max.x <= clip.min.x
                || cell_rect.min.x >= clip.max.x
                || cell_rect.max.y <= clip.min.y
                || cell_rect.min.y >= clip.max.y
            {
                continue;
            }
            let value = game.board[index];
            let given = game.givens[index] != 0;
            let same_peer = (row == sr
                || col == sc
                || (row / box_rows == sr / box_rows && col / box_cols == sc / box_cols))
                && index != selected;
            let fill =
                if s.save.sudoku.highlight_errors && !given && value != 0 && game.conflict(index) {
                    ERROR_BG
                } else if index == selected {
                    Color::srgb(0.12, 0.16, 0.18)
                } else if s.save.sudoku.highlight_same
                    && selected_value != 0
                    && value == selected_value
                {
                    SAME
                } else if same_peer {
                    PEER
                } else {
                    BOARD_BG
                };
            let visible_min = cell_rect.min.max(clip.min);
            let visible_max = cell_rect.max.min(clip.max);
            f.rect(visible_min, visible_max - visible_min, fill);
            if value != 0 {
                let label = symbol(value);
                let fs = (cell * if n >= 16 { 0.48 } else { 0.56 }).clamp(12.0, 30.0);
                let tx = cell_rect.min.x + (cell - label.len() as f32 * fs * 0.58) * 0.5;
                let ty = cell_rect.min.y + (cell - fs) * 0.5 - 1.0;
                if cell_rect.min.x >= clip.min.x
                    && cell_rect.max.x <= clip.max.x
                    && cell_rect.min.y >= clip.min.y
                    && cell_rect.max.y <= clip.max.y
                {
                    f.text(
                        Vec2::new(tx, ty),
                        label,
                        fs,
                        if given {
                            CLUE
                        } else if s.save.sudoku.highlight_errors && game.conflict(index) {
                            INK
                        } else {
                            ENTRY
                        },
                    );
                }
            } else if game.notes[index] != 0
                && cell >= 32.0
                && cell_rect.min.x >= clip.min.x
                && cell_rect.max.x <= clip.max.x
                && cell_rect.min.y >= clip.min.y
                && cell_rect.max.y <= clip.max.y
            {
                let note_fs = (cell * 0.19).clamp(7.0, 10.0);
                let cols = (n as f32).sqrt().ceil() as usize;
                for value in 1..=n {
                    if game.notes[index] & (1u16 << (value - 1)) == 0 {
                        continue;
                    }
                    let k = value - 1;
                    let x = cell_rect.min.x + ((k % cols) as f32 + 0.5) * cell / cols as f32
                        - note_fs * 0.27;
                    let y = cell_rect.min.y + (k / cols) as f32 * (cell / cols as f32) + 1.0;
                    f.text(Vec2::new(x, y), symbol(value as u8), note_fs, MUTED);
                }
            }
        }
    }
    // Grid lines have clipped endpoints; box boundaries read stronger than cell lines.
    for i in 0..=n {
        let weight = if i % box_cols == 0 { 2.0 } else { 0.7 };
        let color = if i % box_cols == 0 { CYAN } else { EDGE };
        let x = origin.x + i as f32 * cell;
        if x >= clip.min.x && x <= clip.max.x {
            f.line(
                Vec2::new(x, clip.min.y),
                Vec2::new(x, clip.max.y),
                weight,
                color,
            );
        }
        let weight = if i % box_rows == 0 { 2.0 } else { 0.7 };
        let color = if i % box_rows == 0 { CYAN } else { EDGE };
        let y = origin.y + i as f32 * cell;
        if y >= clip.min.y && y <= clip.max.y {
            f.line(
                Vec2::new(clip.min.x, y),
                Vec2::new(clip.max.x, y),
                weight,
                color,
            );
        }
    }
    let completion_pulse = if !s.save.settings.reduced_motion {
        s.sudoku_ui
            .completion_age
            .filter(|age| *age < 0.8)
            .map(|age| 1.0 - age / 0.8)
            .unwrap_or(0.0)
    } else {
        0.0
    };
    f.outline(
        clip.min,
        clip.size(),
        2.0 + completion_pulse * 2.5,
        if completion_pulse > 0.0 { GOLD } else { CYAN },
    );
    let selected_rect = rect(
        origin.x + sc as f32 * cell,
        origin.y + sr as f32 * cell,
        cell,
        cell,
    );
    if selected_rect.min.x >= clip.min.x
        && selected_rect.max.x <= clip.max.x
        && selected_rect.min.y >= clip.min.y
        && selected_rect.max.y <= clip.max.y
    {
        f.outline(selected_rect.min, selected_rect.size(), 2.5, GOLD);
    }

    let panel_x = m + board_w + panel_gap;
    let panel_y = board_top;
    if !mobile {
        f.text(
            Vec2::new(panel_x, panel_y),
            "VALUES / REMAINING",
            11.0,
            MUTED,
        );
        let bank_cols = if n > 9 && short {
            8
        } else if n > 9 {
            4
        } else {
            n.min(5)
        };
        let gap = 5.0;
        let bw = (panel_w - gap * (bank_cols as f32 - 1.0)) / bank_cols as f32;
        for value in 1..=n {
            let x = panel_x + ((value - 1) % bank_cols) as f32 * (bw + gap);
            let y = panel_y + 18.0 + ((value - 1) / bank_cols) as f32 * 35.0;
            number_button(f, s, l, rect(x, y, bw, 30.0), value as u8, selected);
        }
        let rows = n.div_ceil(bank_cols);
        let controls_y = panel_y + 22.0 + rows as f32 * 35.0;
        let controls_bottom = panel_controls(f, s, l, panel_x, controls_y, panel_w, false, short);
        keyboard_help(f, panel_x, controls_bottom + 3.0, n);
        if !s.sudoku_ui.message.is_empty() {
            draw_message(
                f,
                &s.sudoku_ui.message,
                panel_x,
                controls_bottom + 17.0,
                (panel_w / 6.0) as usize,
            );
        }
    } else {
        let bank_y = board_bottom + 9.0;
        f.text(Vec2::new(m, bank_y), "VALUES / REMAINING", 9.0, MUTED);
        let cols = if n > 9 { 8 } else { n };
        let gap = 3.0;
        let bw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
        for value in 1..=n {
            let x = m + ((value - 1) % cols) as f32 * (bw + gap);
            let y = bank_y + 14.0 + ((value - 1) / cols) as f32 * 31.0;
            number_button(f, s, l, rect(x, y, bw, 27.0), value as u8, selected);
        }
        let control_y = bank_y + 18.0 + n.div_ceil(cols) as f32 * 31.0;
        let controls_bottom = panel_controls(f, s, l, m, control_y, w, true, short);
        keyboard_help(f, m, controls_bottom + 3.0, n);
        if !s.sudoku_ui.message.is_empty() {
            draw_message(
                f,
                &s.sudoku_ui.message,
                m,
                (controls_bottom + 17.0).max(f.size.y - 78.0),
                (w / 6.0) as usize,
            );
        }
    }

    if game.complete {
        let result = if game.assisted {
            format!(
                "SOLVED {} / ASSISTED / RECORD NOT UPDATED",
                format_time(game.elapsed_ms)
            )
        } else {
            format!(
                "SOLVED {} / BEST {}",
                format_time(game.elapsed_ms),
                best.map(format_time).unwrap_or_else(|| "--".into())
            )
        };
        f.text(Vec2::new(m, board_top - 16.0), result, 10.0, GREEN);
    }
}

#[allow(clippy::too_many_arguments)]
fn panel_controls(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    x: f32,
    y: f32,
    w: f32,
    mobile: bool,
    dense: bool,
) -> f32 {
    let gap = if dense { 3.0 } else { 6.0 };
    let cols = if dense || (mobile && w > 340.0) {
        4
    } else if mobile {
        3
    } else {
        2
    };
    let bw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
    let bh = if dense {
        25.0
    } else if mobile {
        29.0
    } else {
        34.0
    };
    let mut row = 0;
    let mut col = 0;
    let mut add = |label: &str,
                   action: Action,
                   active: bool,
                   f: &mut Frame,
                   s: &Arcade,
                   l: &mut ViewLayout| {
        let r = rect(
            x + col as f32 * (bw + gap),
            y + row as f32 * (bh + gap),
            bw,
            bh,
        );
        button(f, s, l, r, label, action, active);
        col += 1;
        if col == cols {
            col = 0;
            row += 1;
        }
    };
    add(
        if s.sudoku_ui.pencil {
            "PENCIL ON"
        } else {
            "PENCIL"
        },
        Action::SudokuPencil,
        s.sudoku_ui.pencil,
        f,
        s,
        l,
    );
    add(
        "ERASE",
        Action::Sudoku(Command::Erase(s.sudoku_ui.selected)),
        false,
        f,
        s,
        l,
    );
    add(
        "UNDO",
        Action::Sudoku(Command::Undo),
        !s.sudoku_ui.history.is_empty(),
        f,
        s,
        l,
    );
    add("HINT", Action::Sudoku(Command::Hint), false, f, s, l);
    add(
        if s.save.sudoku.highlight_errors {
            "ERRORS ON"
        } else {
            "ERRORS OFF"
        },
        Action::SudokuErrors,
        s.save.sudoku.highlight_errors,
        f,
        s,
        l,
    );
    add(
        if s.save.sudoku.highlight_same {
            "SAME ON"
        } else {
            "SAME OFF"
        },
        Action::SudokuSame,
        s.save.sudoku.highlight_same,
        f,
        s,
        l,
    );
    add(
        &format!("SIZE > {}", next_size(s.save.sudoku.game.size)),
        Action::SudokuNew(
            next_size(s.save.sudoku.game.size),
            s.save.sudoku.game.difficulty,
        ),
        false,
        f,
        s,
        l,
    );
    add(
        &format!(
            "DIFF > {}",
            difficulty_short(next_difficulty(s.save.sudoku.game.difficulty))
        ),
        Action::SudokuNew(
            s.save.sudoku.game.size,
            next_difficulty(s.save.sudoku.game.difficulty),
        ),
        false,
        f,
        s,
        l,
    );
    if col > 0 {
        row += 1;
    }
    let new_w = (w - gap) * 0.5;
    button(
        f,
        s,
        l,
        rect(x, y + row as f32 * (bh + gap), new_w, bh),
        "NEW BOARD",
        Action::SudokuNew(s.save.sudoku.game.size, s.save.sudoku.game.difficulty),
        false,
    );
    f.text(
        Vec2::new(x, y + (row + 1) as f32 * (bh + gap) + 2.0),
        if s.save.sudoku.game.size == 16 {
            "16x16 experimental / hints off"
        } else {
            "Difficulty sets clue density"
        },
        9.0,
        MUTED,
    );
    y + (row + 1) as f32 * (bh + gap) + 12.0
}

fn keyboard_help(f: &mut Frame, x: f32, y: f32, size: usize) {
    f.text(
        Vec2::new(x, y),
        if size == 16 {
            "Keys: arrows, 1-9/A-G, P notes, U undo"
        } else {
            "Keys: arrows, 1-9, P notes, U undo, H hint"
        },
        9.0,
        MUTED,
    );
}

fn draw_message(f: &mut Frame, message: &str, x: f32, y: f32, width: usize) {
    for (i, line) in wrap_lines(message, width, 2).iter().enumerate() {
        f.text(Vec2::new(x, y + i as f32 * 13.0), line, 10.0, GOLD);
    }
}

fn difficulty_short(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Easy => "EASY",
        Difficulty::Medium => "MED",
        Difficulty::Hard => "HARD",
        Difficulty::Expert => "EXP",
    }
}

fn number_button(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    value: u8,
    selected: usize,
) {
    let n = usize::from(s.save.sudoku.game.size);
    let remaining = s
        .save
        .sudoku
        .game
        .board
        .iter()
        .filter(|&&v| v == value)
        .count();
    let count = n.saturating_sub(remaining);
    let idx = l.hits.len();
    let focused = s.keyboard && s.focus == idx;
    f.rect(
        r.min,
        r.size(),
        if focused {
            Color::srgb(0.055, 0.14, 0.18)
        } else {
            PANEL
        },
    );
    f.outline(
        r.min,
        r.size(),
        if focused { 2.0 } else { 1.0 },
        if focused { GOLD } else { CYAN },
    );
    let fs = if r.width() < 34.0 { 11.0 } else { 14.0 };
    f.text(
        Vec2::new(r.min.x + 4.0, r.min.y + (r.height() - fs) * 0.5),
        symbol(value),
        fs,
        if count == 0 { MUTED } else { INK },
    );
    let count_font = if r.width() < 50.0 { 8.0 } else { 10.0 };
    let count_label = count.to_string();
    f.text(
        Vec2::new(
            r.max.x - 4.0 - count_label.len() as f32 * count_font * 0.6,
            r.min.y + (r.height() - count_font) * 0.5,
        ),
        count_label,
        count_font,
        MUTED,
    );
    l.hits.push(Hit {
        rect: r,
        action: Action::Sudoku(Command::Enter {
            index: selected,
            value,
            pencil: s.sudoku_ui.pencil,
        }),
    });
}

fn symbol(value: u8) -> String {
    if value <= 9 {
        value.to_string()
    } else {
        char::from(b'A' + value - 10).to_string()
    }
}

fn wrap_lines(message: &str, width: usize, limit: usize) -> Vec<String> {
    let width = width.max(12);
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in message.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
            if lines.len() == limit {
                let last = lines.last_mut().unwrap();
                last.truncate(last.len().min(width.saturating_sub(3)));
                last.push_str("...");
                return lines;
            }
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() && lines.len() < limit {
        lines.push(line);
    }
    lines
}

fn next_size(size: u8) -> u8 {
    match size {
        4 => 6,
        6 => 9,
        9 => 16,
        _ => 4,
    }
}
fn next_difficulty(d: Difficulty) -> Difficulty {
    let i = Difficulty::ALL.iter().position(|&v| v == d).unwrap_or(0);
    Difficulty::ALL[(i + 1) % Difficulty::ALL.len()]
}

#[cfg(test)]
mod tests {
    #[test]
    fn hint_text_uses_both_lines_before_truncating() {
        assert_eq!(
            super::wrap_lines("One short line. Another short line.", 19, 2),
            ["One short line.", "Another short line."]
        );
        let lines = super::wrap_lines(
            "One short line. Another short line. More words beyond the limit.",
            19,
            2,
        );
        assert_eq!(lines.len(), 2);
        assert!(lines[1].ends_with("..."));
    }
}
