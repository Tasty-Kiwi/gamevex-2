use serde::{Deserialize, Serialize};

pub const BAR: u8 = 24;
pub const OFF: u8 = 25;
const CHECKERS: u8 = 15;
const MAX_ELAPSED_MS: u64 = 7 * 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const fn index(self) -> usize {
        match self {
            Self::White => 0,
            Self::Black => 1,
        }
    }
    pub const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Hard,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub color: Option<Color>,
    pub count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub points: [Point; 24],
    pub bar: [u8; 2],
    pub off: [u8; 2],
}

impl Default for Board {
    fn default() -> Self {
        let mut b = Self {
            points: [Point::default(); 24],
            bar: [0; 2],
            off: [0; 2],
        };
        for (i, c, n) in [
            (23, Color::White, 2),
            (12, Color::White, 5),
            (7, Color::White, 3),
            (5, Color::White, 5),
            (0, Color::Black, 2),
            (11, Color::Black, 5),
            (16, Color::Black, 3),
            (18, Color::Black, 5),
        ] {
            b.points[i] = Point {
                color: Some(c),
                count: n,
            };
        }
        b
    }
}

impl Board {
    pub fn validate(&self) -> Result<(), String> {
        let mut totals = [0u16; 2];
        for p in self.points {
            if p.count == 0 && p.color.is_some() || p.count > 0 && p.color.is_none() {
                return Err("point color/count mismatch".into());
            }
            if let Some(c) = p.color {
                totals[c.index()] += p.count as u16;
            }
        }
        for c in [Color::White, Color::Black] {
            let i = c.index();
            totals[i] += self.bar[i] as u16 + self.off[i] as u16;
            if totals[i] != CHECKERS as u16 {
                return Err("each side must have exactly 15 checkers".into());
            }
        }
        Ok(())
    }
    pub fn pip_count(&self, c: Color) -> u16 {
        let mut total = self.bar[c.index()] as u16 * 25;
        for (i, p) in self.points.iter().enumerate() {
            if p.color == Some(c) {
                total += p.count as u16
                    * if c == Color::White {
                        (i + 1) as u16
                    } else {
                        (24 - i) as u16
                    };
            }
        }
        total
    }
    pub fn all_in_home(&self, c: Color) -> bool {
        if self.bar[c.index()] > 0 {
            return false;
        }
        self.points
            .iter()
            .enumerate()
            .all(|(i, p)| p.color != Some(c) || if c == Color::White { i < 6 } else { i >= 18 })
    }
    pub fn winner(&self) -> Option<Color> {
        if self.off[0] >= CHECKERS {
            Some(Color::White)
        } else if self.off[1] >= CHECKERS {
            Some(Color::Black)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub die: u8,
    pub is_hit: bool,
}

fn direction(c: Color) -> i16 {
    if c == Color::White { -1 } else { 1 }
}
fn single_moves(b: &Board, die: u8, c: Color) -> Vec<Move> {
    if !(1..=6).contains(&die) {
        return vec![];
    }
    let mut out = Vec::new();
    let opp = c.opponent();
    if b.bar[c.index()] > 0 {
        let dest = if c == Color::White { 24 - die } else { die - 1 };
        let p = b.points[dest as usize];
        if p.color != Some(opp) || p.count == 1 {
            out.push(Move {
                from: BAR,
                to: dest,
                die,
                is_hit: p.color == Some(opp),
            });
        }
        return out;
    }
    let home = b.all_in_home(c);
    for i in 0..24 {
        let p = b.points[i];
        if p.color != Some(c) {
            continue;
        }
        let dest = i as i16 + direction(c) * die as i16;
        if (0..24).contains(&dest) {
            let q = b.points[dest as usize];
            if q.color != Some(opp) || q.count == 1 {
                out.push(Move {
                    from: i as u8,
                    to: dest as u8,
                    die,
                    is_hit: q.color == Some(opp),
                });
            }
        } else if home {
            let exact = (c == Color::White && dest == -1) || (c == Color::Black && dest == 24);
            let overshoot = if c == Color::White {
                dest < -1 && !(i + 1..24).any(|j| b.points[j].color == Some(c))
            } else {
                dest > 24 && !(0..i).any(|j| b.points[j].color == Some(c))
            };
            if exact || overshoot {
                out.push(Move {
                    from: i as u8,
                    to: OFF,
                    die,
                    is_hit: false,
                });
            }
        }
    }
    out
}

pub fn apply_move(b: &Board, m: Move, c: Color) -> Board {
    let mut n = b.clone();
    let opp = c.opponent();
    if m.from == BAR {
        n.bar[c.index()] -= 1;
    } else {
        let p = &mut n.points[m.from as usize];
        p.count -= 1;
        if p.count == 0 {
            p.color = None;
        }
    }
    if m.to == OFF {
        n.off[c.index()] += 1;
    } else {
        let p = &mut n.points[m.to as usize];
        if m.is_hit {
            n.bar[opp.index()] += p.count;
            p.count = 0;
            p.color = None;
        }
        p.count += 1;
        p.color = Some(c);
    }
    n
}

pub fn legal_sequences(board: &Board, dice: &[u8], color: Color) -> Vec<Vec<Move>> {
    fn walk(b: &Board, dice: &[u8], c: Color, seq: &mut Vec<Move>, out: &mut Vec<Vec<Move>>) {
        if dice.is_empty() {
            out.push(seq.clone());
            return;
        }
        let mut tried = [false; 7];
        let mut found = false;
        for (ix, &die) in dice.iter().enumerate() {
            if die > 6 || tried[die as usize] {
                continue;
            }
            tried[die as usize] = true;
            for m in single_moves(b, die, c) {
                found = true;
                let n = apply_move(b, m, c);
                let mut rest = dice.to_vec();
                rest.remove(ix);
                seq.push(m);
                walk(&n, &rest, c, seq, out);
                seq.pop();
            }
        }
        if !found {
            out.push(seq.clone());
        }
    }
    if dice.is_empty() {
        return vec![vec![]];
    }
    if dice.len() > 4 || dice.iter().any(|d| !(1..=6).contains(d)) {
        return vec![vec![]];
    }
    let mut results = Vec::new();
    walk(board, dice, color, &mut Vec::new(), &mut results);
    let max = results.iter().map(Vec::len).max().unwrap_or(0);
    results.retain(|s| s.len() == max);
    if max == 1 && dice.len() == 2 && dice[0] != dice[1] {
        let high = dice[0].max(dice[1]);
        if results.iter().any(|s| s[0].die == high) {
            results.retain(|s| s[0].die == high);
        }
    }
    results.sort_by_key(|s| s.iter().map(|m| (m.from, m.to, m.die)).collect::<Vec<_>>());
    results.dedup();
    if results.is_empty() {
        vec![vec![]]
    } else {
        results
    }
}

fn evaluate(b: &Board, c: Color, d: Difficulty) -> f64 {
    let o = c.opponent();
    let mut s = b.off[c.index()] as f64 * 100.
        - b.off[o.index()] as f64 * 80.
        - b.bar[c.index()] as f64 * 120.
        + b.bar[o.index()] as f64 * if d == Difficulty::Hard { 100. } else { 60. };
    s -= b.pip_count(c) as f64 * 2.;
    s += b.pip_count(o) as f64 * if d == Difficulty::Hard { 1.5 } else { 1. };
    for (i, p) in b.points.iter().enumerate() {
        if p.color != Some(c) {
            continue;
        }
        if p.count == 1 {
            let opp_home = if c == Color::White { i >= 18 } else { i <= 5 };
            s -= if opp_home {
                if d == Difficulty::Hard { 40. } else { 20. }
            } else if d == Difficulty::Hard {
                20.
            } else {
                10.
            };
        }
        if p.count >= 2 {
            let home = if c == Color::White { i < 6 } else { i >= 18 };
            s += if home { 15. } else { 8. };
            if d == Difficulty::Hard {
                let adjacent = if c == Color::White {
                    i > 0 && b.points[i - 1].color == Some(c) && b.points[i - 1].count >= 2
                } else {
                    i < 23 && b.points[i + 1].color == Some(c) && b.points[i + 1].count >= 2
                };
                if adjacent {
                    s += 12.;
                }
            }
        }
    }
    s
}

/// Deterministic seed-based replacement for the oracle's random tie selector.
/// The same board, dice, color, difficulty, and seed always select the same sequence.
pub fn choose_sequence(
    board: &Board,
    dice: &[u8],
    color: Color,
    difficulty: Difficulty,
    seed: u64,
) -> Vec<Move> {
    let seqs = legal_sequences(board, dice, color);
    if seqs.len() < 2 {
        return seqs.into_iter().next().unwrap_or_default();
    }
    let scores: Vec<f64> = seqs
        .iter()
        .map(|s| {
            s.iter()
                .fold(board.clone(), |b, &m| apply_move(&b, m, color))
        })
        .map(|b| evaluate(&b, color, difficulty))
        .collect();
    let best = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut state = if seed == 0 { 0x9e3779b97f4a7c15 } else { seed };
    let mut random = || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        (state.wrapping_mul(0x2545f4914f6cdd1d) >> 11) as f64 / (1u64 << 53) as f64
    };
    let selected = if difficulty == Difficulty::Easy && seqs.len() > 2 && random() < 0.3 {
        (random() * seqs.len() as f64) as usize
    } else {
        let candidates: Vec<usize> = scores
            .iter()
            .enumerate()
            .filter_map(|(i, &s)| (s == best).then_some(i))
            .collect();
        candidates[(random() * candidates.len() as f64) as usize]
    };
    seqs[selected].clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    AwaitRoll,
    Moving,
    Won,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub board: Board,
    pub turn: Color,
    pub phase: Phase,
    pub dice: Vec<u8>,
    pub rolled: Vec<u8>,
    pub difficulty: Difficulty,
    pub winner: Option<Color>,
    pub elapsed_ms: u64,
    pub rng_state: u64,
    pub turn_number: u32,
}

impl Game {
    pub fn new(seed: u64, difficulty: Difficulty) -> Self {
        Self {
            board: Board::default(),
            turn: Color::White,
            phase: Phase::AwaitRoll,
            dice: vec![],
            rolled: vec![],
            difficulty,
            winner: None,
            elapsed_ms: 0,
            rng_state: if seed == 0 { 0x9e3779b97f4a7c15 } else { seed },
            turn_number: 0,
        }
    }
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut out = Vec::new();
        for s in legal_sequences(&self.board, &self.dice, self.turn) {
            if let Some(&m) = s.first()
                && !out.contains(&m)
            {
                out.push(m);
            }
        }
        out
    }
    fn random_u32(&mut self) -> u32 {
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        (x.wrapping_mul(0x2545f4914f6cdd1d) >> 32) as u32
    }
    pub fn roll(&mut self) -> bool {
        if self.phase != Phase::AwaitRoll || self.winner.is_some() {
            return false;
        }
        let a = (self.random_u32() % 6 + 1) as u8;
        let b = (self.random_u32() % 6 + 1) as u8;
        self.rolled = if a == b { vec![a; 4] } else { vec![a, b] };
        self.dice = self.rolled.clone();
        self.phase = Phase::Moving;
        if self.legal_moves().is_empty() {
            self.end_turn();
        }
        true
    }
    fn end_turn(&mut self) {
        if let Some(w) = self.board.winner() {
            self.winner = Some(w);
            self.phase = Phase::Won;
            self.dice.clear();
            return;
        }
        self.turn = self.turn.opponent();
        self.phase = Phase::AwaitRoll;
        self.dice.clear();
        self.turn_number = self.turn_number.saturating_add(1);
    }
    pub fn apply(&mut self, m: Move) -> bool {
        if self.phase != Phase::Moving || !self.legal_moves().contains(&m) {
            return false;
        }
        let Some(i) = self.dice.iter().position(|&d| d == m.die) else {
            return false;
        };
        let next_board = apply_move(&self.board, m, self.turn);
        let mut next_dice = self.dice.clone();
        next_dice.remove(i);
        self.board = next_board;
        self.dice = next_dice;
        if let Some(w) = self.board.winner() {
            self.winner = Some(w);
            self.phase = Phase::Won;
            self.dice.clear();
            return true;
        }
        if self.dice.is_empty() || self.legal_moves().is_empty() {
            self.end_turn();
        }
        true
    }
    pub fn validate(&self) -> Result<(), String> {
        self.board.validate()?;
        if self.board.off[0] >= CHECKERS && self.board.off[1] >= CHECKERS {
            return Err("both sides cannot have won".into());
        }
        if self.elapsed_ms > MAX_ELAPSED_MS {
            return Err("elapsed time exceeds seven days".into());
        }
        if self.rng_state == 0 {
            return Err("invalid random state".into());
        }
        if !self.rolled.is_empty() && self.rolled.len() != 2 && self.rolled.len() != 4 {
            return Err("invalid rolled dice shape".into());
        }
        if self.rolled.iter().any(|d| !(1..=6).contains(d))
            || self.dice.iter().any(|d| !(1..=6).contains(d))
            || self.dice.len() > 4
        {
            return Err("invalid dice value or count".into());
        }
        if self.rolled.len() == 2 && self.rolled[0] == self.rolled[1]
            || self.rolled.len() == 4 && (self.rolled.iter().any(|d| *d != self.rolled[0]))
        {
            return Err("invalid original dice shape".into());
        }
        if self.dice.len() > self.rolled.len() {
            return Err("remaining dice exceed original roll".into());
        }
        let mut remaining = [0u8; 7];
        let mut original = [0u8; 7];
        for &d in &self.dice {
            remaining[d as usize] += 1;
        }
        for &d in &self.rolled {
            original[d as usize] += 1;
        }
        if (1..=6).any(|d| remaining[d] > original[d]) {
            return Err("remaining dice do not match original roll".into());
        }
        match self.phase {
            Phase::AwaitRoll => {
                if !self.dice.is_empty() || self.winner.is_some() || self.board.winner().is_some() {
                    return Err("invalid await-roll state".into());
                }
            }
            Phase::Moving => {
                if self.dice.is_empty()
                    || self.winner.is_some()
                    || self.board.winner().is_some()
                    || !self
                        .dice
                        .iter()
                        .any(|&d| !single_moves(&self.board, d, self.turn).is_empty())
                {
                    return Err("invalid moving state".into());
                }
            }
            Phase::Won => {
                if self.winner != self.board.winner()
                    || self.winner.is_none()
                    || self.turn != self.winner.unwrap()
                    || !self.dice.is_empty()
                {
                    return Err("invalid won state".into());
                }
            }
        }
        Ok(())
    }
    pub fn advance_time(&mut self, ms: u64) {
        if self.phase != Phase::Won && (self.turn_number > 0 || self.phase == Phase::Moving) {
            self.elapsed_ms = self.elapsed_ms.saturating_add(ms).min(MAX_ELAPSED_MS);
        }
    }
    pub fn finished(&self) -> bool {
        self.phase == Phase::Won
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sparse(
        white: &[(usize, u8)],
        black: &[(usize, u8)],
        white_bar: u8,
        black_bar: u8,
        white_off: u8,
        black_off: u8,
    ) -> Board {
        let mut b = Board {
            points: [Point::default(); 24],
            bar: [white_bar, black_bar],
            off: [white_off, black_off],
        };
        for &(i, n) in white {
            b.points[i] = Point {
                color: Some(Color::White),
                count: n,
            };
        }
        for &(i, n) in black {
            b.points[i] = Point {
                color: Some(Color::Black),
                count: n,
            };
        }
        b
    }

    #[test]
    fn standard_position_and_conservation_validate() {
        let b = Board::default();
        assert!(b.validate().is_ok());
        assert_eq!(b.pip_count(Color::White), 167);
        assert_eq!(b.pip_count(Color::Black), 167);
        assert_eq!(
            b.points[23],
            Point {
                color: Some(Color::White),
                count: 2
            }
        );
        assert_eq!(
            b.points[18],
            Point {
                color: Some(Color::Black),
                count: 5
            }
        );
    }

    #[test]
    fn bar_entry_precedes_all_other_checkers_and_hits_a_blot() {
        let b = sparse(&[(10, 1)], &[(23, 1)], 1, 0, 13, 14);
        let seqs = legal_sequences(&b, &[1, 2], Color::White);
        assert!(!seqs.is_empty());
        assert!(
            seqs.iter()
                .all(|s| s.first().is_some_and(|m| m.from == BAR))
        );
        let hit = seqs
            .iter()
            .flatten()
            .find(|m| m.to == 23 && m.is_hit)
            .copied()
            .unwrap();
        let next = apply_move(&b, hit, Color::White);
        assert_eq!(next.bar[1], 1);
        assert_eq!(
            next.points[23],
            Point {
                color: Some(Color::White),
                count: 1
            }
        );
        assert!(next.validate().is_ok());
    }

    #[test]
    fn higher_die_required_when_only_one_of_two_dice_can_be_played() {
        let b = sparse(&[(5, 15)], &[(2, 2), (4, 2)], 0, 0, 0, 11);
        assert!(b.validate().is_ok());
        let seqs = legal_sequences(&b, &[1, 2], Color::White);
        assert!(!seqs.is_empty());
        assert!(seqs.iter().all(|s| s.len() == 1 && s[0].die == 2));
    }

    #[test]
    fn doubles_use_as_many_dice_as_possible() {
        let b = sparse(&[(23, 15)], &[], 0, 0, 0, 15);
        let seqs = legal_sequences(&b, &[1, 1, 1, 1], Color::White);
        assert!(seqs.iter().all(|s| s.len() == 4));
        assert!(seqs.iter().flatten().all(|m| m.die == 1));
    }

    #[test]
    fn exact_and_overshoot_bearing_off_obey_farther_checker_rule_both_directions() {
        let white = sparse(&[(0, 14), (2, 1)], &[], 0, 0, 0, 15);
        let white_moves = legal_sequences(&white, &[1], Color::White);
        assert!(
            white_moves
                .iter()
                .flatten()
                .any(|m| m.from == 0 && m.to == OFF)
        );
        assert!(
            !white_moves
                .iter()
                .flatten()
                .any(|m| m.from == 2 && m.to == OFF)
        );
        let white_clear = sparse(&[(0, 14), (2, 1)], &[], 0, 0, 0, 15);
        let mut white_clear = white_clear;
        white_clear.points[0] = Point::default();
        white_clear.points[2] = Point {
            color: Some(Color::White),
            count: 15,
        };
        white_clear.off[0] = 0;
        assert!(
            legal_sequences(&white_clear, &[3], Color::White)
                .iter()
                .flatten()
                .any(|m| m.to == OFF)
        );

        let black = sparse(&[], &[(21, 1), (23, 14)], 0, 0, 15, 0);
        assert!(
            !legal_sequences(&black, &[3], Color::Black)
                .iter()
                .flatten()
                .any(|m| m.from == 23 && m.to == OFF)
        );
        let black_clear = sparse(&[], &[(21, 15)], 0, 0, 15, 0);
        assert!(
            legal_sequences(&black_clear, &[3], Color::Black)
                .iter()
                .flatten()
                .any(|m| m.to == OFF)
        );
    }

    #[test]
    fn illegal_game_move_is_atomic_and_blocked_roll_passes_turn() {
        let mut g = Game::new(91, Difficulty::Hard);
        assert!(g.roll());
        let before = g.clone();
        assert!(!g.apply(Move {
            from: 4,
            to: 8,
            die: 4,
            is_hit: false
        }));
        assert_eq!(g, before);

        let mut blocked = Game::new(123, Difficulty::Hard);
        blocked.board = sparse(
            &[],
            &[(18, 2), (19, 2), (20, 2), (21, 2), (22, 2), (23, 2)],
            1,
            0,
            14,
            3,
        );
        assert!(blocked.board.validate().is_ok());
        assert!(blocked.roll());
        assert_eq!(blocked.turn, Color::Black);
        assert_eq!(blocked.phase, Phase::AwaitRoll);
        assert!(blocked.dice.is_empty());
        assert_eq!(blocked.rolled.len(), 2);
    }

    #[test]
    fn hard_cpu_completes_one_hundred_seeded_games_with_valid_transitions() {
        for seed in 1..=100 {
            let mut g = Game::new(seed, Difficulty::Hard);
            let mut actions = 0usize;
            while !g.finished() && actions < 20_000 {
                if g.phase == Phase::AwaitRoll {
                    assert!(g.roll());
                } else {
                    let seq =
                        choose_sequence(&g.board, &g.dice, g.turn, Difficulty::Hard, g.rng_state);
                    assert!(!seq.is_empty(), "moving state has no sequence, seed={seed}");
                    for m in seq {
                        assert!(g.apply(m), "CPU selected illegal move, seed={seed}");
                        actions += 1;
                        assert!(
                            g.validate().is_ok(),
                            "invalid intermediate state, seed={seed}"
                        );
                        if g.finished() || g.phase != Phase::Moving {
                            break;
                        }
                    }
                }
                actions += 1;
            }
            assert!(
                g.finished(),
                "game did not finish within action cap, seed={seed}"
            );
            assert_eq!(g.winner, g.board.winner());
            assert!(g.validate().is_ok());
        }
    }

    #[test]
    fn malformed_save_states_are_rejected_and_search_stays_bounded_by_four_dice() {
        let mut g = Game::new(2, Difficulty::Easy);
        g.elapsed_ms = 7 * 24 * 60 * 60 * 1000 + 1;
        assert!(g.validate().is_err());
        let mut g = Game::new(2, Difficulty::Easy);
        g.rolled = vec![2, 2];
        assert!(g.validate().is_err());
        let mut g = Game::new(2, Difficulty::Easy);
        g.board.off = [15, 15];
        g.phase = Phase::Won;
        g.winner = Some(Color::White);
        assert!(g.validate().is_err());
        assert_eq!(
            legal_sequences(&Board::default(), &[1, 2, 3, 4, 5], Color::White),
            vec![vec![]]
        );
        assert_eq!(
            legal_sequences(&Board::default(), &[0], Color::White),
            vec![vec![]]
        );
        let seqs = legal_sequences(&Board::default(), &[6, 6, 6, 6], Color::White);
        assert!(seqs.len() < 100_000);
        assert!(seqs.iter().all(|s| s.len() <= 4));
    }
}
