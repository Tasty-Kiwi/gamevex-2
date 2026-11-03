//! Connect 4 cabinet interaction state and arcade lifecycle methods.
use crate::{
    Arcade, Pending,
    presentation::{Action, Screen, ViewLayout},
};
use bevy::prelude::*;
use gamevex_core::connect4::{COLS, Game, Mode, Player, ROWS, Status};

/// The default focusable column index (center column: 3).
/// In layout.hits: Logo (0), Settings (1), New (2), Columns 0..6 (3..9).
pub(crate) const BOARD_FOCUS: usize = 6;

/// Describes an in-flight falling disc animation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Connect4Fall {
    pub column: usize,
    pub target_row: usize,
    pub player: Player,
}

/// Cabinet presentation state for Connect 4.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Connect4Ui {
    pub selected: usize,
    pub falling: Option<Connect4Fall>,
    pub fall_age: f32,
    pub fall_duration: f32,
    pub bot_timer: f32,
}

impl Default for Connect4Ui {
    fn default() -> Self {
        Self {
            selected: 3,
            falling: None,
            fall_age: 10.0,
            fall_duration: 0.0,
            bot_timer: 0.0,
        }
    }
}

impl Connect4Ui {
    /// Returns true when a fall animation is actively in progress and motion is enabled.
    #[inline]
    pub fn busy(&self, reduced_motion: bool) -> bool {
        !reduced_motion && self.falling.is_some() && self.fall_age < self.fall_duration
    }

    /// Clears any active disc drop animation and ages it out.
    /// Does not terminate or cancel future bot turns.
    pub fn settle_motion(&mut self) {
        self.falling = None;
        self.fall_age = 10.0;
        self.fall_duration = 0.0;
    }
}

impl Arcade {
    /// Resets the Connect 4 session to a new game with the given mode.
    pub(crate) fn new_connect4(&mut self, mode: Mode) {
        let diff = self.save.connect4.difficulty;
        self.save.connect4 = Game::new(mode);
        self.save.connect4.set_difficulty(diff);
        self.connect4_ui = Connect4Ui::default();
        self.modal = None;
        self.fractional_ms = 0.0;
        self.last_active = false;
        self.focus = BOARD_FOCUS;
        self.persist();
    }

    /// Requests starting a new game in `mode`. Prompts for confirmation
    /// if the current game has progress and is still in play.
    pub(crate) fn connect4_request_new(&mut self, mode: Mode) {
        if self.save.connect4.has_progress() && self.save.connect4.status == Status::Playing {
            self.open_modal(Pending::NewConnect4(mode));
        } else {
            self.new_connect4(mode);
        }
    }

    /// Drops a disc into `column` if legal and allowed by current cabinet state.
    pub(crate) fn connect4_drop(&mut self, column: usize) {
        if self.screen != Screen::Connect4 || self.modal.is_some() {
            return;
        }
        if self.connect4_ui.busy(self.save.settings.reduced_motion) {
            return;
        }
        if self.save.connect4.status != Status::Playing {
            return;
        }
        if self.save.connect4.mode == Mode::Computer && self.save.connect4.turn == Player::Yellow {
            return;
        }
        if column >= COLS || !self.save.connect4.legal(column) {
            self.sound = Some(160.0);
            return;
        }
        self.connect4_do_drop(column);
    }

    /// Internal drop routine that modifies domain game state, initiates animation,
    /// sets sound feedback, and persists the save file.
    fn connect4_do_drop(&mut self, column: usize) {
        let target_row = (0..ROWS)
            .rev()
            .find(|&r| self.save.connect4.board[r * COLS + column] == 0)
            .unwrap_or(0);
        let player = self.save.connect4.turn;

        if !self.save.connect4.play(column) {
            return;
        }

        self.connect4_ui.selected = column;
        self.connect4_ui.falling = Some(Connect4Fall {
            column,
            target_row,
            player,
        });
        self.connect4_ui.fall_age = 0.0;
        self.connect4_ui.fall_duration = if self.save.settings.reduced_motion {
            0.0
        } else {
            0.18 + target_row as f32 * 0.04
        };
        self.connect4_ui.bot_timer = 0.0;

        self.sound = match self.save.connect4.status {
            Status::Won(winner) => {
                if self.save.connect4.mode == Mode::Computer && winner == Player::Yellow {
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

    /// Ticks animations and computer bot response timers.
    pub(crate) fn connect4_tick(&mut self, delta: f32) {
        if self.screen != Screen::Connect4
            || self.modal.is_some()
            || !delta.is_finite()
            || delta <= 0.0
        {
            return;
        }

        let was_busy = self.connect4_ui.busy(self.save.settings.reduced_motion);
        let remaining_fall = if was_busy {
            (self.connect4_ui.fall_duration - self.connect4_ui.fall_age).max(0.0)
        } else {
            0.0
        };
        self.connect4_ui.fall_age += delta;

        if was_busy && !self.connect4_ui.busy(self.save.settings.reduced_motion) {
            self.connect4_ui.falling = None;
        }

        if self.connect4_ui.busy(self.save.settings.reduced_motion) {
            return;
        }

        if self.save.connect4.mode == Mode::Computer
            && self.save.connect4.turn == Player::Yellow
            && self.save.connect4.status == Status::Playing
        {
            // The CPU delay starts after the human disc lands, including when
            // a single slow frame spans the end of its fall animation.
            self.connect4_ui.bot_timer += (delta - remaining_fall).max(0.0);
            const BOT_DELAY: f32 = 0.45;
            if self.connect4_ui.bot_timer >= BOT_DELAY
                && let Some(col) = self.save.connect4.bot_column()
            {
                self.connect4_do_drop(col);
            }
        } else {
            self.connect4_ui.bot_timer = 0.0;
        }
    }

    /// Handles keyboard input for Connect 4 when a board column is focused.
    ///
    /// Left/Right selects adjacent legal columns.
    /// Enter/Space drops into the selected column.
    /// Digit1..7 / Numpad1..7 drops directly into that column.
    /// Does not capture Tab.
    pub(crate) fn connect4_key(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        layout: &ViewLayout,
    ) -> bool {
        if self.screen != Screen::Connect4 || self.modal.is_some() {
            return false;
        }

        let focused_col = layout
            .hits
            .get(self.focus)
            .and_then(|hit| match hit.action {
                Action::Connect4Drop(c) => Some(c),
                _ => None,
            });

        let Some(current_col) = focused_col else {
            return false;
        };

        self.connect4_ui.selected = current_col;

        let legals: Vec<usize> = (0..COLS).filter(|&c| self.save.connect4.legal(c)).collect();

        let mut handled = false;
        if keys.just_pressed(KeyCode::ArrowLeft) {
            if let Some(&prev) = legals.iter().rev().find(|&&c| c < current_col) {
                self.connect4_ui.selected = prev;
                if let Some(idx) = layout
                    .hits
                    .iter()
                    .position(|h| matches!(h.action, Action::Connect4Drop(c) if c == prev))
                {
                    self.focus = idx;
                    self.keyboard = true;
                }
            }
            handled = true;
        }

        if keys.just_pressed(KeyCode::ArrowRight) {
            if let Some(&next) = legals.iter().find(|&&c| c > current_col) {
                self.connect4_ui.selected = next;
                if let Some(idx) = layout
                    .hits
                    .iter()
                    .position(|h| matches!(h.action, Action::Connect4Drop(c) if c == next))
                {
                    self.focus = idx;
                    self.keyboard = true;
                }
            }
            handled = true;
        }

        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
            if self.save.connect4.legal(self.connect4_ui.selected) {
                self.connect4_drop(self.connect4_ui.selected);
            }
            return true;
        }

        for (key, col) in [
            (KeyCode::Digit1, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
            (KeyCode::Digit4, 3),
            (KeyCode::Digit5, 4),
            (KeyCode::Digit6, 5),
            (KeyCode::Digit7, 6),
            (KeyCode::Numpad1, 0),
            (KeyCode::Numpad2, 1),
            (KeyCode::Numpad3, 2),
            (KeyCode::Numpad4, 3),
            (KeyCode::Numpad5, 4),
            (KeyCode::Numpad6, 5),
            (KeyCode::Numpad7, 6),
        ] {
            if keys.just_pressed(key) {
                self.connect4_ui.selected = col;
                if let Some(idx) = layout
                    .hits
                    .iter()
                    .position(|h| matches!(h.action, Action::Connect4Drop(c) if c == col))
                {
                    self.focus = idx;
                    self.keyboard = true;
                }
                self.connect4_drop(col);
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
    fn connect4_ui_defaults() {
        let ui = Connect4Ui::default();
        assert_eq!(ui.selected, 3);
        assert!(ui.falling.is_none());
        assert!(!ui.busy(false));
        assert!(!ui.busy(true));
        assert_eq!(ui.bot_timer, 0.0);
    }

    #[test]
    fn connect4_ui_busy_and_settle() {
        let mut ui = Connect4Ui {
            selected: 2,
            falling: Some(Connect4Fall {
                column: 2,
                target_row: 5,
                player: Player::Red,
            }),
            fall_age: 0.1,
            fall_duration: 0.35,
            bot_timer: 0.2,
        };

        assert!(ui.busy(false));
        assert!(!ui.busy(true)); // reduced motion overrides busy

        ui.settle_motion();
        assert!(!ui.busy(false));
        assert!(ui.falling.is_none());
        // Settle motion does not reset or abort future bot turns
        assert_eq!(ui.bot_timer, 0.2);
    }
}
