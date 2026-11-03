use super::screens::{button, circle};
use super::*;
use gamevex_core::backgammon::{BAR, Color as Side, OFF, Phase};

const FELT: Color = Color::srgb(0.018, 0.047, 0.067);
const WOOD: Color = Color::srgb(0.042, 0.075, 0.09);
const POINT_A: Color = Color::srgb(0.075, 0.15, 0.18);
const POINT_B: Color = Color::srgb(0.13, 0.08, 0.17);

pub(super) fn backgammon(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let hit_start = l.hits.len();
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
        INK,
    );
    let top = if small { 126.0 } else { 94.0 };
    let control_h = if short { 29.0 } else { 34.0 };
    let gap = if small { 5.0 } else { 8.0 };
    let control_widths = if small {
        [45.0, 52.0, 71.0, 49.0]
    } else {
        [64.0, 68.0, 105.0, 72.0]
    };
    let total: f32 = control_widths.iter().sum::<f32>() + gap * 3.0;
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
        ("Rules", Action::BackgammonRules),
    ]
    .into_iter()
    .enumerate()
    {
        if ui.rules {
            x += control_widths[i] + gap;
            continue;
        }
        let enabled = match i {
            0 => game.phase == Phase::AwaitRoll && game.turn == Side::White && !ui.rules && !busy,
            _ => true,
        };
        let control_rect = rect(x, top, control_widths[i], control_h);
        if i == 0 && !enabled {
            f.rect(control_rect.min, control_rect.size(), PANEL);
            f.outline(control_rect.min, control_rect.size(), 1.0, EDGE);
            centered(
                f,
                Vec2::new(control_rect.center().x, control_rect.min.y + 7.0),
                label,
                12.0,
                MUTED,
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
        CYAN,
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
        MAGENTA,
    );

    let board_top = stat_y + stat_size + 6.0;
    let board_bottom = f.size.y - if short { 85.0 } else { 90.0 };
    let board_h = (board_bottom - board_top).max(100.0);
    let board_x = m;
    let board_w = w;
    let table = rect(board_x, board_top, board_w, board_h);
    f.rounded_rect(table, 9.0, WOOD);
    f.rounded_outline(table.min, table.size(), 9.0, 1.0, EDGE);
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
        FELT,
    );
    f.rect(
        rect(bar_x, board_top + 4.0, bar_w, board_h - 8.0).min,
        Vec2::new(bar_w, board_h - 8.0),
        Color::srgb(0.023, 0.042, 0.06),
    );
    f.line(
        Vec2::new(bar_x + bar_w * 0.5, board_top + 5.0),
        Vec2::new(bar_x + bar_w * 0.5, board_bottom - 5.0),
        1.0,
        EDGE,
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
        let c = if col % 2 == 0 { POINT_A } else { POINT_B };
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
            if c == POINT_A { POINT_B } else { POINT_A },
        );
        f.text(
            Vec2::new(px + col_w * 0.5 - 4.0, board_top + 4.0),
            format!("{}", top_point + 1),
            if small { 7.0 } else { 9.0 },
            MUTED,
        );
        f.text(
            Vec2::new(px + col_w * 0.5 - 4.0, board_bottom - 14.0),
            format!("{}", bottom_point + 1),
            if small { 7.0 } else { 9.0 },
            MUTED,
        );
    }
    slots[BAR as usize] = rect(bar_x, mid_y, bar_w, board_bottom - mid_y);
    slots[OFF as usize] = rect(tray_x, mid_y, tray_w, board_bottom - mid_y);
    f.rounded_rect(
        rect(tray_x, board_top + 4.0, tray_w - 2.0, board_h - 8.0),
        4.0,
        Color::srgb(0.013, 0.031, 0.048),
    );
    f.line(
        Vec2::new(tray_x + 2.0, mid_y),
        Vec2::new(tray_x + tray_w - 4.0, mid_y),
        1.0,
        EDGE,
    );
    let legal = if !ui.rules && game.turn == Side::White && game.phase == Phase::Moving {
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
    let moves_available =
        game.turn == Side::White && game.phase == Phase::Moving && !ui.rules && !busy;
    // Keep every slot in diagnostics; the parent filters eligible IDs when
    // exporting click targets. Inactive slots also block controls underneath.
    for (idx, r) in slots.iter().copied().enumerate() {
        let focused = moves_available && s.keyboard && s.focus == l.hits.len();
        let hovered = moves_available && s.cursor.is_some_and(|point| r.contains(point));
        if moves_available && legal.iter().any(|mv| mv.from == idx as u8) {
            f.rounded_outline(r.min, r.size(), 4.0, 2.0, CYAN);
        }
        if source == Some(idx as u8) {
            f.rounded_outline(
                r.min + Vec2::splat(1.0),
                r.size() - Vec2::splat(2.0),
                4.0,
                2.0,
                GOLD,
            );
        } else if destinations.contains(&(idx as u8)) {
            f.rounded_outline(
                r.min + Vec2::splat(2.0),
                r.size() - Vec2::splat(4.0),
                4.0,
                2.0,
                MAGENTA,
            );
        } else if focused || hovered {
            f.rounded_outline(
                r.min + Vec2::splat(2.0),
                r.size() - Vec2::splat(4.0),
                4.0,
                if focused { 2.0 } else { 1.5 },
                if focused { GOLD } else { INK },
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
                draw_checker(f, Vec2::new(r.center().x, cy), checker_r, color, s.cursor);
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
            );
        }
    }
    for (side, amount, y) in [
        (Side::Black, game.board.off[1], board_top + board_h * 0.25),
        (Side::White, game.board.off[0], board_top + board_h * 0.75),
    ] {
        let color = side_color(side);
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
            INK,
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
            draw_die(f, r, shown, side_color(dice_side));
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
            MUTED,
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
            draw_checker(f, p, checker_r, motion.color, s.cursor);
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
                draw_checker(f, from.lerp(to, t), checker_r, victim, s.cursor);
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
            CYAN
        } else {
            MAGENTA
        },
    );
    if ui.rules {
        let panel = rect(
            m + 8.0,
            board_top + board_h * 0.13,
            w - 16.0,
            board_h * 0.74,
        );
        f.rounded_rect(panel, 8.0, Color::srgba(0.008, 0.016, 0.031, 0.96));
        f.rounded_outline(panel.min, panel.size(), 8.0, 1.0, CYAN);
        for (i, line) in [
            "Race all 15 checkers off the board first.",
            "Each die is one move; doubles give four.",
            "Blots can be hit. Bar checkers re-enter first.",
            "Bear off after reaching your home board.",
            "Higher dice bear off if no farther checker remains.",
        ]
        .into_iter()
        .enumerate()
        {
            f.text(
                panel.min + Vec2::new(12.0, 14.0 + i as f32 * if small { 23.0 } else { 31.0 }),
                line,
                if small { 9.0 } else { 12.0 },
                INK,
            );
        }
    }
    if ui.rules {
        l.hits.truncate(hit_start);
        button(
            f,
            s,
            l,
            rect(m + w - 72.0, top, 68.0, control_h),
            "Close",
            Action::BackgammonRules,
            true,
        );
    }
    let elapsed = format!(
        "{:02}:{:02}",
        game.elapsed_ms / 60_000,
        game.elapsed_ms / 1_000 % 60
    );
    f.text(
        Vec2::new(m + 2.0, f.size.y - 62.0),
        &elapsed,
        if small { 9.0 } else { 11.0 },
        MUTED,
    );
    let wins = format!("WINS {}", s.save.backgammon_wins);
    f.text(
        Vec2::new(
            m + w - wins.len() as f32 * if small { 5.0 } else { 6.4 },
            f.size.y - 62.0,
        ),
        wins,
        if small { 9.0 } else { 11.0 },
        GOLD,
    );
    if game.phase == Phase::Won && !busy {
        let won = game.winner == Some(Side::White);
        let panel = rect(
            m + w * 0.5 - if small { 90.0 } else { 125.0 },
            mid_y - 30.0,
            if small { 180.0 } else { 250.0 },
            60.0,
        );
        f.rounded_rect(panel, 8.0, Color::srgba(0.008, 0.016, 0.031, 0.94));
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
            if won { CYAN } else { MAGENTA },
        );
        centered(
            f,
            Vec2::new(panel.center().x, panel.min.y + 15.0),
            if won { "YOU WIN" } else { "CPU WINS" },
            if small { 15.0 } else { 20.0 },
            if won { CYAN } else { MAGENTA },
        );
        centered(
            f,
            Vec2::new(panel.center().x, panel.min.y + 39.0),
            &format!("COMPLETED IN {elapsed}"),
            if small { 9.0 } else { 11.0 },
            INK,
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
        disk(f, p, r.width() * 0.065, BG);
    }
}

fn side_color(side: Side) -> Color {
    if side == Side::White { CYAN } else { MAGENTA }
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
fn draw_checker(f: &mut Frame, at: Vec2, r: f32, side: Side, cursor: Option<Vec2>) {
    let color = side_color(side);
    disk(
        f,
        at + Vec2::new(0.0, 2.0),
        r,
        Color::srgb(0.003, 0.008, 0.014),
    );
    disk(
        f,
        at,
        r,
        if side == Side::White {
            Color::srgb(0.025, 0.21, 0.29)
        } else {
            Color::srgb(0.26, 0.015, 0.15)
        },
    );
    circle(f, at, r, 1.5, color);
    circle(f, at, r * 0.72, 0.8, color);
    if cursor.is_some_and(|p| (p - at).length() < r * 1.2) {
        circle(f, at, r + 2.0, 1.2, INK);
    }
}

fn pip_badge(f: &mut Frame, at: Vec2, n: u8, side: Side, small: bool) {
    let r = if small { 7.0 } else { 9.0 };
    f.rounded_rect(
        rect(at.x - r, at.y - r * 0.62, r * 2.0, r * 1.24),
        r * 0.3,
        side_color(side),
    );
    centered(
        f,
        Vec2::new(at.x, at.y - r * 0.45),
        &n.to_string(),
        if small { 8.0 } else { 10.0 },
        BG,
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
