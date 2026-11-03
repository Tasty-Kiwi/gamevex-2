//! Cabinet interaction state stays outside the serializable Sudoku session.
use crate::{
    Arcade, Pending, fresh_seed,
    presentation::{Action, Screen, ViewLayout},
};
use bevy::prelude::*;
use gamevex_core::sudoku::{Command, Difficulty, History, Sudoku};

#[derive(Default)]
pub(crate) struct SudokuUi {
    pub selected: usize,
    pub pencil: bool,
    pub pan: Vec2,
    pub follow: bool,
    pub message: String,
    pub history: History,
    pub completion_age: Option<f32>,
    pub show_highlights: bool,
    pub setup: Option<(u8, Difficulty)>,
}

// The board is the first focusable item after the logo and Settings.
pub(crate) const BOARD_FOCUS: usize = 2;

impl Arcade {
    pub(crate) fn new_sudoku(&mut self, size: u8, difficulty: Difficulty) {
        self.save.sudoku.game = Sudoku::new(size, difficulty, fresh_seed());
        self.sudoku_ui = SudokuUi {
            selected: self
                .save
                .sudoku
                .game
                .givens
                .iter()
                .position(|v| *v == 0)
                .unwrap_or(0),
            follow: true,
            ..default()
        };
        self.modal = None;
        self.fractional_ms = 0.0;
        self.last_active = false;
        self.focus = BOARD_FOCUS;
        self.persist();
    }

    pub(crate) fn open_sudoku_setup(&mut self) {
        self.sudoku_ui.setup = Some((self.save.sudoku.game.size, self.save.sudoku.game.difficulty));
        self.open_modal(Pending::SudokuSetup);
    }

    pub(crate) fn start_sudoku_setup(&mut self) {
        if !matches!(self.modal, Some(Pending::SudokuSetup)) {
            return;
        }
        let Some((size, difficulty)) = self.sudoku_ui.setup.take() else {
            return;
        };
        self.restore_modal_focus();
        self.request_sudoku(size, difficulty);
    }

    pub(crate) fn request_sudoku(&mut self, size: u8, difficulty: Difficulty) {
        let game = &self.save.sudoku.game;
        let edited = game
            .board
            .iter()
            .zip(&game.givens)
            .any(|(v, given)| *given == 0 && *v != 0)
            || game.notes.iter().any(|notes| *notes != 0);
        if edited && !game.complete {
            self.open_modal(Pending::NewSudoku(size, difficulty));
        } else {
            self.new_sudoku(size, difficulty);
        }
    }

    pub(crate) fn sudoku_command(&mut self, command: Command) {
        // Number-bank hit regions may have been painted before the current cell click.
        let command = match command {
            Command::Enter { value, .. } => Command::Enter {
                index: self.sudoku_ui.selected,
                value,
                pencil: self.sudoku_ui.pencil,
            },
            Command::Erase(_) => Command::Erase(self.sudoku_ui.selected),
            command => command,
        };
        let hint = (command == Command::Hint)
            .then(|| self.save.sudoku.game.find_hint())
            .flatten();
        let was_complete = self.save.sudoku.game.complete;
        let undo_before = (command == Command::Undo).then(|| {
            (
                self.save.sudoku.game.board.clone(),
                self.save.sudoku.game.notes.clone(),
            )
        });
        if self
            .save
            .sudoku
            .command(command, &mut self.sudoku_ui.history)
        {
            // Mouse actions return entry focus to the board so typing can continue.
            self.focus = BOARD_FOCUS;
            if let Some((board, notes)) = undo_before
                && let Some(index) =
                    self.save
                        .sudoku
                        .game
                        .board
                        .iter()
                        .enumerate()
                        .find_map(|(i, value)| {
                            (board[i] != *value || notes[i] != self.save.sudoku.game.notes[i])
                                .then_some(i)
                        })
            {
                self.sudoku_ui.selected = index;
                self.sudoku_ui.follow = true;
            }
            self.sudoku_ui.message = match command {
                Command::Hint => {
                    let hint = hint.unwrap();
                    self.sudoku_ui.selected = hint.index;
                    self.sudoku_ui.follow = true;
                    format!("{} Assisted run; best times disabled.", hint.explanation)
                }
                Command::Undo if self.save.sudoku.game.assisted => {
                    "Move undone. This run remains assisted.".into()
                }
                Command::Undo => "Move undone.".into(),
                _ => String::new(),
            };
            let game = &self.save.sudoku.game;
            self.sound = Some(if game.conflict(self.sudoku_ui.selected) {
                220.0
            } else {
                650.0
            });
            if !was_complete && game.complete {
                self.sudoku_ui.completion_age = Some(0.0);
                self.sudoku_ui.message = if game.assisted {
                    "Puzzle complete. Assisted run; no best time."
                } else {
                    "Puzzle complete. Your time is saved for this mode."
                }
                .into();
                self.sound = Some(1040.0);
            }
            self.persist();
        } else if command == Command::Hint && !self.save.sudoku.game.complete {
            let game = &self.save.sudoku.game;
            self.sudoku_ui.message = if game.size == 16 {
                "Hints are unavailable while 16x16 is experimental."
            } else if (0..game.board.len()).any(|i| game.conflict(i)) {
                "Correct conflicting entries before asking for a hint."
            } else {
                "No safe single-step hint. Check your entries or try another technique."
            }
            .into();
        }
    }
}

/// Only a focused board consumes values and movement; Tab always reaches controls.
pub(crate) fn board_keys(state: &mut Arcade, keys: &ButtonInput<KeyCode>, layout: &ViewLayout) {
    if state.screen != Screen::Sudoku
        || state.modal.is_some()
        || !layout
            .hits
            .get(state.focus)
            .is_some_and(|hit| matches!(hit.action, Action::SudokuCell))
    {
        return;
    }
    let size = usize::from(state.save.sudoku.game.size);
    let mut row = state.sudoku_ui.selected / size;
    let mut col = state.sudoku_ui.selected % size;
    if keys.just_pressed(KeyCode::ArrowLeft) {
        col = col.saturating_sub(1);
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        col = (col + 1).min(size - 1);
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        row = row.saturating_sub(1);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        row = (row + 1).min(size - 1);
    }
    if row * size + col != state.sudoku_ui.selected {
        state.sudoku_ui.selected = row * size + col;
        state.sudoku_ui.follow = true;
    }
    for (key, value) in [
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
        (KeyCode::Numpad1, 1),
        (KeyCode::Numpad2, 2),
        (KeyCode::Numpad3, 3),
        (KeyCode::Numpad4, 4),
        (KeyCode::Numpad5, 5),
        (KeyCode::Numpad6, 6),
        (KeyCode::Numpad7, 7),
        (KeyCode::Numpad8, 8),
        (KeyCode::Numpad9, 9),
        (KeyCode::KeyA, 10),
        (KeyCode::KeyB, 11),
        (KeyCode::KeyC, 12),
        (KeyCode::KeyD, 13),
        (KeyCode::KeyE, 14),
        (KeyCode::KeyF, 15),
        (KeyCode::KeyG, 16),
    ] {
        if keys.just_pressed(key) && value <= state.save.sudoku.game.size {
            state.sudoku_command(Command::Enter {
                index: state.sudoku_ui.selected,
                value,
                pencil: state.sudoku_ui.pencil,
            });
        }
    }
    if keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::Delete) {
        state.sudoku_command(Command::Erase(state.sudoku_ui.selected));
    }
    if keys.just_pressed(KeyCode::KeyP) {
        state.act(Action::SudokuPencil);
    }
    if keys.just_pressed(KeyCode::KeyU) {
        state.sudoku_command(Command::Undo);
    }
    if keys.just_pressed(KeyCode::KeyH) {
        state.sudoku_command(Command::Hint);
    }
}
