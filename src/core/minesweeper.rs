use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// The timer is capped at seven days to keep corrupt or adversarial saves bounded.
pub const MAX_ELAPSED_MS: u64 = 7 * 24 * 60 * 60 * 1_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
}

impl Difficulty {
    pub const ALL: [Self; 3] = [Self::Beginner, Self::Intermediate, Self::Expert];

    pub const fn config(self) -> (usize, usize, usize) {
        match self {
            Self::Beginner => (9, 9, 10),
            Self::Intermediate => (16, 16, 40),
            Self::Expert => (30, 16, 99),
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Beginner => "9 × 9 · 10 mines",
            Self::Intermediate => "16 × 16 · 40 mines",
            Self::Expert => "30 × 16 · 99 mines",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Ready,
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub mine: bool,
    pub adjacent: u8,
    pub revealed: bool,
    pub flagged: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub difficulty: Difficulty,
    pub cells: Vec<Cell>,
    pub status: Status,
    pub elapsed_ms: u64,
    pub seed: u64,
    pub first_reveal: Option<usize>,
    pub exploded: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Reveal(usize),
    ToggleFlag(usize),
    Chord(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainEvent {
    Started,
    Revealed(usize),
    Flagged(usize),
    Won,
    Lost,
}

impl Game {
    pub fn new(difficulty: Difficulty, seed: u64) -> Self {
        let (cols, rows, _) = difficulty.config();
        Self {
            difficulty,
            cells: vec![Cell::default(); cols * rows],
            status: Status::Ready,
            elapsed_ms: 0,
            seed,
            first_reveal: None,
            exploded: None,
        }
    }

    pub const fn cols(&self) -> usize {
        self.difficulty.config().0
    }
    pub const fn rows(&self) -> usize {
        self.difficulty.config().1
    }
    pub const fn mine_count(&self) -> usize {
        self.difficulty.config().2
    }

    pub fn remaining_mines(&self) -> i32 {
        self.mine_count() as i32 - self.cells.iter().filter(|c| c.flagged).count() as i32
    }

    pub fn neighbors(&self, index: usize) -> Vec<usize> {
        if index >= self.cells.len() {
            return Vec::new();
        }
        let (row, col) = (index / self.cols(), index % self.cols());
        let mut result = Vec::with_capacity(8);
        for dr in -1isize..=1 {
            for dc in -1isize..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let r = row as isize + dr;
                let c = col as isize + dc;
                if r >= 0 && c >= 0 && r < self.rows() as isize && c < self.cols() as isize {
                    result.push(r as usize * self.cols() + c as usize);
                }
            }
        }
        result
    }

    pub fn advance_elapsed(&mut self, delta_ms: u64, active: bool) {
        if active && self.status == Status::Playing {
            self.elapsed_ms = self.elapsed_ms.saturating_add(delta_ms).min(MAX_ELAPSED_MS);
        }
    }

    pub fn command(&mut self, command: Command) -> Vec<DomainEvent> {
        if matches!(self.status, Status::Won | Status::Lost) {
            return Vec::new();
        }
        match command {
            Command::ToggleFlag(i) => {
                if i >= self.cells.len() || self.cells[i].revealed {
                    return Vec::new();
                }
                self.cells[i].flagged = !self.cells[i].flagged;
                vec![DomainEvent::Flagged(i)]
            }
            Command::Reveal(i) => {
                if i >= self.cells.len() || self.cells[i].revealed || self.cells[i].flagged {
                    return Vec::new();
                }
                let mut events = Vec::new();
                if self.status == Status::Ready {
                    self.place_mines(i);
                    self.first_reveal = Some(i);
                    self.status = Status::Playing;
                    events.push(DomainEvent::Started);
                }
                self.reveal_region(i, &mut events);
                self.finish_if_won(&mut events);
                events
            }
            Command::Chord(i) => self.chord(i),
        }
    }

    fn place_mines(&mut self, safe: usize) {
        let safe_set: Vec<bool> = (0..self.cells.len())
            .map(|i| i == safe || self.neighbors(safe).contains(&i))
            .collect();
        let mut candidates: Vec<usize> = (0..self.cells.len()).filter(|&i| !safe_set[i]).collect();
        let mut rng = self.seed;
        for n in (1..candidates.len()).rev() {
            let j = (next_random(&mut rng) as usize) % (n + 1);
            candidates.swap(n, j);
        }
        for &i in candidates.iter().take(self.mine_count()) {
            self.cells[i].mine = true;
        }
        self.recompute_adjacency();
    }

    fn recompute_adjacency(&mut self) {
        for i in 0..self.cells.len() {
            self.cells[i].adjacent = if self.cells[i].mine {
                0
            } else {
                self.neighbors(i)
                    .into_iter()
                    .filter(|&n| self.cells[n].mine)
                    .count() as u8
            };
        }
    }

    fn reveal_region(&mut self, start: usize, events: &mut Vec<DomainEvent>) {
        if start >= self.cells.len() || self.cells[start].revealed || self.cells[start].flagged {
            return;
        }
        if self.cells[start].mine {
            self.exploded = Some(start);
            for i in 0..self.cells.len() {
                if self.cells[i].mine && !self.cells[i].revealed {
                    self.cells[i].revealed = true;
                    events.push(DomainEvent::Revealed(i));
                }
            }
            self.status = Status::Lost;
            events.push(DomainEvent::Lost);
            return;
        }
        let mut queue = VecDeque::from([start]);
        while let Some(i) = queue.pop_front() {
            if self.cells[i].revealed || self.cells[i].flagged || self.cells[i].mine {
                continue;
            }
            self.cells[i].revealed = true;
            events.push(DomainEvent::Revealed(i));
            if self.cells[i].adjacent == 0 {
                queue.extend(
                    self.neighbors(i)
                        .into_iter()
                        .filter(|&n| !self.cells[n].revealed && !self.cells[n].flagged),
                );
            }
        }
    }

    fn chord(&mut self, i: usize) -> Vec<DomainEvent> {
        if i >= self.cells.len() || !self.cells[i].revealed || self.cells[i].adjacent == 0 {
            return Vec::new();
        }
        let neighbors = self.neighbors(i);
        if neighbors.iter().filter(|&&n| self.cells[n].flagged).count()
            != self.cells[i].adjacent as usize
        {
            return Vec::new();
        }
        let mut events = Vec::new();
        for n in neighbors {
            if !self.cells[n].revealed && !self.cells[n].flagged {
                self.reveal_region(n, &mut events);
                if self.status == Status::Lost {
                    return events;
                }
            }
        }
        self.finish_if_won(&mut events);
        events
    }

    fn finish_if_won(&mut self, events: &mut Vec<DomainEvent>) {
        if self.status == Status::Playing && self.cells.iter().all(|c| c.mine || c.revealed) {
            self.status = Status::Won;
            events.push(DomainEvent::Won);
        }
    }

    /// Checks that a persisted or externally constructed game is a possible game state.
    pub fn validate(&self) -> Result<(), String> {
        let (cols, rows, mines) = self.difficulty.config();
        if self.cells.len() != cols * rows {
            return Err("cell count does not match difficulty".into());
        }
        if self.elapsed_ms > MAX_ELAPSED_MS {
            return Err("elapsed time exceeds limit".into());
        }
        if self.cells.iter().filter(|c| c.mine).count() != mines && self.status != Status::Ready {
            return Err("placed mine count does not match difficulty".into());
        }
        if self.status == Status::Ready {
            if self.elapsed_ms != 0
                || self.first_reveal.is_some()
                || self.exploded.is_some()
                || self
                    .cells
                    .iter()
                    .any(|c| c.mine || c.adjacent != 0 || c.revealed)
            {
                return Err("ready game contains placed or revealed state".into());
            }
        } else {
            let first = self
                .first_reveal
                .ok_or("started game is missing first reveal")?;
            if first >= self.cells.len() {
                return Err("first reveal is out of bounds".into());
            }
            if !self.cells[first].revealed {
                return Err("first reveal is not revealed".into());
            }
            if self.cells[first].mine || self.neighbors(first).iter().any(|&n| self.cells[n].mine) {
                return Err("first reveal is not protected".into());
            }
            for i in 0..self.cells.len() {
                let expected = if self.cells[i].mine {
                    0
                } else {
                    self.neighbors(i)
                        .iter()
                        .filter(|&&n| self.cells[n].mine)
                        .count() as u8
                };
                if self.cells[i].adjacent != expected {
                    return Err(format!("incorrect adjacency at cell {i}"));
                }
            }
        }
        match self.status {
            Status::Ready | Status::Playing | Status::Won => {
                if self.exploded.is_some() {
                    return Err("non-lost game has exploded cell".into());
                }
            }
            Status::Lost => {
                let i = self.exploded.ok_or("lost game is missing exploded cell")?;
                if i >= self.cells.len() || !self.cells[i].mine || !self.cells[i].revealed {
                    return Err("exploded cell is not a revealed mine".into());
                }
                if self.cells.iter().any(|c| c.mine && !c.revealed) {
                    return Err("lost game has hidden mines".into());
                }
            }
        }
        if self.status == Status::Won && self.cells.iter().any(|c| !c.mine && !c.revealed) {
            return Err("won game has unrevealed safe cells".into());
        }
        if self.status == Status::Playing && self.cells.iter().all(|c| c.mine || c.revealed) {
            return Err("playing game is already won".into());
        }
        if self
            .cells
            .iter()
            .any(|c| c.revealed && c.flagged && !(self.status == Status::Lost && c.mine))
        {
            return Err("only a revealed mine in a lost game may remain flagged".into());
        }
        if self.status != Status::Lost && self.cells.iter().any(|c| c.mine && c.revealed) {
            return Err("non-lost game reveals a mine".into());
        }
        Ok(())
    }
}

fn next_random(state: &mut u64) -> u64 {
    // SplitMix64 produces a deterministic stream for every persisted seed, including zero.
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_reveal_is_safe_and_board_counts_match_for_many_seeds() {
        for difficulty in Difficulty::ALL {
            for seed in 0..32 {
                let mut game = Game::new(difficulty, seed);
                let first = game.cells.len() / 2;
                game.command(Command::Reveal(first));
                assert_eq!(
                    game.cells.iter().filter(|c| c.mine).count(),
                    game.mine_count()
                );
                assert!(!game.cells[first].mine);
                assert!(game.neighbors(first).iter().all(|&i| !game.cells[i].mine));
                assert!(
                    game.validate().is_ok(),
                    "{difficulty:?} seed {seed}: {:?}",
                    game.validate()
                );
                for i in 0..game.cells.len() {
                    if !game.cells[i].mine {
                        assert_eq!(
                            game.cells[i].adjacent as usize,
                            game.neighbors(i)
                                .iter()
                                .filter(|&&n| game.cells[n].mine)
                                .count()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn zero_seed_is_deterministic() {
        let mut first = Game::new(Difficulty::Expert, 0);
        let mut second = Game::new(Difficulty::Expert, 0);
        first.command(Command::Reveal(240));
        second.command(Command::Reveal(240));
        assert_eq!(first, second);
        assert!(first.validate().is_ok());
    }

    #[test]
    fn flags_before_first_reveal_are_preserved_and_flood_does_not_reveal_them() {
        let mut game = Game::new(Difficulty::Beginner, 90);
        game.command(Command::ToggleFlag(0));
        game.command(Command::Reveal(40));
        assert!(game.cells[0].flagged);
        assert!(!game.cells[0].revealed);
        assert!(game.cells.iter().any(|c| c.revealed));
        assert_eq!(game.remaining_mines(), 9);
    }

    #[test]
    fn correct_chord_reveals_safe_neighbors_and_wrong_flag_can_lose() {
        let mut game = Game::new(Difficulty::Beginner, 17);
        game.command(Command::Reveal(40));
        let anchor = (0..game.cells.len())
            .find(|&i| {
                game.cells[i].revealed
                    && game.cells[i].adjacent > 0
                    && game.neighbors(i).iter().any(|&n| game.cells[n].mine)
            })
            .unwrap();
        let adjacent = game.neighbors(anchor);
        let mines: Vec<_> = adjacent
            .iter()
            .copied()
            .filter(|&i| game.cells[i].mine)
            .collect();
        let mut safe: Vec<_> = adjacent
            .iter()
            .copied()
            .filter(|&i| !game.cells[i].mine && !game.cells[i].revealed)
            .collect();
        for &i in &mines {
            game.command(Command::ToggleFlag(i));
        }
        let correct = game.command(Command::Chord(anchor));
        assert!(
            correct
                .iter()
                .any(|e| matches!(e, DomainEvent::Revealed(_)))
        );
        assert_eq!(game.status, Status::Playing);

        // On a fresh identical board, replace one true flag with a safe one.
        let mut wrong = Game::new(Difficulty::Beginner, 17);
        wrong.command(Command::Reveal(40));
        for &i in &mines {
            wrong.command(Command::ToggleFlag(i));
        }
        // The same deterministic board and anchor are present.
        let wa = anchor;
        if let Some(s) = safe.pop() {
            wrong.command(Command::ToggleFlag(mines[0]));
            wrong.command(Command::ToggleFlag(s));
            let events = wrong.command(Command::Chord(wa));
            assert_eq!(wrong.status, Status::Lost);
            assert!(events.contains(&DomainEvent::Lost));
            assert!(wrong.validate().is_ok());
        }
    }

    #[test]
    fn loss_and_win_invariants_and_terminal_commands() {
        let mut lost = Game::new(Difficulty::Beginner, 5);
        lost.command(Command::Reveal(40));
        let mine = lost.cells.iter().position(|c| c.mine).unwrap();
        assert!(
            lost.command(Command::Reveal(mine))
                .contains(&DomainEvent::Lost)
        );
        assert_eq!(lost.status, Status::Lost);
        assert!(lost.validate().is_ok());
        let snapshot = lost.clone();
        lost.advance_elapsed(5_000, true);
        assert!(lost.command(Command::ToggleFlag(1)).is_empty());
        assert_eq!(lost, snapshot);

        let mut won = Game::new(Difficulty::Beginner, 12);
        won.command(Command::Reveal(40));
        for i in 0..won.cells.len() {
            if !won.cells[i].mine && !won.cells[i].revealed {
                won.command(Command::Reveal(i));
            }
        }
        assert_eq!(won.status, Status::Won);
        assert!(won.validate().is_ok());
        let elapsed = won.elapsed_ms;
        won.advance_elapsed(5_000, true);
        assert_eq!(won.elapsed_ms, elapsed);
        assert!(won.command(Command::Reveal(0)).is_empty());
    }

    #[test]
    fn correctly_flagged_mines_remain_valid_when_loss_reveals_all_mines() {
        let mut game = Game::new(Difficulty::Beginner, 5);
        game.command(Command::Reveal(40));
        let mines: Vec<_> = game
            .cells
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.mine.then_some(i))
            .collect();
        for &i in &mines[..2] {
            game.command(Command::ToggleFlag(i));
        }
        game.command(Command::Reveal(mines[2]));
        assert_eq!(game.status, Status::Lost);
        assert!(
            mines[..2]
                .iter()
                .all(|&i| game.cells[i].revealed && game.cells[i].flagged)
        );
        assert!(game.validate().is_ok());
    }

    #[test]
    fn elapsed_time_advances_only_during_active_play_and_saturates() {
        let mut game = Game::new(Difficulty::Beginner, 3);
        game.advance_elapsed(100, true);
        assert_eq!(game.elapsed_ms, 0);
        game.command(Command::Reveal(40));
        game.advance_elapsed(1_250, false);
        assert_eq!(game.elapsed_ms, 0);
        game.advance_elapsed(u64::MAX, true);
        assert_eq!(game.elapsed_ms, MAX_ELAPSED_MS);
        assert!(game.validate().is_ok());
    }

    #[test]
    fn serde_round_trip_and_validation_reject_impossible_states() {
        let mut game = Game::new(Difficulty::Expert, 0x1234);
        game.command(Command::Reveal(240));
        game.advance_elapsed(2_000, true);
        let encoded = serde_json::to_string(&game).unwrap();
        let decoded: Game = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, game);
        assert!(decoded.validate().is_ok());

        let mut corrupt = decoded;
        corrupt.cells[0].adjacent = corrupt.cells[0].adjacent.wrapping_add(1);
        assert!(corrupt.validate().is_err());
    }

    #[test]
    fn out_of_range_commands_are_noops() {
        let mut game = Game::new(Difficulty::Beginner, 1);
        assert!(game.command(Command::Reveal(usize::MAX)).is_empty());
        assert!(game.command(Command::ToggleFlag(usize::MAX)).is_empty());
        assert!(game.command(Command::Chord(usize::MAX)).is_empty());
        assert_eq!(game.status, Status::Ready);
    }
}
