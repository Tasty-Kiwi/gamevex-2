use super::screens::{button, circle};
use super::*;
use gamevex_core::backgammon::{BAR, Color as Side, OFF, Phase};

#[derive(Clone, Copy)]
pub struct BackgammonPalette {
    pub felt: Color,
    pub wood: Color,
    pub bar: Color,
    pub tray: Color,
    pub point_a: Color,
    pub point_b: Color,
    pub white_team: Color,
    pub black_team: Color,
    pub white_body: Color,
    pub black_body: Color,
    pub shadow: Color,
    pub dialog_panel: Color,
}

impl BackgammonPalette {
    pub fn for_scheme(scheme: gamevex_core::ColorScheme) -> Self {
        match scheme {
            gamevex_core::ColorScheme::Cyan => Self {
                felt: Color::srgb(0.018, 0.047, 0.067),
                wood: Color::srgb(0.042, 0.075, 0.09),
                bar: Color::srgb(0.023, 0.042, 0.06),
                tray: Color::srgb(0.013, 0.031, 0.048),
                point_a: Color::srgb(0.075, 0.15, 0.18),
                point_b: Color::srgb(0.13, 0.08, 0.17),
                white_team: Color::srgb(0.0, 0.83, 1.0),
                black_team: Color::srgb(1.0, 0.0, 0.67),
                white_body: Color::srgb(0.025, 0.21, 0.29),
                black_body: Color::srgb(0.26, 0.015, 0.15),
                shadow: Color::srgb(0.003, 0.008, 0.014),
                dialog_panel: Color::srgba(0.008, 0.016, 0.031, 0.94),
            },
            gamevex_core::ColorScheme::Amber => Self {
                felt: Color::srgb(0.05, 0.034, 0.020),
                wood: Color::srgb(0.09, 0.06, 0.035),
                bar: Color::srgb(0.065, 0.045, 0.025),
                tray: Color::srgb(0.04, 0.027, 0.016),
                point_a: Color::srgb(0.16, 0.11, 0.05),
                point_b: Color::srgb(0.17, 0.07, 0.06),
                white_team: Color::srgb(1.0, 0.76, 0.20),
                black_team: Color::srgb(1.0, 0.36, 0.25),
                white_body: Color::srgb(0.24, 0.15, 0.04),
                black_body: Color::srgb(0.24, 0.06, 0.05),
                shadow: Color::srgb(0.015, 0.010, 0.005),
                dialog_panel: Color::srgba(0.04, 0.028, 0.016, 0.94),
            },
            gamevex_core::ColorScheme::Emerald => Self {
                felt: Color::srgb(0.018, 0.048, 0.032),
                wood: Color::srgb(0.035, 0.080, 0.052),
                bar: Color::srgb(0.024, 0.055, 0.038),
                tray: Color::srgb(0.014, 0.035, 0.024),
                point_a: Color::srgb(0.05, 0.16, 0.11),
                point_b: Color::srgb(0.14, 0.09, 0.07),
                white_team: Color::srgb(0.35, 0.95, 0.70),
                black_team: Color::srgb(1.0, 0.48, 0.36),
                white_body: Color::srgb(0.035, 0.24, 0.15),
                black_body: Color::srgb(0.28, 0.08, 0.07),
                shadow: Color::srgb(0.005, 0.012, 0.008),
                dialog_panel: Color::srgba(0.016, 0.038, 0.026, 0.94),
            },
            gamevex_core::ColorScheme::Violet => Self {
                felt: Color::srgb(0.038, 0.022, 0.052),
                wood: Color::srgb(0.075, 0.045, 0.095),
                bar: Color::srgb(0.050, 0.030, 0.068),
                tray: Color::srgb(0.028, 0.016, 0.040),
                point_a: Color::srgb(0.14, 0.08, 0.18),
                point_b: Color::srgb(0.05, 0.14, 0.15),
                white_team: Color::srgb(0.82, 0.58, 1.0),
                black_team: Color::srgb(0.25, 0.95, 0.88),
                white_body: Color::srgb(0.22, 0.08, 0.28),
                black_body: Color::srgb(0.04, 0.22, 0.22),
                shadow: Color::srgb(0.012, 0.006, 0.016),
                dialog_panel: Color::srgba(0.032, 0.018, 0.044, 0.94),
            },
            gamevex_core::ColorScheme::Ice => Self {
                felt: Color::srgb(0.016, 0.032, 0.060),
                wood: Color::srgb(0.038, 0.065, 0.105),
                bar: Color::srgb(0.024, 0.045, 0.078),
                tray: Color::srgb(0.012, 0.024, 0.045),
                point_a: Color::srgb(0.06, 0.13, 0.22),
                point_b: Color::srgb(0.14, 0.08, 0.12),
                white_team: Color::srgb(0.50, 0.88, 1.0),
                black_team: Color::srgb(1.0, 0.45, 0.62),
                white_body: Color::srgb(0.06, 0.18, 0.32),
                black_body: Color::srgb(0.26, 0.06, 0.14),
                shadow: Color::srgb(0.004, 0.008, 0.016),
                dialog_panel: Color::srgba(0.014, 0.026, 0.048, 0.94),
            },
            gamevex_core::ColorScheme::Paper => Self {
                felt: Color::srgb(0.93, 0.91, 0.86),
                wood: Color::srgb(0.85, 0.82, 0.76),
                bar: Color::srgb(0.78, 0.75, 0.68),
                tray: Color::srgb(0.88, 0.86, 0.80),
                point_a: Color::srgb(0.84, 0.72, 0.74),
                point_b: Color::srgb(0.72, 0.80, 0.74),
                white_team: Color::srgb(0.65, 0.08, 0.18),
                black_team: Color::srgb(0.12, 0.14, 0.16),
                white_body: Color::srgb(0.96, 0.94, 0.90),
                black_body: Color::srgb(0.22, 0.24, 0.26),
                shadow: Color::srgb(0.65, 0.62, 0.56),
                dialog_panel: Color::srgba(0.95, 0.93, 0.89, 0.98),
            },
        }
    }
}

pub(super) fn backgammon(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let palette = BackgammonPalette::for_scheme(f.palette);
    let theme = f.theme();
    let game = s.save.backgammon.clone();
    s.backgammon_ui.refresh(&game);
    let ui = &s.backgammon_ui;
    let busy = ui.busy(s.save.settings.reduced_motion);
    let mut display_board = game.board.clone();
    if !s.save.settings.reduced_motion
        && let Some(motion) = ui.motion.as_ref()
    {
        display_board = motion.before.clone();
        if motion.movement.from == BAR {
            display_board.bar[motion.color.index()] =
                display_board.bar[motion.color.index()].saturating_sub(1);
        } else if motion.movement.from < 24 {
            let p = &mut display_board.points[motion.movement.from as usize];
            p.count = p.count.saturating_sub(1);
            if p.count == 0 {
                p.color = None;
            }
        }
        if motion.movement.is_hit && motion.movement.to < 24 {
            let p = &mut display_board.points[motion.movement.to as usize];
            p.count = p.count.saturating_sub(1);
            if p.count == 0 {
                p.color = None;
            }
        }
    }
    let small = w < 620.0;
    let short = f.size.y <= 600.0;
    f.text(
        Vec2::new(m, 82.0),
        "Backgammon",
        if small { 25.0 } else { 34.0 },
        theme.ink,
    );
    let top = if small { 126.0 } else { 94.0 };
    let control_h = if short { 29.0 } else { 34.0 };
    let gap = if small { 5.0 } else { 8.0 };
    let control_widths = if small {
        [45.0, 52.0, 71.0]
    } else {
        [64.0, 68.0, 105.0]
    };
    let total: f32 = control_widths.iter().sum::<f32>() + gap * 2.0;
    let mut x = (m + w - total).max(m);
    for (i, (label, action)) in [
        ("Roll", Action::BackgammonRoll),
        (
            if game.difficulty == gamevex_core::backgammon::Difficulty::Easy {
                "Easy"
            } else {
                "Hard"
            },
            Action::BackgammonDifficulty,
        ),
        ("New", Action::BackgammonNew),
    ]
    .into_iter()
    .enumerate()
    {
        let enabled = match i {
            0 => game.phase == Phase::AwaitRoll && game.turn == Side::White && !busy,
            _ => true,
        };
        let control_rect = rect(x, top, control_widths[i], control_h);
        if i == 0 && !enabled {
            f.rect(control_rect.min, control_rect.size(), theme.panel);
            f.outline(control_rect.min, control_rect.size(), 1.0, theme.edge);
            centered(
                f,
                Vec2::new(control_rect.center().x, control_rect.min.y + 7.0),
                label,
                12.0,
                theme.muted,
            );
        } else {
            button(f, s, l, control_rect, label, action, enabled);
        }
        x += control_widths[i] + gap;
    }

    let stat_y = top + control_h + if small { 8.0 } else { 5.0 };
    let stat_size = if small { 10.0 } else { 13.0 };
    f.text(
        Vec2::new(m + 3.0, stat_y),
        format!(
            "YOU  {} pips  /  {} off",
            game.board.pip_count(Side::White),
            game.board.off[0]
        ),
        stat_size,
        palette.white_team,
    );
    let right_stat = format!(
        "CPU  {} pips  /  {} off",
        game.board.pip_count(Side::Black),
        game.board.off[1]
    );
    f.text(
        Vec2::new(
            m + w - right_stat.chars().count() as f32 * stat_size * 0.57,
            stat_y,
        ),
        right_stat,
        stat_size,
        palette.black_team,
    );

    let board_top = stat_y + stat_size + 6.0;
    let board_bottom = f.size.y - if short { 85.0 } else { 90.0 };
    let board_h = (board_bottom - board_top).max(100.0);
    let board_x = m;
    let board_w = w;
    let table = rect(board_x, board_top, board_w, board_h);
    f.rounded_rect(table, 9.0, palette.wood);
    f.rounded_outline(table.min, table.size(), 9.0, 1.0, theme.edge);
    let pad = if small { 6.0 } else { 12.0 };
    let tray_w = if small { 24.0 } else { 37.0 };
    let bar_w = if small { 22.0 } else { 62.0 };
    let inner_x = board_x + pad;
    let inner_w = (board_w - pad * 2.0 - tray_w - bar_w).max(80.0);
    let half_w = inner_w * 0.5;
    let col_w = half_w / 6.0;
    let bar_x = inner_x + half_w;
    let tray_x = bar_x + bar_w + half_w;
    let mid_y = board_top + board_h * 0.5;
    let edge_y = if short { 26.0 } else { board_h * 0.13 }.min(board_h * 0.22);
    let tri_h = (board_h * 0.39).max(30.0);
    let checker_r = ((col_w * 0.43).min((board_h * 0.052).max(7.5))).max(6.0);
    let step = checker_r * 1.12;

    f.rounded_rect(
        rect(inner_x, board_top + 3.0, inner_w, board_h - 6.0),
        5.0,
        palette.felt,
    );
    f.rect(
        rect(bar_x, board_top + 4.0, bar_w, board_h - 8.0).min,
        Vec2::new(bar_w, board_h - 8.0),
        palette.bar,
    );
    f.line(
        Vec2::new(bar_x + bar_w * 0.5, board_top + 5.0),
        Vec2::new(bar_x + bar_w * 0.5, board_bottom - 5.0),
        1.0,
        theme.edge,
    );
    // The point sequence follows conventional numbering: 12..17 across the
    // upper left, 18..23 back toward the right; 11..6 then 0..5 below.
    let mut slots = [Rect::default(); 26];
    for col in 0..12 {
        let left_side = col < 6;
        let local_col = if left_side { col } else { col - 6 };
        let px = if left_side {
            inner_x + local_col as f32 * col_w
        } else {
            bar_x + bar_w + local_col as f32 * col_w
        };
        let top_point = if left_side {
            12 + local_col
        } else {
            18 + local_col
        };
        let bottom_point = if left_side {
            11 - local_col
        } else {
            5 - local_col
        };
        let top_rect = rect(px, board_top + 2.0, col_w, mid_y - board_top - 2.0);
        let bottom_rect = rect(px, mid_y, col_w, board_bottom - mid_y - 2.0);
        slots[top_point] = top_rect;
        slots[bottom_point] = bottom_rect;
        let c = if col % 2 == 0 {
            palette.point_a
        } else {
            palette.point_b
        };
        f.polygon(
            &[
                Vec2::new(px + col_w * 0.10, board_top + edge_y),
                Vec2::new(px + col_w * 0.90, board_top + edge_y),
                Vec2::new(px + col_w * 0.5, board_top + tri_h),
            ],
            c,
        );
        f.polygon(
            &[
                Vec2::new(px + col_w * 0.10, board_bottom - edge_y),
                Vec2::new(px + col_w * 0.90, board_bottom - edge_y),
                Vec2::new(px + col_w * 0.5, board_bottom - tri_h),
            ],
            if col % 2 == 0 {
                palette.point_b
            } else {
                palette.point_a
            },
        );
        f.text(
            Vec2::new(px + col_w * 0.5 - 4.0, board_top + 4.0),
            format!("{}", top_point + 1),
            if small { 7.0 } else { 9.0 },
            theme.muted,
        );
        f.text(
            Vec2::new(px + col_w * 0.5 - 4.0, board_bottom - 14.0),
            format!("{}", bottom_point + 1),
            if small { 7.0 } else { 9.0 },
            theme.muted,
        );
    }
    slots[BAR as usize] = rect(bar_x, mid_y, bar_w, board_bottom - mid_y);
    slots[OFF as usize] = rect(tray_x, mid_y, tray_w, board_bottom - mid_y);
    f.rounded_rect(
        rect(tray_x, board_top + 4.0, tray_w - 2.0, board_h - 8.0),
        4.0,
        palette.tray,
    );
    f.line(
        Vec2::new(tray_x + 2.0, mid_y),
        Vec2::new(tray_x + tray_w - 4.0, mid_y),
        1.0,
        theme.edge,
    );
    let legal = if game.turn == Side::White && game.phase == Phase::Moving {
        ui.legal.clone()
    } else {
        vec![]
    };
    let selected = ui.selected;
    let source = selected;
    let destinations: Vec<u8> = legal
        .iter()
        .filter(|mv| Some(mv.from) == source)
        .map(|mv| mv.to)
        .collect();
    let moves_available = game.turn == Side::White && game.phase == Phase::Moving && !busy;
    // Keep every slot in diagnostics; the parent filters eligible IDs when
    // exporting click targets. Inactive slots also block controls underneath.
    for (idx, r) in slots.iter().copied().enumerate() {
        let focused = moves_available && s.keyboard && s.focus == l.hits.len();
        let hovered = moves_available && s.cursor.is_some_and(|point| r.contains(point));
        if moves_available && legal.iter().any(|mv| mv.from == idx as u8) {
            f.rounded_outline(r.min, r.size(), 4.0, 2.0, theme.accent);
        }
        if source == Some(idx as u8) {
            f.rounded_outline(
                r.min + Vec2::splat(1.0),
                r.size() - Vec2::splat(2.0),
                4.0,
                2.0,
                theme.gold,
            );
        } else if destinations.contains(&(idx as u8)) {
            f.rounded_outline(
                r.min + Vec2::splat(2.0),
                r.size() - Vec2::splat(4.0),
                4.0,
                2.0,
                theme.secondary,
            );
        } else if focused || hovered {
            f.rounded_outline(
                r.min + Vec2::splat(2.0),
                r.size() - Vec2::splat(4.0),
                4.0,
                if focused { 2.0 } else { 1.5 },
                if focused { theme.gold } else { theme.ink },
            );
        }
        if moves_available {
            l.hits.push(Hit {
                rect: r,
                action: Action::BackgammonPoint(idx as u8),
            });
        }
    }
    for (index, r) in slots.iter().copied().enumerate().take(24) {
        let p = display_board.points[index];
        if let Some(color) = p.color {
            let upper = r.center().y < mid_y;
            let n = usize::from(p.count.min(5));
            for i in 0..n {
                let cy = if upper {
                    board_top + edge_y + checker_r + i as f32 * step
                } else {
                    board_bottom - edge_y - checker_r - i as f32 * step
                };
                draw_checker(
                    f,
                    Vec2::new(r.center().x, cy),
                    checker_r,
                    color,
                    s.cursor,
                    &palette,
                );
            }
            if p.count > 5 {
                pip_badge(
                    f,
                    Vec2::new(
                        r.center().x,
                        if upper {
                            board_top + edge_y + checker_r + 4.5 * step
                        } else {
                            board_bottom - edge_y - checker_r - 4.5 * step
                        },
                    ),
                    p.count,
                    color,
                    small,
                    &palette,
                );
            }
        }
    }
    for side in [Side::White, Side::Black] {
        let n = display_board.bar[side.index()].min(5);
        for i in 0..n {
            let y = if side == Side::White {
                mid_y + checker_r + i as f32 * step
            } else {
                mid_y - checker_r - i as f32 * step
            };
            draw_checker(
                f,
                Vec2::new(bar_x + bar_w * 0.5, y),
                checker_r,
                side,
                s.cursor,
                &palette,
            );
        }
        if display_board.bar[side.index()] > 5 {
            let badge_y = if side == Side::White {
                mid_y + checker_r * 2.0
            } else {
                mid_y - checker_r * 2.0
            };
            pip_badge(
                f,
                Vec2::new(bar_x + bar_w * 0.5, badge_y),
                display_board.bar[side.index()],
                side,
                small,
                &palette,
            );
        }
    }
    for (side, amount, y) in [
        (Side::Black, game.board.off[1], board_top + board_h * 0.25),
        (Side::White, game.board.off[0], board_top + board_h * 0.75),
    ] {
        let color = side_color(side, &palette);
        circle(
            f,
            Vec2::new(tray_x + tray_w * 0.5, y),
            checker_r.min(tray_w * 0.33),
            2.0,
            color,
        );
        f.text(
            Vec2::new(tray_x + 4.0, y + checker_r + 5.0),
            format!("{amount}"),
            if small { 9.0 } else { 11.0 },
            color,
        );
    }
    let dice_y = mid_y;
    if game.rolled.is_empty() {
        centered(
            f,
            Vec2::new(inner_x + half_w * 0.5, dice_y),
            if game.phase == Phase::AwaitRoll {
                "ROLL"
            } else {
                ""
            },
            if small { 8.0 } else { 10.0 },
            theme.ink,
        );
    } else {
        let visible: Vec<u8> = game.rolled.iter().take(2).copied().collect();
        for (i, die) in visible.iter().enumerate() {
            let base_d = if small { 22.0 } else { 32.0 };
            let rolling = ui.motion.is_none() && busy && !s.save.settings.reduced_motion;
            let d = base_d
                * if rolling {
                    0.84 + (ui.age * 43.0 + i as f32).sin().abs() * 0.25
                } else {
                    1.0
                };
            let r = rect(
                inner_x + half_w * 0.5 - d - 2.0 + i as f32 * (d + 4.0),
                dice_y - d * 0.5,
                d,
                d,
            );
            let dice_side = if game.phase == Phase::AwaitRoll {
                game.turn.opponent()
            } else {
                game.turn
            };
            let shown = if rolling {
                (ui.age * 24.0) as u8 % 6 + 1
            } else {
                *die
            };
            draw_die(f, r, shown, side_color(dice_side, &palette));
        }
        let remaining = game
            .dice
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(" / ");
        centered(
            f,
            Vec2::new(
                inner_x + half_w * 0.5,
                dice_y + if small { 18.0 } else { 24.0 },
            ),
            &remaining,
            if small { 7.0 } else { 9.0 },
            theme.muted,
        );
    }
    let moving = ui
        .motion
        .as_ref()
        .filter(|_| !s.save.settings.reduced_motion);
    if let Some(motion) = moving {
        let t = (ui.age / ui.duration.max(0.01)).clamp(0.0, 1.0);
        let mv = motion.movement;
        {
            let start = checker_position(
                &motion.before,
                mv.from,
                motion.color,
                slots,
                bar_x,
                bar_w,
                tray_x,
                tray_w,
                board_top,
                board_bottom,
                mid_y,
                edge_y,
                checker_r,
                step,
                true,
            );
            let end = checker_position(
                &motion.before,
                mv.to,
                motion.color,
                slots,
                bar_x,
                bar_w,
                tray_x,
                tray_w,
                board_top,
                board_bottom,
                mid_y,
                edge_y,
                checker_r,
                step,
                false,
            );
            let p = start.lerp(end, t);
            draw_checker(f, p, checker_r, motion.color, s.cursor, &palette);
            if mv.is_hit && mv.to < 24 {
                let victim = motion.color.opponent();
                let from = checker_position(
                    &motion.before,
                    mv.to,
                    victim,
                    slots,
                    bar_x,
                    bar_w,
                    tray_x,
                    tray_w,
                    board_top,
                    board_bottom,
                    mid_y,
                    edge_y,
                    checker_r,
                    step,
                    true,
                );
                let to = checker_position(
                    &motion.before,
                    BAR,
                    victim,
                    slots,
                    bar_x,
                    bar_w,
                    tray_x,
                    tray_w,
                    board_top,
                    board_bottom,
                    mid_y,
                    edge_y,
                    checker_r,
                    step,
                    false,
                );
                draw_checker(f, from.lerp(to, t), checker_r, victim, s.cursor, &palette);
            }
        }
    }
    let status = match game.phase {
        Phase::AwaitRoll => {
            if game.turn == Side::White {
                "Your turn / roll the dice"
            } else {
                "CPU turn"
            }
        }
        Phase::Moving if game.turn == Side::White => {
            "Choose a highlighted source, then destination"
        }
        Phase::Moving => "CPU is moving...",
        Phase::Won => {
            if game.winner == Some(Side::White) {
                "You win!"
            } else {
                "CPU wins"
            }
        }
    };
    centered(
        f,
        Vec2::new(m + w * 0.5, f.size.y - 62.0),
        status,
        if small { 9.0 } else { 12.0 },
        if game.turn == Side::White {
            palette.white_team
        } else {
            palette.black_team
        },
    );
    let elapsed = format!(
        "{:02}:{:02}",
        game.elapsed_ms / 60_000,
        game.elapsed_ms / 1_000 % 60
    );
    f.text(
        Vec2::new(m + 2.0, f.size.y - 62.0),
        &elapsed,
        if small { 9.0 } else { 11.0 },
        theme.muted,
    );
    let wins = format!("WINS {}", s.save.backgammon_wins);
    f.text(
        Vec2::new(
            m + w - wins.len() as f32 * if small { 5.0 } else { 6.4 },
            f.size.y - 62.0,
        ),
        wins,
        if small { 9.0 } else { 11.0 },
        theme.gold,
    );
    if game.phase == Phase::Won && !busy {
        let won = game.winner == Some(Side::White);
        let panel = rect(
            m + w * 0.5 - if small { 90.0 } else { 125.0 },
            mid_y - 30.0,
            if small { 180.0 } else { 250.0 },
            60.0,
        );
        f.rounded_rect(panel, 8.0, palette.dialog_panel);
        let pulse = if s.save.settings.reduced_motion {
            0.0
        } else {
            ui.feedback
                .events
                .iter()
                .find(|event| {
                    matches!(
                        event.kind,
                        crate::table_feedback::EffectKind::Win
                            | crate::table_feedback::EffectKind::Stuck
                    )
                })
                .map(|event| (1.0 - event.age / event.duration()) * (event.age * 15.0).sin().abs())
                .unwrap_or(0.0)
        };
        f.rounded_outline(
            panel.min,
            panel.size(),
            8.0,
            1.0 + pulse,
            if won {
                palette.white_team
            } else {
                palette.black_team
            },
        );
        centered(
            f,
            Vec2::new(panel.center().x, panel.min.y + 15.0),
            if won { "YOU WIN" } else { "CPU WINS" },
            if small { 15.0 } else { 20.0 },
            if won {
                palette.white_team
            } else {
                palette.black_team
            },
        );
        centered(
            f,
            Vec2::new(panel.center().x, panel.min.y + 39.0),
            &format!("COMPLETED IN {elapsed}"),
            if small { 9.0 } else { 11.0 },
            theme.ink,
        );
    }
    s.backgammon_ui.point_bounds = slots.to_vec();
}

fn draw_die(f: &mut Frame, r: Rect, value: u8, color: Color) {
    f.rounded_rect(r, 4.0, color);
    let lo = r.min + r.size() * 0.25;
    let hi = r.min + r.size() * 0.75;
    let center = r.center();
    let mut pips = Vec::with_capacity(6);
    if value % 2 == 1 {
        pips.push(center);
    }
    if value >= 2 {
        pips.extend([lo, hi]);
    }
    if value >= 4 {
        pips.extend([Vec2::new(hi.x, lo.y), Vec2::new(lo.x, hi.y)]);
    }
    if value == 6 {
        pips.extend([Vec2::new(lo.x, center.y), Vec2::new(hi.x, center.y)]);
    }
    for p in pips {
        disk(f, p, r.width() * 0.065, f.theme().bg);
    }
}

fn side_color(side: Side, palette: &BackgammonPalette) -> Color {
    if side == Side::White {
        palette.white_team
    } else {
        palette.black_team
    }
}
#[allow(clippy::too_many_arguments)]
fn checker_position(
    board: &gamevex_core::backgammon::Board,
    index: u8,
    side: Side,
    slots: [Rect; 26],
    bar_x: f32,
    bar_w: f32,
    tray_x: f32,
    tray_w: f32,
    board_top: f32,
    board_bottom: f32,
    mid_y: f32,
    edge_y: f32,
    radius: f32,
    step: f32,
    source: bool,
) -> Vec2 {
    if index == BAR {
        let count = board.bar[side.index()] as f32;
        let offset = if source {
            (count - 1.0).clamp(0.0, 4.0)
        } else {
            count.min(4.0)
        };
        let sign = if side == Side::White { 1.0 } else { -1.0 };
        return Vec2::new(bar_x + bar_w * 0.5, mid_y + sign * (radius + offset * step));
    }
    if index == OFF {
        return Vec2::new(
            tray_x + tray_w * 0.5,
            if side == Side::White {
                board_top + (board_bottom - board_top) * 0.75
            } else {
                board_top + (board_bottom - board_top) * 0.25
            },
        );
    }
    let slot = slots[index as usize];
    let upper = slot.center().y < mid_y;
    let count = if board.points[index as usize].color == Some(side) {
        board.points[index as usize].count as f32
    } else {
        0.0
    };
    let offset = if source {
        (count - 1.0).clamp(0.0, 4.0)
    } else {
        count.min(4.0)
    };
    let y = if upper {
        board_top + edge_y + radius + offset * step
    } else {
        board_bottom - edge_y - radius - offset * step
    };
    Vec2::new(slot.center().x, y)
}
fn disk(f: &mut Frame, at: Vec2, r: f32, color: Color) {
    let points: Vec<_> = (0..48)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 48.0;
            at + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect();
    f.polygon(&points, color);
}
fn draw_checker(
    f: &mut Frame,
    at: Vec2,
    r: f32,
    side: Side,
    cursor: Option<Vec2>,
    palette: &BackgammonPalette,
) {
    let color = side_color(side, palette);
    disk(f, at + Vec2::new(0.0, 2.0), r, palette.shadow);
    disk(
        f,
        at,
        r,
        if side == Side::White {
            palette.white_body
        } else {
            palette.black_body
        },
    );
    circle(f, at, r, 1.5, color);
    circle(f, at, r * 0.72, 0.8, color);
    if cursor.is_some_and(|p| (p - at).length() < r * 1.2) {
        circle(f, at, r + 2.0, 1.2, f.theme().ink);
    }
}

fn pip_badge(f: &mut Frame, at: Vec2, n: u8, side: Side, small: bool, palette: &BackgammonPalette) {
    let r = if small { 7.0 } else { 9.0 };
    f.rounded_rect(
        rect(at.x - r, at.y - r * 0.62, r * 2.0, r * 1.24),
        r * 0.3,
        side_color(side, palette),
    );
    centered(
        f,
        Vec2::new(at.x, at.y - r * 0.45),
        &n.to_string(),
        if small { 8.0 } else { 10.0 },
        f.theme().bg,
    );
}
fn centered(f: &mut Frame, p: Vec2, text: &str, size: f32, color: Color) {
    f.text(
        p - Vec2::new(text.chars().count() as f32 * size * 0.30, 0.0),
        text,
        size,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamevex_core::ColorScheme;

    #[test]
    fn backgammon_palette_supports_all_schemes_and_paper_is_light() {
        for scheme in [
            ColorScheme::Cyan,
            ColorScheme::Amber,
            ColorScheme::Emerald,
            ColorScheme::Violet,
            ColorScheme::Ice,
            ColorScheme::Paper,
        ] {
            let pal = BackgammonPalette::for_scheme(scheme);
            let felt = pal.felt.to_srgba();
            let wood = pal.wood.to_srgba();
            assert!(felt.red.is_finite() && felt.green.is_finite() && felt.blue.is_finite());
            assert!(wood.red.is_finite() && wood.green.is_finite() && wood.blue.is_finite());
        }

        let paper = BackgammonPalette::for_scheme(ColorScheme::Paper);
        let felt = paper.felt.to_srgba();
        assert!(felt.red > 0.85 && felt.green > 0.85 && felt.blue > 0.80);
        let white_team = paper.white_team.to_srgba();
        let black_team = paper.black_team.to_srgba();
        // Burgundy white_team has high red and low green/blue
        assert!(white_team.red > 0.50 && white_team.green < 0.20 && white_team.blue < 0.25);
        // Charcoal black_team is dark
        assert!(black_team.red < 0.20 && black_team.green < 0.20 && black_team.blue < 0.20);
    }
}
