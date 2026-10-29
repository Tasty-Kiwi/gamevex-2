use serde::{Deserialize, Serialize};

use super::blackjack::MAX_CREDITS;
use super::cards::{Card, Shoe};

pub const MAX_BET: u64 = MAX_CREDITS / 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BetSide {
    Player,
    Banker,
    Tie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Betting,
    Settled,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Win,
    Push,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    SetBet(u64),
    Select(BetSide),
    MaxBet,
    Deal,
    NextRound,
    Restart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baccarat {
    pub balance: u64,
    pub bet: u64,
    pub selected: Option<BetSide>,
    pub phase: Phase,
    pub player: Vec<Card>,
    pub banker: Vec<Card>,
    pub result: Option<BetSide>,
    pub outcome: Option<Outcome>,
    /// Gross amount returned to the bankroll, including returned stake.
    pub payout: u64,
    pub shoe: Shoe,
    pub seed: u64,
}

impl Baccarat {
    pub fn new(seed: u64) -> Self {
        Self {
            balance: 1000,
            bet: 10,
            selected: None,
            phase: Phase::Betting,
            player: Vec::new(),
            banker: Vec::new(),
            result: None,
            outcome: None,
            payout: 0,
            shoe: Shoe::shuffled(seed),
            seed,
        }
    }

    pub fn with_shoe(seed: u64, shoe: Shoe) -> Self {
        let mut game = Self::new(seed);
        game.shoe = shoe;
        game
    }

    pub fn legal(&self, cmd: Command) -> bool {
        match cmd {
            Command::SetBet(amount) => {
                self.phase == Phase::Betting && amount > 0 && amount <= MAX_BET
            }
            Command::Select(_) | Command::MaxBet => self.phase == Phase::Betting,
            Command::Deal => {
                self.phase == Phase::Betting
                    && self.selected.is_some()
                    && self.bet > 0
                    && self.balance >= self.bet
                    && (self.shoe.reshuffle_when_low || self.shoe.len() >= 4)
            }
            Command::NextRound => self.phase == Phase::Settled,
            Command::Restart => self.phase == Phase::GameOver,
        }
    }

    pub fn command(&mut self, cmd: Command) -> bool {
        if !self.legal(cmd) {
            return false;
        }
        let before = self.clone();
        match cmd {
            Command::SetBet(amount) => self.bet = amount,
            Command::Select(side) => self.selected = Some(side),
            Command::MaxBet => self.bet = self.balance.min(MAX_BET),
            Command::Deal => {
                let mut next = self.clone();
                if !next.deal() {
                    return false;
                }
                *self = next;
            }
            Command::NextRound => self.clear_round(),
            Command::Restart => *self = Self::new(self.seed),
        }
        *self != before
    }

    fn deal(&mut self) -> bool {
        if self.shoe.reshuffle_when_low && self.shoe.len() < 20 {
            self.shoe = Shoe::shuffled(self.seed.wrapping_add(1));
            self.seed = self.seed.wrapping_add(1);
        }

        let Some(p1) = self.shoe.draw() else {
            return false;
        };
        let Some(p2) = self.shoe.draw() else {
            return false;
        };
        let Some(b1) = self.shoe.draw() else {
            return false;
        };
        let Some(b2) = self.shoe.draw() else {
            return false;
        };
        self.player = vec![p1, p2];
        self.banker = vec![b1, b2];
        self.balance -= self.bet;

        let player_total = hand_value(&self.player);
        let banker_total = hand_value(&self.banker);
        if player_total < 8 && banker_total < 8 {
            if player_total <= 5 {
                let Some(card) = self.shoe.draw() else {
                    return false;
                };
                let third = card_value(card);
                self.player.push(card);
                if should_banker_draw(banker_total, Some(third)) {
                    let Some(card) = self.shoe.draw() else {
                        return false;
                    };
                    self.banker.push(card);
                }
            } else if should_banker_draw(banker_total, None) {
                let Some(card) = self.shoe.draw() else {
                    return false;
                };
                self.banker.push(card);
            }
        }

        let player_total = hand_value(&self.player);
        let banker_total = hand_value(&self.banker);
        let result = if player_total > banker_total {
            BetSide::Player
        } else if banker_total > player_total {
            BetSide::Banker
        } else {
            BetSide::Tie
        };
        self.result = Some(result);
        let bet_side = self.selected.expect("legal Deal requires a selection");
        let (outcome, payout) = match (bet_side, result) {
            (side, winner) if side == winner => {
                let payout = match result {
                    BetSide::Player => self.bet * 2,
                    BetSide::Banker => (self.bet * 195 + 50) / 100,
                    BetSide::Tie => self.bet * 9,
                };
                (Outcome::Win, payout)
            }
            (BetSide::Player | BetSide::Banker, BetSide::Tie) => (Outcome::Push, self.bet),
            _ => (Outcome::Loss, 0),
        };
        self.balance = self.balance.saturating_add(payout).min(MAX_CREDITS);
        self.outcome = Some(outcome);
        self.payout = payout;
        self.phase = if self.balance == 0 {
            Phase::GameOver
        } else {
            Phase::Settled
        };
        true
    }

    fn clear_round(&mut self) {
        self.selected = None;
        self.player.clear();
        self.banker.clear();
        self.result = None;
        self.outcome = None;
        self.payout = 0;
        self.phase = if self.balance == 0 {
            Phase::GameOver
        } else {
            Phase::Betting
        };
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.balance > MAX_CREDITS || self.bet == 0 || self.bet > MAX_BET {
            return Err("invalid credit amount".into());
        }
        let dealt_cards = self.player.len() + self.banker.len();
        if self.shoe.cards.len() > 52 || self.shoe.cards.len() + dealt_cards > 52 {
            return Err("shoe and dealt cards cannot exceed 52 cards".into());
        }
        if self.player.len() > 3
            || self.banker.len() > 3
            || self
                .player
                .iter()
                .chain(self.banker.iter())
                .chain(self.shoe.cards.iter())
                .any(|card| !(2..=14).contains(&card.rank))
        {
            return Err("invalid card tableau".into());
        }
        match self.phase {
            Phase::Betting => {
                if self.balance == 0
                    || !self.player.is_empty()
                    || !self.banker.is_empty()
                    || self.result.is_some()
                    || self.outcome.is_some()
                    || self.payout != 0
                {
                    return Err("invalid betting state".into());
                }
            }
            Phase::Settled | Phase::GameOver => {
                if !(2..=3).contains(&self.player.len())
                    || !(2..=3).contains(&self.banker.len())
                    || self.result.is_none()
                    || self.outcome.is_none()
                    || self.selected.is_none()
                    || self.payout > MAX_CREDITS
                    || (self.phase == Phase::GameOver) != (self.balance == 0)
                {
                    return Err("invalid settled state".into());
                }
                if self.result != Some(compare_hands(&self.player, &self.banker)) {
                    return Err("result does not match the hands".into());
                }
                let initial_player = hand_value(&self.player[..2]);
                let initial_banker = hand_value(&self.banker[..2]);
                if initial_player >= 8 || initial_banker >= 8 {
                    if self.player.len() != 2 || self.banker.len() != 2 {
                        return Err("natural hands cannot draw third cards".into());
                    }
                } else {
                    let player_should_draw = initial_player <= 5;
                    if (self.player.len() == 3) != player_should_draw {
                        return Err("player third card does not match the tableau".into());
                    }
                    let banker_third = self.player.get(2).map(|card| card_value(*card));
                    let banker_should_draw = should_banker_draw(initial_banker, banker_third);
                    if (self.banker.len() == 3) != banker_should_draw {
                        return Err("banker third card does not match the tableau".into());
                    }
                }
                let expected = match (self.selected.unwrap(), self.result.unwrap()) {
                    (side, result) if side == result => Outcome::Win,
                    (BetSide::Player | BetSide::Banker, BetSide::Tie) => Outcome::Push,
                    _ => Outcome::Loss,
                };
                if self.outcome != Some(expected)
                    || (expected == Outcome::Loss && self.payout != 0)
                    || (expected != Outcome::Loss && self.payout == 0)
                {
                    return Err("outcome or payout is inconsistent".into());
                }
                let expected_payout = match expected {
                    Outcome::Loss => 0,
                    Outcome::Push => self.bet,
                    Outcome::Win => match self.result.unwrap() {
                        BetSide::Player => self.bet * 2,
                        BetSide::Banker => (self.bet * 195 + 50) / 100,
                        BetSide::Tie => self.bet * 9,
                    },
                };
                if self.payout != expected_payout {
                    return Err("payout does not match the bet".into());
                }
            }
        }
        Ok(())
    }
}

pub const fn card_value(card: Card) -> u8 {
    match card.rank {
        2..=9 => card.rank,
        14 => 1,
        _ => 0,
    }
}

pub fn hand_value(cards: &[Card]) -> u8 {
    cards
        .iter()
        .fold(0u8, |total, card| (total + card_value(*card)) % 10)
}

pub fn should_banker_draw(total: u8, player_third: Option<u8>) -> bool {
    match player_third {
        None => total <= 5,
        Some(third) => match total {
            0..=2 => true,
            3 => third != 8,
            4 => (2..=7).contains(&third),
            5 => (4..=7).contains(&third),
            6 => (6..=7).contains(&third),
            _ => false,
        },
    }
}

fn compare_hands(player: &[Card], banker: &[Card]) -> BetSide {
    match hand_value(player).cmp(&hand_value(banker)) {
        std::cmp::Ordering::Greater => BetSide::Player,
        std::cmp::Ordering::Less => BetSide::Banker,
        std::cmp::Ordering::Equal => BetSide::Tie,
    }
}

#[cfg(test)]
mod tests {
    use super::super::cards::Suit;
    use super::*;

    fn c(rank: u8) -> Card {
        Card::new(rank, Suit::Spades, true)
    }
    fn game(cards: &[u8], selected: BetSide, bet: u64) -> Baccarat {
        let mut game =
            Baccarat::with_shoe(7, Shoe::from_cards(cards.iter().copied().map(c).collect()));
        game.bet = bet;
        game.selected = Some(selected);
        game
    }
    fn deal(cards: &[u8], selected: BetSide, bet: u64) -> Baccarat {
        let mut g = game(cards, selected, bet);
        assert!(g.command(Command::Deal));
        g
    }

    #[test]
    fn original_typescript_value_and_tableau_fixtures_match() {
        let oracle: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/baccarat-oracle.json"))
                .unwrap();
        for case in oracle["values"].as_array().unwrap() {
            let cards: Vec<_> = case["ranks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| c(r.as_u64().unwrap() as u8))
                .collect();
            assert_eq!(hand_value(&cards) as u64, case["total"].as_u64().unwrap());
        }
        for case in oracle["tableau"].as_array().unwrap() {
            assert_eq!(
                should_banker_draw(
                    case["banker"].as_u64().unwrap() as u8,
                    case["third"].as_u64().map(|v| v as u8)
                ),
                case["draw"].as_bool().unwrap()
            );
        }
    }

    #[test]
    fn values_modulo_ten_and_face_cards_are_zero() {
        assert_eq!(card_value(c(14)), 1);
        assert_eq!(card_value(c(10)), 0);
        assert_eq!(hand_value(&[c(14), c(8), c(9)]), 8);
        assert_eq!(hand_value(&[c(8), c(9)]), 7);
    }

    #[test]
    fn banker_tableau_matches_every_total_and_third_card() {
        let expected = [
            [true; 10],
            [true; 10],
            [true; 10],
            [true, true, true, true, true, true, true, true, false, true],
            [
                false, false, true, true, true, true, true, true, false, false,
            ],
            [
                false, false, false, false, true, true, true, true, false, false,
            ],
            [
                false, false, false, false, false, false, true, true, false, false,
            ],
            [false; 10],
            [false; 10],
            [false; 10],
        ];
        for total in 0..=9 {
            for third in 0..=9 {
                assert_eq!(
                    should_banker_draw(total, Some(third)),
                    expected[total as usize][third as usize],
                    "total={total}, third={third}"
                );
            }
        }
        assert!(should_banker_draw(5, None));
        assert!(!should_banker_draw(6, None));
    }

    #[test]
    fn naturals_end_the_round_without_third_cards() {
        let g = deal(&[4, 4, 4, 4], BetSide::Player, 10);
        assert_eq!(g.player.len(), 2);
        assert_eq!(g.banker.len(), 2);
        let g = deal(&[4, 5, 3, 5], BetSide::Player, 10);
        assert_eq!(g.player.len(), 2);
        assert_eq!(g.banker.len(), 2);
    }

    #[test]
    fn all_banker_draw_tableau_outcomes_resolve_to_correct_hand_lengths() {
        for total in 0..=7 {
            for third in 0..=9 {
                let banker_cards = [total, 0].map(|v| match v {
                    0 => 10,
                    1 => 14,
                    n => n,
                });
                // Initial player hand is 0, then its third card is the requested value.
                let third_rank = match third {
                    0 => 10,
                    1 => 14,
                    n => n,
                };
                let p = [8, 2, third_rank];
                let cards = [p[0], p[1], banker_cards[0], banker_cards[1], p[2], 2];
                let g = deal(&cards, BetSide::Player, 1);
                let should_draw = should_banker_draw(total, Some(third));
                assert_eq!(
                    g.banker.len(),
                    if should_draw { 3 } else { 2 },
                    "banker={total}, third={third}"
                );
            }
        }
    }

    #[test]
    fn wager_resolution_covers_player_banker_tie_and_push() {
        let p = deal(&[5, 2, 4, 2, 10], BetSide::Player, 10);
        assert_eq!(
            (p.result, p.outcome, p.payout, p.balance),
            (Some(BetSide::Player), Some(Outcome::Win), 20, 1010)
        );
        let b = deal(&[4, 2, 5, 2, 10], BetSide::Banker, 10);
        assert_eq!(
            (b.result, b.outcome, b.payout),
            (Some(BetSide::Banker), Some(Outcome::Win), 20)
        );
        let tie = deal(&[4, 4, 4, 4], BetSide::Tie, 10);
        assert_eq!((tie.outcome, tie.payout), (Some(Outcome::Win), 90));
        let push = deal(&[4, 4, 4, 4], BetSide::Banker, 10);
        assert_eq!(
            (push.outcome, push.payout, push.balance),
            (Some(Outcome::Push), 10, 1000)
        );
        let push = deal(&[4, 4, 4, 4], BetSide::Player, 10);
        assert_eq!((push.outcome, push.payout), (Some(Outcome::Push), 10));
        let loss = deal(&[5, 2, 4, 2, 10], BetSide::Banker, 10);
        assert_eq!(
            (loss.outcome, loss.payout, loss.balance),
            (Some(Outcome::Loss), 0, 990)
        );
        let loss = deal(&[5, 2, 4, 2, 10], BetSide::Tie, 10);
        assert_eq!((loss.outcome, loss.payout), (Some(Outcome::Loss), 0));
    }

    #[test]
    fn banker_commission_rounds_half_up() {
        for (bet, expected) in [(1, 2), (10, 20), (20, 39), (50, 98)] {
            let g = deal(&[4, 2, 5, 2, 10], BetSide::Banker, bet);
            assert_eq!(g.payout, expected);
        }
    }

    #[test]
    fn max_bet_low_bankroll_bankruptcy_restart_and_next_round() {
        let mut g = Baccarat::new(9);
        g.balance = 3;
        assert!(g.command(Command::MaxBet));
        assert_eq!(g.bet, 3);
        g.selected = Some(BetSide::Player);
        // The selected low-stakes fixed fixture loses, ending the session.
        g.shoe = Shoe::from_cards(vec![c(2), c(2), c(4), c(4)]);
        assert!(g.command(Command::Deal));
        assert_eq!(g.phase, Phase::GameOver);
        assert!(!g.legal(Command::NextRound));
        assert!(g.command(Command::Restart));
        assert_eq!(
            (g.balance, g.bet, g.selected, g.phase),
            (1000, 10, None, Phase::Betting)
        );

        let mut g = deal(&[5, 2, 4, 2, 10], BetSide::Player, 10);
        assert!(g.command(Command::NextRound));
        assert_eq!(
            (g.phase, g.bet, g.selected, g.player.len(), g.payout),
            (Phase::Betting, 10, None, 0, 0)
        );
    }

    #[test]
    fn short_fixed_shoes_fail_atomically_even_if_third_card_is_needed() {
        for cards in [vec![c(4); 3], vec![c(2), c(2), c(2), c(2), c(2)]] {
            let mut g = Baccarat::with_shoe(5, Shoe::from_cards(cards));
            g.selected = Some(BetSide::Player);
            let before = g.clone();
            assert!(!g.command(Command::Deal));
            assert_eq!(g, before);
        }
        let natural = deal(&[4, 4, 4, 4], BetSide::Tie, 10);
        assert_eq!(natural.player.len(), 2);
    }

    #[test]
    fn deterministic_shoe_save_round_trip_and_validation() {
        let mut a = Baccarat::new(123);
        a.selected = Some(BetSide::Player);
        assert!(a.command(Command::Deal));
        assert!(a.validate().is_ok());
        let encoded = serde_json::to_string(&a).unwrap();
        let restored: Baccarat = serde_json::from_str(&encoded).unwrap();
        assert_eq!(a, restored);
        let mut replay = Baccarat::new(123);
        replay.selected = Some(BetSide::Player);
        replay.command(Command::Deal);
        assert_eq!(a.player, replay.player);
        assert_eq!(a.banker, replay.banker);
        assert_eq!(a.result, replay.result);

        let mut invalid = restored;
        invalid.player[0].rank = 1;
        assert!(invalid.validate().is_err());
        let mut invalid = a;
        invalid.payout = 1;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn validation_rejects_tableaux_that_violate_naturals_or_draw_rules() {
        let natural = deal(&[4, 4, 4, 4], BetSide::Tie, 10);
        let mut extra_natural_card = natural.clone();
        extra_natural_card.player.push(c(2));
        assert!(extra_natural_card.validate().is_err());

        let mut missing_player_third = deal(&[2, 2, 2, 2, 2, 2], BetSide::Player, 10);
        missing_player_third.player.pop();
        assert!(missing_player_third.validate().is_err());

        let mut extra_banker_card = deal(&[3, 3, 3, 3], BetSide::Player, 10);
        extra_banker_card.banker.push(c(2));
        assert!(extra_banker_card.validate().is_err());
    }

    #[test]
    fn validation_rejects_fixed_shoes_over_a_deck() {
        let oversized = Shoe::from_cards(vec![c(2); 53]);
        let game = Baccarat::with_shoe(5, oversized);
        assert!(game.validate().is_err());
    }
}
