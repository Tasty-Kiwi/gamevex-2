//! Presentation-only card flights for Hearts. This module never changes game state.
use bevy::prelude::Vec2;
use gamevex_core::{
    cards::Card,
    hearts::{Game, PassDirection},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Hand(usize),
    Trick(usize),
    Deck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Pass,
    Collect,
    Play,
    Deal,
}

#[derive(Clone, Copy, Debug)]
pub struct Flight {
    pub card: Card,
    pub from: Anchor,
    pub to: Anchor,
    pub from_position: Option<Vec2>,
    pub elapsed: f32,
    pub delay: f32,
    pub duration: f32,
    pub kind: Kind,
}

impl Flight {
    /// Smoothly eased progress, including the flight's stagger delay.
    pub fn progress(&self) -> f32 {
        if !self.elapsed.is_finite() || !self.delay.is_finite() || !self.duration.is_finite() {
            return 0.0;
        }
        let duration = self.duration.max(f32::EPSILON);
        let t = ((self.elapsed - self.delay) / duration).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

#[derive(Clone, Debug, Default)]
pub struct HeartsMotion {
    flights: Vec<Flight>,
    kind: Option<Kind>,
}

impl HeartsMotion {
    pub fn busy(&self) -> bool {
        self.flights.iter().any(|flight| flight.progress() < 1.0)
    }
    pub fn kind(&self) -> Option<Kind> {
        self.kind
    }
    pub fn flights(&self) -> &[Flight] {
        &self.flights
    }
    pub fn clear(&mut self) {
        self.flights.clear();
        self.kind = None;
    }
    pub fn tick(&mut self, dt: f32) {
        if dt.is_finite() && dt > 0.0 {
            for flight in &mut self.flights {
                flight.elapsed += dt;
            }
            if !self.flights.is_empty() && self.flights.iter().all(|f| f.progress() >= 1.0) {
                self.clear();
            }
        }
    }

    /// Animate each card whose owning hand changed during a pass.
    pub fn pass(&mut self, before: &Game, after: &Game, human_positions: &[Vec2]) {
        self.clear();
        self.kind = Some(Kind::Pass);
        let direction = before.pass_direction();
        if direction == PassDirection::Hold {
            return;
        }

        let same = |a: Card, b: Card| a.rank == b.rank && a.suit == b.suit;
        let mut outgoing = Vec::new();
        for source in 0..4 {
            for card in before.hands[source]
                .iter()
                .copied()
                .filter(|card| !after.hands[source].iter().any(|other| same(*card, *other)))
            {
                let recipient = (0..4).find(|&seat| {
                    seat != source
                        && after.hands[seat].iter().any(|other| same(card, *other))
                        && !before.hands[seat].iter().any(|other| same(card, *other))
                });
                if let Some(recipient) = recipient {
                    let card = after.hands[recipient]
                        .iter()
                        .copied()
                        .find(|other| same(card, *other))
                        .unwrap_or(card);
                    outgoing.push((source, recipient, card));
                }
            }
        }
        let count = outgoing.len();
        for (i, (source, recipient, card)) in outgoing.into_iter().enumerate() {
            let source_index = before.hands[source]
                .iter()
                .position(|candidate| same(*candidate, card));
            self.flights.push(Flight {
                card,
                from: Anchor::Hand(source),
                to: Anchor::Hand(recipient),
                from_position: if source == 0 {
                    source_index.and_then(|index| human_positions.get(index).copied())
                } else {
                    None
                },
                elapsed: 0.0,
                delay: stagger(i, count, 0.12),
                duration: 0.60,
                kind: Kind::Pass,
            });
        }
    }

    /// Animate the visible trick into the winner's hand.
    pub fn collect(&mut self, before: &Game, winner: usize) {
        self.clear();
        self.kind = Some(Kind::Collect);
        let count = before.trick.len();
        for (i, play) in before.trick.iter().enumerate() {
            self.flights.push(Flight {
                card: play.card,
                from: Anchor::Trick(play.seat),
                to: Anchor::Hand(winner),
                from_position: None,
                elapsed: 0.0,
                delay: stagger(i, count, 0.10),
                duration: 0.42,
                kind: Kind::Collect,
            });
        }
    }

    pub fn play(&mut self, seat: usize, card: Card, from_position: Option<Vec2>) {
        self.clear();
        self.kind = Some(Kind::Play);
        self.flights.push(Flight {
            card,
            from: Anchor::Hand(seat),
            to: Anchor::Trick(seat),
            from_position,
            elapsed: 0.0,
            delay: 0.0,
            duration: 0.28,
            kind: Kind::Play,
        });
    }

    pub fn deal(&mut self, game: &Game) {
        self.clear();
        self.kind = Some(Kind::Deal);
        let cards: Vec<_> = (0..13)
            .flat_map(|index| {
                game.hands
                    .iter()
                    .enumerate()
                    .filter_map(move |(seat, hand)| {
                        hand.get(index).copied().map(|card| (seat, card))
                    })
            })
            .collect();
        let count = cards.len();
        for (i, (seat, card)) in cards.into_iter().enumerate() {
            self.flights.push(Flight {
                card,
                from: Anchor::Deck,
                to: Anchor::Hand(seat),
                from_position: None,
                elapsed: 0.0,
                delay: stagger(i, count, 0.48),
                duration: 0.36,
                kind: Kind::Deal,
            });
        }
    }
}

fn stagger(index: usize, count: usize, max: f32) -> f32 {
    if count <= 1 {
        0.0
    } else {
        max * index as f32 / (count - 1) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamevex_core::hearts::Play;

    #[test]
    fn pass_tracks_each_real_transfer_for_left_right_and_across() {
        for hand_number in [1, 2, 3] {
            let mut before = Game::new(100 + hand_number as u64);
            before.hand_number = hand_number;
            let mut after = before.clone();
            assert!(after.pass(&[0, 1, 2]));
            let mut motion = HeartsMotion::default();
            motion.pass(&before, &after, &[Vec2::new(7.0, 9.0); 13]);
            assert_eq!(motion.flights.len(), 12);
            for flight in &motion.flights {
                let (Anchor::Hand(source), Anchor::Hand(destination)) = (flight.from, flight.to)
                else {
                    panic!("pass anchors")
                };
                assert!(
                    before.hands[source]
                        .iter()
                        .any(|c| c.rank == flight.card.rank && c.suit == flight.card.suit)
                );
                assert!(
                    after.hands[destination]
                        .iter()
                        .any(|c| c.rank == flight.card.rank && c.suit == flight.card.suit)
                );
                assert_eq!(
                    flight.to,
                    Anchor::Hand(match before.pass_direction() {
                        PassDirection::Left => (source + 1) % 4,
                        PassDirection::Right => (source + 3) % 4,
                        PassDirection::Across => (source + 2) % 4,
                        PassDirection::Hold => source,
                    })
                );
            }
            assert_eq!(
                motion
                    .flights
                    .iter()
                    .filter(|f| f.from_position.is_some())
                    .count(),
                3
            );
            assert!(
                motion
                    .flights
                    .iter()
                    .all(|f| f.delay + f.duration <= 0.72001)
            );
        }
    }

    #[test]
    fn collection_uses_each_play_seat_and_winner_destination() {
        let mut game = Game::new(88);
        let cards: Vec<_> = game.hands.iter().map(|h| h[0]).collect();
        game.trick = cards
            .into_iter()
            .enumerate()
            .map(|(seat, card)| Play { seat, card })
            .collect();
        let mut motion = HeartsMotion::default();
        motion.collect(&game, 3);
        assert_eq!(motion.flights.len(), 4);
        for (seat, flight) in motion.flights.iter().enumerate() {
            assert_eq!(flight.from, Anchor::Trick(seat));
            assert_eq!(flight.to, Anchor::Hand(3));
        }
        assert!(
            motion
                .flights
                .iter()
                .all(|f| f.delay + f.duration <= 0.52001)
        );
    }

    #[test]
    fn play_deal_and_elapsed_time_are_bounded_and_finite() {
        let game = Game::new(17);
        let mut motion = HeartsMotion::default();
        motion.deal(&game);
        assert_eq!(motion.flights.len(), 52);
        assert!(motion.flights.iter().all(|f| f.delay.is_finite()
            && f.duration.is_finite()
            && f.delay + f.duration <= 0.84001));
        motion.tick(f32::NAN);
        motion.tick(f32::INFINITY);
        assert!(motion.busy());
        motion.tick(100.0);
        assert!(!motion.busy());
        motion.play(2, game.hands[2][0], None);
        assert_eq!(motion.flights.len(), 1);
        motion.tick(1.0);
        assert!(!motion.busy());
        assert!(motion.flights.is_empty());
    }
}
