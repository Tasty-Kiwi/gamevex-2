//! Persist domain transitions before playing transient checker and dice motion.
use crate::table_feedback::{EffectKind, TableFeedback};
use crate::*;
use gamevex_core::backgammon::{
    self as bg, Board, Color as Side, Difficulty as BgDifficulty, Move, Phase,
};

pub(crate) struct BackgammonMotion {
    pub before: Board,
    pub movement: Move,
    pub color: Side,
}
pub(crate) struct BackgammonUi {
    pub selected: Option<u8>,
    pub rules: bool,
    pub age: f32,
    pub duration: f32,
    pub motion: Option<BackgammonMotion>,
    pub feedback: TableFeedback,
    pub point_bounds: Vec<Rect>,
    pub legal: Vec<Move>,
    pub message: String,
    legal_for: Option<(Board, Vec<u8>, Side, Phase)>,
    cpu_plan: Vec<Move>,
    fractional_ms: f64,
}
impl Default for BackgammonUi {
    fn default() -> Self {
        Self {
            selected: None,
            rules: false,
            age: 1.0,
            duration: 0.0,
            motion: None,
            feedback: TableFeedback::default(),
            point_bounds: vec![],
            legal: vec![],
            message: "Roll the dice to begin.".into(),
            legal_for: None,
            cpu_plan: vec![],
            fractional_ms: 0.0,
        }
    }
}
impl BackgammonUi {
    pub fn busy(&self, reduced: bool) -> bool {
        !reduced && self.age < self.duration
    }
    pub fn settle_motion(&mut self) {
        self.age = 10.0;
        self.motion = None;
        self.feedback.clear();
    }
    pub fn refresh(&mut self, g: &bg::Game) {
        let key = (g.board.clone(), g.dice.clone(), g.turn, g.phase);
        if self.legal_for.as_ref() != Some(&key) {
            self.legal = if g.phase == Phase::Moving {
                g.legal_moves()
            } else {
                vec![]
            };
            self.legal_for = Some(key);
        }
    }
}
impl Arcade {
    fn backgammon_ready(&self) -> bool {
        self.screen == Screen::Backgammon
            && !self.backgammon_ui.rules
            && !self.backgammon_ui.busy(self.save.settings.reduced_motion)
    }
    pub(crate) fn backgammon_point(&mut self, point: u8) {
        if !self.backgammon_ready()
            || self.save.backgammon.turn != Side::White
            || self.save.backgammon.phase != Phase::Moving
        {
            return;
        }
        self.backgammon_ui.refresh(&self.save.backgammon);
        if let Some(source) = self.backgammon_ui.selected {
            // Prefer the larger die if the same source/destination can consume either.
            let movement = self
                .backgammon_ui
                .legal
                .iter()
                .filter(|m| m.from == source && m.to == point)
                .max_by_key(|m| m.die)
                .copied();
            if let Some(m) = movement {
                self.backgammon_move(m);
                return;
            }
        }
        if self.backgammon_ui.legal.iter().any(|m| m.from == point) {
            self.backgammon_ui.selected = if self.backgammon_ui.selected == Some(point) {
                None
            } else {
                Some(point)
            };
            self.backgammon_ui.feedback.emit(EffectKind::Pick, &[]);
        }
    }
    pub(crate) fn backgammon_roll(&mut self) {
        if self.backgammon_ready()
            && self.save.backgammon.turn == Side::White
            && self.save.backgammon.phase == Phase::AwaitRoll
        {
            self.backgammon_do_roll();
        }
    }
    fn backgammon_do_roll(&mut self) {
        let turn = self.save.backgammon.turn;
        if !self.save.backgammon.roll() {
            return;
        }
        self.backgammon_ui.selected = None;
        self.backgammon_ui.motion = None;
        self.backgammon_ui.age = 0.0;
        self.backgammon_ui.duration = 0.45;
        self.backgammon_ui.cpu_plan.clear();
        self.backgammon_ui.message = if self.save.backgammon.turn != turn {
            "No legal moves. Turn passes.".into()
        } else if turn == Side::White {
            "Select a highlighted checker, then its destination.".into()
        } else {
            "CPU rolled.".into()
        };
        self.backgammon_ui.feedback.emit(EffectKind::Shuffle, &[]);
        self.backgammon_ui.refresh(&self.save.backgammon);
        self.persist();
    }
    fn backgammon_move(&mut self, movement: Move) {
        let before = self.save.backgammon.board.clone();
        let color = self.save.backgammon.turn;
        if !self.save.backgammon.apply(movement) {
            return;
        }
        self.backgammon_ui.selected = None;
        self.backgammon_ui.age = 0.0;
        self.backgammon_ui.duration = if movement.is_hit { 0.65 } else { 0.38 };
        self.backgammon_ui.motion = Some(BackgammonMotion {
            before,
            movement,
            color,
        });
        self.backgammon_ui.message = if movement.to == bg::OFF {
            "Checker borne off."
        } else if movement.is_hit {
            "Blot hit to the bar."
        } else {
            "Checker moved."
        }
        .into();
        self.backgammon_ui.feedback.emit(
            if movement.is_hit {
                EffectKind::Run
            } else {
                EffectKind::Move
            },
            &[],
        );
        if self.save.backgammon.finished() {
            if self.save.backgammon.winner == Some(Side::White) {
                self.save.backgammon_wins = self.save.backgammon_wins.saturating_add(1);
            }
            self.backgammon_ui.cpu_plan.clear();
        }
        self.backgammon_ui.refresh(&self.save.backgammon);
        self.persist();
    }
    pub(crate) fn backgammon_tick(&mut self, dt: f32) {
        if self.screen != Screen::Backgammon
            || self.modal.is_some()
            || !dt.is_finite()
            || dt <= 0.0
            || self.backgammon_ui.rules
        {
            return;
        }
        self.backgammon_ui.feedback.tick(dt);
        let was_busy = self.backgammon_ui.busy(self.save.settings.reduced_motion);
        self.backgammon_ui.age += dt;
        let g = &mut self.save.backgammon;
        if !g.finished() && (!g.rolled.is_empty() || g.turn_number > 0) {
            self.backgammon_ui.fractional_ms += f64::from(dt) * 1000.0;
            let ms = self.backgammon_ui.fractional_ms.floor() as u64;
            self.backgammon_ui.fractional_ms -= ms as f64;
            g.advance_time(ms);
        }
        if !self.backgammon_ui.busy(self.save.settings.reduced_motion)
            && self.backgammon_ui.motion.take().is_some()
            && self.save.backgammon.finished()
        {
            self.backgammon_ui.feedback.emit(
                if self.save.backgammon.winner == Some(Side::White) {
                    EffectKind::Win
                } else {
                    EffectKind::Stuck
                },
                &[],
            );
        }
        if self.save.backgammon.turn == Side::Black
            && !self.save.backgammon.finished()
            && self.backgammon_ui.age >= self.backgammon_ui.duration.max(0.8)
        {
            if self.save.backgammon.phase == Phase::AwaitRoll {
                self.backgammon_do_roll();
            } else {
                if self.backgammon_ui.cpu_plan.is_empty() {
                    let g = &self.save.backgammon;
                    self.backgammon_ui.cpu_plan = bg::choose_sequence(
                        &g.board,
                        &g.dice,
                        Side::Black,
                        g.difficulty,
                        g.rng_state,
                    );
                }
                if !self.backgammon_ui.cpu_plan.is_empty() {
                    let m = self.backgammon_ui.cpu_plan.remove(0);
                    self.backgammon_move(m);
                }
            }
        }
        if was_busy && !self.backgammon_ui.busy(self.save.settings.reduced_motion) {
            self.backgammon_ui.refresh(&self.save.backgammon);
        }
    }
    pub(crate) fn backgammon_new(&mut self, toggle_difficulty: bool) {
        if self.screen != Screen::Backgammon || self.backgammon_ui.rules {
            return;
        }
        let current = self.save.backgammon.difficulty;
        let difficulty = if toggle_difficulty {
            if current == BgDifficulty::Easy {
                BgDifficulty::Hard
            } else {
                BgDifficulty::Easy
            }
        } else {
            current
        };
        if !self.save.backgammon.finished()
            && (!self.save.backgammon.rolled.is_empty() || self.save.backgammon.turn_number > 0)
        {
            self.open_modal(Pending::BackgammonReplace(difficulty));
        } else {
            self.start_backgammon(difficulty);
        }
    }
    pub(crate) fn start_backgammon(&mut self, difficulty: BgDifficulty) {
        self.save.backgammon = bg::Game::new(fresh_seed(), difficulty);
        self.backgammon_ui = Default::default();
        self.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn almost_won() -> bg::Game {
        let mut game = bg::Game::new(1, BgDifficulty::Easy);
        game.board = Board {
            points: [bg::Point::default(); 24],
            bar: [0, 0],
            off: [14, 0],
        };
        game.board.points[0] = bg::Point {
            color: Some(Side::White),
            count: 1,
        };
        game.board.points[23] = bg::Point {
            color: Some(Side::Black),
            count: 15,
        };
        game.phase = Phase::Moving;
        game.dice = vec![1];
        game.rolled = vec![1, 2];
        game.turn_number = 8;
        game
    }
    #[test]
    fn human_cpu_transitions_save_only_this_game_and_pause_for_rules_settings_and_dialogs() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Backgammon;
        let original = s.save.clone();
        s.backgammon_roll();
        let mut actions = 0;
        while s.save.backgammon.turn == Side::White && s.save.backgammon.phase == Phase::Moving {
            s.backgammon_ui.settle_motion();
            s.backgammon_ui.refresh(&s.save.backgammon);
            let m = s.backgammon_ui.legal[0];
            s.backgammon_point(m.from);
            assert_eq!(s.backgammon_ui.selected, Some(m.from));
            s.backgammon_point(m.to);
            actions += 1;
            assert!(actions <= 4);
            assert!(s.save.validate().is_ok());
        }
        let cpu = s.save.backgammon.clone();
        s.backgammon_ui.rules = true;
        s.backgammon_tick(20.0);
        assert_eq!(s.save.backgammon, cpu);
        s.backgammon_ui.rules = false;
        s.open(Screen::Settings);
        s.backgammon_tick(20.0);
        assert_eq!(s.save.backgammon, cpu);
        s.open(Screen::Backgammon);
        s.backgammon_new(false);
        s.backgammon_tick(20.0);
        assert_eq!(s.save.backgammon, cpu);
        s.act(Action::Cancel);
        assert_eq!(s.save.backgammon, cpu);
        for _ in 0..12 {
            if s.save.backgammon.turn == Side::White {
                break;
            }
            s.backgammon_tick(1.0);
        }
        assert_eq!(s.save.backgammon.turn, Side::White);
        assert!(s.save.validate().is_ok());
        let restored = Save::from_json(&s.save.to_json().unwrap()).unwrap();
        assert_eq!(restored, s.save);
        let mut normalized = s.save.clone();
        normalized.backgammon = original.backgammon.clone();
        normalized.backgammon_wins = original.backgammon_wins;
        assert_eq!(normalized, original);
        s.backgammon_new(true);
        s.act(Action::Confirm);
        assert_eq!(s.save.backgammon.difficulty, BgDifficulty::Hard);
        assert!(s.save.backgammon.rolled.is_empty());
    }
    #[test]
    fn final_bear_off_credits_once_and_reload_does_not_repeat_sound_or_win() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Backgammon;
        s.save.backgammon = almost_won();
        s.backgammon_point(0);
        s.backgammon_point(bg::OFF);
        assert_eq!(s.save.backgammon_wins, 1);
        assert!(s.save.backgammon.finished());
        let ended = s.save.backgammon.clone();
        s.backgammon_tick(1.0);
        assert!(
            s.backgammon_ui
                .feedback
                .take_sounds()
                .contains(&casino_audio::SoundCue::TableWin)
        );
        s.backgammon_tick(1.0);
        assert!(s.backgammon_ui.feedback.take_sounds().is_empty());
        s.backgammon_point(bg::OFF);
        s.backgammon_roll();
        assert_eq!(s.save.backgammon_wins, 1);
        assert_eq!(s.save.backgammon, ended);
        s.save = Save::from_json(&s.save.to_json().unwrap()).unwrap();
        s.backgammon_ui = Default::default();
        s.backgammon_tick(2.0);
        assert_eq!(s.save.backgammon_wins, 1);
        assert!(s.backgammon_ui.feedback.take_sounds().is_empty());
    }
    #[test]
    fn board_controls_and_all_point_slots_fit_compact_phone_and_desktop() {
        for (width, height) in [(360, 560), (390, 740), (1280, 720), (1280, 800)] {
            for mode in 0..5 {
                let mut s = crate::tests::test_arcade();
                s.screen = Screen::Backgammon;
                if mode == 1 {
                    s.save.backgammon.roll();
                }
                if mode == 2 {
                    s.backgammon_ui.rules = true;
                }
                if mode == 3 {
                    s.save.backgammon.turn = Side::Black;
                    s.save.backgammon.turn_number = 1;
                }
                if mode == 4 {
                    s.save.backgammon = almost_won();
                    let m = s.save.backgammon.legal_moves()[0];
                    s.save.backgammon.apply(m);
                }
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
                let state = app.world().resource::<Arcade>();
                let slots = &state.backgammon_ui.point_bounds;
                assert_eq!(slots.len(), 26);
                for r in slots {
                    assert!(
                        r.min.x >= 0.0
                            && r.max.x <= width as f32
                            && r.min.y >= 150.0
                            && r.max.y <= height as f32 - 80.0,
                        "bad slot {width}x{height}: {r:?}"
                    );
                }
                assert!(slots[12].center().x < slots[23].center().x);
                assert!(slots[11].center().x < slots[0].center().x);
                for i in 0..11 {
                    assert!(
                        slots[i + 1].center().x < slots[i].center().x,
                        "bottompoints must descendlefttoright"
                    );
                }
                for i in 12..23 {
                    assert!(
                        slots[i].center().x < slots[i + 1].center().x,
                        "toppoints must ascendlefttoright"
                    );
                }
                let hits = &app.world().resource::<ViewLayout>().hits;
                for hit in hits {
                    assert!(
                        hit.rect.min.y >= 0.0
                            && hit.rect.max.y <= height as f32 - 33.0
                            && hit.rect.min.x >= 0.0
                            && hit.rect.max.x <= width as f32,
                        "bad control {width}x{height}: {:?}",
                        hit.rect
                    );
                }
                if mode == 2 {
                    assert!(hits.iter().all(|h| !matches!(
                        h.action,
                        Action::BackgammonPoint(_)
                            | Action::BackgammonRoll
                            | Action::BackgammonNew
                            | Action::BackgammonDifficulty
                    )));
                }
                if mode == 3 || mode == 4 {
                    assert!(
                        !hits
                            .iter()
                            .any(|h| matches!(h.action, Action::BackgammonRoll))
                    );
                }
            }
        }
    }
}
