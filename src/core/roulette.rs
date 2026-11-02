use serde::{Deserialize, Serialize};

pub const WHEEL_ORDER: [u8; 37] = [
    0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10, 5, 24, 16, 33, 1, 20,
    14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26,
];

pub const MAX_BALANCE: u64 = 1_000_000_000_000;
pub const MAX_STAKE: u64 = 100_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bet {
    Number(u8),
    Red,
    Black,
    Odd,
    Even,
    Low,
    High,
    Dozen1,
    Dozen2,
    Dozen3,
    Column1,
    Column2,
    Column3,
}

impl Bet {
    pub fn label(self) -> String {
        match self {
            Self::Number(n) => format!("STRAIGHT {n}"),
            Self::Red => "RED".into(),
            Self::Black => "BLACK".into(),
            Self::Odd => "ODD".into(),
            Self::Even => "EVEN".into(),
            Self::Low => "1 TO 18".into(),
            Self::High => "19 TO 36".into(),
            Self::Dozen1 => "1ST 12".into(),
            Self::Dozen2 => "2ND 12".into(),
            Self::Dozen3 => "3RD 12".into(),
            Self::Column1 => "COLUMN 1".into(),
            Self::Column2 => "COLUMN 2".into(),
            Self::Column3 => "COLUMN 3".into(),
        }
    }

    /// Profit multiplier; a winning bet returns this amount plus the stake.
    pub fn multiplier(self) -> u64 {
        match self {
            Self::Number(_) => 35,
            Self::Dozen1
            | Self::Dozen2
            | Self::Dozen3
            | Self::Column1
            | Self::Column2
            | Self::Column3 => 2,
            _ => 1,
        }
    }

    pub fn wins(self, number: u8) -> bool {
        if number > 36 {
            return false;
        }
        match self {
            Self::Number(n) => n <= 36 && number == n,
            Self::Red => color(number) == PocketColor::Red,
            Self::Black => color(number) == PocketColor::Black,
            Self::Odd => number != 0 && number % 2 == 1,
            Self::Even => number != 0 && number.is_multiple_of(2),
            Self::Low => (1..=18).contains(&number),
            Self::High => (19..=36).contains(&number),
            Self::Dozen1 => (1..=12).contains(&number),
            Self::Dozen2 => (13..=24).contains(&number),
            Self::Dozen3 => (25..=36).contains(&number),
            Self::Column1 => number != 0 && number % 3 == 1,
            Self::Column2 => number % 3 == 2,
            Self::Column3 => number != 0 && number.is_multiple_of(3),
        }
    }

    pub fn valid(self) -> bool {
        !matches!(self, Self::Number(n) if n > 36)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PocketColor {
    Red,
    Black,
    Green,
}

pub fn color(number: u8) -> PocketColor {
    if number == 0 || number > 36 {
        return PocketColor::Green;
    }
    if matches!(
        number,
        1 | 3 | 5 | 7 | 9 | 12 | 14 | 16 | 18 | 19 | 21 | 23 | 25 | 27 | 30 | 32 | 34 | 36
    ) {
        PocketColor::Red
    } else {
        PocketColor::Black
    }
}

pub fn stake_bounds(balance: u64) -> (u64, u64) {
    let balance = balance.min(MAX_BALANCE);
    (5.min(balance), balance.min(MAX_STAKE))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub payout: u64,
    pub net: i64,
    pub balance: u64,
    pub won: bool,
    pub number: u8,
    pub bet: Bet,
    pub stake: u64,
}

pub fn resolve(balance: u64, stake: u64, bet: Bet, number: u8) -> Option<Resolution> {
    if balance > MAX_BALANCE
        || number > 36
        || !bet.valid()
        || stake == 0
        || stake > balance
        || stake > MAX_STAKE
    {
        return None;
    }
    if stake < 5.min(balance) {
        return None;
    }
    let won = bet.wins(number);
    let payout = if won {
        let gross = stake.checked_mul(bet.multiplier().checked_add(1)?)?;
        // Keep the bankroll bounded while preserving the actual credited amount.
        gross.min(stake.checked_add(MAX_BALANCE - balance)?)
    } else {
        0
    };
    let net_i128 = payout as i128 - stake as i128;
    let next = balance as i128 + net_i128;
    if next < 0 || next > MAX_BALANCE as i128 {
        return None;
    }
    Some(Resolution {
        payout,
        net: i64::try_from(net_i128).ok()?,
        balance: next as u64,
        won,
        number,
        bet,
        stake,
    })
}

/// Deterministic SplitMix64 spin with rejection sampling to avoid modulo bias.
pub fn spin(seed: u64) -> u8 {
    let mut state = seed;
    loop {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut x = state;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        x ^= x >> 31;
        let zone = u64::MAX - (u64::MAX % WHEEL_ORDER.len() as u64);
        if x < zone {
            return WHEEL_ORDER[(x % WHEEL_ORDER.len() as u64) as usize];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bets() -> Vec<Bet> {
        let mut bets: Vec<_> = (0..=36).map(Bet::Number).collect();
        bets.extend([
            Bet::Red,
            Bet::Black,
            Bet::Odd,
            Bet::Even,
            Bet::Low,
            Bet::High,
            Bet::Dozen1,
            Bet::Dozen2,
            Bet::Dozen3,
            Bet::Column1,
            Bet::Column2,
            Bet::Column3,
        ]);
        bets
    }

    #[test]
    fn exhaustive_settlement_is_bounded_and_consistent() {
        for bet in bets() {
            for number in 0..=36 {
                let result = resolve(1_000, 5, bet, number).unwrap();
                assert_eq!(result.won, bet.wins(number));
                assert_eq!(result.balance as i64, 1_000 + result.net);
                assert_eq!(
                    result.net,
                    if result.won {
                        (result.payout - 5) as i64
                    } else {
                        -5
                    }
                );
            }
        }
    }

    #[test]
    fn rejects_bad_inputs_and_caps_maximum_win() {
        assert!(resolve(10, 4, Bet::Red, 1).is_none());
        assert!(resolve(10, 11, Bet::Red, 1).is_none());
        assert!(resolve(MAX_BALANCE + 1, 5, Bet::Red, 1).is_none());
        assert!(resolve(100, 5, Bet::Number(37), 1).is_none());
        let r = resolve(MAX_BALANCE - 1, 5, Bet::Number(1), 1).unwrap();
        assert_eq!(r.balance, MAX_BALANCE);
        assert_eq!(r.payout, 6);
        assert_eq!(r.net, 1);
    }

    #[test]
    fn wheel_has_each_pocket_once_and_spin_returns_valid_pocket() {
        let mut wheel = WHEEL_ORDER;
        wheel.sort_unstable();
        assert_eq!(wheel, std::array::from_fn(|i| i as u8));
        for seed in 0..10_000 {
            assert!(spin(seed) <= 36);
        }
        assert_eq!(spin(0), 12);
        assert_eq!(spin(1), 9);
        assert_eq!(spin(42), 8);
        assert_eq!(spin(0x0123_4567_89ab_cdef), 16);
        assert_eq!(spin(u64::MAX), 11);
    }
}
