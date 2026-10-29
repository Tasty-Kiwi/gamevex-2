//! Presentation-only timing for Blackjack card dealing and dealer reveal.
//!
//! The game model remains authoritative and can be saved immediately. This
//! state only supplies a temporary visual snapshot while cards are entering or
//! the dealer is revealing the hole card.

use gamevex_core::blackjack::{Blackjack, Command, Phase};
use gamevex_core::cards::blackjack_value;

const CARD_IN: f32 = 0.22;
const DEALER_STAGGER: f32 = 0.14;
const PLAYER_STAGGER: f32 = 0.14;
const ACTION_STAGGER: f32 = 0.20;
const REVEAL_DURATION: f32 = 0.34;
const TERMINAL_PAUSE: f32 = 0.14;

#[derive(Debug, Clone)]
struct MotionPlan {
    elapsed: f32,
    dealer_starts: Vec<f32>,
    hand_starts: Vec<Vec<f32>>,
    reveal: Option<(f32, f32)>,
    settled: bool,
    display_balance: u64,
    display_outcome: Option<gamevex_core::blackjack::HandOutcome>,
    display_message: Option<String>,
    display_phase: Phase,
    finished_at: f32,
}

/// Temporary Blackjack presentation state. It never mutates the model.
#[derive(Debug, Default, Clone)]
pub struct BlackjackMotion {
    plan: Option<MotionPlan>,
}

impl BlackjackMotion {
    /// Start a timeline for an already-applied model command.
    pub fn start(&mut self, command: Command, before: &Blackjack, after: &Blackjack) {
        self.clear();
        if before == after
            || matches!(
                command,
                Command::SetBet(_)
                    | Command::SetHandCount(_)
                    | Command::NextRound
                    | Command::Restart
            )
        {
            return;
        }

        let mut dealer_starts = vec![-CARD_IN; after.dealer.len()];
        let mut hand_starts: Vec<Vec<f32>> = after
            .hands
            .iter()
            .map(|hand| vec![-CARD_IN; hand.cards.len()])
            .collect();

        match command {
            Command::Deal => {
                for (index, start) in dealer_starts.iter_mut().take(2).enumerate() {
                    *start = index as f32 * DEALER_STAGGER;
                }
                let first_player = DEALER_STAGGER + TERMINAL_PAUSE;
                let mut next = first_player;
                for starts in &mut hand_starts {
                    for start in starts {
                        *start = next;
                        next += PLAYER_STAGGER;
                    }
                }
            }
            Command::Split => {
                // Keep the two original cards in their new hands stationary;
                // only animate the replacement card drawn for each hand.
                for (hand_index, starts) in hand_starts.iter_mut().enumerate() {
                    for (index, start) in starts.iter_mut().enumerate() {
                        if index == 0 {
                            *start = -CARD_IN;
                        } else {
                            *start = (hand_index + index - 1) as f32 * ACTION_STAGGER;
                        }
                    }
                }
            }
            _ => {
                // Existing cards remain visible; newly appended cards enter
                // one at a time, including a split-created second hand.
                for (hand_index, starts) in hand_starts.iter_mut().enumerate() {
                    let old_len = before.hands.get(hand_index).map_or(0, |h| h.cards.len());
                    for (index, start) in starts.iter_mut().enumerate().skip(old_len) {
                        *start = (index - old_len) as f32 * ACTION_STAGGER;
                    }
                }
                let common_dealer_cards = before.dealer.len().min(after.dealer.len());
                for (index, start) in dealer_starts
                    .iter_mut()
                    .enumerate()
                    .skip(common_dealer_cards)
                {
                    *start = (index - common_dealer_cards) as f32 * ACTION_STAGGER;
                }
            }
        }

        let terminal = matches!(after.phase, Phase::Settled | Phase::GameOver)
            && (matches!(before.phase, Phase::Insurance | Phase::PlayerTurn)
                || (command == Command::Deal && before.phase == Phase::Betting));
        let mut reveal = None;
        if terminal && !after.dealer.is_empty() {
            let latest_player = hand_starts
                .iter()
                .flat_map(|starts| starts.iter().copied())
                .fold(-CARD_IN, f32::max);
            let reveal_start = latest_player + CARD_IN + TERMINAL_PAUSE;
            reveal = Some((reveal_start, REVEAL_DURATION));
            let after_reveal = reveal_start + REVEAL_DURATION;
            let existing = if command == Command::Deal {
                2
            } else {
                before.dealer.len()
            }
            .min(after.dealer.len());
            for (offset, start) in dealer_starts.iter_mut().enumerate().skip(existing) {
                *start =
                    after_reveal + TERMINAL_PAUSE + (offset - existing) as f32 * ACTION_STAGGER;
            }
        }

        let cards_done = dealer_starts
            .iter()
            .chain(hand_starts.iter().flat_map(|starts| starts.iter()))
            .copied()
            .fold(0.0f32, f32::max)
            + CARD_IN;
        let mut finished_at = reveal
            .map(|(start, duration)| (start + duration).max(cards_done))
            .unwrap_or(cards_done);
        if command == Command::Stand && !terminal {
            // A short pause lets the caller visibly pulse the newly active
            // hand even though no card was drawn.
            finished_at = finished_at.max(0.20);
        }

        let settled = matches!(after.phase, Phase::Settled | Phase::GameOver)
            && !matches!(before.phase, Phase::Settled | Phase::GameOver);
        self.plan = Some(MotionPlan {
            elapsed: 0.0,
            dealer_starts,
            hand_starts,
            reveal,
            settled,
            // Deduct committed stakes immediately and defer dealer payouts.
            display_balance: {
                let before_stake: u64 = before.hands.iter().map(|h| h.bet).sum();
                let after_stake: u64 = after.hands.iter().map(|h| h.bet).sum();
                let committed = after_stake.saturating_sub(before_stake)
                    + after.insurance_bet.saturating_sub(before.insurance_bet);
                let refund = if command == Command::Surrender {
                    before
                        .hands
                        .get(before.active_hand)
                        .map_or(0, |h| h.bet / 2)
                } else {
                    0
                };
                before
                    .balance
                    .saturating_sub(committed)
                    .saturating_add(refund)
            },
            display_outcome: before.outcome,
            display_message: if settled {
                Some("DEALER'S TURN...".into())
            } else {
                before.message.clone()
            },
            display_phase: before.phase,
            finished_at,
        });
    }

    /// Advance the timeline by a non-negative number of seconds.
    pub fn tick(&mut self, dt: f32) {
        if let Some(plan) = &mut self.plan {
            plan.elapsed += if dt.is_finite() { dt.max(0.0) } else { 0.0 };
            if plan.elapsed >= plan.finished_at {
                self.clear();
            }
        }
    }

    pub fn busy(&self) -> bool {
        self.plan.is_some()
    }

    pub fn turn_progress(&self, reduced: bool) -> f32 {
        self.plan
            .as_ref()
            .filter(|_| !reduced)
            .map_or(1.0, |p| unit(p.elapsed / 0.22))
    }

    pub fn clear(&mut self) {
        self.plan = None;
    }

    /// Return the visible card subset and stage terminal bookkeeping until the
    /// animation completes. In reduced-motion mode the authoritative state is
    /// returned verbatim.
    pub fn sample(&self, authoritative: &Blackjack, reduced: bool) -> Blackjack {
        let Some(plan) = self.plan.as_ref().filter(|_| !reduced) else {
            return authoritative.clone();
        };
        let mut sample = authoritative.clone();
        sample
            .dealer
            .truncate(visible_count(&plan.dealer_starts, plan.elapsed));
        for (index, hand) in sample.hands.iter_mut().enumerate() {
            let visible = plan
                .hand_starts
                .get(index)
                .map_or(hand.cards.len(), |starts| {
                    visible_count(starts, plan.elapsed)
                });
            hand.cards.truncate(visible);
        }

        if plan.settled {
            sample.phase = if sample.hands.is_empty() {
                plan.display_phase
            } else {
                Phase::PlayerTurn
            };
            sample.balance = plan.display_balance;
            sample.outcome = plan.display_outcome;
            sample.message = plan.display_message.clone();
            for hand in &mut sample.hands {
                hand.outcome = None;
            }
        }
        sample
    }

    /// Card entrance amount and horizontal flip scale, each clamped to 0..=1.
    pub fn card_pose(&self, dealer: bool, hand: usize, index: usize, reduced: bool) -> (f32, f32) {
        let Some(plan) = self.plan.as_ref().filter(|_| !reduced) else {
            return (1.0, 1.0);
        };
        let start = if dealer {
            plan.dealer_starts.get(index)
        } else {
            plan.hand_starts
                .get(hand)
                .and_then(|starts| starts.get(index))
        };
        let entrance = start.map_or(1.0, |start| unit((plan.elapsed - start) / CARD_IN));
        let flip = if dealer && index == 0 {
            plan.reveal.map_or(1.0, |(start, duration)| {
                let progress = unit((plan.elapsed - start) / duration);
                if progress <= 0.5 {
                    1.0 - progress * 2.0
                } else {
                    (progress - 0.5) * 2.0
                }
            })
        } else {
            1.0
        };
        (entrance, flip)
    }

    /// Whether the dealer's first card is still visually face down.
    pub fn dealer_hidden(&self, authoritative: &Blackjack, reduced: bool) -> bool {
        if reduced {
            return !authoritative.dealer.is_empty()
                && matches!(authoritative.phase, Phase::Insurance | Phase::PlayerTurn);
        }
        let Some(plan) = self.plan.as_ref() else {
            return !authoritative.dealer.is_empty()
                && matches!(authoritative.phase, Phase::Insurance | Phase::PlayerTurn);
        };
        plan.reveal
            .map(|(start, duration)| plan.elapsed < start + duration * 0.5)
            .unwrap_or_else(|| {
                matches!(authoritative.phase, Phase::Insurance | Phase::PlayerTurn)
                    && !authoritative.dealer.is_empty()
            })
    }
}

/// Sum dealer up-cards only while the hole card is hidden.
pub fn visible_dealer_total(game: &Blackjack, hidden: bool) -> u8 {
    if hidden {
        blackjack_value(game.dealer.get(1..).unwrap_or_default())
    } else {
        blackjack_value(&game.dealer)
    }
}

fn visible_count(starts: &[f32], elapsed: f32) -> usize {
    starts.partition_point(|start| *start <= elapsed)
}

fn unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamevex_core::blackjack::{HandOutcome, PlayerHand};
    use gamevex_core::cards::{Card, Shoe, Suit};

    fn card(rank: u8, face_up: bool) -> Card {
        Card::new(rank, Suit::Spades, face_up)
    }

    fn live_game() -> Blackjack {
        let mut game = Blackjack::new(7);
        game.phase = Phase::PlayerTurn;
        game.balance = 900;
        game.dealer = vec![card(10, false), card(6, true)];
        game.hands = vec![PlayerHand {
            cards: vec![card(10, true), card(8, true)],
            bet: 100,
            doubled: false,
            surrendered: false,
            blackjack_eligible: true,
            stood: false,
            outcome: None,
        }];
        game
    }

    fn settled_from(before: &Blackjack) -> Blackjack {
        let mut after = before.clone();
        after.dealer.push(card(10, true));
        after.phase = Phase::Settled;
        after.balance = 1100;
        after.outcome = Some(HandOutcome::Win);
        after.message = Some("WON!".into());
        after.hands[0].outcome = Some(HandOutcome::Win);
        after
    }

    #[test]
    fn terminal_timeline_reveals_hole_mid_flip_and_hides_total_until_then() {
        let before = live_game();
        let after = settled_from(&before);
        let mut motion = BlackjackMotion::default();
        motion.start(Command::Stand, &before, &after);
        let staged_cards = motion.sample(&after, false);
        let hidden_total =
            visible_dealer_total(&staged_cards, motion.dealer_hidden(&staged_cards, false));
        assert_eq!(hidden_total, 6);
        assert_eq!(motion.sample(&after, false).dealer.len(), 2);

        // Existing player cards stay still; reveal begins after a brief pause.
        motion.tick(TERMINAL_PAUSE + 0.04);
        assert!(motion.dealer_hidden(&after, false));
        let (_, flip) = motion.card_pose(true, 0, 0, false);
        assert!(flip < 1.0 && flip > 0.0);
        motion.tick(REVEAL_DURATION * 0.5);
        assert!(!motion.dealer_hidden(&after, false));
        assert_eq!(
            visible_dealer_total(&motion.sample(&after, false), false),
            16
        );
        motion.tick(0.4);
        assert_eq!(
            visible_dealer_total(&motion.sample(&after, false), false),
            26
        );
    }

    #[test]
    fn terminal_results_and_payout_are_staged_until_timeline_finishes() {
        let before = live_game();
        let after = settled_from(&before);
        let mut motion = BlackjackMotion::default();
        motion.start(Command::Stand, &before, &after);

        let staged = motion.sample(&after, false);
        assert_eq!(staged.balance, before.balance);
        assert_eq!(staged.phase, Phase::PlayerTurn);
        assert_eq!(staged.outcome, before.outcome);
        assert_eq!(staged.hands[0].outcome, None);

        motion.tick(3.0);
        assert!(!motion.busy());
        assert_eq!(motion.sample(&after, false), after);
    }

    #[test]
    fn reduced_motion_returns_authoritative_state_without_busy_animation() {
        let before = live_game();
        let after = settled_from(&before);
        let mut motion = BlackjackMotion::default();
        motion.start(Command::Stand, &before, &after);
        assert_eq!(motion.sample(&after, true), after);
        assert_eq!(motion.card_pose(true, 0, 0, true), (1.0, 1.0));
        assert!(!motion.dealer_hidden(&after, true));
    }

    #[test]
    fn deal_staggers_initial_cards_and_visible_total_omits_hole_card() {
        let before = Blackjack::with_shoe(
            2,
            Shoe::from_cards(vec![
                card(10, true),
                card(6, true),
                card(9, true),
                card(7, true),
            ]),
        );
        let mut after = before.clone();
        after.phase = Phase::PlayerTurn;
        after.dealer = vec![card(10, false), card(6, true)];
        after.hands = vec![PlayerHand {
            cards: vec![card(9, true), card(7, true)],
            bet: 10,
            doubled: false,
            surrendered: false,
            blackjack_eligible: true,
            stood: false,
            outcome: None,
        }];
        let mut motion = BlackjackMotion::default();
        motion.start(Command::Deal, &before, &after);
        assert_eq!(motion.sample(&after, false).dealer.len(), 1);
        assert_eq!(visible_dealer_total(&after, true), 6);
        motion.tick(0.14);
        assert_eq!(motion.sample(&after, false).dealer.len(), 2);
        assert_eq!(motion.sample(&after, false).hands[0].cards.len(), 0);
    }

    #[test]
    fn natural_deal_shows_initial_pair_before_dealer_draws_and_payout() {
        let before = Blackjack::with_shoe(
            2,
            Shoe::from_cards(vec![
                card(5, true),
                card(6, true),
                card(14, true),
                card(13, true),
                card(6, true),
            ]),
        );
        let mut after = before.clone();
        assert!(after.command(Command::Deal));
        assert_eq!(after.phase, Phase::Settled);
        let mut motion = BlackjackMotion::default();
        motion.start(Command::Deal, &before, &after);
        assert_eq!(motion.sample(&after, false).dealer.len(), 1);
        assert_eq!(motion.sample(&after, false).balance, 990);
        motion.tick(0.15);
        let view = motion.sample(&after, false);
        assert_eq!(view.dealer.len(), 2);
        assert_eq!(
            visible_dealer_total(&view, motion.dealer_hidden(&after, false)),
            6
        );
        motion.tick(3.0);
        assert_eq!(motion.sample(&after, false), after);
    }

    #[test]
    fn existing_cards_stay_still_when_another_card_is_drawn() {
        let before = live_game();
        let mut after = before.clone();
        after.hands[0].cards.push(card(2, true));
        let mut motion = BlackjackMotion::default();
        motion.start(Command::Hit, &before, &after);
        assert_eq!(motion.card_pose(false, 0, 0, false), (1.0, 1.0));
        assert_eq!(motion.card_pose(false, 0, 1, false), (1.0, 1.0));
        assert_eq!(motion.card_pose(false, 0, 2, false), (0.0, 1.0));
        motion.tick(0.11);
        assert_eq!(motion.card_pose(false, 0, 2, false), (0.5, 1.0));
    }
}
