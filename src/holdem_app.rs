//! Hold'em bankroll deltas commit immediately; the current hand stays in memory.
use crate::table_feedback::{EffectKind, TableFeedback};
use crate::*;
use gamevex_core::holdem::{Action as Play, Game, MAX_BUY_IN};

pub(crate) struct HoldemUi {
    pub game: Game,
    pub buy_in: u32,
    pub raise_target: u32,
    pub age: f32,
    pub duration: f32,
    pub board_from: usize,
    pub dealing: bool,
    pub hovered: Option<usize>,
    pub positions: Vec<Vec2>,
    pub board_bounds: Vec<Rect>,
    pub hand_bounds: Vec<Rect>,
    pub feedback: TableFeedback,
    pub pending_result: Option<EffectKind>,
}
impl Default for HoldemUi {
    fn default() -> Self {
        Self {
            game: Game::idle(1),
            buy_in: 100,
            raise_target: 0,
            age: 1.0,
            duration: 0.0,
            board_from: 0,
            dealing: false,
            hovered: None,
            positions: vec![],
            board_bounds: vec![],
            hand_bounds: vec![],
            feedback: TableFeedback::default(),
            pending_result: None,
        }
    }
}
impl HoldemUi {
    pub fn busy(&self, reduced: bool) -> bool {
        !reduced && self.age < self.duration
    }
    pub fn settle_motion(&mut self) {
        self.age = 10.0;
        self.board_from = self.game.board.len();
        self.dealing = false;
        self.hovered = None;
        self.feedback.clear();
        self.pending_result = None;
    }
}
impl Arcade {
    pub(crate) fn holdem_stake_bounds(&self) -> (u32, u32) {
        let max = self.save.holdem_balance.min(u64::from(MAX_BUY_IN)) as u32;
        (max.min(20), max)
    }
    fn holdem_ready(&self) -> bool {
        self.screen == Screen::Holdem && !self.holdem_ui.busy(self.save.settings.reduced_motion)
    }
    pub(crate) fn holdem_stake(&mut self, delta: i32) {
        if !self.holdem_ready() || !self.holdem_ui.game.terminal() {
            return;
        }
        let (min, max) = self.holdem_stake_bounds();
        self.holdem_ui.buy_in = (i64::from(self.holdem_ui.buy_in) + i64::from(delta))
            .clamp(i64::from(min), i64::from(max)) as u32;
    }
    pub(crate) fn holdem_stake_min(&mut self) {
        if self.holdem_ready() && self.holdem_ui.game.terminal() {
            self.holdem_ui.buy_in = self.holdem_stake_bounds().0;
        }
    }
    pub(crate) fn holdem_stake_max(&mut self) {
        if self.holdem_ready() && self.holdem_ui.game.terminal() {
            self.holdem_ui.buy_in = self.holdem_stake_bounds().1;
        }
    }
    pub(crate) fn holdem_opponents(&mut self, delta: i32) {
        if !self.holdem_ready() || !self.holdem_ui.game.terminal() {
            return;
        }
        let n = (self.holdem_ui.game.seats.len() as i32 - 1 + delta).clamp(1, 4) as usize;
        self.holdem_ui.game = Game::idle(n);
        self.holdem_ui.positions.clear();
    }
    pub(crate) fn holdem_raise(&mut self, delta: i32) {
        if !self.holdem_ready() || self.holdem_ui.game.turn != Some(0) {
            return;
        }
        if let Some(b) = self.holdem_ui.game.bet_bounds(0) {
            self.holdem_ui.raise_target =
                (i64::from(self.holdem_ui.raise_target) + i64::from(delta))
                    .clamp(i64::from(b.min_total), i64::from(b.max_total)) as u32;
        }
    }
    pub(crate) fn holdem_raise_min(&mut self) {
        if let Some(b) = self.holdem_ui.game.bet_bounds(0) {
            self.holdem_ui.raise_target = b.min_total;
        }
    }
    pub(crate) fn holdem_raise_pot(&mut self) {
        if let Some(b) = self.holdem_ui.game.bet_bounds(0) {
            self.holdem_ui.raise_target = (self.holdem_ui.game.current_bet
                + self.holdem_ui.game.pot())
            .clamp(b.min_total, b.max_total);
        }
    }
    pub(crate) fn holdem_start(&mut self) {
        if !self.holdem_ready() || !self.holdem_ui.game.terminal() {
            return;
        }
        let (min, max) = self.holdem_stake_bounds();
        if max == 0 {
            return;
        }
        let buy = self.holdem_ui.buy_in.clamp(min, max);
        if self.holdem_ui.game.start(buy, fresh_seed()) {
            self.holdem_ui.buy_in = buy;
            self.holdem_changed(buy, 0, true);
        }
    }
    pub(crate) fn holdem_play(&mut self, action: Play) {
        if !self.holdem_ready() || self.holdem_ui.game.turn != Some(0) {
            return;
        }
        let action = match action {
            Play::Bet(_) => Play::Bet(self.holdem_ui.raise_target),
            Play::Raise(_) => Play::Raise(self.holdem_ui.raise_target),
            other => other,
        };
        let before = self.holdem_ui.game.seats[0].stack;
        let board = self.holdem_ui.game.board.len();
        if self.holdem_ui.game.apply(action) {
            self.holdem_changed(before, board, false);
        }
    }
    pub(crate) fn holdem_skip(&mut self) {
        if !self.holdem_ready()
            || !self.holdem_ui.game.seats[0].folded
            || self.holdem_ui.game.terminal()
        {
            return;
        }
        // Resolve the same CPU decisions without replaying every action cue.
        let mut resolved = self.holdem_ui.game.clone();
        for _ in 0..1024 {
            if resolved.terminal() {
                break;
            }
            if !resolved.cpu_step() {
                return;
            }
        }
        if !resolved.terminal() {
            return;
        }
        let before = self.holdem_ui.game.seats[0].stack;
        let board = self.holdem_ui.game.board.len();
        self.holdem_ui.game = resolved;
        self.holdem_ui.feedback.clear();
        self.holdem_changed(before, board, false);
    }
    fn holdem_changed(&mut self, previous_stack: u32, board_from: usize, dealing: bool) {
        let stack = self.holdem_ui.game.seats[0].stack;
        self.save.holdem_balance = self
            .save
            .holdem_balance
            .saturating_sub(u64::from(previous_stack))
            .saturating_add(u64::from(stack))
            .min(1_000_000_000_000);
        self.holdem_ui.age = 0.0;
        self.holdem_ui.board_from = board_from;
        self.holdem_ui.dealing = dealing;
        self.holdem_ui.duration =
            0.35 + (self.holdem_ui.game.board.len().saturating_sub(board_from)) as f32 * 0.12;
        if dealing {
            self.holdem_ui.duration = self
                .holdem_ui
                .duration
                .max(0.25 + self.holdem_ui.game.seats.len() as f32 * 0.05);
        }
        self.holdem_ui.feedback.emit(
            if dealing || self.holdem_ui.game.board.len() > board_from {
                EffectKind::Deal
            } else {
                EffectKind::Move
            },
            &[],
        );
        if let Some(b) = self.holdem_ui.game.bet_bounds(0) {
            self.holdem_ui.raise_target = b.suggested_total;
        }
        if self.holdem_ui.game.terminal() {
            let kind = if stack > self.holdem_ui.game.buy_in {
                EffectKind::Win
            } else if stack < self.holdem_ui.game.buy_in {
                EffectKind::Stuck
            } else {
                EffectKind::Run
            };
            if self.save.settings.reduced_motion {
                self.holdem_ui.feedback.emit(kind, &[]);
            } else {
                self.holdem_ui.pending_result = Some(kind);
            }
        }
        self.persist();
    }
    pub(crate) fn holdem_tick(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.holdem_ui.feedback.tick(dt);
        self.holdem_ui.age += dt;
        if !self.holdem_ui.busy(self.save.settings.reduced_motion)
            && let Some(kind) = self.holdem_ui.pending_result.take()
        {
            self.holdem_ui.feedback.emit(kind, &[]);
        }
        if self.holdem_ui.age >= self.holdem_ui.duration.max(0.85)
            && self.holdem_ui.game.turn.is_some_and(|i| i != 0)
        {
            let before = self.holdem_ui.game.seats[0].stack;
            let board = self.holdem_ui.game.board.len();
            if self.holdem_ui.game.cpu_step() {
                self.holdem_changed(before, board, false);
            }
        }
    }
    pub(crate) fn holdem_abandon(&mut self) {
        self.holdem_ui.game = Game::idle(self.holdem_ui.game.seats.len() - 1);
        self.holdem_ui.positions.clear();
        self.holdem_ui.settle_motion();
    }
    pub(crate) fn holdem_restart(&mut self) {
        if self.holdem_ready() && self.holdem_ui.game.terminal() && self.save.holdem_balance == 0 {
            self.save.holdem_balance = 1000;
            self.holdem_abandon();
            self.holdem_ui.buy_in = 100;
            self.persist();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamevex_core::holdem::Phase;
    #[test]
    fn blinds_actions_and_settlement_save_only_the_true_stack_delta() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Holdem;
        s.save.settings.reduced_motion = true;
        let original = s.save.clone();
        s.holdem_start();
        assert_eq!(s.save.holdem_balance, 995);
        assert_eq!(s.holdem_ui.game.seats[0].stack, 95);
        let before = s.save.holdem_balance;
        s.holdem_play(Play::Check);
        assert_eq!(s.save.holdem_balance, before);
        s.holdem_play(Play::AllIn);
        for _ in 0..100 {
            if s.holdem_ui.game.terminal() {
                break;
            }
            if s.holdem_ui.game.turn == Some(0) {
                s.holdem_play(if s.holdem_ui.game.to_call(0) == 0 {
                    Play::Check
                } else {
                    Play::Call
                });
            } else {
                s.holdem_tick(2.0);
            }
        }
        assert!(s.holdem_ui.game.terminal());
        assert_eq!(
            s.save.holdem_balance,
            original.holdem_balance - 100 + u64::from(s.holdem_ui.game.seats[0].stack)
        );
        let mut normalized = s.save.clone();
        normalized.holdem_balance = original.holdem_balance;
        assert_eq!(normalized, original);
        assert_eq!(Save::from_json(&s.save.to_json().unwrap()).unwrap(), s.save);
        let before = s.save.holdem_balance;
        let hand = s.holdem_ui.game.hand_number;
        s.holdem_start();
        if before > 0 {
            assert_eq!(s.holdem_ui.game.hand_number, hand + 1);
            assert_eq!(s.holdem_ui.game.dealer, 1);
        }
    }
    #[test]
    fn settings_and_leave_confirmation_preserve_unspent_chips() {
        let mut s = crate::tests::test_arcade();
        s.screen = Screen::Holdem;
        s.save.settings.reduced_motion = true;
        s.holdem_start();
        let hand = s.holdem_ui.game.clone();
        let balance = s.save.holdem_balance;
        s.open(Screen::Settings);
        s.open(Screen::Holdem);
        assert_eq!(s.holdem_ui.game, hand);
        s.open(Screen::Lobby);
        assert!(matches!(s.modal, Some(Pending::LeaveHoldem(Screen::Lobby))));
        s.act(Action::Cancel);
        assert_eq!(s.holdem_ui.game, hand);
        assert_eq!(s.save.holdem_balance, balance);
        s.open(Screen::Settings);
        s.open(Screen::Lobby);
        assert!(matches!(s.modal, Some(Pending::LeaveHoldem(_))));
        s.act(Action::Confirm);
        assert_eq!(s.screen, Screen::Lobby);
        assert_eq!(s.holdem_ui.game.phase, Phase::Idle);
        assert_eq!(s.save.holdem_balance, balance);
    }
    #[test]
    fn tiny_bankrolls_and_animation_interruption_do_not_repeat_credits() {
        for balance in 1..20 {
            let mut s = crate::tests::test_arcade();
            s.screen = Screen::Holdem;
            s.save.holdem_balance = balance;
            s.holdem_start();
            for _ in 0..100 {
                if s.holdem_ui.game.terminal() {
                    break;
                }
                s.holdem_ui.settle_motion();
                if s.holdem_ui.game.turn == Some(0) {
                    s.holdem_play(Play::AllIn);
                } else {
                    s.holdem_tick(2.0);
                }
            }
            assert!(s.holdem_ui.game.terminal());
            let settled = s.save.holdem_balance;
            s.holdem_ui.settle_motion();
            s.holdem_tick(5.0);
            s.open(Screen::Settings);
            assert_eq!(s.save.holdem_balance, settled);
            assert_eq!(Save::from_json(&s.save.to_json().unwrap()).unwrap(), s.save);
        }
    }
    #[test]
    fn skip_only_resolves_folded_hands_and_matches_normal_cpu_settlement() {
        for seed in 1..65 {
            let mut s = crate::tests::test_arcade();
            s.screen = Screen::Holdem;
            s.save.settings.reduced_motion = true;
            s.holdem_ui.game = Game::idle(4);
            assert!(s.holdem_ui.game.start(100, seed));
            while s.holdem_ui.game.turn != Some(0) && !s.holdem_ui.game.terminal() {
                s.holdem_tick(2.0);
            }
            let active = s.holdem_ui.game.clone();
            s.holdem_skip();
            assert_eq!(s.holdem_ui.game, active);
            if active.terminal() {
                continue;
            }
            s.holdem_play(Play::Fold);
            let balance = s.save.holdem_balance;
            let mut expected = s.holdem_ui.game.clone();
            for _ in 0..1024 {
                if expected.terminal() {
                    break;
                }
                assert!(expected.cpu_step());
            }
            assert!(expected.terminal());
            s.holdem_skip();
            assert_eq!(s.holdem_ui.game, expected);
            assert_eq!(s.save.holdem_balance, balance);
            s.holdem_skip();
            s.holdem_tick(5.0);
            assert_eq!(s.holdem_ui.game, expected);
            assert_eq!(s.save.holdem_balance, balance);
        }
    }
    #[test]
    fn table_controls_fit_all_supported_viewports() {
        for (width, height) in [(360, 560), (390, 740), (1280, 720), (1280, 800)] {
            for mode in 0..4 {
                let mut s = crate::tests::test_arcade();
                s.screen = Screen::Holdem;
                s.save.settings.reduced_motion = true;
                if mode == 3 {
                    s.holdem_opponents(3);
                }
                if mode > 0 {
                    s.holdem_start();
                }
                if mode >= 2 {
                    while !s.holdem_ui.game.terminal() && s.holdem_ui.game.turn != Some(0) {
                        s.holdem_tick(2.0);
                    }
                    s.holdem_play(Play::Fold);
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
                let ui = &app.world().resource::<Arcade>().holdem_ui;
                for hand in &ui.hand_bounds {
                    for board in &ui.board_bounds {
                        assert!(
                            hand.max.x <= board.min.x
                                || hand.min.x >= board.max.x
                                || hand.max.y <= board.min.y
                                || hand.min.y >= board.max.y,
                            "board overlaps the hand at {width}x{height}: {hand:?} / {board:?}"
                        );
                    }
                }
                for hit in &app.world().resource::<ViewLayout>().hits {
                    assert!(
                        hit.rect.min.x >= 0.0 && hit.rect.max.x <= width as f32,
                        "x {width} {height} {:?}",
                        hit.rect
                    );
                    assert!(
                        hit.rect.min.y >= 0.0 && hit.rect.max.y <= height as f32 - 33.0,
                        "y {width} {height} {:?}",
                        hit.rect
                    );
                }
            }
        }
    }
}
