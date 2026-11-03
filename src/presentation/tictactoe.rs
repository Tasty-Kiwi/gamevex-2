//! Tic-tac-toe presentation rendering, board drawing, neon marks, and palette definitions.
use super::screens::{button, circle};
use super::*;
use gamevex_core::ColorScheme;
use gamevex_core::tictactoe::{Player, Status, WINNING_LINES};

/// Authored theme styling for Tic-tac-toe across all cabinet color schemes.
#[derive(Clone, Copy, Debug)]
pub struct TicTacToePalette {
    pub board_bg: Color,
    pub board_border: Color,
    pub grid_line: Color,
    pub cell_bg: Color,
    pub cell_hover: Color,
    pub cell_focus: Color,
    pub player_stroke: Color,
    pub player_fill: Color,
    pub cpu_stroke: Color,
    pub cpu_fill: Color,
    pub win_line_player: Color,
    pub win_line_cpu: Color,
}

impl TicTacToePalette {
    pub fn for_scheme(scheme: ColorScheme) -> Self {
        let shared = super::backgammon::BackgammonPalette::for_scheme(scheme);
        Self {
            board_bg: shared.wood,
            board_border: shared.white_team,
            grid_line: shared.point_a,
            cell_bg: shared.tray,
            cell_hover: shared.point_a.with_alpha(0.35),
            cell_focus: shared.white_team,
            player_stroke: shared.white_team,
            player_fill: shared.white_body,
            cpu_stroke: shared.black_team,
            cpu_fill: shared.black_body,
            win_line_player: shared.white_team,
            win_line_cpu: shared.black_team,
        }
    }
}

/// Helper to render a neon X mark with glow underlay and crisp core stroke.
fn draw_x(
    f: &mut Frame,
    center: Vec2,
    r: f32,
    stroke_w: f32,
    stroke_color: Color,
    fill_color: Color,
) {
    let d = r * 0.72;
    // Glow underlay
    f.line(
        center + Vec2::new(-d, -d),
        center + Vec2::new(d, d),
        stroke_w * 2.2,
        fill_color,
    );
    f.line(
        center + Vec2::new(d, -d),
        center + Vec2::new(-d, d),
        stroke_w * 2.2,
        fill_color,
    );
    // Core neon line
    f.line(
        center + Vec2::new(-d, -d),
        center + Vec2::new(d, d),
        stroke_w,
        stroke_color,
    );
    f.line(
        center + Vec2::new(d, -d),
        center + Vec2::new(-d, d),
        stroke_w,
        stroke_color,
    );
}

// Joined ring segments keep thick O strokes smooth at every board size.
fn ring(f: &mut Frame, center: Vec2, radius: f32, width: f32, color: Color) {
    let outer = radius + width * 0.5;
    let inner = (radius - width * 0.5).max(0.0);
    for segment in 0..64 {
        let angle = segment as f32 * std::f32::consts::TAU / 64.0;
        let next = (segment + 1) as f32 * std::f32::consts::TAU / 64.0;
        let a = Vec2::new(angle.cos(), angle.sin());
        let b = Vec2::new(next.cos(), next.sin());
        f.polygon(
            &[
                center + a * inner,
                center + a * outer,
                center + b * outer,
                center + b * inner,
            ],
            color,
        );
    }
}

/// Helper to render a neon O mark with glow underlay and crisp core ring.
fn draw_o(
    f: &mut Frame,
    center: Vec2,
    r: f32,
    stroke_w: f32,
    stroke_color: Color,
    fill_color: Color,
) {
    // Glow underlay ring
    ring(f, center, r, stroke_w * 2.2, fill_color);
    // Core neon ring
    ring(f, center, r, stroke_w, stroke_color);
}

/// Renders the complete Tic-tac-toe cabinet screen.
pub(super) fn draw(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let palette = TicTacToePalette::for_scheme(f.palette);
    let theme = *f.theme();
    let game = &s.save.tictactoe;
    let small = w < 620.0;
    let narrow = w < 360.0;

    // Header title and shared controls
    let top = if small { 86.0 } else { 94.0 };
    let title_size = if narrow {
        18.0
    } else if small {
        22.0
    } else {
        30.0
    };
    f.text(Vec2::new(m, top), "TIC-TAC-TOE", title_size, theme.ink);

    let control_h = if small { 36.0 } else { 34.0 };
    let new_w = if narrow {
        80.0
    } else if small {
        90.0
    } else {
        110.0
    };
    let new_rect = rect(m + w - new_w, top - 2.0, new_w, control_h);
    button(f, s, l, new_rect, "New game", Action::TicTacToeNew, false);

    // Status message and turn badge
    let status_y = top + control_h + if small { 8.0 } else { 12.0 };
    let badge_center = Vec2::new(m + 10.0, status_y + 8.0);
    let (status_text, status_color, badge_player) = match game.status {
        Status::Playing if game.turn == Player::Cpu => (
            "Computer thinking...",
            palette.cpu_stroke,
            Some(Player::Cpu),
        ),
        Status::Playing => ("Your turn", palette.player_stroke, Some(Player::Human)),
        Status::Won(Player::Human) => ("You win! Three in a row", theme.gold, Some(Player::Human)),
        Status::Won(Player::Cpu) => (
            "Computer wins! Three in a row",
            theme.danger,
            Some(Player::Cpu),
        ),
        Status::Draw => ("Draw — Board is full", theme.muted, None),
    };

    match badge_player {
        Some(Player::Human) => {
            draw_x(
                f,
                badge_center,
                6.0,
                1.8,
                palette.player_stroke,
                palette.player_fill,
            );
        }
        Some(Player::Cpu) => {
            draw_o(
                f,
                badge_center,
                6.0,
                1.8,
                palette.cpu_stroke,
                palette.cpu_fill,
            );
        }
        None => {
            circle(f, badge_center, 6.0, 1.2, theme.muted);
        }
    }

    f.text(
        Vec2::new(m + 24.0, status_y + 1.0),
        status_text,
        if narrow {
            11.0
        } else if small {
            12.0
        } else {
            14.0
        },
        status_color,
    );

    // Difficulty button row (placed between status and board)
    let difficulty_y = status_y + if small { 22.0 } else { 26.0 };
    let difficulty_h = 34.0;
    let difficulty_w = w.min(600.0);
    let difficulty_x = m;
    let difficulty_rect = Rect::from_corners(
        Vec2::new(difficulty_x, difficulty_y),
        Vec2::new(difficulty_x + difficulty_w, difficulty_y + difficulty_h),
    );

    // Responsive 3x3 board dimensions
    let board_top = difficulty_rect.max.y + if small { 10.0 } else { 14.0 };
    let bottom_limit = (f.size.y - 36.0).max(board_top + 160.0);
    let available_h = bottom_limit - board_top;
    let max_cell_w = ((w - 16.0) / 3.0).floor();
    let max_cell_h = (available_h / 3.0).floor();
    let cell = max_cell_w.min(max_cell_h).clamp(44.0, 120.0);

    let board_w = cell * 3.0;
    let board_h = cell * 3.0;
    let board_x = m + (w - board_w) * 0.5;

    // Cabinet board surface
    let board_rect = Rect::from_corners(
        Vec2::new(board_x, board_top),
        Vec2::new(board_x + board_w, board_top + board_h),
    );
    f.rounded_rect(board_rect, 12.0, palette.board_bg);
    f.rounded_outline(
        board_rect.min,
        board_rect.size(),
        12.0,
        2.0,
        palette.board_border,
    );

    let mark_r = cell * 0.28;
    let stroke_w = (cell * 0.065).clamp(2.5, 5.5);

    // Register 3x3 cells in row-major order: cells 0..8
    // Hit index sequence: Logo (0), Settings (1), New (2), cells 0..8 (3..11), center focus is 7.
    for i in 0..9 {
        let r = i / 3;
        let c = i % 3;
        let cell_x = board_x + c as f32 * cell;
        let cell_y = board_top + r as f32 * cell;
        let cell_rect = Rect::from_corners(
            Vec2::new(cell_x, cell_y),
            Vec2::new(cell_x + cell, cell_y + cell),
        );
        let cell_inner = Rect::from_corners(
            Vec2::new(cell_x + 4.0, cell_y + 4.0),
            Vec2::new(cell_x + cell - 4.0, cell_y + cell - 4.0),
        );
        let center = Vec2::new(cell_x + cell * 0.5, cell_y + cell * 0.5);

        let hit_idx = l.hits.len();
        l.hits.push(Hit {
            rect: cell_rect,
            action: Action::TicTacToeCell(i),
        });

        let is_hovered = s.cursor.is_some_and(|p| cell_rect.contains(p));
        let is_focused = s.keyboard && s.focus == hit_idx;
        let can_interact =
            game.status == Status::Playing && game.turn == Player::Human && game.legal(i);

        // Cell background
        f.rounded_rect(cell_inner, 8.0, palette.cell_bg);

        if (is_hovered || is_focused) && can_interact {
            f.rounded_rect(cell_inner, 8.0, palette.cell_hover);
        }

        if is_focused {
            f.rounded_outline(
                cell_inner.min,
                cell_inner.size(),
                8.0,
                2.0,
                palette.cell_focus,
            );
        }

        let cell_val = game.board[i];
        if cell_val == 1 {
            draw_x(
                f,
                center,
                mark_r,
                stroke_w,
                palette.player_stroke,
                palette.player_fill,
            );
        } else if cell_val == 2 {
            draw_o(
                f,
                center,
                mark_r,
                stroke_w,
                palette.cpu_stroke,
                palette.cpu_fill,
            );
        } else {
            // Empty cell: subtle row-order key hint in corner
            f.text(
                Vec2::new(cell_x + 8.0, cell_y + 16.0),
                (i + 1).to_string(),
                10.0,
                theme.muted.with_alpha(0.3),
            );
            // Hover preview
            if is_hovered && can_interact {
                draw_x(
                    f,
                    center,
                    mark_r,
                    stroke_w * 0.8,
                    palette.player_stroke.with_alpha(0.35),
                    palette.player_fill.with_alpha(0.2),
                );
            }
        }
    }

    // Grid divider lines
    for idx in 1..3 {
        let vx = board_x + idx as f32 * cell;
        f.line(
            Vec2::new(vx, board_top + 8.0),
            Vec2::new(vx, board_top + board_h - 8.0),
            2.0,
            palette.grid_line,
        );
        let hy = board_top + idx as f32 * cell;
        f.line(
            Vec2::new(board_x + 8.0, hy),
            Vec2::new(board_x + board_w - 8.0, hy),
            2.0,
            palette.grid_line,
        );
    }

    // Show winning highlights and line across winning combination
    if let Status::Won(winner) = game.status {
        let win_color = if winner == Player::Human {
            palette.win_line_player
        } else {
            palette.win_line_cpu
        };

        for idx in game.winning_cells() {
            let r = idx / 3;
            let c = idx % 3;
            let c_center = Vec2::new(
                board_x + (c as f32 + 0.5) * cell,
                board_top + (r as f32 + 0.5) * cell,
            );
            circle(f, c_center, mark_r + 6.0, 2.5, win_color);
        }

        let value = match winner {
            Player::Human => 1,
            Player::Cpu => 2,
        };
        for line in WINNING_LINES {
            if line.iter().all(|&idx| game.board[idx] == value) {
                let point = |idx: usize| {
                    Vec2::new(
                        board_x + ((idx % 3) as f32 + 0.5) * cell,
                        board_top + ((idx / 3) as f32 + 0.5) * cell,
                    )
                };
                let p0 = point(line[0]);
                let p2 = point(line[2]);
                let dir = (p2 - p0).normalize();
                let ext = dir * (cell * 0.25);
                let start = p0 - ext;
                let end = p2 + ext;
                f.line(start, end, 7.0, win_color.with_alpha(0.35));
                f.line(start, end, 3.5, win_color);
            }
        }
    }

    screens::difficulty_buttons(
        f,
        s,
        l,
        difficulty_rect,
        game.difficulty,
        Action::TicTacToeDifficulty(game.difficulty),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_available_for_all_schemes() {
        for scheme in [
            ColorScheme::Cyan,
            ColorScheme::Amber,
            ColorScheme::Emerald,
            ColorScheme::Violet,
            ColorScheme::Paper,
        ] {
            let p = TicTacToePalette::for_scheme(scheme);
            let reference = super::super::backgammon::BackgammonPalette::for_scheme(scheme);
            assert_eq!(p.player_stroke, reference.white_team);
            assert_eq!(p.player_fill, reference.white_body);
            assert_eq!(p.cpu_stroke, reference.black_team);
            assert_eq!(p.cpu_fill, reference.black_body);
            for color in [
                p.board_bg,
                p.board_border,
                p.grid_line,
                p.cell_bg,
                p.player_stroke,
                p.player_fill,
                p.cpu_stroke,
                p.cpu_fill,
                p.win_line_player,
                p.win_line_cpu,
            ] {
                assert!(
                    color
                        .to_linear()
                        .to_f32_array()
                        .iter()
                        .all(|c| c.is_finite())
                );
            }
        }
    }
}
