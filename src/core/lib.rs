pub mod baccarat;
pub mod backgammon;
pub mod blackjack;
pub mod cards;
pub mod connect4;
pub mod cribbage;
pub mod hearts;
pub mod holdem;
pub mod mahjong;
pub mod minesweeper;
pub mod patience;
pub mod poker;
pub mod roulette;
pub mod save;
pub mod sudoku;
pub mod tictactoe;

pub use blackjack::{
    Blackjack, Command as BlackjackCommand, HandOutcome as BlackjackHandOutcome,
    Phase as BlackjackPhase, PlayerHand as BlackjackPlayerHand,
};
pub use minesweeper::*;
pub use save::*;

#[cfg(test)]
mod cribbage_parity;

#[cfg(test)]
mod holdem_parity;

#[cfg(test)]
mod roulette_parity;

#[cfg(test)]
mod backgammon_parity;
