pub mod baccarat;
pub mod blackjack;
pub mod cards;
pub mod cribbage;
pub mod hearts;
pub mod mahjong;
pub mod minesweeper;
pub mod patience;
pub mod save;
pub mod sudoku;

pub use blackjack::{
    Blackjack, Command as BlackjackCommand, HandOutcome as BlackjackHandOutcome,
    Phase as BlackjackPhase, PlayerHand as BlackjackPlayerHand,
};
pub use minesweeper::*;
pub use save::*;

#[cfg(test)]
mod cribbage_parity;
