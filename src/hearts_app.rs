//! Hearts actions and foreground-only pacing; the saved game is always authoritative.
use crate::table_feedback::{EffectKind, TableFeedback};
use crate::*;
use gamevex_core::hearts::Phase;

#[derive(Default)]
pub(crate) struct HeartsUi {
    pub motion: crate::hearts_motion::HeartsMotion,
    pub selected: Vec<usize>,
    pub hovered: Option<usize>,
    pub hand_cards: Vec<gamevex_core::cards::Card>,
    pub card_positions: Vec<Vec2>,
    pub message: String,
    pub age: f32,
    pub feedback: TableFeedback,
}

impl Arcade {
    pub(crate) fn hearts_card(&mut self, index: usize) {
        if self.screen != Screen::Hearts || self.hearts_ui.motion.busy() {
            return;
        }
        if self.save.hearts.phase == Phase::Passing {
            if index >= self.save.hearts.hands[0].len() {
                return;
            }
            if let Some(at) = self.hearts_ui.selected.iter().position(|i| *i == index) {
                self.hearts_ui.selected.remove(at);
            } else if self.hearts_ui.selected.len() < 3 {
                self.hearts_ui.selected.push(index);
            } else {
                return;
            }
            self.hearts_ui.feedback.emit(EffectKind::Pick, &[]);
        } else {
            let card = self.save.hearts.hands[0].get(index).copied();
            let position = self.hearts_ui.card_positions.get(index).copied();
            if self.save.hearts.play(index) {
                self.hearts_changed(EffectKind::Move);
                if !self.save.settings.reduced_motion
                    && let Some(card) = card
                {
                    self.hearts_ui.motion.play(0, card, position);
                }
            }
        }
    }
    pub(crate) fn hearts_pass(&mut self) {
        if self.screen != Screen::Hearts || self.hearts_ui.motion.busy() {
            return;
        }
        let before = self.save.hearts.clone();
        if self.save.hearts.pass(&self.hearts_ui.selected) {
            let positions = self.hearts_ui.card_positions.clone();
            self.hearts_ui.selected.clear();
            self.hearts_changed(EffectKind::Deal);
            if !self.save.settings.reduced_motion {
                self.hearts_ui
                    .motion
                    .pass(&before, &self.save.hearts, &positions);
            }
        }
    }
    pub(crate) fn hearts_continue(&mut self) {
        if self.screen != Screen::Hearts || self.hearts_ui.motion.busy() {
            return;
        }
        if self.save.hearts.phase == Phase::MatchComplete {
            self.new_hearts();
        } else if self.save.hearts.next_hand() {
            self.hearts_changed(EffectKind::Deal);
            if !self.save.settings.reduced_motion {
                self.hearts_ui.motion.deal(&self.save.hearts);
            }
        }
    }
    pub(crate) fn request_hearts(&mut self) {
        if self.screen != Screen::Hearts {
            return;
        }
        if self.save.hearts.phase == Phase::MatchComplete {
            self.new_hearts();
        } else {
            self.open_modal(Pending::NewHearts);
        }
    }
    pub(crate) fn new_hearts(&mut self) {
        self.save.hearts = hearts::Game::new(fresh_seed());
        self.hearts_ui = Default::default();
        self.modal = None;
        self.focus = 3;
        self.hearts_changed(EffectKind::Deal);
        if !self.save.settings.reduced_motion {
            self.hearts_ui.motion.deal(&self.save.hearts);
        }
    }
    fn hearts_changed(&mut self, effect: EffectKind) {
        self.hearts_ui.hovered = None;
        self.hearts_ui.age = 0.0;
        self.hearts_ui.message.clear();
        self.hearts_ui.feedback.emit(effect, &[]);
        if self.save.hearts.phase == Phase::MatchComplete {
            let score = self.save.hearts.totals[0];
            if self.save.hearts_best.is_none_or(|old| score < old) {
                self.save.hearts_best = Some(score);
            }
            self.hearts_ui.feedback.emit(
                if self.save.hearts.totals.iter().all(|other| score <= *other) {
                    EffectKind::Win
                } else {
                    EffectKind::Stuck
                },
                &[],
            );
        }
        self.persist();
    }
    pub(crate) fn hearts_tick(&mut self, delta: f32) {
        if !delta.is_finite() || delta <= 0.0 {
            return;
        }
        self.hearts_ui.feedback.tick(delta);
        self.hearts_ui.age += delta;
        if self.save.settings.reduced_motion {
            self.hearts_ui.motion.clear();
        } else {
            self.hearts_ui.motion.tick(delta);
        }
        if self.hearts_ui.motion.busy() {
            return;
        }
        let phase = self.save.hearts.phase;
        match phase {
            Phase::Playing if self.save.hearts.turn != 0 && self.hearts_ui.age >= 0.55 => {
                if self.save.hearts.cpu_step() {
                    self.hearts_changed(EffectKind::Move);
                    if !self.save.settings.reduced_motion
                        && let Some(play) = self.save.hearts.trick.last()
                    {
                        self.hearts_ui.motion.play(play.seat, play.card, None);
                    }
                }
            }
            Phase::TrickComplete if self.hearts_ui.age >= 1.2 => {
                let before = self.save.hearts.clone();
                if !self.save.hearts.resolve_trick() {
                    return;
                }
                self.hearts_changed(EffectKind::Run);
                if !self.save.settings.reduced_motion {
                    self.hearts_ui
                        .motion
                        .collect(&before, self.save.hearts.turn);
                }
                if self.save.hearts.phase == Phase::Playing {
                    self.hearts_ui.message = format!(
                        "{} took the trick.",
                        ["YOU", "LEFT", "ACROSS", "RIGHT"][self.save.hearts.turn]
                    );
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pass_selection_confirmation_and_navigation_preserve_other_cabinets() {
        let mut s = crate::tests::test_arcade();
        let before = s.save.clone();
        s.open(Screen::Hearts);
        for i in 0..3 {
            s.act(Action::HeartsCard(i));
        }
        s.act(Action::HeartsCard(3));
        assert_eq!(s.hearts_ui.selected, vec![0, 1, 2]);
        s.act(Action::HeartsPass);
        assert_eq!(s.save.hearts.phase, Phase::Playing);
        assert!(s.hearts_ui.selected.is_empty());
        s.act(Action::HeartsNew);
        let current = s.save.hearts.clone();
        assert!(matches!(s.modal, Some(Pending::NewHearts)));
        s.act(Action::Cancel);
        assert_eq!(s.save.hearts, current);
        s.open(Screen::Settings);
        assert_eq!(s.save.hearts, current);
        s.act(Action::Back);
        assert_eq!(s.screen, Screen::Hearts);
        s.act(Action::HeartsNew);
        s.act(Action::Confirm);
        assert_eq!(s.save.hearts.phase, Phase::Passing);
        assert_eq!(s.save.solitaire, before.solitaire);
        assert_eq!(s.save.freecell, before.freecell);
        assert_eq!(s.save.spider, before.spider);
        assert_eq!(s.save.blackjack, before.blackjack);
        assert_eq!(s.save.baccarat, before.baccarat);
        assert_eq!(s.save.mahjong, before.mahjong);
        assert_eq!(s.save.sudoku, before.sudoku);
        assert_eq!(s.save.minesweeper, before.minesweeper);
    }
    #[test]
    fn paced_cpu_and_visible_tricks() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Hearts;
        s.save.hearts.pass(&[0, 1, 2]);
        while s.save.hearts.phase != Phase::TrickComplete {
            s.hearts_ui.motion.clear();
            if s.save.hearts.turn == 0 {
                let index = s.save.hearts.legal_indices(0)[0];
                s.hearts_card(index);
            } else {
                let before = s.save.hearts.clone();
                s.hearts_tick(0.54);
                assert_eq!(s.save.hearts, before);
                s.hearts_tick(0.02);
                assert_ne!(s.save.hearts, before);
            }
        }
        let trick = s.save.hearts.clone();
        s.hearts_tick(1.19);
        assert_eq!(s.save.hearts, trick);
        s.hearts_tick(0.02);
        assert_eq!(s.save.hearts.completed_tricks, 1);
        assert!(s.save.hearts.trick.is_empty());
        s.save.validate().unwrap();
    }
    #[test]
    fn passes_and_tricks_save_before_motion_and_settle_safely_on_navigation() {
        use crate::hearts_motion::{Anchor, Kind};
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Hearts;
        s.hearts_ui.selected = vec![0, 1, 2];
        s.hearts_pass();
        assert_eq!(s.save.hearts.phase, Phase::Playing);
        assert_eq!(s.hearts_ui.motion.kind(), Some(Kind::Pass));
        assert_eq!(s.hearts_ui.motion.flights().len(), 12);
        assert_eq!(Save::from_json(&s.save.to_json().unwrap()).unwrap(), s.save);
        let committed = s.save.hearts.clone();
        if s.save.hearts.turn == 0 {
            s.hearts_card(s.save.hearts.legal_indices(0)[0]);
        }
        assert_eq!(s.save.hearts, committed);
        s.open(Screen::Lobby);
        s.open(Screen::Hearts);
        assert!(!s.hearts_ui.motion.busy());
        assert_eq!(s.save.hearts, committed);
        while s.save.hearts.phase != Phase::TrickComplete {
            if s.save.hearts.turn == 0 {
                s.save.hearts.play(s.save.hearts.legal_indices(0)[0]);
            } else {
                s.save.hearts.cpu_step();
            }
        }
        let before = s.save.hearts.clone();
        s.hearts_ui.age = 1.2;
        s.hearts_tick(0.01);
        assert_eq!(s.save.hearts.completed_tricks, 1);
        let winner = s.save.hearts.turn;
        assert_eq!(s.save.hearts.captured[winner].len(), 4);
        assert_eq!(s.hearts_ui.motion.kind(), Some(Kind::Collect));
        assert!(
            s.hearts_ui
                .motion
                .flights()
                .iter()
                .all(|f| f.to == Anchor::Hand(winner)
                    && before.trick.iter().any(|p| p.card == f.card))
        );
        s.open_modal(Pending::NewHearts);
        assert!(!s.hearts_ui.motion.busy());
        assert_eq!(Save::from_json(&s.save.to_json().unwrap()).unwrap(), s.save);
    }

    #[test]
    fn reduced_motion_commits_without_flights_and_deals_are_interruptible() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Hearts;
        s.save.settings.reduced_motion = true;
        s.hearts_ui.selected = vec![0, 1, 2];
        s.hearts_pass();
        assert!(!s.hearts_ui.motion.busy());
        while s.save.hearts.phase != Phase::TrickComplete {
            if s.save.hearts.turn == 0 {
                s.save.hearts.play(s.save.hearts.legal_indices(0)[0]);
            } else {
                s.save.hearts.cpu_step();
            }
        }
        s.hearts_ui.age = 1.2;
        s.hearts_tick(0.01);
        assert_eq!(s.save.hearts.completed_tricks, 1);
        assert!(!s.hearts_ui.motion.busy());
        s.save.settings.reduced_motion = false;
        s.new_hearts();
        assert_eq!(
            s.hearts_ui.motion.kind(),
            Some(crate::hearts_motion::Kind::Deal)
        );
        let committed = s.save.hearts.clone();
        s.open(Screen::Settings);
        assert!(!s.hearts_ui.motion.busy());
        assert_eq!(s.save.hearts, committed);
    }

    #[test]
    fn hovering_keeps_the_resting_footprint_and_selection_lift_with_reduced_motion() {
        for reduced in [false, true] {
            let mut s = crate::tests::test_arcade();
            s.screen = Screen::Hearts;
            s.save.settings.reduced_motion = reduced;
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
            let mut app = App::new();
            app.insert_resource(s)
                .insert_resource(time)
                .init_resource::<ViewLayout>()
                .init_resource::<rendering::Frame>()
                .init_resource::<rendering::RenderOptions>()
                .add_systems(Update, presentation::paint);
            app.world_mut().spawn((
                Window {
                    resolution: WindowResolution::new(1280, 800),
                    ..default()
                },
                PrimaryWindow,
            ));
            app.update();
            let before = app.world().resource::<Arcade>().hearts_ui.card_positions[0];
            let hit = app
                .world()
                .resource::<ViewLayout>()
                .hits
                .iter()
                .find(|h| matches!(h.action, Action::HeartsCard(0)))
                .unwrap()
                .rect;
            let point = Vec2::new(hit.min.x + 8.0, hit.max.y - 2.0);
            app.world_mut().resource_mut::<Arcade>().cursor = Some(point);
            for _ in 0..40 {
                app.update();
                assert_eq!(app.world().resource::<Arcade>().hearts_ui.hovered, Some(0));
            }
            assert!(
                app.world().resource::<Arcade>().hearts_ui.card_positions[0].y < before.y - 17.0
            );
            let action = app
                .world()
                .resource::<ViewLayout>()
                .hits
                .iter()
                .rev()
                .find(|h| h.rect.contains(point))
                .unwrap()
                .action;
            assert!(matches!(action, Action::HeartsCard(0)));
            app.world_mut().resource_mut::<Arcade>().act(action);
            app.world_mut().resource_mut::<Arcade>().cursor = None;
            app.update();
            let s = app.world().resource::<Arcade>();
            assert_eq!(s.hearts_ui.hovered, None);
            assert_eq!(s.hearts_ui.selected, vec![0]);
            assert!(s.hearts_ui.card_positions[0].y < before.y - 17.0);
        }
    }

    #[test]
    fn cards_and_controls_fit_supported_viewports() {
        for (width, height) in [(360, 560), (390, 740), (1280, 720), (1280, 800)] {
            let mut s = crate::tests::test_arcade();
            s.screen = Screen::Hearts;
            let mut f = rendering::Frame::default();
            f.size = Vec2::new(width as f32, height as f32);
            let l = ViewLayout::default();
            // Paint through the shared system to retain its actual margins.
            let mut app = App::new();
            app.insert_resource(s)
                .insert_resource(Time::<()>::default())
                .insert_resource(l)
                .insert_resource(f)
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
            let mut count = 0;
            for h in &l.hits {
                if matches!(h.action, Action::HeartsCard(_) | Action::HeartsNew) {
                    assert!(
                        h.rect.min.x >= 0.0 && h.rect.max.x <= width as f32,
                        "horizontal {width}x{height}"
                    );
                    assert!(
                        h.rect.min.y >= 0.0 && h.rect.max.y <= height as f32 - 33.0,
                        "vertical {width}x{height}: {:?}",
                        h.rect
                    );
                    if matches!(h.action, Action::HeartsCard(_)) {
                        count += 1;
                    }
                }
            }
            assert_eq!(count, 13);
        }
    }
}
