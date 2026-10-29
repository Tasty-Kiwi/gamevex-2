use super::{Difficulty, Sudoku};
use serde::Deserialize;

const FIXTURES: &str = include_str!("../../tests/fixtures/sudoku-parity.json");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixtures {
    generated: Vec<Generated>,
    checks: Vec<Check>,
    hint_cases: std::collections::HashMap<String, HintCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Generated {
    size: u8,
    difficulty: String,
    input_seed: u64,
    puzzle: Vec<Vec<u8>>,
    solution: Vec<Vec<u8>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Check {
    name: String,
    size: u8,
    board: Vec<Vec<u8>>,
    solved: bool,
    conflict_at: Option<usize>,
    conflict: Option<bool>,
}

#[derive(Deserialize)]
struct HintCase {
    size: u8,
    board: Vec<Vec<u8>>,
    hint: Option<OracleHint>,
}

#[derive(Deserialize)]
struct OracleHint {
    row: usize,
    col: usize,
    value: u8,
    explanation: String,
}

fn parse() -> Fixtures {
    serde_json::from_str(FIXTURES).expect("oracle Sudoku fixture JSON parses")
}

fn flatten(grid: &[Vec<u8>]) -> Vec<u8> {
    grid.iter().flatten().copied().collect()
}

fn valid_complete_grid(grid: &[u8], size: u8) -> bool {
    let n = usize::from(size);
    if grid.len() != n * n || grid.iter().any(|&v| v == 0 || v > size) {
        return false;
    }
    for i in 0..n {
        let mut row = grid[i * n..(i + 1) * n].to_vec();
        row.sort_unstable();
        if row != (1..=size).collect::<Vec<_>>() {
            return false;
        }
        let mut col = (0..n).map(|r| grid[r * n + i]).collect::<Vec<_>>();
        col.sort_unstable();
        if col != (1..=size).collect::<Vec<_>>() {
            return false;
        }
    }
    let (br, bc) = match size {
        4 => (2, 2),
        6 => (2, 3),
        9 => (3, 3),
        16 => (4, 4),
        _ => return false,
    };
    for sr in (0..n).step_by(br) {
        for sc in (0..n).step_by(bc) {
            let mut values = (sr..sr + br)
                .flat_map(|r| (sc..sc + bc).map(move |c| grid[r * n + c]))
                .collect::<Vec<_>>();
            values.sort_unstable();
            if values != (1..=size).collect::<Vec<_>>() {
                return false;
            }
        }
    }
    true
}

#[test]
fn oracle_generated_boards_are_valid_and_clues_preserve_the_solution() {
    let fixtures = parse();
    assert_eq!(fixtures.generated.len(), 28);
    for case in fixtures.generated {
        assert!(case.input_seed == 8 || case.input_seed == 42);
        let solution = flatten(&case.solution);
        let puzzle = flatten(&case.puzzle);
        assert!(
            valid_complete_grid(&solution, case.size),
            "{} seed {} solution",
            case.difficulty,
            case.input_seed
        );
        assert_eq!(puzzle.len(), usize::from(case.size).pow(2));
        assert!(
            puzzle
                .iter()
                .zip(&solution)
                .all(|(&clue, &value)| clue == 0 || clue == value)
        );
        let expected_clues = usize::from(case.size).pow(2)
            - (f64::from(usize::from(case.size).pow(2) as u32)
                * match case.difficulty.as_str() {
                    "easy" => 0.38,
                    "medium" => 0.48,
                    "hard" => 0.56,
                    "expert" => 0.64,
                    other => panic!("unexpected difficulty {other}"),
                })
            .floor() as usize;
        assert!(
            puzzle.iter().filter(|&&v| v != 0).count() >= expected_clues,
            "oracle generated fewer clues than its requested removal target"
        );
    }
}

#[test]
fn oracle_conflict_and_solved_grid_results_match_rust_rules() {
    for case in parse().checks {
        let mut game = Sudoku::new(case.size, Difficulty::Easy, 1);
        game.board = flatten(&case.board);
        game.givens = vec![0; game.board.len()];
        game.complete = valid_complete_grid(&game.board, case.size);
        assert_eq!(
            game.complete, case.solved,
            "{} completion fixture",
            case.name
        );
        if let (Some(index), Some(expected)) = (case.conflict_at, case.conflict) {
            assert_eq!(
                game.conflict(index),
                expected,
                "{} conflict fixture",
                case.name
            );
        }
        if case.solved {
            assert!(
                valid_complete_grid(&game.board, case.size),
                "{} fixture must be solved",
                case.name
            );
        }
    }
}

#[test]
fn oracle_hint_categories_return_exact_index_value_and_explanation() {
    for (name, case) in parse().hint_cases {
        let mut game = Sudoku::new(case.size, Difficulty::Easy, 7);
        game.board = flatten(&case.board);
        game.givens = game.board.clone();
        let actual = game.find_hint();
        match case.hint {
            Some(expected) => {
                let actual = actual.unwrap_or_else(|| panic!("missing {name} hint"));
                assert_eq!(
                    actual.index,
                    expected.row * usize::from(case.size) + expected.col,
                    "{name} index"
                );
                assert_eq!(actual.value, expected.value, "{name} value");
                assert_eq!(
                    actual.explanation, expected.explanation,
                    "{name} explanation"
                );
            }
            None => assert!(actual.is_none(), "{name} should have no hint"),
        }
    }
}
