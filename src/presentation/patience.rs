use super::*;
use gamevex_core::{
    cards::Card,
    patience::{Pile, Variant},
};

pub(super) fn patience(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let theme = *f.theme();
    let game = s.patience_game();
    let ui = &s.patience_ui;
    let variant = game.variant;
    let dragging = ui.drag.filter(|d| d.active).map(|d| d.source);
    let active_source = dragging.or(ui.selected);
    let lifted = |pile, index| dragging.is_some_and(|(p, i)| p == pile && index >= i);
    let title_y = 96.0;
    f.text(
        Vec2::new(m, title_y),
        match s.screen {
            Screen::Solitaire => "KLONDIKE",
            Screen::FreeCell => "FREECELL",
            _ => "SPIDER",
        },
        if w < 460.0 { 25.0 } else { 31.0 },
        theme.ink,
    );
    f.text(
        Vec2::new(m, title_y + 38.0),
        format!(
            "{} MOVES  /  {}",
            game.moves,
            super::format_time(game.elapsed_ms)
        ),
        11.0,
        theme.accent,
    );
    let record_index = Variant::ALL
        .iter()
        .position(|candidate| *candidate == variant)
        .unwrap_or(0);
    let best = s.save.patience_records[record_index]
        .map(super::format_time)
        .unwrap_or_else(|| "--:--".into());
    f.text(
        Vec2::new(m + w * 0.54, title_y + 38.0),
        format!(
            "BEST {best}{}",
            if game.assisted { " / ASSISTED" } else { "" }
        ),
        10.0,
        if game.assisted {
            theme.gold
        } else {
            theme.muted
        },
    );

    let mode_y = title_y + 56.0;
    let (modes, labels): (&[Variant], &[&str]) = match s.screen {
        Screen::Solitaire => (
            &[Variant::Klondike1, Variant::Klondike3],
            &["Draw 1", "Draw 3"],
        ),
        Screen::FreeCell => (
            &[Variant::FreeCell, Variant::FreeCellRelaxed],
            &["Standard", "Relaxed"],
        ),
        _ => (
            &[Variant::Spider1, Variant::Spider2, Variant::Spider4],
            &["1 Suit", "2 Suits", "4 Suits"],
        ),
    };
    let gap = 5.0;
    let bw =
        ((w - gap * (modes.len().saturating_sub(1) as f32)) / modes.len().max(1) as f32).min(100.0);
    for (i, (&mode, &label)) in modes.iter().zip(labels).enumerate() {
        button(
            f,
            s,
            l,
            rect(m + i as f32 * (bw + gap), mode_y, bw, 28.0),
            label,
            Action::PatienceNew(mode, false),
            variant == mode,
        );
    }
    let controls_y = mode_y + 33.0;
    let compact_controls = w < 620.0;
    let columns = if compact_controls { 3 } else { 6 };
    let control_height = if compact_controls { 33.0 } else { 29.0 };
    let row_gap = if compact_controls { gap } else { 0.0 };
    let available_control_width = (w - gap * (columns - 1) as f32) / columns as f32;
    let cw = if compact_controls {
        available_control_width
    } else {
        available_control_width.min(86.0)
    };
    let actions = [
        (
            "Deal",
            Action::PatienceDeal,
            !game.won() && !variant.is_freecell(),
        ),
        ("Undo", Action::PatienceUndo, game.can_undo()),
        ("Hint", Action::PatienceHint, !game.won()),
        ("Finish", Action::PatienceFinish, ui.can_finish),
        ("Restart", Action::PatienceNew(variant, true), true),
        ("New", Action::PatienceNew(variant, false), true),
    ];
    for (i, (label, action, enabled)) in actions.into_iter().enumerate() {
        let column = i % columns;
        let row = i / columns;
        let r = rect(
            m + column as f32 * (cw + gap),
            controls_y + row as f32 * (control_height + row_gap),
            cw,
            control_height,
        );
        if enabled {
            button(f, s, l, r, label, action, false);
        } else {
            f.rect(r.min, r.size(), theme.panel);
            f.outline(r.min, r.size(), 1.0, theme.edge);
            f.text(r.min + Vec2::new(5.0, 8.0), label, 11.0, theme.muted);
        }
    }

    let clip_top = controls_y
        + if compact_controls {
            2.0 * control_height + row_gap + 8.0
        } else {
            control_height + 7.0
        };
    let clip_bottom = f.size.y - 75.0;
    let clip = rect(m, clip_top, w, (clip_bottom - clip_top).max(0.0));
    l.patience_clip = Some(clip);
    let count = variant.columns();
    let gapx = 5.0;
    let card_w = ((w - gapx * (count.saturating_sub(1) as f32)) / count as f32).clamp(48.0, 96.0);
    let card_h = (card_w * 1.42).max(68.0);
    let stride = card_w + gapx;
    let content_w = count as f32 * card_w + count.saturating_sub(1) as f32 * gapx;
    let tableau_extent = game
        .tableau
        .iter()
        .map(|column| {
            let advances = column
                .iter()
                .map(|card| if card.face_up { 19.0 } else { 12.0 })
                .sum::<f32>();
            advances + card_h
        })
        .fold(card_h, f32::max);
    // Reserve the full top row and gap, then the final card's entire height.
    let content_h = 3.0 + card_h + 12.0 + tableau_extent;
    l.patience_max_pan = Vec2::new(
        (content_w - clip.width()).max(0.0),
        (content_h - clip.height()).max(0.0),
    );
    let pan = ui.pan.clamp(Vec2::ZERO, l.patience_max_pan);
    f.rect(
        clip.min,
        clip.size(),
        super::game_palette::table(f.palette).patience_felt,
    );
    f.outline(clip.min, clip.size(), 1.0, theme.edge);
    let top_y = clip.min.y - pan.y;
    let x0 = clip.min.x + (clip.width() - content_w).max(0.0) * 0.5 - pan.x;

    // Upper row differs by game: FreeCell has four cells plus four homes,
    // Spider has eight completed-run slots and the stock, and Klondike has
    // stock, waste, and four foundations.
    let slot_y = top_y + 3.0;
    if variant.is_freecell() {
        for i in 0..8 {
            let r = rect(x0 + i as f32 * stride, slot_y, card_w, card_h);
            if i < 4 {
                let p = Pile::Cell(i);
                match game.cells[i] {
                    Some(card) => {
                        if !lifted(p, 0) && !ui.motion.hides(p, 0) {
                            draw_card(f, clip, r, card, active_source == Some((p, 0)), false, true);
                        }
                        add_card_hit(l, clip, r, p, 0);
                    }
                    None => pile_slot(f, l, s, clip, r, p, false),
                }
            } else {
                let p = Pile::Foundation(i - 4);
                if let Some((idx, card)) = game.foundations[i - 4]
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(idx, _)| !ui.motion.hides(p, *idx) && !lifted(p, *idx))
                {
                    draw_card(
                        f,
                        clip,
                        r,
                        *card,
                        ui.selected == Some((p, idx)),
                        false,
                        true,
                    );
                }
                if let Some((idx, _)) = game.foundations[i - 4].iter().enumerate().next_back() {
                    add_card_hit(l, clip, r, p, idx);
                } else {
                    pile_slot(f, l, s, clip, r, p, false);
                }
            }
        }
    } else if matches!(
        variant,
        Variant::Spider1 | Variant::Spider2 | Variant::Spider4
    ) {
        for i in 0..8 {
            let r = rect(x0 + i as f32 * stride, slot_y, card_w, card_h);
            if let Some(run) = game.completed.get(i) {
                if let Some(card) = run.last().copied() {
                    draw_card(f, clip, r, card, false, false, true);
                }
            } else {
                pile_slot_visual(f, clip, r);
            }
        }
        pile_slot(
            f,
            l,
            s,
            clip,
            rect(x0 + stride * 8.0, slot_y, card_w, card_h),
            Pile::Stock,
            true,
        );
    } else {
        pile_slot(
            f,
            l,
            s,
            clip,
            rect(x0, slot_y, card_w, card_h),
            Pile::Stock,
            true,
        );
        if let Some((visible_index, card)) = game
            .waste
            .iter()
            .copied()
            .enumerate()
            .rev()
            .find(|(idx, _)| !ui.motion.hides(Pile::Waste, *idx) && !lifted(Pile::Waste, *idx))
        {
            draw_card(
                f,
                clip,
                rect(x0 + stride, slot_y, card_w, card_h),
                card,
                ui.selected == Some((Pile::Waste, visible_index)),
                false,
                true,
            );
        }
        if let Some(index) = game.waste.len().checked_sub(1) {
            add_card_hit(
                l,
                clip,
                rect(x0 + stride, slot_y, card_w, card_h),
                Pile::Waste,
                index,
            );
        } else {
            pile_slot(
                f,
                l,
                s,
                clip,
                rect(x0 + stride, slot_y, card_w, card_h),
                Pile::Waste,
                false,
            );
        }
        let home_start = x0 + stride * 3.0;
        for i in 0..4 {
            let p = Pile::Foundation(i);
            let r = rect(home_start + i as f32 * stride, slot_y, card_w, card_h);
            if let Some((idx, card)) = game.foundations[i]
                .iter()
                .enumerate()
                .rev()
                .find(|(idx, _)| !ui.motion.hides(p, *idx) && !lifted(p, *idx))
            {
                draw_card(
                    f,
                    clip,
                    r,
                    *card,
                    ui.selected == Some((p, idx)),
                    false,
                    true,
                );
            }
            if let Some((idx, _)) = game.foundations[i].iter().enumerate().next_back() {
                add_card_hit(l, clip, r, p, idx);
            } else {
                pile_slot(f, l, s, clip, r, p, false);
            }
        }
    }
    let tableau_y = slot_y + card_h + 12.0;
    for col in 0..count {
        let p = Pile::Tableau(col);
        let x = x0 + col as f32 * stride;
        let empty = rect(x, tableau_y, card_w, card_h);
        if game.tableau[col].is_empty() {
            pile_slot(f, l, s, clip, empty, p, false);
        }
        let mut y = tableau_y;
        for (index, card) in game.tableau[col].iter().copied().enumerate() {
            let r = rect(x, y, card_w, card_h);
            if r.max.y > clip.min.y
                && r.min.y < clip.max.y
                && r.max.x > clip.min.x
                && r.min.x < clip.max.x
            {
                let selected = active_source.is_some_and(|(src, at)| src == p && index >= at);
                let legal = active_source.is_some_and(|(src, at)| game.legal_move(src, at, p));
                if !lifted(p, index) && !ui.motion.hides(p, index) {
                    draw_card(
                        f,
                        clip,
                        r,
                        card,
                        selected,
                        legal,
                        index + 1 == game.tableau[col].len(),
                    );
                }
                add_card_hit(l, clip, r, p, index);
            }
            y += if card.face_up { 19.0 } else { 12.0 };
        }
        // Empty targets and column tails accept a selected stack.
        let last_step = game.tableau[col]
            .last()
            .map(|c| if c.face_up { 19.0 } else { 12.0 })
            .unwrap_or(card_h);
        let tail = rect(x, y + card_h - last_step, card_w, 18.0);
        add_target_hit(
            l,
            clip,
            if game.tableau[col].is_empty() {
                empty
            } else {
                tail
            },
            p,
        );
    }
    for flight in ui.motion.flights() {
        let t = ((flight.elapsed - flight.delay) / flight.duration).clamp(0.0, 1.0);
        let ease = t * t * (3.0 - 2.0 * t);
        let origin = |pile: Pile, stack_y: f32| -> Option<Vec2> {
            match pile {
                Pile::Tableau(i) => Some(Vec2::new(x0 + i as f32 * stride, tableau_y + stack_y)),
                Pile::Stock => Some(Vec2::new(
                    if variant.is_freecell() {
                        clip.min.x + (clip.width() - card_w) * 0.5
                    } else if matches!(
                        variant,
                        Variant::Spider1 | Variant::Spider2 | Variant::Spider4
                    ) {
                        x0 + stride * 8.0
                    } else {
                        x0
                    },
                    slot_y,
                )),
                Pile::Waste => Some(Vec2::new(x0 + stride, slot_y)),
                Pile::Cell(i) => Some(Vec2::new(x0 + i as f32 * stride, slot_y)),
                Pile::Foundation(i) => Some(Vec2::new(
                    x0 + (if variant.is_freecell() { 4.0 } else { 3.0 } + i as f32) * stride,
                    slot_y,
                )),
            }
        };
        if let (Some(from), Some(to)) = (
            origin(flight.from, flight.from_stack_y),
            origin(flight.to, flight.to_stack_y),
        ) {
            let mut pos = from.lerp(to, ease);
            pos.y -= 48.0 * t * (1.0 - t);
            let mut card = flight.card;
            if flight.flip && t < 0.5 {
                card.face_up = !card.face_up;
            }
            let flip_scale = if flight.flip {
                (std::f32::consts::PI * t).cos().abs()
            } else {
                1.0
            };
            if flip_scale > 0.06 {
                let width = card_w * flip_scale;
                pos.x += (card_w - width) * 0.5;
                draw_card(
                    f,
                    clip,
                    rect(pos.x, pos.y, width, card_h),
                    card,
                    false,
                    false,
                    true,
                );
            }
        }
    }
    if let (Some(drag), Some(cursor)) = (ui.drag, s.cursor)
        && drag.active
    {
        let (pile, index) = drag.source;
        let cards = game.cards(pile);
        let ghost = cards.get(index..).unwrap_or_default();
        let origin = cursor - drag.offset;
        let mut y = origin.y;
        for card in ghost {
            draw_card(
                f,
                clip,
                rect(origin.x, y, card_w, card_h),
                *card,
                true,
                false,
                index + ghost.len() == cards.len(),
            );
            y += if card.face_up { 19.0 } else { 12.0 };
        }
    }
    super::table_fx::draw(
        f,
        clip,
        &ui.feedback,
        |anchor| {
            let crate::table_feedback::Anchor::Pile(pile) = anchor else {
                return None;
            };
            match pile {
                Pile::Tableau(i) => Some(rect(
                    x0 + i as f32 * stride,
                    tableau_y,
                    card_w,
                    game.tableau
                        .get(i)
                        .map(|cards| {
                            cards
                                .iter()
                                .take(cards.len().saturating_sub(1))
                                .map(|c| if c.face_up { 19.0 } else { 12.0 })
                                .sum::<f32>()
                        })
                        .unwrap_or(0.0)
                        + card_h,
                )),
                _ => l.hits.iter().rev().find_map(|h| match h.action {
                    Action::PatiencePick(p, _) | Action::PatienceTarget(p) if p == pile => {
                        Some(h.rect)
                    }
                    Action::PatienceDeal if pile == Pile::Stock => {
                        Some(rect(x0, slot_y, card_w, card_h))
                    }
                    _ => None,
                }),
            }
        },
        s.save.settings.reduced_motion,
    );
    // Completed Spider runs remain visible in a compact rack below the tableau.
    if !game.completed.is_empty() {
        f.text(
            Vec2::new(clip.min.x + 4.0, clip.max.y - 18.0),
            format!("{} COMPLETED", game.completed.len()),
            10.0,
            theme.success,
        );
    }
    f.text(
        Vec2::new(m, f.size.y - 49.0),
        "Drag or tap to move. Swipe empty table to pan.",
        10.0,
        theme.muted,
    );
    if game.won() {
        f.text(
            Vec2::new(clip.min.x + 6.0, clip.min.y + 6.0),
            "TABLE CLEARED",
            13.0,
            theme.success,
        );
    } else if !ui.message.is_empty() {
        f.text(
            Vec2::new(clip.min.x + 6.0, clip.max.y - 18.0),
            ui.message
                .chars()
                .take((clip.width() / 6.0) as usize)
                .collect::<String>(),
            10.0,
            theme.gold,
        );
    }
}

fn pile_slot(
    f: &mut Frame,
    l: &mut ViewLayout,
    s: &Arcade,
    clip: Rect,
    full: Rect,
    pile: Pile,
    stock: bool,
) {
    let r = intersect(full, clip);
    if r.width() <= 0.0 || r.height() <= 0.0 {
        return;
    }
    let theme = *f.theme();
    let card_pal = super::game_palette::card(f.palette);
    f.rect(r.min, r.size(), card_pal.face_bg);
    f.outline(
        r.min + Vec2::splat(1.0),
        r.size() - Vec2::splat(2.0),
        1.0,
        if s.patience_ui
            .drag
            .filter(|d| d.active)
            .map(|d| d.source)
            .or(s.patience_ui.selected)
            .is_some_and(|(p, i)| s.patience_game().legal_move(p, i, pile))
        {
            card_pal.border_legal
        } else if s.patience_ui.selected.is_some_and(|(p, _)| p == pile) {
            card_pal.border_selected
        } else {
            card_pal.border_normal
        },
    );
    if stock {
        for inset in [4.0, 7.0] {
            f.outline(
                r.min + Vec2::splat(inset),
                r.size() - Vec2::splat(inset * 2.0),
                0.7,
                theme.edge,
            );
        }
    }
    let label = match pile {
        Pile::Stock => "DEAL",
        Pile::Waste => "WASTE",
        Pile::Foundation(_) => "ACE",
        Pile::Cell(_) => "FREE",
        Pile::Tableau(_) => "EMPTY",
    };
    if r == full {
        f.text(
            r.min
                + Vec2::new(
                    (r.width() - label.len() as f32 * 6.0) * 0.5,
                    r.height() * 0.43,
                ),
            label,
            10.0,
            theme.muted,
        );
    }
    l.hits.push(Hit {
        rect: r,
        action: if stock {
            Action::PatienceDeal
        } else {
            Action::PatienceTarget(pile)
        },
    });
}

fn pile_slot_visual(f: &mut Frame, clip: Rect, full: Rect) {
    let r = intersect(full, clip);
    if r.width() <= 0.0 || r.height() <= 0.0 {
        return;
    }
    let theme = *f.theme();
    let card_pal = super::game_palette::card(f.palette);
    f.rect(r.min, r.size(), card_pal.face_bg);
    f.outline(
        r.min + Vec2::splat(1.0),
        r.size() - Vec2::splat(2.0),
        1.0,
        theme.edge,
    );
}

fn add_target_hit(l: &mut ViewLayout, clip: Rect, r: Rect, pile: Pile) {
    let clipped = intersect(r, clip);
    if clipped.width() > 0.0 && clipped.height() > 0.0 {
        l.hits.push(Hit {
            rect: clipped,
            action: Action::PatienceTarget(pile),
        });
    }
}
fn add_card_hit(l: &mut ViewLayout, clip: Rect, r: Rect, pile: Pile, index: usize) {
    l.patience_origins.push((pile, index, r.min));
    let clipped = intersect(r, clip);
    if clipped.width() > 0.0 && clipped.height() > 0.0 {
        l.hits.push(Hit {
            rect: clipped,
            action: Action::PatiencePick(pile, index),
        });
    }
}
fn intersect(a: Rect, b: Rect) -> Rect {
    Rect {
        min: a.min.max(b.min),
        max: a.max.min(b.max),
    }
}

fn draw_card(
    f: &mut Frame,
    clip: Rect,
    full: Rect,
    card: Card,
    selected: bool,
    legal: bool,
    show_center: bool,
) {
    let r = intersect(full, clip);
    if r.width() <= 0.0 || r.height() <= 0.0 {
        return;
    }
    let theme = *f.theme();
    let card_pal = super::game_palette::card(f.palette);
    let border = if selected {
        card_pal.border_selected
    } else if legal {
        card_pal.border_legal
    } else {
        card_pal.border_normal
    };
    f.rect(
        r.min + Vec2::new(2.0, 3.0),
        r.size() - Vec2::new(2.0, 3.0),
        card_pal.shadow,
    );
    f.rect(
        r.min,
        r.size(),
        if card.face_up {
            card_pal.face_bg
        } else {
            card_pal.back_bg
        },
    );
    let stroke = if selected { 2.5 } else { 1.2 };
    f.outline(
        r.min + Vec2::splat(stroke),
        r.size() - Vec2::splat(stroke * 2.0),
        stroke,
        border,
    );
    f.outline(
        r.min + Vec2::splat(4.0),
        r.size() - Vec2::splat(8.0),
        0.7,
        theme.edge,
    );
    if !card.face_up {
        let inset = 7.0;
        let center = r.center();
        let radius = ((r.width().min(r.height()) - inset * 2.0) * 0.28).max(0.0);
        if radius >= 3.0 {
            let top = center + Vec2::new(0.0, -radius);
            let right = center + Vec2::new(radius, 0.0);
            let bottom = center + Vec2::new(0.0, radius);
            let left = center + Vec2::new(-radius, 0.0);
            for (a, b) in [(top, right), (right, bottom), (bottom, left), (left, top)] {
                f.line(a, b, 0.8, card_pal.border_normal);
            }
        }
    }
    if card.face_up {
        let color = card_pal.suit_color(card.suit);
        let rank_pos = full.min + Vec2::new(5.0, 2.0);
        let rank_size = (full.width() * 0.27).clamp(9.0, 14.0);
        let rank_width = card.rank_label().len() as f32 * rank_size * 0.65;
        if rank_pos.x >= clip.min.x
            && rank_pos.y >= clip.min.y
            && rank_pos.x + rank_width <= clip.max.x
            && rank_pos.y + rank_size <= clip.max.y
        {
            let corner = full.min + Vec2::new(29.0, 8.0);
            if clip.contains(corner - Vec2::splat(5.0)) && clip.contains(corner + Vec2::splat(5.0))
            {
                super::screens::suit(f, corner, 4.0, card.suit, color);
            }
            f.text(rank_pos, card.rank_label(), rank_size, color);
        }
        if show_center && r == full && r.width() >= 30.0 && r.height() >= 45.0 {
            super::screens::suit(
                f,
                r.center(),
                (r.width() * 0.2).clamp(6.0, 18.0),
                card.suit,
                color,
            );
        }
    }
}
