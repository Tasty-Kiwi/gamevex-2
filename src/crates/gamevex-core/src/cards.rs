use serde::{Deserialize, Serialize};

/// A standard playing-card suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
}

impl Suit {
    /// Unicode suit glyph, useful for labels alongside rendered suit geometry.
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Spades => "♠",
            Self::Hearts => "♥",
            Self::Clubs => "♣",
            Self::Diamonds => "♦",
        }
    }

    pub const fn is_red(self) -> bool {
        matches!(self, Self::Hearts | Self::Diamonds)
    }
}

/// A playing card. Ranks use 2–14, where 11–14 are Jack, Queen, King, Ace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
    pub face_up: bool,
}

impl Card {
    pub const fn new(rank: u8, suit: Suit, face_up: bool) -> Self {
        Self {
            rank,
            suit,
            face_up,
        }
    }

    pub const fn rank_label(self) -> &'static str {
        match self.rank {
            2..=10 => RANK_LABELS[(self.rank - 2) as usize],
            11 => "J",
            12 => "Q",
            13 => "K",
            14 => "A",
            _ => "?",
        }
    }
}

const RANK_LABELS: [&str; 9] = ["2", "3", "4", "5", "6", "7", "8", "9", "10"];

/// Small engine-independent state model for the interactive Card Lab.
///
/// Cards remain in hand order. A card's index is its stable identity for the
/// lifetime of the current hand; the three slots store those indices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardLab {
    pub cards: Vec<Card>,
    pub selected: Option<usize>,
    pub slots: [Option<usize>; 3],
    pub drag: Option<usize>,
}

impl Default for CardLab {
    fn default() -> Self {
        let mut lab = Self {
            cards: Vec::new(),
            selected: None,
            slots: [None; 3],
            drag: None,
        };
        lab.reset();
        lab
    }
}

impl CardLab {
    /// Restore the same five-card hand and clear all interaction state.
    pub fn reset(&mut self) {
        self.cards = vec![
            Card::new(14, Suit::Spades, true),
            Card::new(13, Suit::Hearts, true),
            Card::new(12, Suit::Clubs, true),
            Card::new(11, Suit::Diamonds, true),
            Card::new(10, Suit::Spades, true),
        ];
        self.selected = None;
        self.slots = [None; 3];
        self.drag = None;
    }

    /// Select a card by its hand index. Returns false for an invalid index.
    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.cards.len() {
            return false;
        }
        self.selected = Some(index);
        true
    }

    /// Move focus to the next card, wrapping around the hand.
    pub fn select_next(&mut self) -> Option<usize> {
        if self.cards.is_empty() {
            self.selected = None;
            return None;
        }
        let next = self.selected.map_or(0, |i| (i + 1) % self.cards.len());
        self.selected = Some(next);
        Some(next)
    }

    /// Move focus to the previous card, wrapping around the hand.
    pub fn select_previous(&mut self) -> Option<usize> {
        if self.cards.is_empty() {
            self.selected = None;
            return None;
        }
        let previous = self.selected.map_or(self.cards.len() - 1, |i| {
            (i + self.cards.len() - 1) % self.cards.len()
        });
        self.selected = Some(previous);
        Some(previous)
    }

    /// Flip the selected card. Returns false when there is no valid selection.
    pub fn flip_selected(&mut self) -> bool {
        let Some(card) = self.selected.and_then(|i| self.cards.get_mut(i)) else {
            return false;
        };
        card.face_up = !card.face_up;
        true
    }

    /// Begin dragging a card. Its existing slot is retained until a valid drop.
    pub fn pick(&mut self, index: usize) -> bool {
        if index >= self.cards.len() || self.drag.is_some() {
            return false;
        }
        self.selected = Some(index);
        self.drag = Some(index);
        true
    }

    /// Drop the dragged card into a slot, or outside the slots with `None`.
    /// A valid drop into an empty slot (or its current slot) commits the move.
    /// Invalid drops leave the prior layout intact and end the drag.
    pub fn drop_in(&mut self, slot: Option<usize>) -> bool {
        let Some(card) = self.drag.take() else {
            return false;
        };
        let Some(destination) = slot.filter(|&i| i < self.slots.len()) else {
            return false;
        };
        if self.slots[destination].is_some_and(|occupant| occupant != card) {
            return false;
        }

        for occupied in &mut self.slots {
            if *occupied == Some(card) {
                *occupied = None;
            }
        }
        self.slots[destination] = Some(card);
        true
    }

    /// End a drag without changing the committed slot layout.
    pub fn cancel_drag(&mut self) {
        self.drag = None;
    }

    /// Clear transient interaction state when the application loses focus.
    pub fn focus_lost(&mut self) {
        self.cancel_drag();
        self.selected = None;
    }

    pub fn slot_of(&self, card: usize) -> Option<usize> {
        self.slots
            .iter()
            .position(|&occupant| occupant == Some(card))
    }

    /// Label for the selected card, or an empty string when none is selected.
    pub fn rank_label(&self) -> &'static str {
        self.selected
            .and_then(|i| self.cards.get(i))
            .map_or("", |card| card.rank_label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_restores_repeatable_five_card_hand_and_clears_state() {
        let mut lab = CardLab::default();
        let initial = lab.cards.clone();
        assert_eq!(initial.len(), 5);
        assert_eq!(initial[0], Card::new(14, Suit::Spades, true));
        assert_eq!(initial[1], Card::new(13, Suit::Hearts, true));
        assert_eq!(initial[2], Card::new(12, Suit::Clubs, true));
        assert_eq!(initial[3], Card::new(11, Suit::Diamonds, true));
        assert_eq!(initial[4], Card::new(10, Suit::Spades, true));

        lab.select(2);
        lab.pick(2);
        lab.drop_in(Some(1));
        lab.flip_selected();
        lab.reset();

        assert_eq!(lab.cards, initial);
        assert_eq!(lab.selected, None);
        assert_eq!(lab.slots, [None; 3]);
        assert_eq!(lab.drag, None);
    }

    #[test]
    fn valid_moves_vacate_the_cards_previous_slot() {
        let mut lab = CardLab::default();
        assert!(lab.pick(0));
        assert!(lab.drop_in(Some(0)));
        assert_eq!(lab.slot_of(0), Some(0));

        assert!(lab.pick(0));
        assert!(lab.drop_in(Some(2)));
        assert_eq!(lab.slots, [None, None, Some(0)]);
        assert_eq!(lab.slot_of(0), Some(2));
    }

    #[test]
    fn invalid_and_occupied_drops_restore_the_previous_layout() {
        let mut lab = CardLab::default();
        assert!(lab.pick(0));
        assert!(lab.drop_in(Some(0)));
        assert!(lab.pick(1));
        assert!(lab.drop_in(Some(1)));
        let before = lab.slots;

        assert!(lab.pick(0));
        assert!(!lab.drop_in(Some(1)));
        assert_eq!(lab.slots, before);
        assert_eq!(lab.drag, None);

        assert!(lab.pick(0));
        assert!(!lab.drop_in(Some(3)));
        assert_eq!(lab.slots, before);
    }

    #[test]
    fn cancellation_and_dropping_outside_preserve_layout() {
        let mut lab = CardLab::default();
        assert!(lab.pick(0));
        assert!(lab.drop_in(Some(0)));
        let before = lab.slots;

        assert!(lab.pick(0));
        lab.cancel_drag();
        assert_eq!(lab.slots, before);
        assert_eq!(lab.drag, None);
        assert!(!lab.drop_in(Some(1)));

        assert!(lab.pick(0));
        assert!(!lab.drop_in(None));
        assert_eq!(lab.slots, before);
    }

    #[test]
    fn flipping_selection_focus_loss_and_keyboard_wrap_work() {
        let mut lab = CardLab::default();
        assert!(!lab.flip_selected());
        assert_eq!(lab.select_previous(), Some(4));
        assert_eq!(lab.rank_label(), "10");
        assert_eq!(lab.select_next(), Some(0));
        assert_eq!(lab.rank_label(), "A");
        assert!(lab.flip_selected());
        assert!(!lab.cards[0].face_up);

        assert!(lab.pick(0));
        lab.focus_lost();
        assert_eq!(lab.selected, None);
        assert_eq!(lab.drag, None);
        assert_eq!(lab.slots, [None; 3]);
    }

    #[test]
    fn suits_expose_symbols_and_color() {
        assert_eq!(Suit::Spades.symbol(), "♠");
        assert!(!Suit::Clubs.is_red());
        assert!(Suit::Hearts.is_red());
        assert!(Suit::Diamonds.is_red());
    }
}
