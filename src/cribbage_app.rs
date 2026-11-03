//! Foreground-only Cribbage pacing. Save results before presenting their motion.
use crate::table_feedback::{EffectKind, TableFeedback};
use crate::*;
use gamevex_core::cribbage::{Game, Phase};

#[derive(Default)]
pub(crate) struct CribbageUi {
    pub selected: Vec<usize>,
    pub hovered: Option<usize>,
    pub positions: Vec<Vec2>,
    pub age: f32,
    pub feedback: TableFeedback,
}
impl Arcade {
    fn cribbage_ready(&self) -> bool {
        self.screen == Screen::Cribbage
            && (self.save.settings.reduced_motion || self.cribbage_ui.age >= 0.35)
    }
    pub(crate) fn cribbage_card(&mut self, index: usize) {
        if !self.cribbage_ready() {
            return;
        }
        if self.save.cribbage.phase == Phase::Discard {
            if index >= self.save.cribbage.hands[0].len() {
                return;
            }
            if let Some(i) = self.cribbage_ui.selected.iter().position(|&i| i == index) {
                self.cribbage_ui.selected.remove(i);
            } else if self.cribbage_ui.selected.len() < 2 {
                self.cribbage_ui.selected.push(index);
            } else {
                return;
            }
            self.cribbage_ui.feedback.emit(EffectKind::Pick, &[]);
        } else if self.save.cribbage.turn == 0 && self.save.cribbage.play(index) {
            self.cribbage_changed(EffectKind::Move);
        }
    }
    pub(crate) fn cribbage_discard(&mut self) {
        if self.cribbage_ready() && self.save.cribbage.discard(&self.cribbage_ui.selected) {
            self.cribbage_ui.selected.clear();
            self.cribbage_changed(EffectKind::Deal);
        }
    }
    pub(crate) fn cribbage_go(&mut self) {
        if self.cribbage_ready() && self.save.cribbage.turn == 0 && self.save.cribbage.go() {
            self.cribbage_changed(EffectKind::Move);
        }
    }
    pub(crate) fn cribbage_continue(&mut self) {
        if !self.cribbage_ready() {
            return;
        }
        if self.save.cribbage.phase == Phase::MatchComplete {
            self.new_cribbage();
        } else if self.save.cribbage.advance() {
            self.cribbage_changed(EffectKind::Deal);
        }
    }
    pub(crate) fn new_cribbage(&mut self) {
        self.save.cribbage = Game::new(fresh_seed());
        self.cribbage_ui = Default::default();
        self.modal = None;
        self.focus = 0;
        self.cribbage_changed(EffectKind::Deal);
    }
    fn cribbage_changed(&mut self, kind: EffectKind) {
        self.cribbage_ui.age = 0.0;
        self.cribbage_ui.hovered = None;
        self.cribbage_ui.positions.clear();
        self.cribbage_ui.feedback.emit(kind, &[]);
        if self.save.cribbage.phase == Phase::MatchComplete {
            let won = self.save.cribbage.scores[0] >= 121;
            if won {
                self.save.cribbage_wins = self.save.cribbage_wins.saturating_add(1);
            }
            self.cribbage_ui.feedback.emit(
                if won {
                    EffectKind::Win
                } else {
                    EffectKind::Stuck
                },
                &[],
            );
        } else if self.save.cribbage.last_message.contains("+") {
            self.cribbage_ui.feedback.emit(EffectKind::Run, &[]);
        }
        self.persist();
    }
    pub(crate) fn cribbage_tick(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.cribbage_ui.feedback.tick(dt);
        self.cribbage_ui.age += dt;
        if self.cribbage_ui.age >= 0.85
            && self.save.cribbage.phase == Phase::Pegging
            && self.save.cribbage.turn == 1
            && self.save.cribbage.cpu_step()
        {
            self.cribbage_changed(EffectKind::Move);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn confirmed_discard_saves_and_cpu_advances() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Cribbage;
        s.save.settings.reduced_motion = true;
        let untouched = (s.save.hearts.clone(), s.save.solitaire.clone());
        s.cribbage_card(0);
        s.cribbage_card(1);
        s.cribbage_discard();
        assert_eq!(s.save.cribbage.crib.len(), 4);
        assert_eq!(Save::from_json(&s.save.to_json().unwrap()).unwrap(), s.save);
        assert_eq!((s.save.hearts.clone(), s.save.solitaire.clone()), untouched);
        s.save.cribbage.turn = 1;
        let paused = s.save.cribbage.clone();
        s.cribbage_tick(2.0);
        assert_ne!(paused, s.save.cribbage);
    }
    #[test]
    fn controls_and_cards_fit_supported_viewports() {
        for (width, height) in [(360, 560), (390, 740), (1280, 720), (1280, 800)] {
            let mut s = crate::tests::test_arcade();
            s.screen = Screen::Cribbage;
            s.save.settings.reduced_motion = true;
            let mut app = App::new();
            app.insert_resource(s)
                .insert_resource(Time::<()>::default())
                .init_resource::<ViewLayout>()
                .init_resource::<rendering::Frame>()
                .init_resource::<rendering::RenderOptions>()
                .add_systems(Update, presentation::paint);
            app.world_mut().spawn((
                Window {
                    resolution: WindowResolution::new(width, height),
                    ..default()
                },
                PrimaryWindow,
            ));
            app.update();
            let l = app.world().resource::<ViewLayout>();
            let mut cards = 0;
            for hit in &l.hits {
                if matches!(hit.action, Action::CribbageCard(_) | Action::CribbageNew) {
                    assert!(
                        hit.rect.min.x >= 0.0 && hit.rect.max.x <= width as f32,
                        "x {width}x{height} {:?}",
                        hit.rect
                    );
                    assert!(
                        hit.rect.min.y >= 0.0 && hit.rect.max.y <= height as f32 - 33.0,
                        "y {width}x{height} {:?}",
                        hit.rect
                    );
                    if matches!(hit.action, Action::CribbageCard(_)) {
                        cards += 1;
                    }
                }
            }
            assert_eq!(cards, 6);
        }
    }
}
