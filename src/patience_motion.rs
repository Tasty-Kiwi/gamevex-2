//! Lightweight, presentation-only card flights for patience games.
use gamevex_core::{
    cards::Card,
    patience::{Game, Pile},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct CardFlight {
    pub card: Card,
    pub from: Pile,
    pub from_stack_y: f32,
    pub to: Pile,
    pub to_index: usize,
    pub to_stack_y: f32,
    pub elapsed: f32,
    pub delay: f32,
    pub duration: f32,
    pub flip: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PatienceMotion {
    flights: Vec<CardFlight>,
}

#[derive(Clone, Debug, Default)]
struct TableSnapshot {
    piles: Vec<(Pile, Vec<Card>)>,
}

impl TableSnapshot {
    fn capture(g: &Game) -> Self {
        let mut piles = vec![
            (Pile::Stock, g.stock.clone()),
            (Pile::Waste, g.waste.clone()),
        ];
        piles.extend(
            g.foundations
                .iter()
                .enumerate()
                .map(|(i, v)| (Pile::Foundation(i), v.clone())),
        );
        piles.extend(
            g.cells
                .iter()
                .enumerate()
                .filter_map(|(i, c)| c.map(|c| (Pile::Cell(i), vec![c]))),
        );
        piles.extend(
            g.tableau
                .iter()
                .enumerate()
                .map(|(i, v)| (Pile::Tableau(i), v.clone())),
        );
        Self { piles }
    }
    fn stack_y(&self, pile: Pile, index: usize) -> f32 {
        if let Pile::Tableau(_) = pile {
            return self
                .piles
                .iter()
                .find(|(p, _)| *p == pile)
                .map(|(_, cards)| {
                    cards
                        .iter()
                        .take(index)
                        .map(|c| if c.face_up { 19.0 } else { 12.0 })
                        .sum()
                })
                .unwrap_or(0.0);
        }
        0.0
    }
}

impl PatienceMotion {
    pub(crate) fn deal(&mut self, game: &Game) {
        self.clear();
        let snap = TableSnapshot::capture(game);
        // The initial tableau is dealt from the stock. Stagger the visible cards,
        // while keeping the full deal under roughly 0.6 seconds.
        let mut n = 0;
        for (pile, cards) in &snap.piles {
            if let Pile::Tableau(_) = pile {
                for (index, card) in cards.iter().copied().enumerate() {
                    self.flights.push(CardFlight {
                        card,
                        from: Pile::Stock,
                        from_stack_y: 0.0,
                        to: *pile,
                        to_index: index,
                        to_stack_y: snap.stack_y(*pile, index),
                        elapsed: 0.0,
                        delay: (n as f32 * 0.008).min(0.42),
                        duration: 0.18,
                        flip: false,
                    });
                    n += 1;
                }
            }
        }
    }

    pub(crate) fn transition(&mut self, before: &Game, after: &Game) {
        self.clear();
        let a = TableSnapshot::capture(before);
        let b = TableSnapshot::capture(after);
        let mut removed = Vec::<(Pile, usize, Card)>::new();
        let mut added = Vec::<(Pile, usize, Card)>::new();
        for (pile, old) in &a.piles {
            let new = b
                .piles
                .iter()
                .find(|(p, _)| p == pile)
                .map(|(_, v)| v.as_slice())
                .unwrap_or(&[]);
            let prefix = old
                .iter()
                .zip(new)
                .take_while(|(x, y)| same_card(**x, **y))
                .count();
            removed.extend(
                old.iter()
                    .copied()
                    .enumerate()
                    .skip(prefix)
                    .map(|(i, c)| (*pile, i, c)),
            );
        }
        for (pile, new) in &b.piles {
            let old = a
                .piles
                .iter()
                .find(|(p, _)| p == pile)
                .map(|(_, v)| v.as_slice())
                .unwrap_or(&[]);
            let prefix = old
                .iter()
                .zip(new)
                .take_while(|(x, y)| same_card(**x, **y))
                .count();
            added.extend(
                new.iter()
                    .copied()
                    .enumerate()
                    .skip(prefix)
                    .map(|(i, c)| (*pile, i, c)),
            );
        }
        let mut used = vec![false; added.len()];
        for (from, from_index, card) in removed {
            if let Some((j, (to, to_index, dest_card))) = added
                .iter()
                .enumerate()
                .find(|(j, (to, _, c))| !used[*j] && *to != from && same_card(card, *c))
            {
                used[j] = true;
                self.flights.push(CardFlight {
                    card: *dest_card,
                    from,
                    from_stack_y: a.stack_y(from, from_index),
                    to: *to,
                    to_index: *to_index,
                    to_stack_y: b.stack_y(*to, *to_index),
                    elapsed: 0.0,
                    delay: 0.0,
                    duration: 0.22,
                    flip: card.face_up != dest_card.face_up,
                });
            }
        }
        // Exposure changes, including Undo, turn the card over in its own slot.
        for (pile, old) in &a.piles {
            if let Some((_, new)) = b.piles.iter().find(|(p, _)| p == pile) {
                for (i, (x, y)) in old.iter().zip(new).enumerate() {
                    if same_card(*x, *y) && x.face_up != y.face_up {
                        self.flights.push(CardFlight {
                            card: *y,
                            from: *pile,
                            from_stack_y: a.stack_y(*pile, i),
                            to: *pile,
                            to_index: i,
                            to_stack_y: b.stack_y(*pile, i),
                            elapsed: 0.0,
                            delay: 0.0,
                            duration: 0.20,
                            flip: true,
                        });
                    }
                }
            }
        }
        if after.won() {
            // Foundation indices encode ascending rank. Reveal a Finish in
            // legal order, with the last card arriving within ~1.5 seconds.
            let first_rank = self
                .flights
                .iter()
                .filter(|f| matches!(f.to, Pile::Foundation(_)))
                .map(|f| f.to_index)
                .min()
                .unwrap_or(0);
            for flight in self
                .flights
                .iter_mut()
                .filter(|f| matches!(f.to, Pile::Foundation(_)))
            {
                flight.delay = (flight.to_index - first_rank) as f32 * 0.10;
            }
            self.flights.sort_by_key(|f| match f.to {
                Pile::Foundation(_) => (0, f.to_index),
                _ => (1, 0),
            });
        } else if self.flights.len() > 1 {
            let packet = self
                .flights
                .iter()
                .all(|f| f.from == self.flights[0].from && f.to == self.flights[0].to);
            if !packet || self.flights[0].from == Pile::Stock {
                let step = if self.flights.len() > 20 {
                    0.008
                } else {
                    0.018
                };
                for (i, flight) in self.flights.iter_mut().enumerate() {
                    flight.delay = i as f32 * step;
                }
            }
        }
    }

    pub(crate) fn tick(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        for f in &mut self.flights {
            f.elapsed += dt;
        }
        self.flights.retain(|f| f.elapsed < f.delay + f.duration);
    }
    pub(crate) fn clear(&mut self) {
        self.flights.clear();
    }
    pub(crate) fn busy(&self) -> bool {
        !self.flights.is_empty()
    }
    pub(crate) fn flights(&self) -> &[CardFlight] {
        &self.flights
    }
    pub(crate) fn hides(&self, pile: Pile, index: usize) -> bool {
        self.flights
            .iter()
            .any(|f| f.to == pile && f.to_index == index && f.elapsed < f.delay + f.duration)
    }
}

fn same_card(a: Card, b: Card) -> bool {
    a.rank == b.rank && a.suit == b.suit
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamevex_core::patience::Variant;
    #[test]
    fn tick_settles_by_elapsed_time_and_ignores_bad_deltas() {
        let game = Game::new(Variant::Klondike1, 3);
        let mut motion = PatienceMotion::default();
        motion.deal(&game);
        assert!(motion.busy());
        motion.tick(f32::NAN);
        assert!(motion.busy());
        motion.tick(10.0);
        assert!(!motion.busy());
    }
    #[test]
    fn interrupted_transition_replaces_old_flights() {
        let mut a = Game::new(Variant::Klondike1, 3);
        let pristine = a.clone();
        assert!(a.deal());
        let b = a.clone();
        let mut m = PatienceMotion::default();
        m.transition(&pristine, &a);
        assert!(m.busy());
        assert!(a.undo());
        m.transition(&b, &a);
        assert!(m.busy());
        m.clear();
        assert!(!m.busy());
    }

    #[test]
    fn klondike_stock_deal_only_animates_stock_to_waste() {
        let before = Game::new(Variant::Klondike3, 31);
        let mut after = before.clone();
        assert!(after.deal());

        let mut motion = PatienceMotion::default();
        motion.transition(&before, &after);

        assert_eq!(motion.flights().len(), 3);
        assert!(
            motion
                .flights()
                .iter()
                .all(|flight| { flight.from == Pile::Stock && flight.to == Pile::Waste })
        );
        assert!(
            motion
                .flights()
                .iter()
                .all(|flight| !matches!(flight.to, Pile::Tableau(_)))
        );
    }
    #[test]
    fn new_deal_cascade_is_bounded() {
        let mut m = PatienceMotion::default();
        m.deal(&Game::new(Variant::Spider4, 1));
        assert!(
            m.flights()
                .iter()
                .map(|f| f.delay + f.duration)
                .fold(0.0, f32::max)
                < 0.61
        );
    }

    #[test]
    fn exposed_card_flip_is_one_flight_with_snapshot_offsets() {
        let mut before = Game::new(Variant::Klondike1, 19);
        let column = before.tableau.iter().position(|c| c.len() >= 2).unwrap();
        let index = before.tableau[column].len() - 2;
        before.tableau[column][index].face_up = false;
        let mut after = before.clone();
        after.tableau[column][index].face_up = true;
        let mut motion = PatienceMotion::default();
        motion.transition(&before, &after);
        assert_eq!(motion.flights().len(), 1);
        assert!(motion.flights()[0].flip);
        assert_eq!(
            motion.flights()[0].from_stack_y,
            before.tableau[column][..index]
                .iter()
                .map(|c| if c.face_up { 19.0 } else { 12.0 })
                .sum::<f32>()
        );
        assert_eq!(
            motion.flights()[0].to_stack_y,
            after.tableau[column][..index]
                .iter()
                .map(|c| if c.face_up { 19.0 } else { 12.0 })
                .sum::<f32>()
        );
    }

    #[test]
    fn moved_card_keeps_its_original_stack_offset() {
        let mut before = Game::new(Variant::FreeCell, 21);
        let source = before.tableau.iter().position(|c| !c.is_empty()).unwrap();
        let source_index = before.tableau[source].len() - 1;
        let destination = (0..4).find(|i| before.cells[*i].is_none()).unwrap();
        assert!(before.move_cards(Pile::Tableau(source), source_index, Pile::Cell(destination)));
        let after = before.clone();
        // Recreate the pre-move state by undoing the real command.
        assert!(before.undo());
        let mut motion = PatienceMotion::default();
        motion.transition(&before, &after);
        let flight = motion
            .flights()
            .iter()
            .find(|f| f.from == Pile::Tableau(source))
            .unwrap();
        assert_eq!(flight.from_stack_y, source_index as f32 * 19.0);
        assert_eq!(flight.to_stack_y, 0.0);
    }

    #[test]
    fn finish_arrives_queen_before_king_and_stays_bounded() {
        use gamevex_core::cards::Suit;
        let mut before = Game::new(Variant::FreeCell, 18);
        before.stock.clear();
        before.waste.clear();
        before.cells = vec![None; 4];
        before.tableau = vec![Vec::new(); 8];
        before.foundations = vec![Vec::new(); 4];
        let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
        for (i, suit) in suits.into_iter().enumerate() {
            let last = if i == 0 { 11 } else { 12 };
            before.foundations[i] = std::iter::once(14)
                .chain(2..=last)
                .map(|rank| Card::new(rank, suit, true))
                .collect();
        }
        before.tableau[0].push(Card::new(12, Suit::Spades, true));
        before.tableau[1].push(Card::new(13, Suit::Spades, true));
        before.tableau[2].push(Card::new(13, Suit::Hearts, true));
        before.tableau[3].push(Card::new(13, Suit::Clubs, true));
        before.tableau[4].push(Card::new(13, Suit::Diamonds, true));
        assert!(before.can_auto_finish());
        let mut after = before.clone();
        assert_eq!(after.auto_finish().len(), 5);
        let mut motion = PatienceMotion::default();
        motion.transition(&before, &after);
        let spades = motion
            .flights()
            .iter()
            .filter(|f| f.to == Pile::Foundation(0))
            .collect::<Vec<_>>();
        assert_eq!(
            spades.iter().map(|f| f.to_index).collect::<Vec<_>>(),
            vec![11, 12]
        );
        assert_eq!(spades[0].delay, 0.0);
        assert!(spades[0].delay < spades[1].delay);
        assert!(
            motion
                .flights()
                .iter()
                .map(|f| f.delay + f.duration)
                .fold(0.0, f32::max)
                <= 1.5
        );
    }
}
