use super::{
    cards::Card,
    holdem::{Action, Game, Phase, Seat},
    poker::evaluate,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    schema_version: u32,
    oracle: String,
    oracle_git_revision: Option<String>,
    #[serde(rename = "sourceSHA256")]
    source_sha256: String,
    counts: Counts,
    hand_cases: Vec<HandCase>,
    cpu_cases: Vec<CpuCase>,
    transition_cases: Vec<TransitionCase>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Counts {
    known_hands: usize,
    random_seven_card_hands: usize,
    cpu_action_cases: usize,
    cpu_branch_policy: String,
    transition_cases: usize,
}
#[derive(Deserialize)]
struct HandCase {
    id: String,
    cards: Vec<Card>,
    expected: EvaluationWithLabel,
}
#[derive(Deserialize)]
struct EvaluationWithLabel {
    score: Vec<u8>,
    cards: Vec<Card>,
    highlights: Vec<Card>,
    label: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CpuCase {
    id: String,
    phase: String,
    actor: usize,
    pressure: f32,
    cards: CpuCards,
    table: CpuTable,
    expected: CpuExpected,
}
#[derive(Deserialize)]
struct CpuCards {
    board: Vec<Card>,
    deck: Vec<Card>,
    burned: Vec<Card>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CpuTable {
    buy_in: u32,
    dealer_index: usize,
    small_blind_index: usize,
    big_blind_index: usize,
    acting_seat_index: Option<usize>,
    action_queue: Vec<usize>,
    current_bet: u32,
    last_full_raise: u32,
    hand_number: u32,
    seats: Vec<CpuSeat>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CpuSeat {
    stack: u32,
    hole_cards: Vec<Card>,
    folded: bool,
    is_all_in: bool,
    street_committed: u32,
    hand_committed: u32,
    can_raise: bool,
    last_action: String,
}
#[derive(Deserialize)]
struct CpuExpected {
    action: CpuAction,
    phase: String,
    #[serde(rename = "actingSeatIndex")]
    acting_seat_index: Option<usize>,
    #[serde(rename = "actionQueue")]
    action_queue: Vec<usize>,
    #[serde(rename = "currentBet")]
    current_bet: u32,
    #[serde(rename = "lastFullRaise")]
    last_full_raise: u32,
    board: Vec<Card>,
    pot: u32,
    seats: Vec<CpuExpectedSeat>,
}
#[derive(Deserialize)]
struct CpuAction {
    #[serde(rename = "type")]
    kind: String,
    total: Option<u32>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CpuExpectedSeat {
    stack: u32,
    folded: bool,
    street_committed: u32,
    hand_committed: u32,
    can_raise: bool,
    last_action: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransitionCase {
    id: String,
    action: CpuAction,
    board: Vec<Card>,
    deck: Vec<Card>,
    burned: Vec<Card>,
    phase: String,
    table: CpuTable,
    expected: TransitionExpected,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransitionExpected {
    phase: String,
    acting_seat_index: Option<usize>,
    action_queue: Vec<usize>,
    current_bet: u32,
    last_full_raise: u32,
    board: Vec<Card>,
    pot: u32,
    seats: Vec<CpuExpectedSeat>,
}
fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../../tests/fixtures/holdem-oracle.json"))
        .expect("generated Hold'em oracle fixture parses")
}

#[test]
fn fixture_records_traceable_oracle_and_case_counts() {
    let f = fixture();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.oracle, "packages/shared/src/games/holdem/engine.ts");
    assert!(f.oracle_git_revision.as_ref().is_some_and(|r| r.len() >= 7));
    assert_eq!(f.source_sha256.len(), 64);
    assert_eq!(f.counts.known_hands, 11);
    assert_eq!(f.counts.random_seven_card_hands, 512);
    assert_eq!(f.counts.cpu_action_cases, 128);
    assert!(
        f.counts
            .cpu_branch_policy
            .contains("randomized branches excluded")
    );
    assert_eq!(f.hand_cases.len(), 523);
    assert_eq!(f.cpu_cases.len(), 128);
    assert_eq!(f.counts.transition_cases, 32);
    assert_eq!(f.transition_cases.len(), 32);
}

#[test]
fn normal_action_transitions_match_typescript_oracle_snapshots() {
    let f = fixture();
    let mut actions = [0usize; 4];
    for case in f.transition_cases {
        let seats: Vec<_> = case
            .table
            .seats
            .iter()
            .map(|s| Seat {
                stack: s.stack,
                hole: s.hole_cards.clone(),
                folded: s.folded,
                all_in: s.is_all_in,
                street_committed: s.street_committed,
                committed: s.hand_committed,
                can_raise: s.can_raise,
                last_action: s.last_action.clone(),
            })
            .collect();
        let mut game = Game {
            seats,
            board: case.board,
            deck: case.deck,
            burned: case.burned,
            buy_in: case.table.buy_in,
            dealer: case.table.dealer_index,
            small_blind: case.table.small_blind_index,
            big_blind: case.table.big_blind_index,
            turn: case.table.acting_seat_index,
            queue: case.table.action_queue,
            current_bet: case.table.current_bet,
            last_full_raise: case.table.last_full_raise,
            hand_number: case.table.hand_number,
            payouts: vec![0; case.table.seats.len()],
            winners: vec![false; case.table.seats.len()],
            evaluations: vec![None; case.table.seats.len()],
            winner_text: String::new(),
            log: vec![],
            rng_state: 1,
            phase: phase(&case.phase),
        };
        let action = match case.action.kind.as_str() {
            "check" => {
                actions[0] += 1;
                Action::Check
            }
            "call" => {
                actions[1] += 1;
                Action::Call
            }
            "fold" => {
                actions[2] += 1;
                Action::Fold
            }
            "bet" => {
                actions[3] += 1;
                Action::Bet(case.action.total.unwrap())
            }
            other => panic!("{} unexpected transition action {other}", case.id),
        };
        assert!(game.apply(action), "{} action rejected", case.id);
        let want = &case.expected;
        let expected_queue = normalized_raise_queue(
            &want.action_queue,
            case.table.acting_seat_index,
            &want.seats,
        );
        assert_eq!(game.phase, phase(&want.phase), "{} phase", case.id);
        assert_eq!(game.turn, want.acting_seat_index, "{} turn", case.id);
        assert_eq!(game.queue, expected_queue, "{} queue", case.id);
        assert_eq!(
            game.current_bet, want.current_bet,
            "{} current bet",
            case.id
        );
        assert_eq!(
            game.last_full_raise, want.last_full_raise,
            "{} last raise",
            case.id
        );
        assert_eq!(game.board, want.board, "{} board", case.id);
        assert_eq!(game.pot(), want.pot, "{} pot", case.id);
        assert_eq!(game.seats.len(), want.seats.len(), "{} seat count", case.id);
        for (seat_index, (actual, expected)) in game.seats.iter().zip(&want.seats).enumerate() {
            assert_eq!(
                actual.stack, expected.stack,
                "{} seat {seat_index} stack",
                case.id
            );
            assert_eq!(
                actual.folded, expected.folded,
                "{} seat {seat_index} folded",
                case.id
            );
            assert_eq!(
                actual.street_committed, expected.street_committed,
                "{} seat {seat_index} street committed",
                case.id
            );
            assert_eq!(
                actual.committed, expected.hand_committed,
                "{} seat {seat_index} committed",
                case.id
            );
            assert_eq!(
                actual.can_raise, expected.can_raise,
                "{} seat {seat_index} can raise",
                case.id
            );
            assert_eq!(
                actual.last_action, expected.last_action,
                "{} seat {seat_index} action",
                case.id
            );
        }
    }
    assert_eq!(actions, [8, 8, 8, 8]);
}

fn normalized_raise_queue(
    oracle_queue: &[usize],
    actor: Option<usize>,
    expected_seats: &[CpuExpectedSeat],
) -> Vec<usize> {
    let Some(actor) = actor else {
        return oracle_queue.to_vec();
    };
    let is_raise = expected_seats
        .get(actor)
        .is_some_and(|seat| matches!(seat.last_action.as_str(), "BET" | "RAISE"));
    if !is_raise || !oracle_queue.contains(&actor) {
        return oracle_queue.to_vec();
    }
    // The TS full-raise queue retains the bettor because it filters only for
    // seat activity, making the aggressor eligible to act again immediately.
    // Rust excludes the aggressor to preserve one action per turn.
    oracle_queue
        .iter()
        .copied()
        .filter(|seat| *seat != actor)
        .collect()
}

#[test]
fn cpu_action_and_transition_matches_typescript_oracle_after_queue_normalization() {
    let f = fixture();
    let mut actions = [0usize; 6];
    let mut phases = [0usize; 4];
    for case in f.cpu_cases {
        assert!(
            case.pressure >= 0.32,
            "{} enters a randomized action branch",
            case.id
        );
        phases[match case.phase.as_str() {
            "preflop" => 0,
            "flop" => 1,
            "turn" => 2,
            "river" => 3,
            _ => panic!("unknown phase {}", case.phase),
        }] += 1;
        let seats: Vec<_> = case
            .table
            .seats
            .iter()
            .map(|s| Seat {
                stack: s.stack,
                hole: s.hole_cards.clone(),
                folded: s.folded,
                all_in: s.is_all_in,
                street_committed: s.street_committed,
                committed: s.hand_committed,
                can_raise: s.can_raise,
                last_action: s.last_action.clone(),
            })
            .collect();
        let mut game = Game {
            seats,
            board: case.cards.board,
            deck: case.cards.deck,
            burned: case.cards.burned,
            buy_in: case.table.buy_in,
            dealer: case.table.dealer_index,
            small_blind: case.table.small_blind_index,
            big_blind: case.table.big_blind_index,
            turn: case.table.acting_seat_index,
            queue: case.table.action_queue,
            current_bet: case.table.current_bet,
            last_full_raise: case.table.last_full_raise,
            hand_number: case.table.hand_number,
            payouts: vec![0; case.table.seats.len()],
            winners: vec![false; case.table.seats.len()],
            evaluations: vec![None; case.table.seats.len()],
            winner_text: String::new(),
            log: vec![],
            rng_state: case.table.hand_number.max(1) as u64,
            phase: phase(&case.phase),
        };
        assert_eq!(game.turn, Some(case.actor), "{} actor", case.id);
        assert!(game.cpu_step(), "{} CPU step rejected", case.id);
        let actual_action = game.seats[case.actor].last_action.as_str();
        let expected_action = match case.expected.action.kind.as_str() {
            "fold" => "FOLD",
            "check" => "CHECK",
            "call" => "CALL",
            "bet" => "BET",
            "raise" => "RAISE",
            "all-in" => "ALL-IN",
            other => panic!("{} unknown oracle action {other}", case.id),
        };
        actions[match case.expected.action.kind.as_str() {
            "fold" => 0,
            "check" => 1,
            "call" => 2,
            "bet" => 3,
            "raise" => 4,
            _ => 5,
        }] += 1;
        assert_eq!(actual_action, expected_action, "{} action", case.id);
        let expected_queue = normalized_raise_queue(
            &case.expected.action_queue,
            Some(case.actor),
            &case.expected.seats,
        );
        assert_eq!(game.phase, phase(&case.expected.phase), "{} phase", case.id);
        assert_eq!(
            game.turn, case.expected.acting_seat_index,
            "{} turn",
            case.id
        );
        assert_eq!(game.queue, expected_queue, "{} queue", case.id);
        assert_eq!(
            game.current_bet, case.expected.current_bet,
            "{} current bet",
            case.id
        );
        assert_eq!(
            game.last_full_raise, case.expected.last_full_raise,
            "{} last raise",
            case.id
        );
        assert_eq!(game.board, case.expected.board, "{} board", case.id);
        assert_eq!(game.pot(), case.expected.pot, "{} pot", case.id);
        assert_eq!(
            game.seats.len(),
            case.expected.seats.len(),
            "{} seat count",
            case.id
        );
        for (seat_index, (actual, expected)) in
            game.seats.iter().zip(&case.expected.seats).enumerate()
        {
            assert_eq!(
                actual.stack, expected.stack,
                "{} seat {seat_index} stack",
                case.id
            );
            assert_eq!(
                actual.folded, expected.folded,
                "{} seat {seat_index} folded",
                case.id
            );
            assert_eq!(
                actual.street_committed, expected.street_committed,
                "{} seat {seat_index} street committed",
                case.id
            );
            assert_eq!(
                actual.committed, expected.hand_committed,
                "{} seat {seat_index} committed",
                case.id
            );
            assert_eq!(
                actual.can_raise, expected.can_raise,
                "{} seat {seat_index} can raise",
                case.id
            );
            assert_eq!(
                actual.last_action, expected.last_action,
                "{} seat {seat_index} action",
                case.id
            );
        }
    }
    assert_eq!(phases, [32, 32, 32, 32]);
    assert_eq!(actions, [96, 0, 22, 0, 10, 0]);
}

fn phase(value: &str) -> Phase {
    match value {
        "preflop" => Phase::Preflop,
        "flop" => Phase::Flop,
        "turn" => Phase::Turn,
        "river" => Phase::River,
        "showdown" => Phase::Showdown,
        "handComplete" => Phase::Complete,
        _ => Phase::Idle,
    }
}

#[test]
fn evaluator_matches_typescript_oracle_scores_cards_highlights_and_labels() {
    for case in fixture().hand_cases {
        let actual =
            evaluate(&case.cards).unwrap_or_else(|| panic!("oracle case {} rejected", case.id));
        assert_eq!(actual.score, case.expected.score, "{} score", case.id);
        assert_eq!(
            actual.cards, case.expected.cards,
            "{} chosen cards",
            case.id
        );
        assert_eq!(
            actual.highlights, case.expected.highlights,
            "{} highlights",
            case.id
        );
        assert_eq!(actual.label(), case.expected.label, "{} label", case.id);
    }
}
