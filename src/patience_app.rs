//! Shared interaction state for the solitaire family.
use crate::table_feedback::{Anchor, EffectKind, TableFeedback};
use crate::*;
use gamevex_core::patience::{Pile, Variant};
#[derive(Default)]
pub(crate) struct PatienceUi {
    pub selected: Option<(Pile, usize)>,
    pub drag: Option<PatienceDrag>,
    pub pan: Vec2,
    pub message: String,
    pub feedback: TableFeedback,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct PatienceDrag {
    pub source: (Pile, usize),
    pub start: Vec2,
    pub offset: Vec2,
    pub active: bool,
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
        self.patience_ui.feedback.emit(EffectKind::Deal, &[]);
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
            if self.patience_move(from, at, pile) {
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
            self.patience_ui
                .feedback
                .emit(EffectKind::Pick, &[Anchor::Pile(pile)]);
        }
    }
    pub(crate) fn patience_press(&mut self, layout: &ViewLayout) -> bool {
        let Some(cursor) = self.cursor else {
            return false;
        };
        let Some(hit) = layout.hits.iter().rev().find(|h| h.rect.contains(cursor)) else {
            return false;
        };
        let Action::PatiencePick(pile, index) = hit.action else {
            return false;
        };
        if !self.patience_game().can_pick(pile, index) {
            return false;
        }
        self.patience_ui.drag = Some(PatienceDrag {
            source: (pile, index),
            start: cursor,
            offset: cursor
                - layout
                    .patience_origins
                    .iter()
                    .find(|(p, i, _)| *p == pile && *i == index)
                    .map(|(_, _, origin)| *origin)
                    .unwrap_or(hit.rect.min),
            active: false,
        });
        true
    }
    pub(crate) fn patience_drag_update(&mut self) {
        if let (Some(drag), Some(cursor)) = (self.patience_ui.drag.as_mut(), self.cursor)
            && cursor.distance(drag.start) > 8.0
            && !drag.active
        {
            self.patience_ui
                .feedback
                .emit(EffectKind::Pick, &[Anchor::Pile(drag.source.0)]);
            drag.active = true;
        }
    }
    pub(crate) fn patience_release(&mut self, layout: &ViewLayout) {
        let Some(drag) = self.patience_ui.drag.take() else {
            return;
        };
        if !drag.active {
            self.patience_pick(drag.source.0, drag.source.1);
            return;
        }
        let Some(cursor) = self.cursor else { return };
        let Some(hit) = layout.hits.iter().rev().find(|h| h.rect.contains(cursor)) else {
            self.patience_ui
                .feedback
                .emit(EffectKind::Invalid, &[Anchor::Pile(drag.source.0)]);
            return;
        };
        let destination = match hit.action {
            Action::PatiencePick(p, _) | Action::PatienceTarget(p) => p,
            _ => return,
        };
        if !self.patience_move(drag.source.0, drag.source.1, destination) {
            self.patience_ui.message = "That move is not allowed.".into();
            self.patience_ui
                .feedback
                .emit(EffectKind::Invalid, &[Anchor::Pile(destination)]);
        }
    }
    pub(crate) fn patience_target(&mut self, pile: Pile) {
        if let Some((from, index)) = self.patience_ui.selected
            && !self.patience_move(from, index, pile)
        {
            self.patience_ui.message = "That move is not allowed.".into();
            self.patience_ui
                .feedback
                .emit(EffectKind::Invalid, &[Anchor::Pile(pile)]);
        }
    }
    fn patience_move(&mut self, from: Pile, index: usize, to: Pile) -> bool {
        let runs = self.patience_game().completed.len();
        if !self.patience_game_mut().move_cards(from, index, to) {
            return false;
        }
        self.patience_changed("Moved cards.");
        if !self.patience_game().won() {
            self.patience_ui
                .feedback
                .emit(EffectKind::Move, &[Anchor::Pile(from), Anchor::Pile(to)]);
            if self.patience_game().completed.len() > runs
                || matches!(to, Pile::Foundation(i) if self.patience_game().foundations.get(i).is_some_and(|p| p.len()==13))
            {
                self.patience_ui
                    .feedback
                    .emit(EffectKind::Run, &[Anchor::Pile(to)]);
            }
        }
        true
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
        self.patience_ui.drag = None;
        self.patience_ui.message = if self.patience_game().won() {
            "Table cleared!"
        } else {
            message
        }
        .into();
        if self.patience_game().won() {
            self.patience_ui.feedback.clear();
            self.patience_ui.feedback.emit(EffectKind::Win, &[]);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::Hit;

    fn setup() -> (Arcade, ViewLayout, (Pile, usize), Pile) {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Solitaire;
        s.save.solitaire = patience::Game::new(Variant::Klondike1, 7);
        let game = s.patience_game();
        let sources = (0..game.variant.columns())
            .flat_map(|column| {
                game.cards(Pile::Tableau(column))
                    .into_iter()
                    .enumerate()
                    .map(move |(index, _)| (Pile::Tableau(column), index))
            })
            .chain(
                (!game.waste.is_empty())
                    .then_some((Pile::Waste, game.waste.len().saturating_sub(1))),
            );
        let destinations = (0..game.variant.columns())
            .map(Pile::Tableau)
            .chain((0..4).map(Pile::Foundation))
            .chain((0..4).map(Pile::Cell))
            .collect::<Vec<_>>();
        let (source, destination) = sources
            .filter(|(pile, index)| game.can_pick(*pile, *index))
            .find_map(|(pile, index)| {
                destinations
                    .iter()
                    .copied()
                    .find(|to| game.legal_move(pile, index, *to))
                    .map(|to| ((pile, index), to))
            })
            .expect("seeded game has a movable card and legal destination");
        let layout = ViewLayout {
            hits: vec![
                Hit {
                    rect: Rect::from_corners(Vec2::ZERO, Vec2::splat(40.0)),
                    action: Action::PatiencePick(source.0, source.1),
                },
                Hit {
                    rect: Rect::from_corners(Vec2::new(100.0, 0.0), Vec2::new(140.0, 40.0)),
                    action: Action::PatienceTarget(destination),
                },
            ],
            ..default()
        };
        (s, layout, source, destination)
    }

    #[test]
    fn press_is_pending_and_does_not_change_selection_or_game() {
        let (mut s, layout, source, _) = setup();
        let before = s.patience_game().clone();
        s.cursor = Some(Vec2::new(12.0, 16.0));
        assert!(s.patience_press(&layout));
        assert_eq!(s.patience_ui.selected, None);
        assert_eq!(s.patience_game(), &before);
        let drag = s.patience_ui.drag.unwrap();
        assert_eq!(drag.source, source);
        assert!(!drag.active);
        assert_eq!(drag.offset, Vec2::new(12.0, 16.0));
    }

    #[test]
    fn grabbing_a_clipped_card_keeps_its_original_anchor() {
        let (mut s, mut layout, source, _) = setup();
        layout
            .patience_origins
            .push((source.0, source.1, Vec2::new(-10.0, -20.0)));
        s.cursor = Some(Vec2::new(12.0, 16.0));
        assert!(s.patience_press(&layout));
        assert_eq!(s.patience_ui.drag.unwrap().offset, Vec2::new(22.0, 36.0));
    }

    #[test]
    fn active_drag_moves_source_stack_to_release_target() {
        let (mut s, layout, source, destination) = setup();
        let mut expected = s.patience_game().clone();
        assert!(expected.move_cards(source.0, source.1, destination));
        s.patience_ui.selected = Some(source);
        s.cursor = Some(Vec2::new(12.0, 16.0));
        assert!(s.patience_press(&layout));
        assert_eq!(s.patience_ui.selected, Some(source));
        s.cursor = Some(Vec2::new(120.0, 20.0));
        s.patience_drag_update();
        assert!(s.patience_ui.drag.unwrap().active);
        s.patience_release(&layout);
        assert_eq!(s.patience_game(), &expected);
        assert!(s.patience_ui.drag.is_none());
    }

    #[test]
    fn invalid_drop_and_cancel_leave_game_unchanged() {
        let (mut s, layout, _, _) = setup();
        let before = s.patience_game().clone();
        s.cursor = Some(Vec2::new(12.0, 16.0));
        assert!(s.patience_press(&layout));
        s.cursor = Some(Vec2::new(300.0, 300.0));
        s.patience_drag_update();
        s.patience_release(&layout);
        assert_eq!(s.patience_game(), &before);
        s.cursor = Some(Vec2::new(12.0, 16.0));
        assert!(s.patience_press(&layout));
        s.patience_ui.drag = None;
        assert_eq!(s.patience_game(), &before);
    }
}
