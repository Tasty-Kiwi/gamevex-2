//! Presentation-only dealing timeline for Baccarat.
//!
//! The game model settles and saves immediately. This snapshot reveals the
//! dealt cards in order, then exposes the authoritative result and payout.

use gamevex_core::baccarat::{Baccarat, Command, Outcome, Phase};

const CARD_IN: f32 = 0.24;
const DEAL_STAGGER: f32 = 0.18;

#[derive(Debug, Clone)]
struct MotionPlan {
    elapsed: f32,
    player_starts: Vec<f32>,
    banker_starts: Vec<f32>,
    terminal: bool,
    display_balance: u64,
    display_phase: Phase,
    display_result: Option<gamevex_core::baccarat::BetSide>,
    display_outcome: Option<Outcome>,
    display_payout: u64,
    finished_at: f32,
}

/// Temporary Baccarat presentation state. It never mutates the game model.
#[derive(Debug, Default, Clone)]
pub struct BaccaratMotion {
    plan: Option<MotionPlan>,
}

impl BaccaratMotion {
    /// Start a timeline for a command that has already been applied.
    pub fn start(&mut self, command: Command, before: &Baccarat, after: &Baccarat) {
        self.clear();
        if command != Command::Deal || before == after {
            return;
        }

        let mut player_starts = vec![-CARD_IN; after.player.len()];
        let mut banker_starts = vec![-CARD_IN; after.banker.len()];

        // Initial cards arrive P1, B1, P2, B2; any rule-driven third cards
        // follow in player/banker order.
        if !player_starts.is_empty() {
            player_starts[0] = 0.0;
        }
        if !banker_starts.is_empty() {
            banker_starts[0] = DEAL_STAGGER;
        }
        if player_starts.len() > 1 {
            player_starts[1] = DEAL_STAGGER * 2.0;
        }
        if banker_starts.len() > 1 {
            banker_starts[1] = DEAL_STAGGER * 3.0;
        }
        if player_starts.len() > 2 {
            player_starts[2] = DEAL_STAGGER * 4.0;
        }
        if banker_starts.len() > 2 {
            banker_starts[2] = DEAL_STAGGER * 5.0;
        }

        let latest = player_starts
            .iter()
            .chain(banker_starts.iter())
            .copied()
            .fold(0.0_f32, f32::max);
        let finished_at = latest + CARD_IN;
        let terminal = matches!(after.phase, Phase::Settled | Phase::GameOver)
            && matches!(before.phase, Phase::Betting);

        self.plan = Some(MotionPlan {
            elapsed: 0.0,
            player_starts,
            banker_starts,
            terminal,
            // A committed wager is already deducted while the cards are dealt.
            display_balance: before.balance.saturating_sub(before.bet),
            display_phase: before.phase,
            display_result: before.result,
            display_outcome: before.outcome,
            display_payout: before.payout,
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

    pub fn clear(&mut self) {
        self.plan = None;
    }

    /// Return the cards and bookkeeping that should be visible this frame.
    /// Reduced motion and completed timelines return the authoritative state.
    pub fn sample(&self, authoritative: &Baccarat, reduced: bool) -> Baccarat {
        let Some(plan) = self.plan.as_ref().filter(|_| !reduced) else {
            return authoritative.clone();
        };
        let mut sample = authoritative.clone();
        sample
            .player
            .truncate(visible_count(&plan.player_starts, plan.elapsed));
        sample
            .banker
            .truncate(visible_count(&plan.banker_starts, plan.elapsed));
        if plan.terminal {
            sample.phase = plan.display_phase;
            sample.balance = plan.display_balance;
            sample.result = plan.display_result;
            sample.outcome = plan.display_outcome;
            sample.payout = plan.display_payout;
        }
        sample
    }

    /// Entrance progress for a card, clamped to 0..=1.
    pub fn card_progress(&self, player: bool, index: usize, reduced: bool) -> f32 {
        let Some(plan) = self.plan.as_ref().filter(|_| !reduced) else {
            return 1.0;
        };
        let starts = if player {
            &plan.player_starts
        } else {
            &plan.banker_starts
        };
        starts
            .get(index)
            .map_or(1.0, |start| unit((plan.elapsed - start) / CARD_IN))
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
    use gamevex_core::cards::{Card, Suit};

    const STEP: f32 = DEAL_STAGGER + 0.0001;

    fn card(rank: u8) -> Card {
        Card::new(rank, Suit::Spades, true)
    }

    fn terminal_round() -> (Baccarat, Baccarat) {
        let mut before = Baccarat::new(17);
        before.balance = 400;
        before.bet = 20;
        before.selected = Some(gamevex_core::baccarat::BetSide::Banker);
        let mut after = before.clone();
        after.player = vec![card(7), card(8), card(4)];
        after.banker = vec![card(9), card(8), card(2)];
        after.balance = 419;
        after.phase = Phase::Settled;
        after.result = Some(gamevex_core::baccarat::BetSide::Banker);
        after.outcome = Some(Outcome::Win);
        after.payout = 39;
        (before, after)
    }

    #[test]
    fn deal_reveals_player_banker_alternation_then_third_cards() {
        let (before, after) = terminal_round();
        let mut motion = BaccaratMotion::default();
        motion.start(Command::Deal, &before, &after);

        let initial = motion.sample(&after, false);
        assert_eq!(initial.player.len(), 1);
        assert_eq!(initial.banker.len(), 0);
        motion.tick(STEP);
        let second = motion.sample(&after, false);
        assert_eq!(second.player.len(), 1);
        assert_eq!(second.banker.len(), 1);
        motion.tick(STEP);
        let pair = motion.sample(&after, false);
        assert_eq!(pair.player.len(), 2);
        assert_eq!(pair.banker.len(), 1);
        motion.tick(STEP);
        let initial_four = motion.sample(&after, false);
        assert_eq!(initial_four.player.len(), 2);
        assert_eq!(initial_four.banker.len(), 2);
        motion.tick(STEP);
        assert_eq!(motion.sample(&after, false).player.len(), 3);
        assert_eq!(motion.sample(&after, false).banker.len(), 2);
        motion.tick(STEP);
        assert_eq!(motion.sample(&after, false).banker.len(), 3);
    }

    #[test]
    fn balance_and_result_are_staged_until_cards_finish_then_match_exactly() {
        let (before, after) = terminal_round();
        let mut motion = BaccaratMotion::default();
        motion.start(Command::Deal, &before, &after);

        let staged = motion.sample(&after, false);
        assert_eq!(staged.balance, before.balance - before.bet);
        assert_eq!(staged.phase, Phase::Betting);
        assert_eq!(staged.result, None);
        assert_eq!(staged.outcome, None);
        assert_eq!(staged.payout, 0);

        motion.tick(2.0);
        assert!(!motion.busy());
        assert_eq!(motion.sample(&after, false), after);
    }

    #[test]
    fn reduced_motion_uses_authoritative_round_immediately() {
        let (before, after) = terminal_round();
        let mut motion = BaccaratMotion::default();
        motion.start(Command::Deal, &before, &after);
        assert_eq!(motion.sample(&after, true), after);
        assert_eq!(motion.card_progress(true, 0, true), 1.0);
        assert!(!motion.busy() || motion.sample(&after, true) == after);
    }
}
