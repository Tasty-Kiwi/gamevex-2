//! Spins settle before presentation so navigation/reload cannot lose or repeat awards.
use crate::table_feedback::{EffectKind, TableFeedback};
use crate::*;
use gamevex_core::roulette::{self, Bet, Resolution};

pub(crate) const SPIN_DURATION: f32 = 2.6;
pub(crate) struct RouletteUi {
    pub bet: Bet,
    pub stake: u64,
    pub result: Option<Resolution>,
    pub age: f32,
    pub spinning: bool,
    pub history: Vec<u8>,
    pub feedback: TableFeedback,
    pub strip_from: f32,
    pub strip_to: f32,
    pub table_pan: f32,
    balance_during_spin: u64,
}
impl Default for RouletteUi {
    fn default() -> Self {
        Self {
            bet: Bet::Red,
            stake: 25,
            result: None,
            age: SPIN_DURATION,
            spinning: false,
            history: vec![],
            feedback: TableFeedback::default(),
            strip_from: 0.0,
            strip_to: 0.0,
            table_pan: 0.0,
            balance_during_spin: 0,
        }
    }
}
impl RouletteUi {
    pub fn busy(&self) -> bool {
        self.spinning
    }
    pub fn progress(&self) -> f32 {
        (self.age / SPIN_DURATION).clamp(0.0, 1.0)
    }
    pub fn strip_position(&self) -> f32 {
        let progress = if self.spinning { self.progress() } else { 1.0 };
        let eased = 1.0 - (1.0 - progress).powi(3);
        self.strip_from + (self.strip_to - self.strip_from) * eased
    }
    pub fn display_balance(&self, saved: u64) -> u64 {
        if self.spinning {
            self.balance_during_spin
        } else {
            saved
        }
    }
    fn reveal(&mut self, audible: bool) {
        if !self.spinning {
            return;
        }
        self.spinning = false;
        self.age = SPIN_DURATION;
        if let Some(result) = self.result {
            self.history.insert(0, result.number);
            self.history.truncate(10);
            if audible {
                self.feedback.emit(
                    if result.won {
                        EffectKind::Win
                    } else {
                        EffectKind::Stuck
                    },
                    &[],
                );
            }
        }
    }
    pub fn settle_motion(&mut self) {
        self.reveal(false);
        self.feedback.clear();
    }
}
impl Arcade {
    fn roulette_ready(&self) -> bool {
        self.screen == Screen::Roulette && !self.roulette_ui.busy()
    }
    pub(crate) fn roulette_bet(&mut self, bet: Bet) {
        if self.roulette_ready() && bet.valid() {
            self.roulette_ui.bet = bet;
        }
    }
    pub(crate) fn roulette_stake(&mut self, delta: i64) {
        if !self.roulette_ready() {
            return;
        }
        let (min, max) = roulette::stake_bounds(self.save.roulette_balance);
        self.roulette_ui.stake =
            (self.roulette_ui.stake as i64 + delta).clamp(min as i64, max as i64) as u64;
    }
    pub(crate) fn roulette_stake_limit(&mut self, maximum: bool) {
        if self.roulette_ready() {
            let (min, max) = roulette::stake_bounds(self.save.roulette_balance);
            self.roulette_ui.stake = if maximum { max } else { min };
        }
    }
    pub(crate) fn roulette_spin(&mut self) {
        if self.roulette_ready() {
            self.roulette_spin_number(roulette::spin(fresh_seed()));
        }
    }
    fn roulette_spin_number(&mut self, number: u8) {
        if !self.roulette_ready() {
            return;
        }
        let (min, max) = roulette::stake_bounds(self.save.roulette_balance);
        let stake = self.roulette_ui.stake.clamp(min, max);
        let Some(result) = roulette::resolve(
            self.save.roulette_balance,
            stake,
            self.roulette_ui.bet,
            number,
        ) else {
            return;
        };
        self.roulette_ui.stake = stake;
        self.roulette_ui.balance_during_spin = self.save.roulette_balance - stake;
        self.save.roulette_balance = result.balance;
        let target = roulette::WHEEL_ORDER
            .iter()
            .position(|&n| n == number)
            .unwrap() as f32;
        self.roulette_ui.strip_from = self.roulette_ui.strip_to.rem_euclid(37.0);
        self.roulette_ui.strip_to = self.roulette_ui.strip_from
            + 37.0 * 4.0
            + (target - self.roulette_ui.strip_from).rem_euclid(37.0);
        self.roulette_ui.result = Some(result);
        self.roulette_ui.spinning = true;
        self.roulette_ui.age = 0.0;
        self.roulette_ui.feedback.clear();
        self.roulette_ui.feedback.emit(EffectKind::Shuffle, &[]);
        self.persist();
        if self.save.settings.reduced_motion {
            self.roulette_ui.reveal(true);
        }
    }
    pub(crate) fn roulette_tick(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.roulette_ui.feedback.tick(dt);
        if self.roulette_ui.spinning {
            self.roulette_ui.age += dt;
            if self.roulette_ui.age >= SPIN_DURATION || self.save.settings.reduced_motion {
                self.roulette_ui.reveal(true);
            }
        }
    }
    pub(crate) fn roulette_restart(&mut self) {
        if self.roulette_ready() && self.save.roulette_balance == 0 {
            self.save.roulette_balance = 1000;
            self.roulette_ui = Default::default();
            self.persist();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn awards_commit_once_before_animation_and_leave_reload_preserve_them() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Roulette;
        let before = s.save.clone();
        s.roulette_ui.bet = Bet::Number(0);
        s.roulette_spin_number(0);
        assert_eq!(s.save.roulette_balance, 1875);
        assert_eq!(s.roulette_ui.display_balance(s.save.roulette_balance), 975);
        assert!(s.roulette_ui.busy());
        assert!(s.roulette_ui.history.is_empty());
        s.roulette_spin_number(3);
        s.roulette_bet(Bet::Black);
        s.roulette_stake(5);
        assert_eq!(s.roulette_ui.bet, Bet::Number(0));
        assert_eq!(s.roulette_ui.stake, 25);
        s.open(Screen::Settings);
        s.open(Screen::Roulette);
        assert!(s.roulette_ui.busy());
        s.roulette_tick(3.0);
        assert_eq!(s.roulette_ui.history, vec![0]);
        s.roulette_tick(3.0);
        s.open(Screen::Lobby);
        assert!(s.roulette_ui.feedback.take_sounds().is_empty());
        let reload = Save::from_json(&s.save.to_json().unwrap()).unwrap();
        assert_eq!(reload.roulette_balance, 1875);
        let mut normalized = reload;
        normalized.roulette_balance = before.roulette_balance;
        assert_eq!(normalized, before);
    }
    #[test]
    fn tiny_balances_can_finish_and_restart_only_at_zero() {
        for balance in 1..5 {
            let mut s = crate::tests::test_arcade();
            s.screen = Screen::Roulette;
            s.save.settings.reduced_motion = true;
            s.save.roulette_balance = balance;
            s.roulette_restart();
            assert_eq!(s.save.roulette_balance, balance);
            s.roulette_spin_number(0);
            assert!(!s.roulette_ui.busy());
            assert_eq!(s.save.roulette_balance, 0);
            s.roulette_restart();
            assert_eq!(s.save.roulette_balance, 1000);
            assert_eq!(s.roulette_ui.stake, 25);
        }
    }
    #[test]
    fn leaving_during_spin_finishes_silently_without_replaying_results() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Roulette;
        s.roulette_spin_number(1);
        s.open(Screen::Lobby);
        assert_eq!(s.save.roulette_balance, 1025);
        assert!(!s.roulette_ui.busy());
        assert_eq!(s.roulette_ui.history, vec![1]);
        s.open(Screen::Roulette);
        s.roulette_tick(4.0);
        assert_eq!(s.roulette_ui.history, vec![1]);
        assert!(s.roulette_ui.feedback.take_sounds().is_empty());
    }
    #[test]
    fn controls_fit_supported_sizes() {
        for (width, height) in [(360, 560), (390, 740), (1280, 720), (1280, 800)] {
            for mode in [0, 1, 3] {
                let mut s = crate::tests::test_arcade();
                s.screen = Screen::Roulette;
                if mode == 1 {
                    s.roulette_spin_number(7);
                }
                if mode == 3 {
                    s.save.roulette_balance = 0;
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
                let hits = &app.world().resource::<ViewLayout>().hits;
                for hit in hits {
                    assert!(
                        hit.rect.min.x >= 0.0 && hit.rect.max.x <= width as f32,
                        "horizontal overflow {width}x{height}: {:?}",
                        hit.rect
                    );
                    assert!(
                        hit.rect.min.y >= 0.0 && hit.rect.max.y <= height as f32 - 33.0,
                        "vertical overflow {width}x{height}: {:?}",
                        hit.rect
                    );
                }
                let bets: Vec<_> = hits
                    .iter()
                    .filter(|h| {
                        matches!(h.action, Action::RouletteBet(_))
                            && h.rect.width() > 0.0
                            && h.rect.height() > 0.0
                    })
                    .collect();
                if mode == 0 {
                    if width >= 620 {
                        assert_eq!(bets.len(), 49);
                    } else {
                        assert!(bets.len() < 49);
                    }
                    assert!(!bets.is_empty());
                    for (i, a) in bets.iter().enumerate() {
                        assert!(a.rect.height() >= 28.0, "small target {:?}", a.rect);
                        for b in bets.iter().skip(i + 1) {
                            assert!(
                                a.rect.max.x <= b.rect.min.x
                                    || b.rect.max.x <= a.rect.min.x
                                    || a.rect.max.y <= b.rect.min.y
                                    || b.rect.max.y <= a.rect.min.y,
                                "overlapping bet controls {:?} {:?}",
                                a.rect,
                                b.rect
                            );
                        }
                    }
                } else if mode == 1 {
                    assert!(bets.is_empty());
                }
                if mode == 3 {
                    assert!(
                        hits.iter()
                            .any(|h| matches!(h.action, Action::RouletteRestart))
                    );
                    assert!(
                        !hits
                            .iter()
                            .any(|h| matches!(h.action, Action::RouletteSpin))
                    );
                }
            }
        }
    }
    #[test]
    fn keyboard_focus_scrolls_hidden_column_into_view_and_strip_lands_on_result() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Roulette;
        s.roulette_spin_number(17);
        s.roulette_tick(3.0);
        let position = s.roulette_ui.strip_position();
        assert_eq!(
            roulette::WHEEL_ORDER[position.rem_euclid(37.0) as usize],
            17
        );
        s.roulette_spin_number(0);
        assert_eq!(s.roulette_ui.strip_from, position.rem_euclid(37.0));
        s.roulette_tick(3.0);
        assert_eq!(
            roulette::WHEEL_ORDER[s.roulette_ui.strip_position().rem_euclid(37.0) as usize],
            0
        );
        let mut app = App::new();
        app.insert_resource(s)
            .insert_resource(Time::<()>::default())
            .init_resource::<ViewLayout>()
            .init_resource::<rendering::Frame>()
            .init_resource::<rendering::RenderOptions>()
            .add_systems(Update, presentation::paint);
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(360, 560),
                ..default()
            },
            PrimaryWindow,
        ));
        app.update();
        let index = app
            .world()
            .resource::<ViewLayout>()
            .hits
            .iter()
            .position(|h| matches!(h.action, Action::RouletteBet(Bet::Column3)))
            .unwrap();
        {
            let mut state = app.world_mut().resource_mut::<Arcade>();
            state.keyboard = true;
            state.focus = index;
        }
        app.update();
        let layout = app.world().resource::<ViewLayout>();
        assert_eq!(layout.hits[index].rect.width(), 44.0);
        assert!(app.world().resource::<Arcade>().roulette_ui.table_pan > 0.0);
    }
}
