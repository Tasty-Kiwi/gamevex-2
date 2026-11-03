use serde::{Deserialize, Serialize};

pub const COLS: usize = 7;
pub const ROWS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Red,
    Yellow,
}

impl Player {
    pub const fn other(self) -> Self {
        match self {
            Self::Red => Self::Yellow,
            Self::Yellow => Self::Red,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Red => "Red",
            Self::Yellow => "Yellow",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[serde(alias = "TwoPlayers")]
    Computer,
}

impl Mode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Computer => "Computer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Playing,
    Won(Player),
    Draw,
}

const fn generate_winning_lines() -> [[usize; 4]; 69] {
    let mut lines = [[0usize; 4]; 69];
    let mut idx = 0;

    // Horizontal (6 rows * 4 columns = 24 lines)
    let mut r = 0;
    while r < ROWS {
        let mut c = 0;
        while c <= COLS - 4 {
            lines[idx] = [
                r * COLS + c,
                r * COLS + c + 1,
                r * COLS + c + 2,
                r * COLS + c + 3,
            ];
            idx += 1;
            c += 1;
        }
        r += 1;
    }

    // Vertical (3 rows * 7 columns = 21 lines)
    r = 0;
    while r <= ROWS - 4 {
        let mut c = 0;
        while c < COLS {
            lines[idx] = [
                r * COLS + c,
                (r + 1) * COLS + c,
                (r + 2) * COLS + c,
                (r + 3) * COLS + c,
            ];
            idx += 1;
            c += 1;
        }
        r += 1;
    }

    // Diagonal down-right (\) (3 rows * 4 columns = 12 lines)
    r = 0;
    while r <= ROWS - 4 {
        let mut c = 0;
        while c <= COLS - 4 {
            lines[idx] = [
                r * COLS + c,
                (r + 1) * COLS + c + 1,
                (r + 2) * COLS + c + 2,
                (r + 3) * COLS + c + 3,
            ];
            idx += 1;
            c += 1;
        }
        r += 1;
    }

    // Diagonal up-right (/) (3 rows * 4 columns = 12 lines)
    r = 3;
    while r < ROWS {
        let mut c = 0;
        while c <= COLS - 4 {
            lines[idx] = [
                r * COLS + c,
                (r - 1) * COLS + c + 1,
                (r - 2) * COLS + c + 2,
                (r - 3) * COLS + c + 3,
            ];
            idx += 1;
            c += 1;
        }
        r += 1;
    }

    assert!(idx == 69);
    lines
}

pub const WINNING_LINES: [[usize; 4]; 69] = generate_winning_lines();

fn has_won(board: &[u8], target: u8) -> bool {
    WINNING_LINES
        .iter()
        .any(|line| line.iter().all(|&idx| board[idx] == target))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub board: Vec<u8>,
    pub turn: Player,
    pub mode: Mode,
    pub status: Status,
    pub history: Vec<u8>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Mode::Computer)
    }
}

impl Game {
    pub fn new(mode: Mode) -> Self {
        Self {
            board: vec![0u8; ROWS * COLS],
            turn: Player::Red,
            mode,
            status: Status::Playing,
            history: Vec::new(),
        }
    }

    pub fn legal(&self, column: usize) -> bool {
        if self.status != Status::Playing || column >= COLS {
            return false;
        }
        self.board[column] == 0
    }

    pub fn play(&mut self, column: usize) -> bool {
        if !self.legal(column) {
            return false;
        }
        let Some(row) = (0..ROWS)
            .rev()
            .find(|&r| self.board[r * COLS + column] == 0)
        else {
            return false;
        };
        let player = self.turn;
        let cell_val = match player {
            Player::Red => 1,
            Player::Yellow => 2,
        };
        self.board[row * COLS + column] = cell_val;
        self.history.push(column as u8);

        if has_won(&self.board, cell_val) {
            self.status = Status::Won(player);
        } else if self.history.len() == ROWS * COLS {
            self.status = Status::Draw;
        }
        self.turn = player.other();
        true
    }

    pub fn has_progress(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn winning_cells(&self) -> Vec<usize> {
        let Status::Won(winner) = self.status else {
            return Vec::new();
        };
        let target = match winner {
            Player::Red => 1,
            Player::Yellow => 2,
        };
        let mut cells = Vec::new();
        for line in &WINNING_LINES {
            if line.iter().all(|&idx| self.board[idx] == target) {
                cells.extend_from_slice(line);
            }
        }
        cells.sort_unstable();
        cells.dedup();
        cells
    }

    pub fn bot_column(&self) -> Option<usize> {
        if self.mode != Mode::Computer
            || self.turn != Player::Yellow
            || self.status != Status::Playing
        {
            return None;
        }

        const MOVE_ORDER: [usize; COLS] = [3, 2, 4, 1, 5, 0, 6];
        let legal_moves: Vec<usize> = MOVE_ORDER
            .iter()
            .copied()
            .filter(|&c| self.board[c] == 0)
            .collect();

        if legal_moves.is_empty() {
            return None;
        }

        let mut scratch_board = self.board.clone();

        // 1. Immediate Win: If Yellow can win immediately, take it.
        for &col in &legal_moves {
            let row = (0..ROWS)
                .rev()
                .find(|&r| scratch_board[r * COLS + col] == 0)
                .unwrap();
            scratch_board[row * COLS + col] = 2;
            let wins = has_won(&scratch_board, 2);
            scratch_board[row * COLS + col] = 0;
            if wins {
                return Some(col);
            }
        }

        // 2. Immediate Block: If Red has an immediate winning threat, Yellow must block it.
        let mut red_threats = Vec::new();
        for &col in &legal_moves {
            let row = (0..ROWS)
                .rev()
                .find(|&r| scratch_board[r * COLS + col] == 0)
                .unwrap();
            scratch_board[row * COLS + col] = 1;
            let red_wins = has_won(&scratch_board, 1);
            scratch_board[row * COLS + col] = 0;
            if red_wins {
                red_threats.push(col);
            }
        }
        if red_threats.len() == 1 {
            return Some(red_threats[0]);
        }

        // 3. Minimax search with alpha-beta pruning, bounded depth suitable for WASM.
        let empty_count = self.board.iter().filter(|&&c| c == 0).count();
        let depth = 6.min(empty_count);

        let candidate_moves = if !red_threats.is_empty() {
            // Must block at least one threat if multiple exist
            red_threats
        } else {
            legal_moves
        };

        let mut best_col = candidate_moves[0];
        let mut best_score = i32::MIN;

        for &col in &candidate_moves {
            let row = (0..ROWS)
                .rev()
                .find(|&r| scratch_board[r * COLS + col] == 0)
                .unwrap();
            scratch_board[row * COLS + col] = 2;
            let score = minimax(&mut scratch_board, depth - 1, i32::MIN, i32::MAX, false);
            scratch_board[row * COLS + col] = 0;

            if score > best_score {
                best_score = score;
                best_col = col;
            }
        }

        Some(best_col)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.board.len() != ROWS * COLS {
            return Err("board must have exactly 42 cells".into());
        }
        for (i, &cell) in self.board.iter().enumerate() {
            if cell > 2 {
                return Err(format!("cell at index {i} has invalid value {cell}"));
            }
        }

        // Check for floating pieces
        for c in 0..COLS {
            let mut empty_seen = false;
            for r in (0..ROWS).rev() {
                let cell = self.board[r * COLS + c];
                if cell == 0 {
                    empty_seen = true;
                } else if empty_seen {
                    return Err(format!("floating piece at row {r}, column {c}"));
                }
            }
        }

        // History bounded to 42
        if self.history.len() > ROWS * COLS {
            return Err("history exceeds 42 moves".into());
        }
        for (i, &col) in self.history.iter().enumerate() {
            if col as usize >= COLS {
                return Err(format!(
                    "history move at index {i} has invalid column {col}"
                ));
            }
        }

        // Authoritative replay of history
        let mut replay = Game::new(self.mode);
        for (i, &col) in self.history.iter().enumerate() {
            if replay.status != Status::Playing {
                return Err(format!(
                    "history contains move at index {i} after game ended"
                ));
            }
            if !replay.legal(col as usize) {
                return Err(format!(
                    "illegal move in history at index {i}: column {col}"
                ));
            }
            if !replay.play(col as usize) {
                return Err(format!("failed to play move at index {i}: column {col}"));
            }
        }

        if self.board != replay.board {
            return Err("board does not match replay history".into());
        }
        if self.turn != replay.turn {
            return Err("turn does not match replay history".into());
        }
        if self.status != replay.status {
            return Err("status does not match replay history".into());
        }

        Ok(())
    }
}

fn evaluate_board(board: &[u8]) -> i32 {
    let mut score = 0i32;

    const COLUMN_WEIGHTS: [i32; COLS] = [1, 2, 4, 7, 4, 2, 1];
    for r in 0..ROWS {
        for c in 0..COLS {
            let cell = board[r * COLS + c];
            if cell == 2 {
                score += COLUMN_WEIGHTS[c] * 3;
            } else if cell == 1 {
                score -= COLUMN_WEIGHTS[c] * 3;
            }
        }
    }

    for line in &WINNING_LINES {
        let mut yellow_count = 0;
        let mut red_count = 0;
        for &idx in line {
            match board[idx] {
                1 => red_count += 1,
                2 => yellow_count += 1,
                _ => {}
            }
        }
        if yellow_count > 0 && red_count > 0 {
            continue;
        }
        if yellow_count == 3 {
            score += 100;
        } else if yellow_count == 2 {
            score += 10;
        } else if yellow_count == 1 {
            score += 1;
        } else if red_count == 3 {
            score -= 100;
        } else if red_count == 2 {
            score -= 10;
        } else if red_count == 1 {
            score -= 1;
        }
    }

    score
}

fn minimax(board: &mut [u8], depth: usize, mut alpha: i32, mut beta: i32, is_yellow: bool) -> i32 {
    if is_yellow {
        if has_won(board, 1) {
            return -100_000 - (depth as i32);
        }
    } else if has_won(board, 2) {
        return 100_000 + (depth as i32);
    }

    if board[..COLS].iter().all(|&cell| cell != 0) {
        return 0;
    }
    if depth == 0 {
        return evaluate_board(board);
    }

    const MOVE_ORDER: [usize; COLS] = [3, 2, 4, 1, 5, 0, 6];
    let mut any_legal = false;

    if is_yellow {
        let mut max_eval = i32::MIN;
        for &col in &MOVE_ORDER {
            if board[col] != 0 {
                continue;
            }
            any_legal = true;
            let row = (0..ROWS)
                .rev()
                .find(|&r| board[r * COLS + col] == 0)
                .unwrap();
            board[row * COLS + col] = 2;
            let eval = minimax(board, depth - 1, alpha, beta, false);
            board[row * COLS + col] = 0;

            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        if !any_legal {
            return 0;
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for &col in &MOVE_ORDER {
            if board[col] != 0 {
                continue;
            }
            any_legal = true;
            let row = (0..ROWS)
                .rev()
                .find(|&r| board[r * COLS + col] == 0)
                .unwrap();
            board[row * COLS + col] = 1;
            let eval = minimax(board, depth - 1, alpha, beta, true);
            board[row * COLS + col] = 0;

            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        if !any_legal {
            return 0;
        }
        min_eval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_and_mode_labels() {
        assert_eq!(Player::Red.other(), Player::Yellow);
        assert_eq!(Player::Yellow.other(), Player::Red);
        assert_eq!(Player::Red.label(), "Red");
        assert_eq!(Player::Yellow.label(), "Yellow");
        assert_eq!(Mode::Computer.label(), "Computer");
    }

    #[test]
    fn horizontal_win_both_players() {
        let mut g = Game::new(Mode::Computer);
        for col in [0, 0, 1, 1, 2, 2, 3] {
            assert!(g.play(col));
        }
        assert_eq!(g.status, Status::Won(Player::Red));
        let winning = g.winning_cells();
        assert_eq!(winning, vec![35, 36, 37, 38]); // row 5 cols 0..=3
        assert!(g.validate().is_ok());

        let mut g2 = Game::new(Mode::Computer);
        for col in [6, 0, 6, 1, 6, 2, 5, 3] {
            assert!(g2.play(col));
        }
        assert_eq!(g2.status, Status::Won(Player::Yellow));
        assert_eq!(g2.winning_cells(), vec![35, 36, 37, 38]);
        assert!(g2.validate().is_ok());
    }

    #[test]
    fn vertical_win() {
        let mut g = Game::new(Mode::Computer);
        for col in [0, 1, 0, 1, 0, 1, 0] {
            assert!(g.play(col));
        }
        assert_eq!(g.status, Status::Won(Player::Red));
        let mut expected = vec![14, 21, 28, 35];
        expected.sort_unstable();
        assert_eq!(g.winning_cells(), expected);
        assert!(g.validate().is_ok());

        let mut g2 = Game::new(Mode::Computer);
        for col in [0, 3, 0, 3, 0, 3, 1, 3] {
            assert!(g2.play(col));
        }
        assert_eq!(g2.status, Status::Won(Player::Yellow));
        let mut expected_yellow = vec![17, 24, 31, 38];
        expected_yellow.sort_unstable();
        assert_eq!(g2.winning_cells(), expected_yellow);
        assert!(g2.validate().is_ok());
    }

    #[test]
    fn both_diagonals_win() {
        let mut g = Game::new(Mode::Computer);
        for col in [0, 1, 1, 2, 3, 2, 2, 3, 4, 3, 3] {
            assert!(g.play(col));
        }
        assert_eq!(g.status, Status::Won(Player::Red));
        let mut expected = vec![17, 23, 29, 35];
        expected.sort_unstable();
        assert_eq!(g.winning_cells(), expected);
        assert!(g.validate().is_ok());

        let mut g2 = Game::new(Mode::Computer);
        for col in [3, 2, 2, 1, 6, 1, 1, 0, 6, 0, 6, 0, 0] {
            assert!(g2.play(col));
        }
        assert_eq!(g2.status, Status::Won(Player::Red));
        let mut expected_dr = vec![14, 22, 30, 38];
        expected_dr.sort_unstable();
        assert_eq!(g2.winning_cells(), expected_dr);
        assert!(g2.validate().is_ok());
    }

    #[test]
    fn gravity_full_and_invalid_columns() {
        let mut g = Game::new(Mode::Computer);
        assert!(!g.has_progress());

        assert!(!g.legal(COLS));
        assert!(!g.legal(99));
        assert!(!g.play(COLS));
        assert!(!g.play(100));

        for row in (0..ROWS).rev() {
            assert!(g.legal(0));
            assert!(g.play(0));
            let expected_cell = if row % 2 == 1 { 1 } else { 2 };
            assert_eq!(g.board[row * COLS], expected_cell);
        }
        assert!(g.has_progress());
        assert_eq!(g.history.len(), 6);

        assert!(!g.legal(0));
        assert!(!g.play(0));
        assert_eq!(g.history.len(), 6);
        assert!(g.validate().is_ok());
    }

    #[test]
    fn terminal_and_draw() {
        let mut g = Game::new(Mode::Computer);
        for col in [0, 0, 1, 1, 2, 2, 3] {
            assert!(g.play(col));
        }
        assert_eq!(g.status, Status::Won(Player::Red));

        for c in 0..COLS {
            assert!(!g.legal(c));
            assert!(!g.play(c));
        }
        assert_eq!(g.history.len(), 7);

        let mut draw_game = Game::new(Mode::Computer);
        let moves = [
            0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, // cols 0, 1 full
            2, 3, 2, 3, 2, 3, 3, 2, 3, 2, 3, 2, // cols 2, 3 full
            4, 5, 4, 5, 4, 5, 5, 4, 5, 4, 5, 4, // cols 4, 5 full
            6, 6, 6, 6, 6, 6, // col 6 full
        ];
        for &m in &moves {
            assert!(draw_game.play(m));
        }
        assert_eq!(draw_game.history.len(), 42);
        assert_eq!(draw_game.status, Status::Draw);
        assert_eq!(draw_game.winning_cells(), Vec::<usize>::new());
        assert!(!draw_game.legal(0));
        assert!(!draw_game.play(0));
        assert!(draw_game.validate().is_ok());
    }

    #[test]
    fn bot_win_and_block() {
        let comp = Game::new(Mode::Computer);
        assert_eq!(comp.turn, Player::Red);
        assert_eq!(comp.bot_column(), None);

        let mut g = Game::new(Mode::Computer);
        assert!(g.play(0));
        assert!(g.play(0));
        assert!(g.play(1));
        assert!(g.play(1));
        assert!(g.play(2));
        assert_eq!(g.turn, Player::Yellow);
        assert_eq!(g.bot_column(), Some(3));
        assert!(g.play(g.bot_column().unwrap()));
        assert_ne!(g.status, Status::Won(Player::Red));

        let mut g2 = Game::new(Mode::Computer);
        assert!(g2.play(6));
        assert!(g2.play(0));
        assert!(g2.play(6));
        assert!(g2.play(1));
        assert!(g2.play(6));
        assert!(g2.play(2));
        assert!(g2.play(5));
        assert_eq!(g2.turn, Player::Yellow);
        assert_eq!(g2.bot_column(), Some(3));
        assert!(g2.play(g2.bot_column().unwrap()));
        assert_eq!(g2.status, Status::Won(Player::Yellow));

        assert_eq!(g2.bot_column(), None);
    }

    #[test]
    fn bot_avoids_supporting_an_opponents_winning_cell() {
        let mut game = Game::new(Mode::Computer);
        for col in [0, 2, 2, 1, 1, 5, 1, 0, 3, 3, 5, 2, 3] {
            assert!(game.play(col));
        }
        assert!(game.validate().is_ok());
        let mut trap = game.clone();
        assert!(trap.play(0));
        assert!(trap.play(0));
        assert_eq!(trap.status, Status::Won(Player::Red));
        let choice = game.bot_column().unwrap();
        assert_ne!(choice, 0);
        assert!(game.play(choice));
        for reply in 0..COLS {
            let mut next = game.clone();
            if next.play(reply) {
                assert_ne!(next.status, Status::Won(Player::Red));
            }
        }
    }

    #[test]
    fn replay_validation_and_corruption_rejection() {
        let mut g = Game::new(Mode::Computer);
        for col in [3, 2, 3, 2, 3, 2, 4] {
            assert!(g.play(col));
        }
        assert!(g.validate().is_ok());

        let mut bad = g.clone();
        bad.board.pop();
        assert!(bad.validate().is_err());

        let mut bad = g.clone();
        bad.board[0] = 3;
        assert!(bad.validate().is_err());

        let mut bad = g.clone();
        bad.board[0] = 1; // row 0 is empty below in col 0
        assert!(bad.validate().is_err());

        let mut bad = g.clone();
        bad.history = vec![0; 43];
        assert!(bad.validate().is_err());

        let mut bad = g.clone();
        bad.history.push(7);
        assert!(bad.validate().is_err());

        let mut terminal = Game::new(Mode::Computer);
        for col in [0, 0, 1, 1, 2, 2, 3] {
            terminal.play(col);
        }
        assert_eq!(terminal.status, Status::Won(Player::Red));
        let mut bad_post_win = terminal.clone();
        bad_post_win.history.push(4);
        assert!(bad_post_win.validate().is_err());

        let mut bad_board = g.clone();
        bad_board.board[35] = 2; // change a cell
        assert!(bad_board.validate().is_err());

        let mut bad_turn = g.clone();
        bad_turn.turn = bad_turn.turn.other();
        assert!(bad_turn.validate().is_err());

        let mut bad_status = g.clone();
        bad_status.status = Status::Won(Player::Yellow);
        assert!(bad_status.validate().is_err());
    }
}
