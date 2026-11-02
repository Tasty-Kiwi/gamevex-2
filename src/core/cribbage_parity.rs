use super::{
    cards::{Card, Suit},
    cribbage::{Game, Phase, hand_score},
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
    discard_cases: Vec<DiscardCase>,
    peg_cases: Vec<PegCase>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Counts {
    hands: usize,
    cpu_discards: usize,
    cpu_pegging: usize,
}
#[derive(Deserialize)]
struct HandCase {
    id: String,
    hand: Vec<Card>,
    starter: Card,
    is_crib: bool,
    expected: ExpectedScore,
}
#[derive(Deserialize)]
struct ExpectedScore {
    total: u32,
    breakdown: Vec<ExpectedBreakdown>,
}
#[derive(Deserialize)]
struct ExpectedBreakdown {
    label: String,
    points: u32,
}
#[derive(Deserialize)]
struct DiscardCase {
    id: String,
    dealer: usize,
    hand: Vec<Card>,
    keep: Vec<Card>,
    discard: Vec<Card>,
}
#[derive(Deserialize)]
struct PegCase {
    id: String,
    sequence: Vec<Card>,
    count: u8,
    hand: Vec<Card>,
    expected: Card,
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../../tests/fixtures/cribbage-oracle.json"))
        .expect("generated TS oracle fixture parses")
}
fn suit_order(suit: Suit) -> u8 {
    match suit {
        Suit::Clubs => 0,
        Suit::Diamonds => 1,
        Suit::Hearts => 2,
        Suit::Spades => 3,
    }
}
fn face_set(cards: &[Card]) -> Vec<(u8, u8)> {
    let mut faces: Vec<_> = cards
        .iter()
        .map(|card| (card.rank, suit_order(card.suit)))
        .collect();
    faces.sort_unstable();
    faces
}

#[test]
fn generated_fixture_has_traceable_oracle_metadata_and_expected_corpus_size() {
    let f = fixture();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.oracle, "apps/local/src/utils/cribbage.ts");
    assert!(
        f.oracle_git_revision
            .as_ref()
            .is_some_and(|revision| revision.len() >= 7)
    );
    assert_eq!(f.source_sha256.len(), 64);
    assert_eq!(f.counts.hands, 128);
    assert_eq!(f.hand_cases.len(), 128);
    assert_eq!(f.counts.cpu_discards, 256);
    assert_eq!(f.discard_cases.len(), 256);
    assert_eq!(f.counts.cpu_pegging, 128);
    assert_eq!(f.peg_cases.len(), 128);
}

#[test]
fn rust_hand_scores_match_typescript_oracle_corpus() {
    for case in fixture().hand_cases {
        let actual = hand_score(&case.hand, case.starter, case.is_crib);
        let expected: Vec<_> = case
            .expected
            .breakdown
            .into_iter()
            .map(|entry| (entry.label, entry.points))
            .collect();
        assert_eq!(actual.total, case.expected.total, "{}", case.id);
        assert_eq!(actual.breakdown, expected, "{}", case.id);
    }
}

#[test]
fn cpu_discard_choices_match_typescript_oracle_for_both_dealer_positions() {
    for case in fixture().discard_cases {
        assert!(case.dealer < 2, "{}", case.id);
        let mut game = Game::new(0xC1B_BA6E);
        game.phase = Phase::Discard;
        game.dealer = case.dealer;
        game.hands[1] = case.hand.clone();
        game.remaining[1] = case.hand;
        game.discards = [vec![], vec![]];
        game.crib.clear();
        game.starter = None;
        assert!(game.cpu_step(), "{}", case.id);
        assert_eq!(
            face_set(&game.hands[1]),
            face_set(&case.keep),
            "{} kept",
            case.id
        );
        assert_eq!(
            face_set(&game.discards[1]),
            face_set(&case.discard),
            "{} discarded",
            case.id
        );
    }
}

#[test]
fn cpu_pegging_choices_match_typescript_oracle_corpus() {
    for case in fixture().peg_cases {
        let mut game = Game::new(0xC1B_BA6E);
        game.phase = Phase::Pegging;
        game.turn = 1;
        game.count = case.count;
        game.sequence = case.sequence;
        let hand_len = case.hand.len();
        game.remaining = [vec![], case.hand];
        assert!(game.cpu_step(), "{} CPU step", case.id);
        let expected = face_set(&[case.expected]);
        assert_eq!(
            game.remaining[1].len() + 1,
            hand_len,
            "{} hand count",
            case.id
        );
        let played = game.played.last().copied().expect("CPU played card");
        assert_eq!(face_set(&[played]), expected, "{}", case.id);
    }
}
