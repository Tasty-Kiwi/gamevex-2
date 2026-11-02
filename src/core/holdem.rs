use crate::cards::{Card, Suit};
use crate::poker::{Evaluation, evaluate};
use serde::Serialize;

const SMALL_BLIND: u32 = 5;
const BIG_BLIND: u32 = 10;
pub const MAX_BUY_IN: u32 = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Fold,
    Check,
    Call,
    Bet(u32),
    Raise(u32),
    AllIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BetBounds {
    pub min_total: u32,
    pub max_total: u32,
    pub suggested_total: u32,
    pub is_raise: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub stack: u32,
    pub hole: Vec<Card>,
    pub folded: bool,
    pub all_in: bool,
    pub street_committed: u32,
    pub committed: u32,
    pub can_raise: bool,
    pub last_action: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    pub seats: Vec<Seat>,
    pub board: Vec<Card>,
    pub deck: Vec<Card>,
    pub burned: Vec<Card>,
    pub buy_in: u32,
    pub dealer: usize,
    pub small_blind: usize,
    pub big_blind: usize,
    pub turn: Option<usize>,
    pub queue: Vec<usize>,
    pub current_bet: u32,
    pub last_full_raise: u32,
    pub hand_number: u32,
    pub payouts: Vec<u32>,
    pub winners: Vec<bool>,
    pub evaluations: Vec<Option<Evaluation>>,
    pub winner_text: String,
    pub log: Vec<String>,
    pub rng_state: u64,
    pub phase: Phase,
}

impl Game {
    fn add_log(&mut self, entry: String) {
        self.log.insert(0, entry);
        self.log.truncate(8);
    }
    pub fn idle(opponents: usize) -> Self {
        let n = opponents.clamp(1, 4) + 1;
        Self {
            seats: (0..n)
                .map(|_| Seat {
                    stack: 0,
                    hole: vec![],
                    folded: false,
                    all_in: true,
                    street_committed: 0,
                    committed: 0,
                    can_raise: false,
                    last_action: String::new(),
                })
                .collect(),
            board: vec![],
            deck: vec![],
            burned: vec![],
            buy_in: 0,
            dealer: n - 1,
            small_blind: if n == 2 { n - 1 } else { 0 },
            big_blind: if n == 2 { 0 } else { 1 },
            turn: None,
            queue: vec![],
            current_bet: 0,
            last_full_raise: BIG_BLIND,
            hand_number: 0,
            payouts: vec![0; n],
            winners: vec![false; n],
            evaluations: vec![None; n],
            winner_text: String::new(),
            log: vec!["CHOOSE YOUR BUY-IN".into()],
            rng_state: 1,
            phase: Phase::Idle,
        }
    }

    pub fn start(&mut self, buy_in: u32, seed: u64) -> bool {
        if !matches!(self.phase, Phase::Idle | Phase::Showdown | Phase::Complete)
            || buy_in == 0
            || buy_in > MAX_BUY_IN
        {
            return false;
        }
        let n = self.seats.len();
        self.dealer = if self.hand_number == 0 {
            0
        } else {
            (self.dealer + 1) % n
        };
        self.buy_in = buy_in;
        self.hand_number += 1;
        self.phase = Phase::Preflop;
        self.board.clear();
        self.burned.clear();
        self.payouts = vec![0; n];
        self.winners = vec![false; n];
        self.evaluations = vec![None; n];
        self.winner_text.clear();
        self.log.clear();
        self.queue.clear();
        self.current_bet = 0;
        self.last_full_raise = BIG_BLIND;
        self.seats = (0..n)
            .map(|_| Seat {
                stack: buy_in,
                hole: vec![],
                folded: false,
                all_in: false,
                street_committed: 0,
                committed: 0,
                can_raise: true,
                last_action: String::new(),
            })
            .collect();
        self.rng_state = seed.max(1);
        self.deck = Vec::with_capacity(52);
        for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
            for rank in 2..=14 {
                self.deck.push(Card::new(rank, suit, true));
            }
        }
        for i in (1..self.deck.len()).rev() {
            let j = (self.next_random() % ((i + 1) as u64)) as usize;
            self.deck.swap(i, j);
        }
        self.small_blind = if n == 2 {
            self.dealer
        } else {
            (self.dealer + 1) % n
        };
        self.big_blind = if n == 2 {
            (self.dealer + 1) % n
        } else {
            (self.dealer + 2) % n
        };
        for _ in 0..2 {
            for offset in 1..=n {
                let i = (self.dealer + offset) % n;
                let c = self.deck.pop().unwrap();
                self.seats[i].hole.push(c);
            }
        }
        self.commit(self.small_blind, SMALL_BLIND.min(buy_in));
        self.commit(self.big_blind, BIG_BLIND.min(buy_in));
        self.current_bet = self.seats[self.small_blind]
            .street_committed
            .max(self.seats[self.big_blind].street_committed);
        let first = if n == 2 {
            self.dealer
        } else {
            (self.big_blind + 1) % n
        };
        self.queue = (0..n)
            .map(|o| (first + o) % n)
            .filter(|&i| self.can_act(i))
            .collect();
        self.add_log(format!(
            "BLINDS ${} / ${}",
            self.seats[self.small_blind].street_committed,
            self.seats[self.big_blind].street_committed
        ));
        self.resolve();
        true
    }

    pub fn terminal(&self) -> bool {
        matches!(self.phase, Phase::Idle | Phase::Showdown | Phase::Complete)
    }
    pub fn pot(&self) -> u32 {
        self.seats.iter().map(|s| s.committed).sum()
    }
    pub fn to_call(&self, seat: usize) -> u32 {
        self.seats
            .get(seat)
            .map_or(0, |s| self.current_bet.saturating_sub(s.street_committed))
    }
    fn can_act(&self, i: usize) -> bool {
        self.seats
            .get(i)
            .is_some_and(|s| !s.folded && !s.all_in && s.stack > 0)
    }
    fn commit(&mut self, i: usize, amount: u32) -> u32 {
        let s = &mut self.seats[i];
        let paid = amount.min(s.stack);
        s.stack -= paid;
        s.street_committed += paid;
        s.committed += paid;
        s.all_in = s.stack == 0;
        if s.all_in {
            s.can_raise = false;
        }
        paid
    }
    fn next_random(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn legal_actions(&self, seat: usize) -> Vec<Action> {
        if self.turn != Some(seat) || !self.can_act(seat) {
            return vec![];
        }
        let call = self.to_call(seat);
        let s = &self.seats[seat];
        let mut a = vec![];
        if call > 0 {
            a.push(Action::Fold);
            a.push(Action::Call)
        } else {
            a.push(Action::Check)
        }
        if self.bet_bounds(seat).is_some() {
            a.push(if self.current_bet == 0 {
                Action::Bet(0)
            } else {
                Action::Raise(0)
            })
        }
        if s.stack > 0 && (s.can_raise || s.street_committed + s.stack <= self.current_bet) {
            a.push(Action::AllIn)
        }
        a
    }
    pub fn bet_bounds(&self, seat: usize) -> Option<BetBounds> {
        let s = self.seats.get(seat)?;
        if !self.can_act(seat) || !s.can_raise {
            return None;
        }
        let max = s.street_committed + s.stack;
        let (required_min, is_raise) = if self.current_bet == 0 {
            (BIG_BLIND.max(s.street_committed + 1), false)
        } else {
            (self.current_bet.saturating_add(self.last_full_raise), true)
        };
        if max < required_min || is_raise && max <= self.current_bet {
            return None;
        }
        let min = required_min;
        let suggested = if self.current_bet == 0 {
            (self.pot() / 2).max(min).min(max)
        } else {
            (self.current_bet + (self.last_full_raise.max(self.pot() / 2)))
                .max(min)
                .min(max)
        };
        Some(BetBounds {
            min_total: min,
            max_total: max,
            suggested_total: suggested,
            is_raise,
        })
    }
    pub fn apply(&mut self, action: Action) -> bool {
        let Some(i) = self.turn else { return false };
        if !self
            .legal_actions(i)
            .iter()
            .any(|a| std::mem::discriminant(a) == std::mem::discriminant(&action))
        {
            return false;
        }
        match action {
            Action::Bet(total) | Action::Raise(total) => {
                let Some(bounds) = self.bet_bounds(i) else {
                    return false;
                };
                if matches!(action, Action::Bet(_)) == bounds.is_raise
                    || total < bounds.min_total
                    || total > bounds.max_total
                {
                    return false;
                }
            }
            Action::AllIn => {
                let total = self.seats[i].street_committed + self.seats[i].stack;
                if total > self.current_bet && !self.seats[i].can_raise {
                    return false;
                }
            }
            _ => {}
        }
        let label = if i == 0 {
            "YOU".to_string()
        } else {
            format!("OPPONENT {i}")
        };
        let description = match action {
            Action::Fold => format!("{label} FOLDS"),
            Action::Check => format!("{label} CHECKS"),
            Action::Call => format!(
                "{label} CALLS ${}",
                self.to_call(i).min(self.seats[i].stack)
            ),
            Action::Bet(total) => format!("{label} BETS TO ${total}"),
            Action::Raise(total) => format!("{label} RAISES TO ${total}"),
            Action::AllIn => format!("{label} SHOVES ${}", self.seats[i].stack),
        };
        self.add_log(description);
        let call = self.to_call(i);
        let prev = self.current_bet;
        self.queue.retain(|&x| x != i);
        let mut increased = false;
        let mut full = false;
        match action {
            Action::Fold => {
                self.seats[i].folded = true;
                self.seats[i].can_raise = false;
                self.seats[i].last_action = "FOLD".into()
            }
            Action::Check => {
                self.seats[i].can_raise = false;
                self.seats[i].last_action = "CHECK".into()
            }
            Action::Call => {
                let paid = self.commit(i, call);
                self.seats[i].can_raise = false;
                self.seats[i].last_action = if paid < call { "CALL ALL-IN" } else { "CALL" }.into()
            }
            Action::AllIn | Action::Bet(_) | Action::Raise(_) => {
                let requested = match action {
                    Action::AllIn => self.seats[i].street_committed + self.seats[i].stack,
                    Action::Bet(t) | Action::Raise(t) => t,
                    _ => 0,
                };
                let low = self.seats[i].street_committed + call.max(1).min(self.seats[i].stack);
                let total = requested
                    .max(low)
                    .min(self.seats[i].street_committed + self.seats[i].stack);
                self.commit(i, total - self.seats[i].street_committed);
                let delta = total.saturating_sub(prev);
                if total > prev {
                    increased = true;
                    self.current_bet = total;
                    full = if prev == 0 {
                        total >= BIG_BLIND
                    } else {
                        delta >= self.last_full_raise
                    };
                }
                if full {
                    self.last_full_raise = if prev == 0 { total } else { delta };
                    for s in &mut self.seats {
                        s.can_raise = !s.folded && !s.all_in
                    }
                    self.seats[i].can_raise = false;
                    self.queue = (1..self.seats.len())
                        .map(|o| (i + o) % self.seats.len())
                        .filter(|&x| self.can_act(x))
                        .collect();
                } else {
                    self.seats[i].can_raise = false;
                    if increased {
                        self.queue = (1..self.seats.len())
                            .map(|o| (i + o) % self.seats.len())
                            .filter(|&x| {
                                self.can_act(x) && self.seats[x].street_committed < self.current_bet
                            })
                            .collect();
                    }
                }
                self.seats[i].last_action = if action == Action::AllIn {
                    "ALL-IN"
                } else if prev == 0 {
                    "BET"
                } else {
                    "RAISE"
                }
                .into();
            }
        }
        if !increased {
            self.queue.retain(|&x| {
                !self.seats[x].folded && !self.seats[x].all_in && self.seats[x].stack > 0
            });
        }
        self.resolve();
        true
    }
    fn burn(&mut self) {
        if let Some(c) = self.deck.pop() {
            self.burned.push(c)
        }
    }
    fn deal(&mut self, n: usize) {
        for _ in 0..n {
            if let Some(c) = self.deck.pop() {
                self.board.push(c)
            }
        }
    }
    fn advance(&mut self) {
        match self.phase {
            Phase::Preflop => {
                self.burn();
                self.deal(3);
                self.phase = Phase::Flop;
                self.add_log("FLOP DEALT".into());
            }
            Phase::Flop => {
                self.burn();
                self.deal(1);
                self.phase = Phase::Turn;
                self.add_log("TURN DEALT".into());
            }
            Phase::Turn => {
                self.burn();
                self.deal(1);
                self.phase = Phase::River;
                self.add_log("RIVER DEALT".into());
            }
            _ => {
                self.settle();
                return;
            }
        }
        for s in &mut self.seats {
            s.street_committed = 0;
            s.can_raise = !s.folded && !s.all_in;
            s.last_action.clear()
        }
        self.current_bet = 0;
        self.last_full_raise = BIG_BLIND;
        let first = if self.seats.len() == 2 {
            self.big_blind
        } else {
            (self.dealer + 1) % self.seats.len()
        };
        self.queue = (0..self.seats.len())
            .map(|o| (first + o) % self.seats.len())
            .filter(|&i| self.can_act(i))
            .collect();
    }
    fn runout_to_river(&mut self) {
        while self.board.len() < 5 {
            match self.board.len() {
                0 => {
                    self.burn();
                    self.deal(3);
                    self.phase = Phase::Flop;
                    self.add_log("FLOP DEALT".into());
                }
                3 => {
                    self.burn();
                    self.deal(1);
                    self.phase = Phase::Turn;
                    self.add_log("TURN DEALT".into());
                }
                4 => {
                    self.burn();
                    self.deal(1);
                    self.phase = Phase::River;
                    self.add_log("RIVER DEALT".into());
                }
                _ => break,
            }
        }
    }
    fn resolve(&mut self) {
        loop {
            let live: Vec<_> = self
                .seats
                .iter()
                .enumerate()
                .filter(|(_, s)| !s.folded)
                .map(|(i, _)| i)
                .collect();
            if live.len() == 1 {
                self.payouts[live[0]] = self.pot();
                self.winners[live[0]] = true;
                self.seats[live[0]].stack += self.pot();
                self.clear_committed();
                self.phase = Phase::Complete;
                self.winner_text = format!(
                    "{} WINS UNCONTESTED ${}",
                    if live[0] == 0 {
                        "YOU".into()
                    } else {
                        format!("OPPONENT {}", live[0])
                    },
                    self.payouts[live[0]]
                );
                self.add_log(self.winner_text.clone());
                self.turn = None;
                self.queue.clear();
                return;
            }
            self.queue.retain(|&i| {
                !self.seats[i].folded && !self.seats[i].all_in && self.seats[i].stack > 0
            });
            let actionable = self.seats.iter().filter(|s| !s.folded && !s.all_in).count();
            let no_action_needed = actionable == 1
                && self
                    .seats
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| !s.folded && !s.all_in)
                    .all(|(_, s)| s.street_committed >= self.current_bet);
            if self.seats.iter().all(|s| s.folded || s.all_in) || no_action_needed {
                self.runout_to_river();
                self.settle();
                return;
            }
            if self.queue.is_empty() {
                if self.phase == Phase::River {
                    self.settle();
                    return;
                }
                self.advance();
                continue;
            }
            self.turn = self.queue.first().copied();
            return;
        }
    }
    fn clear_committed(&mut self) {
        for s in &mut self.seats {
            s.committed = 0;
            s.street_committed = 0;
            s.all_in = s.stack == 0 && !s.folded;
            s.can_raise = false
        }
    }
    fn settle(&mut self) {
        let n = self.seats.len();
        self.evaluations = self
            .seats
            .iter()
            .map(|s| {
                if s.folded {
                    None
                } else {
                    let mut c = s.hole.clone();
                    c.extend(self.board.iter().copied());
                    evaluate(&c)
                }
            })
            .collect();
        let mut levels: Vec<u32> = self
            .seats
            .iter()
            .map(|s| s.committed)
            .filter(|&v| v > 0)
            .collect();
        levels.sort_unstable();
        levels.dedup();
        let mut prev = 0;
        let mut payouts = vec![0; n];
        let mut winners_seen = vec![false; n];
        for level in levels {
            let contributors: Vec<_> = self
                .seats
                .iter()
                .enumerate()
                .filter(|(_, s)| s.committed >= level)
                .map(|(i, _)| i)
                .collect();
            let amount = (level - prev) * contributors.len() as u32;
            let layer_width = level - prev;
            let contributor_count = contributors.len();
            prev = level;
            let eligible: Vec<_> = contributors
                .into_iter()
                .filter(|&i| !self.seats[i].folded)
                .collect();
            if eligible.is_empty() {
                for (i, seat) in self.seats.iter().enumerate() {
                    if seat.committed >= level {
                        payouts[i] += layer_width;
                    }
                }
                continue;
            }
            if contributor_count == 1 {
                payouts[eligible[0]] += amount;
                continue;
            }
            let best = eligible
                .iter()
                .filter_map(|&i| self.evaluations[i].as_ref().map(|e| (i, &e.score)))
                .max_by(|a, b| a.1.cmp(b.1))
                .map(|x| x.1.clone())
                .unwrap();
            let mut winners: Vec<_> = eligible
                .into_iter()
                .filter(|&i| {
                    self.evaluations[i]
                        .as_ref()
                        .is_some_and(|e| e.score == best)
                })
                .collect();
            winners.sort_by_key(|&i| (i + n - self.dealer - 1) % n);
            let share = amount / winners.len() as u32;
            let odd = amount % winners.len() as u32;
            for (k, &i) in winners.iter().enumerate() {
                payouts[i] += share + u32::from(k < (odd as usize));
                winners_seen[i] = true;
            }
        }
        self.winners = winners_seen.clone();
        self.payouts = payouts.clone();
        for (i, s) in self.seats.iter_mut().enumerate() {
            s.stack += payouts[i]
        }
        self.clear_committed();
        self.phase = Phase::Showdown;
        self.turn = None;
        self.queue.clear();
        let winners: Vec<_> = winners_seen
            .iter()
            .enumerate()
            .filter(|(_, is_winner)| **is_winner)
            .map(|(i, _)| {
                if i == 0 {
                    "YOU".to_string()
                } else {
                    format!("OPPONENT {i}")
                }
            })
            .collect();
        self.winner_text = if winners.len() > 1 {
            format!("SPLIT POT: {}", winners.join(" / "))
        } else {
            format!("{} WINS", winners.first().cloned().unwrap_or_default())
        };
        self.add_log(self.winner_text.clone());
    }
    pub fn cpu_step(&mut self) -> bool {
        let Some(i) = self.turn else { return false };
        if i == 0 {
            return false;
        }
        let hole = &self.seats[i].hole;
        let cards = hole
            .iter()
            .chain(self.board.iter())
            .copied()
            .collect::<Vec<_>>();
        let mut strength = if self.phase == Phase::Preflop {
            let high = hole[0].rank.max(hole[1].rank);
            let low = hole[0].rank.min(hole[1].rank);
            let paired = high == low;
            let gap = high - low;
            let mut score = (f32::from(high) + f32::from(low)) / 28.0;
            if paired {
                score += if high >= 11 { 0.45 } else { 0.28 };
            }
            if hole[0].suit == hole[1].suit {
                score += 0.08;
            }
            if gap == 1 {
                score += 0.08;
            } else if gap == 2 {
                score += 0.03;
            }
            if high >= 13 && low >= 10 {
                score += 0.12;
            }
            score
        } else {
            let Some(evaluation) = evaluate(&cards) else {
                return self.apply(if self.to_call(i) == 0 {
                    Action::Check
                } else {
                    Action::Call
                });
            };
            let mut score = f32::from(evaluation.score[0]) / 9.0;
            if evaluation.score[0] == 2 {
                score += if evaluation.score[1] >= 12 {
                    0.12
                } else {
                    0.02
                };
            }
            if evaluation.score[0] == 3 {
                score += 0.08;
            }
            if evaluation.score[0] == 4 {
                score += 0.12;
            }
            if matches!(evaluation.score[0], 5 | 6) {
                score += 0.14;
            }
            let flush_draw = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades]
                .iter()
                .any(|suit| cards.iter().filter(|c| c.suit == *suit).count() >= 4);
            if flush_draw {
                score += 0.08;
            }
            let mut values = cards
                .iter()
                .flat_map(|c| {
                    if c.rank == 14 {
                        vec![14, 1]
                    } else {
                        vec![c.rank]
                    }
                })
                .collect::<Vec<_>>();
            values.sort_unstable();
            values.dedup();
            let mut run = 1;
            let mut longest = 1;
            for w in values.windows(2) {
                if w[1] == w[0] + 1 {
                    run += 1;
                    longest = longest.max(run)
                } else {
                    run = 1
                }
            }
            score += if longest >= 4 {
                0.12
            } else if longest == 3 {
                0.05
            } else {
                0.0
            };
            score
        };
        strength = strength.clamp(0.0, 1.0);
        let adjustment = match i {
            1 => -0.08,
            3 => 0.08,
            _ => 0.0,
        };
        strength = (strength + adjustment).clamp(0.0, 1.0);
        let call = self.to_call(i);
        let pressure = call as f32 / (self.pot() + call).max(1) as f32;
        let target = self.bet_bounds(i).map(|b| {
            let pot = self.pot();
            let desired = if strength > 0.9 {
                b.max_total
            } else if strength > 0.78 {
                self.current_bet + self.last_full_raise.max(pot)
            } else if strength > 0.66 {
                self.current_bet + self.last_full_raise.max(pot / 2)
            } else {
                b.min_total
            };
            desired.clamp(b.min_total, b.max_total)
        });
        let random = |g: &mut Self| -> bool { g.next_random().is_multiple_of(2) };
        let aggressive = |g: &Self, t: u32| {
            if g.current_bet == 0 {
                Action::Bet(t)
            } else {
                Action::Raise(t)
            }
        };
        let action = if let (true, Some(total)) = (strength > 0.82, target) {
            aggressive(self, total)
        } else if let (true, Some(total)) = (strength > 0.68 && pressure < 0.32, target) {
            if random(self) {
                aggressive(self, total)
            } else if call > 0 {
                Action::Call
            } else {
                Action::Check
            }
        } else if call == 0 {
            if let Some(total) = target.filter(|_| strength > 0.58) {
                if self.next_random() % 100 < 35 {
                    aggressive(self, total)
                } else {
                    Action::Check
                }
            } else {
                Action::Check
            }
        } else if strength > 0.48 || pressure < 0.18 {
            Action::Call
        } else if self.legal_actions(i).contains(&Action::AllIn)
            && strength > 0.4
            && self.seats[i].stack <= self.pot() / 3
        {
            Action::AllIn
        } else {
            Action::Fold
        };
        self.apply(action)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !(2..=5).contains(&self.seats.len()) {
            return Err("seat count must be 2..=5".into());
        }
        if self.payouts.len() != self.seats.len() || self.evaluations.len() != self.seats.len() {
            return Err("seat result vectors have wrong length".into());
        }
        if self.seats.iter().any(|s| {
            s.street_committed > s.committed
                || s.stack > MAX_BUY_IN.saturating_mul(5)
                || s.hole.len() > 2
                || s.all_in != (s.stack == 0 && !s.folded)
                || (s.folded || s.all_in) && s.can_raise
        }) {
            return Err("invalid seat chip or card state".into());
        }
        let n = self.seats.len();
        if self.dealer >= n
            || self.small_blind >= n
            || self.big_blind >= n
            || self.small_blind == self.big_blind
        {
            return Err("invalid dealer or blind indices".into());
        }
        let expected_sb = if n == 2 {
            self.dealer
        } else {
            (self.dealer + 1) % n
        };
        let expected_bb = if n == 2 {
            (self.dealer + 1) % n
        } else {
            (self.dealer + 2) % n
        };
        if self.small_blind != expected_sb || self.big_blind != expected_bb {
            return Err("blind positions do not match dealer".into());
        }
        if self.queue.iter().any(|&i| i >= n || !self.can_act(i))
            || self
                .queue
                .iter()
                .enumerate()
                .any(|(i, x)| self.queue[..i].contains(x))
        {
            return Err("invalid action queue".into());
        }
        if let Some(t) = self.turn {
            if t >= n || self.queue.first() != Some(&t) || !self.can_act(t) {
                return Err("invalid turn".into());
            }
        } else if !self.queue.is_empty() {
            return Err("queued actions without a turn".into());
        }
        if !self.terminal() && self.turn.is_none() {
            return Err("live hand has no acting seat".into());
        }
        let expected_board = match self.phase {
            Phase::Idle => 0,
            Phase::Preflop => 0,
            Phase::Flop => 3,
            Phase::Turn => 4,
            Phase::River => 5,
            Phase::Showdown => 5,
            Phase::Complete => self.board.len(),
        };
        if self.board.len() != expected_board {
            return Err("board count does not match phase".into());
        }
        if !matches!(self.phase, Phase::Idle) {
            let cards = self
                .deck
                .iter()
                .chain(&self.burned)
                .chain(&self.board)
                .chain(self.seats.iter().flat_map(|s| s.hole.iter()))
                .copied()
                .collect::<Vec<_>>();
            if cards.len() != 52 {
                return Err(format!("hand has {} cards instead of 52", cards.len()));
            }
            let mut ids = cards
                .iter()
                .map(|c| {
                    (
                        c.rank,
                        match c.suit {
                            Suit::Clubs => 0,
                            Suit::Diamonds => 1,
                            Suit::Hearts => 2,
                            Suit::Spades => 3,
                        },
                    )
                })
                .collect::<Vec<_>>();
            ids.sort_unstable();
            ids.dedup();
            if ids.len() != 52 {
                return Err("duplicate or invalid physical card".into());
            }
        }
        let total = self.pot() as u64 + self.seats.iter().map(|s| u64::from(s.stack)).sum::<u64>();
        if total != u64::from(self.buy_in) * n as u64 {
            return Err("chip conservation failed".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_all_in_does_not_reopen_a_closed_raise() {
        let mut g = Game::idle(2);
        assert!(g.start(70, 7));
        g.seats[0].stack += 30;
        g.seats[1].stack -= 30;
        assert_eq!(g.turn, Some(0));
        assert!(g.apply(Action::Raise(30)));
        assert_eq!(g.turn, Some(1));
        assert!(g.apply(Action::AllIn)); // Small blind moves to 40, a 10-chip short raise.
        assert_eq!(g.turn, Some(2));
        assert!(g.bet_bounds(2).is_some());
        assert!(g.legal_actions(2).contains(&Action::Raise(0)));
        assert!(g.apply(Action::Call));
        assert_eq!(g.turn, Some(0));
        assert!(!g.legal_actions(0).contains(&Action::Raise(0)));
        assert!(!g.legal_actions(0).contains(&Action::AllIn));
        assert!(g.apply(Action::Call));
        g.validate().unwrap();
    }

    #[test]
    fn heads_up_big_blind_keeps_option_to_raise_after_checks() {
        let mut g = Game::idle(1);
        assert!(g.start(200, 9));
        assert!(g.apply(Action::Call));
        assert_eq!(g.turn, Some(1));
        assert!(g.bet_bounds(1).is_some_and(|b| b.is_raise));
        assert!(g.apply(Action::Raise(30)));
        assert_eq!(g.turn, Some(0));
    }

    #[test]
    fn seeded_hands_progress_with_chip_conservation() {
        let mut g = Game::idle(4);
        for hand in 0..128u64 {
            assert!(g.start(200, hand + 1));
            let mut steps = 0;
            while !g.terminal() {
                g.validate().unwrap();
                let seat = g.turn.expect("active hand must have an actor");
                let action = if g.to_call(seat) == 0 {
                    Action::Check
                } else {
                    Action::Call
                };
                assert!(g.apply(action));
                steps += 1;
                assert!(steps < 64, "hand failed to advance");
            }
            g.validate().unwrap();
            assert_eq!(g.seats.iter().map(|s| s.stack).sum::<u32>(), 1_000);
        }
    }

    #[test]
    fn invalid_underraise_is_rejected_without_mutating_state() {
        let mut g = Game::idle(2);
        assert!(g.start(200, 33));
        let before = g.clone();
        assert!(!g.apply(Action::Raise(15)));
        assert_eq!(g, before);
    }

    #[test]
    fn mixed_cpu_human_hands_cover_two_through_five_seats() {
        for opponents in 1..=4 {
            let mut g = Game::idle(opponents);
            for hand in 0..32u64 {
                assert!(g.start(200, 1000 + hand + opponents as u64 * 100));
                let mut steps = 0;
                while !g.terminal() {
                    g.validate().unwrap();
                    if g.turn == Some(0) {
                        let legal = g.legal_actions(0);
                        let action = if g.to_call(0) == 0 {
                            if steps % 9 == 0 {
                                g.bet_bounds(0)
                                    .map(|b| Action::Bet(b.min_total))
                                    .unwrap_or(Action::Check)
                            } else {
                                Action::Check
                            }
                        } else if steps % 17 == 0 && legal.contains(&Action::AllIn) {
                            Action::AllIn
                        } else if steps % 13 == 0 && legal.contains(&Action::Fold) {
                            Action::Fold
                        } else {
                            Action::Call
                        };
                        assert!(g.apply(action));
                    } else {
                        assert!(g.cpu_step());
                    }
                    steps += 1;
                    assert!(steps < 160, "hand did not progress ({opponents} opponents)");
                }
                g.validate().unwrap();
                assert_eq!(
                    g.seats.iter().map(|s| s.stack).sum::<u32>(),
                    200 * (opponents as u32 + 1)
                );
            }
        }
    }

    fn card(rank: u8, suit: Suit) -> Card {
        Card::new(rank, suit, true)
    }

    #[test]
    fn side_pots_are_paid_only_to_eligible_winners() {
        let mut g = Game::idle(2);
        g.phase = Phase::River;
        g.board = vec![
            card(2, Suit::Clubs),
            card(2, Suit::Diamonds),
            card(14, Suit::Diamonds),
            card(13, Suit::Hearts),
            card(9, Suit::Spades),
        ];
        g.seats[0].hole = vec![card(14, Suit::Spades), card(14, Suit::Hearts)];
        g.seats[1].hole = vec![card(13, Suit::Diamonds), card(13, Suit::Clubs)];
        g.seats[2].hole = vec![card(12, Suit::Spades), card(11, Suit::Spades)];
        for (s, amount) in g.seats.iter_mut().zip([50, 100, 100]) {
            s.committed = amount;
            s.all_in = true;
        }
        g.dealer = 0;
        g.settle();
        assert_eq!(g.payouts, vec![150, 100, 0]);
        assert_eq!(g.seats.iter().map(|s| s.stack).sum::<u32>(), 250);
    }

    #[test]
    fn folded_only_excess_layer_is_returned_to_each_contributor() {
        let mut g = Game::idle(2);
        g.phase = Phase::River;
        g.board = vec![
            card(2, Suit::Clubs),
            card(2, Suit::Diamonds),
            card(14, Suit::Diamonds),
            card(13, Suit::Hearts),
            card(9, Suit::Spades),
        ];
        g.seats[0].hole = vec![card(14, Suit::Spades), card(14, Suit::Hearts)];
        g.seats[1].hole = vec![card(13, Suit::Diamonds), card(13, Suit::Clubs)];
        g.seats[2].hole = vec![card(12, Suit::Spades), card(11, Suit::Spades)];
        for (i, (s, amount)) in g.seats.iter_mut().zip([50, 100, 100]).enumerate() {
            s.committed = amount;
            s.all_in = true;
            s.folded = i > 0;
        }
        g.settle();
        assert_eq!(g.payouts, vec![150, 50, 50]);
        assert!(g.winner_text.contains("YOU WINS"));
        assert!(!g.winner_text.contains("OPPONENT"));
    }

    #[test]
    fn odd_split_chip_goes_clockwise_from_dealer() {
        let mut g = Game::idle(2);
        g.phase = Phase::River;
        g.dealer = 2;
        g.board = vec![
            card(14, Suit::Clubs),
            card(14, Suit::Diamonds),
            card(7, Suit::Spades),
            card(2, Suit::Clubs),
            card(2, Suit::Hearts),
        ];
        g.seats[0].hole = vec![card(13, Suit::Spades), card(12, Suit::Spades)];
        g.seats[1].hole = vec![card(13, Suit::Clubs), card(11, Suit::Clubs)];
        g.seats[2].hole = vec![card(12, Suit::Clubs), card(11, Suit::Diamonds)];
        for s in &mut g.seats {
            s.committed = 25;
            s.all_in = true;
        }
        g.settle();
        assert_eq!(g.payouts, vec![38, 37, 0]);
    }
}
