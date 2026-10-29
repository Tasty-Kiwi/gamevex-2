use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "sudoku_parity.rs"]
mod sudoku_parity;

pub const SIZES: [u8; 4] = [4, 6, 9, 16];
const HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    pub const ALL: [Self; 4] = [Self::Easy, Self::Medium, Self::Hard, Self::Expert];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
            Self::Expert => "Expert",
        }
    }

    const fn removal_ratio(self) -> f64 {
        match self {
            Self::Easy => 0.38,
            Self::Medium => 0.48,
            Self::Hard => 0.56,
            Self::Expert => 0.64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sudoku {
    pub size: u8,
    pub difficulty: Difficulty,
    pub board: Vec<u8>,
    /// Fixed clue values, with zero for every editable cell.
    pub givens: Vec<u8>,
    /// Bit n-1 denotes a pencil mark for value n.
    pub notes: Vec<u16>,
    pub elapsed_ms: u64,
    pub assisted: bool,
    pub complete: bool,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub index: usize,
    pub value: u8,
    pub explanation: String,
}

impl Sudoku {
    pub fn new(size: u8, difficulty: Difficulty, seed: u64) -> Self {
        let size = if SIZES.contains(&size) { size } else { 9 };
        let solution = make_solution(size, seed);
        let total = usize::from(size) * usize::from(size);
        let mut puzzle = solution.clone();
        let mut rng = seed;
        let mut positions: Vec<usize> = (0..total).collect();
        shuffle(&mut positions, &mut rng);
        let remove_goal = (total as f64 * difficulty.removal_ratio()).floor() as usize;
        let mut removed = 0;
        for index in positions {
            if removed >= remove_goal {
                break;
            }
            let old = puzzle[index];
            puzzle[index] = 0;
            // A bounded search treats an exhausted budget as not unique.
            if size <= 9 && count_solutions(&mut puzzle, size, 2, &mut 5_000) != 1 {
                puzzle[index] = old;
            } else {
                removed += 1;
            }
        }
        let givens = puzzle.clone();
        Self {
            size,
            difficulty,
            board: puzzle,
            givens,
            notes: vec![0; total],
            elapsed_ms: 0,
            assisted: false,
            complete: false,
            seed,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !SIZES.contains(&self.size) {
            return Err("unsupported Sudoku size".into());
        }
        let n = usize::from(self.size);
        let total = n * n;
        if self.board.len() != total || self.givens.len() != total || self.notes.len() != total {
            return Err("Sudoku arrays must match the grid size".into());
        }
        let allowed_notes = if self.size == 16 {
            u16::MAX
        } else {
            (1u16 << self.size) - 1
        };
        for i in 0..total {
            if self.board[i] > self.size || self.givens[i] > self.size {
                return Err("cell value is outside the grid".into());
            }
            if self.givens[i] != 0 && self.board[i] != self.givens[i] {
                return Err("given cells cannot be changed".into());
            }
            if self.notes[i] & !allowed_notes != 0 {
                return Err("note value is outside the grid".into());
            }
            if self.board[i] != 0 && self.notes[i] != 0 {
                return Err("filled cells cannot have notes".into());
            }
        }
        if has_conflict(&self.givens, self.size) {
            return Err("given clues contradict Sudoku rules".into());
        }
        let solved = is_solved(&self.board, self.size);
        if self.complete != solved {
            return Err("completion flag does not match the board".into());
        }
        if self.elapsed_ms > 7 * 24 * 60 * 60 * 1_000 {
            return Err("elapsed time exceeds the supported limit".into());
        }
        Ok(())
    }

    pub fn box_dims(&self) -> (usize, usize) {
        dims(self.size)
    }

    pub fn conflict(&self, index: usize) -> bool {
        if !SIZES.contains(&self.size)
            || self.board.len() != usize::from(self.size).pow(2)
            || index >= self.board.len()
            || self.board[index] == 0
        {
            return false;
        }
        let n = usize::from(self.size);
        let row = index / n;
        let col = index % n;
        let value = self.board[index];
        if (0..n).any(|c| c != col && self.board[row * n + c] == value) {
            return true;
        }
        if (0..n).any(|r| r != row && self.board[r * n + col] == value) {
            return true;
        }
        let (br, bc) = dims(self.size);
        let sr = row / br * br;
        let sc = col / bc * bc;
        (sr..sr + br).any(|r| {
            (sc..sc + bc).any(|c| (r != row || c != col) && self.board[r * n + c] == value)
        })
    }

    pub fn find_hint(&self) -> Option<Hint> {
        let n = usize::from(self.size);
        if !SIZES.contains(&self.size)
            || self.size == 16
            || self.board.len() != n * n
            || self.board.iter().any(|&v| v > self.size)
            || (0..self.board.len()).any(|i| self.conflict(i))
        {
            return None;
        }
        let mut candidates = vec![Vec::<u8>::new(); n * n];
        for (i, options) in candidates.iter_mut().enumerate() {
            if self.board[i] == 0 {
                *options = (1..=self.size).filter(|&v| self.can_place(i, v)).collect();
                if options.is_empty() {
                    return None;
                }
            }
        }
        for (i, options) in candidates.iter().enumerate() {
            if self.board[i] == 0 && options.len() == 1 {
                let row = i / n + 1;
                let col = i % n + 1;
                let value = options[0];
                return Some(Hint {
                    index: i,
                    value,
                    explanation: format!(
                        "Row {row}, column {col} has only one legal value: {value}."
                    ),
                });
            }
        }
        // Stable scan order: rows, columns, then boxes; values ascend within each unit.
        for row in 0..n {
            for value in 1..=self.size {
                let spots: Vec<_> = (0..n)
                    .map(|col| row * n + col)
                    .filter(|&i| self.board[i] == 0 && candidates[i].contains(&value))
                    .collect();
                if spots.len() == 1 {
                    return Some(Hint {
                        index: spots[0],
                        value,
                        explanation: format!("{value} can go in only one cell in row {}.", row + 1),
                    });
                }
            }
        }
        for col in 0..n {
            for value in 1..=self.size {
                let spots: Vec<_> = (0..n)
                    .map(|row| row * n + col)
                    .filter(|&i| self.board[i] == 0 && candidates[i].contains(&value))
                    .collect();
                if spots.len() == 1 {
                    return Some(Hint {
                        index: spots[0],
                        value,
                        explanation: format!(
                            "{value} can go in only one cell in column {}.",
                            col + 1
                        ),
                    });
                }
            }
        }
        let (br, bc) = dims(self.size);
        for sr in (0..n).step_by(br) {
            for sc in (0..n).step_by(bc) {
                for value in 1..=self.size {
                    let spots: Vec<_> = (sr..sr + br)
                        .flat_map(|r| (sc..sc + bc).map(move |c| r * n + c))
                        .filter(|&i| self.board[i] == 0 && candidates[i].contains(&value))
                        .collect();
                    if spots.len() == 1 {
                        return Some(Hint {
                            index: spots[0],
                            value,
                            explanation: format!("{value} can go in only one cell in this box."),
                        });
                    }
                }
            }
        }
        None
    }

    pub fn advance_time(&mut self, ms: u64) {
        if !self.complete {
            self.elapsed_ms = self
                .elapsed_ms
                .saturating_add(ms)
                .min(7 * 24 * 60 * 60 * 1_000);
        }
    }

    fn can_place(&self, index: usize, value: u8) -> bool {
        let n = usize::from(self.size);
        let row = index / n;
        let col = index % n;
        if (0..n).any(|c| self.board[row * n + c] == value)
            || (0..n).any(|r| self.board[r * n + col] == value)
        {
            return false;
        }
        let (br, bc) = dims(self.size);
        let sr = row / br * br;
        let sc = col / bc * bc;
        !(sr..sr + br).any(|r| (sc..sc + bc).any(|c| self.board[r * n + c] == value))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Records {
    times: [Option<u64>; 16],
}
impl Records {
    pub fn best(&self, size: u8, difficulty: Difficulty) -> Option<u64> {
        self.times[record_index(size, difficulty)?]
    }
    pub fn update(&mut self, game: &Sudoku) {
        if !game.complete || game.assisted || !SIZES.contains(&game.size) {
            return;
        }
        if let Some(i) = record_index(game.size, game.difficulty) {
            let time = game.elapsed_ms.max(1_000);
            self.times[i] = Some(self.times[i].map_or(time, |old| old.min(time)));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub game: Sudoku,
    pub records: Records,
    pub highlight_errors: bool,
    pub highlight_same: bool,
}
impl Session {
    pub fn new(seed: u64) -> Self {
        Self {
            game: Sudoku::new(9, Difficulty::Medium, seed),
            records: Records::default(),
            highlight_errors: true,
            highlight_same: true,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.game.validate()?;
        if self
            .records
            .times
            .iter()
            .flatten()
            .any(|&time| !(1_000..=7 * 24 * 60 * 60 * 1_000).contains(&time))
        {
            return Err("record time is outside the supported range".into());
        }
        Ok(())
    }
    pub fn command(&mut self, command: Command, history: &mut History) -> bool {
        if self.game.complete {
            return false;
        }
        match command {
            Command::Undo => {
                let Some(s) = history.snapshots.pop() else {
                    return false;
                };
                self.game.board = s.board;
                self.game.notes = s.notes;
                self.game.complete = is_solved(&self.game.board, self.game.size);
                true
            }
            Command::Enter {
                index,
                value,
                pencil,
            } => {
                if index >= self.game.board.len()
                    || value == 0
                    || value > self.game.size
                    || self.game.givens[index] != 0
                {
                    return false;
                }
                if pencil && self.game.board[index] != 0 {
                    return false;
                }
                if !pencil && self.game.board[index] == value {
                    return false;
                }
                history.push(&self.game);
                if pencil {
                    self.game.notes[index] ^= 1u16 << (value - 1);
                } else {
                    self.game.notes[index] = 0;
                    self.game.board[index] = value;
                }
                self.finish_edit();
                true
            }
            Command::Erase(index) => {
                if index >= self.game.board.len()
                    || self.game.givens[index] != 0
                    || (self.game.board[index] == 0 && self.game.notes[index] == 0)
                {
                    return false;
                }
                history.push(&self.game);
                self.game.board[index] = 0;
                self.game.notes[index] = 0;
                self.game.complete = false;
                true
            }
            Command::Hint => {
                let Some(hint) = self.game.find_hint() else {
                    return false;
                };
                history.push(&self.game);
                self.game.assisted = true;
                self.game.board[hint.index] = hint.value;
                self.game.notes[hint.index] = 0;
                self.finish_edit();
                true
            }
        }
    }
    fn finish_edit(&mut self) {
        self.game.complete = is_solved(&self.game.board, self.game.size);
        if self.game.complete {
            self.records.update(&self.game);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Enter {
        index: usize,
        value: u8,
        pencil: bool,
    },
    Erase(usize),
    Hint,
    Undo,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    board: Vec<u8>,
    notes: Vec<u16>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    snapshots: Vec<Snapshot>,
}
impl History {
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
    fn push(&mut self, game: &Sudoku) {
        if self.snapshots.len() == HISTORY_LIMIT {
            self.snapshots.remove(0);
        }
        self.snapshots.push(Snapshot {
            board: game.board.clone(),
            notes: game.notes.clone(),
        });
    }
}

fn record_index(size: u8, difficulty: Difficulty) -> Option<usize> {
    let s = SIZES.iter().position(|&x| x == size)?;
    Some(s * 4 + Difficulty::ALL.iter().position(|&d| d == difficulty)?)
}
fn dims(size: u8) -> (usize, usize) {
    match size {
        4 => (2, 2),
        6 => (2, 3),
        9 => (3, 3),
        _ => (4, 4),
    }
}
fn is_solved(board: &[u8], size: u8) -> bool {
    let n = usize::from(size);
    board.len() == n * n
        && board.iter().all(|&v| v != 0 && v <= size)
        && (0..board.len()).all(|i| {
            let row = i / n;
            let col = i % n;
            let value = board[i];
            (0..n).all(|c| c == col || board[row * n + c] != value)
                && (0..n).all(|r| r == row || board[r * n + col] != value)
                && {
                    let (br, bc) = dims(size);
                    let sr = row / br * br;
                    let sc = col / bc * bc;
                    !(sr..sr + br).any(|r| {
                        (sc..sc + bc).any(|c| (r != row || c != col) && board[r * n + c] == value)
                    })
                }
        })
}

fn has_conflict(board: &[u8], size: u8) -> bool {
    if board.len() != usize::from(size) * usize::from(size) {
        return true;
    }
    let n = usize::from(size);
    for i in 0..board.len() {
        let value = board[i];
        if value == 0 {
            continue;
        }
        let row = i / n;
        let col = i % n;
        if (0..n).any(|c| c != col && board[row * n + c] == value)
            || (0..n).any(|r| r != row && board[r * n + col] == value)
        {
            return true;
        }
        let (br, bc) = dims(size);
        let sr = row / br * br;
        let sc = col / bc * bc;
        if (sr..sr + br)
            .any(|r| (sc..sc + bc).any(|c| (r != row || c != col) && board[r * n + c] == value))
        {
            return true;
        }
    }
    false
}
fn make_solution(size: u8, seed: u64) -> Vec<u8> {
    let n = usize::from(size);
    let (br, bc) = dims(size);
    let mut rng = seed;
    let mut bands: Vec<usize> = (0..n / br).collect();
    let mut stacks: Vec<usize> = (0..n / bc).collect();
    shuffle(&mut bands, &mut rng);
    shuffle(&mut stacks, &mut rng);
    let mut rows = Vec::new();
    for b in bands {
        let mut within: Vec<_> = (0..br).collect();
        shuffle(&mut within, &mut rng);
        rows.extend(within.into_iter().map(|r| b * br + r));
    }
    let mut cols = Vec::new();
    for b in stacks {
        let mut within: Vec<_> = (0..bc).collect();
        shuffle(&mut within, &mut rng);
        cols.extend(within.into_iter().map(|c| b * bc + c));
    }
    let mut symbols: Vec<u8> = (1..=size).collect();
    shuffle(&mut symbols, &mut rng);
    let mut board = Vec::with_capacity(n * n);
    for &r in &rows {
        for &c in &cols {
            board.push(symbols[(bc * (r % br) + r / br + c) % n]);
        }
    }
    board
}
fn count_solutions(board: &mut [u8], size: u8, limit: u8, budget: &mut usize) -> u8 {
    if *budget == 0 {
        return limit;
    }
    *budget -= 1;
    let n = usize::from(size);
    let mut best = None;
    let mut best_options = Vec::new();
    for i in 0..n * n {
        if board[i] != 0 {
            continue;
        }
        let row = i / n;
        let col = i % n;
        let (br, bc) = dims(size);
        let sr = row / br * br;
        let sc = col / bc * bc;
        let opts: Vec<_> = (1..=size)
            .filter(|&v| {
                !(0..n).any(|c| board[row * n + c] == v)
                    && !(0..n).any(|r| board[r * n + col] == v)
                    && !(sr..sr + br).any(|r| (sc..sc + bc).any(|c| board[r * n + c] == v))
            })
            .collect();
        if opts.is_empty() {
            return 0;
        }
        if best.is_none() || opts.len() < best_options.len() {
            best = Some(i);
            best_options = opts;
            if best_options.len() == 1 {
                break;
            }
        }
    }
    let Some(i) = best else { return 1 };
    let mut count: u8 = 0;
    for v in best_options {
        board[i] = v;
        count = count.saturating_add(count_solutions(board, size, limit - count, budget));
        board[i] = 0;
        if count >= limit {
            return count;
        }
    }
    count
}
fn shuffle<T>(items: &mut [T], state: &mut u64) {
    for i in (1..items.len()).rev() {
        let j = (next_random(state) % (i as u64 + 1)) as usize;
        items.swap(i, j);
    }
}
fn next_random(state: &mut u64) -> u64 {
    let mut x = if *state == 0 {
        0x9e37_79b9_7f4a_7c15
    } else {
        *state
    };
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shuffle_seed_has_the_same_permutation_on_32_and_64_bit_targets() {
        let mut values: Vec<_> = (0..10).collect();
        shuffle(&mut values, &mut 0x1234_5678_8765_4321);
        // Reduce in u64 before conversion, so WASM does not truncate the PRNG output.
        assert_eq!(values, [5, 9, 7, 0, 6, 2, 3, 1, 8, 4]);
    }

    #[test]
    fn generated_boards_are_deterministic_valid_and_unique_when_supported() {
        for size in SIZES {
            for difficulty in Difficulty::ALL {
                for seed in [0, 1, 17, u64::MAX] {
                    let game = Sudoku::new(size, difficulty, seed);
                    assert_eq!(game, Sudoku::new(size, difficulty, seed));
                    game.validate().unwrap();
                    let solved = make_solution(size, seed);
                    assert!(is_solved(&solved, size));
                    assert!(
                        game.board
                            .iter()
                            .zip(&solved)
                            .all(|(&cell, &solution)| cell == 0 || cell == solution)
                    );
                    if size <= 9 {
                        assert_eq!(
                            count_solutions(&mut game.board.clone(), size, 2, &mut 5_000),
                            1
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn dimensions_and_hint_order_are_stable() {
        for (size, expected) in [(4, (2, 2)), (6, (2, 3)), (9, (3, 3)), (16, (4, 4))] {
            assert_eq!(Sudoku::new(size, Difficulty::Easy, 1).box_dims(), expected);
        }
        let solution = make_solution(4, 9);
        let mut game = Sudoku::new(4, Difficulty::Easy, 9);
        game.board = solution.clone();
        game.board[0] = 0;
        game.givens[0] = 0;
        let hint = game.find_hint().unwrap();
        assert_eq!((hint.index, hint.value), (0, solution[0]));
        game.board[1] = game.board[2];
        assert!(game.find_hint().is_none());
        game.board[1] = solution[1];
        game.size = 16;
        assert!(game.find_hint().is_none());
    }

    #[test]
    fn commands_obey_givens_notes_undo_limit_and_sticky_assistance() {
        let mut session = Session::new(55);
        let mut history = History::default();
        let empty = session.game.givens.iter().position(|&v| v == 0).unwrap();
        assert!(session.command(
            Command::Enter {
                index: empty,
                value: 1,
                pencil: true
            },
            &mut history
        ));
        assert_ne!(session.game.notes[empty], 0);
        assert!(session.command(Command::Undo, &mut history));
        assert_eq!(session.game.notes[empty], 0);
        assert!(!session.command(Command::Erase(usize::MAX), &mut history));
        let clue = session.game.givens.iter().position(|&v| v != 0).unwrap();
        assert!(!session.command(Command::Erase(clue), &mut history));

        for _ in 0..110 {
            let idx = session.game.givens.iter().position(|&v| v == 0).unwrap();
            let value = (session.game.board[idx] % session.game.size) + 1;
            session.command(
                Command::Enter {
                    index: idx,
                    value,
                    pencil: true,
                },
                &mut history,
            );
        }
        assert_eq!(history.len(), HISTORY_LIMIT);
        session.game.assisted = true;
        session.game.elapsed_ms = 12_345;
        assert!(session.command(Command::Undo, &mut history));
        assert!(session.game.assisted);
        assert_eq!(session.game.elapsed_ms, 12_345);
    }

    #[test]
    fn completion_records_only_unassisted_games_and_uses_one_second_floor() {
        fn ready_to_finish(
            size: u8,
            difficulty: Difficulty,
            seed: u64,
            elapsed_ms: u64,
            assisted: bool,
        ) -> Session {
            let solution = make_solution(size, seed);
            let mut game = Sudoku::new(size, difficulty, seed);
            game.board = solution.clone();
            game.givens = solution;
            game.notes = vec![0; game.board.len()];
            game.board[0] = 0;
            game.givens[0] = 0;
            game.elapsed_ms = elapsed_ms;
            game.assisted = assisted;
            let mut session = Session::new(seed);
            session.game = game;
            session
        }

        let mut session = ready_to_finish(4, Difficulty::Easy, 8, 0, false);
        let mut history = History::default();
        assert!(session.command(
            Command::Enter {
                index: 0,
                value: make_solution(4, 8)[0],
                pencil: false
            },
            &mut history
        ));
        assert!(session.game.complete);
        assert_eq!(session.records.best(4, Difficulty::Easy), Some(1_000));
        assert!(!session.command(Command::Undo, &mut history));

        let mut different_mode = ready_to_finish(4, Difficulty::Hard, 11, 2_000, false);
        different_mode.game.board[0] = make_solution(4, 11)[0];
        different_mode.game.complete = true;
        session.records.update(&different_mode.game);
        assert_eq!(session.records.best(4, Difficulty::Hard), Some(2_000));
        assert_eq!(session.records.best(4, Difficulty::Medium), None);
        assert_eq!(session.records.best(9, Difficulty::Easy), None);

        let mut assisted = ready_to_finish(4, Difficulty::Easy, 12, 2_000, true);
        assert!(assisted.command(
            Command::Enter {
                index: 0,
                value: make_solution(4, 12)[0],
                pencil: false
            },
            &mut History::default()
        ));
        assert!(assisted.game.complete);
        assert_eq!(assisted.records.best(4, Difficulty::Easy), None);
    }

    #[test]
    fn validation_allows_user_errors_but_rejects_corrupt_shapes_and_fixed_cells() {
        let mut game = Sudoku::new(4, Difficulty::Easy, 2);
        let empty = game.givens.iter().position(|&v| v == 0).unwrap();
        game.board[empty] = 1;
        game.validate().unwrap(); // wrong entries remain editable gameplay state
        game.board.push(0);
        assert!(game.validate().is_err());

        let mut contradictory = Sudoku::new(4, Difficulty::Easy, 3);
        let (first, second) = (0..contradictory.givens.len())
            .flat_map(|a| (a + 1..contradictory.givens.len()).map(move |b| (a, b)))
            .find(|&(a, b)| {
                a / 4 == b / 4 && contradictory.givens[a] != 0 && contradictory.givens[b] != 0
            })
            .unwrap();
        contradictory.givens[second] = contradictory.givens[first];
        contradictory.board[second] = contradictory.givens[first];
        assert!(contradictory.validate().is_err());

        let mut large = Sudoku::new(16, Difficulty::Easy, 4);
        let empty = large.board.iter().position(|&v| v == 0).unwrap();
        large.notes[empty] = 1 << 15;
        large.validate().unwrap();
        let mut small = Sudoku::new(4, Difficulty::Easy, 4);
        let small_empty = small.board.iter().position(|&v| v == 0).unwrap();
        small.notes[small_empty] = 1 << 4;
        assert!(small.validate().is_err());
    }
}
