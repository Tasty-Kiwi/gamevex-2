use super::blackjack_motion::visible_dealer_total;
use super::casino_fx::{self, ResultKind};
use super::screens::{button, card_face};
use super::*;
use gamevex_core::blackjack::{Command, HandOutcome, MAX_BET, Phase, hand_value};

pub(super) fn blackjack(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let reduced = s.save.settings.reduced_motion;
    let view = s.blackjack_motion.sample(&s.save.blackjack, reduced);
    let game = &view;
    let small = w < 620.0;
    let short = f.size.y < 640.0;
    let compact_view = small || short;
    let finished = matches!(game.phase, Phase::Settled | Phase::GameOver);
    f.text(
        Vec2::new(m, if short { 82.0 } else { 96.0 }),
        "BLACKJACK",
        if short {
            if small { 27.0 } else { 34.0 }
        } else if small {
            31.0
        } else {
            42.0
        },
        INK,
    );
    let info_y = if short {
        if small { 119.0 } else { 130.0 }
    } else if small {
        136.0
    } else {
        151.0
    };
    f.text(
        Vec2::new(m, info_y),
        format!("CREDITS  {}", game.balance),
        15.0,
        GOLD,
    );
    f.text(
        Vec2::new(m + w * 0.47, info_y),
        format!("BET  {} / HAND", game.bet),
        14.0,
        CYAN,
    );
    let wager_hands = if game.balance == 0 {
        game.hand_count.clamp(1, 5)
    } else {
        game.hand_count
            .clamp(1, 5)
            .min(game.balance as usize)
            .max(1)
    };
    let wager_each = if game.balance == 0 {
        0
    } else {
        game.bet.min(game.balance / wager_hands as u64)
    };
    let active_stake = game.hands.iter().fold(game.insurance_bet, |total, hand| {
        total.saturating_add(hand.bet)
    });
    let round_total = if game.phase == Phase::Betting {
        wager_each.saturating_mul(wager_hands as u64)
    } else {
        active_stake
    };
    f.text(
        Vec2::new(m, info_y + 28.0),
        format!("HANDS  {}", game.hand_count),
        12.0,
        MUTED,
    );
    f.text(
        Vec2::new(m + w * 0.47, info_y + 28.0),
        format!(
            "{}  {}",
            if game.phase == Phase::Betting {
                "ROUND TOTAL"
            } else {
                "ROUND STAKE"
            },
            round_total
        ),
        12.0,
        MUTED,
    );

    let control_y = if short {
        164.0
    } else if small {
        183.0
    } else {
        198.0
    };
    control(
        f,
        s,
        l,
        rect(m, control_y, 34.0, 30.0),
        "-",
        Command::SetBet(game.bet.saturating_sub(10).max(1)),
    );
    control(
        f,
        s,
        l,
        rect(m + 40.0, control_y, 34.0, 30.0),
        "+",
        Command::SetBet(game.bet.saturating_add(10).min(MAX_BET)),
    );
    control(
        f,
        s,
        l,
        rect(m + 88.0, control_y, 34.0, 30.0),
        "-",
        Command::SetHandCount(game.hand_count.saturating_sub(1).max(1)),
    );
    control(
        f,
        s,
        l,
        rect(m + 128.0, control_y, 34.0, 30.0),
        "+",
        Command::SetHandCount((game.hand_count + 1).min(5)),
    );
    if !short {
        f.text(Vec2::new(m, control_y + 34.0), "BET / HAND", 10.0, MUTED);
        f.text(
            Vec2::new(m + 88.0, control_y + 34.0),
            "NUMBER OF HANDS",
            10.0,
            MUTED,
        );
    }

    let buttons_y = if short {
        f.size.y - 88.0
    } else {
        f.size.y - 116.0
    };
    let table_top = if short {
        218.0
    } else if small {
        254.0
    } else {
        270.0
    };
    let table_bottom = (buttons_y
        - if finished && small {
            78.0
        } else if short {
            20.0
        } else {
            24.0
        })
    .max(table_top + 110.0);
    let table_rect = rect(m, table_top, w, table_bottom - table_top);
    f.rect(
        table_rect.min,
        table_rect.size(),
        Color::srgb(0.008, 0.045, 0.041),
    );
    f.outline(table_rect.min, table_rect.size(), 1.0, EDGE);
    f.line(
        table_rect.min + Vec2::new(8.0, 6.0),
        Vec2::new(table_rect.max.x - 8.0, table_rect.min.y + 6.0),
        1.5,
        CYAN,
    );
    f.line(
        Vec2::new(table_rect.min.x + 8.0, table_rect.max.y - 6.0),
        table_rect.max - Vec2::new(8.0, 6.0),
        1.5,
        GOLD,
    );
    let hidden = s.blackjack_motion.dealer_hidden(&s.save.blackjack, reduced);
    if platform::diagnostics_enabled() {
        platform::diagnostics_attribute(
            "data-blackjack-view",
            &serde_json::json!({
                "dealerTotal": visible_dealer_total(game, hidden),
                "dealerHidden": hidden,
                "animating": s.blackjack_motion.busy() && !reduced,
                "dealerCards": game.dealer.len(),
                "playerCards": game.hands.iter().map(|h| h.cards.len()).collect::<Vec<_>>(),
            })
            .to_string(),
        );
    }
    let base_card_w = if short {
        38.0
    } else if small {
        50.0
    } else {
        90.0_f32
            .min((w / 6.0).max(66.0))
            .min(((table_bottom - table_top - 146.0) / 2.8).max(34.0))
    };
    let card_w = base_card_w;
    let card_h = card_w * 1.4;
    let dealer_label_y = table_top + if short { 16.0 } else { 24.0 };
    f.text(
        Vec2::new(m + 14.0, dealer_label_y),
        format!(
            "DEALER  {}{}",
            visible_dealer_total(game, hidden),
            if hidden { " + ?" } else { "" }
        ),
        12.0,
        GOLD,
    );
    let dealer_step = (card_w * 0.64)
        .min((w - card_w - 46.0).max(0.0) / game.dealer.len().saturating_sub(1).max(1) as f32);
    let dealer_span = if game.dealer.is_empty() {
        0.0
    } else {
        card_w + dealer_step * game.dealer.len().saturating_sub(1) as f32
    };
    let dealer_x = m + (w - dealer_span).max(0.0) * 0.5;
    let dealer_y = table_top + if short { 24.0 } else { 36.0 };
    for (i, c) in game.dealer.iter().enumerate() {
        let mut shown = *c;
        if i == 0 && hidden {
            shown.face_up = false;
        }
        let r = rect(dealer_x + i as f32 * dealer_step, dealer_y, card_w, card_h);
        card_face(f, animated_rect(s, r, true, 0, i), shown, false, true);
    }

    let hands_y = table_top
        + if short {
            if finished && small { 65.0 } else { 100.0 }
        } else {
            card_h + 54.0
        };
    f.text(Vec2::new(m + 14.0, hands_y), "PLAYER", 12.0, MUTED);
    let narrow_live = compact_view && game.phase == Phase::PlayerTurn;
    if narrow_live {
        if short {
            let summary = game
                .hands
                .iter()
                .enumerate()
                .map(|(i, hand)| {
                    let outcome = hand_state(hand, i == game.active_hand);
                    format!(
                        "H{} {} / {} {}",
                        i + 1,
                        hand_value(&hand.cards),
                        hand.bet,
                        outcome
                    )
                })
                .collect::<Vec<_>>()
                .join("   ");
            f.text(
                Vec2::new(m + 78.0, hands_y),
                compact(&summary, (w / 5.0) as usize),
                11.0,
                MUTED,
            );
        }
        for (i, hand) in game.hands.iter().enumerate() {
            let outcome = hand_state(hand, i == game.active_hand);
            let label = format!(
                "H{}  {}  BET {}  {}",
                i + 1,
                hand_value(&hand.cards),
                hand.bet,
                outcome
            );
            if !short {
                f.text(
                    Vec2::new(m + 14.0, hands_y + 20.0 + i as f32 * 18.0),
                    compact(&label, (w / 6.0) as usize),
                    11.0,
                    if i == game.active_hand { GOLD } else { MUTED },
                );
            }
        }

        if let Some(hand) = game.hands.get(game.active_hand) {
            let cards_y = if short {
                hands_y + 36.0
            } else {
                hands_y + 30.0 + game.hands.len() as f32 * 18.0
            };
            let gaps = 6.0 * hand.cards.len().saturating_sub(1) as f32;
            let active_card_w = (if short { 38.0_f32 } else { 90.0_f32 })
                .min(((w - gaps) / hand.cards.len().max(1) as f32).max(1.0))
                .min(
                    ((buttons_y - if w < 700.0 { 42.0 } else { 0.0 } - cards_y - 26.0) / 1.4)
                        .max(1.0),
                );
            let active_card_h = active_card_w * 1.4;
            let span = active_card_w * hand.cards.len() as f32 + gaps;
            let start_x = m + (w - span).max(0.0) * 0.5;
            f.text(
                Vec2::new(m + 14.0, cards_y - 16.0),
                "ACTIVE HAND",
                12.0,
                GOLD,
            );
            for (i, card) in hand.cards.iter().enumerate() {
                let card_rect = rect(
                    start_x + i as f32 * (active_card_w + 6.0),
                    cards_y,
                    active_card_w,
                    active_card_h,
                );
                turn_outline(f, s, card_rect);
                card_face(
                    f,
                    animated_rect(s, card_rect, false, game.active_hand, i),
                    *card,
                    true,
                    true,
                );
            }
            f.text(
                Vec2::new(m, cards_y + active_card_h + 5.0),
                format!("VALUE  {}   BET  {}", hand_value(&hand.cards), hand.bet),
                14.0,
                GOLD,
            );
        }
    } else if compact_view {
        // A compact result ledger keeps all five outcomes visible on phones.
        for (i, hand) in game.hands.iter().enumerate() {
            if let Some(kind) = hand.outcome.map(hand_result_kind) {
                if short {
                    // A side marker leaves enough room for the compact text line.
                    f.rect(
                        Vec2::new(m + 8.0, hands_y + 20.0 + i as f32 * 18.0),
                        Vec2::new(2.0, 15.0),
                        casino_fx::kind_color(kind),
                    );
                } else {
                    casino_fx::highlight(
                        f,
                        rect(m + 8.0, hands_y + 15.0 + i as f32 * 25.0, w - 16.0, 24.0),
                        kind,
                        s.casino_feedback
                            .result
                            .as_ref()
                            .map(|_| s.casino_feedback.result_age),
                        reduced,
                    );
                }
            }
            let result = hand.outcome.map(outcome_label).unwrap_or("READY");
            let label = format!(
                "HAND {}  {}  BET {}  {}",
                i + 1,
                hand_value(&hand.cards),
                hand.bet,
                result
            );
            f.text(
                Vec2::new(
                    m + 14.0,
                    hands_y + 20.0 + i as f32 * if short { 18.0 } else { 25.0 },
                ),
                compact(&label, (w / 6.0) as usize),
                if short { 13.0 } else { 14.0 },
                outcome_color(hand.outcome),
            );
        }
        if game.hands.is_empty() {
            f.text(
                Vec2::new(m, hands_y + 26.0),
                "Choose a wager, then deal.",
                12.0,
                MUTED,
            );
        }
    } else {
        let count = game.hands.len().max(game.hand_count).max(1);
        let col_w = w / count as f32;
        for i in 0..game.hands.len() {
            let hand = &game.hands[i];
            let x = m + i as f32 * col_w;
            let active = game.phase == Phase::PlayerTurn && game.active_hand == i;
            let hand_card_w =
                card_w.min(col_w / (1.0 + 0.45 * hand.cards.len().saturating_sub(1) as f32));
            let hand_card_h = hand_card_w * 1.4;
            let y = hands_y + 38.0;
            if let Some(kind) = hand
                .outcome
                .map(hand_result_kind)
                .or_else(|| (hand_value(&hand.cards) > 21).then_some(ResultKind::Loss))
            {
                casino_fx::highlight(
                    f,
                    rect(x + 4.0, hands_y + 16.0, col_w - 8.0, hand_card_h + 58.0),
                    kind,
                    s.casino_feedback
                        .result
                        .as_ref()
                        .map(|_| s.casino_feedback.result_age),
                    reduced,
                );
            }
            let card_gap = if hand.cards.len() <= 1 {
                0.0
            } else {
                (hand_card_w * 0.45)
                    .min((col_w - hand_card_w).max(0.0) / (hand.cards.len() - 1) as f32)
            };
            let hand_span = hand_card_w + card_gap * hand.cards.len().saturating_sub(1) as f32;
            let hand_x = x + (col_w - hand_span).max(0.0) * 0.5;
            f.text(
                Vec2::new(hand_x, hands_y + 18.0),
                format!("HAND {} / BET {}", i + 1, hand.bet),
                11.0,
                if active { GOLD } else { MUTED },
            );
            let card_rects = hand
                .cards
                .iter()
                .enumerate()
                .map(|(j, _)| rect(hand_x + j as f32 * card_gap, y, hand_card_w, hand_card_h))
                .collect::<Vec<_>>();
            for (j, card) in hand.cards.iter().enumerate() {
                let rank = rect(card_rects[j].min.x + 4.0, y + 4.0, hand_card_w * 0.34, 26.0);
                let rank_visible = !card_rects[j + 1..]
                    .iter()
                    .any(|later| rects_overlap(rank, *later));
                if active {
                    turn_outline(f, s, card_rects[j]);
                }
                card_face(
                    f,
                    animated_rect(s, card_rects[j], false, i, j),
                    *card,
                    active,
                    rank_visible,
                );
            }
            let result = hand.outcome.map(outcome_label).unwrap_or("");
            f.text(
                Vec2::new(hand_x, y + hand_card_h + 5.0),
                format!("{}  {}", hand_value(&hand.cards), result),
                14.0,
                outcome_color(hand.outcome),
            );
        }
    }
    if let Some(message) = &game.message
        && !short
        && !finished
        && !(compact_view
            && matches!(
                game.phase,
                Phase::PlayerTurn | Phase::Settled | Phase::GameOver
            ))
    {
        let message_y = table_bottom - 24.0;
        f.text(
            Vec2::new(m + 14.0, message_y),
            compact(message, ((w - 28.0) / 8.1) as usize),
            13.0,
            CYAN,
        );
    }

    if finished {
        let result = s
            .casino_feedback
            .result
            .clone()
            .unwrap_or_else(|| crate::casino_feedback::blackjack_result(game));
        let bounds = if small {
            rect(m, buttons_y - 66.0, w, 54.0)
        } else {
            rect(
                m + w * 0.43,
                if short { 174.0 } else { 198.0 },
                w * 0.57,
                if short { 40.0 } else { 54.0 },
            )
        };
        casino_fx::banner(
            f,
            bounds,
            result.kind.into(),
            &result.title,
            &format!("{:+} CREDITS / BANK {}", result.net, game.balance),
            s.casino_feedback
                .result
                .as_ref()
                .map(|_| s.casino_feedback.result_age),
            reduced,
        );
    } else if let Some(label) = s.casino_feedback.action {
        casino_fx::action(
            f,
            Vec2::new(m + w * 0.76, if short { 191.0 } else { 230.0 }),
            label,
            s.casino_feedback.action_age,
            reduced,
            if label == "BUST" { RED } else { GOLD },
        );
    }

    if game.hands.is_empty() && game.dealer.is_empty() {
        f.text(
            Vec2::new(table_rect.center().x - 57.0, table_rect.center().y + 4.0),
            "PLACE YOUR BET",
            if compact_view { 13.0 } else { 17.0 },
            CYAN,
        );
    }

    match game.phase {
        Phase::Betting => control(
            f,
            s,
            l,
            rect(m, buttons_y, 100.0, 36.0),
            "DEAL / ENTER",
            Command::Deal,
        ),
        Phase::Insurance => {
            control(
                f,
                s,
                l,
                rect(m, buttons_y, 100.0, 36.0),
                "YES / Y",
                Command::Insure,
            );
            control(
                f,
                s,
                l,
                rect(m + 108.0, buttons_y, 100.0, 36.0),
                "NO / N",
                Command::DeclineInsurance,
            );
        }
        Phase::PlayerTurn => {
            let commands = [
                ("HIT / H", Command::Hit),
                ("STAND / S", Command::Stand),
                ("DOUBLE / D", Command::Double),
                ("SPLIT / P", Command::Split),
                ("SURRENDER / R", Command::Surrender),
            ];
            let gap = 6.0;
            if w < 700.0 {
                let bw = (w - gap * 2.0) / 3.0;
                for (i, (label, cmd)) in commands.iter().copied().take(3).enumerate() {
                    control(
                        f,
                        s,
                        l,
                        rect(m + i as f32 * (bw + gap), buttons_y - 42.0, bw, 36.0),
                        label,
                        cmd,
                    );
                }
                for (i, (label, cmd)) in commands.iter().copied().skip(3).enumerate() {
                    control(
                        f,
                        s,
                        l,
                        rect(m + i as f32 * (bw + gap), buttons_y, bw, 36.0),
                        label,
                        cmd,
                    );
                }
            } else {
                let bw = (w - 126.0 - gap * 5.0) / 5.0;
                for (i, (label, cmd)) in commands.into_iter().enumerate() {
                    control(
                        f,
                        s,
                        l,
                        rect(m + i as f32 * (bw + gap), buttons_y, bw, 36.0),
                        label,
                        cmd,
                    );
                }
            }
        }
        Phase::Settled => control(
            f,
            s,
            l,
            rect(m, buttons_y, 120.0, 36.0),
            "NEXT / ENTER",
            Command::NextRound,
        ),
        Phase::GameOver => control(
            f,
            s,
            l,
            rect(m, buttons_y, 120.0, 36.0),
            "RESTART",
            Command::Restart,
        ),
    }
}

fn outcome_label(outcome: HandOutcome) -> &'static str {
    match outcome {
        HandOutcome::Surrendered => "SURRENDERED",
        HandOutcome::Bust => "BUST",
        HandOutcome::Blackjack => "BLACKJACK",
        HandOutcome::Win => "WON",
        HandOutcome::Push => "PUSH",
        HandOutcome::Loss => "LOST",
    }
}

fn outcome_color(outcome: Option<HandOutcome>) -> Color {
    match outcome {
        Some(HandOutcome::Blackjack | HandOutcome::Win) => GREEN,
        Some(HandOutcome::Bust | HandOutcome::Loss | HandOutcome::Surrendered) => RED,
        Some(HandOutcome::Push) => GOLD,
        None => INK,
    }
}

fn control(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, r: Rect, label: &str, cmd: Command) {
    if s.save.blackjack.legal(cmd) && (!s.blackjack_motion.busy() || s.save.settings.reduced_motion)
    {
        button(f, s, l, r, label, Action::Blackjack(cmd), false);
    } else {
        f.rect(r.min, r.size(), PANEL);
        f.outline(r.min, r.size(), 1.0, EDGE);
        let tx = r.min.x + (r.width() - label.chars().count() as f32 * 7.0) * 0.5;
        f.text(
            Vec2::new(tx.max(r.min.x + 4.0), r.min.y + 10.0),
            label,
            11.0,
            MUTED,
        );
    }
}

fn animated_rect(s: &Arcade, target: Rect, dealer: bool, hand: usize, index: usize) -> Rect {
    let (arrival, flip) =
        s.blackjack_motion
            .card_pose(dealer, hand, index, s.save.settings.reduced_motion);
    let eased = 1.0 - (1.0 - arrival).powi(3);
    let offset = Vec2::new(34.0, -28.0) * (1.0 - eased);
    let scale = 0.78 + 0.22 * eased;
    let size = target.size() * Vec2::new(scale * flip.max(0.025), scale);
    Rect::from_center_size(target.center() + offset, size)
}

fn turn_outline(f: &mut Frame, s: &Arcade, target: Rect) {
    let pulse = 1.0
        - s.blackjack_motion
            .turn_progress(s.save.settings.reduced_motion);
    if pulse > 0.0 {
        let padding = Vec2::splat(3.0 + pulse * 4.0);
        f.outline(
            target.min - padding,
            target.size() + padding * 2.0,
            1.0 + pulse * 2.0,
            GOLD.with_alpha(pulse),
        );
    }
}

fn hand_result_kind(outcome: HandOutcome) -> ResultKind {
    match outcome {
        HandOutcome::Blackjack => ResultKind::Natural,
        HandOutcome::Win => ResultKind::Win,
        HandOutcome::Push => ResultKind::Push,
        _ => ResultKind::Loss,
    }
}

fn hand_state(hand: &gamevex_core::blackjack::PlayerHand, active: bool) -> &'static str {
    hand.outcome.map(outcome_label).unwrap_or_else(|| {
        if hand_value(&hand.cards) > 21 {
            "BUST"
        } else if hand_value(&hand.cards) == 21 {
            "21"
        } else if active {
            "PLAYING"
        } else if hand.stood {
            "STOOD"
        } else {
            "WAITING"
        }
    })
}
