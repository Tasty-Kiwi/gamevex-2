use super::roulette::{self, Bet, MAX_BALANCE, MAX_STAKE, PocketColor};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    schema_version: u32,
    #[serde(rename = "sourceSHA256")]
    source_sha256: String,
    counts: Counts,
    cases: Vec<Case>,
    rust_only_column_cases: Vec<ColumnCase>,
}
#[derive(Deserialize)]
struct Counts {
    bets: usize,
    pockets: usize,
    cases: usize,
    balance: u64,
    stake: u64,
}
#[derive(Deserialize)]
struct Case {
    bet: BetInput,
    number: u8,
    color: String,
    label: String,
    multiplier: u64,
    expected: Expected,
}
#[derive(Deserialize)]
struct ColumnCase {
    bet: Bet,
    number: u8,
    label: String,
    multiplier: u64,
    won: bool,
    payout: u64,
    net: i64,
    balance: u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BetInput {
    #[serde(rename = "type")]
    kind: String,
    number: Option<u8>,
    stake: u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Expected {
    payout: u64,
    net: i64,
    next_balance: u64,
    did_win: bool,
}

fn bet(input: &BetInput) -> Bet {
    match input.kind.as_str() {
        "number" => Bet::Number(input.number.expect("number bet has number")),
        "red" => Bet::Red,
        "black" => Bet::Black,
        "odd" => Bet::Odd,
        "even" => Bet::Even,
        "low" => Bet::Low,
        "high" => Bet::High,
        "dozen1" => Bet::Dozen1,
        "dozen2" => Bet::Dozen2,
        "dozen3" => Bet::Dozen3,
        "column1" => Bet::Column1,
        "column2" => Bet::Column2,
        "column3" => Bet::Column3,
        other => panic!("unknown oracle bet type {other}"),
    }
}

#[test]
fn rust_only_column_cases_cover_every_pocket_without_claiming_typescript_parity() {
    let fixture = fixture();
    assert_eq!(fixture.rust_only_column_cases.len(), 3 * 37);
    for case in fixture.rust_only_column_cases {
        assert_eq!(case.label, case.bet.label());
        assert_eq!(case.multiplier, case.bet.multiplier());
        let result = roulette::resolve(1_000, 5, case.bet, case.number).unwrap();
        assert_eq!(result.won, case.won, "{:?} on {}", case.bet, case.number);
        assert_eq!(result.payout, case.payout);
        assert_eq!(result.net, case.net);
        assert_eq!(result.balance, case.balance);
    }
    assert!(!Bet::Column1.wins(0));
    assert!(!Bet::Column2.wins(0));
    assert!(!Bet::Column3.wins(0));
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../../tests/fixtures/roulette-oracle.json"))
        .expect("valid generated roulette oracle fixture")
}

#[test]
fn matches_oracle_for_all_bets_and_pockets() {
    let fixture = fixture();
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.source_sha256.len(), 64);
    assert_eq!((fixture.counts.bets, fixture.counts.pockets), (46, 37));
    assert_eq!(fixture.counts.cases, fixture.cases.len());
    assert_eq!(fixture.cases.len(), 1_702);
    let balance = fixture.counts.balance;
    for case in fixture.cases {
        let bet = bet(&case.bet);
        assert_eq!(case.bet.stake, fixture.counts.stake);
        assert_eq!(
            case.color,
            match roulette::color(case.number) {
                PocketColor::Red => "red",
                PocketColor::Black => "black",
                PocketColor::Green => "green",
            }
        );
        assert_eq!(case.label, bet.label());
        assert_eq!(case.multiplier, bet.multiplier());
        let actual = roulette::resolve(balance, case.bet.stake, bet, case.number).unwrap();
        assert_eq!(actual.won, case.expected.did_win);
        assert_eq!(actual.payout, case.expected.payout);
        assert_eq!(actual.net, case.expected.net);
        assert_eq!(actual.balance, case.expected.next_balance);
    }
}

#[test]
fn bounds_cover_small_bankrolls_and_maximum_stake() {
    for balance in 0..=6 {
        assert_eq!(
            roulette::stake_bounds(balance),
            (5.min(balance), balance.min(MAX_STAKE))
        );
    }
    assert_eq!(roulette::stake_bounds(MAX_STAKE + 10), (5, MAX_STAKE));
    assert_eq!(roulette::stake_bounds(MAX_BALANCE), (5, MAX_STAKE));
    // The TS UI's fixed five-credit minimum would make balances 1-4 unplayable.
    // Rust intentionally permits the whole remaining balance as a minimum stake.
    for balance in 1..=4 {
        let (min, max) = roulette::stake_bounds(balance);
        assert_eq!((min, max), (balance, balance));
        assert!(roulette::resolve(balance, balance, Bet::Red, 1).is_some());
    }
}

#[test]
fn maximum_bankroll_is_a_safe_extension_to_oracle_settlement() {
    let result = roulette::resolve(MAX_BALANCE - 1, 5, Bet::Number(1), 1).unwrap();
    assert_eq!(result.balance, MAX_BALANCE);
    assert_eq!(result.payout, 6);
    assert_eq!(result.net, 1);
    assert!(roulette::resolve(MAX_BALANCE + 1, 5, Bet::Red, 1).is_none());
}
