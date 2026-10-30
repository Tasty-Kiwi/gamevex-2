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
    let selection_notice = game.givens.get(selected).copied().unwrap_or(0) != 0
        || (s.sudoku_ui.pencil && game.board[selected] != 0);
    let standard_cols = match n {
        4 => 2,
        6 | 9 => 3,
        _ => 4,
    };
    let compact_rows = n.div_ceil(n.min(8));
    let standard_rows = n.div_ceil(standard_cols);
    let short_controls = 23.0
        + compact_rows as f32 * 36.0
        + 142.0
        + if s.sudoku_ui.show_highlights {
            35.0
        } else {
            0.0
        }
        + if !s.sudoku_ui.message.is_empty() {
            26.0
        } else {
            0.0
        }
        + if selection_notice { 12.0 } else { 0.0 };
    let normal_controls = 23.0
        + standard_rows as f32 * 47.0
        + 144.0
        + if s.sudoku_ui.show_highlights {
            35.0
        } else {
            0.0
        }
        + if !s.sudoku_ui.message.is_empty() {
            26.0
        } else {
            0.0
        }
        + if selection_notice { 12.0 } else { 0.0 };
    let board_bottom = if mobile {
        footer_top
            - if short {
                short_controls
            } else {
                normal_controls
            }
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
    // The side panel can start beside the header, leaving the full height for
    // the board and the bank on short desktop windows.
    let panel_y = if !mobile && short {
        board_top - 22.0
    } else {
        board_top
    };
    if !mobile {
        let cols = match n {
            4 => 2,
            6 | 9 => 3,
            16 if short => 8,
            16 => 4,
            _ => n.min(5),
        };
        let controls_bottom = draw_panel(f, s, l, panel_x, panel_y, panel_w, cols, short, selected);
        if !s.sudoku_ui.message.is_empty() {
            draw_message(
                f,
                &s.sudoku_ui.message,
                panel_x,
                controls_bottom + 3.0,
                (panel_w / 6.0) as usize,
            );
        }
    } else {
        let bank_y = board_bottom + 7.0;
        let cols = if short { n.min(8) } else { standard_cols };
        let controls_bottom = draw_mobile(f, s, l, m, bank_y, w, cols, selected, short);
        if !s.sudoku_ui.message.is_empty() {
            draw_message(
                f,
                &s.sudoku_ui.message,
                m,
                controls_bottom + 1.0,
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
fn draw_panel(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    x: f32,
    y: f32,
    w: f32,
    cols: usize,
    dense: bool,
    selected: usize,
) -> f32 {
    let n = usize::from(s.save.sudoku.game.size);
    bank_heading(f, s, x, y, w);
    let gap = 5.0;
    let button_h = if dense { 38.0 } else { 46.0 };
    let bw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
    let rows = n.div_ceil(cols);
    for value in 1..=n {
        let bx = x + ((value - 1) % cols) as f32 * (bw + gap);
        let by = y + 18.0 + ((value - 1) / cols) as f32 * (button_h + gap);
        number_button(f, s, l, rect(bx, by, bw, button_h), value as u8, selected);
    }
    draw_editor_controls(
        f,
        s,
        l,
        x,
        y + 22.0 + rows as f32 * (button_h + gap),
        w,
        dense,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn draw_mobile(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    x: f32,
    y: f32,
    w: f32,
    cols: usize,
    selected: usize,
    dense: bool,
) -> f32 {
    let n = usize::from(s.save.sudoku.game.size);
    bank_heading(f, s, x, y, w);
    let gap = 4.0;
    let button_h = if dense { 32.0 } else { 42.0 };
    let row_gap = if dense { 4.0 } else { 5.0 };
    let bw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
    let rows = n.div_ceil(cols);
    for value in 1..=n {
        let bx = x + ((value - 1) % cols) as f32 * (bw + gap);
        let by = y + 16.0 + ((value - 1) / cols) as f32 * (button_h + row_gap);
        number_button(f, s, l, rect(bx, by, bw, button_h), value as u8, selected);
    }
    draw_editor_controls(
        f,
        s,
        l,
        x,
        y + 16.0 + rows as f32 * (button_h + row_gap),
        w,
        dense,
        true,
    )
}

fn bank_heading(f: &mut Frame, s: &Arcade, x: f32, y: f32, w: f32) {
    f.text(
        Vec2::new(x, y),
        if s.sudoku_ui.pencil {
            "ADD NOTES"
        } else {
            "ENTER VALUE"
        },
        10.0,
        MUTED,
    );
    f.text(Vec2::new(x + w - 128.0, y), "REMAINING", 10.0, MUTED);
    f.text(
        Vec2::new(x + w - 62.0, y),
        if s.sudoku_ui.pencil {
            "NOTES ON"
        } else {
            "NOTES OFF"
        },
        8.5,
        if s.sudoku_ui.pencil { GOLD } else { MUTED },
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_editor_controls(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    x: f32,
    y: f32,
    w: f32,
    dense: bool,
    mobile: bool,
) -> f32 {
    let game = &s.save.sudoku.game;
    let selected = s.sudoku_ui.selected.min(game.board.len().saturating_sub(1));
    let completed = game.complete;
    let given = game.givens.get(selected).copied().unwrap_or(0) != 0;
    let has_value = game.board.get(selected).copied().unwrap_or(0) != 0;
    let has_notes = game.notes.get(selected).copied().unwrap_or(0) != 0;
    let gap = if dense { 4.0 } else { 6.0 };
    let bh = if mobile { 32.0 } else { 34.0 };
    let cols = if mobile || dense { 4 } else { 2 };
    let bw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
    let actions = [
        (
            if s.sudoku_ui.pencil {
                "NOTES ON"
            } else {
                "NOTES OFF"
            },
            Action::SudokuPencil,
            !completed,
            s.sudoku_ui.pencil,
        ),
        (
            "ERASE",
            Action::Sudoku(Command::Erase(selected)),
            !completed && !given && (has_value || has_notes),
            false,
        ),
        (
            "UNDO  U",
            Action::Sudoku(Command::Undo),
            !completed && !s.sudoku_ui.history.is_empty(),
            false,
        ),
        (
            "HINT  H",
            Action::Sudoku(Command::Hint),
            hint_available(s),
            false,
        ),
    ];
    for (i, (label, action, enabled, active)) in actions.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let r = rect(
            x + col as f32 * (bw + gap),
            y + row as f32 * (bh + gap),
            bw,
            bh,
        );
        control_button(f, s, l, r, label, *action, *enabled, *active);
    }
    let mut cursor = y + (bh + gap) * actions.len().div_ceil(cols) as f32;
    f.text(
        Vec2::new(x, cursor),
        if hint_available(s) {
            "Hint marks this run assisted."
        } else {
            hint_reason(s)
        },
        9.0,
        MUTED,
    );
    cursor += 12.0;
    let disclosure_h = 32.0;
    control_button(
        f,
        s,
        l,
        rect(x, cursor, w, disclosure_h),
        if s.sudoku_ui.show_highlights {
            "HIGHLIGHTS -"
        } else {
            "HIGHLIGHTS +"
        },
        Action::SudokuHighlights,
        true,
        s.sudoku_ui.show_highlights,
    );
    cursor += disclosure_h + 3.0;
    if s.sudoku_ui.show_highlights {
        let pw = (w - gap) * 0.5;
        control_button(
            f,
            s,
            l,
            rect(x, cursor, pw, 32.0),
            if s.save.sudoku.highlight_errors {
                "ERRORS  ON"
            } else {
                "ERRORS  OFF"
            },
            Action::SudokuErrors,
            true,
            s.save.sudoku.highlight_errors,
        );
        control_button(
            f,
            s,
            l,
            rect(x + pw + gap, cursor, pw, 32.0),
            if s.save.sudoku.highlight_same {
                "SAME  ON"
            } else {
                "SAME  OFF"
            },
            Action::SudokuSame,
            true,
            s.save.sudoku.highlight_same,
        );
        cursor += 35.0;
    }
    control_button(
        f,
        s,
        l,
        rect(x, cursor, w, if mobile { 32.0 } else { 34.0 }),
        "NEW PUZZLE",
        Action::SudokuSetup,
        true,
        true,
    );
    cursor += if mobile { 35.0 } else { 37.0 };
    f.text(
        Vec2::new(x, cursor),
        if game.size == 16 {
            "16x16 experimental - hints off"
        } else {
            "Difficulty sets clue density"
        },
        9.0,
        MUTED,
    );
    cursor += 12.0;
    f.text(
        Vec2::new(x, cursor),
        if game.size == 16 {
            "Arrows / 1-9,A-G / P notes / U undo"
        } else {
            "Arrows / 1-9 / P notes / U undo / H hint"
        },
        if mobile { 8.5 } else { 9.0 },
        MUTED,
    );
    cursor += 12.0;
    if given || (s.sudoku_ui.pencil && has_value) {
        f.text(
            Vec2::new(x, cursor),
            if given {
                "Given clue. Choose an empty cell."
            } else {
                "Erase this value before adding notes."
            },
            9.0,
            GOLD,
        );
        cursor += 12.0;
    }
    cursor
}

fn draw_message(f: &mut Frame, message: &str, x: f32, y: f32, width: usize) {
    for (i, line) in wrap_lines(message, width, 2).iter().enumerate() {
        f.text(Vec2::new(x, y + i as f32 * 13.0), line, 10.0, GOLD);
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
    let game = &s.save.sudoku.game;
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
    let selected_cell = selected.min(game.board.len().saturating_sub(1));
    let selected_mark = game.board.get(selected_cell).copied().unwrap_or(0) == value
        || (game.board.get(selected_cell).copied().unwrap_or(0) == 0
            && game
                .notes
                .get(selected_cell)
                .is_some_and(|notes| notes & (1u16 << (value - 1)) != 0));
    let enabled = !game.complete
        && game.givens.get(selected_cell).copied().unwrap_or(0) == 0
        && (!s.sudoku_ui.pencil || game.board[selected_cell] == 0);
    let focused = enabled && s.keyboard && s.focus == idx;
    let hovered = enabled && s.cursor.is_some_and(|p| r.contains(p));
    f.rect(
        r.min,
        r.size(),
        if !enabled {
            Color::srgb(0.018, 0.035, 0.05)
        } else if focused || hovered {
            Color::srgb(0.055, 0.14, 0.18)
        } else {
            PANEL
        },
    );
    f.outline(
        r.min,
        r.size(),
        if focused || selected_mark { 2.0 } else { 1.0 },
        if focused || selected_mark {
            GOLD
        } else if enabled {
            CYAN
        } else {
            EDGE
        },
    );
    if selected_mark {
        f.rect(r.min, Vec2::new(3.0, r.height()), GOLD);
    }
    let fs = if r.width() < 50.0 { 16.0 } else { 19.0 };
    f.text(
        Vec2::new(r.min.x + 6.0, r.min.y + (r.height() - fs) * 0.5),
        symbol(value),
        fs,
        if !enabled || count == 0 { MUTED } else { INK },
    );
    let count_font = if r.width() < 50.0 { 8.0 } else { 10.0 };
    let count_label = count.to_string();
    f.text(
        Vec2::new(
            r.max.x - 5.0 - count_label.len() as f32 * count_font * 0.6,
            r.min.y + r.height() - count_font - 3.0,
        ),
        count_label,
        count_font,
        if enabled { MUTED } else { EDGE },
    );
    if enabled {
        l.hits.push(Hit {
            rect: r,
            action: Action::Sudoku(Command::Enter {
                index: selected,
                value,
                pencil: s.sudoku_ui.pencil,
            }),
        });
    }
}

pub(super) fn setup_dialog(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32) {
    let width = (f.size.x - 2.0 * m).min(590.0);
    let height = (f.size.y - 32.0).min(500.0);
    let r = rect(
        (f.size.x - width) * 0.5,
        (f.size.y - height) * 0.5,
        width,
        height,
    );
    let inner_x = r.min.x + 22.0;
    let inner_w = width - 44.0;
    let gap = 8.0;
    let footer_y = r.max.y - 58.0;
    let bw = (inner_w - gap) * 0.5;
    let (draft_size, draft_difficulty) = s
        .sudoku_ui
        .setup
        .unwrap_or((s.save.sudoku.game.size, s.save.sudoku.game.difficulty));
    f.outline(
        r.min - Vec2::splat(6.0),
        r.size() + Vec2::splat(12.0),
        1.0,
        EDGE,
    );
    f.rect(r.min, r.size(), PANEL);
    f.rounded_outline(r.min, r.size(), 8.0, 2.0, GOLD);
    // Register Cancel first to make it the default modal keyboard focus.
    // Its footer region does not overlap the choices drawn afterward.
    control_button(
        f,
        s,
        l,
        rect(inner_x, footer_y, bw, 40.0),
        "CANCEL",
        Action::Cancel,
        true,
        false,
    );
    f.text(
        r.min + Vec2::new(22.0, 22.0),
        "NEW SUDOKU PUZZLE",
        if width < 400.0 { 23.0 } else { 28.0 },
        INK,
    );
    f.text(
        r.min + Vec2::new(22.0, 56.0),
        "Choose a board size and clue density.",
        12.0,
        MUTED,
    );
    f.text(r.min + Vec2::new(22.0, 87.0), "BOARD SIZE", 10.0, CYAN);
    let size_w = (inner_w - gap) * 0.5;
    for (i, size) in [4u8, 6, 9, 16].into_iter().enumerate() {
        let row = i / 2;
        let col = i % 2;
        let option = rect(
            inner_x + col as f32 * (size_w + gap),
            r.min.y + 108.0 + row as f32 * 49.0,
            size_w,
            42.0,
        );
        let label = if size == 16 {
            "16x16  EXPERIMENTAL".to_owned()
        } else {
            format!("{size}x{size}")
        };
        control_button(
            f,
            s,
            l,
            option,
            &label,
            Action::SudokuChooseSize(size),
            true,
            draft_size == size,
        );
    }
    f.text(r.min + Vec2::new(22.0, 216.0), "DIFFICULTY", 10.0, CYAN);
    for (i, difficulty) in Difficulty::ALL.into_iter().enumerate() {
        let row = i / 2;
        let col = i % 2;
        let option = rect(
            inner_x + col as f32 * (size_w + gap),
            r.min.y + 237.0 + row as f32 * 49.0,
            size_w,
            42.0,
        );
        control_button(
            f,
            s,
            l,
            option,
            difficulty.label(),
            Action::SudokuChooseDifficulty(difficulty),
            true,
            draft_difficulty == difficulty,
        );
    }
    f.text(
        r.min + Vec2::new(22.0, 350.0),
        if draft_size == 16 {
            "16x16 is experimental - hints unavailable."
        } else {
            "Difficulty controls clue density."
        },
        10.0,
        MUTED,
    );
    control_button(
        f,
        s,
        l,
        rect(inner_x + bw + gap, footer_y, bw, 40.0),
        if s.sudoku_ui.setup.is_some() {
            "START PUZZLE"
        } else {
            "NEW PUZZLE"
        },
        Action::SudokuStart,
        true,
        true,
    );
}

#[allow(clippy::too_many_arguments)]
fn control_button(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    label: &str,
    action: Action,
    enabled: bool,
    active: bool,
) {
    let idx = l.hits.len();
    let focused = enabled && s.keyboard && s.focus == idx;
    let hovered = enabled && s.cursor.is_some_and(|p| r.contains(p));
    f.rect(
        r.min,
        r.size(),
        if !enabled {
            Color::srgb(0.018, 0.035, 0.05)
        } else if focused || hovered {
            Color::srgb(0.055, 0.14, 0.18)
        } else {
            PANEL
        },
    );
    let edge = if !enabled {
        Color::srgb(0.045, 0.09, 0.12)
    } else if focused {
        GOLD
    } else if active {
        CYAN
    } else {
        EDGE
    };
    f.outline(r.min, r.size(), if focused { 2.0 } else { 1.0 }, edge);
    if active && enabled {
        f.rect(r.min, Vec2::new(3.0, r.height()), CYAN);
    }
    let font: f32 = if r.width() < 62.0 {
        8.5
    } else if r.width() < 100.0 {
        10.0
    } else {
        12.0
    };
    let fit = (r.width() - 8.0) / (label.chars().count().max(1) as f32 * 0.58);
    let font = font.min(fit.max(7.0));
    f.text(
        Vec2::new(
            r.min.x + (r.width() - label.chars().count() as f32 * font * 0.58).max(0.0) * 0.5,
            r.min.y + (r.height() - font) * 0.5,
        ),
        label,
        font,
        if !enabled {
            EDGE
        } else if focused || active {
            INK
        } else {
            Color::srgb(0.6, 0.78, 0.85)
        },
    );
    if enabled {
        l.hits.push(Hit { rect: r, action });
    }
}

fn hint_available(s: &Arcade) -> bool {
    let game = &s.save.sudoku.game;
    game.size != 16
        && !game.complete
        && !(0..game.board.len()).any(|i| game.conflict(i))
        && game.find_hint().is_some()
}

fn hint_reason(s: &Arcade) -> &'static str {
    let game = &s.save.sudoku.game;
    if game.size == 16 {
        "16x16 is experimental; hints unavailable."
    } else if game.complete {
        "Puzzle complete."
    } else if (0..game.board.len()).any(|i| game.conflict(i)) {
        "Resolve conflicts before asking for a hint."
    } else {
        "No safe single-step hint is available."
    }
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
