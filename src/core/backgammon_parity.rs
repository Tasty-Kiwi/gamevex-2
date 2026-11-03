use super::backgammon::{self, Board, Color, Difficulty, Move, Point};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    schema_version: u32,
    #[serde(rename = "sourceSHA256")]
    source_sha256: String,
    counts: Counts,
    known_divergences: Vec<Divergence>,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Counts {
    cases: usize,
    sequences: usize,
    transition_samples: usize,
}
#[derive(Deserialize)]
struct Divergence {
    id: String,
    status: String,
    reason: String,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    board: BoardInput,
    dice: Vec<u8>,
    color: String,
    #[serde(rename = "allInHome")]
    all_in_home: BoolPair,
    #[serde(rename = "pipCount")]
    pip_count: NumPair,
    #[serde(rename = "legalSequences")]
    legal_sequences: Vec<Vec<MoveInput>>,
    #[serde(rename = "sampledTransitions")]
    sampled_transitions: Vec<SampledTransition>,
    #[serde(rename = "hardBestSequences")]
    hard_best_sequences: Vec<Vec<MoveInput>>,
}
#[derive(Deserialize)]
struct BoardInput {
    p: [i8; 24],
    b: [u8; 2],
    o: [u8; 2],
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SampledTransition {
    sequence_index: usize,
    boards: Vec<BoardInput>,
}
#[derive(Deserialize, Clone, Debug)]
struct MoveInput(u8, u8, u8, bool);
#[derive(Deserialize)]
struct BoolPair {
    white: bool,
    black: bool,
}
#[derive(Deserialize)]
struct NumPair {
    white: u16,
    black: u16,
}

impl BoardInput {
    fn board(&self) -> Board {
        Board {
            points: self.p.map(|n| Point {
                color: if n > 0 {
                    Some(Color::White)
                } else if n < 0 {
                    Some(Color::Black)
                } else {
                    None
                },
                count: n.unsigned_abs(),
            }),
            bar: self.b,
            off: self.o,
        }
    }
}
fn color(name: &str) -> Color {
    match name {
        "white" => Color::White,
        "black" => Color::Black,
        other => panic!("unknown fixture color {other}"),
    }
}
impl MoveInput {
    fn movement(&self) -> Move {
        Move {
            from: self.0,
            to: self.1,
            die: self.2,
            is_hit: self.3,
        }
    }
    fn key(&self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }
}
fn signature(moves: &[Move]) -> Vec<(u8, u8, u8)> {
    moves.iter().map(|m| (m.from, m.to, m.die)).collect()
}
fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../../tests/fixtures/backgammon-oracle.json"))
        .expect("valid generated backgammon oracle fixture")
}

#[test]
fn matches_oracle_legal_sequences_boards_and_pip_counts() {
    let fixture = fixture();
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.source_sha256.len(), 64);
    assert_eq!(fixture.cases.len(), fixture.counts.cases);
    assert!(
        fixture
            .known_divergences
            .iter()
            .any(|d| d.id == "bearing-off-overshoot"
                && d.status == "excluded-from-parity-cases"
                && d.reason.contains("farther checker"))
    );
    let mut sequence_count = 0;
    let mut transition_count = 0;
    for case in fixture.cases {
        let color = color(&case.color);
        let board = case.board.board();
        assert_eq!(board.points.len(), 24, "{}", case.id);
        assert_eq!(
            board.pip_count(Color::White),
            case.pip_count.white,
            "{} white",
            case.id
        );
        assert_eq!(
            board.pip_count(Color::Black),
            case.pip_count.black,
            "{} black",
            case.id
        );
        assert_eq!(
            board.all_in_home(Color::White),
            case.all_in_home.white,
            "{} white",
            case.id
        );
        assert_eq!(
            board.all_in_home(Color::Black),
            case.all_in_home.black,
            "{} black",
            case.id
        );
        let actual = backgammon::legal_sequences(&board, &case.dice, color);
        let expected: Vec<Vec<(u8, u8, u8)>> = case
            .legal_sequences
            .iter()
            .map(|s| s.iter().map(MoveInput::key).collect())
            .collect();
        let mut actual_keys: Vec<_> = actual.iter().map(|s| signature(s)).collect();
        actual_keys.sort();
        let mut expected_keys = expected.clone();
        expected_keys.sort();
        assert_eq!(
            actual_keys, expected_keys,
            "legal sequence set: {}",
            case.id
        );
        sequence_count += expected.len();

        assert!(
            case.sampled_transitions.len() <= 8,
            "transition sample cap: {}",
            case.id
        );
        transition_count += case.sampled_transitions.len();
        for transition in &case.sampled_transitions {
            let sequence = &case.legal_sequences[transition.sequence_index];
            assert_eq!(transition.boards.len(), sequence.len() + 1, "{}", case.id);
            let mut current = board.clone();
            assert_eq!(
                current,
                transition.boards[0].board(),
                "initial transition board: {}",
                case.id
            );
            for (i, step) in sequence.iter().enumerate() {
                current = backgammon::apply_move(&current, step.movement(), color);
                assert_eq!(
                    current,
                    transition.boards[i + 1].board(),
                    "transition {} of {}",
                    i,
                    case.id
                );
            }
        }

        let hard: Vec<Vec<(u8, u8, u8)>> = case
            .hard_best_sequences
            .iter()
            .map(|s| s.iter().map(MoveInput::key).collect())
            .collect();
        let picked =
            backgammon::choose_sequence(&board, &case.dice, color, Difficulty::Hard, 0x5eed);
        assert!(
            hard.contains(&signature(&picked)),
            "hard heuristic choice must be among oracle maxima: {}",
            case.id
        );
    }
    assert_eq!(sequence_count, fixture.counts.sequences);
    assert_eq!(transition_count, fixture.counts.transition_samples);
}
