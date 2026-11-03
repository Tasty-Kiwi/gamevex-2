use super::screens::{button, card_face, card_face_with_emphasis, card_hit_bounds, rects_overlap};
use super::*;
use crate::hearts_motion::{Anchor, Kind};
use gamevex_core::hearts::Phase;

const SEATS: [&str; 4] = ["You", "Left", "Across", "Right"];

pub(super) fn hearts(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32, dt: f32) {
    let theme = *f.theme();
    let table_pal = super::game_palette::table(f.palette);
    let g = s.save.hearts.clone();
    let busy = s.hearts_ui.motion.busy();
    let small = w < 620.0;
    let compact = f.size.y <= 640.0;
    f.text(
        Vec2::new(m, 82.0),
        "Hearts",
        if small { 26.0 } else { 36.0 },
        theme.ink,
    );
    f.text(
        Vec2::new(m, if small { 114.0 } else { 126.0 }),
        format!(
            "Hand {} / Pass {} / First to 100 ends the match",
            g.hand_number,
            g.pass_direction().label()
        ),
        if small { 9.0 } else { 12.0 },
        theme.muted,
    );

    let controls_y = if small { 132.0 } else { 94.0 };
    let controls_x = if small { m } else { m + w - 230.0 };
    let selected = s.hearts_ui.selected.len();
    let pass_rect = rect(controls_x, controls_y, 88.0, 30.0);
    if g.phase == Phase::Passing && selected == 3 && !busy {
        button(f, s, l, pass_rect, "Pass cards", Action::HeartsPass, true);
    } else {
        muted_button(f, pass_rect, &format!("Pass {selected}/3"));
    }
    button(
        f,
        s,
        l,
        rect(
            controls_x + 96.0,
            controls_y,
            if small { 100.0 } else { 134.0 },
            30.0,
        ),
        "New match",
        Action::HeartsNew,
        false,
    );

    let top = if small { 174.0 } else { 154.0 };
    let bottom = f.size.y - 58.0;
    let table = rect(m, top, w, bottom - top);
    rounded(
        f,
        rect(m - 5.0, top - 5.0, w + 10.0, table.height() + 10.0),
        24.0,
        theme.panel,
    );
    rounded(f, table, 20.0, table_pal.round_surface);
    // Keep the table in the same dark, cool palette as the other cabinets.
    rounded(
        f,
        rect(m + 7.0, top + 7.0, w - 14.0, table.height() - 14.0),
        16.0,
        table_pal.round_inset,
    );
    f.rounded_outline(table.min, table.size(), 20.0, 1.5, theme.edge);

    let card_w = if small {
        if compact { 58.0 } else { 66.0 }
    } else {
        (w / 10.0).clamp(78.0, 104.0)
    };
    let card_h = card_w * 1.38;
    let rows = if small { 2 } else { 1 };
    let row_gap = if small { card_h * 0.82 } else { 0.0 };
    let hand_top = bottom - card_h - row_gap - 12.0;
    let center = Vec2::new(
        m + w * 0.5,
        if compact {
            hand_top - 55.0
        } else {
            (top + hand_top) * 0.5 + 18.0
        },
    );
    let face_w = if compact {
        32.0
    } else if small {
        51.0
    } else {
        78.0
    };
    let face_h = face_w * 1.4;

    // Seat labels belong next to their hands, rather than a separate scoreboard.
    let badge_w = if small { 86.0 } else { 132.0 };
    let top_badge = rect(
        m + w * 0.5 - badge_w * 0.5,
        top + 8.0,
        badge_w,
        if compact { 23.0 } else { 32.0 },
    );
    seat(
        f,
        top_badge,
        2,
        &g,
        small,
        s.hearts_ui.motion.kind() == Some(Kind::Collect) && g.turn == 2,
    );
    let top_back_w = if compact {
        19.0
    } else if small {
        28.0
    } else {
        43.0
    };
    let top_back_h = if compact {
        23.0
    } else if small {
        36.0
    } else {
        56.0
    };
    let step = if compact {
        7.0
    } else if small {
        9.0
    } else {
        15.0
    };
    let mut hand_anchors = [rect(center.x - card_w * 0.5, hand_top, card_w, card_h); 4];
    let n = g.hands[2]
        .len()
        .saturating_sub(pending(&s.hearts_ui.motion, 2));
    let span = top_back_w + step * n.saturating_sub(1) as f32;
    hand_anchors[2] = rect(
        m + w * 0.5 - top_back_w * 0.5,
        top_badge.max.y + 4.0,
        top_back_w,
        top_back_h,
    );
    for i in 0..n {
        back(
            f,
            rect(
                m + w * 0.5 - span * 0.5 + i as f32 * step,
                top_badge.max.y + 4.0,
                top_back_w,
                top_back_h,
            ),
        );
    }
    let side_y = center.y - if compact { 40.0 } else { 65.0 };
    for player in [1, 3] {
        let x = if player == 1 {
            m + 10.0
        } else {
            m + w - badge_w - 10.0
        };
        seat(
            f,
            rect(x, side_y - 34.0, badge_w, 32.0),
            player,
            &g,
            small,
            s.hearts_ui.motion.kind() == Some(Kind::Collect) && g.turn == player,
        );
        let n = g.hands[player]
            .len()
            .saturating_sub(pending(&s.hearts_ui.motion, player));
        let bw = if compact {
            30.0
        } else if small {
            39.0
        } else {
            66.0
        };
        let bh = if compact {
            21.0
        } else if small {
            27.0
        } else {
            45.0
        };
        let step = if compact {
            5.0
        } else if small {
            8.0
        } else {
            12.0
        };
        let bx = if player == 1 {
            m + 12.0
        } else {
            m + w - bw - 12.0
        };
        hand_anchors[player] = rect(
            bx,
            side_y + (n.saturating_sub(1) as f32 * step) * 0.5,
            bw,
            bh,
        );
        for i in 0..n {
            let bx = if player == 1 {
                m + 12.0
            } else {
                m + w - bw - 12.0
            };
            back(f, rect(bx, side_y + i as f32 * step, bw, bh));
        }
    }

    let targets = trick_targets(center, face_w, face_h);
    for (seat, target) in targets.into_iter().enumerate() {
        if let Some(play) = g.trick.iter().find(|play| play.seat == seat) {
            if s.hearts_ui
                .motion
                .flights()
                .iter()
                .any(|flight| flight.kind == Kind::Play && flight.to == Anchor::Trick(seat))
            {
                continue;
            }
            face(f, target, play.card, false, false);
        }
    }
    if g.trick.is_empty() && !busy {
        let prompt = match g.phase {
            Phase::Passing => "Choose three cards",
            Phase::Playing if g.turn == 0 => "Your lead",
            Phase::Playing => "Waiting for the lead",
            Phase::HandComplete => "Hand complete",
            Phase::MatchComplete => "Match complete",
            Phase::TrickComplete => "",
        };
        centered(
            f,
            Vec2::new(center.x, center.y - 14.0),
            prompt,
            if small { 12.0 } else { 18.0 },
            theme.ink,
        );
        if g.phase == Phase::Passing {
            centered(
                f,
                Vec2::new(center.x, center.y + 10.0),
                &format!("Passing {}", g.pass_direction().label()),
                if small { 10.0 } else { 13.0 },
                theme.muted,
            );
        }
    }

    let human_badge = rect(
        m + 12.0,
        hand_top - 37.0,
        if small { 105.0 } else { 160.0 },
        32.0,
    );
    seat(
        f,
        human_badge,
        0,
        &g,
        small,
        s.hearts_ui.motion.kind() == Some(Kind::Collect) && g.turn == 0,
    );
    if g.hearts_broken {
        f.text(
            Vec2::new(m + w - if small { 104.0 } else { 154.0 }, hand_top - 26.0),
            "Hearts broken",
            if small { 10.0 } else { 13.0 },
            theme.ink,
        );
    }
    let legal = g.legal_indices(0);
    let cards = &g.hands[0];
    let columns = if small { 7 } else { 13 };
    let mut resting = Vec::with_capacity(cards.len());
    for row in 0..rows {
        let start = row * columns;
        let count = cards.len().saturating_sub(start).min(columns);
        if count == 0 {
            continue;
        }
        let spread = (w - if small { 24.0 } else { 90.0 } - card_w)
            / columns.saturating_sub(1).max(1) as f32;
        let step = spread.min(card_w * 0.68);
        let width = card_w + step * count.saturating_sub(1) as f32;
        let left = m + (w - width) * 0.5;
        for col in 0..count {
            let offset = (col as f32 - (count - 1) as f32 * 0.5) / count.max(1) as f32;
            let arc = offset * offset * if small { 12.0 } else { 30.0 };
            resting.push(rect(
                left + col as f32 * step,
                hand_top + row as f32 * row_gap + arc,
                card_w,
                card_h,
            ));
        }
    }
    if s.hearts_ui.hand_cards != *cards || s.hearts_ui.card_positions.len() != cards.len() {
        let positions = cards
            .iter()
            .enumerate()
            .map(|(i, card)| {
                s.hearts_ui
                    .hand_cards
                    .iter()
                    .position(|old| old == card)
                    .and_then(|old| s.hearts_ui.card_positions.get(old).copied())
                    .unwrap_or(resting[i].min)
            })
            .collect();
        s.hearts_ui.hand_cards = cards.clone();
        s.hearts_ui.card_positions = positions;
        s.hearts_ui.hovered = None;
    }
    let mut order: Vec<_> = (0..cards.len()).collect();
    // Card Lab paints selected cards last, and picking follows that same order.
    order.sort_by_key(|i| s.hearts_ui.selected.contains(i));
    s.hearts_ui.hovered = if busy {
        None
    } else {
        s.cursor.and_then(|point| {
            order.iter().rev().copied().find(|&i| {
                let current = Rect::from_corners(
                    s.hearts_ui.card_positions[i],
                    s.hearts_ui.card_positions[i] + resting[i].size(),
                );
                card_hit_bounds(current, resting[i]).contains(point)
            })
        })
    };
    let t = if s.save.settings.reduced_motion {
        1.0
    } else {
        1.0 - (-dt.clamp(0.0, 0.1) * 22.0).exp()
    };
    let drawn: Vec<_> = resting
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let selectable = !busy
                && ((g.phase == Phase::Passing
                    && (s.hearts_ui.selected.len() < 3 || s.hearts_ui.selected.contains(&i)))
                    || legal.contains(&i));
            let lifted =
                s.hearts_ui.selected.contains(&i) || (s.hearts_ui.hovered == Some(i) && selectable);
            let target = r.min - Vec2::new(0.0, if lifted { 18.0 } else { 0.0 });
            s.hearts_ui.card_positions[i] = s.hearts_ui.card_positions[i].lerp(target, t);
            Rect::from_corners(
                s.hearts_ui.card_positions[i],
                s.hearts_ui.card_positions[i] + r.size(),
            )
        })
        .collect();
    let arriving: Vec<_> = cards
        .iter()
        .map(|card| {
            s.hearts_ui.motion.flights().iter().any(|flight| {
                matches!(flight.kind, Kind::Pass | Kind::Deal)
                    && flight.to == Anchor::Hand(0)
                    && flight.card == *card
                    && flight.progress() < 1.0
            })
        })
        .collect();
    for (stack, &index) in order.iter().enumerate() {
        let selected = s.hearts_ui.selected.contains(&index);
        let hovered = s.hearts_ui.hovered == Some(index);
        let allowed = !busy
            && ((g.phase == Phase::Passing && (s.hearts_ui.selected.len() < 3 || selected))
                || legal.contains(&index));
        let r = drawn[index];
        if arriving[index] {
            continue;
        }
        let rank_bounds = rect(r.min.x + 5.0, r.min.y + 5.0, 32.0, 27.0);
        let rank_visible = !order[stack + 1..]
            .iter()
            .any(|&later| !arriving[later] && rects_overlap(rank_bounds, drawn[later]));
        card_face_with_emphasis(
            f,
            r,
            cards[index],
            selected || (hovered && allowed),
            rank_visible,
            if allowed { 1.0 } else { 0.28 },
        );
        if !busy {
            // Include disabled cards as blockers so clicks cannot pass through
            // their visible faces to a legal card underneath. The model rejects
            // illegal plays; diagnostics expose only enabled card actions.
            l.hits.push(Hit {
                rect: card_hit_bounds(r, resting[index]),
                action: Action::HeartsCard(index),
            });
        }
    }
    let mut flight_art = Vec::with_capacity(s.hearts_ui.motion.flights().len());
    for flight in s.hearts_ui.motion.flights() {
        let t = flight.progress();
        if t >= 1.0 {
            continue;
        }
        let anchor = |a: Anchor| match a {
            Anchor::Hand(seat) => hand_anchors[seat.min(3)],
            Anchor::Trick(seat) => targets[seat.min(3)],
            Anchor::Deck => rect(
                center.x - face_w * 0.5,
                center.y - face_h * 0.5,
                face_w,
                face_h,
            ),
        };
        let mut from = anchor(flight.from);
        if let Some(p) = flight.from_position {
            from = Rect::from_corners(p, p + Vec2::new(card_w, card_h));
        }
        let mut to = anchor(flight.to);
        if flight.to == Anchor::Hand(0)
            && matches!(flight.kind, Kind::Pass | Kind::Deal)
            && let Some(index) = cards.iter().position(|card| *card == flight.card)
        {
            to = drawn[index];
        }
        if flight.kind == Kind::Collect {
            let size = to.size() * 0.58;
            to = Rect::from_center_size(to.center(), size);
        }
        let delta = to.center() - from.center();
        let bend = if delta.length() > 1.0 {
            Vec2::new(-delta.y, delta.x).normalize()
                * delta.length().min(240.0)
                * 0.075
                * (std::f32::consts::PI * t).sin()
        } else {
            Vec2::ZERO
        };
        let size = from.size().lerp(to.size(), t);
        let pos = from.min.lerp(to.min, t) + bend;
        let mut r = Rect::from_corners(pos, pos + size);
        let flip_at = match flight.kind {
            Kind::Collect => Some(0.85),
            Kind::Play if flight.from != Anchor::Hand(0) => Some(0.20),
            Kind::Pass if flight.from == Anchor::Hand(0) || flight.to == Anchor::Hand(0) => {
                Some(0.65)
            }
            Kind::Deal if flight.to == Anchor::Hand(0) => Some(0.65),
            _ => None,
        };
        if let Some(mid) = flip_at {
            let width = r.width() * ((t - mid).abs() / 0.10).clamp(0.04, 1.0);
            r = Rect::from_center_size(r.center(), Vec2::new(width, r.height()));
        }
        let show_face = match flight.kind {
            Kind::Collect => t < 0.85,
            Kind::Play => flight.from == Anchor::Hand(0) || t > 0.20,
            Kind::Pass if flight.from == Anchor::Hand(0) => t < 0.65,
            Kind::Pass | Kind::Deal => flight.to == Anchor::Hand(0) && t > 0.65,
        };
        flight_art.push((r, flight.card, show_face && r.width() >= 12.0));
    }
    for (stack, (r, card, show_face)) in flight_art.iter().copied().enumerate() {
        if show_face {
            let rank_bounds = rect(r.min.x + 5.0, r.min.y + 5.0, 36.0, 30.0);
            let visible = !flight_art[stack + 1..]
                .iter()
                .any(|(later, _, _)| rects_overlap(rank_bounds, *later));
            card_face(f, r, card, false, visible);
        } else {
            back(f, r);
        }
    }
    // Status is outside the hand, so no text competes with the card corners.
    let status = match s.hearts_ui.motion.kind() {
        Some(Kind::Collect) if g.turn == 0 => "You take the trick.".to_owned(),
        Some(Kind::Collect) => format!("{} takes the trick.", SEATS[g.turn]),
        Some(Kind::Pass) => format!("Passing cards {}...", g.pass_direction().label()),
        Some(Kind::Deal) => "Dealing a new hand...".to_owned(),
        _ => match g.phase {
            Phase::Passing => "Select 3 cards, then pass.",
            Phase::Playing if g.turn == 0 => "Your turn. Follow suit when you can.",
            Phase::Playing => "Opponent's turn...",
            Phase::TrickComplete => "Trick complete.",
            Phase::HandComplete => "Scores updated. Ready for the next hand?",
            Phase::MatchComplete => "Lowest score wins.",
        }
        .to_owned(),
    };
    f.text(
        Vec2::new(m, f.size.y - 48.0),
        status,
        if small { 10.0 } else { 12.0 },
        theme.muted,
    );

    if !busy && matches!(g.phase, Phase::HandComplete | Phase::MatchComplete) {
        let label = if g.phase == Phase::MatchComplete {
            "New match"
        } else {
            "Next hand"
        };
        let result = if let Some(winner) = g.winner() {
            format!("{} wins with {} points", SEATS[winner], g.totals[winner])
        } else {
            format!("Your total: {} points", g.totals[0])
        };
        centered(
            f,
            Vec2::new(center.x, center.y + 12.0),
            &result,
            if small { 11.0 } else { 16.0 },
            theme.ink,
        );
        button(
            f,
            s,
            l,
            rect(center.x - 60.0, center.y + 44.0, 120.0, 32.0),
            label,
            Action::HeartsContinue,
            true,
        );
    }
}

fn seat(
    f: &mut Frame,
    r: Rect,
    seat: usize,
    g: &gamevex_core::hearts::Game,
    small: bool,
    receiving: bool,
) {
    let theme = *f.theme();
    let active = (g.phase == Phase::Playing && g.turn == seat) || receiving;
    if active {
        let active_bg = super::game_palette::table(f.palette).seat_active_bg;
        rounded(f, r, 6.0, active_bg);
        f.rounded_outline(r.min, r.size(), 6.0, 1.5, theme.accent);
    }
    f.text(
        r.min + Vec2::new(5.0, 2.0),
        format!("{}  {}", SEATS[seat], g.totals[seat]),
        if small { 11.0 } else { 16.0 },
        if active { theme.accent } else { theme.ink },
    );
    if r.height() > 25.0 {
        f.text(
            r.min + Vec2::new(5.0, if small { 17.0 } else { 21.0 }),
            format!(
                "+{} hand / {} cards",
                g.hand_points[seat],
                g.hands[seat].len()
            ),
            if small { 8.0 } else { 10.0 },
            theme.muted,
        );
    }
}
fn centered(f: &mut Frame, p: Vec2, text: &str, size: f32, color: Color) {
    f.text(
        p - Vec2::new(text.chars().count() as f32 * size * 0.30, 0.0),
        text,
        size,
        color,
    );
}
fn muted_button(f: &mut Frame, r: Rect, label: &str) {
    let theme = *f.theme();
    f.rect(r.min, r.size(), theme.panel);
    f.outline(r.min, r.size(), 1.0, theme.edge);
    centered(
        f,
        Vec2::new(r.center().x, r.min.y + 7.0),
        label,
        12.0,
        theme.muted,
    );
}

// The cabinet shares the exact face/back artwork with the other card games.
fn face(f: &mut Frame, r: Rect, card: gamevex_core::cards::Card, selected: bool, dimmed: bool) {
    card_face(f, r, card, selected, true);
    if dimmed {
        let dim = super::game_palette::card(f.palette).card_dim;
        f.rect(r.min, r.size(), dim);
    }
}
fn back(f: &mut Frame, r: Rect) {
    card_face(
        f,
        r,
        gamevex_core::cards::Card::new(2, Suit::Spades, false),
        false,
        false,
    );
}
fn rounded(f: &mut Frame, r: Rect, radius: f32, color: Color) {
    f.rounded_rect(r, radius, color);
}

fn pending(motion: &crate::hearts_motion::HeartsMotion, seat: usize) -> usize {
    motion
        .flights()
        .iter()
        .filter(|flight| {
            matches!(flight.kind, Kind::Pass | Kind::Deal)
                && flight.to == Anchor::Hand(seat)
                && flight.progress() < 1.0
        })
        .count()
}

/// Keep the top and bottom cards compact while offsetting the side cards far
/// enough to leave a clear gap between every pair.
fn trick_targets(center: Vec2, card_w: f32, card_h: f32) -> [Rect; 4] {
    [
        rect(center.x - card_w * 0.5, center.y + 5.0, card_w, card_h),
        rect(
            center.x - card_w * 1.5 - 6.0,
            center.y - card_h * 0.5,
            card_w,
            card_h,
        ),
        rect(
            center.x - card_w * 0.5,
            center.y - card_h - 5.0,
            card_w,
            card_h,
        ),
        rect(
            center.x + card_w * 0.5 + 6.0,
            center.y - card_h * 0.5,
            card_w,
            card_h,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trick_cards_are_separated_at_desktop_and_compact_sizes() {
        for (w, h) in [(32.0, 44.8), (51.0, 71.4), (78.0, 109.2)] {
            let targets = trick_targets(Vec2::new(300.0, 400.0), w, h);
            for left in 0..targets.len() {
                for right in left + 1..targets.len() {
                    assert!(
                        !rects_overlap(targets[left], targets[right]),
                        "trick targets {left} and {right} overlap at {w}x{h}"
                    );
                }
            }
            assert_eq!(targets[0].center().x, targets[2].center().x);
            assert_eq!(targets[1].center().y, targets[3].center().y);
            assert!((targets[2].min.y - (400.0 - h - 5.0)).abs() < 0.001);
            assert!((targets[0].max.y - (400.0 + 5.0 + h)).abs() < 0.001);
        }
    }

    #[test]
    fn hearts_seat_and_dim_colors_valid_for_all_schemes() {
        for scheme in [
            ColorScheme::Cyan,
            ColorScheme::Amber,
            ColorScheme::Emerald,
            ColorScheme::Violet,
            ColorScheme::Ice,
            ColorScheme::Paper,
        ] {
            let active_bg = crate::presentation::game_palette::table(scheme)
                .seat_active_bg
                .to_srgba();
            assert!(active_bg.red.is_finite() && active_bg.alpha > 0.0);
            let dim = crate::presentation::game_palette::card(scheme)
                .card_dim
                .to_srgba();
            assert!(dim.red.is_finite() && dim.alpha > 0.0);
        }
    }
}
