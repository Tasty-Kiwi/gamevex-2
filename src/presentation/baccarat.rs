use super::casino_fx::{self, ResultKind};
use super::screens::{button, card_face};
use super::*;
use gamevex_core::baccarat::{BetSide, Command, MAX_BET, Outcome, Phase, hand_value};

pub(super) fn baccarat(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let reduced = s.save.settings.reduced_motion;
    let busy = s.baccarat_motion.busy() && !reduced;
    let game = s.baccarat_motion.sample(&s.save.baccarat, reduced);
    let small = w < 620.0;
    let short = f.size.y < 640.0;
    let title_y = if short { 82.0 } else { 92.0 };
    let title_size = if small {
        30.0
    } else if short {
        34.0
    } else {
        42.0
    };
    f.text(Vec2::new(m, title_y), "BACCARAT", title_size, INK);

    let info_y = if short {
        122.0
    } else if small {
        130.0
    } else {
        145.0
    };
    f.text(
        Vec2::new(m, info_y),
        format!("CREDITS  {}", game.balance),
        if small { 13.0 } else { 15.0 },
        GOLD,
    );
    f.text(
        Vec2::new(m + w * 0.46, info_y),
        format!(
            "{}  {}",
            match game.selected {
                Some(BetSide::Player) => "PLAYER BET",
                Some(BetSide::Banker) => "BANKER BET",
                Some(BetSide::Tie) => "TIE BET",
                Some(BetSide::PlayerPair) => "PLAYER PAIR",
                Some(BetSide::BankerPair) => "BANKER PAIR",
                None => "BET",
            },
            game.bet
        ),
        if small { 13.0 } else { 15.0 },
        CYAN,
    );
    if !short && game.phase != Phase::Betting {
        f.text(
            Vec2::new(m, info_y + 18.0),
            match game.selected {
                Some(BetSide::Player) => "BET ON PLAYER",
                Some(BetSide::Banker) => "BET ON BANKER",
                Some(BetSide::Tie) => "BET ON TIE",
                Some(BetSide::PlayerPair) => "BET ON PLAYER PAIR",
                Some(BetSide::BankerPair) => "BET ON BANKER PAIR",
                None => "PLAYER / BANKER",
            },
            11.0,
            MUTED,
        );
    }

    let betting = game.phase == Phase::Betting;
    let select_y = if short {
        152.0
    } else if small {
        162.0
    } else {
        178.0
    };
    if betting {
        let gap = 7.0;
        let button_w = (w - gap * 2.0) / 3.0;
        for (i, (side, label)) in [
            (BetSide::Player, "PLAYER  1:1"),
            (
                BetSide::Banker,
                if small {
                    "BANKER .95:1"
                } else {
                    "BANKER  0.95:1"
                },
            ),
            (BetSide::Tie, "TIE  8:1"),
            (BetSide::PlayerPair, "PLAYER PAIR 11:1"),
            (BetSide::BankerPair, "BANKER PAIR 11:1"),
        ]
        .into_iter()
        .enumerate()
        {
            let h = if short { 32.0 } else { 36.0 };
            let (x, y, width) = if i < 3 {
                (m + i as f32 * (button_w + gap), select_y, button_w)
            } else {
                let width = (w - gap) / 2.0;
                (
                    m + (i - 3) as f32 * (width + gap),
                    select_y + h + gap,
                    width,
                )
            };
            let r = rect(x, y, width, h);
            let cmd = Command::Select(side);
            if !busy && game.legal(cmd) {
                button(
                    f,
                    s,
                    l,
                    r,
                    label,
                    Action::Baccarat(cmd),
                    game.selected == Some(side),
                );
            } else {
                disabled_button(f, r, label, game.selected == Some(side));
            }
        }
    }

    let bet_y = select_y + if short { 78.0 } else { 87.0 };
    if betting {
        let h = if short { 28.0 } else { 32.0 };
        let widths = [58.0, 58.0, 60.0];
        let labels = ["-10", "+10", "MAX"];
        let commands = [
            Command::SetBet(game.bet.saturating_sub(10).max(10)),
            Command::SetBet(game.bet.saturating_add(10).min(MAX_BET)),
            Command::MaxBet,
        ];
        let gap = 6.0;
        for i in 0..3 {
            let x = m + i as f32 * (widths[i] + gap);
            let r = rect(x, bet_y, widths[i], h);
            if !busy && game.legal(commands[i]) {
                button(f, s, l, r, labels[i], Action::Baccarat(commands[i]), false);
            } else {
                disabled_button(f, r, labels[i], false);
            }
        }
        f.text(
            Vec2::new(m + 208.0, bet_y + if short { 8.0 } else { 10.0 }),
            "BET",
            10.0,
            MUTED,
        );
    }

    let action_y = f.size.y - if short { 84.0 } else { 104.0 };
    let table_top = bet_y + if short { 37.0 } else { 47.0 };
    let has_result = matches!(game.phase, Phase::Settled | Phase::GameOver);
    let result_h = if has_result { 28.0 } else { 0.0 };
    let action_clearance = if small { 30.0 } else { 16.0 };
    let table_bottom = (action_y - action_clearance - result_h).max(table_top + 82.0);
    let panel_gap = if small { 12.0 } else { 18.0 };
    let panel_h = if small {
        (table_bottom - table_top - panel_gap) / 2.0
    } else {
        table_bottom - table_top
    };
    let panels = if small {
        [
            rect(m, table_top, w, panel_h),
            rect(m, table_top + panel_h + panel_gap, w, panel_h),
        ]
    } else {
        let panel_w = (w - panel_gap) / 2.0;
        [
            rect(m, table_top, panel_w, panel_h),
            rect(m + panel_w + panel_gap, table_top, panel_w, panel_h),
        ]
    };

    let player_total = hand_value(&game.player);
    let banker_total = hand_value(&game.banker);
    if platform::diagnostics_enabled() {
        platform::diagnostics_attribute(
            "data-baccarat-view",
            &serde_json::json!({
                "animating": busy,
                "playerCards": game.player.len(),
                "bankerCards": game.banker.len(),
                "playerTotal": player_total,
                "bankerTotal": banker_total,
            })
            .to_string(),
        );
    }

    draw_lane(f, panels[0], player_total, &game.player, true, small, s);
    draw_lane(f, panels[1], banker_total, &game.banker, false, small, s);

    if has_result {
        let result = s
            .casino_feedback
            .result
            .clone()
            .unwrap_or_else(|| crate::casino_feedback::baccarat_result(&game));
        let age = s
            .casino_feedback
            .result
            .as_ref()
            .map(|_| s.casino_feedback.result_age);
        for (i, side) in [BetSide::Player, BetSide::Banker].into_iter().enumerate() {
            let kind = if game.result == Some(BetSide::Tie) {
                ResultKind::Push
            } else if game.result == Some(side) {
                ResultKind::Win
            } else {
                ResultKind::Loss
            };
            casino_fx::highlight(f, panels[i], kind, age, reduced);
        }
        casino_fx::banner(
            f,
            rect(
                m,
                if short {
                    151.0
                } else if small {
                    183.0
                } else {
                    187.0
                },
                w,
                if small || short { 56.0 } else { 66.0 },
            ),
            result.kind.into(),
            &result.title,
            &format!("{:+} CREDITS / RETURN {}", result.net, game.payout),
            age,
            reduced,
        );
        let line = result_line(&game);
        f.text(
            Vec2::new(m, table_bottom + 7.0),
            fit_text(&line, (w / 6.8) as usize),
            if small { 10.0 } else { 12.0 },
            match game.outcome {
                Some(Outcome::Win) => GREEN,
                Some(Outcome::Push) => GOLD,
                Some(Outcome::Loss) => RED,
                None => MUTED,
            },
        );
    } else if let Some(label) = s.casino_feedback.action {
        casino_fx::action(
            f,
            Vec2::new(m + w * 0.78, bet_y + 16.0),
            label,
            s.casino_feedback.action_age,
            reduced,
            GOLD,
        );
    }

    match game.phase {
        Phase::Betting => {
            let can_deal = game.selected.is_some()
                && game.balance >= game.bet
                && !busy
                && game.legal(Command::Deal);
            let r = rect(m, action_y, if small { w } else { 156.0 }, 36.0);
            if can_deal {
                button(f, s, l, r, "DEAL  D", Action::Baccarat(Command::Deal), true);
            } else {
                disabled_button(f, r, "DEAL  D", false);
            }
            if game.balance < game.bet {
                deal_help(f, m, w, small, action_y, "BET EXCEEDS CREDITS");
            } else if game.selected.is_none() {
                deal_help(f, m, w, small, action_y, "SELECT A BET");
            } else if busy {
                deal_help(f, m, w, small, action_y, "DEALING");
            }
        }
        Phase::Settled => {
            let r = rect(m, action_y, 154.0, 36.0);
            if !busy && game.legal(Command::NextRound) {
                button(
                    f,
                    s,
                    l,
                    r,
                    "NEXT  N",
                    Action::Baccarat(Command::NextRound),
                    false,
                );
            } else {
                disabled_button(f, r, "NEXT  N", false);
            }
        }
        Phase::GameOver => {
            let r = rect(m, action_y, 130.0, 36.0);
            if !busy && game.legal(Command::Restart) {
                button(
                    f,
                    s,
                    l,
                    r,
                    "RESTART  R",
                    Action::Baccarat(Command::Restart),
                    false,
                );
            } else {
                disabled_button(f, r, "RESTART  R", false);
            }
        }
    }
}

fn draw_lane(
    f: &mut Frame,
    r: Rect,
    total: u8,
    cards: &[gamevex_core::cards::Card],
    player: bool,
    stacked: bool,
    state: &Arcade,
) {
    let side_color = if player { CYAN } else { GOLD };
    let label = if player { "PLAYER" } else { "BANKER" };
    f.rect(r.min, r.size(), Color::srgb(0.008, 0.045, 0.041));
    f.outline(r.min, r.size(), 1.0, EDGE);
    f.line(
        r.min + Vec2::new(8.0, 5.0),
        Vec2::new(r.max.x - 8.0, r.min.y + 5.0),
        1.5,
        side_color,
    );
    f.text(
        r.min + Vec2::new(12.0, if stacked { 12.0 } else { 18.0 }),
        label,
        if stacked { 11.0 } else { 13.0 },
        side_color,
    );
    let total_label = format!("TOTAL  {total}");
    let size = if stacked { 12.0 } else { 14.0 };
    let label_width = total_label.chars().count() as f32 * size * 0.6;
    f.text(
        Vec2::new(
            r.max.x - label_width - 12.0,
            r.min.y + if stacked { 11.0 } else { 17.0 },
        ),
        total_label,
        size,
        INK,
    );

    if cards.is_empty() {
        f.text(
            Vec2::new(r.min.x + 12.0, r.min.y + r.height() * 0.58),
            "AWAITING DEAL",
            10.0,
            MUTED,
        );
        return;
    }
    let card_w = if stacked {
        (r.width() / 6.2).clamp(36.0, 58.0)
    } else {
        (r.width() / 5.5).clamp(48.0, 88.0)
    }
    .min((r.height() - 50.0).max(28.0) / 1.4);
    let card_h = card_w * 1.4;
    let available = (r.width() - card_w - 28.0).max(0.0);
    let step = if cards.len() > 1 {
        (card_w * 0.66).min(available / (cards.len() - 1) as f32)
    } else {
        0.0
    };
    let span = card_w + step * cards.len().saturating_sub(1) as f32;
    let x = r.min.x + (r.width() - span).max(0.0) * 0.5;
    let y = r.min.y + if stacked { 39.0 } else { 48.0 };
    for (index, card) in cards.iter().copied().enumerate() {
        let target = rect(x + index as f32 * step, y, card_w, card_h);
        card_face(
            f,
            animated_rect(state, target, player, index),
            card,
            false,
            true,
        );
    }
}

fn deal_help(f: &mut Frame, m: f32, w: f32, small: bool, action_y: f32, label: &str) {
    if small {
        let width = label.chars().count() as f32 * 6.0;
        f.text(
            Vec2::new(m + (w - width).max(0.0) * 0.5, action_y - 25.0),
            label,
            10.0,
            MUTED,
        );
    } else {
        f.text(Vec2::new(m + 168.0, action_y + 11.0), label, 10.0, MUTED);
    }
}

fn animated_rect(state: &Arcade, target: Rect, player: bool, index: usize) -> Rect {
    let arrival =
        state
            .baccarat_motion
            .card_progress(player, index, state.save.settings.reduced_motion);
    let eased = 1.0 - (1.0 - arrival).powi(3);
    let offset = Vec2::new(26.0, -20.0) * (1.0 - eased);
    let scale = 0.78 + 0.22 * eased;
    Rect::from_center_size(target.center() + offset, target.size() * scale)
}

fn result_line(game: &gamevex_core::baccarat::Baccarat) -> String {
    let outcome = match game.outcome {
        Some(Outcome::Win) => "WIN",
        Some(Outcome::Push) => "PUSH / BET RETURNED",
        Some(Outcome::Loss) => "LOSS",
        None => "ROUND COMPLETE",
    };
    let commission = if game.selected == Some(BetSide::Banker) && game.outcome == Some(Outcome::Win)
    {
        " / BANKER 5% COMMISSION / WHOLE CREDIT"
    } else {
        ""
    };
    let winner = match game.result {
        Some(BetSide::Player) => "PLAYER WINS",
        Some(BetSide::Banker) => "BANKER WINS",
        Some(BetSide::Tie) => "TIE",
        _ => "DEALING",
    };
    format!("{winner} / {outcome} / RETURN {}{commission}", game.payout)
}

fn fit_text(text: &str, max: usize) -> String {
    let max = max.max(4);
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        format!("{}...", text.chars().take(max - 3).collect::<String>())
    }
}

fn disabled_button(f: &mut Frame, r: Rect, label: &str, active: bool) {
    f.rect(r.min, r.size(), if active { PANEL } else { BG });
    f.outline(
        r.min,
        r.size(),
        1.0,
        if active { EDGE } else { EDGE.with_alpha(0.6) },
    );
    let font = if r.width() < 150.0 { 11.0 } else { 13.0 };
    let tx = r.min.x + (r.width() - label.chars().count() as f32 * font * 0.58) * 0.5;
    f.text(
        Vec2::new(tx.max(r.min.x + 4.0), r.min.y + (r.height() - font) * 0.5),
        label,
        font,
        if active { MUTED } else { EDGE },
    );
}
