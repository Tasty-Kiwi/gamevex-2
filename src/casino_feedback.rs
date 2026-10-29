//! Transient casino presentation events. Rules and persisted credits stay authoritative.
use crate::casino_audio::SoundCue;
use gamevex_core::{baccarat as bc, blackjack as bj};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundKind {
    Win,
    Natural,
    BigWin,
    Push,
    Loss,
    GameOver,
}

#[derive(Clone, Debug)]
pub struct RoundResult {
    pub kind: RoundKind,
    pub title: String,
    pub net: i64,
}

#[derive(Default)]
pub struct CasinoFeedback {
    pub result: Option<RoundResult>,
    pub result_age: f32,
    pub action: Option<&'static str>,
    pub action_age: f32,
    pending: Option<RoundResult>,
    pending_bust: bool,
    player_counts: Vec<usize>,
    dealer_count: usize,
    hidden: bool,
    sounds: Vec<SoundCue>,
    opening_balance: Option<u64>,
    tracking: bool,
}

impl CasinoFeedback {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn tick(&mut self, dt: f32) {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.1)
        } else {
            0.0
        };
        self.result_age = (self.result_age + dt).min(3.0);
        self.action_age = (self.action_age + dt).min(2.0);
    }
    fn cue(&mut self, cue: SoundCue) {
        if self.sounds.len() < 8 {
            self.sounds.push(cue);
        }
    }
    fn action(&mut self, label: &'static str, cue: SoundCue) {
        self.action = Some(label);
        self.action_age = 0.0;
        self.cue(cue);
    }
    pub fn take_sounds(&mut self) -> Vec<SoundCue> {
        std::mem::take(&mut self.sounds)
    }

    pub fn blackjack(
        &mut self,
        command: bj::Command,
        before: &bj::Blackjack,
        after: &bj::Blackjack,
    ) {
        use bj::Command::*;
        if matches!(command, Deal | NextRound | Restart) {
            self.clear();
        }
        if command == Deal {
            self.opening_balance = Some(before.balance);
        }
        self.tracking = true;
        if self.opening_balance.is_none() && !before.hands.is_empty() {
            self.opening_balance = Some(
                before.balance
                    + before.hands.iter().map(|h| h.bet).sum::<u64>()
                    + before.insurance_bet
                    - before
                        .hands
                        .iter()
                        .filter(|h| h.surrendered)
                        .map(|h| h.bet / 2)
                        .sum::<u64>(),
            );
        }
        self.player_counts = if command == Deal {
            vec![]
        } else if command == Split {
            vec![1, 1]
        } else {
            before.hands.iter().map(|h| h.cards.len()).collect()
        };
        self.dealer_count = if command == Deal {
            0
        } else {
            before.dealer.len()
        };
        self.hidden =
            matches!(before.phase, bj::Phase::Insurance | bj::Phase::PlayerTurn) || command == Deal;
        match command {
            SetBet(_) | SetHandCount(_) => self.action("WAGER", SoundCue::Bet),
            Deal => self.action("DEAL", SoundCue::Deal),
            Hit => self.action("HIT", SoundCue::Hit),
            Stand => self.action("STAND", SoundCue::Stand),
            Double => self.action("DOUBLE DOWN", SoundCue::Double),
            Split => self.action("SPLIT", SoundCue::Split),
            Surrender => self.action("SURRENDER", SoundCue::Surrender),
            Insure => self.action("INSURANCE", SoundCue::Insurance),
            DeclineInsurance => self.action("NO INSURANCE", SoundCue::Stand),
            NextRound | Restart => self.cue(SoundCue::Bet),
            Abandon => {}
        }
        self.pending_bust = matches!(command, Hit | Double)
            && after
                .hands
                .get(before.active_hand)
                .is_some_and(|h| bj::hand_value(&h.cards) > 21);
        if matches!(after.phase, bj::Phase::Settled | bj::Phase::GameOver)
            && !matches!(before.phase, bj::Phase::Settled | bj::Phase::GameOver)
        {
            let result = self.opening_balance.map_or_else(
                || blackjack_result(after),
                |opening| blackjack_result_with_net(after, after.balance as i64 - opening as i64),
            );
            self.pending = Some(result);
        }
    }

    pub fn baccarat(&mut self, command: bc::Command, before: &bc::Baccarat, after: &bc::Baccarat) {
        use bc::Command::*;
        if matches!(command, Deal | NextRound | Restart) {
            self.clear();
        }
        self.tracking = true;
        self.player_counts = if command == Deal {
            vec![0]
        } else {
            vec![before.player.len()]
        };
        self.dealer_count = if command == Deal {
            0
        } else {
            before.banker.len()
        };
        match command {
            SetBet(_) | MaxBet => self.action("WAGER", SoundCue::Bet),
            Select(side) => self.action(
                match side {
                    bc::BetSide::Player => "PLAYER",
                    bc::BetSide::Banker => "BANKER",
                    bc::BetSide::Tie => "TIE",
                },
                SoundCue::Bet,
            ),
            Deal => {
                self.action("DEAL", SoundCue::Deal);
                self.pending = Some(baccarat_result(after));
            }
            NextRound | Restart => self.cue(SoundCue::Bet),
        }
    }

    pub fn observe(
        &mut self,
        player_counts: Vec<usize>,
        dealer_count: usize,
        hidden: bool,
        finished: bool,
    ) {
        if !self.tracking {
            return;
        }
        let arrived = dealer_count > self.dealer_count
            || player_counts
                .iter()
                .enumerate()
                .any(|(i, n)| *n > self.player_counts.get(i).copied().unwrap_or(0));
        // One cue per update avoids a burst when reduced motion reveals all cards at once.
        if arrived {
            self.cue(SoundCue::Card);
        }
        if self.hidden && !hidden {
            self.cue(SoundCue::Reveal);
        }
        self.player_counts = player_counts;
        self.dealer_count = dealer_count;
        self.hidden = hidden;
        if finished {
            let bust = self.pending_bust;
            if bust {
                self.action("BUST", SoundCue::Bust);
                self.pending_bust = false;
            }
            if let Some(result) = self.pending.take() {
                let cue = match result.kind {
                    RoundKind::Win => SoundCue::Win,
                    RoundKind::Natural | RoundKind::BigWin => SoundCue::Natural,
                    RoundKind::Push => SoundCue::Push,
                    RoundKind::Loss => SoundCue::Loss,
                    RoundKind::GameOver => SoundCue::GameOver,
                };
                if !(bust && result.kind == RoundKind::Loss) {
                    self.cue(cue);
                }
                self.result = Some(result);
                self.result_age = 0.0;
            }
        }
    }
}

pub fn blackjack_result(game: &bj::Blackjack) -> RoundResult {
    let stakes: u64 = game.hands.iter().map(|h| h.bet).sum::<u64>() + game.insurance_bet;
    let refunds: u64 = game
        .hands
        .iter()
        .filter(|h| h.surrendered)
        .map(|h| h.bet / 2)
        .sum();
    let mut returns_halves = 0;
    if game.message.as_deref() != Some("ROUND ABANDONED") {
        if bj::hand_value(&game.dealer) == 21 && game.dealer.len() == 2 {
            returns_halves += game.insurance_bet * 6;
        }
        for h in &game.hands {
            returns_halves += match h.outcome {
                Some(bj::HandOutcome::Blackjack) => h.bet * 5,
                Some(bj::HandOutcome::Win) => h.bet * 4,
                Some(bj::HandOutcome::Push) => h.bet * 2,
                _ => 0,
            };
        }
    }
    let net = (returns_halves.div_ceil(2) + refunds) as i64 - stakes as i64;
    blackjack_result_with_net(game, net)
}

fn blackjack_result_with_net(game: &bj::Blackjack, net: i64) -> RoundResult {
    let natural = game
        .hands
        .iter()
        .any(|h| h.outcome == Some(bj::HandOutcome::Blackjack));
    let kind = if game.balance == 0 {
        RoundKind::GameOver
    } else if net > 0 && natural {
        RoundKind::Natural
    } else if net > 0 {
        RoundKind::Win
    } else if net == 0 {
        RoundKind::Push
    } else {
        RoundKind::Loss
    };
    let title = match kind {
        RoundKind::Win => "ROUND WON",
        RoundKind::Natural => "NATURAL BLACKJACK",
        RoundKind::BigWin => "BIG WIN",
        RoundKind::Push => "ROUND EVEN",
        RoundKind::Loss => "ROUND LOST",
        RoundKind::GameOver => "BANKROLL EMPTY",
    }
    .to_owned();
    RoundResult { kind, title, net }
}

pub fn baccarat_result(game: &bc::Baccarat) -> RoundResult {
    let kind = if game.balance == 0 {
        RoundKind::GameOver
    } else {
        match game.outcome {
            Some(bc::Outcome::Win) if game.selected == Some(bc::BetSide::Tie) => RoundKind::BigWin,
            Some(bc::Outcome::Win) => RoundKind::Win,
            Some(bc::Outcome::Push) => RoundKind::Push,
            _ => RoundKind::Loss,
        }
    };
    let title = if kind == RoundKind::GameOver {
        "BANKROLL EMPTY"
    } else if game.outcome == Some(bc::Outcome::Push) {
        "TIE / STAKE RETURNED"
    } else {
        match (game.result, game.outcome) {
            (Some(bc::BetSide::Player), Some(bc::Outcome::Win)) => "PLAYER WIN",
            (Some(bc::BetSide::Banker), Some(bc::Outcome::Win)) => "BANKER WIN",
            (Some(bc::BetSide::Tie), Some(bc::Outcome::Win)) => "TIE WIN / 8:1",
            (Some(bc::BetSide::Player), _) => "PLAYER WINS / BET LOST",
            (Some(bc::BetSide::Banker), _) => "BANKER WINS / BET LOST",
            _ => "TIE / BET LOST",
        }
    }
    .to_owned();
    RoundResult {
        kind,
        title,
        net: game.payout as i64 - game.bet as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamevex_core::cards::{Card, Shoe, Suit};
    fn shoe(ranks: &[u8]) -> Shoe {
        Shoe::from_cards(
            ranks
                .iter()
                .map(|rank| Card::new(*rank, Suit::Spades, true))
                .collect(),
        )
    }

    #[test]
    fn natural_fanfare_waits_for_visible_settlement_and_only_plays_once() {
        let before = bj::Blackjack::with_shoe(1, shoe(&[10, 7, 14, 13]));
        let mut after = before.clone();
        after.command(bj::Command::Deal);
        let mut feedback = CasinoFeedback::default();
        feedback.blackjack(bj::Command::Deal, &before, &after);
        assert_eq!(feedback.take_sounds(), vec![SoundCue::Deal]);
        feedback.observe(vec![1], 1, true, false);
        assert!(feedback.result.is_none());
        assert_eq!(feedback.take_sounds(), vec![SoundCue::Card]);
        feedback.observe(vec![2], 2, false, false);
        assert_eq!(
            feedback.take_sounds(),
            vec![SoundCue::Card, SoundCue::Reveal]
        );
        feedback.observe(vec![2], 2, false, true);
        assert_eq!(feedback.take_sounds(), vec![SoundCue::Natural]);
        assert_eq!(feedback.result.as_ref().unwrap().net, 15);
        feedback.observe(vec![2], 2, false, true);
        assert!(feedback.take_sounds().is_empty());
    }

    #[test]
    fn restored_rounds_are_silent_and_baccarat_push_uses_its_own_cue() {
        let mut feedback = CasinoFeedback::default();
        feedback.observe(vec![2], 2, false, true);
        assert!(feedback.take_sounds().is_empty());
        let mut before = bc::Baccarat::with_shoe(1, shoe(&[4, 4, 4, 4]));
        before.command(bc::Command::Select(bc::BetSide::Player));
        let mut after = before.clone();
        after.command(bc::Command::Deal);
        feedback.baccarat(bc::Command::Deal, &before, &after);
        feedback.take_sounds();
        feedback.observe(vec![2], 2, false, true);
        assert_eq!(feedback.take_sounds(), vec![SoundCue::Card, SoundCue::Push]);
        let result = feedback.result.as_ref().unwrap();
        assert_eq!((result.kind, result.net), (RoundKind::Push, 0));
        feedback.clear();
        assert!(feedback.result.is_none());
        assert!(feedback.take_sounds().is_empty());
    }

    #[test]
    fn bust_is_deferred_until_the_draw_finishes_and_does_not_double_the_loss_sound() {
        let mut before = bj::Blackjack::with_shoe(1, shoe(&[10, 7, 10, 6, 10]));
        before.command(bj::Command::Deal);
        let mut after = before.clone();
        after.command(bj::Command::Hit);
        let mut feedback = CasinoFeedback::default();
        feedback.blackjack(bj::Command::Hit, &before, &after);
        assert_eq!(feedback.action, Some("HIT"));
        assert_eq!(feedback.take_sounds(), vec![SoundCue::Hit]);
        feedback.observe(vec![3], 2, true, false);
        assert!(feedback.result.is_none());
        feedback.take_sounds();
        feedback.observe(vec![3], 2, false, true);
        assert_eq!(
            feedback.take_sounds(),
            vec![SoundCue::Reveal, SoundCue::Bust]
        );
        assert_eq!(feedback.action, Some("BUST"));
        assert_eq!(feedback.result.unwrap().net, -10);
    }

    #[test]
    fn baccarat_tie_win_and_bankruptcy_have_distinct_results() {
        let mut game = bc::Baccarat::with_shoe(1, shoe(&[4, 4, 4, 4]));
        game.selected = Some(bc::BetSide::Tie);
        game.command(bc::Command::Deal);
        let result = baccarat_result(&game);
        assert_eq!((result.kind, result.net), (RoundKind::BigWin, 80));
        let mut before = bc::Baccarat::with_shoe(1, shoe(&[4, 4, 4, 5]));
        before.selected = Some(bc::BetSide::Player);
        before.bet = 1000;
        let mut after = before.clone();
        after.command(bc::Command::Deal);
        let mut feedback = CasinoFeedback::default();
        feedback.baccarat(bc::Command::Deal, &before, &after);
        feedback.take_sounds();
        feedback.observe(vec![2], 2, false, true);
        assert_eq!(
            feedback.take_sounds(),
            vec![SoundCue::Card, SoundCue::GameOver]
        );
        assert_eq!(feedback.result.unwrap().kind, RoundKind::GameOver);
    }

    #[test]
    fn round_delta_accounts_for_combined_rounding_surrender_and_insurance() {
        let mut game = bj::Blackjack::with_shoe(1, shoe(&[5, 6, 14, 13, 14, 13, 6]));
        game.balance = 3;
        game.bet = 1;
        game.hand_count = 2;
        game.command(bj::Command::Deal);
        assert_eq!(blackjack_result(&game).net, 3);
        let mut game = bj::Blackjack::with_shoe(1, shoe(&[10, 7, 10, 8]));
        game.bet = 5;
        game.command(bj::Command::Deal);
        game.command(bj::Command::Surrender);
        assert_eq!(blackjack_result(&game).net, -3);
        let mut game = bj::Blackjack::with_shoe(1, shoe(&[10, 14, 10, 9]));
        game.command(bj::Command::Deal);
        game.command(bj::Command::Insure);
        game.command(bj::Command::Stand);
        assert_eq!(blackjack_result(&game).net, 0);
    }
}
