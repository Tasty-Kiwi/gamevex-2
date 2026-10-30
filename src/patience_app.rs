//! Shared interaction state for the solitaire family.
use crate::*;
use gamevex_core::patience::{Pile, Variant};
#[derive(Default)]
pub(crate) struct PatienceUi {
    pub selected: Option<(Pile, usize)>,
    pub drag_start: Option<Vec2>,
    pub pan: Vec2,
    pub message: String,
}
impl Arcade {
    pub(crate) fn patience_game(&self) -> &patience::Game {
        match self.screen {
            Screen::FreeCell => &self.save.freecell,
            Screen::Spider => &self.save.spider,
            _ => &self.save.solitaire,
        }
    }
    pub(crate) fn patience_game_mut(&mut self) -> &mut patience::Game {
        match self.screen {
            Screen::FreeCell => &mut self.save.freecell,
            Screen::Spider => &mut self.save.spider,
            _ => &mut self.save.solitaire,
        }
    }
    pub(crate) fn request_patience(&mut self, variant: Variant, repeat: bool) {
        let seed = if repeat {
            self.patience_game().seed
        } else {
            fresh_seed()
        };
        if self.patience_game().has_progress() && !self.patience_game().won() {
            self.open_modal(Pending::NewPatience(variant, seed));
        } else {
            self.new_patience(variant, seed);
        }
    }
    pub(crate) fn new_patience(&mut self, variant: Variant, seed: u64) {
        *self.patience_game_mut() = patience::Game::new(variant, seed);
        self.patience_ui = Default::default();
        self.modal = None;
        self.last_active = false;
        self.fractional_ms = 0.0;
        self.persist();
    }
    pub(crate) fn patience_pick(&mut self, pile: Pile, index: usize) {
        if let Some((from, at)) = self.patience_ui.selected {
            if from == pile && at == index {
                self.patience_ui.selected = None;
                return;
            }
            if self.patience_game_mut().move_cards(from, at, pile) {
                self.patience_changed("Moved cards.");
                return;
            }
        }
        if self
            .patience_game()
            .cards(pile)
            .get(index)
            .is_some_and(|c| c.face_up)
        {
            self.patience_ui.selected = Some((pile, index));
            self.patience_ui.message = "Choose a destination pile.".into();
        }
    }
    pub(crate) fn patience_target(&mut self, pile: Pile) {
        if let Some((from, index)) = self.patience_ui.selected {
            if self.patience_game_mut().move_cards(from, index, pile) {
                self.patience_changed("Moved cards.");
            } else {
                self.patience_ui.message = "That move is not allowed.".into();
            }
        }
    }
    pub(crate) fn patience_changed(&mut self, message: &str) {
        if self.patience_game().won() && !self.patience_game().assisted {
            let game = self.patience_game();
            let index = Variant::ALL
                .iter()
                .position(|v| *v == game.variant)
                .unwrap();
            let elapsed = game.elapsed_ms.max(1000);
            if self.save.patience_records[index].is_none_or(|old| elapsed < old) {
                self.save.patience_records[index] = Some(elapsed);
            }
        }
        self.patience_ui.selected = None;
        self.patience_ui.drag_start = None;
        self.patience_ui.message = if self.patience_game().won() {
            "Table cleared!"
        } else {
            message
        }
        .into();
        self.sound = Some(if self.patience_game().won() {
            1040.0
        } else {
            650.0
        });
        self.persist();
    }
}
pub(crate) fn is_family(screen: Screen) -> bool {
    matches!(
        screen,
        Screen::Solitaire | Screen::FreeCell | Screen::Spider
    )
}
pub(crate) fn keys(s: &mut Arcade, keys: &ButtonInput<KeyCode>, l: &ViewLayout) {
    if !is_family(s.screen) || s.modal.is_some() {
        return;
    }
    for (key, action) in [
        (KeyCode::KeyD, Action::PatienceDeal),
        (KeyCode::KeyU, Action::PatienceUndo),
        (KeyCode::KeyH, Action::PatienceHint),
    ] {
        if keys.just_pressed(key) {
            s.act(action);
        }
    }
    let x = i32::from(keys.just_pressed(KeyCode::ArrowRight))
        - i32::from(keys.just_pressed(KeyCode::ArrowLeft));
    let y = i32::from(keys.just_pressed(KeyCode::ArrowDown))
        - i32::from(keys.just_pressed(KeyCode::ArrowUp));
    s.patience_ui.pan = (s.patience_ui.pan + Vec2::new(x as f32, y as f32) * 48.0)
        .clamp(Vec2::ZERO, l.patience_max_pan);
}

pub(crate) fn pile_name(p: Pile) -> String {
    match p {
        Pile::Stock => "stock".into(),
        Pile::Waste => "waste".into(),
        Pile::Foundation(i) => format!("foundation {}", i + 1),
        Pile::Cell(i) => format!("free cell {}", i + 1),
        Pile::Tableau(i) => format!("column {}", i + 1),
    }
}
