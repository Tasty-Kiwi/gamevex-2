use serde::{Deserialize, Serialize};

use super::cards::{Card, Shoe, blackjack_value};

/// Match JavaScript's exact integer credit range for safe save and payout handling.
pub const MAX_CREDITS: u64 = 9_007_199_254_740_991;
/// Per-hand wager ceiling keeps the presentation's configured five-hand total
/// bet display within the same exact-integer credit range.
pub const MAX_BET: u64 = MAX_CREDITS / 5;
const MAX_EMERGENCY_REFILLS_PER_ROUND: u8 = 4;

pub fn hand_value(cards: &[Card]) -> u8 {
    blackjack_value(cards)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Betting,
    Insurance,
    PlayerTurn,
    Settled,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandOutcome {
    Surrendered,
    Bust,
    Blackjack,
    Win,
    Push,
    Loss,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerHand {
    pub cards: Vec<Card>,
    pub bet: u64,
    pub doubled: bool,
    pub surrendered: bool,
    pub blackjack_eligible: bool,
    pub stood: bool,
    pub outcome: Option<HandOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    SetBet(u64),
    SetHandCount(usize),
    Deal,
    Hit,
    Stand,
    Double,
    Split,
    Surrender,
    Insure,
    DeclineInsurance,
    NextRound,
    Restart,
    Abandon,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blackjack {
    pub balance: u64,
    pub bet: u64,
    pub hand_count: usize,
    pub phase: Phase,
    pub hands: Vec<PlayerHand>,
    pub dealer: Vec<Card>,
    pub active_hand: usize,
    pub message: Option<String>,
    pub outcome: Option<HandOutcome>,
    pub shoe: Shoe,
    pub seed: u64,
    pub insurance_bet: u64,
    pub emergency_refills: u8,
}

impl Blackjack {
    pub fn new(seed: u64) -> Self {
        Self {
            balance: 1000,
            bet: 10,
            hand_count: 1,
            phase: Phase::Betting,
            hands: vec![],
            dealer: vec![],
            active_hand: 0,
            message: Some("PLACE YOUR BET".into()),
            outcome: None,
            shoe: Shoe::shuffled(seed),
            seed,
            insurance_bet: 0,
            emergency_refills: 0,
        }
    }

    pub fn with_shoe(seed: u64, shoe: Shoe) -> Self {
        let mut game = Self::new(seed);
        game.shoe = shoe;
        game
    }

    fn playable_count(&self) -> usize {
        if self.balance == 0 {
            self.hand_count.clamp(1, 5)
        } else {
            self.hand_count
                .clamp(1, 5)
                .min(self.balance.min(usize::MAX as u64) as usize)
                .max(1)
        }
    }
    fn draw(&mut self) -> Option<Card> {
        if self.shoe.is_empty() && self.shoe.reshuffle_when_low {
            if self.emergency_refills >= MAX_EMERGENCY_REFILLS_PER_ROUND {
                return None;
            }
            self.shoe = Shoe::shuffled(self.seed.wrapping_add(1));
            self.seed = self.seed.wrapping_add(1);
            self.emergency_refills += 1;
        }
        self.shoe.draw()
    }
    pub fn can(&self, cmd: Command) -> bool {
        self.legal(cmd)
    }
    pub fn legal(&self, cmd: Command) -> bool {
        match cmd {
            Command::SetBet(n) => self.phase == Phase::Betting && n > 0 && n <= MAX_BET,
            Command::SetHandCount(n) => self.phase == Phase::Betting && (1..=5).contains(&n),
            Command::Deal => {
                self.phase == Phase::Betting
                    && self.bet > 0
                    && self.balance > 0
                    && (self.shoe.reshuffle_when_low
                        || self.shoe.len() >= 2 + 2 * self.playable_count())
            }
            Command::Insure => {
                self.phase == Phase::Insurance && self.balance >= self.insurance_cost()
            }
            Command::DeclineInsurance => self.phase == Phase::Insurance,
            Command::Hit => {
                self.phase == Phase::PlayerTurn
                    && self.has_draws(1)
                    && self
                        .active()
                        .is_some_and(|h| !h.stood && hand_value(&h.cards) < 21)
            }
            Command::Stand => {
                self.phase == Phase::PlayerTurn && self.active().is_some_and(|h| !h.stood)
            }
            Command::Double => {
                self.phase == Phase::PlayerTurn
                    && self.has_draws(1)
                    && self
                        .active()
                        .is_some_and(|h| h.cards.len() == 2 && !h.stood && self.balance >= h.bet)
            }
            Command::Split => {
                self.phase == Phase::PlayerTurn
                    && self.hands.len() == 1
                    && self.has_draws(2)
                    && self.active().is_some_and(|h| {
                        h.cards.len() == 2
                            && h.cards[0].rank == h.cards[1].rank
                            && self.balance >= h.bet
                    })
            }
            Command::Surrender => {
                self.phase == Phase::PlayerTurn
                    && self.active().is_some_and(|h| {
                        h.cards.len() == 2 && h.blackjack_eligible && !h.surrendered && !h.stood
                    })
            }
            Command::NextRound => self.phase == Phase::Settled,
            Command::Restart => self.phase == Phase::GameOver,
            Command::Abandon => matches!(self.phase, Phase::Insurance | Phase::PlayerTurn),
        }
    }
    fn active(&self) -> Option<&PlayerHand> {
        self.hands.get(self.active_hand)
    }
    fn has_draws(&self, count: usize) -> bool {
        self.shoe.len() >= count
            || (self.shoe.reshuffle_when_low
                && self.emergency_refills < MAX_EMERGENCY_REFILLS_PER_ROUND)
    }
    fn insurance_cost(&self) -> u64 {
        let total = self
            .hands
            .iter()
            .fold(0u64, |sum, hand| sum.saturating_add(hand.bet));
        total / 2 + total % 2
    }
    pub fn command(&mut self, cmd: Command) -> bool {
        if !self.legal(cmd) {
            return false;
        }
        let before = self.clone();
        match cmd {
            Command::SetBet(n) => self.bet = n,
            Command::SetHandCount(n) => self.hand_count = n,
            Command::Deal => self.deal(),
            Command::Hit => {
                let c = self.draw().expect("validated shoe draw");
                self.hands[self.active_hand].cards.push(c);
                if hand_value(&self.hands[self.active_hand].cards) >= 21 {
                    self.advance();
                }
            }
            Command::Stand => {
                self.hands[self.active_hand].stood = true;
                self.advance()
            }
            Command::Double => {
                let bet = self.hands[self.active_hand].bet;
                self.balance -= bet;
                self.hands[self.active_hand].bet = bet * 2;
                self.hands[self.active_hand].doubled = true;
                let c = self.draw().expect("validated shoe draw");
                self.hands[self.active_hand].cards.push(c);
                self.hands[self.active_hand].stood = true;
                self.advance()
            }
            Command::Split => {
                let bet = self.hands[0].bet;
                self.balance -= bet;
                let old = self.hands.remove(0);
                let first = old.cards[0];
                let second = old.cards[1];
                let mut a = old.clone();
                let mut b = old;
                a.cards = vec![first, self.draw().expect("shoe")];
                b.cards = vec![second, self.draw().expect("shoe")];
                for h in [&mut a, &mut b] {
                    h.blackjack_eligible = false;
                    h.doubled = false;
                    h.surrendered = false;
                    h.stood = false;
                    h.outcome = None;
                }
                self.hands = vec![a, b];
                self.hands[0].bet = bet;
                self.hands[1].bet = bet;
                if hand_value(&self.hands[0].cards) == 21 {
                    self.hands[0].stood = true;
                    self.advance();
                }
            }
            Command::Surrender => {
                let h = &mut self.hands[self.active_hand];
                h.surrendered = true;
                h.stood = true;
                h.outcome = Some(HandOutcome::Surrendered);
                self.balance += h.bet / 2;
                self.advance()
            }
            Command::Insure => {
                let cost = self.insurance_cost();
                self.balance -= cost;
                self.insurance_bet = cost;
                self.evaluate_initial();
            }
            Command::DeclineInsurance => self.evaluate_initial(),
            Command::NextRound => self.clear_round(),
            Command::Restart => {
                let hand_count = self.hand_count;
                *self = Self::new(self.seed);
                self.hand_count = hand_count;
            }
            Command::Abandon => {
                for hand in &mut self.hands {
                    if hand.outcome.is_none() {
                        hand.outcome = Some(if hand.surrendered {
                            HandOutcome::Surrendered
                        } else {
                            HandOutcome::Loss
                        });
                    }
                }
                self.phase = if self.balance == 0 {
                    Phase::GameOver
                } else {
                    Phase::Settled
                };
                self.message = Some("ROUND ABANDONED".into());
            }
        }
        *self != before
    }
    fn deal(&mut self) {
        let count = self.playable_count();
        if self.shoe.reshuffle_when_low && self.shoe.len() < 20 + count * 4 {
            self.shoe = Shoe::shuffled(self.seed.wrapping_add(1));
            self.seed = self.seed.wrapping_add(1);
        }
        self.emergency_refills = 0;
        let wager = self.bet.min(self.balance / count as u64);
        if wager == 0 {
            return;
        }
        self.balance -= wager * count as u64;
        self.insurance_bet = 0;
        self.outcome = None;
        self.hands.clear();
        self.dealer.clear();
        self.active_hand = 0;
        // Match source order: dealer receives both cards before players, second is up card.
        let d1 = self.draw().expect("shoe");
        let d2 = self.draw().expect("shoe");
        self.dealer = vec![d1, d2];
        for _ in 0..count {
            let a = self.draw().expect("shoe");
            let b = self.draw().expect("shoe");
            self.hands.push(PlayerHand {
                cards: vec![a, b],
                bet: wager,
                doubled: false,
                surrendered: false,
                blackjack_eligible: true,
                stood: false,
                outcome: None,
            });
        }
        if d2.rank == 14 {
            self.phase = Phase::Insurance;
            self.message = Some("BUY INSURANCE?".into());
        } else {
            self.evaluate_initial();
        }
    }
    fn evaluate_initial(&mut self) {
        if let Some(i) = self.hands.iter().position(|h| hand_value(&h.cards) < 21) {
            self.active_hand = i;
            self.phase = Phase::PlayerTurn;
            self.message = Some(if self.hands.len() > 1 {
                format!("PLAYING HAND {} OF {}", i + 1, self.hands.len())
            } else {
                "YOUR TURN!".into()
            });
        } else {
            self.dealer_play();
        }
    }
    fn advance(&mut self) {
        if let Some(i) = self
            .hands
            .iter()
            .enumerate()
            .skip(self.active_hand + 1)
            .find(|(_, h)| !h.stood && hand_value(&h.cards) < 21)
            .map(|(i, _)| i)
        {
            self.active_hand = i;
            self.message = Some(if self.hands.len() > 1 {
                format!("PLAYING HAND {} OF {}", i + 1, self.hands.len())
            } else {
                "YOUR TURN!".into()
            });
        } else {
            self.dealer_play();
        }
    }
    fn dealer_play(&mut self) {
        if !self
            .hands
            .iter()
            .all(|h| h.surrendered || hand_value(&h.cards) > 21)
        {
            while hand_value(&self.dealer) < 17 {
                let Some(c) = self.draw() else { break };
                self.dealer.push(c);
            }
        }
        self.settle();
    }
    fn settle(&mut self) {
        let dv = hand_value(&self.dealer);
        // Keep half-dollar units through all hands. The TS implementation
        // rounds the combined payout once, after evaluating every hand.
        let mut winnings_halves = 0u64;
        let mut parts = vec![];
        if dv == 21 && self.dealer.len() == 2 && self.insurance_bet > 0 {
            winnings_halves += self.insurance_bet * 6;
            parts.push("INSURANCE PAID!".to_string());
        }
        for i in 0..self.hands.len() {
            let (pv, bet, surrendered, eligible, card_count) = {
                let h = &self.hands[i];
                (
                    hand_value(&h.cards),
                    h.bet,
                    h.surrendered,
                    h.blackjack_eligible,
                    h.cards.len(),
                )
            };
            let out = if surrendered {
                HandOutcome::Surrendered
            } else if pv > 21 {
                HandOutcome::Bust
            } else if dv > 21 || pv > dv {
                if pv == 21 && card_count == 2 && eligible {
                    HandOutcome::Blackjack
                } else {
                    HandOutcome::Win
                }
            } else if pv == dv {
                HandOutcome::Push
            } else {
                HandOutcome::Loss
            };
            self.hands[i].outcome = Some(out);
            let prefix = if self.hands.len() > 1 {
                format!("HAND {} ", i + 1)
            } else {
                String::new()
            };
            match out {
                HandOutcome::Blackjack => {
                    winnings_halves += bet * 5;
                    parts.push(format!("{prefix}BLACKJACK!"));
                }
                HandOutcome::Win => {
                    winnings_halves += bet * 4;
                    parts.push(format!("{prefix}WON!"));
                }
                HandOutcome::Push => {
                    winnings_halves += bet * 2;
                    parts.push(format!("{prefix}PUSH."));
                }
                HandOutcome::Bust => parts.push(format!("{prefix}BUST.")),
                HandOutcome::Surrendered => parts.push(format!("{prefix}SURRENDERED.")),
                HandOutcome::Loss => parts.push(format!("{prefix}LOST.")),
            }
        }
        let winnings = winnings_halves.div_ceil(2);
        self.balance = self.balance.saturating_add(winnings).min(MAX_CREDITS);
        self.phase = if self.balance == 0 {
            Phase::GameOver
        } else {
            Phase::Settled
        };
        self.message = Some(parts.join(" "));
    }
    fn clear_round(&mut self) {
        self.hands.clear();
        self.dealer.clear();
        self.insurance_bet = 0;
        self.emergency_refills = 0;
        self.active_hand = 0;
        self.phase = if self.balance == 0 {
            Phase::GameOver
        } else {
            Phase::Betting
        };
        self.message = Some("PLACE YOUR BET".into());
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.hand_count == 0 || self.hand_count > 5 {
            return Err("hand_count must be 1..=5".into());
        }
        if self.bet == 0 || self.bet > MAX_BET {
            return Err("bet must be within the supported per-hand range".into());
        }
        if self.balance > MAX_CREDITS || self.bet > MAX_CREDITS || self.insurance_bet > MAX_CREDITS
        {
            return Err("credit amount exceeds the exact integer limit".into());
        }
        if self.emergency_refills > MAX_EMERGENCY_REFILLS_PER_ROUND {
            return Err("round emergency refill limit exceeded".into());
        }
        if self.hands.len() > 5 {
            return Err("a round cannot contain more than five hands".into());
        }
        if self.active_hand >= self.hands.len() && !self.hands.is_empty() {
            return Err("active hand out of range".into());
        }
        if self.hands.is_empty() && self.active_hand != 0 {
            return Err("empty round must have active_hand zero".into());
        }
        if self.hands.iter().any(|h| {
            h.bet == 0
                || h.bet > MAX_CREDITS
                || h.cards.len() < 2
                || h.cards.iter().any(|c| !(2..=14).contains(&c.rank))
                || (h.surrendered
                    && (!h.stood
                        || h.outcome
                            .is_some_and(|outcome| outcome != HandOutcome::Surrendered)))
        }) || self.dealer.iter().any(|c| !(2..=14).contains(&c.rank))
            || self.shoe.cards.iter().any(|c| !(2..=14).contains(&c.rank))
        {
            return Err("invalid card or wager".into());
        }
        if self.phase == Phase::Insurance
            && (self.hands.is_empty()
                || self.dealer.len() != 2
                || self.dealer[1].rank != 14
                || self.active_hand != 0
                || self.insurance_bet != 0
                || self
                    .hands
                    .iter()
                    .any(|h| h.cards.len() != 2 || h.stood || h.surrendered || h.outcome.is_some()))
        {
            return Err("invalid insurance state".into());
        }
        if self.phase == Phase::PlayerTurn
            && (self.hands.is_empty()
                || self.active_hand >= self.hands.len()
                || self.dealer.len() != 2
                || self.hands[self.active_hand].stood
                || self.hands[self.active_hand].surrendered
                || self.hands[self.active_hand].outcome.is_some()
                || hand_value(&self.hands[self.active_hand].cards) >= 21)
        {
            return Err("invalid player turn".into());
        }
        if matches!(
            self.phase,
            Phase::Insurance | Phase::PlayerTurn | Phase::Settled | Phase::GameOver
        ) && (self.hands.is_empty() || self.dealer.len() < 2)
        {
            return Err("completed or active round must contain dealt hands".into());
        }
        if self.phase == Phase::PlayerTurn
            && self
                .hands
                .iter()
                .take(self.active_hand)
                .any(|h| !h.stood && !h.surrendered && hand_value(&h.cards) < 21)
        {
            return Err("player turn skipped a playable hand".into());
        }
        if self.phase == Phase::PlayerTurn
            && self
                .hands
                .iter()
                .skip(self.active_hand + 1)
                .any(|h| h.stood || h.surrendered || h.outcome.is_some())
        {
            return Err("player turn has already-resolved later hands".into());
        }
        if self.phase == Phase::Betting && (!self.hands.is_empty() || !self.dealer.is_empty()) {
            return Err("betting state cannot contain an active round".into());
        }
        if self.phase == Phase::Betting && (self.balance == 0 || self.insurance_bet != 0) {
            return Err("invalid betting bankroll state".into());
        }
        if self.phase == Phase::GameOver && self.balance != 0 {
            return Err("game-over phase requires zero balance".into());
        }
        if self.phase == Phase::Settled && self.balance == 0 {
            return Err("zero balance must be in game-over phase".into());
        }
        if matches!(self.phase, Phase::Settled | Phase::GameOver)
            && self.hands.iter().any(|hand| hand.outcome.is_none())
        {
            return Err("finished rounds must have an outcome for every hand".into());
        }
        if self.insurance_bet > 0
            && (self.dealer.get(1).is_none_or(|card| card.rank != 14)
                || self.insurance_bet > self.insurance_cost())
        {
            return Err("invalid insurance stake".into());
        }
        let card_count = self.shoe.len()
            + self.dealer.len()
            + self.hands.iter().map(|h| h.cards.len()).sum::<usize>();
        if self.shoe.cards.len() > 52
            || self.dealer.len() > 52
            || self.hands.iter().any(|hand| hand.cards.len() > 52)
            || card_count > 52 * (1 + self.emergency_refills as usize)
        {
            return Err("card count exceeds the bounded shoe/refill state".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Suit;

    fn c(rank: u8) -> Card {
        Card::new(rank, Suit::Spades, true)
    }
    fn fixed(cards: &[u8]) -> Shoe {
        Shoe::from_cards(cards.iter().copied().map(c).collect())
    }

    #[test]
    fn bankrupt_restart_restores_entry_credits_and_keeps_configured_hand_count() {
        let mut game = Blackjack::with_shoe(1, fixed(&[10, 10, 2, 3, 2, 3]));
        game.balance = 2;
        game.bet = 1;
        game.hand_count = 2;
        game.command(Command::Deal);
        game.command(Command::Stand);
        game.command(Command::Stand);
        assert_eq!(game.phase, Phase::GameOver);
        assert_eq!(game.balance, 0);
        assert!(game.command(Command::Restart));
        assert_eq!(game.balance, 1000);
        assert_eq!(game.hand_count, 2);
        assert_eq!(game.phase, Phase::Betting);
        assert!(game.validate().is_ok());
    }

    #[test]
    fn fixed_shoe_produces_natural_and_rounds_half_credit_payout() {
        // draw order: dealer 10,7; player A,K
        let mut game = Blackjack::with_shoe(1, fixed(&[10, 7, 14, 13]));
        assert!(game.command(Command::Deal));
        assert_eq!(game.phase, Phase::Settled);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Blackjack));
        assert_eq!(game.balance, 1015);
        assert_eq!(game.message.as_deref(), Some("BLACKJACK!"));
    }

    #[test]
    fn odd_natural_payouts_are_rounded_once_across_all_hands() {
        // Two one-credit naturals produce 5 credits total, not two separately
        // rounded 3-credit returns. Dealer starts at 11 and draws to 17.
        let mut game = Blackjack::with_shoe(19, fixed(&[5, 6, 14, 13, 14, 13, 6]));
        game.balance = 3;
        game.bet = 1;
        game.hand_count = 2;
        game.command(Command::Deal);
        assert_eq!(game.phase, Phase::Settled);
        assert!(
            game.hands
                .iter()
                .all(|h| h.outcome == Some(HandOutcome::Blackjack))
        );
        assert_eq!(game.balance, 6);
    }

    #[test]
    fn insurance_natural_pays_three_times_and_keeps_integer_credits() {
        // dealer A,10; player 10,9. Cost is ceil(10/2) = 5.
        let mut game = Blackjack::with_shoe(2, fixed(&[10, 14, 10, 9]));
        game.command(Command::Deal);
        assert_eq!(game.phase, Phase::Insurance);
        game.command(Command::Insure);
        assert_eq!(game.phase, Phase::PlayerTurn);
        game.command(Command::Stand);
        assert_eq!(game.phase, Phase::Settled);
        assert_eq!(game.insurance_bet, 5);
        assert_eq!(game.balance, 1000);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Loss));
    }

    #[test]
    fn odd_five_credit_insurance_cost_and_natural_payout_match_reference() {
        let mut game = Blackjack::with_shoe(18, fixed(&[10, 14, 10, 9, 2]));
        game.balance = 8;
        game.bet = 5;
        game.command(Command::Deal);
        assert_eq!(game.balance, 3);
        assert_eq!(game.phase, Phase::Insurance);
        game.command(Command::Insure);
        assert_eq!(game.insurance_bet, 3);
        assert_eq!(game.balance, 0);
        game.command(Command::Stand);
        assert_eq!(game.phase, Phase::Settled);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Loss));
        assert_eq!(game.balance, 9);
    }

    #[test]
    fn insurance_is_illegal_when_the_premium_is_not_affordable() {
        let mut game = Blackjack::with_shoe(25, fixed(&[10, 14, 10, 9]));
        game.balance = 10;
        game.command(Command::Deal);
        assert_eq!(game.balance, 0);
        assert!(!game.legal(Command::Insure));
        assert!(!game.command(Command::Insure));
        assert_eq!(game.phase, Phase::Insurance);
        assert_eq!(game.balance, 0);
        assert!(game.command(Command::DeclineInsurance));
        assert_eq!(game.phase, Phase::PlayerTurn);
    }

    #[test]
    fn split_removes_natural_eligibility_and_splits_once() {
        // dealer 5,10; player 8,8; split draws 3 and 2
        let mut game = Blackjack::with_shoe(3, fixed(&[5, 10, 8, 8, 3, 2, 2, 2, 2]));
        game.command(Command::Deal);
        assert!(game.command(Command::Split));
        assert_eq!(game.hands.len(), 2);
        assert!(game.hands.iter().all(|h| !h.blackjack_eligible));
        assert!(!game.legal(Command::Split));
        assert_eq!(game.balance, 980);
    }

    #[test]
    fn surrender_refunds_floor_half_stake() {
        let mut game = Blackjack::with_shoe(4, fixed(&[10, 7, 9, 8]));
        game.balance = 5;
        game.bet = 5;
        game.command(Command::Deal);
        assert!(game.command(Command::Surrender));
        assert_eq!(game.balance, 2);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Surrendered));
    }

    #[test]
    fn dealer_natural_with_insurance_and_player_natural_pushes_both_wagers() {
        let mut game = Blackjack::with_shoe(5, fixed(&[10, 14, 14, 13]));
        game.command(Command::Deal);
        assert_eq!(game.phase, Phase::Insurance);
        game.command(Command::Insure);
        assert_eq!(game.phase, Phase::Settled);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Push));
        // Base bet returned plus three times the five-credit insurance stake.
        assert_eq!(game.balance, 1010);
    }

    #[test]
    fn a_three_card_twenty_one_pushes_against_dealer_natural() {
        let mut game = Blackjack::with_shoe(6, fixed(&[10, 14, 10, 6, 5]));
        game.command(Command::Deal);
        game.command(Command::DeclineInsurance);
        game.command(Command::Hit);
        assert_eq!(game.phase, Phase::Settled);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Push));
        assert_eq!(game.balance, 1000);
    }

    #[test]
    fn split_aces_make_ordinary_wins_and_auto_skip_both_twenty_ones() {
        // Split A,A into A,10 and A,10 against dealer 5,10. Neither split hand
        // qualifies for a natural premium; both pay an ordinary 2x return.
        let mut game = Blackjack::with_shoe(7, fixed(&[5, 10, 14, 14, 10, 10, 2]));
        game.command(Command::Deal);
        assert!(game.command(Command::Split));
        assert_eq!(game.phase, Phase::Settled);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Win));
        assert_eq!(game.hands[1].outcome, Some(HandOutcome::Win));
        assert_eq!(game.balance, 1020);
        assert!(!game.legal(Command::Split));
    }

    #[test]
    fn split_is_only_available_for_equal_ranks_and_one_unsplit_hand() {
        let mut unequal = Blackjack::with_shoe(8, fixed(&[10, 7, 11, 13]));
        unequal.command(Command::Deal);
        assert_eq!(hand_value(&unequal.hands[0].cards), 20);
        assert!(!unequal.legal(Command::Split));

        let mut split = Blackjack::with_shoe(9, fixed(&[10, 7, 8, 8, 2, 3]));
        split.command(Command::Deal);
        assert!(split.command(Command::Split));
        assert_eq!(split.hands.len(), 2);
        assert!(!split.legal(Command::Split));
    }

    #[test]
    fn double_requires_a_two_card_hand_and_an_affordable_extra_wager() {
        let mut poor = Blackjack::with_shoe(10, fixed(&[10, 7, 10, 6, 2]));
        poor.balance = 19;
        poor.command(Command::Deal);
        assert_eq!(poor.balance, 9);
        assert!(!poor.legal(Command::Double));

        let mut game = Blackjack::with_shoe(11, fixed(&[10, 7, 10, 6, 2]));
        game.command(Command::Deal);
        assert!(game.legal(Command::Double));
        game.command(Command::Hit);
        assert_eq!(game.hands[0].cards.len(), 3);
        assert!(!game.legal(Command::Double));
    }

    #[test]
    fn configured_hand_count_scales_down_to_balance_and_affordable_wagers() {
        let mut game = Blackjack::with_shoe(12, fixed(&[10, 5, 2, 3, 2, 3, 2, 3, 2, 3, 2, 3]));
        game.balance = 12;
        game.hand_count = 5;
        assert!(game.command(Command::Deal));
        assert_eq!(game.hands.len(), 5);
        assert!(game.hands.iter().all(|h| h.bet == 2));
        assert_eq!(game.balance, 2);

        let mut one_credit = Blackjack::with_shoe(13, fixed(&[10, 5, 2, 3]));
        one_credit.balance = 1;
        one_credit.hand_count = 5;
        assert!(one_credit.command(Command::Deal));
        assert_eq!(one_credit.hands.len(), 1);
        assert_eq!(one_credit.hands[0].bet, 1);
    }

    #[test]
    fn hand_count_and_credit_limits_reject_invalid_or_overflowing_commands() {
        let mut game = Blackjack::new(14);
        assert!(!game.legal(Command::SetHandCount(0)));
        assert!(!game.legal(Command::SetHandCount(6)));
        assert!(game.command(Command::SetHandCount(5)));
        assert!(!game.legal(Command::SetBet(MAX_BET + 1)));
        assert!(game.command(Command::SetBet(MAX_BET)));

        game.balance = MAX_CREDITS + 1;
        assert!(game.validate().is_err());
        game.balance = 1000;
        game.hands.push(PlayerHand {
            cards: vec![c(10), c(10)],
            bet: MAX_CREDITS + 1,
            doubled: false,
            surrendered: false,
            blackjack_eligible: true,
            stood: false,
            outcome: None,
        });
        assert!(game.validate().is_err());

        let mut capped = Blackjack::with_shoe(20, fixed(&[10, 7, 14, 13]));
        capped.balance = MAX_CREDITS;
        capped.command(Command::Deal);
        assert_eq!(capped.balance, MAX_CREDITS);
        assert!(capped.validate().is_ok());
    }

    #[test]
    fn settled_round_cannot_be_settled_again_by_player_commands() {
        let mut game = Blackjack::with_shoe(15, fixed(&[10, 7, 14, 13]));
        game.command(Command::Deal);
        assert_eq!(game.phase, Phase::Settled);
        let balance = game.balance;
        let message = game.message.clone();
        for cmd in [
            Command::Hit,
            Command::Stand,
            Command::Double,
            Command::Surrender,
        ] {
            assert!(!game.command(cmd));
        }
        assert_eq!(game.balance, balance);
        assert_eq!(game.message, message);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Blackjack));
        assert!(game.command(Command::NextRound));
        assert!(!game.command(Command::NextRound));
        assert_eq!(game.balance, balance);
    }

    #[test]
    fn dealer_hits_below_seventeen_and_stands_on_seventeen() {
        let mut hits = Blackjack::with_shoe(21, fixed(&[10, 6, 10, 6, 2]));
        hits.command(Command::Deal);
        hits.command(Command::Stand);
        assert_eq!(hits.dealer.len(), 3);
        assert_eq!(hand_value(&hits.dealer), 18);

        let mut stands = Blackjack::with_shoe(22, fixed(&[10, 7, 10, 6, 2]));
        stands.command(Command::Deal);
        stands.command(Command::Stand);
        assert_eq!(stands.dealer.len(), 2);
        assert_eq!(hand_value(&stands.dealer), 17);
    }

    #[test]
    fn abandoning_round_keeps_only_unspent_credits_and_skips_payout() {
        let mut game = Blackjack::with_shoe(23, fixed(&[10, 7, 10, 6, 2]));
        game.command(Command::Deal);
        assert_eq!(game.balance, 990);
        assert!(game.command(Command::Abandon));
        assert_eq!(game.phase, Phase::Settled);
        assert_eq!(game.balance, 990);
        assert_eq!(game.hands[0].outcome, Some(HandOutcome::Loss));
        assert!(!game.command(Command::Abandon));

        let mut all_in = Blackjack::with_shoe(24, fixed(&[10, 7, 10, 6]));
        all_in.balance = 10;
        all_in.command(Command::Deal);
        assert_eq!(all_in.balance, 0);
        assert!(all_in.command(Command::Abandon));
        assert_eq!(all_in.phase, Phase::GameOver);
        assert_eq!(all_in.balance, 0);

        let mut surrendered = Blackjack::with_shoe(28, fixed(&[10, 7, 10, 8, 10, 8]));
        surrendered.balance = 100;
        surrendered.bet = 5;
        surrendered.hand_count = 2;
        surrendered.command(Command::Deal);
        surrendered.command(Command::Surrender);
        assert_eq!(surrendered.balance, 92);
        surrendered.command(Command::Abandon);
        assert_eq!(surrendered.balance, 92);
        assert_eq!(surrendered.hands[0].outcome, Some(HandOutcome::Surrendered));
        assert_eq!(surrendered.hands[1].outcome, Some(HandOutcome::Loss));
        assert!(surrendered.validate().is_ok());
    }

    #[test]
    fn validation_rejects_corrupt_serialized_blackjack_states() {
        let mut game = Blackjack::with_shoe(16, fixed(&[10, 6, 10, 6, 2]));
        game.command(Command::Deal);
        let valid = serde_json::to_value(&game).unwrap();
        let roundtrip: Blackjack = serde_json::from_value(valid.clone()).unwrap();
        assert!(roundtrip.validate().is_ok());

        let mut bad_rank = valid.clone();
        bad_rank["hands"][0]["cards"][0]["rank"] = 1.into();
        assert!(
            serde_json::from_value::<Blackjack>(bad_rank)
                .unwrap()
                .validate()
                .is_err()
        );

        let mut bad_index = valid.clone();
        bad_index["active_hand"] = 9.into();
        assert!(
            serde_json::from_value::<Blackjack>(bad_index)
                .unwrap()
                .validate()
                .is_err()
        );

        let mut bad_phase = valid;
        bad_phase["dealer"] = serde_json::json!([]);
        assert!(
            serde_json::from_value::<Blackjack>(bad_phase)
                .unwrap()
                .validate()
                .is_err()
        );

        let mut bad_active = game.clone();
        bad_active.hands[bad_active.active_hand].stood = true;
        assert!(bad_active.validate().is_err());

        let mut bad_many_hands = game.clone();
        let hand = bad_many_hands.hands[0].clone();
        bad_many_hands.hands = vec![hand; 6];
        assert!(bad_many_hands.validate().is_err());

        let mut bad_shoe_size = game.clone();
        bad_shoe_size.shoe.cards = vec![c(2); 60];
        assert!(bad_shoe_size.validate().is_err());

        let mut bad_hand_size = game.clone();
        bad_hand_size.hands[0].cards = vec![c(2); 53];
        assert!(bad_hand_size.validate().is_err());

        let mut empty_active = Blackjack::new(26);
        empty_active.active_hand = 1;
        assert!(empty_active.validate().is_err());
    }

    #[test]
    fn fixed_shoe_deal_requires_enough_cards_for_every_initial_hand() {
        let mut game = Blackjack::with_shoe(27, fixed(&[10, 7, 10]));
        assert!(!game.legal(Command::Deal));
        assert!(!game.command(Command::Deal));
        assert_eq!(game.phase, Phase::Betting);
        assert!(game.hands.is_empty());
    }

    #[test]
    fn production_shoe_emergency_refills_on_empty_for_player_and_dealer_draws() {
        let mut player = Blackjack::with_shoe(29, fixed(&[10, 6, 10, 6]));
        player.command(Command::Deal);
        player.shoe.reshuffle_when_low = true;
        player.shoe.cards.clear();
        assert!(player.legal(Command::Hit));
        assert!(player.command(Command::Hit));
        assert_eq!(player.emergency_refills, 1);
        assert!(player.validate().is_ok());

        let mut dealer = Blackjack::with_shoe(30, fixed(&[10, 6, 10, 6]));
        dealer.command(Command::Deal);
        dealer.shoe.reshuffle_when_low = true;
        dealer.shoe.cards.clear();
        assert!(dealer.command(Command::Stand));
        assert_eq!(dealer.emergency_refills, 1);
        assert!(dealer.dealer.len() >= 3);
        assert!(dealer.validate().is_ok());

        let mut fixed_empty = Blackjack::with_shoe(31, fixed(&[10, 6, 10, 6]));
        fixed_empty.command(Command::Deal);
        fixed_empty.shoe.cards.clear();
        assert!(!fixed_empty.legal(Command::Hit));
        assert_eq!(fixed_empty.emergency_refills, 0);
    }

    #[test]
    fn ts_settlement_reference_json_fixtures_match_rust_outcomes() {
        // Expected values were generated by invoking blackjackSettlement.ts.
        let fixtures: Vec<serde_json::Value> = serde_json::from_str(
            r#"[
              {"name":"natural","draw":[10,7,14,13],"actions":["deal"],"outcome":"blackjack","balance":1015},
              {"name":"21 tie","draw":[10,14,10,6,5],"actions":["deal","decline","hit"],"outcome":"push","balance":1000},
              {"name":"split 21","draw":[5,10,14,14,10,10,2],"actions":["deal","split"],"outcome":"win","balance":1020}
            ]"#,
        )
        .unwrap();
        for fixture in fixtures {
            let draw: Vec<u8> = serde_json::from_value(fixture["draw"].clone()).unwrap();
            let mut game = Blackjack::with_shoe(17, fixed(&draw));
            let actions: Vec<String> = serde_json::from_value(fixture["actions"].clone()).unwrap();
            for action in actions {
                let command = match action.as_str() {
                    "deal" => Command::Deal,
                    "decline" => Command::DeclineInsurance,
                    "hit" => Command::Hit,
                    "split" => Command::Split,
                    _ => panic!("unknown fixture action"),
                };
                assert!(
                    game.command(command),
                    "fixture {}: {action}",
                    fixture["name"]
                );
            }
            assert_eq!(
                format!("{:?}", game.hands[0].outcome.unwrap()).to_lowercase(),
                fixture["outcome"].as_str().unwrap(),
                "fixture {}",
                fixture["name"]
            );
            assert_eq!(game.balance, fixture["balance"].as_u64().unwrap());
        }
    }

    #[test]
    fn deterministic_seed_and_save_validation() {
        assert_eq!(Blackjack::new(42).shoe, Blackjack::new(42).shoe);
        let mut game = Blackjack::new(1);
        assert!(!game.command(Command::SetBet(game.bet)));
        game.hand_count = 6;
        assert!(game.validate().is_err());
        game.hand_count = 5;
        game.bet = MAX_BET;
        assert!(game.bet * game.hand_count as u64 <= MAX_CREDITS);
    }
}
