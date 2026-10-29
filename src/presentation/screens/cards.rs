use super::*;

pub(crate) fn cards(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32, dt: f32) {
    let small = w < 620.0;
    f.text(
        Vec2::new(m, 102.0),
        "Card Lab",
        if small { 34.0 } else { 44.0 },
        INK,
    );
    f.text(
        Vec2::new(m, 154.0),
        "An interaction sandbox. No stakes, just a hand of cards.",
        if small { 11.0 } else { 15.0 },
        MUTED,
    );
    let controls_y = 190.0;
    button(
        f,
        s,
        l,
        rect(m, controls_y, 110.0, 34.0),
        "Deal again",
        Action::Deal,
        false,
    );
    button(
        f,
        s,
        l,
        rect(m + 118.0, controls_y, 90.0, 34.0),
        "Flip / F",
        Action::Flip,
        false,
    );
    button(
        f,
        s,
        l,
        rect(
            m + 216.0,
            controls_y,
            if small { 100.0 } else { 132.0 },
            34.0,
        ),
        "Return card",
        Action::ReturnCard,
        false,
    );
    let card_h =
        (((w - 36.0) / 3.0).clamp(82.0, 132.0) * 1.4).min(((f.size.y - 400.0) / 2.0).max(80.0));
    let card_w = card_h / 1.4;
    let slots_y = 256.0;
    let gap = if small { 12.0 } else { 34.0 };
    let start = m + (w - (card_w * 3.0 + gap * 2.0)) * 0.5;
    for i in 0..3 {
        let r = rect(start + i as f32 * (card_w + gap), slots_y, card_w, card_h);
        l.slots[i] = r;
        let hovered = s.lab.drag.is_some() && s.cursor.is_some_and(|p| r.contains(p));
        f.rect(r.min, r.size(), PANEL);
        f.outline(
            r.min,
            r.size(),
            if hovered { 2.0 } else { 1.0 },
            if hovered { GOLD } else { EDGE },
        );
        if s.lab.slots[i].is_none() {
            f.text(
                r.min + Vec2::new(card_w * 0.35, card_h * 0.38),
                format!("{}", i + 1),
                28.0,
                EDGE,
            );
        }
        f.text(
            r.min - Vec2::new(0.0, 23.0),
            format!("Slot {}", i + 1),
            12.0,
            MUTED,
        );
    }
    let hand_y = (f.size.y - card_h - 105.0).max(slots_y + card_h + 36.0);
    let spread = (card_w * 0.65).min((w - card_w) / 4.0);
    let hand_x = m + (w - (card_w + spread * 4.0)) * 0.5;
    let mut order: Vec<usize> = (0..5).collect();
    if let Some(i) = s.lab.selected {
        order.retain(|&v| v != i);
        order.push(i);
    }
    if let Some(i) = s.lab.drag {
        order.retain(|&v| v != i);
        order.push(i);
    }
    // Lifted cards retain their resting footprint so a stationary pointer at
    // the bottom edge cannot repeatedly enter and leave the moving card.
    let hovered = s.cursor.and_then(|p| {
        order
            .iter()
            .rev()
            .copied()
            .find(|&i| s.lab.slot_of(i).is_none() && l.card_hits[i].contains(p))
    });
    for i in order.iter().copied() {
        let mut target = if let Some(slot) = s.lab.slot_of(i) {
            l.slots[slot].min
        } else {
            Vec2::new(hand_x + i as f32 * spread, hand_y)
        };
        let resting = rect(target.x, target.y, card_w, card_h);
        if s.lab.slot_of(i).is_none() && (s.lab.selected == Some(i) || hovered == Some(i)) {
            target.y -= 18.0;
        }
        if s.lab.drag == Some(i)
            && let Some(p) = s.cursor
        {
            target = p - s.drag_offset;
        }
        let reduced = s.save.settings.reduced_motion;
        if s.card_positions[i].x < -900.0 {
            s.card_positions[i] = if reduced {
                target
            } else {
                Vec2::new(hand_x - 80.0, hand_y + 70.0)
            };
        }
        let ready = reduced || s.motion_time > i as f32 * 0.045;
        if ready {
            let t = if reduced || s.lab.drag == Some(i) {
                1.0
            } else {
                1.0 - (-dt.min(0.1) * 22.0).exp()
            };
            let current = s.card_positions[i];
            s.card_positions[i] = current.lerp(target, t);
        }
        let r = rect(s.card_positions[i].x, s.card_positions[i].y, card_w, card_h);
        l.cards[i] = r;
        l.card_hits[i] = if s.lab.slot_of(i).is_none() && s.lab.drag != Some(i) {
            card_hit_bounds(r, resting)
        } else {
            r
        };
    }
    for (stack, &i) in order.iter().enumerate() {
        let rank_bounds = rect(l.cards[i].min.x + 12.0, l.cards[i].min.y + 11.0, 32.0, 27.0);
        let rank_visible = !order[stack + 1..]
            .iter()
            .any(|&later| rects_overlap(rank_bounds, l.cards[later]));
        card_face(
            f,
            l.cards[i],
            s.lab.cards[i],
            s.lab.selected == Some(i) || hovered == Some(i),
            rank_visible,
        );
    }
    l.card_order = order;
    f.text(
        Vec2::new(m, f.size.y - 79.0),
        if small {
            "Drag to slots / 1 / 2 / 3 place"
        } else {
            "Drag to a slot. Invalid drops snap back. Click a card to select it."
        },
        12.0,
        INK,
    );
    f.text(
        Vec2::new(m, f.size.y - 57.0),
        if small {
            "Arrows select / right-click or F flips"
        } else {
            "Left/Right select   Right-click or F flip   1 / 2 / 3 place in slot   Backspace returns card   Esc lobby"
        },
        12.0,
        MUTED,
    );
}

pub(crate) fn card_hit_bounds(card: Rect, resting: Rect) -> Rect {
    Rect {
        min: card.min.min(resting.min),
        max: card.max.max(resting.max),
    }
}

pub(crate) fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}

pub(crate) fn card_face(
    f: &mut Frame,
    r: Rect,
    card: gamevex_core::cards::Card,
    selected: bool,
    rank_visible: bool,
) {
    let compact = r.width() < 100.0 || r.height() < 140.0;
    f.rect(
        r.min
            + if compact {
                Vec2::new(2.0, 3.0)
            } else {
                Vec2::new(4.0, 7.0)
            },
        r.size(),
        Color::srgb(0.0, 0.01, 0.025),
    );
    f.rect(r.min, r.size(), Color::srgb(0.025, 0.055, 0.095));
    f.outline(
        r.min,
        r.size(),
        if selected { 2.5 } else { 1.5 },
        if selected { GOLD } else { CYAN },
    );
    let inset_size = if compact { 4.0 } else { 7.0 };
    let inset = r.min + Vec2::splat(inset_size);
    f.outline(inset, r.size() - Vec2::splat(inset_size * 2.0), 0.75, EDGE);
    if card.face_up {
        let color = if card.suit.is_red() { MAGENTA } else { CYAN };
        // Rank stays on the exposed left edge of an overlapping hand.
        if rank_visible {
            let rank_size = if compact {
                (r.width() * 0.28).clamp(11.0, 16.0)
            } else {
                22.0
            };
            let rank_inset = if compact { 5.0 } else { 12.0 };
            f.text(
                r.min + Vec2::splat(rank_inset),
                card.rank_label(),
                rank_size,
                color,
            );
        }
        if !compact || r.height() >= 85.0 {
            if compact {
                suit(
                    f,
                    r.min + Vec2::new(r.width() * 0.28, r.height() * 0.36),
                    (r.width() * 0.12).max(4.0),
                    card.suit,
                    color,
                );
            } else {
                suit(f, r.min + Vec2::new(20.0, 49.0), 8.0, card.suit, color);
            }
        }
        // The large central suit uses geometry, so later cards cover it naturally.
        suit(
            f,
            if compact {
                r.center()
            } else {
                r.center() + Vec2::new(5.0, 10.0)
            },
            if compact {
                (r.width() * 0.18).max(3.0)
            } else {
                r.width() * 0.22
            },
            card.suit,
            color,
        );
    } else {
        for i in 0..5 {
            let d = 8.0 + i as f32 * 8.0;
            let p = r.min + Vec2::new(d, d);
            let size = r.size() - Vec2::splat(d * 2.0);
            if size.min_element() > 0.0 {
                f.outline(p, size, 1.0, if i % 2 == 0 { MAGENTA } else { EDGE });
            }
        }
    }
}
