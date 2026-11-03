//! Tic-tac-toe cabinet interaction state and arcade lifecycle methods.
use crate::{
    Arcade, Pending,
    presentation::{Action, Screen, ViewLayout},
};
use bevy::prelude::*;
use gamevex_core::tictactoe::{Game, Player, Status};

/// The default focusable cell index (center cell: 4).
/// In layout.hits: Logo (0), Settings (1), New (2), Cells 0..8 (3..11) -> Center focus 7.
pub(crate) const BOARD_FOCUS: usize = 7;

/// Cabinet presentation state for Tic-tac-toe.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TicTacToeUi {
    pub selected: usize,
    pub bot_timer: f32,
}

impl Default for TicTacToeUi {
    fn default() -> Self {
        Self {
            selected: 4,
            bot_timer: 0.0,
        }
    }
}

impl Arcade {
    /// Resets the Tic-tac-toe session to a new game.
    pub(crate) fn new_tictactoe(&mut self) {
        let diff = self.save.tictactoe.difficulty;
        self.save.tictactoe = Game::new();
        self.save.tictactoe.set_difficulty(diff);
        self.tictactoe_ui = TicTacToeUi::default();
        self.modal = None;
        self.fractional_ms = 0.0;
        self.last_active = false;
        self.focus = BOARD_FOCUS;
        self.persist();
    }

    /// Requests starting a new game. Prompts for confirmation
    /// if the current game has progress and is still in play.
    pub(crate) fn tictactoe_request_new(&mut self) {
        if self.save.tictactoe.has_progress() && self.save.tictactoe.status == Status::Playing {
            self.open_modal(Pending::NewTicTacToe);
        } else {
            self.new_tictactoe();
        }
    }

    /// Plays into `cell` if legal and allowed by current cabinet state.
    /// Strictly human-only.
    pub(crate) fn tictactoe_play(&mut self, cell: usize) {
        if self.screen != Screen::TicTacToe || self.modal.is_some() {
            return;
        }
        if self.save.tictactoe.status != Status::Playing {
            return;
        }
        if self.save.tictactoe.turn != Player::Human {
            return;
        }
        if cell >= 9 || !self.save.tictactoe.legal(cell) {
            self.sound = Some(160.0);
            return;
        }
        self.tictactoe_do_play(cell);
    }

    /// Internal move routine that modifies domain game state,
    /// sets sound feedback, and persists the save file.
    fn tictactoe_do_play(&mut self, cell: usize) {
        if !self.save.tictactoe.play(cell) {
            return;
        }

        self.tictactoe_ui.selected = cell;
        self.tictactoe_ui.bot_timer = 0.0;

        self.sound = match self.save.tictactoe.status {
            Status::Won(winner) => {
                if winner == Player::Cpu {
                    Some(220.0)
                } else {
                    Some(1040.0)
                }
            }
            Status::Draw => Some(330.0),
            Status::Playing => Some(520.0),
        };

        self.persist();
    }

    /// Ticks computer bot response timers when in foreground without modal.
    pub(crate) fn tictactoe_tick(&mut self, delta: f32) {
        if self.screen != Screen::TicTacToe
            || self.modal.is_some()
            || !delta.is_finite()
            || delta <= 0.0
        {
            return;
        }

        if self.save.tictactoe.turn == Player::Cpu && self.save.tictactoe.status == Status::Playing
        {
            self.tictactoe_ui.bot_timer += delta;
            const BOT_DELAY: f32 = 0.45;
            if self.tictactoe_ui.bot_timer >= BOT_DELAY
                && let Some(cell) = self.save.tictactoe.bot_cell()
            {
                self.tictactoe_do_play(cell);
            }
        } else {
            self.tictactoe_ui.bot_timer = 0.0;
        }
    }

    /// Handles keyboard input for Tic-tac-toe when a board cell is focused.
    ///
    /// Arrow keys navigate adjacent cells in 3x3 grid.
    /// Enter/Space places into the focused cell.
    /// Digit1..9 / Numpad1..9 places directly into that cell in row-major order.
    /// Does not swallow activation when arrow and Enter arrive in the same frame.
    /// Does not capture Tab. Respects modal dialogs.
    pub(crate) fn tictactoe_key(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        layout: &ViewLayout,
    ) -> bool {
        if self.screen != Screen::TicTacToe || self.modal.is_some() {
            return false;
        }

        let focused_cell = layout
            .hits
            .get(self.focus)
            .and_then(|hit| match hit.action {
                Action::TicTacToeCell(c) => Some(c),
                _ => None,
            });

        let Some(current_cell) = focused_cell else {
            return false;
        };

        self.tictactoe_ui.selected = current_cell;

        let mut row = current_cell / 3;
        let mut col = current_cell % 3;
        let mut moved = false;
        let mut handled = false;

        if keys.just_pressed(KeyCode::ArrowLeft) {
            handled = true;
            if col > 0 {
                col -= 1;
                moved = true;
            }
        }
        if keys.just_pressed(KeyCode::ArrowRight) {
            handled = true;
            if col < 2 {
                col += 1;
                moved = true;
            }
        }
        if keys.just_pressed(KeyCode::ArrowUp) {
            handled = true;
            if row > 0 {
                row -= 1;
                moved = true;
            }
        }
        if keys.just_pressed(KeyCode::ArrowDown) {
            handled = true;
            if row < 2 {
                row += 1;
                moved = true;
            }
        }

        if moved {
            let next_cell = row * 3 + col;
            self.tictactoe_ui.selected = next_cell;
            if let Some(idx) = layout
                .hits
                .iter()
                .position(|h| matches!(h.action, Action::TicTacToeCell(c) if c == next_cell))
            {
                self.focus = idx;
                self.keyboard = true;
            }
        }

        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
            self.tictactoe_play(self.tictactoe_ui.selected);
            return true;
        }

        for (key, cell) in [
            (KeyCode::Digit1, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
            (KeyCode::Digit4, 3),
            (KeyCode::Digit5, 4),
            (KeyCode::Digit6, 5),
            (KeyCode::Digit7, 6),
            (KeyCode::Digit8, 7),
            (KeyCode::Digit9, 8),
            (KeyCode::Numpad1, 0),
            (KeyCode::Numpad2, 1),
            (KeyCode::Numpad3, 2),
            (KeyCode::Numpad4, 3),
            (KeyCode::Numpad5, 4),
            (KeyCode::Numpad6, 5),
            (KeyCode::Numpad7, 6),
            (KeyCode::Numpad8, 7),
            (KeyCode::Numpad9, 8),
        ] {
            if keys.just_pressed(key) {
                self.tictactoe_ui.selected = cell;
                if let Some(idx) = layout
                    .hits
                    .iter()
                    .position(|h| matches!(h.action, Action::TicTacToeCell(c) if c == cell))
                {
                    self.focus = idx;
                    self.keyboard = true;
                }
                self.tictactoe_play(cell);
                return true;
            }
        }

        handled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tictactoe_ui_defaults() {
        let ui = TicTacToeUi::default();
        assert_eq!(ui.selected, 4);
        assert_eq!(ui.bot_timer, 0.0);
    }
}
