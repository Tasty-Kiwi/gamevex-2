//! Deterministic four-seat Hearts engine.
use serde::{Deserialize, Serialize};

use super::cards::{Card, Suit};

const MATCH_LIMIT: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Passing,
    Playing,
    TrickComplete,
    HandComplete,
    MatchComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassDirection {
    Left,
    Right,
    Across,
    Hold,
}

impl PassDirection {
    pub const fn for_hand(hand: u32) -> Self {
        match hand.saturating_sub(1) % 4 {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Across,
            _ => Self::Hold,
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Across => "across",
            Self::Hold => "hold",
        }
    }
    fn target(self, seat: usize) -> usize {
        match self {
            Self::Left => (seat + 1) % 4,
            Self::Right => (seat + 3) % 4,
            Self::Across => (seat + 2) % 4,
            Self::Hold => seat,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Play {
    pub seat: usize,
    pub card: Card,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub hands: [Vec<Card>; 4],
    pub trick: Vec<Play>,
    pub captured: [Vec<Card>; 4],
    pub totals: [u32; 4],
    pub hand_points: [u8; 4],
    pub hand_number: u32,
    pub turn: usize,
    pub hearts_broken: bool,
    pub seed: u64,
    pub phase: Phase,
    pub completed_tricks: u8,
    pub leader: usize,
}

impl Game {
    /// Deal the first hand and begin its left pass.
    pub fn new(seed: u64) -> Self {
        let mut g = Self {
            hands: std::array::from_fn(|_| Vec::new()),
            trick: vec![],
            captured: std::array::from_fn(|_| Vec::new()),
            totals: [0; 4],
            hand_points: [0; 4],
            hand_number: 1,
            turn: 0,
            hearts_broken: false,
            seed: seed.max(1),
            phase: Phase::Passing,
            completed_tricks: 0,
            leader: 0,
        };
        g.deal();
        g
    }

    pub const fn pass_direction(&self) -> PassDirection {
        PassDirection::for_hand(self.hand_number)
    }

    /// Final match winner, with lower seat number breaking tied totals.
    pub fn winner(&self) -> Option<usize> {
        (self.phase == Phase::MatchComplete)
            .then(|| (0..4).min_by_key(|&seat| (self.totals[seat], seat)))
            .flatten()
    }

    fn random(&mut self) -> u64 {
        let mut x = self.seed.max(1);
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.seed = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn deal(&mut self) {
        let mut deck = Vec::with_capacity(52);
        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            for rank in 2..=14 {
                deck.push(Card::new(rank, suit, true));
            }
        }
        for i in (1..deck.len()).rev() {
            let j = (self.random() % (i as u64 + 1)) as usize;
            deck.swap(i, j);
        }
        self.hands = std::array::from_fn(|_| Vec::with_capacity(13));
        for (i, c) in deck.into_iter().enumerate() {
            self.hands[i % 4].push(c);
        }
        for hand in &mut self.hands {
            sort_hand(hand);
        }
        self.trick.clear();
        self.captured = std::array::from_fn(|_| Vec::new());
        self.hand_points = [0; 4];
        self.completed_tricks = 0;
        self.hearts_broken = false;
        self.leader = self
            .hands
            .iter()
            .position(|h| h.contains(&two_clubs()))
            .unwrap_or(0);
        self.turn = self.leader;
        self.phase = if self.pass_direction() == PassDirection::Hold {
            Phase::Playing
        } else {
            Phase::Passing
        };
        if self.phase == Phase::Playing {
            self.apply_opening_lead();
        }
    }

    fn apply_opening_lead(&mut self) {
        self.leader = self
            .hands
            .iter()
            .position(|h| h.contains(&two_clubs()))
            .unwrap_or(0);
        self.turn = self.leader;
    }

    /// Submit exactly three distinct human hand indices. CPU passes are chosen automatically.
    pub fn pass(&mut self, indices: &[usize]) -> bool {
        if self.phase != Phase::Passing || indices.len() != 3 {
            return false;
        }
        let mut chosen = indices.to_vec();
        chosen.sort_unstable();
        chosen.dedup();
        if chosen.len() != 3 || chosen.iter().any(|&i| i >= self.hands[0].len()) {
            return false;
        }
        let dir = self.pass_direction();
        let mut selections: [Vec<usize>; 4] = std::array::from_fn(|_| Vec::new());
        selections[0] = chosen;
        for (seat, selection) in selections.iter_mut().enumerate().skip(1) {
            *selection = pass_choices(&self.hands[seat]);
        }
        let mut out: [Vec<Card>; 4] = std::array::from_fn(|_| Vec::new());
        for ((hand, selected), outgoing) in self.hands.iter().zip(&selections).zip(&mut out) {
            for (i, card) in hand.iter().enumerate() {
                if selected.contains(&i) {
                    outgoing.push(*card);
                }
            }
        }
        for (hand, selected) in self.hands.iter_mut().zip(&selections) {
            *hand = hand
                .iter()
                .enumerate()
                .filter_map(|(i, card)| (!selected.contains(&i)).then_some(*card))
                .collect();
        }
        for (source, outgoing) in out.iter().enumerate() {
            let target = dir.target(source);
            self.hands[target].extend(outgoing.iter().copied());
        }
        for hand in &mut self.hands {
            sort_hand(hand);
        }
        self.phase = Phase::Playing;
        self.apply_opening_lead();
        true
    }

    /// Legal indices for a seat in the current trick. Empty unless that seat acts.
    pub fn legal_indices(&self, seat: usize) -> Vec<usize> {
        if seat >= 4 || self.phase != Phase::Playing || seat != self.turn || self.trick.len() >= 4 {
            return vec![];
        }
        legal(
            &self.hands[seat],
            &self.trick,
            self.hearts_broken,
            self.completed_tricks,
        )
    }

    pub fn play(&mut self, index: usize) -> bool {
        if self.phase != Phase::Playing || self.turn != 0 || !self.legal_indices(0).contains(&index)
        {
            return false;
        }
        self.play_at(0, index)
    }

    /// Perform one CPU move only when the current actor is a CPU.
    pub fn cpu_step(&mut self) -> bool {
        if self.phase != Phase::Playing || self.turn == 0 {
            return false;
        }
        let seat = self.turn;
        let legal = self.legal_indices(seat);
        let Some(index) = choose_play(&self.hands[seat], &legal, &self.trick) else {
            return false;
        };
        self.play_at(seat, index)
    }

    fn play_at(&mut self, seat: usize, index: usize) -> bool {
        let Some(card) = self.hands[seat].get(index).copied() else {
            return false;
        };
        if !legal(
            &self.hands[seat],
            &self.trick,
            self.hearts_broken,
            self.completed_tricks,
        )
        .contains(&index)
        {
            return false;
        }
        self.hands[seat].remove(index);
        self.trick.push(Play { seat, card });
        if card.suit == Suit::Hearts {
            self.hearts_broken = true;
        }
        if self.trick.len() == 4 {
            self.phase = Phase::TrickComplete;
        } else {
            self.turn = (seat + 1) % 4;
        }
        true
    }

    /// Resolve a visible four-card trick, capture cards for inventory tracking, and score after trick 13.
    pub fn resolve_trick(&mut self) -> bool {
        if self.phase != Phase::TrickComplete || self.trick.len() != 4 {
            return false;
        }
        let winner = trick_winner(&self.trick);
        for p in &self.trick {
            self.hand_points[winner] = self.hand_points[winner].saturating_add(points(p.card));
            self.captured[winner].push(p.card);
        }
        self.completed_tricks += 1;
        self.trick.clear();
        self.leader = winner;
        self.turn = winner;
        if self.completed_tricks == 13 {
            if let Some(shooter) = self.hand_points.iter().position(|&p| p == 26) {
                self.hand_points = std::array::from_fn(|s| if s == shooter { 0 } else { 26 });
            }
            for s in 0..4 {
                self.totals[s] = self.totals[s].saturating_add(u32::from(self.hand_points[s]));
            }
            self.phase = if self.totals.iter().any(|&v| v >= MATCH_LIMIT) {
                Phase::MatchComplete
            } else {
                Phase::HandComplete
            };
        } else {
            self.phase = Phase::Playing;
        }
        true
    }

    pub fn next_hand(&mut self) -> bool {
        if self.phase != Phase::HandComplete {
            return false;
        }
        self.hand_number = self.hand_number.saturating_add(1);
        self.deal();
        true
    }

    /// Reject corrupt or impossible saved state before restoring it.
    pub fn validate(&self) -> Result<(), String> {
        if self.hand_number == 0
            || self.turn >= 4
            || self.leader >= 4
            || self.trick.len() > 4
            || self.completed_tricks > 13
        {
            return Err("invalid Hearts counters or seat".into());
        }
        if self.seed == 0 {
            return Err("invalid random state".into());
        }
        if self
            .trick
            .iter()
            .enumerate()
            .any(|(i, p)| p.seat >= 4 || p.seat != (self.leader + i) % 4 || !valid_card(p.card))
        {
            return Err("invalid trick".into());
        }
        if self
            .hands
            .iter()
            .flatten()
            .chain(self.trick.iter().map(|p| &p.card))
            .chain(self.captured.iter().flatten())
            .any(|c| !valid_card(*c) || !c.face_up)
        {
            return Err("invalid card".into());
        }
        let mut seen = [[false; 13]; 4];
        for c in self
            .hands
            .iter()
            .flatten()
            .chain(self.trick.iter().map(|p| &p.card))
            .chain(self.captured.iter().flatten())
        {
            let si = suit_idx(c.suit);
            let ri = (c.rank - 2) as usize;
            if seen[si][ri] {
                return Err("duplicate card".into());
            }
            seen[si][ri] = true;
        }
        if seen.iter().flatten().any(|v| !v) {
            return Err("incomplete card inventory".into());
        }
        if self.hands.iter().any(|h| h.len() > 13)
            || self.trick.len()
                + self.completed_tricks as usize * 4
                + self.hands.iter().map(Vec::len).sum::<usize>()
                != 52
        {
            return Err("card count disagrees with trick count".into());
        }
        if self.captured.iter().map(Vec::len).sum::<usize>() != self.completed_tricks as usize * 4 {
            return Err("captured count disagrees with trick count".into());
        }
        if self.hand_points.iter().any(|&p| p > 26) {
            return Err("invalid hand points".into());
        }
        let captured_points: [u8; 4] = std::array::from_fn(|s| {
            self.captured[s]
                .iter()
                .fold(0u8, |n, c| n.saturating_add(points(*c)))
        });
        if self.phase == Phase::Playing
            || self.phase == Phase::Passing
            || self.phase == Phase::TrickComplete
        {
            if captured_points != self.hand_points {
                return Err("hand score disagrees with captured cards".into());
            }
        } else {
            let raw_total = captured_points.iter().map(|&n| u16::from(n)).sum::<u16>();
            let final_total = self.hand_points.iter().map(|&n| u16::from(n)).sum::<u16>();
            let moon = captured_points
                .iter()
                .position(|&n| n == 26)
                .is_some_and(|shooter| {
                    captured_points
                        .iter()
                        .enumerate()
                        .all(|(seat, &score)| score == if seat == shooter { 26 } else { 0 })
                        && self
                            .hand_points
                            .iter()
                            .enumerate()
                            .all(|(seat, &score)| score == if seat == shooter { 0 } else { 26 })
                });
            if raw_total != 26
                || (self.hand_points != captured_points && !moon)
                || final_total != if moon { 78 } else { 26 }
            {
                return Err("completed hand score disagrees with captured cards".into());
            }
        }
        let any_heart_played = self.trick.iter().any(|p| p.card.suit == Suit::Hearts)
            || self
                .captured
                .iter()
                .flatten()
                .any(|c| c.suit == Suit::Hearts);
        if self.hearts_broken != any_heart_played {
            return Err("hearts broken flag lost".into());
        }
        let match_over = self.totals.iter().any(|&score| score >= MATCH_LIMIT);
        if (self.phase == Phase::MatchComplete) != match_over
            || self.totals.iter().any(|&score| score > MATCH_LIMIT + 25)
        {
            return Err("match score disagrees with phase".into());
        }
        if self.phase != Phase::MatchComplete && match_over {
            return Err("match ended before threshold".into());
        }
        match self.phase {
            Phase::Passing
                if self.pass_direction() == PassDirection::Hold
                    || self.completed_tricks != 0
                    || !self.trick.is_empty()
                    || self.hands.iter().any(|h| h.len() != 13)
                    || self.turn != self.leader
                    || !self.hands[self.leader].contains(&two_clubs()) =>
            {
                return Err("invalid passing state".into());
            }
            Phase::Playing
                if self.trick.len() >= 4
                    || self.turn != (self.leader + self.trick.len()) % 4
                    || (0..4).any(|seat| {
                        self.hands[seat].len()
                            + self.completed_tricks as usize
                            + self.trick.iter().filter(|p| p.seat == seat).count()
                            != 13
                    }) =>
            {
                return Err("invalid playing state".into());
            }
            Phase::TrickComplete
                if self.trick.len() != 4
                    || self.turn != (self.leader + 3) % 4
                    || (0..4).any(|seat| {
                        self.hands[seat].len()
                            + self.completed_tricks as usize
                            + self.trick.iter().filter(|p| p.seat == seat).count()
                            != 13
                    }) =>
            {
                return Err("invalid completed trick state".into());
            }
            Phase::HandComplete | Phase::MatchComplete
                if self.completed_tricks != 13
                    || !self.trick.is_empty()
                    || self.hands.iter().any(|h| !h.is_empty()) =>
            {
                return Err("invalid completed hand state".into());
            }
            _ => {}
        }
        if (self.phase == Phase::Playing
            && self.completed_tricks == 0
            && self.trick.is_empty()
            && !self.hands[self.turn].contains(&two_clubs()))
            || (self.completed_tricks == 0
                && !self.trick.is_empty()
                && self.trick[0].card != two_clubs())
        {
            return Err("opening lead must be the two of clubs".into());
        }
        Ok(())
    }
}

fn valid_card(c: Card) -> bool {
    (2..=14).contains(&c.rank)
}
fn suit_idx(s: Suit) -> usize {
    match s {
        Suit::Spades => 0,
        Suit::Hearts => 1,
        Suit::Clubs => 2,
        Suit::Diamonds => 3,
    }
}
fn sort_hand(hand: &mut [Card]) {
    hand.sort_by_key(|card| (suit_idx(card.suit), card.rank));
}
fn two_clubs() -> Card {
    Card::new(2, Suit::Clubs, true)
}
fn is_point(c: Card) -> bool {
    c.suit == Suit::Hearts || (c.suit == Suit::Spades && c.rank == 12)
}
fn points(c: Card) -> u8 {
    if c.suit == Suit::Hearts {
        1
    } else if c.suit == Suit::Spades && c.rank == 12 {
        13
    } else {
        0
    }
}
fn legal(hand: &[Card], trick: &[Play], broken: bool, completed: u8) -> Vec<usize> {
    if hand.is_empty() {
        return vec![];
    }
    if let Some(lead) = trick.first().map(|p| p.card.suit) {
        let suited: Vec<_> = hand
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (c.suit == lead).then_some(i))
            .collect();
        if !suited.is_empty() {
            return suited;
        }
        if completed == 0 {
            let safe: Vec<_> = hand
                .iter()
                .enumerate()
                .filter_map(|(i, c)| (!is_point(*c)).then_some(i))
                .collect();
            if !safe.is_empty() {
                return safe;
            }
        }
        return (0..hand.len()).collect();
    }
    if completed == 0 {
        let clubs: Vec<_> = hand
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (*c == two_clubs()).then_some(i))
            .collect();
        if !clubs.is_empty() {
            return clubs;
        }
    }
    let nonhearts: Vec<_> = hand
        .iter()
        .enumerate()
        .filter_map(|(i, c)| (c.suit != Suit::Hearts).then_some(i))
        .collect();
    if !broken && !nonhearts.is_empty() {
        return nonhearts;
    }
    (0..hand.len()).collect()
}
fn priority_pass(c: Card) -> u16 {
    if c.suit == Suit::Spades && c.rank == 12 {
        1000
    } else if c.suit == Suit::Spades && c.rank >= 13 {
        850 + c.rank as u16
    } else if c.suit == Suit::Hearts {
        700 + c.rank as u16
    } else {
        400 + c.rank as u16
    }
}
fn pass_choices(hand: &[Card]) -> Vec<usize> {
    let mut ix: Vec<_> = (0..hand.len()).collect();
    ix.sort_by_key(|&i| std::cmp::Reverse(priority_pass(hand[i])));
    ix.truncate(3);
    ix
}
fn trick_winner(trick: &[Play]) -> usize {
    let suit = trick[0].card.suit;
    trick
        .iter()
        .filter(|p| p.card.suit == suit)
        .max_by_key(|p| p.card.rank)
        .map_or(trick[0].seat, |p| p.seat)
}
fn choose_play(hand: &[Card], indices: &[usize], trick: &[Play]) -> Option<usize> {
    if indices.is_empty() {
        return None;
    }
    let lead = trick.first().map(|p| p.card.suit);
    if lead.is_some_and(|suit| indices.iter().all(|&i| hand[i].suit != suit)) {
        // Match the legacy bot: when void, discard the highest danger card first.
        Some(*indices.iter().max_by_key(|&&i| discard_priority(hand[i]))?)
    } else {
        // Leads and suit-following plays use the lowest legal rank.
        Some(*indices.iter().min_by_key(|&&i| hand[i].rank)?)
    }
}

fn discard_priority(card: Card) -> u16 {
    if card.suit == Suit::Spades && card.rank == 12 {
        1000
    } else if card.suit == Suit::Hearts {
        800 + u16::from(card.rank)
    } else if card.suit == Suit::Spades && card.rank >= 12 {
        650 + u16::from(card.rank)
    } else {
        u16::from(card.rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_and_valid_initial_deal() {
        let a = Game::new(42);
        let b = Game::new(42);
        assert_eq!(a, b);
        assert_eq!(a.validate(), Ok(()));
    }

    #[test]
    fn fixed_seed_sorted_hand_prefix_hash_is_stable() {
        let g = Game::new(0xD00D_F00D_1234_5678);
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for card in g.hands[0].iter().take(8) {
            hash ^= (suit_idx(card.suit) * 13 + usize::from(card.rank - 2)) as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(hash, 2_076_499_505_484_092_409);
    }
    #[test]
    fn rejects_invalid_pass_and_accepts_exact_distinct_indices() {
        let mut g = Game::new(4);
        let before = g.hands.clone();
        assert!(!g.pass(&[0, 0, 1]));
        assert_eq!(before, g.hands);
        assert!(g.pass(&[0, 1, 2]));
        assert_eq!(g.phase, Phase::Playing);
        assert!(g.validate().is_ok());
    }
    #[test]
    fn complete_seeded_match_and_tie_breaking_is_stable() {
        let mut g = Game::new(987);
        let mut guard = 0;
        while g.phase != Phase::MatchComplete && guard < 2000 {
            guard += 1;
            match g.phase {
                Phase::Passing => {
                    let x = [0, 1, 2];
                    assert!(g.pass(&x));
                }
                Phase::Playing => {
                    if g.turn == 0 {
                        let i = g.legal_indices(0)[0];
                        assert!(g.play(i));
                    } else {
                        assert!(g.cpu_step());
                    }
                }
                Phase::TrickComplete => assert!(g.resolve_trick()),
                Phase::HandComplete => assert!(g.next_hand()),
                Phase::MatchComplete => {}
            }
            assert!(g.validate().is_ok(), "{:?}", g.validate());
        }
        assert_eq!(g.phase, Phase::MatchComplete);
        assert!(g.totals.iter().any(|&v| v >= 100));
    }

    fn play_to_match_end(seed: u64) -> Game {
        let mut g = Game::new(seed);
        let mut guard = 0;
        while g.phase != Phase::MatchComplete && guard < 2000 {
            guard += 1;
            match g.phase {
                Phase::Passing => assert!(g.pass(&[0, 1, 2])),
                Phase::Playing if g.turn == 0 => {
                    let legal = g.legal_indices(0);
                    assert!(!legal.is_empty());
                    assert!(g.play(legal[0]));
                }
                Phase::Playing => assert!(g.cpu_step()),
                Phase::TrickComplete => assert!(g.resolve_trick()),
                Phase::HandComplete => assert!(g.next_hand()),
                Phase::MatchComplete => unreachable!(),
            }
            assert!(g.validate().is_ok(), "seed {seed}: {:?}", g.validate());
        }
        assert_eq!(g.phase, Phase::MatchComplete, "seed {seed}");
        g
    }

    #[test]
    fn pass_directions_match_legacy_rotation_and_preserve_inventory() {
        assert_eq!(PassDirection::for_hand(1), PassDirection::Left);
        assert_eq!(PassDirection::for_hand(2), PassDirection::Right);
        assert_eq!(PassDirection::for_hand(3), PassDirection::Across);
        assert_eq!(PassDirection::for_hand(4), PassDirection::Hold);
        for (direction, targets) in [
            (PassDirection::Left, [1, 2, 3, 0]),
            (PassDirection::Right, [3, 0, 1, 2]),
            (PassDirection::Across, [2, 3, 0, 1]),
        ] {
            let mut g = Game::new(73);
            g.hand_number = match direction {
                PassDirection::Left => 1,
                PassDirection::Right => 2,
                PassDirection::Across => 3,
                PassDirection::Hold => unreachable!(),
            };
            let outgoing: [Vec<Card>; 4] = std::array::from_fn(|seat| {
                let mut indices = if seat == 0 {
                    vec![0, 1, 2]
                } else {
                    pass_choices(&g.hands[seat])
                };
                indices.sort_unstable();
                indices.into_iter().map(|i| g.hands[seat][i]).collect()
            });
            let mut expected = g.hands.clone();
            for (seat, hand) in expected.iter_mut().enumerate() {
                let selected = if seat == 0 {
                    vec![0, 1, 2]
                } else {
                    pass_choices(&g.hands[seat])
                };
                *hand = g.hands[seat]
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| (!selected.contains(&i)).then_some(*c))
                    .collect();
            }
            for seat in 0..4 {
                expected[targets[seat]].extend(outgoing[seat].iter().copied());
            }
            for hand in &mut expected {
                sort_hand(hand);
            }
            assert!(g.pass(&[0, 1, 2]));
            assert_eq!(g.hands, expected, "{} pass", direction.label());
            assert!(g.validate().is_ok());
        }
        let mut hold = Game::new(4);
        hold.hand_number = 4;
        hold.phase = Phase::Playing;
        assert_eq!(hold.pass_direction(), PassDirection::Hold);
        assert_eq!(hold.phase, Phase::Playing);
        assert!(!hold.pass(&[0, 1, 2]));
        assert!(hold.validate().is_ok());
    }

    #[test]
    fn moon_scoring_and_match_threshold_follow_legacy_rules() {
        let final_trick = [
            Card::new(2, Suit::Clubs, true),
            Card::new(3, Suit::Clubs, true),
            Card::new(4, Suit::Clubs, true),
            Card::new(5, Suit::Clubs, true),
        ];
        let mut rest: Vec<Card> = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
            .into_iter()
            .flat_map(|suit| (2..=14).map(move |rank| Card::new(rank, suit, true)))
            .filter(|c| !final_trick.contains(c))
            .collect();
        assert_eq!(rest.len(), 48);
        let mut g = Game::new(1);
        g.hands = std::array::from_fn(|_| Vec::new());
        g.trick = final_trick
            .iter()
            .enumerate()
            .map(|(seat, &card)| Play { seat, card })
            .collect();
        g.captured = std::array::from_fn(|seat| {
            if seat == 0 {
                std::mem::take(&mut rest)
            } else {
                Vec::new()
            }
        });
        g.hand_points = [26, 0, 0, 0];
        g.hand_number = 1;
        g.completed_tricks = 12;
        g.leader = 0;
        g.turn = 3;
        g.hearts_broken = true;
        g.phase = Phase::TrickComplete;
        g.totals = [0, 99, 0, 0];
        assert!(g.validate().is_ok());
        assert!(g.resolve_trick());
        assert_eq!(g.hand_points, [0, 26, 26, 26]);
        assert_eq!(g.totals, [0, 125, 26, 26]);
        assert_eq!(g.phase, Phase::MatchComplete);
        assert_eq!(g.winner(), Some(0));
        assert!(g.validate().is_ok());
    }

    #[test]
    fn all_states_validate_across_128_seeded_matches() {
        for seed in 1..=128 {
            let g = play_to_match_end(seed * 7919);
            assert!(g.totals.iter().any(|&score| score >= MATCH_LIMIT));
            assert!(g.winner().is_some());
        }
    }

    #[test]
    fn cpu_strategy_matches_legacy_danger_priorities_without_hidden_cards() {
        let hand = [
            Card::new(12, Suit::Spades, true),
            Card::new(14, Suit::Hearts, true),
            Card::new(2, Suit::Diamonds, true),
        ];
        let trick = [Play {
            seat: 0,
            card: Card::new(2, Suit::Clubs, true),
        }];
        assert_eq!(choose_play(&hand, &[0, 1, 2], &trick), Some(0));
        let followers = [
            Card::new(3, Suit::Clubs, true),
            Card::new(13, Suit::Clubs, true),
            Card::new(14, Suit::Spades, true),
        ];
        assert_eq!(choose_play(&followers, &[0, 1], &trick), Some(0));
    }
    #[test]
    fn invalid_play_does_not_mutate() {
        let mut g = Game::new(99);
        assert!(g.pass(&[0, 1, 2]));
        let h = g.hands.clone();
        assert!(!g.play(usize::MAX));
        assert_eq!(h, g.hands);
    }
    #[test]
    fn malformed_save_rejected() {
        let mut g = Game::new(1);
        g.hands[0][0] = g.hands[0][1];
        assert!(g.validate().is_err());
    }

    #[test]
    fn opening_follow_suit_and_first_trick_discard_rules() {
        let hand = [
            Card::new(2, Suit::Clubs, true),
            Card::new(14, Suit::Hearts, true),
            Card::new(3, Suit::Diamonds, true),
        ];
        assert_eq!(legal(&hand, &[], false, 0), vec![0]);
        let first_lead = [Play {
            seat: 0,
            card: Card::new(7, Suit::Clubs, true),
        }];
        assert_eq!(legal(&hand, &first_lead, false, 0), vec![0]);
        let void_clubs = [
            Card::new(14, Suit::Hearts, true),
            Card::new(3, Suit::Diamonds, true),
        ];
        assert_eq!(legal(&void_clubs, &first_lead, false, 0), vec![1]);
        assert_eq!(legal(&void_clubs, &first_lead, false, 1), vec![0, 1]);
    }

    #[test]
    fn hearts_lead_unlocks_only_after_broken() {
        let hand = [
            Card::new(4, Suit::Hearts, true),
            Card::new(8, Suit::Spades, true),
        ];
        assert_eq!(legal(&hand, &[], false, 2), vec![1]);
        assert_eq!(legal(&hand, &[], true, 2), vec![0, 1]);
        assert_eq!(points(Card::new(12, Suit::Spades, true)), 13);
    }
}
