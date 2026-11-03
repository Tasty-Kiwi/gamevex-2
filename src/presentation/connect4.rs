//! Connect 4 presentation rendering, board drawing, animations, and palette definitions.
use super::screens::{button, circle};
use super::*;
use gamevex_core::ColorScheme;
use gamevex_core::connect4::{COLS, Player, ROWS, Status};

/// Authored theme styling for Connect 4 across all cabinet color schemes.
#[derive(Clone, Copy, Debug)]
pub struct Connect4Palette {
    pub board_bg: Color,
    pub board_border: Color,
    pub hole_bg: Color,
    pub hole_edge: Color,
    pub red_disc: Color,
    pub red_rim: Color,
    pub yellow_disc: Color,
    pub yellow_rim: Color,
    pub win_line: Color,
    pub col_hover: Color,
    pub drop_marker: Color,
}

impl Connect4Palette {
    pub fn for_scheme(scheme: ColorScheme) -> Self {
        let shared = super::backgammon::BackgammonPalette::for_scheme(scheme);
        Self {
            board_bg: shared.wood,
            board_border: shared.white_team,
            hole_bg: shared.tray,
            hole_edge: shared.point_a,
            red_disc: shared.white_body,
            red_rim: shared.white_team,
            yellow_disc: shared.black_body,
            yellow_rim: shared.black_team,
            win_line: shared.white_team,
            col_hover: shared.point_a.with_alpha(0.3),
            drop_marker: shared.white_team,
        }
    }
}

/// Helper to render a filled circular disk.
fn disk(f: &mut Frame, center: Vec2, r: f32, color: Color) {
    let points: Vec<_> = (0..36)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 36.0;
            center + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect();
    f.polygon(&points, color);
}

/// Use the same shaded body, thin rims, and shadow as Backgammon's checkers.
fn draw_disc(f: &mut Frame, center: Vec2, r: f32, player: Player, palette: &Connect4Palette) {
    let (disc, rim) = match player {
        Player::Red => (palette.red_disc, palette.red_rim),
        Player::Yellow => (palette.yellow_disc, palette.yellow_rim),
    };
    let shadow = super::backgammon::BackgammonPalette::for_scheme(f.palette).shadow;
    super::backgammon::checker_piece(f, center, r, disc, rim, shadow);
}

/// Renders the complete Connect 4 cabinet screen.
pub(super) fn draw(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let palette = Connect4Palette::for_scheme(f.palette);
    let theme = *f.theme();
    let game = &s.save.connect4;
    let ui = &s.connect4_ui;
    let busy = ui.busy(s.save.settings.reduced_motion);
    let small = w < 620.0;

    // Header title and shared controls
    let top = if small { 86.0 } else { 94.0 };
    let title_size = if small { 22.0 } else { 30.0 };
    f.text(Vec2::new(m, top), "CONNECT 4", title_size, theme.ink);

    let control_h = if small { 36.0 } else { 34.0 };
    let new_w = if small { 90.0 } else { 110.0 };
    let new_rect = rect(m + w - new_w, top - 2.0, new_w, control_h);
    button(f, s, l, new_rect, "New game", Action::Connect4New, false);

    // Status message and turn badge
    let status_y = top + control_h + if small { 8.0 } else { 12.0 };
    let badge_center = Vec2::new(m + 10.0, status_y + 8.0);
    let (status_text, status_color, badge_player) = match game.status {
        Status::Playing if game.turn == Player::Yellow => (
            "Computer thinking...",
            palette.yellow_rim,
            Some(Player::Yellow),
        ),
        Status::Playing => ("Your turn", palette.red_rim, Some(Player::Red)),
        Status::Won(Player::Red) => ("You win! Four in a row", theme.gold, Some(Player::Red)),
        Status::Won(Player::Yellow) => (
            "Computer wins! Four in a row",
            theme.danger,
            Some(Player::Yellow),
        ),
        Status::Draw => ("Draw — Board is full", theme.muted, None),
    };

    if let Some(player) = badge_player {
        draw_disc(f, badge_center, 7.5, player, &palette);
    } else {
        circle(f, badge_center, 7.5, 1.2, theme.muted);
    }

    f.text(
        Vec2::new(m + 24.0, status_y + 1.0),
        status_text,
        if small { 12.0 } else { 14.0 },
        status_color,
    );

    // Responsive board dimensions
    let board_top = status_y + 46.0;
    let bottom_limit = (f.size.y - 42.0).max(board_top + 180.0);
    let available_h = bottom_limit - board_top;
    let max_cell_w = (w / COLS as f32).floor();
    let max_cell_h = (available_h / ROWS as f32).floor();
    let cell = max_cell_w.min(max_cell_h).clamp(34.0, 72.0);

    let board_w = cell * COLS as f32;
    let board_h = cell * ROWS as f32;
    let board_x = m + (w - board_w) * 0.5;

    let is_animating =
        !s.save.settings.reduced_motion && ui.falling.is_some() && ui.fall_age < ui.fall_duration;

    // Connect 4 cabinet board surface
    let board_rect = Rect::from_corners(
        Vec2::new(board_x, board_top),
        Vec2::new(board_x + board_w, board_top + board_h),
    );
    f.rounded_rect(board_rect, 10.0, palette.board_bg);
    f.rounded_outline(
        board_rect.min,
        board_rect.size(),
        10.0,
        2.0,
        palette.board_border,
    );

    // Register hit regions per column and draw hover/focus markers
    for col in 0..COLS {
        let col_x = board_x + col as f32 * cell;
        let col_rect = Rect::from_corners(
            Vec2::new(col_x, board_top - 20.0),
            Vec2::new(col_x + cell, board_top + board_h),
        );
        let hit_idx = l.hits.len();
        l.hits.push(Hit {
            rect: col_rect,
            action: Action::Connect4Drop(col),
        });

        let is_hovered = s.cursor.is_some_and(|p| col_rect.contains(p));
        let is_focused = s.keyboard && s.focus == hit_idx;
        let is_selected = is_focused;

        let can_interact = game.status == Status::Playing
            && !busy
            && game.turn != Player::Yellow
            && game.legal(col);

        if (is_hovered || is_selected) && can_interact {
            // Column hover highlight strip
            f.rect(
                Vec2::new(col_x, board_top),
                Vec2::new(cell, board_h),
                palette.col_hover,
            );

            // Downward drop indicator arrow above column
            let arrow_center = Vec2::new(col_x + cell * 0.5, board_top - 10.0);
            let arrow_color = if is_focused {
                palette.drop_marker
            } else if game.turn == Player::Red {
                palette.red_rim
            } else {
                palette.yellow_rim
            };
            f.polygon(
                &[
                    Vec2::new(arrow_center.x - 6.0, arrow_center.y - 5.0),
                    Vec2::new(arrow_center.x + 6.0, arrow_center.y - 5.0),
                    Vec2::new(arrow_center.x, arrow_center.y + 4.0),
                ],
                arrow_color,
            );

            if is_focused {
                f.outline(
                    Vec2::new(col_x + 1.0, board_top - 18.0),
                    Vec2::new(cell - 2.0, 16.0),
                    1.5,
                    theme.gold,
                );
            }
        }
    }

    let disc_r = cell * 0.38;

    // Draw grid holes and settled discs
    for r in 0..ROWS {
        for c in 0..COLS {
            let center = Vec2::new(
                board_x + (c as f32 + 0.5) * cell,
                board_top + (r as f32 + 0.5) * cell,
            );

            // Empty hole background
            disk(f, center, disc_r, palette.hole_bg);
            circle(f, center, disc_r, 1.2, palette.hole_edge);

            let is_falling_cell = is_animating
                && ui
                    .falling
                    .as_ref()
                    .is_some_and(|fall| fall.column == c && fall.target_row == r);

            if !is_falling_cell {
                let cell_val = game.board[r * COLS + c];
                if cell_val == 1 {
                    draw_disc(f, center, disc_r, Player::Red, &palette);
                } else if cell_val == 2 {
                    draw_disc(f, center, disc_r, Player::Yellow, &palette);
                }
            }
        }
    }

    // Draw active falling disc in motion
    if is_animating && let Some(fall) = ui.falling.as_ref() {
        let progress = (ui.fall_age / ui.fall_duration).clamp(0.0, 1.0);
        let ease = progress * progress; // quadratic gravity acceleration
        let current_row = fall.target_row as f32 * ease;
        let animated_center = Vec2::new(
            board_x + (fall.column as f32 + 0.5) * cell,
            board_top + (current_row + 0.5) * cell,
        );
        draw_disc(f, animated_center, disc_r, fall.player, &palette);
    }

    // Show results after the final disc lands. Multiple simultaneous lines are
    // rendered independently so highlights never connect unrelated endpoints.
    if !is_animating && let Status::Won(winner) = game.status {
        let value = if winner == Player::Red { 1 } else { 2 };
        for idx in game.winning_cells() {
            let center = Vec2::new(
                board_x + ((idx % COLS) as f32 + 0.5) * cell,
                board_top + ((idx / COLS) as f32 + 0.5) * cell,
            );
            circle(f, center, disc_r + 2.5, 2.5, palette.win_line);
        }
        for line in gamevex_core::connect4::WINNING_LINES {
            if line.iter().all(|&idx| game.board[idx] == value) {
                let point = |idx: usize| {
                    Vec2::new(
                        board_x + ((idx % COLS) as f32 + 0.5) * cell,
                        board_top + ((idx / COLS) as f32 + 0.5) * cell,
                    )
                };
                f.line(point(line[0]), point(line[3]), 3.0, palette.win_line);
            }
        }
    }
    for col in 0..COLS {
        f.centered_text(
            Vec2::new(
                board_x + (col as f32 + 0.5) * cell,
                board_top + board_h + 10.0,
            ),
            (col + 1).to_string(),
            12.0,
            theme.muted,
        );
    }
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
            let p = Connect4Palette::for_scheme(scheme);
            let reference = super::super::backgammon::BackgammonPalette::for_scheme(scheme);
            assert_eq!(p.red_rim, reference.white_team);
            assert_eq!(p.red_disc, reference.white_body);
            assert_eq!(p.yellow_rim, reference.black_team);
            assert_eq!(p.yellow_disc, reference.black_body);
            for color in [p.board_bg, p.red_disc, p.yellow_disc, p.win_line] {
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
