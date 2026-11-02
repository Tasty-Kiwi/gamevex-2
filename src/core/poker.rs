use super::cards::{Card, Suit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub score: Vec<u8>,
    pub cards: Vec<Card>,
    pub highlights: Vec<Card>,
}

impl Evaluation {
    pub fn label(&self) -> &'static str {
        match self.score.first().copied().unwrap_or(0) {
            1 => "HIGH CARD",
            2 => "PAIR",
            3 => "TWO PAIR",
            4 => "THREE OF A KIND",
            5 => "STRAIGHT",
            6 => "FLUSH",
            7 => "FULL HOUSE",
            8 => "FOUR OF A KIND",
            9 => "STRAIGHT FLUSH",
            _ => "",
        }
    }
}

/// Evaluate a five through seven card Texas Hold'em holding. Invalid ranks,
/// duplicate physical cards, and inputs outside that size range return None.
pub fn evaluate(cards: &[Card]) -> Option<Evaluation> {
    if !(5..=7).contains(&cards.len()) || cards.iter().any(|c| !(2..=14).contains(&c.rank)) {
        return None;
    }
    let mut seen = [[false; 13]; 4];
    for card in cards {
        let s = suit_index(card.suit);
        let r = usize::from(card.rank - 2);
        if seen[s][r] {
            return None;
        }
        seen[s][r] = true;
    }
    let mut best: Option<Evaluation> = None;
    for a in 0..cards.len() - 4 {
        for b in a + 1..cards.len() - 3 {
            for c in b + 1..cards.len() - 2 {
                for d in c + 1..cards.len() - 1 {
                    for e in d + 1..cards.len() {
                        let hand = [cards[a], cards[b], cards[c], cards[d], cards[e]];
                        let candidate = evaluate_five(&hand);
                        if best.as_ref().is_none_or(|old| candidate.score > old.score) {
                            best = Some(candidate);
                        }
                    }
                }
            }
        }
    }
    best
}

fn suit_index(s: Suit) -> usize {
    match s {
        Suit::Clubs => 0,
        Suit::Diamonds => 1,
        Suit::Hearts => 2,
        Suit::Spades => 3,
    }
}

fn evaluate_five(cards: &[Card; 5]) -> Evaluation {
    let mut values: Vec<u8> = cards.iter().map(|c| c.rank).collect();
    values.sort_unstable_by(|a, b| b.cmp(a));
    let mut counts: HashMap<u8, u8> = HashMap::new();
    for &v in &values {
        *counts.entry(v).or_default() += 1;
    }
    let mut grouped: Vec<(u8, u8)> = counts.iter().map(|(&v, &n)| (v, n)).collect();
    grouped.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
    let unique: Vec<u8> = grouped.iter().map(|(v, _)| *v).collect();
    let flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let straight = straight_high(&unique);
    let (rank, tie, selected_ranks): (u8, Vec<u8>, Vec<u8>) =
        if let (true, Some(high)) = (flush, straight) {
            (9, vec![high], values.clone())
        } else if grouped[0].1 == 4 {
            (8, vec![grouped[0].0, grouped[1].0], vec![grouped[0].0])
        } else if grouped[0].1 == 3 && grouped[1].1 == 2 {
            (
                7,
                vec![grouped[0].0, grouped[1].0],
                vec![grouped[0].0, grouped[1].0],
            )
        } else if flush {
            (6, unique.clone(), values.clone())
        } else if let Some(high) = straight {
            (5, vec![high], values.clone())
        } else if grouped[0].1 == 3 {
            (
                4,
                std::iter::once(grouped[0].0)
                    .chain(unique.iter().copied().filter(|v| *v != grouped[0].0))
                    .collect(),
                vec![grouped[0].0],
            )
        } else if grouped[0].1 == 2 && grouped[1].1 == 2 {
            let pairs: Vec<u8> = grouped
                .iter()
                .filter(|(_, n)| *n == 2)
                .map(|(v, _)| *v)
                .collect();
            let kicker = unique
                .iter()
                .copied()
                .find(|v| !pairs.contains(v))
                .unwrap_or(0);
            (3, vec![pairs[0], pairs[1], kicker], pairs)
        } else if grouped[0].1 == 2 {
            (
                2,
                std::iter::once(grouped[0].0)
                    .chain(unique.iter().copied().filter(|v| *v != grouped[0].0))
                    .collect(),
                vec![grouped[0].0],
            )
        } else {
            (1, unique.clone(), vec![unique[0]])
        };
    let highlights = cards
        .iter()
        .copied()
        .filter(|c| selected_ranks.contains(&c.rank))
        .collect();
    Evaluation {
        score: std::iter::once(rank).chain(tie).collect(),
        cards: cards.to_vec(),
        highlights,
    }
}

fn straight_high(unique: &[u8]) -> Option<u8> {
    if unique.len() != 5 {
        return None;
    }
    if unique[0] - unique[4] == 4 {
        Some(unique[0])
    } else if unique == [14, 5, 4, 3, 2] {
        Some(5)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(rank: u8, suit: Suit) -> Card {
        Card::new(rank, suit, true)
    }
    fn hand(cards: Vec<Card>, rank: u8, tie: &[u8]) -> Evaluation {
        let ev = evaluate(&cards).unwrap();
        assert_eq!(
            ev.score,
            std::iter::once(rank)
                .chain(tie.iter().copied())
                .collect::<Vec<_>>()
        );
        ev
    }
    #[test]
    fn recognizes_all_nine_categories_and_kickers() {
        let s = Suit::Spades;
        let h = Suit::Hearts;
        let d = Suit::Diamonds;
        let clubs = Suit::Clubs;
        assert_eq!(
            hand(
                vec![c(14, s), c(11, h), c(9, d), c(7, clubs), c(3, s)],
                1,
                &[14, 11, 9, 7, 3]
            )
            .label(),
            "HIGH CARD"
        );
        hand(
            vec![c(14, s), c(14, h), c(9, d), c(7, clubs), c(3, s)],
            2,
            &[14, 9, 7, 3],
        );
        hand(
            vec![c(14, s), c(14, h), c(9, d), c(9, clubs), c(3, s)],
            3,
            &[14, 9, 3],
        );
        hand(
            vec![c(14, s), c(14, h), c(14, d), c(9, clubs), c(3, s)],
            4,
            &[14, 9, 3],
        );
        hand(
            vec![c(10, s), c(9, h), c(8, d), c(7, clubs), c(6, s)],
            5,
            &[10],
        );
        hand(
            vec![c(14, s), c(11, s), c(9, s), c(7, s), c(3, s)],
            6,
            &[14, 11, 9, 7, 3],
        );
        hand(
            vec![c(14, s), c(14, h), c(14, d), c(9, clubs), c(9, s)],
            7,
            &[14, 9],
        );
        hand(
            vec![c(14, s), c(14, h), c(14, d), c(14, clubs), c(9, s)],
            8,
            &[14, 9],
        );
        hand(vec![c(10, s), c(9, s), c(8, s), c(7, s), c(6, s)], 9, &[10]);
    }
    #[test]
    fn wheel_two_trips_full_house_flush_kicker_and_board_only() {
        let s = Suit::Spades;
        let h = Suit::Hearts;
        let d = Suit::Diamonds;
        let clubs = Suit::Clubs;
        hand(
            vec![c(14, s), c(2, h), c(3, d), c(4, clubs), c(5, s)],
            5,
            &[5],
        );
        hand(
            vec![
                c(14, s),
                c(14, h),
                c(14, d),
                c(13, s),
                c(13, h),
                c(13, d),
                c(2, clubs),
            ],
            7,
            &[14, 13],
        );
        hand(
            vec![
                c(14, s),
                c(14, h),
                c(14, d),
                c(13, s),
                c(13, h),
                c(13, d),
                c(12, clubs),
            ],
            7,
            &[14, 13],
        );
        hand(
            vec![
                c(14, s),
                c(11, s),
                c(9, s),
                c(7, s),
                c(3, s),
                c(2, s),
                c(13, h),
            ],
            6,
            &[14, 11, 9, 7, 3],
        );
        let board = vec![
            c(14, s),
            c(13, s),
            c(12, s),
            c(11, s),
            c(10, s),
            c(2, h),
            c(3, d),
        ];
        let ev = hand(board.clone(), 9, &[14]);
        assert_eq!(ev.cards, board[..5]);
    }
    #[test]
    fn rejects_bad_sizes_ranks_and_duplicates() {
        assert!(evaluate(&[]).is_none());
        assert!(evaluate(&[c(2, Suit::Spades); 4]).is_none());
        assert!(evaluate(&[c(2, Suit::Spades); 8]).is_none());
        assert!(
            evaluate(&[
                c(1, Suit::Spades),
                c(2, Suit::Hearts),
                c(3, Suit::Clubs),
                c(4, Suit::Diamonds),
                c(5, Suit::Spades)
            ])
            .is_none()
        );
        assert!(
            evaluate(&[
                c(2, Suit::Spades),
                c(2, Suit::Spades),
                c(3, Suit::Clubs),
                c(4, Suit::Diamonds),
                c(5, Suit::Hearts)
            ])
            .is_none()
        );
    }
}
