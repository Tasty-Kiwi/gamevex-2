//! Two-seat Cribbage table, using the same cards and hover treatment as Card Lab.
use super::screens::{
    button, card_face, card_face_with_emphasis, card_hit_bounds, circle, rects_overlap,
};
use super::*;
use gamevex_core::{
    cards::Card,
    cribbage::{Phase, hand_score},
};

pub(super) fn cribbage(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32, dt: f32) {
    let g = s.save.cribbage.clone();
    let small = w < 620.0;
    let compact = f.size.y < 650.0;
    let motion = !s.save.settings.reduced_motion;
    let busy = motion && s.cribbage_ui.age < 0.35;
    let rules = s.cribbage_ui.rules;
    f.text(
        Vec2::new(m, 82.0),
        "Cribbage",
        if small { 26.0 } else { 36.0 },
        INK,
    );
    f.text(
        Vec2::new(m, if small { 112.0 } else { 126.0 }),
        format!(
            "Round {} / First to 121 / {} wins",
            g.round, s.save.cribbage_wins
        ),
        if small { 10.0 } else { 12.0 },
        MUTED,
    );
    let cy = if small { 132.0 } else { 88.0 };
    button(
        f,
        s,
        l,
        rect(m + w - 178.0, cy, 104.0, 40.0),
        "New match",
        Action::CribbageNew,
        false,
    );
    button(
        f,
        s,
        l,
        rect(m + w - 66.0, cy, 66.0, 40.0),
        "Rules",
        Action::CribbageRules,
        rules,
    );
    let top = if small { 184.0 } else { 154.0 };
    let bottom = f.size.y - 52.0;
    let table = rect(m, top, w, bottom - top);
    f.rounded_rect(table, 18.0, Color::srgb(0.012, 0.028, 0.046));
    f.rounded_outline(table.min, table.size(), 18.0, 1.5, EDGE);
    let badge_w = (w - 38.0) * 0.5;
    for (i, label) in ["YOU", "CPU"].into_iter().enumerate() {
        let r = rect(
            m + 12.0 + i as f32 * (badge_w + 14.0),
            top + 10.0,
            badge_w,
            36.0,
        );
        f.rounded_rect(r, 6.0, PANEL);
        let color = if i == 0 { CYAN } else { MAGENTA };
        f.rounded_outline(
            r.min,
            r.size(),
            6.0,
            if g.turn == i && g.phase == Phase::Pegging {
                2.0
            } else {
                1.0
            },
            if g.turn == i && g.phase == Phase::Pegging {
                color
            } else {
                EDGE
            },
        );
        f.text(
            r.min + Vec2::new(9.0, 8.0),
            format!("{label}{}", if g.dealer == i { " / DEALER" } else { "" }),
            if small { 9.0 } else { 12.0 },
            color,
        );
        f.text(
            Vec2::new(r.max.x - 42.0, r.min.y + 7.0),
            format!("{}", g.scores[i]),
            18.0,
            INK,
        );
    }
    let tx = m + 18.0;
    let tw = w - 36.0;
    for i in 0..2 {
        let y = top + 57.0 + i as f32 * 10.0;
        f.line(Vec2::new(tx, y), Vec2::new(tx + tw, y), 1.0, EDGE);
        for tick in 0..=12 {
            let x = tx + tw * tick as f32 / 12.0;
            f.line(Vec2::new(x, y - 2.0), Vec2::new(x, y + 2.0), 1.0, MUTED);
        }
        circle(
            f,
            Vec2::new(tx + tw * (g.scores[i] as f32 / 121.0).min(1.0), y),
            3.5,
            2.0,
            if i == 0 { CYAN } else { MAGENTA },
        );
    }
    // Keep enough room for both the running count and a six-card touch hand.
    let hw = if compact {
        44.0
    } else if small {
        53.0
    } else {
        88.0
    };
    let hh = hw * 1.4;
    let hy = bottom - hh - 61.0;
    let center_y = (top + 83.0 + hy) * 0.5;
    let show = g.phase == Phase::Show;
    let reveal = matches!(
        g.phase,
        Phase::Show | Phase::RoundComplete | Phase::MatchComplete
    );
    if show {
        let step = g.show_step.min(2);
        let owner = [1 - g.dealer, g.dealer, g.dealer][step];
        let cards = if step == 2 { &g.crib } else { &g.hands[owner] };
        let label = if step == 2 {
            "DEALER'S CRIB"
        } else if owner == 0 {
            "YOUR HAND"
        } else {
            "CPU HAND"
        };
        let st = g.starter.expect("validated show has starter");
        let score = hand_score(cards, st, step == 2);
        let cw = if compact {
            28.0
        } else if small {
            39.0
        } else {
            60.0
        };
        let ch = cw * 1.4;
        let y = if compact { top + 91.0 } else { center_y - 60.0 };
        centered(f, Vec2::new(m + w * 0.5, y - 15.0), label, 11.0, CYAN);
        let width = cw * 5.0 + 4.0 * 6.0;
        for (i, card) in cards.iter().copied().chain(std::iter::once(st)).enumerate() {
            card_face(
                f,
                rect(m + (w - width) * 0.5 + i as f32 * (cw + 6.0), y, cw, ch),
                card,
                false,
                true,
            );
        }
        let detail = score
            .breakdown
            .iter()
            .map(|(label, n)| format!("{label} +{n}"))
            .collect::<Vec<_>>();
        // Two columns keep every scoring category visible on narrow screens.
        for (i, line) in detail.iter().enumerate() {
            let col = i % 2;
            let row = i / 2;
            f.text(
                Vec2::new(
                    m + (w - (w - 28.0).min(360.0)) * 0.5
                        + col as f32 * (w - 28.0).min(360.0) * 0.5,
                    y + ch + 8.0 + row as f32 * 15.0,
                ),
                line,
                if small { 8.0 } else { 11.0 },
                INK,
            );
        }
        centered(
            f,
            Vec2::new(
                m + w * 0.5,
                y + ch + 8.0 + detail.len().div_ceil(2) as f32 * 15.0,
            ),
            &format!("COUNT +{}", score.total),
            12.0,
            GOLD,
        );
    } else {
        let cw = if compact {
            23.0
        } else if small {
            34.0
        } else {
            51.0
        };
        let ch = cw * 1.4;
        let row_y = top + 91.0;
        f.text(Vec2::new(m + 14.0, row_y - 13.0), "CRIB", 9.0, MUTED);
        for i in 0..g.crib.len() {
            let mut card = g.crib[i];
            card.face_up = reveal;
            card_face(
                f,
                rect(m + 14.0 + i as f32 * cw * 0.45, row_y, cw, ch),
                card,
                false,
                i == g.crib.len() - 1,
            );
        }
        let starter = rect(m + w * 0.5 - cw * 0.5, row_y, cw, ch);
        f.text(starter.min - Vec2::new(0.0, 13.0), "CUT", 9.0, MUTED);
        card_face(
            f,
            starter,
            g.starter.unwrap_or(Card::new(2, Suit::Spades, false)),
            false,
            true,
        );
        let cpu = if reveal { &g.hands[1] } else { &g.remaining[1] };
        let stride = cw * 0.44;
        let x = m + w - 14.0 - cw - stride * cpu.len().saturating_sub(1) as f32;
        f.text(Vec2::new(x, row_y - 13.0), "CPU", 9.0, MUTED);
        for (i, c) in cpu.iter().enumerate() {
            let mut card = *c;
            card.face_up = reveal;
            card_face(
                f,
                rect(x + i as f32 * stride, row_y, cw, ch),
                card,
                false,
                i == cpu.len() - 1,
            );
        }
        let seq_y = if compact {
            row_y + ch + 19.0
        } else {
            center_y + 25.0
        };
        centered(
            f,
            Vec2::new(m + w * 0.5, seq_y - 18.0),
            &if g.phase == Phase::Discard {
                "CHOOSE TWO CARDS".into()
            } else {
                format!("COUNT {} / 31", g.count)
            },
            if compact { 12.0 } else { 16.0 },
            GOLD,
        );
        let sw = if compact {
            22.0
        } else if small {
            31.0
        } else {
            48.0
        };
        let step = sw + 4.0;
        let width = sw + step * g.sequence.len().saturating_sub(1) as f32;
        for (i, card) in g.sequence.iter().enumerate() {
            let fade = if motion && i + 1 == g.sequence.len() {
                (s.cribbage_ui.age / 0.35).clamp(0.0, 1.0)
            } else {
                1.0
            };
            card_face(
                f,
                rect(
                    m + (w - width) * 0.5 + i as f32 * step,
                    seq_y + (1.0 - fade) * -18.0,
                    sw,
                    sw * 1.4,
                ),
                *card,
                false,
                true,
            );
        }
    }
    let hand = if g.phase == Phase::Pegging {
        &g.remaining[0]
    } else {
        &g.hands[0]
    };
    let n = hand.len();
    let stride = if n > 1 {
        ((w - 26.0 - hw) / (n - 1) as f32).min(hw + 8.0)
    } else {
        0.0
    };
    let span = hw + stride * n.saturating_sub(1) as f32;
    let x = m + (w - span) * 0.5;
    let legal = g.legal_indices(0);
    let selectable = |i: usize| {
        g.phase == Phase::Discard
            && (s.cribbage_ui.selected.len() < 2 || s.cribbage_ui.selected.contains(&i))
            || g.phase == Phase::Pegging && g.turn == 0 && legal.contains(&i)
    };
    let resting: Vec<_> = (0..n)
        .map(|i| rect(x + i as f32 * stride, hy, hw, hh))
        .collect();
    s.cribbage_ui.hovered = if rules || busy {
        None
    } else {
        s.cursor.and_then(|p| {
            (0..n).rev().find(|&i| {
                selectable(i)
                    && card_hit_bounds(
                        resting[i],
                        s.cribbage_ui
                            .positions
                            .get(i)
                            .map_or(resting[i], |p| rect(p.x, p.y, hw, hh)),
                    )
                    .contains(p)
            })
        })
    };
    let mut drawn = Vec::new();
    if s.cribbage_ui.positions.len() != n {
        s.cribbage_ui.positions = resting
            .iter()
            .map(|r| r.min + Vec2::new(0.0, if motion { 15.0 } else { 0.0 }))
            .collect();
    }
    for (i, r) in resting.iter().enumerate() {
        let lift = if s.cribbage_ui.selected.contains(&i) || s.cribbage_ui.hovered == Some(i) {
            18.0
        } else {
            0.0
        };
        let target = r.min - Vec2::new(0.0, lift);
        let amount = if motion {
            (dt.max(0.0) * 22.0).min(1.0)
        } else {
            1.0
        };
        s.cribbage_ui.positions[i] = s.cribbage_ui.positions[i].lerp(target, amount);
        drawn.push(rect(
            s.cribbage_ui.positions[i].x,
            s.cribbage_ui.positions[i].y,
            hw,
            hh,
        ));
    }
    let mut order: Vec<_> = (0..n)
        .filter(|i| !s.cribbage_ui.selected.contains(i) && s.cribbage_ui.hovered != Some(*i))
        .collect();
    order.extend(
        (0..n).filter(|i| s.cribbage_ui.selected.contains(i) || s.cribbage_ui.hovered == Some(*i)),
    );
    for (at, &i) in order.iter().enumerate() {
        let r = drawn[i];
        let rank = rect(r.min.x + 5.0, r.min.y + 5.0, hw * 0.4, 18.0);
        let visible = !order[at + 1..]
            .iter()
            .any(|&j| rects_overlap(rank, drawn[j]));
        card_face_with_emphasis(
            f,
            r,
            hand[i],
            s.cribbage_ui.selected.contains(&i) || s.cribbage_ui.hovered == Some(i),
            visible,
            if selectable(i) || reveal { 1.0 } else { 0.28 },
        );
        if !rules && !busy {
            l.hits.push(Hit {
                rect: card_hit_bounds(r, resting[i]),
                action: Action::CribbageCard(i),
            });
        }
    }
    let status = match g.phase {
        Phase::Discard => {
            if g.dealer == 0 {
                "Choose two for your crib"
            } else {
                "Choose two for CPU's crib"
            }
        }
        Phase::Pegging if g.turn == 0 && !legal.is_empty() => {
            "Your turn / Keep the count at 31 or less"
        }
        Phase::Pegging if g.turn == 0 => "No playable card / Call go",
        Phase::Pegging => "CPU is thinking...",
        Phase::Show => "Non-dealer, dealer, then crib",
        Phase::RoundComplete => "Round scored / Dealer switches next",
        Phase::MatchComplete if g.scores[0] >= 121 => "You win!",
        Phase::MatchComplete => "CPU wins / Try another match",
    };
    f.text(
        Vec2::new(m + 12.0, bottom - 13.0),
        status,
        if small { 9.0 } else { 12.0 },
        INK,
    );
    f.text(
        Vec2::new(m + 12.0, bottom - 44.0),
        &g.last_message,
        if small { 9.0 } else { 11.0 },
        GOLD,
    );
    if !rules && !busy {
        let action = match g.phase {
            Phase::Discard if s.cribbage_ui.selected.len() == 2 => {
                Some(("Discard two", Action::CribbageDiscard))
            }
            Phase::Pegging if g.turn == 0 && legal.is_empty() => {
                Some(("Call go", Action::CribbageGo))
            }
            Phase::Show => Some(("Count score", Action::CribbageContinue)),
            Phase::RoundComplete => Some(("Next round", Action::CribbageContinue)),
            Phase::MatchComplete => Some(("Rematch", Action::CribbageContinue)),
            _ => None,
        };
        if let Some((label, action)) = action {
            button(
                f,
                s,
                l,
                rect(m + w - 116.0, bottom - 61.0, 104.0, 40.0),
                label,
                action,
                true,
            );
        }
    }
    for event in &s.cribbage_ui.feedback.events {
        use crate::table_feedback::EffectKind;
        if !matches!(
            event.kind,
            EffectKind::Run | EffectKind::Win | EffectKind::Stuck
        ) {
            continue;
        }
        let t = (event.age / event.duration()).clamp(0.0, 1.0);
        let color = if event.kind == EffectKind::Stuck {
            RED
        } else {
            GOLD
        };
        let c = color.to_srgba();
        f.rounded_outline(
            table.min + Vec2::splat(4.0),
            table.size() - Vec2::splat(8.0),
            14.0,
            2.0,
            Color::srgba(c.red, c.green, c.blue, 1.0 - t),
        );
        if event.kind == EffectKind::Win && motion {
            for j in 0..36 {
                let phase = j as f32 / 36.0;
                let x = table.min.x + 8.0 + phase * (table.width() - 16.0);
                let y = table.max.y
                    - 10.0
                    - (phase * std::f32::consts::PI).sin()
                        * (100.0 + (j % 4) as f32 * 24.0)
                        * (t * std::f32::consts::PI).sin();
                f.rect(
                    Vec2::new(x, y),
                    Vec2::splat(2.5),
                    Color::srgba(c.red, c.green, c.blue, 1.0 - t),
                );
            }
        }
    }
    if rules {
        let r = rect(m + 8.0, top + 7.0, w - 16.0, bottom - top - 14.0);
        f.rounded_rect(r, 12.0, PANEL);
        f.rounded_outline(r.min, r.size(), 12.0, 1.5, CYAN);
        f.text(r.min + Vec2::new(12.0, 14.0), "HOW TO PLAY", 18.0, CYAN);
        let lines = [
            "Discard two cards to the dealer's crib.",
            "Non-dealer pegs first. Never exceed 31.",
            "15 or 31: +2. Pairs: +2, +6, or +12.",
            "Runs score their length. Go / last card: +1.",
            "Hands: fifteens, pairs, runs and flushes.",
            "Crib flush needs all five cards matching.",
            "Nobs: jack matches cut suit, +1.",
            "Jack cut: dealer +2. First to 121 wins.",
        ];
        for (i, line) in lines.iter().enumerate() {
            f.text(
                r.min + Vec2::new(12.0, 48.0 + i as f32 * if compact { 23.0 } else { 28.0 }),
                *line,
                if small { 9.0 } else { 13.0 },
                INK,
            );
        }
        button(
            f,
            s,
            l,
            rect(r.max.x - 84.0, r.max.y - 48.0, 72.0, 40.0),
            "Close",
            Action::CribbageRules,
            true,
        );
    }
}
fn centered(f: &mut Frame, p: Vec2, text: &str, size: f32, color: Color) {
    f.text(
        p - Vec2::new(text.chars().count() as f32 * size * 0.3, 0.0),
        text,
        size,
        color,
    );
}
