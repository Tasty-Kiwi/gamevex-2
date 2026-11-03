//! Responsive no-limit table built from the shared GameVex card artwork.
use super::screens::{button, card_face, card_face_with_emphasis, card_hit_bounds, circle};
use super::*;
use gamevex_core::holdem::{Action as Play, Phase};

pub(super) fn holdem(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32, dt: f32) {
    let g = s.holdem_ui.game.clone();
    let small = w < 620.0;
    let compact = f.size.y < 650.0 || (!small && f.size.y < 700.0);
    let short_desktop = !small && !compact && f.size.y < 760.0;
    s.holdem_ui.board_bounds.clear();
    s.holdem_ui.hand_bounds.clear();
    let reduced = s.save.settings.reduced_motion;
    let busy = s.holdem_ui.busy(reduced);
    let terminal = g.terminal();
    let idle = g.phase == Phase::Idle;
    let ready = !busy;
    let top = if small { 172.0 } else { 154.0 };
    let bottom = f.size.y - 52.0;
    let table = rect(m, top, w, bottom - top);
    f.text(
        Vec2::new(m, 82.0),
        "Texas Hold'em",
        if small { 25.0 } else { 36.0 },
        INK,
    );
    f.text(
        Vec2::new(m, if small { 113.0 } else { 126.0 }),
        format!("Bankroll ${} / Blinds $5/$10", s.save.holdem_balance),
        if small { 10.0 } else { 12.0 },
        CYAN,
    );
    if terminal && ready {
        let x = if small { m } else { m + w - 244.0 };
        let y = if small { 127.0 } else { 88.0 };
        button(
            f,
            s,
            l,
            rect(x, y, 40.0, 40.0),
            "-",
            Action::HoldemOpponents(-1),
            false,
        );
        f.text(
            Vec2::new(x + 48.0, y + 14.0),
            format!(
                "{} CPU{}",
                g.seats.len() - 1,
                if g.seats.len() == 2 { "" } else { "s" }
            ),
            11.0,
            MUTED,
        );
        button(
            f,
            s,
            l,
            rect(x + 103.0, y, 40.0, 40.0),
            "+",
            Action::HoldemOpponents(1),
            false,
        );
    }
    f.rounded_rect(table, 18.0, Color::srgb(0.012, 0.028, 0.046));
    f.rounded_outline(table.min, table.size(), 18.0, 1.5, EDGE);
    let ncpu = g.seats.len() - 1;
    let gap = if small { 6.0 } else { 18.0 };
    let seat_w =
        ((w - 24.0 - gap * (ncpu - 1) as f32) / ncpu as f32).min(if small { 190.0 } else { 210.0 });
    let seats_x = m + (w - seat_w * ncpu as f32 - gap * (ncpu - 1) as f32) * 0.5;
    let cpu_card_w = ((seat_w - 18.0) * 0.5).min(if compact {
        22.0
    } else if small {
        43.0
    } else {
        if short_desktop { 38.0 } else { 55.0 }
    });
    let cpu_card_h = cpu_card_w * 1.4;
    let cpu_y = top + 12.0;
    let reveal = g.phase == Phase::Showdown && !busy;
    for i in 1..g.seats.len() {
        let seat = &g.seats[i];
        let x = seats_x + (i - 1) as f32 * (seat_w + gap);
        let panel = rect(
            x,
            cpu_y,
            seat_w,
            if compact {
                70.0
            } else {
                cpu_card_h
                    + if small {
                        38.0
                    } else if short_desktop {
                        42.0
                    } else {
                        48.0
                    }
            },
        );
        f.rounded_rect(panel, 7.0, PANEL);
        let active = g.turn == Some(i) && !terminal;
        f.rounded_outline(
            panel.min,
            panel.size(),
            7.0,
            if active { 2.0 } else { 1.0 },
            if active {
                MAGENTA
            } else if reveal && g.winners[i] {
                GOLD
            } else {
                EDGE
            },
        );
        f.text(
            panel.min + Vec2::new(6.0, 6.0),
            format!("CPU {i}"),
            if small { 9.0 } else { 12.0 },
            if seat.folded { MUTED } else { MAGENTA },
        );
        f.text(
            panel.min + Vec2::new(6.0, if small { 19.0 } else { 23.0 }),
            format!("${}", if idle { s.holdem_ui.buy_in } else { seat.stack }),
            if small { 10.0 } else { 13.0 },
            INK,
        );
        if !idle && g.dealer == i {
            dealer(f, Vec2::new(panel.max.x - 10.0, panel.min.y + 10.0));
        }
        let cards_x = x + (seat_w - cpu_card_w * 2.0 - 4.0) * 0.5;
        for (j, card) in seat.hole.iter().enumerate() {
            let mut card = *card;
            card.face_up = reveal && !seat.folded;
            let delay = if s.holdem_ui.dealing && !reduced {
                (i * 2 + j) as f32 * 0.025
            } else {
                0.0
            };
            let t = if reduced {
                1.0
            } else {
                ((s.holdem_ui.age - delay) / 0.25).clamp(0.0, 1.0)
            };
            card_face_with_emphasis(
                f,
                rect(
                    cards_x + j as f32 * (cpu_card_w + 4.0),
                    cpu_y + 35.0 + (1.0 - t) * -12.0,
                    cpu_card_w,
                    cpu_card_h,
                ),
                card,
                false,
                true,
                if seat.folded { 0.25 } else { 1.0 },
            );
        }
        let caption = if reveal && !seat.folded {
            g.evaluations[i].as_ref().map_or("", |e| e.label())
        } else {
            &seat.last_action
        };
        if compact && reveal && seat_w > 140.0 {
            f.text(Vec2::new(x + 6.0, cpu_y + 38.0), caption, 7.0, GOLD);
        }
        if !caption.is_empty() && !compact {
            f.text(
                Vec2::new(x + 6.0, panel.max.y + 5.0),
                caption,
                if small { 7.0 } else { 10.0 },
                if seat.folded { MUTED } else { GOLD },
            );
        }
        if seat.committed > 0 && !terminal && !compact {
            f.text(
                Vec2::new(
                    x + seat_w * 0.5 - 18.0,
                    panel.max.y + if small { 16.0 } else { 20.0 },
                ),
                format!("IN ${}", seat.committed),
                if small { 8.0 } else { 11.0 },
                MUTED,
            );
        }
    }
    let board_y = if compact {
        top + 113.0
    } else if small {
        top + 162.0
    } else {
        top + if short_desktop { 164.0 } else { 203.0 }
    };
    let bw = if compact {
        40.0
    } else if small {
        56.0
    } else {
        if short_desktop { 64.0 } else { 80.0 }
    };
    let bh = bw * 1.4;
    let bg = if small { 5.0 } else { 10.0 };
    let span = bw * 5.0 + bg * 4.0;
    let bx = m + (w - span) * 0.5;
    let amount = if terminal && !idle {
        g.payouts.iter().sum::<u32>()
    } else {
        g.pot()
    };
    centered(
        f,
        Vec2::new(m + w * 0.5, board_y - 24.0),
        &if idle {
            "CHOOSE YOUR BUY-IN".into()
        } else {
            format!("{} / ${amount}", if terminal { "PAID" } else { "POT" })
        },
        if compact { 12.0 } else { 16.0 },
        GOLD,
    );
    for i in 0..5 {
        let r = rect(bx + i as f32 * (bw + bg), board_y, bw, bh);
        s.holdem_ui.board_bounds.push(r);
        if let Some(card) = g.board.get(i) {
            let t = if reduced || i < s.holdem_ui.board_from {
                1.0
            } else {
                ((s.holdem_ui.age - (i - s.holdem_ui.board_from) as f32 * 0.12) / 0.3)
                    .clamp(0.0, 1.0)
            };
            if t > 0.0 {
                let highlighted = terminal
                    && !busy
                    && g.evaluations.iter().enumerate().any(|(seat, e)| {
                        g.winners[seat] && e.as_ref().is_some_and(|e| e.highlights.contains(card))
                    });
                card_face(
                    f,
                    rect(r.min.x, r.min.y - (1.0 - t) * 24.0, bw, bh),
                    *card,
                    highlighted,
                    true,
                );
            }
        } else {
            f.rounded_outline(r.min, r.size(), 4.0, 1.0, EDGE);
            f.line(
                r.center() - Vec2::new(7.0, 0.0),
                r.center() + Vec2::new(7.0, 0.0),
                1.0,
                EDGE,
            );
        }
    }
    let status_y = if compact {
        board_y - 12.0
    } else {
        board_y + bh + 10.0
    };
    let phase = match g.phase {
        Phase::Idle => "One hand per buy-in",
        Phase::Preflop => "PREFLOP",
        Phase::Flop => "FLOP",
        Phase::Turn => "TURN",
        Phase::River => "RIVER",
        Phase::Showdown => "SHOWDOWN",
        Phase::Complete => "HAND COMPLETE",
    };
    if !terminal || idle {
        centered(
            f,
            if short_desktop {
                Vec2::new(m + 28.0, status_y)
            } else {
                Vec2::new(m + w * 0.5, status_y)
            },
            phase,
            if compact { 8.0 } else { 11.0 },
            MUTED,
        );
    }
    if idle {
        setup(f, s, l, m, w, bottom);
    } else {
        let hw = if compact {
            if terminal { 32.0 } else { 41.0 }
        } else if small {
            64.0
        } else {
            if terminal {
                if short_desktop { 68.0 } else { 80.0 }
            } else {
                88.0
            }
        };
        let hh = hw * 1.4;
        let hy = bottom
            - if compact {
                if terminal { 163.0 } else { 152.0 }
            } else if small {
                if terminal { 220.0 } else { 195.0 }
            } else {
                if terminal && !short_desktop {
                    240.0
                } else {
                    218.0
                }
            };
        let hx = m + w * 0.5 - hw - 5.0;
        let mut hovered = None;
        for j in 0..g.seats[0].hole.len() {
            let r = rect(hx + j as f32 * (hw + 10.0), hy, hw, hh);
            let shown = s
                .holdem_ui
                .positions
                .get(j)
                .map_or(r, |p| rect(p.x, p.y, hw, hh));
            if s.cursor
                .is_some_and(|p| card_hit_bounds(shown, r).contains(p))
            {
                hovered = Some(j);
            }
        }
        s.holdem_ui.hovered = hovered;
        if s.holdem_ui.positions.len() != 2 {
            s.holdem_ui.positions = vec![
                Vec2::new(hx, hy + 15.0),
                Vec2::new(hx + hw + 10.0, hy + 15.0),
            ];
        }
        for (j, card) in g.seats[0].hole.iter().enumerate() {
            let target = Vec2::new(
                hx + j as f32 * (hw + 10.0),
                hy - if hovered == Some(j) { 14.0 } else { 0.0 },
            );
            s.holdem_ui.positions[j] = s.holdem_ui.positions[j].lerp(
                target,
                if reduced {
                    1.0
                } else {
                    (dt.max(0.0) * 22.0).min(1.0)
                },
            );
            let p = s.holdem_ui.positions[j];
            let best = reveal
                && g.winners[0]
                && g.evaluations[0]
                    .as_ref()
                    .is_some_and(|e| e.highlights.contains(card));
            s.holdem_ui.hand_bounds.push(rect(p.x, p.y, hw, hh));
            card_face_with_emphasis(
                f,
                rect(p.x, p.y, hw, hh),
                *card,
                best || hovered == Some(j),
                true,
                if g.seats[0].folded { 0.28 } else { 1.0 },
            );
        }
        f.text(
            Vec2::new(m + 12.0, hy + 5.0),
            format!("YOU / ${}", g.seats[0].stack),
            if small { 10.0 } else { 14.0 },
            CYAN,
        );
        if g.dealer == 0 {
            dealer(
                f,
                if compact {
                    Vec2::new(m + 90.0, hy + 8.0)
                } else {
                    Vec2::new(m + 20.0, hy + 31.0)
                },
            );
        }
        if terminal && !busy {
            let net = i64::from(g.seats[0].stack) - i64::from(g.buy_in);
            let color = if net > 0 {
                GOLD
            } else if net < 0 {
                RED
            } else {
                CYAN
            };
            centered(
                f,
                if compact {
                    Vec2::new(m + 58.0, hy + 15.0)
                } else if short_desktop {
                    Vec2::new(m + 160.0, hy + 15.0)
                } else {
                    Vec2::new(m + w * 0.5, hy - 25.0)
                },
                &format!(
                    "{} ${}",
                    if net > 0 {
                        "YOU WIN"
                    } else if net < 0 {
                        "YOU LOST"
                    } else {
                        "EVEN"
                    },
                    net.abs()
                ),
                if compact { 12.0 } else { 17.0 },
                color,
            );
            if let Some(e) = g.evaluations[0].as_ref() {
                f.text(
                    Vec2::new(m + 12.0, hy + if compact { 34.0 } else { 52.0 }),
                    e.label(),
                    if small { 8.0 } else { 12.0 },
                    MUTED,
                );
            }
            setup(f, s, l, m, w, bottom);
        } else if !terminal {
            let turn = if g.turn == Some(0) {
                format!("Your turn / Call ${}", g.to_call(0).min(g.seats[0].stack))
            } else {
                format!("CPU {} is thinking...", g.turn.unwrap_or(1))
            };
            f.text(
                Vec2::new(m + 12.0, bottom - 13.0),
                turn,
                if small { 9.0 } else { 12.0 },
                INK,
            );
            if g.seats[0].folded && ready {
                button(
                    f,
                    s,
                    l,
                    rect(m + w * 0.5 - 80.0, bottom - 61.0, 160.0, 40.0),
                    "Skip to result",
                    Action::HoldemSkip,
                    true,
                );
            }
            if g.turn == Some(0) && ready {
                betting(f, s, l, m, w, bottom, small);
            }
        }
    }
    for event in &s.holdem_ui.feedback.events {
        use crate::table_feedback::EffectKind;
        if !matches!(
            event.kind,
            EffectKind::Win | EffectKind::Stuck | EffectKind::Run
        ) {
            continue;
        }
        let t = (event.age / event.duration()).clamp(0.0, 1.0);
        let c = if event.kind == EffectKind::Stuck {
            RED
        } else if event.kind == EffectKind::Win {
            GOLD
        } else {
            CYAN
        }
        .to_srgba();
        f.rounded_outline(
            table.min + Vec2::splat(3.0),
            table.size() - Vec2::splat(6.0),
            16.0,
            2.0,
            Color::srgba(c.red, c.green, c.blue, 1.0 - t),
        );
        if event.kind == EffectKind::Win && !reduced {
            for i in 0..32 {
                let phase = i as f32 / 32.0;
                let x = table.min.x + 10.0 + phase * (table.width() - 20.0);
                let y = table.max.y
                    - 8.0
                    - (phase * std::f32::consts::PI).sin()
                        * (80.0 + (i % 4) as f32 * 24.0)
                        * (t * std::f32::consts::PI).sin();
                f.rect(
                    Vec2::new(x, y),
                    Vec2::splat(2.5),
                    Color::srgba(c.red, c.green, c.blue, 1.0 - t),
                );
            }
        }
    }
}
fn setup(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32, bottom: f32) {
    let compact = f.size.y < 650.0;
    let ready = !s.holdem_ui.busy(s.save.settings.reduced_motion);
    if !ready {
        return;
    }
    let (min, max) = s.holdem_stake_bounds();
    s.holdem_ui.buy_in = s.holdem_ui.buy_in.clamp(min, max);
    if max == 0 {
        button(
            f,
            s,
            l,
            rect(m + w * 0.5 - 95.0, bottom - 60.0, 190.0, 40.0),
            "Restart / $1000",
            Action::HoldemRestart,
            true,
        );
        return;
    }
    let y = bottom - 94.0;
    let center = m + w * 0.5;

    centered(
        f,
        Vec2::new(center, y - 20.0),
        &format!("BUY-IN ${}", s.holdem_ui.buy_in),
        if compact { 10.0 } else { 13.0 },
        CYAN,
    );
    let gap = 6.0;
    let bw = ((w - 24.0 - gap * 3.0) / 4.0).min(105.0);
    let x = center - (bw * 4.0 + gap * 3.0) * 0.5;
    for (i, (label, action)) in [
        ("Min", Action::HoldemStakeMin),
        ("-25", Action::HoldemStake(-25)),
        ("+25", Action::HoldemStake(25)),
        ("Max", Action::HoldemStakeMax),
    ]
    .into_iter()
    .enumerate()
    {
        button(
            f,
            s,
            l,
            rect(x + i as f32 * (bw + gap), y, bw, 32.0),
            label,
            action,
            false,
        );
    }
    button(
        f,
        s,
        l,
        rect(center - 80.0, bottom - 51.0, 160.0, 36.0),
        "Deal hand",
        Action::HoldemStart,
        true,
    );
}
fn betting(
    f: &mut Frame,
    s: &mut Arcade,
    l: &mut ViewLayout,
    m: f32,
    w: f32,
    bottom: f32,
    small: bool,
) {
    let g = s.holdem_ui.game.clone();
    let legal = g.legal_actions(0);
    if let Some(b) = g.bet_bounds(0) {
        s.holdem_ui.raise_target = s.holdem_ui.raise_target.clamp(b.min_total, b.max_total);
        let gap = 5.0;
        let bw = ((w - 24.0 - gap * 3.0) / 4.0).min(86.0);
        let start = m + w * 0.5 - (bw * 4.0 + gap * 3.0) * 0.5;
        for (i, (label, action)) in [
            ("-10", Action::HoldemRaise(-10)),
            ("Min", Action::HoldemRaiseMin),
            ("Pot", Action::HoldemRaisePot),
            ("+10", Action::HoldemRaise(10)),
        ]
        .into_iter()
        .enumerate()
        {
            button(
                f,
                s,
                l,
                rect(start + i as f32 * (bw + gap), bottom - 91.0, bw, 30.0),
                label,
                action,
                false,
            );
        }
    }
    let gap = 6.0;
    let bw = ((w - 24.0 - gap * (legal.len().saturating_sub(1)) as f32)
        / legal.len().max(1) as f32)
        .min(154.0);
    let x =
        m + w * 0.5 - (bw * legal.len() as f32 + gap * legal.len().saturating_sub(1) as f32) * 0.5;
    for (i, action) in legal.into_iter().enumerate() {
        let label = match action {
            Play::Fold => "Fold".into(),
            Play::Check => "Check".into(),
            Play::Call => format!("Call ${}", g.to_call(0).min(g.seats[0].stack)),
            Play::Bet(_) => format!("Bet ${}", s.holdem_ui.raise_target),
            Play::Raise(_) => format!("To ${}", s.holdem_ui.raise_target),
            Play::AllIn => "All in".into(),
        };
        button(
            f,
            s,
            l,
            rect(x + i as f32 * (bw + gap), bottom - 57.0, bw, 36.0),
            &label,
            Action::HoldemPlay(action),
            matches!(action, Play::Call | Play::Check),
        );
    }
    let _ = small;
}
fn dealer(f: &mut Frame, p: Vec2) {
    circle(f, p, 7.0, 1.0, GOLD);
    f.text(p - Vec2::new(3.0, 4.0), "D", 8.0, GOLD);
}
fn centered(f: &mut Frame, p: Vec2, text: &str, size: f32, color: Color) {
    f.text(
        p - Vec2::new(text.chars().count() as f32 * size * 0.3, 0.0),
        text,
        size,
        color,
    );
}
