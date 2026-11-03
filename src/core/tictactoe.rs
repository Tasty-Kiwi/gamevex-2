use serde::{Deserialize, Serialize};

pub const WINNING_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

pub const MOVE_ORDER: [usize; 9] = [4, 0, 2, 6, 8, 1, 3, 5, 7];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Human,
    Cpu,
}

impl Player {
    pub const fn other(self) -> Self {
        match self {
            Self::Human => Self::Cpu,
            Self::Cpu => Self::Human,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Playing,
    Won(Player),
    Draw,
}

pub const MIN_DIFFICULTY: u8 = 1;
pub const MAX_DIFFICULTY: u8 = 5;

pub fn default_difficulty() -> u8 {
    5
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub board: Vec<u8>,
    pub turn: Player,
    pub status: Status,
    pub history: Vec<u8>,
    #[serde(default = "default_difficulty")]
    pub difficulty: u8,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: vec![0; 9],
            turn: Player::Human,
            status: Status::Playing,
            history: Vec::new(),
            difficulty: default_difficulty(),
        }
    }

    pub fn set_difficulty(&mut self, level: u8) -> bool {
        if !(MIN_DIFFICULTY..=MAX_DIFFICULTY).contains(&level) || level == self.difficulty {
            return false;
        }
        self.difficulty = level;
        true
    }

    pub fn legal(&self, index: usize) -> bool {
        self.status == Status::Playing && index < 9 && self.board[index] == 0
    }

    pub fn play(&mut self, index: usize) -> bool {
        if !self.legal(index) {
            return false;
        }
        let cell_val = match self.turn {
            Player::Human => 1,
            Player::Cpu => 2,
        };
        self.board[index] = cell_val;
        self.history.push(index as u8);

        let player = self.turn;
        let mut won = false;
        for line in &WINNING_LINES {
            if line.iter().all(|&idx| self.board[idx] == cell_val) {
                won = true;
                break;
            }
        }

        if won {
            self.status = Status::Won(player);
        } else if self.history.len() == 9 || !self.board.contains(&0) {
            self.status = Status::Draw;
        } else {
            self.status = Status::Playing;
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
            Player::Human => 1,
            Player::Cpu => 2,
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

    pub fn bot_cell(&self) -> Option<usize> {
        if self.status != Status::Playing || self.turn != Player::Cpu {
            return None;
        }

        let legal_moves: Vec<usize> = (0..9).filter(|&i| self.board[i] == 0).collect();
        if legal_moves.is_empty() {
            return None;
        }

        // Level 1: lowest varied legal moves (no win or block check)
        if self.difficulty == 1 {
            return Some(deterministic_pick(&legal_moves, &self.history, 1));
        }

        // Level 2: immediate wins plus variety (does not block human threats)
        if self.difficulty == 2 {
            if let Some(m) = find_winning_move(&self.board, 2) {
                return Some(m);
            }
            return Some(deterministic_pick(&legal_moves, &self.history, 2));
        }

        // Level 3: immediate wins, immediate blocks, short search (depth 1 with heuristic)
        if self.difficulty == 3 {
            if let Some(m) = find_winning_move(&self.board, 2) {
                return Some(m);
            }
            if let Some(m) = find_winning_move(&self.board, 1) {
                return Some(m);
            }
            let mut best_score = i32::MIN;
            let mut best_move = None;
            for &idx in &MOVE_ORDER {
                if self.board[idx] == 0 {
                    let mut board = self.board.clone();
                    board[idx] = 2;
                    let score = bounded_minimax(&mut board, 1, 1, false);
                    if score > best_score {
                        best_score = score;
                        best_move = Some(idx);
                    }
                }
            }
            return best_move;
        }

        // Level 4: immediate wins, immediate blocks, deeper search (depth 3 with heuristic)
        if self.difficulty == 4 {
            if let Some(m) = find_winning_move(&self.board, 2) {
                return Some(m);
            }
            if let Some(m) = find_winning_move(&self.board, 1) {
                return Some(m);
            }
            let mut best_score = i32::MIN;
            let mut best_move = None;
            for &idx in &MOVE_ORDER {
                if self.board[idx] == 0 {
                    let mut board = self.board.clone();
                    board[idx] = 2;
                    let score = bounded_minimax(&mut board, 1, 3, false);
                    if score > best_score {
                        best_score = score;
                        best_move = Some(idx);
                    }
                }
            }
            return best_move;
        }

        // Level 5 (Expert): existing exhaustive minimax (never loses)
        let mut best_score = i32::MIN;
        let mut best_move = None;

        for &idx in &MOVE_ORDER {
            if self.board[idx] == 0 {
                let mut board = self.board.clone();
                board[idx] = 2;
                let score = minimax(&mut board, 1, false);
                if score > best_score {
                    best_score = score;
                    best_move = Some(idx);
                }
            }
        }

        best_move
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(MIN_DIFFICULTY..=MAX_DIFFICULTY).contains(&self.difficulty) {
            return Err(format!(
                "difficulty {} must be between {MIN_DIFFICULTY} and {MAX_DIFFICULTY}",
                self.difficulty
            ));
        }
        if self.board.len() != 9 {
            return Err("board must have exactly 9 cells".into());
        }
        for (i, &cell) in self.board.iter().enumerate() {
            if cell > 2 {
                return Err(format!("cell at index {i} has invalid value {cell}"));
            }
        }
        if self.history.len() > 9 {
            return Err("history exceeds 9 moves".into());
        }
        for (i, &cell) in self.history.iter().enumerate() {
            if cell >= 9 {
                return Err(format!("history move at index {i} has invalid cell {cell}"));
            }
        }

        let mut replay = Game::new();
        replay.difficulty = self.difficulty;
        for (i, &cell) in self.history.iter().enumerate() {
            if replay.status != Status::Playing {
                return Err(format!(
                    "history contains move at index {i} after game ended"
                ));
            }
            if !replay.legal(cell as usize) {
                return Err(format!("illegal move in history at index {i}: cell {cell}"));
            }
            if !replay.play(cell as usize) {
                return Err(format!("failed to play move at index {i}: cell {cell}"));
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

fn deterministic_pick(moves: &[usize], history: &[u8], salt: usize) -> usize {
    let mut h = (salt as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for &m in history {
        h = h.wrapping_mul(31).wrapping_add(m as u64 + 1);
    }
    moves[(h % moves.len() as u64) as usize]
}

fn find_winning_move(board: &[u8], player_val: u8) -> Option<usize> {
    for &idx in &MOVE_ORDER {
        if board[idx] == 0 {
            let mut scratch = board.to_vec();
            scratch[idx] = player_val;
            if check_win(&scratch, player_val) {
                return Some(idx);
            }
        }
    }
    None
}

fn evaluate_ttt_board(board: &[u8]) -> i32 {
    let mut score = 0;
    if board[4] == 2 {
        score += 3;
    } else if board[4] == 1 {
        score -= 3;
    }
    for &c in &[0, 2, 6, 8] {
        if board[c] == 2 {
            score += 1;
        } else if board[c] == 1 {
            score -= 1;
        }
    }
    for line in &WINNING_LINES {
        let mut cpu = 0;
        let mut human = 0;
        for &idx in line {
            if board[idx] == 2 {
                cpu += 1;
            } else if board[idx] == 1 {
                human += 1;
            }
        }
        if cpu > 0 && human > 0 {
            continue;
        }
        if cpu == 2 {
            score += 5;
        } else if cpu == 1 {
            score += 1;
        }
        if human == 2 {
            score -= 5;
        } else if human == 1 {
            score -= 1;
        }
    }
    score
}

fn bounded_minimax(board: &mut [u8], depth: i32, max_depth: i32, is_cpu: bool) -> i32 {
    if check_win(board, 2) {
        return 100 - depth;
    }
    if check_win(board, 1) {
        return -100 + depth;
    }
    if is_full(board) {
        return 0;
    }
    if depth >= max_depth {
        return evaluate_ttt_board(board);
    }

    if is_cpu {
        let mut max_eval = i32::MIN;
        for &idx in &MOVE_ORDER {
            if board[idx] == 0 {
                board[idx] = 2;
                let eval = bounded_minimax(board, depth + 1, max_depth, false);
                board[idx] = 0;
                max_eval = max_eval.max(eval);
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for &idx in &MOVE_ORDER {
            if board[idx] == 0 {
                board[idx] = 1;
                let eval = bounded_minimax(board, depth + 1, max_depth, true);
                board[idx] = 0;
                min_eval = min_eval.min(eval);
            }
        }
        min_eval
    }
}

fn check_win(board: &[u8], player_val: u8) -> bool {
    for line in &WINNING_LINES {
        if board[line[0]] == player_val
            && board[line[1]] == player_val
            && board[line[2]] == player_val
        {
            return true;
        }
    }
    false
}

fn is_full(board: &[u8]) -> bool {
    !board.contains(&0)
}

fn minimax(board: &mut [u8], depth: i32, is_cpu: bool) -> i32 {
    if check_win(board, 2) {
        return 10 - depth;
    }
    if check_win(board, 1) {
        return -10 + depth;
    }
    if is_full(board) {
        return 0;
    }

    if is_cpu {
        let mut max_eval = i32::MIN;
        for &idx in &MOVE_ORDER {
            if board[idx] == 0 {
                board[idx] = 2;
                let eval = minimax(board, depth + 1, false);
                board[idx] = 0;
                max_eval = max_eval.max(eval);
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for &idx in &MOVE_ORDER {
            if board[idx] == 0 {
                board[idx] = 1;
                let eval = minimax(board, depth + 1, true);
                board[idx] = 0;
                min_eval = min_eval.min(eval);
            }
        }
        min_eval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_is_default_and_starts_clean() {
        let g = Game::new();
        assert_eq!(g, Game::default());
        assert_eq!(g.board, vec![0; 9]);
        assert_eq!(g.turn, Player::Human);
        assert_eq!(g.status, Status::Playing);
        assert!(g.history.is_empty());
        assert!(!g.has_progress());
        assert!(g.winning_cells().is_empty());
        assert_eq!(g.bot_cell(), None);
        assert!(g.validate().is_ok());
    }

    #[test]
    fn rules_and_move_rejections() {
        let mut g = Game::new();
        assert!(g.legal(0));
        assert!(g.legal(8));
        assert!(!g.legal(9));

        assert!(g.play(4));
        assert_eq!(g.board[4], 1);
        assert_eq!(g.turn, Player::Cpu);
        assert_eq!(g.history, vec![4]);
        assert!(g.has_progress());

        // Occupied cell rejected
        assert!(!g.legal(4));
        assert!(!g.play(4));

        // Out of range rejected
        assert!(!g.legal(9));
        assert!(!g.play(9));

        // CPU play via play() is accepted
        assert!(g.play(0));
        assert_eq!(g.board[0], 2);
        assert_eq!(g.turn, Player::Human);
        assert_eq!(g.history, vec![4, 0]);
        assert!(g.validate().is_ok());
    }

    #[test]
    fn horizontal_vertical_and_diagonal_wins() {
        // Horizontal win: row 0 (0, 1, 2)
        let mut g = Game::new();
        g.play(0); // H
        g.play(3); // C
        g.play(1); // H
        g.play(4); // C
        g.play(2); // H wins
        assert_eq!(g.status, Status::Won(Player::Human));
        assert_eq!(g.winning_cells(), vec![0, 1, 2]);
        assert!(!g.legal(5));
        assert!(!g.play(5));
        assert!(g.validate().is_ok());

        // Vertical win: col 1 (1, 4, 7)
        let mut g = Game::new();
        g.play(1); // H
        g.play(0); // C
        g.play(4); // H
        g.play(2); // C
        g.play(7); // H wins
        assert_eq!(g.status, Status::Won(Player::Human));
        assert_eq!(g.winning_cells(), vec![1, 4, 7]);
        assert!(g.validate().is_ok());

        // Diagonal \ win (0, 4, 8)
        let mut g = Game::new();
        g.play(0); // H
        g.play(1); // C
        g.play(4); // H
        g.play(2); // C
        g.play(8); // H wins
        assert_eq!(g.status, Status::Won(Player::Human));
        assert_eq!(g.winning_cells(), vec![0, 4, 8]);
        assert!(g.validate().is_ok());

        // Diagonal / win (2, 4, 6)
        let mut g = Game::new();
        g.play(2); // H
        g.play(0); // C
        g.play(4); // H
        g.play(1); // C
        g.play(6); // H wins
        assert_eq!(g.status, Status::Won(Player::Human));
        assert_eq!(g.winning_cells(), vec![2, 4, 6]);
        assert!(g.validate().is_ok());
    }

    #[test]
    fn draw_game_reaches_terminal_and_validates() {
        // Human: 0, 1, 5, 6, 8
        // CPU:   4, 2, 3, 7
        // Board:
        // 1 1 2
        // 2 2 1
        // 1 2 1
        let mut g = Game::new();
        let moves = [0, 4, 1, 2, 5, 3, 6, 7, 8];
        for &m in &moves {
            assert!(g.play(m));
        }
        assert_eq!(g.status, Status::Draw);
        assert!(g.winning_cells().is_empty());
        assert_eq!(g.history.len(), 9);
        assert!(!g.legal(0));
        assert!(!g.play(0));
        assert!(g.validate().is_ok());
    }

    #[test]
    fn bot_wins_immediately_when_available() {
        let mut g = Game::new();
        for cell in [3, 0, 4, 1, 8] {
            assert!(g.play(cell));
        }
        assert!(g.validate().is_ok());
        assert_eq!(g.bot_cell(), Some(2));
    }

    #[test]
    fn bot_blocks_human_threat() {
        let mut g = Game::new();
        g.board = vec![1, 1, 0, 2, 0, 0, 0, 0, 0];
        g.turn = Player::Cpu;
        g.status = Status::Playing;
        g.history = vec![0, 3, 1];
        assert_eq!(g.bot_cell(), Some(2));
    }

    #[test]
    fn bot_only_returns_move_on_cpu_turn_while_playing() {
        let mut g = Game::new();
        assert_eq!(g.turn, Player::Human);
        assert_eq!(g.bot_cell(), None);

        g.play(0);
        assert_eq!(g.turn, Player::Cpu);
        assert_eq!(g.bot_cell(), Some(4)); // Center preferred against corner

        // Terminal game returns None
        g.status = Status::Won(Player::Human);
        assert_eq!(g.bot_cell(), None);
    }

    #[test]
    fn exhaustive_reachable_game_tree_cpu_never_loses() {
        fn explore(game: &Game, leaf_count: &mut usize) {
            assert!(game.validate().is_ok());
            match game.status {
                Status::Playing => {
                    if game.turn == Player::Human {
                        for idx in 0..9 {
                            if game.legal(idx) {
                                let mut next = game.clone();
                                assert!(next.play(idx));
                                explore(&next, leaf_count);
                            }
                        }
                    } else {
                        let bot_move = game.bot_cell().expect("bot must find a move when playing");
                        assert!(game.legal(bot_move));
                        let mut next = game.clone();
                        assert!(next.play(bot_move));
                        explore(&next, leaf_count);
                    }
                }
                Status::Won(winner) => {
                    *leaf_count += 1;
                    assert_ne!(
                        winner,
                        Player::Human,
                        "CPU must never lose against any sequence of moves! History: {:?}",
                        game.history
                    );
                    assert_eq!(winner, Player::Cpu);
                }
                Status::Draw => {
                    *leaf_count += 1;
                }
            }
        }

        let mut leaf_count = 0;
        explore(&Game::new(), &mut leaf_count);
        assert!(leaf_count > 0, "must have explored reachable games");
    }

    #[test]
    fn validation_rejects_malformed_state() {
        let mut g = Game::new();
        g.play(0);
        g.play(4);

        // Corrupt board length
        let mut invalid = g.clone();
        invalid.board.push(0);
        assert!(invalid.validate().is_err());
        invalid.board.truncate(8);
        assert!(invalid.validate().is_err());

        // Corrupt board cell value
        let mut invalid = g.clone();
        invalid.board[0] = 3;
        assert!(invalid.validate().is_err());

        // History exceeds 9 moves
        let mut invalid = g.clone();
        invalid.history = vec![0, 4, 1, 2, 5, 3, 6, 7, 8, 0];
        assert!(invalid.validate().is_err());

        // History invalid cell index >= 9
        let mut invalid = g.clone();
        invalid.history = vec![0, 9];
        assert!(invalid.validate().is_err());

        // Post-terminal moves in history
        let mut won_game = Game::new();
        won_game.play(0); // H
        won_game.play(3); // C
        won_game.play(1); // H
        won_game.play(4); // C
        won_game.play(2); // H won!
        assert_eq!(won_game.status, Status::Won(Player::Human));
        let mut post_term = won_game.clone();
        post_term.history.push(5);
        post_term.board[5] = 2;
        assert!(post_term.validate().is_err());

        // Board doesn't match replay
        let mut mismatch_board = g.clone();
        mismatch_board.board[8] = 1;
        assert!(mismatch_board.validate().is_err());

        // Turn doesn't match replay
        let mut mismatch_turn = g.clone();
        mismatch_turn.turn = Player::Cpu;
        assert!(mismatch_turn.validate().is_err());

        // Status doesn't match replay
        let mut mismatch_status = g.clone();
        mismatch_status.status = Status::Draw;
        assert!(mismatch_status.validate().is_err());

        // Illegal move in history (playing same cell twice)
        let mut duplicate_move = Game::new();
        duplicate_move.history = vec![0, 0];
        assert!(duplicate_move.validate().is_err());

        let mut bad_diff_low = g.clone();
        bad_diff_low.difficulty = 0;
        assert!(bad_diff_low.validate().is_err());

        let mut bad_diff_high = g.clone();
        bad_diff_high.difficulty = 6;
        assert!(bad_diff_high.validate().is_err());
    }

    #[test]
    fn difficulty_contract_and_serde_defaults() {
        assert_eq!(MIN_DIFFICULTY, 1);
        assert_eq!(MAX_DIFFICULTY, 5);
        assert_eq!(default_difficulty(), 5);

        let mut g = Game::new();
        assert_eq!(g.difficulty, 5);

        // Invalid changes rejected
        assert!(!g.set_difficulty(0));
        assert!(!g.set_difficulty(6));
        assert!(!g.set_difficulty(5)); // no-op rejected
        assert_eq!(g.difficulty, 5);

        // Play a move
        assert!(g.play(4));
        let snap_board = g.board.clone();
        let snap_turn = g.turn;
        let snap_history = g.history.clone();
        let snap_status = g.status;

        // Valid change preserves board, turn, history, status
        assert!(g.set_difficulty(3));
        assert_eq!(g.difficulty, 3);
        assert_eq!(g.board, snap_board);
        assert_eq!(g.turn, snap_turn);
        assert_eq!(g.history, snap_history);
        assert_eq!(g.status, snap_status);
        assert!(g.validate().is_ok());

        assert!(!g.set_difficulty(3));

        // Missing difficulty in JSON defaults to 5
        let json_without_diff = serde_json::json!({
            "board": snap_board,
            "turn": snap_turn,
            "status": snap_status,
            "history": snap_history,
        });
        let loaded: Game = serde_json::from_value(json_without_diff).unwrap();
        assert_eq!(loaded.difficulty, 5);
        assert_eq!(loaded.board, snap_board);
        assert_eq!(loaded.history, snap_history);
        assert!(loaded.validate().is_ok());
    }

    #[test]
    fn all_difficulty_levels_produce_legal_moves() {
        for diff in MIN_DIFFICULTY..=MAX_DIFFICULTY {
            let mut g = Game::new();
            assert!(g.set_difficulty(diff) || diff == 5);
            assert!(g.play(0));
            assert_eq!(g.turn, Player::Cpu);

            let bot_choice = g.bot_cell();
            assert!(bot_choice.is_some());
            let cell = bot_choice.unwrap();
            assert!(g.legal(cell));
            assert!(g.play(cell));
            assert!(g.validate().is_ok());
        }
    }

    #[test]
    fn difficulty_win_and_block_tactics_and_differing_choices() {
        // Position where CPU has immediate win at cell 2:
        // Moves: H3, C0, H4, C1, H8 -> CPU turn
        let mut win_game = Game::new();
        for &m in &[3, 0, 4, 1, 8] {
            assert!(win_game.play(m));
        }
        assert_eq!(win_game.turn, Player::Cpu);

        // Level 2, 3, 4, 5 take cell 2
        for diff in 2..=5 {
            let mut g = win_game.clone();
            g.difficulty = diff;
            assert_eq!(g.bot_cell(), Some(2));
        }

        // Level 1 does not guarantee win and chooses another legal cell
        let mut l1_win = win_game.clone();
        l1_win.difficulty = 1;
        let l1_choice = l1_win.bot_cell().unwrap();
        assert!(l1_win.legal(l1_choice));
        assert_ne!(l1_choice, 2);

        // Position where Human threatens win at cell 2:
        // Moves: H0, C3, H1 -> CPU turn
        let mut block_game = Game::new();
        for &m in &[0, 3, 1] {
            assert!(block_game.play(m));
        }
        assert_eq!(block_game.turn, Player::Cpu);

        // Level 3, 4, 5 must block at cell 2
        for diff in 3..=5 {
            let mut g = block_game.clone();
            g.difficulty = diff;
            assert_eq!(g.bot_cell(), Some(2));
        }

        // Level 2 does not block
        let mut l2_block = block_game.clone();
        l2_block.difficulty = 2;
        let l2_choice = l2_block.bot_cell().unwrap();
        assert!(l2_block.legal(l2_choice));
        assert_ne!(l2_choice, 2);

        // Positional difference: H0, C4, H8 -> CPU turn
        let mut pos_game = Game::new();
        for &m in &[0, 4, 8] {
            assert!(pos_game.play(m));
        }
        assert_eq!(pos_game.turn, Player::Cpu);

        let mut l3_pos = pos_game.clone();
        l3_pos.difficulty = 3;
        let mut l5_pos = pos_game.clone();
        l5_pos.difficulty = 5;

        let l3_choice = l3_pos.bot_cell().unwrap();
        let l5_choice = l5_pos.bot_cell().unwrap();
        // Level 5 plays edge (1) to prevent fork; Level 3 short search chooses corner (2)
        assert_ne!(l3_choice, l5_choice);
        assert_eq!(l5_choice, 1);
        assert_eq!(l3_choice, 2);
    }

    #[test]
    fn easy_can_lose_and_expert_never_loses() {
        // Demonstrate Easy (Level 1) can lose to Human
        let mut g = Game::new();
        g.set_difficulty(1);
        // H plays 0
        assert!(g.play(0));
        let c1 = g.bot_cell().unwrap();
        assert!(g.play(c1));
        // H plays 1
        assert!(g.play(1));
        let c2 = g.bot_cell().unwrap();
        assert!(g.play(c2));
        // H plays 2 to complete top row
        assert!(g.play(2));
        assert_eq!(g.status, Status::Won(Player::Human));
        assert!(g.validate().is_ok());

        // Expert (Level 5) never loses against optimal or aggressive human play
        let mut exp = Game::new();
        assert_eq!(exp.difficulty, 5);
        assert!(exp.play(0));
        let exp_c1 = exp.bot_cell().unwrap();
        assert_eq!(exp_c1, 4); // Always plays center against corner
        assert!(exp.play(exp_c1));
        assert!(exp.play(8)); // Opposite corner
        let exp_c2 = exp.bot_cell().unwrap();
        assert_eq!(exp_c2, 1); // Edge to prevent corner-corner fork
        assert!(exp.play(exp_c2));
    }

    #[test]
    fn serde_roundtrip() {
        let mut g = Game::new();
        g.play(4);
        g.play(0);
        let serialized = serde_json::to_string(&g).unwrap();
        let deserialized: Game = serde_json::from_str(&serialized).unwrap();
        assert_eq!(g, deserialized);
        assert!(deserialized.validate().is_ok());
    }
}
