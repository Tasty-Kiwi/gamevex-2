use super::cards::{Card, Suit};
use serde::{Deserialize, Serialize};

pub const TARGET: u32 = 121;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Discard,
    Pegging,
    Show,
    RoundComplete,
    MatchComplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub total: u32,
    pub breakdown: Vec<(String, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub hands: [Vec<Card>; 2],
    pub remaining: [Vec<Card>; 2],
    pub crib: Vec<Card>,
    pub starter: Option<Card>,
    pub sequence: Vec<Card>,
    pub played: Vec<Card>,
    pub scores: [u32; 2],
    pub dealer: usize,
    pub turn: usize,
    pub phase: Phase,
    pub round: u32,
    pub count: u8,
    pub show_step: usize,
    pub last_message: String,
    pub seed: u64,
    pub rng_state: u64,
    pub discards: [Vec<Card>; 2],
    pub go: [bool; 2],
    pub last_player: Option<usize>,
}

fn value(c: Card) -> u8 {
    if c.rank == 14 { 1 } else { c.rank.min(10) }
}
fn rank(c: Card) -> u8 {
    if c.rank == 14 { 1 } else { c.rank }
}
fn suit_order(s: Suit) -> u8 {
    match s {
        Suit::Clubs => 0,
        Suit::Diamonds => 1,
        Suit::Hearts => 2,
        Suit::Spades => 3,
    }
}
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for s in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
        for r in 2..=14 {
            d.push(Card::new(r, s, true));
        }
    }
    d
}
fn score_add(v: &mut Vec<(String, u32)>, label: &str, n: u32) {
    if n > 0 {
        v.push((label.into(), n));
    }
}
fn same_face(a: Card, b: Card) -> bool {
    a.rank == b.rank && a.suit == b.suit
}
fn multiset(mut ids: Vec<(u8, u8)>) -> Vec<(u8, u8)> {
    ids.sort_unstable();
    ids
}
fn discard_crib_estimate(discard: [Card; 2]) -> f64 {
    let [a, b] = discard;
    let mut score = 0.0;
    if a.rank == b.rank {
        score += 2.2;
    }
    if value(a) + value(b) == 15 {
        score += 2.6;
    }
    if rank(a).abs_diff(rank(b)) == 1 {
        score += 1.1;
    }
    if value(a) == 5 {
        score += 1.8;
    }
    if value(b) == 5 {
        score += 1.8;
    }
    if a.suit == b.suit {
        score += 0.25;
    }
    if a.rank == 11 || b.rank == 11 {
        score += 0.2;
    }
    score
}
fn safe_weight(count: u8) -> i32 {
    match count {
        5 | 10 => -7,
        20 | 21 => -4,
        25 => -2,
        26 => -6,
        30 => -10,
        4 | 6 | 7 | 8 | 9 | 13 | 14 | 16 | 17 | 18 | 19 | 23 | 24 | 27 => 2,
        _ => 0,
    }
}
fn pegging_score(sequence: &[Card], card: Card, count: u8) -> u32 {
    let next = count + value(card);
    let mut total = if next == 15 || next == 31 { 2 } else { 0 };
    let same = sequence
        .iter()
        .rev()
        .take_while(|c| c.rank == card.rank)
        .count()
        + 1;
    total += match same {
        2 => 2,
        3 => 6,
        4.. => 12,
        _ => 0,
    };
    let mut combined = sequence.to_vec();
    combined.push(card);
    for len in (3..=combined.len().min(7)).rev() {
        let mut ranks: Vec<_> = combined[combined.len() - len..]
            .iter()
            .map(|c| rank(*c))
            .collect();
        ranks.sort_unstable();
        if ranks.windows(2).all(|w| w[1] == w[0] + 1) {
            total += len as u32;
            break;
        }
    }
    total
}
fn pegging_heuristic(sequence: &[Card], count: u8, card: Card) -> f64 {
    let next = count + value(card);
    let score = pegging_score(sequence, card, count) as f64;
    score * 100.0
        + safe_weight(next) as f64
        + if score == 0.0 {
            (11.0 - value(card) as f64) * 0.45
        } else {
            (11.0 - value(card) as f64) * 0.08
        }
        - if card.rank == 5 { 1.8 } else { 0.0 }
}
pub fn hand_score(cards: &[Card], starter: Card, is_crib: bool) -> Score {
    // Score at most a four-card hand plus starter. Keep this guard before any
    // subset enumeration so corrupt saves cannot trigger exponential work.
    if cards.len() != 4 {
        return Score {
            total: 0,
            breakdown: vec![],
        };
    }
    let mut all = cards.to_vec();
    all.push(starter);
    let mut b = vec![];
    let mut fifteens = 0;
    for mask in 1usize..(1 << all.len()) {
        let sum: u8 = all
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, c)| value(*c))
            .sum();
        if sum == 15 {
            fifteens += 1;
        }
    }
    score_add(
        &mut b,
        &format!(
            "FIFTEEN{}",
            if fifteens > 1 {
                format!(" x{fifteens}")
            } else {
                String::new()
            }
        ),
        fifteens * 2,
    );
    let mut pairs = 0;
    for i in 0..all.len() {
        for j in i + 1..all.len() {
            if all[i].rank == all[j].rank {
                pairs += 1;
            }
        }
    }
    score_add(
        &mut b,
        &format!(
            "PAIR{}",
            if pairs > 1 {
                format!(" x{pairs}")
            } else {
                String::new()
            }
        ),
        pairs * 2,
    );
    let mut longest = 0;
    let mut run_count = 0;
    for mask in 0usize..(1 << all.len()) {
        let n = mask.count_ones() as usize;
        if n < 3 {
            continue;
        }
        let mut rs: Vec<u8> = all
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, c)| rank(*c))
            .collect();
        rs.sort();
        rs.dedup();
        if rs.len() == n && rs.windows(2).all(|w| w[1] == w[0] + 1) {
            if n > longest {
                longest = n;
                run_count = 1
            } else if n == longest {
                run_count += 1
            }
        }
    }
    score_add(
        &mut b,
        &format!(
            "RUN OF {longest}{}",
            if run_count > 1 {
                format!(" x{run_count}")
            } else {
                String::new()
            }
        ),
        (longest * run_count) as u32,
    );
    if cards.len() == 4 && cards.iter().all(|c| c.suit == cards[0].suit) {
        let n = if starter.suit == cards[0].suit {
            5
        } else if is_crib {
            0
        } else {
            4
        };
        score_add(
            &mut b,
            if n == 5 {
                "FIVE-CARD FLUSH"
            } else {
                "FOUR-CARD FLUSH"
            },
            n,
        )
    }
    if cards.iter().any(|c| c.rank == 11 && c.suit == starter.suit) {
        score_add(&mut b, "NOBS", 1)
    }
    Score {
        total: b.iter().map(|x| x.1).sum(),
        breakdown: b,
    }
}
impl Game {
    pub fn new(seed: u64) -> Self {
        let mut g = Self {
            hands: [vec![], vec![]],
            remaining: [vec![], vec![]],
            crib: vec![],
            starter: None,
            sequence: vec![],
            played: vec![],
            scores: [0; 2],
            dealer: 1,
            turn: 0,
            phase: Phase::RoundComplete,
            round: 0,
            count: 0,
            show_step: 0,
            last_message: String::new(),
            seed,
            rng_state: seed.max(1),
            discards: [vec![], vec![]],
            go: [false; 2],
            last_player: None,
        };
        g.deal();
        g
    }
    fn random(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn deal(&mut self) {
        let mut d = deck();
        for i in (1..d.len()).rev() {
            let j = (self.random() % (i as u64 + 1)) as usize;
            d.swap(i, j);
        }
        self.hands = [vec![], vec![]];
        for _ in 0..6 {
            for p in 0..2 {
                self.hands[p].push(d.pop().unwrap());
            }
        }
        for hand in &mut self.hands {
            hand.sort_by_key(|c| (rank(*c), suit_order(c.suit)));
        }
        self.remaining = self.hands.clone();
        self.crib = vec![];
        self.discards = [vec![], vec![]];
        self.starter = None;
        self.sequence.clear();
        self.played.clear();
        self.count = 0;
        self.go = [false; 2];
        self.last_player = None;
        self.turn = 1 - self.dealer;
        self.phase = Phase::Discard;
        self.round += 1;
        self.show_step = 0;
        self.last_message = "DISCARD TWO CARDS".into();
    }
    pub fn legal_indices(&self, seat: usize) -> Vec<usize> {
        if seat > 1 || seat != self.turn || self.go[seat] {
            return vec![];
        }
        self.remaining[seat]
            .iter()
            .enumerate()
            .filter(|(_, c)| self.phase == Phase::Pegging && self.count + value(**c) <= 31)
            .map(|(i, _)| i)
            .collect()
    }
    pub fn discard(&mut self, indices: &[usize]) -> bool {
        if self.phase != Phase::Discard
            || indices.len() != 2
            || indices[0] == indices[1]
            || indices.iter().any(|&i| i >= self.hands[0].len())
            || !self.discards[0].is_empty()
        {
            return false;
        }
        let mut ix = indices.to_vec();
        ix.sort_unstable();
        for &i in ix.iter().rev() {
            self.discards[0].push(self.hands[0].remove(i));
        }
        self.hands[0].sort_by_key(|c| (rank(*c), suit_order(c.suit)));
        self.discards[0].sort_by_key(|c| (rank(*c), suit_order(c.suit)));
        self.remaining[0] = self.hands[0].clone();
        if self.discards[1].is_empty() {
            self.cpu_discard();
        }
        self.finish_discard_if_ready();
        true
    }
    fn cpu_discard(&mut self) -> bool {
        if self.discards[1].len() == 2 {
            return false;
        }
        let h = &self.hands[1];
        if h.len() != 6 {
            return false;
        }
        let mut candidates = deck();
        candidates.retain(|c| !h.iter().any(|x| same_face(*x, *c)));
        let mut best = (f64::NEG_INFINITY, [0usize, 1usize]);
        for a in 0..6 {
            for b in a + 1..6 {
                let keep: Vec<_> = h
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != a && *i != b)
                    .map(|(_, c)| *c)
                    .collect();
                let average = candidates
                    .iter()
                    .map(|starter| hand_score(&keep, *starter, false).total as f64)
                    .sum::<f64>()
                    / candidates.len() as f64;
                let discard_score = discard_crib_estimate([h[a], h[b]]);
                let pegging = keep
                    .iter()
                    .map(|c| match value(*c) {
                        1..=4 => 0.75,
                        5..=6 => 0.35,
                        10 => -0.1,
                        _ => 0.15,
                    })
                    .sum::<f64>();
                let score = average
                    + pegging * 0.15
                    + discard_score * if self.dealer == 1 { 0.6 } else { -0.75 };
                if score > best.0 {
                    best = (score, [a, b]);
                }
            }
        }
        for i in best.1.iter().rev() {
            self.discards[1].push(self.hands[1].remove(*i));
        }
        self.hands[1].sort_by_key(|c| (rank(*c), suit_order(c.suit)));
        self.discards[1].sort_by_key(|c| (rank(*c), suit_order(c.suit)));
        self.remaining[1] = self.hands[1].clone();
        true
    }
    fn finish_discard_if_ready(&mut self) {
        if self.discards.iter().all(|x| x.len() == 2) {
            self.crib = self.discards[0]
                .iter()
                .chain(&self.discards[1])
                .copied()
                .collect();
            let mut d = deck();
            let used: Vec<_> = self
                .hands
                .iter()
                .flatten()
                .chain(self.crib.iter())
                .copied()
                .collect();
            d.retain(|c| !used.iter().any(|u| u.rank == c.rank && u.suit == c.suit));
            self.starter = d[(self.random() % d.len() as u64) as usize].into();
            self.phase = Phase::Pegging;
            self.turn = 1 - self.dealer;
            self.last_message = "PEG".into();
            if self.starter.unwrap().rank == 11 {
                self.award(self.dealer, 2, "HEELS");
            }
        }
    }
    fn award(&mut self, p: usize, n: u32, label: &str) {
        if n == 0 || self.phase == Phase::MatchComplete {
            return;
        }
        self.scores[p] = self.scores[p].saturating_add(n);
        self.last_message = format!("{} / {label} +{n}", if p == 0 { "YOU" } else { "CPU" });
        if self.scores[p] >= TARGET {
            self.phase = Phase::MatchComplete;
            self.last_message = if p == 0 { "YOU WIN!" } else { "CPU WINS" }.into();
        }
    }
    pub fn play(&mut self, index: usize) -> bool {
        if self.phase != Phase::Pegging
            || index >= self.remaining[self.turn].len()
            || self.count + value(self.remaining[self.turn][index]) > 31
        {
            return false;
        }
        let p = self.turn;
        let c = self.remaining[p].remove(index);
        self.count += value(c);
        self.sequence.push(c);
        self.played.push(c);
        self.last_player = Some(p);
        self.go[p] = false;
        let mut pts = 0;
        let mut labels = vec![];
        if self.count == 15 {
            pts += 2;
            labels.push("FIFTEEN");
        }
        if self.count == 31 {
            pts += 2;
            labels.push("THIRTY-ONE");
        }
        let same = self
            .sequence
            .iter()
            .rev()
            .take_while(|x| x.rank == c.rank)
            .count();
        if same >= 2 {
            let n = match same {
                2 => 2,
                3 => 6,
                _ => 12,
            };
            pts += n;
            labels.push(match same {
                2 => "PAIR",
                3 => "PAIR ROYAL",
                _ => "DOUBLE PAIR ROYAL",
            });
        }
        for n in (3..=self.sequence.len().min(7)).rev() {
            let mut r: Vec<_> = self.sequence[self.sequence.len() - n..]
                .iter()
                .map(|c| rank(*c))
                .collect();
            r.sort();
            if r.windows(2).all(|w| w[1] == w[0] + 1) {
                pts += n as u32;
                labels.push("RUN");
                break;
            }
        }
        self.last_message = format!(
            "{} / {}{} / Count {}",
            if p == 0 { "YOU" } else { "CPU" },
            c.rank_label(),
            c.suit.symbol(),
            self.count
        );
        self.award(p, pts, &labels.join(" + "));
        if self.phase == Phase::MatchComplete {
            return true;
        }
        if self.count == 31 {
            let leader = 1 - p;
            self.end_sequence(false);
            self.turn = self.first_with_cards(leader);
            self.continue_or_end();
        } else {
            self.turn = 1 - p;
            self.continue_or_end();
        }
        true
    }
    fn end_sequence(&mut self, last: bool) {
        if last
            && self.count < 31
            && let Some(p) = self.last_player
        {
            self.award(p, 1, "LAST CARD");
        }
        self.count = 0;
        self.sequence.clear();
        self.go = [false; 2];
        self.last_player = None;
    }
    fn first_with_cards(&self, preferred: usize) -> usize {
        if !self.remaining[preferred].is_empty() {
            preferred
        } else if !self.remaining[1 - preferred].is_empty() {
            1 - preferred
        } else {
            preferred
        }
    }
    fn continue_or_end(&mut self) {
        if self.remaining[0].is_empty() && self.remaining[1].is_empty() {
            if self.count > 0
                && self.count < 31
                && let Some(p) = self.last_player
            {
                self.award(p, 1, "LAST CARD");
            }
            if self.phase != Phase::MatchComplete {
                self.end_sequence(false);
                self.phase = Phase::Show;
                self.show_step = 0;
                self.turn = 1 - self.dealer;
                self.last_message = "SHOW".into();
            }
            return;
        }
        let next = self.turn;
        let previous = 1 - next;
        if self.go[next] {
            if !self.remaining[previous].is_empty() && self.has_legal(previous) {
                self.turn = previous;
            } else {
                self.finish_go();
            }
        }
    }
    fn has_legal(&self, seat: usize) -> bool {
        self.remaining[seat]
            .iter()
            .any(|c| self.count + value(*c) <= 31)
    }
    fn finish_go(&mut self) {
        if self.count < 31
            && let Some(p) = self.last_player
        {
            self.award(p, 1, "GO");
        }
        if self.phase != Phase::MatchComplete {
            let leader = self.last_player.map_or(1 - self.turn, |p| 1 - p);
            self.end_sequence(false);
            self.turn = self.first_with_cards(leader);
            self.continue_or_end();
        }
    }
    pub fn go(&mut self) -> bool {
        if self.phase != Phase::Pegging || !self.legal_indices(self.turn).is_empty() {
            return false;
        }
        let p = self.turn;
        self.go[p] = true;
        self.last_message = "GO".into();
        let other = 1 - p;
        if self.go[other] || !self.has_legal(other) {
            self.finish_go();
        } else {
            self.turn = other;
        }
        true
    }
    pub fn cpu_step(&mut self) -> bool {
        if self.phase == Phase::Discard && self.discards[1].len() != 2 {
            if !self.cpu_discard() {
                return false;
            }
            self.finish_discard_if_ready();
            return true;
        }
        if self.phase != Phase::Pegging || self.turn != 1 {
            return false;
        }
        let legal = self.legal_indices(1);
        if legal.is_empty() {
            return self.go();
        }
        let mut best = legal[0];
        let mut best_score = pegging_heuristic(&self.sequence, self.count, self.remaining[1][best]);
        for &candidate in &legal[1..] {
            let score = pegging_heuristic(&self.sequence, self.count, self.remaining[1][candidate]);
            if score > best_score
                || (score == best_score
                    && value(self.remaining[1][candidate]) < value(self.remaining[1][best]))
            {
                best = candidate;
                best_score = score;
            }
        }
        let i = best;
        self.play(i)
    }
    pub fn advance(&mut self) -> bool {
        if self.phase == Phase::Show {
            let order = [1 - self.dealer, self.dealer, self.dealer];
            let who = order[self.show_step.min(2)];
            let is_crib = self.show_step == 2;
            let score = hand_score(
                if is_crib {
                    &self.crib
                } else {
                    &self.hands[who]
                },
                self.starter.unwrap(),
                is_crib,
            );
            self.show_step += 1;
            self.award(who, score.total, "SHOW");
            if self.phase != Phase::MatchComplete && self.show_step >= 3 {
                self.phase = Phase::RoundComplete;
                self.last_message = "ROUND COMPLETE".into();
            }
            return true;
        }
        if self.phase == Phase::RoundComplete {
            self.dealer = 1 - self.dealer;
            self.deal();
            return true;
        }
        false
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.dealer > 1
            || self.turn > 1
            || self.round == 0
            || self.round > 1_000
            || self.rng_state == 0
            || self.last_player.is_some_and(|p| p > 1)
        {
            return Err("invalid seat, round, or random state".into());
        }
        if self.last_message.len() > 200 || self.scores.iter().any(|s| *s > TARGET + 31) {
            return Err("message or score outside bounds".into());
        }
        if self.scores.iter().filter(|s| **s >= TARGET).count() > 1 {
            return Err("multiple match winners".into());
        }
        if (self.phase == Phase::MatchComplete) != self.scores.iter().any(|s| *s >= TARGET) {
            return Err("match phase and target score disagree".into());
        }
        let mut inventory: Vec<Card> = self.hands.iter().flatten().copied().collect();
        if self.phase == Phase::Discard {
            inventory.extend(self.discards.iter().flatten().copied());
        } else {
            inventory.extend(self.crib.iter().copied());
        }
        if let Some(starter) = self.starter {
            inventory.push(starter);
        }
        if inventory
            .iter()
            .any(|c| !(2..=14).contains(&c.rank) || !c.face_up)
        {
            return Err("invalid card rank".into());
        }
        if self
            .remaining
            .iter()
            .flatten()
            .chain(self.played.iter())
            .chain(self.sequence.iter())
            .chain(self.discards.iter().flatten())
            .any(|c| !(2..=14).contains(&c.rank) || !c.face_up)
        {
            return Err("invalid card alias".into());
        }
        let mut ids: Vec<_> = inventory.iter().map(|c| (c.rank, c.suit as u8)).collect();
        ids.sort_unstable();
        ids.dedup();
        if ids.len() != inventory.len() || inventory.len() > 13 {
            return Err("duplicate or excessive card inventory".into());
        }
        if self.phase == Phase::Discard {
            for p in 0..2 {
                if ![4, 6].contains(&self.hands[p].len())
                    || ![0, 2].contains(&self.discards[p].len())
                    || self.hands[p].len() + self.discards[p].len() != 6
                {
                    return Err("invalid discard state".into());
                }
            }
            if self.starter.is_some()
                || !self.crib.is_empty()
                || !self.played.is_empty()
                || !self.sequence.is_empty()
                || self.count != 0
                || self.go != [false; 2]
                || self.last_player.is_some()
                || self.show_step != 0
            {
                return Err("premature cut or pegging state".into());
            }
            for p in 0..2 {
                if multiset(
                    self.remaining[p]
                        .iter()
                        .map(|c| (c.rank, c.suit as u8))
                        .collect(),
                ) != multiset(
                    self.hands[p]
                        .iter()
                        .map(|c| (c.rank, c.suit as u8))
                        .collect(),
                ) {
                    return Err("discard remaining cards differ from hands".into());
                }
            }
        } else {
            if self.hands.iter().any(|h| h.len() != 4)
                || self.discards.iter().any(|d| d.len() != 2)
                || self.crib.len() != 4
                || self.starter.is_none()
            {
                return Err("invalid post-discard card counts".into());
            }
            let expected: Vec<_> = self
                .discards
                .iter()
                .flatten()
                .map(|c| (c.rank, c.suit as u8))
                .collect();
            let actual: Vec<_> = self.crib.iter().map(|c| (c.rank, c.suit as u8)).collect();
            if multiset(expected) != multiset(actual) {
                return Err("crib differs from discards".into());
            }
            for p in 0..2 {
                let kept: Vec<_> = self.remaining[p]
                    .iter()
                    .chain(
                        self.played
                            .iter()
                            .filter(|c| self.hands[p].iter().any(|h| same_face(**c, *h))),
                    )
                    .map(|c| (c.rank, c.suit as u8))
                    .collect();
                let original: Vec<_> = self.hands[p]
                    .iter()
                    .map(|c| (c.rank, c.suit as u8))
                    .collect();
                if multiset(kept) != multiset(original) {
                    return Err("remaining and played cards do not partition hand".into());
                }
            }
            if self
                .played
                .iter()
                .any(|c| !self.hands.iter().flatten().any(|h| same_face(*c, *h)))
            {
                return Err("played card is not in a kept hand".into());
            }
            if self.sequence.len() > 8
                || self
                    .sequence
                    .iter()
                    .any(|c| !self.played.iter().any(|p| same_face(*p, *c)))
            {
                return Err("invalid pegging sequence".into());
            }
            if !self.played.ends_with(&self.sequence) {
                return Err("sequence is not the played-card suffix".into());
            }
            if self.sequence.iter().map(|c| value(*c)).sum::<u8>() != self.count {
                return Err("pegging count differs from sequence".into());
            }
        }
        match self.phase {
            Phase::Pegging if self.count > 31 || self.show_step != 0 => {
                return Err("invalid pegging phase state".into());
            }
            Phase::Show if self.show_step > 2 || self.count != 0 || !self.sequence.is_empty() => {
                return Err("invalid show state".into());
            }
            Phase::RoundComplete if self.show_step != 3 || self.count != 0 => {
                return Err("invalid completed round state".into());
            }
            Phase::Show | Phase::RoundComplete
                if !self.remaining.iter().all(Vec::is_empty) || self.played.len() != 8 =>
            {
                return Err("show started before pegging completed".into());
            }
            Phase::MatchComplete if self.show_step > 3 || self.count > 31 => {
                return Err("invalid match-complete state".into());
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(r: u8, s: Suit) -> Card {
        Card::new(r, s, true)
    }
    #[test]
    fn famous_twenty_nine_and_flush_rules() {
        let score = hand_score(
            &[
                c(5, Suit::Clubs),
                c(5, Suit::Diamonds),
                c(5, Suit::Hearts),
                c(11, Suit::Spades),
            ],
            c(5, Suit::Spades),
            false,
        );
        assert_eq!(score.total, 29);
        let four = hand_score(
            &[
                c(2, Suit::Clubs),
                c(4, Suit::Clubs),
                c(7, Suit::Clubs),
                c(9, Suit::Clubs),
            ],
            c(13, Suit::Spades),
            false,
        );
        assert_eq!(four.total, 6);
        let crib = hand_score(
            &[
                c(2, Suit::Clubs),
                c(4, Suit::Clubs),
                c(7, Suit::Clubs),
                c(9, Suit::Clubs),
            ],
            c(13, Suit::Spades),
            true,
        );
        assert_eq!(crib.total, 2);
    }
    #[test]
    fn pegging_scores_pair_run_go_and_thirty_one() {
        let mut g = Game::new(33);
        g.phase = Phase::Pegging;
        g.turn = 0;
        g.remaining = [
            vec![
                c(5, Suit::Clubs),
                c(5, Suit::Diamonds),
                c(6, Suit::Clubs),
                c(10, Suit::Clubs),
            ],
            vec![c(5, Suit::Hearts), c(8, Suit::Clubs)],
        ];
        assert!(g.play(0));
        assert!(g.play(0));
        assert_eq!(g.scores[1], 2);
        assert!(g.play(1));
        assert!(g.play(0));
        g.count = 21;
        g.turn = 0;
        g.sequence = vec![
            c(5, Suit::Clubs),
            c(5, Suit::Diamonds),
            c(6, Suit::Clubs),
            c(5, Suit::Hearts),
        ];
        g.remaining[0] = vec![c(10, Suit::Clubs)];
        assert!(g.play(0));
        assert_eq!(g.count, 0);
        assert_eq!(g.scores[0], 2);
    }
    #[test]
    fn hand_scores_runs_and_nobs_and_rejects_oversized_input() {
        let run = hand_score(
            &[
                c(2, Suit::Clubs),
                c(3, Suit::Diamonds),
                c(4, Suit::Hearts),
                c(9, Suit::Spades),
            ],
            c(5, Suit::Clubs),
            false,
        );
        assert!(
            run.breakdown
                .iter()
                .any(|(label, points)| label.starts_with("RUN OF 4") && *points == 4)
        );
        let nobs = hand_score(
            &[
                c(11, Suit::Clubs),
                c(2, Suit::Diamonds),
                c(7, Suit::Hearts),
                c(9, Suit::Spades),
            ],
            c(4, Suit::Clubs),
            false,
        );
        assert!(
            nobs.breakdown
                .iter()
                .any(|(label, points)| label == "NOBS" && *points == 1)
        );
        let mut oversized = Vec::new();
        oversized.resize(1_000_000, c(2, Suit::Clubs));
        assert_eq!(hand_score(&oversized, c(3, Suit::Hearts), false).total, 0);
    }
    #[test]
    fn thirty_one_and_go_reset_to_the_other_player_and_continue() {
        let mut g = Game::new(5);
        g.phase = Phase::Pegging;
        g.turn = 0;
        g.count = 21;
        g.sequence = vec![
            c(10, Suit::Clubs),
            c(10, Suit::Diamonds),
            c(11, Suit::Hearts),
        ];
        g.remaining = [vec![c(10, Suit::Spades)], vec![c(2, Suit::Clubs)]];
        assert!(g.play(0));
        assert_eq!(g.count, 0);
        assert_eq!(g.turn, 1);
        assert_eq!(g.remaining[1].len(), 1);
        g.remaining = [vec![c(2, Suit::Spades)], vec![c(3, Suit::Clubs)]];
        g.count = 30;
        g.sequence = vec![
            c(10, Suit::Clubs),
            c(10, Suit::Diamonds),
            c(10, Suit::Hearts),
        ];
        g.turn = 0;
        g.last_player = Some(1);
        g.go = [false; 2];
        assert!(g.go());
        assert_eq!(g.count, 0);
        assert_eq!(g.turn, 0);
    }
    #[test]
    fn deterministic_simulated_rounds_validate_and_save_roundtrip() {
        let mut g = Game::new(99);
        let mut guard = 0;
        while !matches!(g.phase, Phase::RoundComplete | Phase::MatchComplete) && guard < 1000 {
            guard += 1;
            assert!(g.validate().is_ok());
            match g.phase {
                Phase::Discard => {
                    assert!(g.discard(&[0, 1]));
                }
                Phase::Pegging => {
                    if g.turn == 1 {
                        g.cpu_step();
                    } else if let Some(i) = g.legal_indices(0).first().copied() {
                        g.play(i);
                    } else {
                        g.go();
                    }
                }
                Phase::Show => {
                    g.advance();
                }
                _ => {}
            }
        }
        assert!(guard < 1000);
        let data = serde_json::to_vec(&g).unwrap();
        let restored: Game = serde_json::from_slice(&data).unwrap();
        assert_eq!(restored, g);
        assert!(restored.validate().is_ok());
    }
    #[test]
    fn sixty_four_seeded_complete_matches_validate_after_every_action() {
        for seed in 0..64 {
            let mut g = Game::new(seed);
            let mut steps = 0usize;
            while g.phase != Phase::MatchComplete && steps < 100_000 {
                assert!(g.validate().is_ok(), "seed={seed} state={g:?}");
                match g.phase {
                    Phase::Discard => {
                        assert!(g.discard(&[0, 1]));
                    }
                    Phase::Pegging => {
                        if g.turn == 1 {
                            assert!(g.cpu_step());
                        } else if let Some(i) = g.legal_indices(0).first().copied() {
                            assert!(g.play(i));
                        } else {
                            assert!(g.go());
                        }
                    }
                    Phase::Show | Phase::RoundComplete => assert!(g.advance()),
                    Phase::MatchComplete => {}
                }
                steps += 1;
            }
            assert_eq!(g.phase, Phase::MatchComplete, "seed={seed}");
            assert!(g.validate().is_ok(), "seed={seed} terminal={g:?}");
        }
    }
    #[test]
    fn validation_rejects_corrupt_saved_card_phase_and_sequence() {
        let mut g = Game::new(71);
        let mut guard = 0;
        while g.phase == Phase::Discard {
            g.discard(&[0, 1]);
            guard += 1;
            assert!(guard < 3);
        }
        let data = serde_json::to_value(&g).unwrap();
        let restored: Game = serde_json::from_value(data.clone()).unwrap();
        assert!(restored.validate().is_ok());
        let mut bad = data.clone();
        bad["count"] = 32.into();
        assert!(
            serde_json::from_value::<Game>(bad)
                .unwrap()
                .validate()
                .is_err()
        );
        let mut bad = data.clone();
        bad["starter"] = bad["hands"][0][0].clone();
        assert!(
            serde_json::from_value::<Game>(bad)
                .unwrap()
                .validate()
                .is_err()
        );
        let mut bad = data.clone();
        bad["sequence"] = serde_json::json!([{"rank":2,"suit":"Clubs","face_up":true}]);
        assert!(
            serde_json::from_value::<Game>(bad)
                .unwrap()
                .validate()
                .is_err()
        );
        let mut bad = data;
        bad["rng_state"] = 0.into();
        assert!(
            serde_json::from_value::<Game>(bad)
                .unwrap()
                .validate()
                .is_err()
        );
    }
}
