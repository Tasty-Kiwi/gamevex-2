use super::*;

impl Arcade {
    pub(super) fn load() -> Self {
        Self::from_storage(platform::load())
    }

    fn from_storage(result: Result<Option<String>, String>) -> Self {
        let (save, persistence, protected_save) = match result {
            Ok(Some(json)) => match Save::from_json(&json) {
                Ok(loaded) => (
                    loaded,
                    "Session restored · saved on this device".into(),
                    false,
                ),
                Err(error) => (
                    Save::new(fresh_seed()),
                    format!("Save needs recovery. Open Settings. {error}"),
                    true,
                ),
            },
            Ok(None) => (Save::new(fresh_seed()), "Local save ready".into(), false),
            Err(error) => (
                Save::new(fresh_seed()),
                format!("Playing in memory · save read failed: {error}"),
                true,
            ),
        };
        let message = match save.minesweeper.game.status {
            Status::Ready => "Your first reveal is always safe.",
            Status::Playing => "Saved field restored. The timer resumes during active play.",
            Status::Won => "Field cleared. Start a new run when ready.",
            Status::Lost => "Mine hit. Start a new run when ready.",
        }
        .to_owned();
        Self {
            save,
            screen: Screen::Lobby,
            settings_return: Screen::Lobby,
            lab: CardLab::default(),
            modal: None,
            focus: 0,
            baccarat_focus: None,
            modal_return_focus: 0,
            settings_return_focus: 0,
            keyboard: false,
            cell: 0,
            flag_mode: false,
            pan: Vec2::ZERO,
            settings_scroll: 0.0,
            lobby_scroll: 0.0,
            lobby_category: None,
            manual_topic: None,
            cursor: None,
            card_positions: [Vec2::splat(-1000.0); 5],
            drag_offset: Vec2::ZERO,
            motion_time: 0.0,
            blackjack_motion: Default::default(),
            baccarat_motion: Default::default(),
            casino_feedback: Default::default(),
            connect4_ui: Default::default(),
            sudoku_ui: Default::default(),
            mahjong_ui: Default::default(),
            patience_ui: Default::default(),
            hearts_ui: Default::default(),
            cribbage_ui: Default::default(),
            holdem_ui: Default::default(),
            backgammon_ui: Default::default(),
            roulette_ui: Default::default(),
            message,
            persistence,
            protected_save,
            backend: "Detecting graphics adapter…".into(),
            fps: 0.0,
            last_active: false,
            lifecycle: platform::lifecycle_epoch(),
            fractional_ms: 0.0,
            save_age: 0.0,
            sound: None,
            audio_started: false,
        }
    }

    pub(super) fn persist(&mut self) {
        self.save_age = 0.0;
        match self.save.to_json() {
            Ok(json) => {
                platform::diagnostics_save(&json);
                if !self.protected_save {
                    platform::stage_snapshot(&self.save, true);
                    self.persistence = match platform::store(&json) {
                        Ok(()) => "Session saved on this device".into(),
                        Err(error) => format!("Playing in memory · save failed: {error}"),
                    };
                }
            }
            Err(error) if !self.protected_save => {
                self.persistence = format!("Playing in memory · save failed: {error}");
            }
            Err(_) => {}
        }
    }
}

pub(super) fn lifecycle(
    time: Res<Time<Real>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<Arcade>,
    mut focused: MessageReader<WindowFocused>,
    mut close: MessageReader<WindowCloseRequested>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let epoch = platform::lifecycle_epoch();
    let transitioned = focused.read().count() > 0 || epoch != state.lifecycle;
    let foreground = window.focused && platform::visible();
    let active = foreground
        && matches!(
            state.screen,
            Screen::Mines
                | Screen::Sudoku
                | Screen::Mahjong
                | Screen::Solitaire
                | Screen::FreeCell
                | Screen::Spider
                | Screen::Backgammon
        )
        && state.modal.is_none();
    let delta = time.delta_secs_f64();
    let millis = account_play_time(
        delta,
        active,
        state.last_active,
        transitioned,
        &mut state.fractional_ms,
    );
    if patience_app::is_family(state.screen) && state.patience_game().has_progress() {
        state.patience_game_mut().advance_time(millis);
    }
    if state.screen == Screen::Mahjong {
        state.save.mahjong.advance_time(millis);
        if active && let Some(age) = &mut state.mahjong_ui.completion_age {
            *age = (*age + delta.min(0.1) as f32).min(2.0);
        }
    } else if state.screen == Screen::Mines {
        state.save.minesweeper.game.advance_elapsed(millis, active);
    } else if state.screen == Screen::Sudoku {
        state.save.sudoku.game.advance_time(millis);
        if active && let Some(age) = &mut state.sudoku_ui.completion_age {
            *age = (*age + delta.min(0.1) as f32).min(3.0);
        }
    }
    if state.screen == Screen::Connect4 && foreground && !transitioned && state.modal.is_none() {
        state.connect4_tick(delta.min(0.1) as f32);
    }
    if state.screen == Screen::Backgammon && foreground && !transitioned && state.modal.is_none() {
        state.backgammon_tick(delta as f32);
    }
    if state.screen == Screen::Roulette && foreground && !transitioned && state.modal.is_none() {
        state.roulette_tick(delta as f32);
    }
    if state.screen == Screen::Holdem && foreground && !transitioned && state.modal.is_none() {
        state.holdem_tick(delta as f32);
    }
    if state.screen == Screen::Cribbage && foreground && !transitioned && state.modal.is_none() {
        state.cribbage_tick(delta as f32);
    }
    if state.screen == Screen::Hearts && foreground && !transitioned && state.modal.is_none() {
        state.hearts_tick(delta as f32);
    }
    if !foreground || transitioned {
        state.lab.cancel_drag();
        state.patience_ui.drag = None;
        state.patience_ui.motion.clear();
        state.patience_ui.feedback.clear();
        state.mahjong_ui.feedback.clear();
        state.connect4_ui.settle_motion();
        state.backgammon_ui.settle_motion();
        state.holdem_ui.settle_motion();
        state.cribbage_ui.age = 1.0;
        state.cribbage_ui.hovered = None;
        state.hearts_ui.motion.clear();
        state.hearts_ui.hovered = None;
        state.hearts_ui.age = 0.0;
        state.hearts_ui.feedback.clear();
    }
    state.save_age += delta.min(0.5) as f32;
    state.motion_time += delta.min(0.1) as f32;
    if state.screen == Screen::Blackjack && foreground && state.modal.is_none() {
        let was_busy = state.blackjack_motion.busy();
        if state.save.settings.reduced_motion {
            state.blackjack_motion.clear();
        } else {
            state.blackjack_motion.tick(delta.min(0.1) as f32);
        }
        if was_busy && !state.blackjack_motion.busy() {
            state.focus = if state.save.blackjack.phase == blackjack::Phase::Betting {
                7
            } else {
                3
            };
        }
    }
    if state.screen == Screen::Baccarat && foreground && state.modal.is_none() {
        if state.save.settings.reduced_motion {
            state.baccarat_motion.clear();
        } else {
            state.baccarat_motion.tick(delta.min(0.1) as f32);
        }
    }
    if foreground {
        if state.save.settings.reduced_motion {
            state.patience_ui.motion.clear();
        } else if state.modal.is_none() {
            state.patience_ui.motion.tick(delta as f32);
        }
        state.mahjong_ui.feedback.tick(delta as f32);
        state.patience_ui.feedback.tick(delta as f32);
    }
    if foreground
        && state.modal.is_none()
        && matches!(state.screen, Screen::Blackjack | Screen::Baccarat)
    {
        state.casino_feedback.tick(delta as f32);
        let reduced = state.save.settings.reduced_motion;
        if state.screen == Screen::Blackjack {
            let view = state
                .blackjack_motion
                .sample(&state.save.blackjack, reduced);
            let hidden = state
                .blackjack_motion
                .dealer_hidden(&state.save.blackjack, reduced);
            let finished = reduced || !state.blackjack_motion.busy();
            state.casino_feedback.observe(
                view.hands.iter().map(|h| h.cards.len()).collect(),
                view.dealer.len(),
                hidden,
                finished,
            );
        } else {
            let view = state.baccarat_motion.sample(&state.save.baccarat, reduced);
            let finished = reduced || !state.baccarat_motion.busy();
            state.casino_feedback.observe(
                vec![view.player.len()],
                view.banker.len(),
                false,
                finished,
            );
        }
    }
    let timed_progress = match state.screen {
        Screen::Mines => state.save.minesweeper.game.status == Status::Playing,
        Screen::Sudoku => !state.save.sudoku.game.complete,
        Screen::Mahjong => !state.save.mahjong.zen && !state.save.mahjong.won(),
        Screen::Backgammon => {
            !state.save.backgammon.finished() && state.save.backgammon.elapsed_ms > 0
        }
        screen if patience_app::is_family(screen) => {
            state.patience_game().has_progress() && !state.patience_game().won()
        }
        _ => false,
    };
    let closing = close.read().count() > 0;
    if (state.last_active && !active)
        || transitioned
        || closing
        || (active && timed_progress && state.save_age >= 5.0)
    {
        state.persist();
    }
    platform::stage_elapsed(&state.save);
    state.last_active = active;
    state.lifecycle = epoch;
    if delta > 0.0 {
        state.fps = state.fps * 0.95 + (1.0 / delta as f32) * 0.05;
    }
}

pub(super) fn account_play_time(
    delta_seconds: f64,
    active: bool,
    was_active: bool,
    transitioned: bool,
    fractional_ms: &mut f64,
) -> u64 {
    if !active
        || !was_active
        || transitioned
        || !delta_seconds.is_finite()
        || !(0.0..0.5).contains(&delta_seconds)
    {
        return 0;
    }
    *fractional_ms += delta_seconds * 1_000.0;
    let whole_ms = fractional_ms.floor() as u64;
    *fractional_ms -= whole_ms as f64;
    whole_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_read_failure_keeps_automatic_writes_disabled() {
        let mut arcade = Arcade::from_storage(Err("permission denied".into()));
        arcade.persist();
        assert!(arcade.protected_save);
        assert!(arcade.persistence.contains("permission denied"));
        assert!(!Arcade::from_storage(Ok(None)).protected_save);
    }
}
