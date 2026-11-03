mod backgammon_app;
#[cfg(any(target_arch = "wasm32", test))]
mod browser_audio;
mod casino_audio;
mod casino_feedback;
mod cribbage_app;
mod hearts_app;
mod hearts_motion;
mod holdem_app;
mod mahjong_app;
mod patience_app;
mod patience_motion;
mod platform;
mod presentation;
mod rendering;
mod roulette_app;
mod sudoku_app;
mod table_feedback;
mod touch_input;

use bevy::{
    audio::{AudioSink, AudioSinkPlayback, AudioSource, Volume},
    input::{
        ButtonState,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
    render::{
        RenderPlugin,
        renderer::RenderAdapterInfo,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
    window::{PrimaryWindow, WindowCloseRequested, WindowEvent, WindowFocused, WindowResolution},
};
use gamevex_core::{Command as GameCommand, cards::CardLab, *};
use presentation::{Action, Screen, ViewLayout};
use rendering::ArcadeRenderPlugin;

#[derive(Resource)]
pub(crate) struct Arcade {
    pub save: Save,
    pub screen: Screen,
    pub settings_return: Screen,
    pub lab: CardLab,
    pub modal: Option<Pending>,
    pub focus: usize,
    pub modal_return_focus: usize,
    pub settings_return_focus: usize,
    pub keyboard: bool,
    pub cell: usize,
    pub flag_mode: bool,
    pub pan: Vec2,
    pub settings_scroll: f32,
    pub lobby_scroll: f32,
    pub cursor: Option<Vec2>,
    pub card_positions: [Vec2; 5],
    pub drag_offset: Vec2,
    pub motion_time: f32,
    pub blackjack_motion: presentation::blackjack_motion::BlackjackMotion,
    pub baccarat_motion: presentation::baccarat_motion::BaccaratMotion,
    pub casino_feedback: casino_feedback::CasinoFeedback,
    pub sudoku_ui: sudoku_app::SudokuUi,
    pub mahjong_ui: mahjong_app::MahjongUi,
    pub patience_ui: patience_app::PatienceUi,
    pub hearts_ui: hearts_app::HeartsUi,
    pub cribbage_ui: cribbage_app::CribbageUi,
    pub holdem_ui: holdem_app::HoldemUi,
    pub backgammon_ui: backgammon_app::BackgammonUi,
    pub roulette_ui: roulette_app::RouletteUi,
    pub message: String,
    pub persistence: String,
    pub protected_save: bool,
    pub backend: String,
    pub fps: f32,
    pub last_active: bool,
    pub lifecycle: u64,
    pub fractional_ms: f64,
    pub save_age: f32,
    pub sound: Option<f32>,
    pub audio_started: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum Pending {
    New(Difficulty, bool),
    Recover,
    NewHearts,
    NewCribbage,
    LeaveBlackjack(Screen),
    LeaveHoldem(Screen),
    BackgammonReplace(gamevex_core::backgammon::Difficulty),
    NewSudoku(u8, sudoku::Difficulty),
    SudokuSetup,
    NewMahjong(mahjong::Layout, u64, bool),
    NewPatience(patience::Variant, u64),
}

impl Arcade {
    fn load() -> Self {
        let mut save = Save::new(fresh_seed());
        let (persistence, protected_save) = match platform::load() {
            Ok(Some(json)) => match Save::from_json(&json) {
                Ok(loaded) => {
                    save = loaded;
                    ("Session restored · saved on this device".into(), false)
                }
                Err(error) => (format!("Save needs recovery. Open Settings. {error}"), true),
            },
            Ok(None) => ("Local save ready".into(), false),
            Err(error) if error.starts_with("Invalid saved data:") => {
                (format!("Save needs recovery. Open Settings. {error}"), true)
            }
            Err(error) => (format!("Playing in memory · {error}"), false),
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
            modal_return_focus: 0,
            settings_return_focus: 0,
            keyboard: false,
            cell: 0,
            flag_mode: false,
            pan: Vec2::ZERO,
            settings_scroll: 0.0,
            lobby_scroll: 0.0,
            cursor: None,
            card_positions: [Vec2::splat(-1000.0); 5],
            drag_offset: Vec2::ZERO,
            motion_time: 0.0,
            blackjack_motion: Default::default(),
            baccarat_motion: Default::default(),
            casino_feedback: Default::default(),
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

    fn persist(&mut self) {
        self.save_age = 0.0;
        platform::stage_snapshot(&self.save, !self.protected_save);
        if let Ok(json) = self.save.to_json() {
            platform::diagnostics_save(&json);
        }
        if self.protected_save {
            return;
        }
        self.persistence = match self.save.to_json().and_then(|json| platform::store(&json)) {
            Ok(()) => "Session saved on this device".into(),
            Err(error) => format!("Playing in memory · save failed: {error}"),
        };
    }

    fn unfinished(&self) -> bool {
        self.save.minesweeper.game.status == Status::Playing
            || (self.save.minesweeper.game.status == Status::Ready
                && self.save.minesweeper.game.cells.iter().any(|c| c.flagged))
    }

    fn open(&mut self, screen: Screen) {
        let leaving_holdem = (self.screen == Screen::Holdem
            || (self.screen == Screen::Settings && self.settings_return == Screen::Holdem))
            && !matches!(screen, Screen::Holdem | Screen::Settings);
        if leaving_holdem && !self.holdem_ui.game.terminal() {
            self.open_modal(Pending::LeaveHoldem(screen));
            return;
        }
        if leaving_holdem {
            self.holdem_abandon();
        }
        if (self.screen == Screen::Roulette
            || (self.screen == Screen::Settings && self.settings_return == Screen::Roulette))
            && !matches!(screen, Screen::Roulette | Screen::Settings)
        {
            self.roulette_ui.settle_motion();
        }
        self.backgammon_ui.settle_motion();
        self.holdem_ui.settle_motion();
        self.cribbage_ui.age = 1.0;
        self.cribbage_ui.hovered = None;
        self.cribbage_ui.feedback.clear();
        self.hearts_ui.motion.clear();
        self.hearts_ui.age = 0.0;
        self.hearts_ui.hovered = None;
        self.hearts_ui.feedback.clear();
        self.patience_ui.drag = None;
        self.patience_ui.motion.clear();
        if self.screen == Screen::Blackjack
            && !matches!(screen, Screen::Blackjack | Screen::Settings)
            && self.save.blackjack.legal(blackjack::Command::Abandon)
        {
            self.open_modal(Pending::LeaveBlackjack(screen));
            return;
        }
        if screen == Screen::Settings {
            self.settings_scroll = 0.0;
            self.settings_return = self.screen;
            self.settings_return_focus = self.focus;
        }
        if screen != self.screen && screen != Screen::Settings && self.screen != Screen::Settings {
            self.casino_feedback.clear();
        }
        if self.screen == Screen::Sudoku && !matches!(screen, Screen::Sudoku | Screen::Settings) {
            self.sudoku_ui.history.clear();
        }
        if screen == Screen::Sudoku && self.screen != Screen::Settings {
            self.sudoku_ui.selected = self
                .save
                .sudoku
                .game
                .givens
                .iter()
                .position(|v| *v == 0)
                .unwrap_or(0);
            self.sudoku_ui.pan = Vec2::ZERO;
            self.sudoku_ui.follow = true;
            self.sudoku_ui.message.clear();
            self.sudoku_ui.completion_age = None;
        }
        if screen == Screen::Mahjong && self.screen != Screen::Settings {
            self.mahjong_ui = Default::default();
        }
        if patience_app::is_family(screen) && self.screen != Screen::Settings {
            self.patience_ui = Default::default();
        }
        self.mahjong_ui.feedback.clear();
        self.patience_ui.feedback.clear();
        self.screen = screen;
        if patience_app::is_family(screen) {
            self.patience_ui.can_finish = self.patience_game().can_auto_finish();
            if self.patience_game().moves == 0 && !self.save.settings.reduced_motion {
                let game = self.patience_game().clone();
                self.patience_ui.motion.deal(&game);
            }
        }
        if !matches!(screen, Screen::Blackjack | Screen::Settings) {
            self.blackjack_motion.clear();
        }
        if !matches!(screen, Screen::Baccarat | Screen::Settings) {
            self.baccarat_motion.clear();
        }
        self.focus = match screen {
            Screen::Mines => 9,
            Screen::Blackjack if self.save.blackjack.phase == blackjack::Phase::Betting => 7,
            Screen::Blackjack => 3,
            Screen::Baccarat | Screen::Sudoku => 3,
            _ => 0,
        };
        self.keyboard = false;
        self.lab.cancel_drag();
        self.last_active = false;
        if screen == Screen::Cards {
            self.card_positions = [Vec2::splat(-1000.0); 5];
            self.motion_time = 0.0;
        }
        self.persist();
    }

    fn new_game(&mut self, difficulty: Difficulty, repeat: bool) {
        let seed = if repeat {
            self.save.minesweeper.game.seed
        } else {
            fresh_seed()
        };
        self.save.minesweeper.game = Game::new(difficulty, seed);
        self.pan = Vec2::ZERO;
        self.cell = 0;
        self.fractional_ms = 0.0;
        self.focus = 9;
        self.message = "Ready. Reveal a cell to begin; the first area is safe.".into();
        self.modal = None;
        self.persist();
    }

    fn command(&mut self, command: GameCommand) {
        let events = self.save.minesweeper.game.command(command);
        if events.is_empty() {
            return;
        }
        self.sound = Some(650.0);
        for event in events {
            match event {
                DomainEvent::Won => {
                    let record = self
                        .save
                        .minesweeper
                        .records
                        .record(&self.save.minesweeper.game);
                    self.message = if record {
                        "Field cleared. A new personal best!"
                    } else {
                        "Field cleared. Every safe cell is open."
                    }
                    .into();
                    self.sound = Some(1040.0);
                }
                DomainEvent::Lost => {
                    self.message =
                        "Mine hit. The field is revealed; start a new run when ready.".into();
                    self.sound = Some(160.0);
                }
                DomainEvent::Started => {
                    self.message =
                        "Sweep carefully. Flag suspected mines; chord matching numbers.".into()
                }
                _ => {}
            }
        }
        self.persist();
    }

    fn act(&mut self, action: Action) {
        let allowed_in_modal = match self.modal {
            Some(Pending::SudokuSetup) => matches!(
                action,
                Action::Cancel
                    | Action::SudokuChooseSize(_)
                    | Action::SudokuChooseDifficulty(_)
                    | Action::SudokuStart
            ),
            Some(_) => matches!(action, Action::Confirm | Action::Cancel),
            None => true,
        };
        if !allowed_in_modal {
            return;
        }
        let table_action = matches!(
            action,
            Action::BackgammonRoll
                | Action::BackgammonPoint(_)
                | Action::RouletteSpin
                | Action::HoldemPlay(..)
                | Action::HoldemStart
                | Action::CribbageCard(..)
                | Action::CribbageDiscard
                | Action::CribbageGo
                | Action::CribbageContinue
                | Action::CribbageNew
                | Action::HeartsCard(..)
                | Action::HeartsPass
                | Action::HeartsContinue
                | Action::HeartsNew
                | Action::PatiencePick(..)
                | Action::PatienceTarget(..)
                | Action::PatienceNew(..)
                | Action::PatienceDeal
                | Action::PatienceUndo
                | Action::PatienceFinish
                | Action::PatienceHint
                | Action::MahjongTile(..)
                | Action::MahjongHint
                | Action::MahjongUndo
                | Action::MahjongShuffle
                | Action::MahjongRestart
                | Action::MahjongNew(..)
                | Action::MahjongZen(..)
        ) || (matches!(action, Action::Confirm)
            && matches!(
                self.modal,
                Some(Pending::NewPatience(..) | Pending::NewMahjong(..))
            ));
        self.sound = if table_action || matches!(action, Action::Blackjack(_) | Action::Baccarat(_))
        {
            None
        } else {
            Some(440.0)
        };
        match action {
            Action::Open(screen) => self.open(screen),
            Action::Back => {
                if self.screen == Screen::Settings {
                    let target = self.settings_return;
                    let focus = self.settings_return_focus;
                    self.open(target);
                    self.focus = focus;
                } else {
                    self.open(Screen::Lobby);
                }
            }
            Action::New(difficulty, repeat) => {
                if self.unfinished() {
                    self.open_modal(Pending::New(difficulty, repeat));
                } else {
                    self.new_game(difficulty, repeat);
                }
            }
            Action::Confirm => match self.modal.take() {
                Some(Pending::New(difficulty, repeat)) => self.new_game(difficulty, repeat),
                Some(Pending::NewSudoku(size, difficulty)) => self.new_sudoku(size, difficulty),
                Some(Pending::NewMahjong(layout, seed, zen)) => self.new_mahjong(layout, seed, zen),
                Some(Pending::NewPatience(v, seed)) => self.new_patience(v, seed),
                Some(Pending::NewHearts) => self.new_hearts(),
                Some(Pending::NewCribbage) => self.new_cribbage(),
                Some(Pending::BackgammonReplace(difficulty)) => self.start_backgammon(difficulty),
                Some(Pending::SudokuSetup) => self.restore_modal_focus(),
                Some(Pending::Recover) => {
                    self.reset_saved_data();
                    self.persist();
                }
                Some(Pending::LeaveHoldem(target)) => {
                    self.holdem_abandon();
                    self.open(target);
                }
                Some(Pending::LeaveBlackjack(target)) => {
                    self.save.blackjack.command(blackjack::Command::Abandon);
                    self.open(target);
                }
                None => {}
            },
            Action::Cancel => {
                if matches!(self.modal, Some(Pending::SudokuSetup)) {
                    self.sudoku_ui.setup = None;
                }
                self.restore_modal_focus();
            }
            Action::BackgammonPoint(point) => self.backgammon_point(point),
            Action::BackgammonRoll => self.backgammon_roll(),
            Action::BackgammonNew => self.backgammon_new(false),
            Action::BackgammonDifficulty => self.backgammon_new(true),
            Action::BackgammonRules => self.backgammon_ui.rules = !self.backgammon_ui.rules,
            Action::RouletteBet(bet) => self.roulette_bet(bet),
            Action::RouletteStake(delta) => self.roulette_stake(delta),
            Action::RouletteStakeMin => self.roulette_stake_limit(false),
            Action::RouletteStakeMax => self.roulette_stake_limit(true),
            Action::RouletteSpin => self.roulette_spin(),
            Action::RouletteRules => self.roulette_ui.rules = !self.roulette_ui.rules,
            Action::RouletteRestart => self.roulette_restart(),
            Action::HoldemStart => self.holdem_start(),
            Action::HoldemPlay(action) => self.holdem_play(action),
            Action::HoldemStake(delta) => self.holdem_stake(delta),
            Action::HoldemStakeMax => self.holdem_stake_max(),
            Action::HoldemStakeMin => self.holdem_stake_min(),
            Action::HoldemOpponents(delta) => self.holdem_opponents(delta),
            Action::HoldemRaise(delta) => self.holdem_raise(delta),
            Action::HoldemRaiseMin => self.holdem_raise_min(),
            Action::HoldemRaisePot => self.holdem_raise_pot(),
            Action::HoldemRules => self.holdem_ui.rules = !self.holdem_ui.rules,
            Action::HoldemRestart => self.holdem_restart(),
            Action::HoldemSkip => self.holdem_skip(),
            Action::CribbageCard(index) => self.cribbage_card(index),
            Action::CribbageDiscard => self.cribbage_discard(),
            Action::CribbageGo => self.cribbage_go(),
            Action::CribbageContinue => self.cribbage_continue(),
            Action::CribbageNew => self.open_modal(Pending::NewCribbage),
            Action::CribbageRules => self.cribbage_ui.rules = !self.cribbage_ui.rules,
            Action::HeartsCard(index) => self.hearts_card(index),
            Action::HeartsPass => self.hearts_pass(),
            Action::HeartsContinue => self.hearts_continue(),
            Action::HeartsNew => self.request_hearts(),
            Action::HeartsRules => self.hearts_ui.rules = !self.hearts_ui.rules,
            Action::Blackjack(command) => {
                if self.blackjack_motion.busy() && !self.save.settings.reduced_motion {
                    return;
                }
                let before = self.save.blackjack.clone();
                let phase = self.save.blackjack.phase;
                if self.save.blackjack.command(command) {
                    self.casino_feedback
                        .blackjack(command, &before, &self.save.blackjack);
                    self.blackjack_motion
                        .start(command, &before, &self.save.blackjack);
                    if self.save.blackjack.phase != phase {
                        self.focus = if self.save.blackjack.phase == blackjack::Phase::Betting {
                            7
                        } else {
                            3
                        };
                    }
                    self.persist();
                }
            }
            Action::Baccarat(command) => {
                if self.baccarat_motion.busy() && !self.save.settings.reduced_motion {
                    return;
                }
                let before = self.save.baccarat.clone();
                if self.save.baccarat.command(command) {
                    self.casino_feedback
                        .baccarat(command, &before, &self.save.baccarat);
                    self.baccarat_motion
                        .start(command, &before, &self.save.baccarat);
                    self.focus = match command {
                        baccarat::Command::Select(_)
                            if self.save.baccarat.legal(baccarat::Command::Deal) =>
                        {
                            9
                        }
                        baccarat::Command::Deal
                        | baccarat::Command::NextRound
                        | baccarat::Command::Restart => 3,
                        _ => self.focus,
                    };
                    self.persist();
                }
            }
            Action::PatiencePick(p, i) => self.patience_pick(p, i),
            Action::PatienceTarget(p) => self.patience_target(p),
            Action::PatienceNew(v, repeat) => self.request_patience(v, repeat),
            Action::PatienceDeal => {
                let before = self.patience_game().clone();
                if self.patience_game_mut().deal() {
                    self.patience_changed("Dealt cards.");
                    self.patience_animate(&before);
                    let game = self.patience_game();
                    let anchors: Vec<_> = if matches!(
                        game.variant,
                        patience::Variant::Spider1
                            | patience::Variant::Spider2
                            | patience::Variant::Spider4
                    ) {
                        (0..game.variant.columns())
                            .map(|i| table_feedback::Anchor::Pile(patience::Pile::Tableau(i)))
                            .collect()
                    } else {
                        vec![table_feedback::Anchor::Pile(patience::Pile::Stock)]
                    };
                    self.patience_ui
                        .feedback
                        .emit(table_feedback::EffectKind::Deal, &anchors);
                } else {
                    self.patience_ui.message =
                        "No deal available. Spider needs every column filled.".into();
                    self.patience_ui.feedback.emit(
                        table_feedback::EffectKind::Invalid,
                        &[table_feedback::Anchor::Pile(patience::Pile::Stock)],
                    );
                }
            }
            Action::PatienceUndo => {
                let before = self.patience_game().clone();
                if self.patience_game_mut().undo() {
                    self.patience_ui.feedback.clear();
                    self.patience_changed("Move restored.");
                    self.patience_animate(&before);
                    self.patience_ui
                        .feedback
                        .emit(table_feedback::EffectKind::Undo, &[]);
                }
            }
            Action::PatienceFinish => self.patience_finish(),
            Action::PatienceHint => {
                self.patience_ui.motion.clear();
                if let Some((p, i, to)) = self.patience_game_mut().hint() {
                    self.patience_ui.selected = Some((p, i));
                    self.patience_ui.feedback.emit(
                        table_feedback::EffectKind::Hint,
                        &[
                            table_feedback::Anchor::Pile(p),
                            table_feedback::Anchor::Pile(to),
                        ],
                    );
                    self.patience_ui.message = format!(
                        "Hint: move to {}. Assisted run.",
                        patience_app::pile_name(to)
                    );
                    self.persist();
                } else {
                    self.patience_ui.message = "No move found. Try the stock or undo.".into();
                    self.patience_ui
                        .feedback
                        .emit(table_feedback::EffectKind::Stuck, &[]);
                }
            }
            Action::MahjongTile(i) => self.mahjong_tile(i),
            Action::MahjongHint => self.mahjong_hint(),
            Action::MahjongUndo => {
                if self.save.mahjong.undo() {
                    self.mahjong_ui.feedback.clear();
                    self.mahjong_ui
                        .feedback
                        .emit(table_feedback::EffectKind::Undo, &[]);
                    self.mahjong_ui.completion_age = None;
                    self.mahjong_ui.message = "Pair restored.".into();
                    self.mahjong_ui.selected = None;
                    self.mahjong_ui.hint = None;
                    self.persist();
                }
            }
            Action::MahjongShuffle => {
                if self.save.mahjong.stuck() && self.save.mahjong.shuffle(fresh_seed()) {
                    self.mahjong_ui.selected = None;
                    self.mahjong_ui.hint = None;
                    self.mahjong_ui.message = "Faces shuffled. Finish not guaranteed.".into();
                    self.mahjong_ui.feedback.clear();
                    self.mahjong_ui
                        .feedback
                        .emit(table_feedback::EffectKind::Shuffle, &[]);
                    self.persist();
                }
            }
            Action::MahjongRestart => {
                self.request_mahjong(self.save.mahjong.layout, self.save.mahjong.seed)
            }
            Action::MahjongNew(layout) => self.request_mahjong(layout, fresh_seed()),
            Action::MahjongZen(zen) => self.request_mahjong_mode(zen),
            Action::Sudoku(command) => self.sudoku_command(command),
            Action::SudokuCell => {}
            Action::SudokuSetup => self.open_sudoku_setup(),
            Action::SudokuStart => self.start_sudoku_setup(),
            Action::SudokuChooseSize(size) => {
                if sudoku::SIZES.contains(&size)
                    && let Some(mode) = &mut self.sudoku_ui.setup
                {
                    mode.0 = size;
                }
            }
            Action::SudokuChooseDifficulty(difficulty) => {
                if let Some(mode) = &mut self.sudoku_ui.setup {
                    mode.1 = difficulty;
                }
            }
            Action::SudokuHighlights => {
                self.sudoku_ui.show_highlights = !self.sudoku_ui.show_highlights
            }
            Action::SudokuPencil => {
                if !self.save.sudoku.game.complete {
                    self.sudoku_ui.pencil = !self.sudoku_ui.pencil;
                    self.focus = 3;
                }
            }
            Action::SudokuErrors => {
                self.save.sudoku.highlight_errors = !self.save.sudoku.highlight_errors;
                self.persist();
            }
            Action::SudokuSame => {
                self.save.sudoku.highlight_same = !self.save.sudoku.highlight_same;
                self.persist();
            }
            Action::FlagMode => self.flag_mode = !self.flag_mode,
            Action::Board => {}
            Action::Preset => {
                self.save.settings.display_preset = match self.save.settings.display_preset {
                    DisplayPreset::VectorCrt => DisplayPreset::PixelCrt,
                    DisplayPreset::PixelCrt => DisplayPreset::VectorCrt,
                };
                self.persist();
            }
            Action::Palette => {
                self.save.settings.color_scheme = match self.save.settings.color_scheme {
                    ColorScheme::Cyan => ColorScheme::Amber,
                    ColorScheme::Amber => ColorScheme::Cyan,
                };
                self.persist();
            }
            Action::Curvature(delta) => {
                self.save.settings.curvature =
                    (self.save.settings.curvature + delta).clamp(0.0, 1.0);
                self.persist();
            }
            Action::Vignette(delta) => {
                self.save.settings.vignette = (self.save.settings.vignette + delta).clamp(0.0, 1.0);
                self.persist();
            }
            Action::ChromaticAberration(delta) => {
                self.save.settings.chromatic_aberration =
                    (self.save.settings.chromatic_aberration + delta).clamp(0.0, 1.0);
                self.persist();
            }
            Action::Bloom(delta) => {
                self.save.settings.bloom = (self.save.settings.bloom + delta).clamp(0.0, 1.0);
                self.persist();
            }
            Action::Scanlines(delta) => {
                self.save.settings.scanlines =
                    (self.save.settings.scanlines + delta).clamp(0.0, 1.0);
                self.persist();
            }
            Action::Volume(delta) => {
                self.save.settings.volume = (self.save.settings.volume + delta).clamp(0.0, 1.0);
                self.persist();
            }
            Action::Crt => {
                self.save.settings.crt = !self.save.settings.crt;
                self.persist();
            }
            Action::Motion => {
                self.save.settings.reduced_motion = !self.save.settings.reduced_motion;
                self.persist();
            }
            Action::Mute => {
                self.save.settings.muted = !self.save.settings.muted;
                self.persist();
            }
            Action::Retry => self.persist(),
            Action::Recover => self.open_modal(Pending::Recover),
            Action::Flip => {
                self.lab.flip_selected();
            }
            Action::Deal => {
                self.lab.reset();
                self.card_positions = [Vec2::splat(-1000.0); 5];
                self.motion_time = 0.0;
            }
            Action::ReturnCard => {
                if let Some(i) = self.lab.selected {
                    for slot in &mut self.lab.slots {
                        if *slot == Some(i) {
                            *slot = None;
                        }
                    }
                }
            }
        }
    }

    fn reset_saved_data(&mut self) {
        self.save = Save::new(fresh_seed());
        self.protected_save = false;
        self.lab.reset();
        self.blackjack_motion.clear();
        self.baccarat_motion.clear();
        self.casino_feedback.clear();
        self.sudoku_ui = Default::default();
        self.mahjong_ui = Default::default();
        self.patience_ui = Default::default();
        self.hearts_ui = Default::default();
        self.cribbage_ui = Default::default();
        self.holdem_ui = Default::default();
        self.backgammon_ui = Default::default();
        self.roulette_ui = Default::default();
        self.modal = None;
        self.screen = Screen::Lobby;
        self.settings_return = Screen::Lobby;
        self.focus = 0;
        self.cell = 0;
        self.flag_mode = false;
        self.pan = Vec2::ZERO;
        self.settings_scroll = 0.0;
        self.fractional_ms = 0.0;
        self.last_active = false;
        self.message = "Your first reveal is always safe.".into();
    }

    fn open_modal(&mut self, pending: Pending) {
        self.backgammon_ui.settle_motion();
        self.holdem_ui.settle_motion();
        self.cribbage_ui.age = 1.0;
        self.cribbage_ui.hovered = None;
        self.cribbage_ui.feedback.clear();
        self.hearts_ui.motion.clear();
        self.patience_ui.drag = None;
        self.patience_ui.motion.clear();
        self.modal_return_focus = self.focus;
        self.modal = Some(pending);
        // Dialog hit order starts with Cancel, keeping destructive confirmation opt-in.
        self.focus = 0;
    }

    fn restore_modal_focus(&mut self) {
        self.modal = None;
        self.focus = self.modal_return_focus;
    }
}

fn fresh_seed() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
    #[cfg(target_arch = "wasm32")]
    {
        (web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(1.0)
            * 1_000_000.0) as u64
    }
}

fn main() {
    #[cfg(target_os = "macos")]
    let backend = Backends::METAL;
    #[cfg(target_arch = "wasm32")]
    let backend = Backends::BROWSER_WEBGPU;
    #[cfg(all(not(target_os = "macos"), not(target_arch = "wasm32")))]
    let backend = Backends::PRIMARY;
    App::new()
        .insert_resource(Arcade::load())
        .init_resource::<ViewLayout>()
        .init_resource::<PerformanceCapture>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "GameVex / Rust Arcade".into(),
                        resolution: WindowResolution::new(1280, 800),
                        resize_constraints: bevy::window::WindowResizeConstraints {
                            min_width: 360.0,
                            min_height: 560.0,
                            ..default()
                        },
                        canvas: Some("#gamevex".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(backend),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(ArcadeRenderPlugin)
        .init_resource::<SoundBank>()
        .add_systems(Startup, diagnostics)
        .add_systems(Update, capture_tools)
        .add_systems(
            Update,
            (
                lifecycle,
                handle_pointer_input,
                handle_input,
                presentation::paint,
                play_sound,
            )
                .chain(),
        )
        .run();
}

fn diagnostics(adapter: Option<Res<RenderAdapterInfo>>, mut state: ResMut<Arcade>) {
    #[cfg(target_arch = "wasm32")]
    browser_audio::install();
    if let Some(adapter) = adapter {
        state.backend = format!(
            "{:?} / {}",
            adapter.backend,
            if adapter.name.is_empty() {
                "adapter name unavailable"
            } else {
                &adapter.name
            }
        );
        info!("GameVex actual adapter: {}", state.backend);
    }
    platform::install_lifecycle_saves();
    platform::stage_snapshot(&state.save, !state.protected_save);
    if let Ok(json) = state.save.to_json() {
        platform::diagnostics_save(&json);
    }
    platform::ready();
}

fn lifecycle(
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
        let was_busy = state.baccarat_motion.busy();
        if state.save.settings.reduced_motion {
            state.baccarat_motion.clear();
        } else {
            state.baccarat_motion.tick(delta.min(0.1) as f32);
        }
        if was_busy && !state.baccarat_motion.busy() {
            state.focus = 3;
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
    if (state.last_active && !active)
        || transitioned
        || close.read().count() > 0
        || (active
            && ((state.screen == Screen::Mines
                && state.save.minesweeper.game.status == Status::Playing)
                || (state.screen == Screen::Sudoku && !state.save.sudoku.game.complete)
                || (state.screen == Screen::Mahjong && !state.save.mahjong.won())
                || (patience_app::is_family(state.screen) && !state.patience_game().won()))
            && state.save_age >= 5.0)
    {
        state.persist();
    }
    platform::stage_snapshot(&state.save, !state.protected_save);
    state.last_active = active;
    state.lifecycle = epoch;
    if delta > 0.0 {
        state.fps = state.fps * 0.95 + (1.0 / delta as f32) * 0.05;
    }
}

fn account_play_time(
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

/// Preserve the OS event order and each press/release position. ButtonInput's
/// per-frame sets can collapse fast clicks and pair them with a later cursor move.
fn handle_pointer_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut events: MessageReader<WindowEvent>,
    layout: Res<ViewLayout>,
    mut state: ResMut<Arcade>,
    mut pointer: Local<Option<Vec2>>,
    mut touch: Local<touch_input::TouchPointer>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let metrics = platform::canvas_metrics();
    let size = metrics
        .map(|[w, h, _]| Vec2::new(w, h))
        .unwrap_or(Vec2::new(window.width(), window.height()));
    let to_css = metrics
        .map(|[_, _, density]| window.scale_factor() / density)
        .unwrap_or(1.0);
    let mut layout_valid = layout.size == size;
    touch.sync_lifecycle(&mut state, &mut pointer);
    // Browser canvas metrics can change before winit emits a resize event.
    if !layout_valid {
        state.backgammon_ui.settle_motion();
        state.holdem_ui.settle_motion();
        state.cribbage_ui.age = 1.0;
        state.cribbage_ui.hovered = None;
        state.hearts_ui.motion.clear();
        state.lab.cancel_drag();
        state.patience_ui.drag = None;
        state.patience_ui.motion.clear();
        touch.reset(&mut state, &mut pointer);
    }
    if !window.focused || !platform::visible() {
        touch.reset(&mut state, &mut pointer);
    }
    let difficulty = state.save.minesweeper.game.difficulty;
    let sudoku_mode = (
        state.save.sudoku.game.size,
        state.save.sudoku.game.difficulty,
    );
    for event in events.read() {
        let input_valid = layout_valid
            && window.focused
            && platform::visible()
            && state.save.minesweeper.game.difficulty == difficulty
            && sudoku_mode
                == (
                    state.save.sudoku.game.size,
                    state.save.sudoku.game.difficulty,
                )
            && layout.surface == Some(state.screen)
            && layout.modal == state.modal.is_some();
        if let WindowEvent::TouchInput(event) = event {
            touch.handle(
                event,
                event.position * to_css,
                input_valid,
                &mut state,
                &layout,
                &mut pointer,
            );
            continue;
        }
        match event {
            WindowEvent::CursorMoved(event) if !touch.active() => {
                *pointer = Some(event.position * to_css)
            }
            WindowEvent::CursorLeft(_) if !touch.active() => {
                *pointer = None;
                state.lab.cancel_drag();
                state.patience_ui.drag = None;
                state.patience_ui.motion.clear();
            }
            WindowEvent::WindowResized(_) | WindowEvent::WindowScaleFactorChanged(_) => {
                state.lab.cancel_drag();
                state.patience_ui.drag = None;
                state.patience_ui.motion.clear();
                touch.reset(&mut state, &mut pointer);
                layout_valid = false;
            }
            WindowEvent::WindowFocused(event) if !event.focused => {
                *pointer = None;
                state.lab.cancel_drag();
                state.patience_ui.drag = None;
                state.patience_ui.motion.clear();
                touch.reset(&mut state, &mut pointer);
            }
            _ => {}
        }
        state.cursor = pointer.and_then(|point| {
            rendering::screen_to_surface(
                point,
                size,
                if state.save.settings.crt {
                    state.save.settings.curvature
                } else {
                    0.0
                },
            )
        });
        if state.cursor.is_none() {
            state.lab.cancel_drag();
            state.patience_ui.drag = None;
        }
        state.patience_drag_update();
        if !window.focused || !platform::visible() {
            continue;
        }
        if let WindowEvent::MouseButtonInput(event) = event {
            // A preceding event may have navigated or opened a dialog. Wait for
            // paint to build the new surface before accepting another action.
            if touch.active() || !input_valid {
                continue;
            }
            match event.state {
                ButtonState::Pressed => {
                    let card_press = event.button == MouseButton::Left
                        && patience_app::is_family(state.screen)
                        && state.modal.is_none()
                        && state.patience_press(&layout);
                    if card_press {
                        state.audio_started = true;
                        state.keyboard = false;
                    } else {
                        pointer_pressed(&mut state, &layout, event.button);
                    }
                }
                ButtonState::Released
                    if event.button == MouseButton::Left && state.patience_ui.drag.is_some() =>
                {
                    state.patience_drag_update();
                    state.patience_release(&layout);
                }
                ButtonState::Released
                    if event.button == MouseButton::Left && state.lab.drag.is_some() =>
                {
                    let slot = state
                        .cursor
                        .and_then(|p| layout.slots.iter().position(|r| r.contains(p)));
                    state.lab.drop_in(slot);
                }
                _ => {}
            }
        }
    }
    state.cursor = pointer.and_then(|point| {
        rendering::screen_to_surface(
            point,
            size,
            if state.save.settings.crt {
                state.save.settings.curvature
            } else {
                0.0
            },
        )
    });
}

fn pointer_pressed(state: &mut Arcade, layout: &ViewLayout, button: MouseButton) {
    if !matches!(button, MouseButton::Left | MouseButton::Right) {
        return;
    }
    state.audio_started = true;
    state.keyboard = false;
    if let Some((index, hit)) = layout
        .hits
        .iter()
        .enumerate()
        .rev()
        .find(|(_, hit)| state.cursor.is_some_and(|p| hit.rect.contains(p)))
    {
        state.focus = index;
        if let Action::Board = hit.action {
            if let Some(board) = &layout.board
                && let Some(i) = state.cursor.and_then(|p| {
                    board.index(
                        p,
                        state.save.minesweeper.game.cols(),
                        state.save.minesweeper.game.rows(),
                    )
                })
            {
                state.cell = i;
                let command = if button == MouseButton::Right || state.flag_mode {
                    GameCommand::ToggleFlag(i)
                } else if state.save.minesweeper.game.cells[i].revealed {
                    GameCommand::Chord(i)
                } else {
                    GameCommand::Reveal(i)
                };
                state.command(command);
            }
        } else if matches!(hit.action, Action::SudokuCell) {
            if button == MouseButton::Left
                && let Some(board) = &layout.sudoku
            {
                let size = usize::from(state.save.sudoku.game.size);
                if let Some(i) = state.cursor.and_then(|p| board.index(p, size, size)) {
                    state.sudoku_ui.selected = i;
                    state.sudoku_ui.follow = false;
                }
            }
        } else if button == MouseButton::Left {
            state.act(hit.action);
        }
    } else if state.screen == Screen::Cards
        && state.modal.is_none()
        && let Some(i) = layout.card_order.iter().rev().copied().find(|i| {
            state
                .cursor
                .is_some_and(|p| layout.card_hits[*i].contains(p))
        })
    {
        if button == MouseButton::Right {
            state.lab.select(i);
            state.lab.flip_selected();
        } else if state.lab.pick(i) {
            state.drag_offset = state.cursor.unwrap() - layout.cards[i].min;
        }
    }
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut wheel: MessageReader<MouseWheel>,
    layout: Res<ViewLayout>,
    mut state: ResMut<Arcade>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused || !platform::visible() {
        wheel.clear();
        return;
    }
    if layout.surface != Some(state.screen) || layout.modal != state.modal.is_some() {
        wheel.clear();
        return;
    }
    let gesture = keys.get_just_pressed().next().is_some();
    if gesture {
        state.audio_started = true;
    }
    if keys.get_just_pressed().next().is_some() {
        state.keyboard = true;
    }
    if [
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
    ]
    .iter()
    .any(|key| keys.pressed(*key))
    {
        wheel.clear();
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        if state.modal.is_some() {
            state.act(Action::Cancel);
        } else if state.screen == Screen::Backgammon && state.backgammon_ui.rules {
            state.backgammon_ui.rules = false;
        } else if state.screen == Screen::Roulette && state.roulette_ui.rules {
            state.roulette_ui.rules = false;
        } else if state.screen == Screen::Holdem && state.holdem_ui.rules {
            state.holdem_ui.rules = false;
        } else if state.screen == Screen::Cribbage && state.cribbage_ui.rules {
            state.cribbage_ui.rules = false;
        } else if state.screen == Screen::Hearts && state.hearts_ui.rules {
            state.hearts_ui.rules = false;
        } else if state.patience_ui.drag.is_some() {
            state.patience_ui.drag = None;
            state.patience_ui.motion.clear();
        } else if state.lab.drag.is_some() {
            state.lab.cancel_drag();
            state.patience_ui.drag = None;
            state.patience_ui.motion.clear();
        } else if state.screen != Screen::Lobby {
            state.act(Action::Back);
        }
        return;
    }
    if state.screen == Screen::Blackjack
        && state.modal.is_none()
        && ((state.blackjack_motion.busy() && !state.save.settings.reduced_motion)
            || (layout.surface == Some(Screen::Blackjack)
                && !layout
                    .hits
                    .iter()
                    .any(|h| matches!(h.action, Action::Blackjack(_)))))
    {
        wheel.clear();
        return;
    }
    if state.screen == Screen::Baccarat
        && state.modal.is_none()
        && ((state.baccarat_motion.busy() && !state.save.settings.reduced_motion)
            || (layout.surface == Some(Screen::Baccarat)
                && !layout
                    .hits
                    .iter()
                    .any(|h| matches!(h.action, Action::Baccarat(_)))))
    {
        wheel.clear();
        return;
    }
    let focus_count = layout.hits.len();
    if keys.just_pressed(KeyCode::Tab) && focus_count > 0 {
        state.keyboard = true;
        let backwards = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        state.focus = if backwards {
            (state.focus + focus_count - 1) % focus_count
        } else {
            (state.focus + 1) % focus_count
        };
    }
    state.focus = state.focus.min(focus_count.saturating_sub(1));
    let board_focused = layout
        .hits
        .get(state.focus)
        .is_some_and(|h| matches!(h.action, Action::Board));
    if state.modal.is_some() {
        if (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
            && let Some(hit) = layout.hits.get(state.focus)
        {
            state.act(hit.action);
        }
        wheel.clear();
        return;
    }
    if state.screen == Screen::Blackjack {
        use blackjack::Command as Bj;
        for (key, command) in [
            (KeyCode::KeyH, Bj::Hit),
            (KeyCode::KeyS, Bj::Stand),
            (KeyCode::KeyD, Bj::Double),
            (KeyCode::KeyP, Bj::Split),
            (KeyCode::KeyR, Bj::Surrender),
            (KeyCode::KeyY, Bj::Insure),
            (KeyCode::KeyN, Bj::DeclineInsurance),
        ] {
            if keys.just_pressed(key) {
                state.act(Action::Blackjack(command));
            }
        }
    }
    if state.screen == Screen::Baccarat {
        use baccarat::{BetSide, Command as Bc};
        for (key, command) in [
            (KeyCode::KeyP, Bc::Select(BetSide::Player)),
            (KeyCode::KeyB, Bc::Select(BetSide::Banker)),
            (KeyCode::KeyT, Bc::Select(BetSide::Tie)),
            (KeyCode::KeyD, Bc::Deal),
            (KeyCode::KeyN, Bc::NextRound),
            (KeyCode::KeyR, Bc::Restart),
        ] {
            if keys.just_pressed(key) {
                state.act(Action::Baccarat(command));
            }
        }
    }
    sudoku_app::board_keys(&mut state, &keys, &layout);
    mahjong_app::keys(&mut state, &keys, &layout);
    patience_app::keys(&mut state, &keys, &layout);
    if state.screen == Screen::Mines && board_focused {
        let cols = state.save.minesweeper.game.cols();
        let rows = state.save.minesweeper.game.rows();
        let mut row = state.cell / cols;
        let mut col = state.cell % cols;
        if keys.just_pressed(KeyCode::ArrowLeft) {
            col = col.saturating_sub(1);
            state.keyboard = true;
        }
        if keys.just_pressed(KeyCode::ArrowRight) {
            col = (col + 1).min(cols - 1);
            state.keyboard = true;
        }
        if keys.just_pressed(KeyCode::ArrowUp) {
            row = row.saturating_sub(1);
            state.keyboard = true;
        }
        if keys.just_pressed(KeyCode::ArrowDown) {
            row = (row + 1).min(rows - 1);
            state.keyboard = true;
        }
        state.cell = row * cols + col;
        if state.keyboard
            && let Some(board) = &layout.board
        {
            let point = board.origin + Vec2::new(col as f32, row as f32) * board.cell;
            if point.x < board.clip.min.x {
                state.pan.x -= board.clip.min.x - point.x;
            }
            if point.y < board.clip.min.y {
                state.pan.y -= board.clip.min.y - point.y;
            }
            if point.x + board.cell > board.clip.max.x {
                state.pan.x += point.x + board.cell - board.clip.max.x;
            }
            if point.y + board.cell > board.clip.max.y {
                state.pan.y += point.y + board.cell - board.clip.max.y;
            }
        }
        let i = state.cell;
        if keys.just_pressed(KeyCode::Space) {
            state.command(GameCommand::Reveal(i));
        }
        if keys.just_pressed(KeyCode::KeyF) {
            state.command(GameCommand::ToggleFlag(i));
        }
        if keys.just_pressed(KeyCode::Enter) {
            state.command(GameCommand::Chord(i));
        }
    } else if (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
        && let Some(hit) = layout.hits.get(state.focus)
    {
        state.act(hit.action);
    }
    if state.screen == Screen::Cards {
        if keys.just_pressed(KeyCode::ArrowRight) {
            state.lab.select_next();
        }
        if keys.just_pressed(KeyCode::ArrowLeft) {
            state.lab.select_previous();
        }
        if keys.just_pressed(KeyCode::KeyF) {
            state.lab.flip_selected();
        }
        for (key, slot) in [
            (KeyCode::Digit1, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
        ] {
            if keys.just_pressed(key)
                && let Some(i) = state.lab.selected
            {
                state.lab.pick(i);
                state.lab.drop_in(Some(slot));
            }
        }
        if keys.just_pressed(KeyCode::Backspace) {
            state.act(Action::ReturnCard);
        }
    }
    if state.screen == Screen::Settings {
        if keys.just_pressed(KeyCode::PageDown) {
            state.settings_scroll += 180.0;
        }
        if keys.just_pressed(KeyCode::PageUp) {
            state.settings_scroll -= 180.0;
        }
    }
    if state.screen == Screen::Lobby && state.modal.is_none() {
        let step = if keys.just_pressed(KeyCode::PageDown) {
            240.0
        } else if keys.just_pressed(KeyCode::PageUp) {
            -240.0
        } else {
            0.0
        };
        if step != 0.0 {
            state.keyboard = false;
            state.lobby_scroll = (state.lobby_scroll + step).clamp(0.0, layout.lobby_max_scroll);
        }
    }
    for event in wheel.read() {
        state.keyboard = false;
        if state.screen == Screen::Lobby && state.modal.is_none() {
            let scale = if event.unit == MouseScrollUnit::Line {
                36.0
            } else {
                1.0
            };
            state.lobby_scroll =
                (state.lobby_scroll - event.y * scale).clamp(0.0, layout.lobby_max_scroll);
        }
        if patience_app::is_family(state.screen)
            && state.modal.is_none()
            && layout
                .patience_clip
                .is_some_and(|r| state.cursor.is_some_and(|p| r.contains(p)))
        {
            let scale = if event.unit == MouseScrollUnit::Line {
                36.0
            } else {
                1.0
            };
            state.patience_ui.pan = (state.patience_ui.pan - Vec2::new(event.x, event.y) * scale)
                .clamp(Vec2::ZERO, layout.patience_max_pan);
        }
        if state.screen == Screen::Mahjong
            && state.modal.is_none()
            && layout
                .mahjong_clip
                .is_some_and(|r| state.cursor.is_some_and(|p| r.contains(p)))
        {
            let scale = if event.unit == MouseScrollUnit::Line {
                36.0
            } else {
                1.0
            };
            state.mahjong_ui.pan = (state.mahjong_ui.pan - Vec2::new(event.x, event.y) * scale)
                .clamp(Vec2::ZERO, layout.mahjong_max_pan);
        }
        if state.screen == Screen::Roulette
            && state.modal.is_none()
            && layout
                .roulette_clip
                .is_some_and(|r| state.cursor.is_some_and(|p| r.contains(p)))
        {
            let scale = if event.unit == MouseScrollUnit::Line {
                36.0
            } else {
                1.0
            };
            let delta = if event.x.abs() > event.y.abs() {
                event.x
            } else {
                event.y
            };
            state.roulette_ui.table_pan =
                (state.roulette_ui.table_pan - delta * scale).clamp(0.0, layout.roulette_max_pan);
        }
        if state.screen == Screen::Settings {
            let scale = if event.unit == MouseScrollUnit::Line {
                36.0
            } else {
                1.0
            };
            state.settings_scroll =
                (state.settings_scroll - event.y * scale).clamp(0.0, layout.settings_max_scroll);
        }
        if state.screen == Screen::Sudoku
            && state.modal.is_none()
            && layout
                .sudoku
                .as_ref()
                .is_some_and(|board| state.cursor.is_some_and(|p| board.clip.contains(p)))
        {
            let scale = if event.unit == MouseScrollUnit::Line {
                36.0
            } else {
                1.0
            };
            state.sudoku_ui.follow = false;
            if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                state.sudoku_ui.pan.x -= event.y * scale;
            } else {
                state.sudoku_ui.pan -= Vec2::new(event.x, event.y) * scale;
            }
            state.sudoku_ui.pan = state.sudoku_ui.pan.clamp(Vec2::ZERO, layout.sudoku_max_pan);
        }
        if state.screen == Screen::Mines
            && layout
                .board
                .as_ref()
                .is_some_and(|b| state.cursor.is_some_and(|p| b.clip.contains(p)))
        {
            let scale = if event.unit == MouseScrollUnit::Line {
                36.0
            } else {
                1.0
            };
            if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                state.pan.x -= event.y * scale;
            } else {
                state.pan -= Vec2::new(event.x, event.y) * scale;
            }
        }
    }
}

#[derive(Resource)]
struct SoundBank {
    casino: std::collections::HashMap<casino_audio::SoundCue, Handle<AudioSource>>,
    legacy: std::collections::HashMap<u32, Handle<AudioSource>>,
    played: u64,
    last: String,
}
const CASINO_GAIN: f32 = 0.5;
const MAX_AUDIO_VOICES: usize = 4;
impl Default for SoundBank {
    fn default() -> Self {
        Self {
            casino: std::collections::HashMap::with_capacity(casino_audio::ALL.len()),
            legacy: Default::default(),
            played: 0,
            last: String::new(),
        }
    }
}
#[derive(Component)]
struct SoundVoice {
    cue: Option<casino_audio::SoundCue>,
    gain: f32,
}

fn cue_priority(cue: Option<casino_audio::SoundCue>) -> u8 {
    use casino_audio::SoundCue::*;
    match cue {
        Some(Win | Natural | Push | Loss | GameOver | TableWin) => 3,
        Some(Bust | Reveal | Double | Split | Surrender | Insurance | TableRun) => 2,
        Some(Card | TablePick | TilePick) => 0,
        _ => 1,
    }
}

fn play_sound(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    mut state: ResMut<Arcade>,
    mut bank: ResMut<SoundBank>,
    mut voices: Query<(Entity, &SoundVoice, Option<&mut AudioSink>)>,
) {
    let cue_screen = matches!(
        state.screen,
        Screen::Blackjack
            | Screen::Baccarat
            | Screen::Mahjong
            | Screen::Hearts
            | Screen::Cribbage
            | Screen::Holdem
            | Screen::Roulette
            | Screen::Backgammon
    ) || patience_app::is_family(state.screen);
    let enabled =
        state.audio_started && !state.save.settings.muted && state.save.settings.volume > 0.0;
    #[cfg(target_arch = "wasm32")]
    browser_audio::update(state.save.settings.volume, !enabled, cue_screen);
    let mut active = Vec::new();
    for (entity, voice, sink) in &mut voices {
        if !enabled || (voice.cue.is_some() && !cue_screen) {
            commands.entity(entity).despawn();
        } else {
            if let Some(mut sink) = sink {
                sink.set_volume(Volume::Linear(state.save.settings.volume * voice.gain));
            }
            active.push((entity, cue_priority(voice.cue)));
        }
    }
    let mut requests: Vec<_> = state
        .casino_feedback
        .take_sounds()
        .into_iter()
        .map(|cue| (Some(cue), 0.0))
        .collect();
    let table_sounds = if state.screen == Screen::Backgammon {
        state.backgammon_ui.feedback.take_sounds()
    } else if state.screen == Screen::Roulette {
        state.roulette_ui.feedback.take_sounds()
    } else if state.screen == Screen::Holdem {
        state.holdem_ui.feedback.take_sounds()
    } else if state.screen == Screen::Cribbage {
        state.cribbage_ui.feedback.take_sounds()
    } else if state.screen == Screen::Hearts {
        state.hearts_ui.feedback.take_sounds()
    } else if state.screen == Screen::Mahjong {
        state.mahjong_ui.feedback.take_sounds()
    } else if patience_app::is_family(state.screen) {
        state.patience_ui.feedback.take_sounds()
    } else {
        Vec::new()
    };
    requests.extend(table_sounds.into_iter().map(|cue| (Some(cue), 0.0)));
    if let Some(frequency) = state.sound.take() {
        requests.push((None, frequency));
    }
    requests.sort_by_key(|(cue, _)| std::cmp::Reverse(cue_priority(*cue)));
    if enabled {
        for (cue, frequency) in requests {
            #[cfg(not(target_arch = "wasm32"))]
            if active.len() >= MAX_AUDIO_VOICES {
                let (i, (_, lowest)) = active
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, (_, p))| p)
                    .unwrap();
                if *lowest >= cue_priority(cue) {
                    continue;
                }
                commands.entity(active.remove(i).0).despawn();
            }
            let source = if let Some(cue) = cue {
                bank.casino
                    .entry(cue)
                    .or_insert_with(|| {
                        sources.add(AudioSource {
                            bytes: casino_audio::synthesize(cue).into(),
                        })
                    })
                    .clone()
            } else {
                bank.legacy
                    .entry(frequency.to_bits())
                    .or_insert_with(|| {
                        sources.add(AudioSource {
                            bytes: tone(frequency).into(),
                        })
                    })
                    .clone()
            };
            let gain = if cue.is_some() { CASINO_GAIN } else { 0.12 };
            #[cfg(target_arch = "wasm32")]
            if !browser_audio::play(
                cue.map_or(frequency.to_bits(), |c| c as u32),
                &sources.get(&source).unwrap().bytes,
                gain,
                cue_priority(cue),
                cue.is_some(),
            ) {
                continue;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let entity = commands
                    .spawn((
                        AudioPlayer::new(source),
                        PlaybackSettings {
                            volume: Volume::Linear(state.save.settings.volume * gain),
                            ..PlaybackSettings::DESPAWN
                        },
                        SoundVoice { cue, gain },
                    ))
                    .id();
                active.push((entity, cue_priority(cue)));
            }
            bank.played += 1;
            bank.last = cue.map_or_else(|| "Ui".to_owned(), |cue| format!("{cue:?}"));
        }
    }
    if platform::diagnostics_enabled() {
        #[cfg(target_arch = "wasm32")]
        let (voice_count, cached, running, backend) = {
            let status = browser_audio::status();
            (status.voices, status.cached, status.running, "WebAudio")
        };
        #[cfg(not(target_arch = "wasm32"))]
        let (voice_count, cached, running, backend) =
            (active.len(), bank.casino.len(), enabled, "BevyAudio");
        platform::diagnostics_attribute(
            "data-audio",
            &serde_json::json!({
                "played": bank.played, "last": bank.last, "cached": cached,
                "voices": voice_count, "running": running, "backend": backend, "muted": state.save.settings.muted,
                "volume": state.save.settings.volume, "gesture": state.audio_started,
            })
            .to_string(),
        );
    }
}

fn tone(frequency: f32) -> Vec<u8> {
    let rate = 22050u32;
    let count = 1800u32;
    let bytes = count * 2;
    let mut wav = Vec::with_capacity(bytes as usize + 44);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + bytes).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&rate.to_le_bytes());
    wav.extend_from_slice(&(rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&bytes.to_le_bytes());
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let envelope = (1.0 - i as f32 / count as f32).powi(2);
        let sample =
            ((t * frequency * std::f32::consts::TAU).sin() * envelope * i16::MAX as f32) as i16;
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn test_arcade() -> Arcade {
        Arcade {
            save: Save::new(1),
            screen: Screen::Lobby,
            settings_return: Screen::Lobby,
            lab: CardLab::default(),
            modal: None,
            focus: 0,
            modal_return_focus: 0,
            settings_return_focus: 0,
            keyboard: false,
            cell: 0,
            flag_mode: false,
            pan: Vec2::ZERO,
            settings_scroll: 0.0,
            lobby_scroll: 0.0,
            cursor: None,
            card_positions: [Vec2::ZERO; 5],
            drag_offset: Vec2::ZERO,
            motion_time: 0.0,
            blackjack_motion: Default::default(),
            baccarat_motion: Default::default(),
            casino_feedback: Default::default(),
            sudoku_ui: Default::default(),
            mahjong_ui: Default::default(),
            patience_ui: Default::default(),
            hearts_ui: Default::default(),
            cribbage_ui: Default::default(),
            holdem_ui: Default::default(),
            backgammon_ui: Default::default(),
            roulette_ui: Default::default(),
            message: String::new(),
            persistence: String::new(),
            protected_save: true,
            backend: String::new(),
            fps: 0.0,
            last_active: false,
            lifecycle: 0,
            fractional_ms: 0.0,
            save_age: 0.0,
            sound: None,
            audio_started: false,
        }
    }

    fn pointer_app(screen: Screen) -> (App, Entity) {
        let mut app = App::new();
        let mut state = test_arcade();
        state.screen = screen;
        state.save.settings.crt = false;
        app.insert_resource(state)
            .insert_resource(ViewLayout {
                surface: Some(screen),
                size: Vec2::new(1280.0, 800.0),
                ..default()
            })
            .add_message::<WindowEvent>()
            .add_systems(Update, handle_pointer_input);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: WindowResolution::new(1280, 800),
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        (app, window)
    }

    fn move_pointer(app: &mut App, window: Entity, x: f32, y: f32) {
        app.world_mut()
            .write_message(WindowEvent::CursorMoved(bevy::window::CursorMoved {
                window,
                position: Vec2::new(x, y),
                delta: None,
            }));
    }
    fn mouse_event(app: &mut App, window: Entity, button: MouseButton, state: ButtonState) {
        app.world_mut().write_message(WindowEvent::MouseButtonInput(
            bevy::input::mouse::MouseButtonInput {
                window,
                button,
                state,
            },
        ));
    }

    fn touch_event(
        app: &mut App,
        window: Entity,
        id: u64,
        phase: bevy::input::touch::TouchPhase,
        x: f32,
        y: f32,
    ) {
        app.world_mut()
            .write_message(WindowEvent::TouchInput(bevy::input::touch::TouchInput {
                window,
                id,
                phase,
                position: Vec2::new(x, y),
                force: None,
            }));
    }

    #[test]
    fn touch_buttons_activate_once_on_release_and_cancelled_or_moved_taps_do_nothing() {
        use bevy::input::touch::TouchPhase::*;
        let (mut app, window) = pointer_app(Screen::Mines);
        app.world_mut()
            .resource_mut::<ViewLayout>()
            .hits
            .push(presentation::Hit {
                rect: Rect::from_corners(Vec2::ZERO, Vec2::splat(100.0)),
                action: Action::FlagMode,
            });
        touch_event(&mut app, window, 1, Started, 30.0, 30.0);
        app.update();
        assert!(!app.world().resource::<Arcade>().flag_mode);
        touch_event(&mut app, window, 1, Ended, 30.0, 30.0);
        app.update();
        assert!(app.world().resource::<Arcade>().flag_mode);
        assert!(app.world().resource::<Arcade>().cursor.is_none());
        assert!(app.world().resource::<Arcade>().audio_started);
        for phase in [Canceled, Ended] {
            touch_event(&mut app, window, 2, Started, 30.0, 30.0);
            if phase == Ended {
                // Moving away and back must not become a tap on release.
                touch_event(&mut app, window, 2, Moved, 60.0, 30.0);
            }
            touch_event(&mut app, window, 2, phase, 30.0, 30.0);
            app.update();
            assert!(app.world().resource::<Arcade>().flag_mode);
        }
    }

    #[test]
    fn touch_card_drag_uses_release_position_and_cancel_never_commits() {
        use bevy::input::touch::TouchPhase::*;
        for end in [Ended, Canceled] {
            let (mut app, window) = pointer_app(Screen::Cards);
            let mut layout = app.world_mut().resource_mut::<ViewLayout>();
            layout.cards[0] = Rect::from_corners(Vec2::new(10.0, 400.0), Vec2::new(110.0, 540.0));
            layout.card_hits = layout.cards;
            layout.card_order = vec![0];
            layout.slots[0] = Rect::from_corners(Vec2::new(200.0, 100.0), Vec2::new(300.0, 240.0));
            touch_event(&mut app, window, 1, Started, 30.0, 420.0);
            touch_event(&mut app, window, 1, Moved, 150.0, 300.0);
            touch_event(&mut app, window, 1, end, 230.0, 150.0);
            app.update();
            let state = app.world().resource::<Arcade>();
            assert_eq!(
                state.lab.slots,
                if end == Ended {
                    [Some(0), None, None]
                } else {
                    [None; 3]
                }
            );
            assert!(state.lab.drag.is_none());
            assert!(state.cursor.is_none());
        }
    }

    #[test]
    fn extra_fingers_and_mouse_events_cannot_steal_or_duplicate_a_touch() {
        use bevy::input::touch::TouchPhase::*;
        let (mut app, window) = pointer_app(Screen::Mines);
        app.world_mut()
            .resource_mut::<ViewLayout>()
            .hits
            .push(presentation::Hit {
                rect: Rect::from_corners(Vec2::ZERO, Vec2::splat(100.0)),
                action: Action::FlagMode,
            });
        touch_event(&mut app, window, 1, Started, 30.0, 30.0);
        touch_event(&mut app, window, 2, Started, 40.0, 40.0);
        move_pointer(&mut app, window, 50.0, 50.0);
        mouse_event(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        app.update();
        assert!(!app.world().resource::<Arcade>().flag_mode);
        assert!(
            app.world()
                .resource::<Arcade>()
                .cursor
                .unwrap()
                .distance(Vec2::splat(30.0))
                < 0.001
        );
        touch_event(&mut app, window, 1, Ended, 30.0, 30.0);
        touch_event(&mut app, window, 2, Moved, 30.0, 30.0);
        touch_event(&mut app, window, 2, Ended, 30.0, 30.0);
        app.update();
        assert!(app.world().resource::<Arcade>().flag_mode);
        touch_event(&mut app, window, 3, Started, 30.0, 30.0);
        touch_event(&mut app, window, 3, Ended, 30.0, 30.0);
        app.update();
        assert!(!app.world().resource::<Arcade>().flag_mode);
        // The owner has ended, the remaining finger disappears while hidden,
        // and no End event arrives. Lifecycle recovery must clear that contact.
        touch_event(&mut app, window, 4, Started, 30.0, 30.0);
        touch_event(&mut app, window, 5, Started, 40.0, 40.0);
        touch_event(&mut app, window, 4, Ended, 30.0, 30.0);
        app.update();
        assert!(app.world().resource::<Arcade>().flag_mode);
        app.world_mut().resource_mut::<Arcade>().lifecycle += 1;
        touch_event(&mut app, window, 6, Started, 30.0, 30.0);
        touch_event(&mut app, window, 6, Ended, 30.0, 30.0);
        app.update();
        assert!(!app.world().resource::<Arcade>().flag_mode);
    }

    #[test]
    fn touch_swipes_pan_both_boards_without_revealing_or_selecting_a_cell() {
        use bevy::input::touch::TouchPhase::*;
        for screen in [Screen::Mines, Screen::Sudoku] {
            let (mut app, window) = pointer_app(screen);
            let before = app.world().resource::<Arcade>().save.clone();
            let mut layout = app.world_mut().resource_mut::<ViewLayout>();
            let clip = Rect::from_corners(Vec2::ZERO, Vec2::splat(288.0));
            let board = presentation::BoardLayout {
                origin: Vec2::ZERO,
                clip,
                cell: 32.0,
            };
            let action = if screen == Screen::Mines {
                layout.board = Some(board);
                Action::Board
            } else {
                layout.sudoku = Some(board);
                layout.sudoku_max_pan = Vec2::splat(64.0);
                Action::SudokuCell
            };
            layout.hits.push(presentation::Hit { rect: clip, action });
            touch_event(&mut app, window, 1, Started, 160.0, 160.0);
            touch_event(&mut app, window, 1, Moved, 80.0, 120.0);
            touch_event(&mut app, window, 1, Ended, 60.0, 100.0);
            app.update();
            let state = app.world().resource::<Arcade>();
            assert_eq!(state.save, before);
            if screen == Screen::Mines {
                assert!(state.pan.distance(Vec2::new(100.0, 60.0)) < 0.001);
            } else {
                assert!(state.sudoku_ui.pan.distance(Vec2::new(64.0, 60.0)) < 0.001);
                assert!(!state.sudoku_ui.follow);
            }
        }
    }

    #[test]
    fn touch_settings_swipe_scrolls_without_changing_the_setting_under_the_finger() {
        use bevy::input::touch::TouchPhase::*;
        let (mut app, window) = pointer_app(Screen::Settings);
        let before = app.world().resource::<Arcade>().save.settings.clone();
        let mut layout = app.world_mut().resource_mut::<ViewLayout>();
        layout.settings_clip = Some(Rect::from_corners(Vec2::ZERO, Vec2::splat(400.0)));
        layout.settings_max_scroll = 300.0;
        layout.hits.push(presentation::Hit {
            rect: Rect::from_corners(Vec2::ZERO, Vec2::splat(400.0)),
            action: Action::Palette,
        });
        touch_event(&mut app, window, 1, Started, 200.0, 300.0);
        touch_event(&mut app, window, 1, Moved, 200.0, 150.0);
        touch_event(&mut app, window, 1, Ended, 200.0, 100.0);
        app.update();
        let state = app.world().resource::<Arcade>();
        assert_eq!(state.settings_scroll, 200.0);
        assert_eq!(state.save.settings, before);
    }

    #[test]
    fn resize_focus_loss_and_lifecycle_changes_cancel_pending_touch_actions() {
        use bevy::input::touch::TouchPhase::*;
        for interruption in 0..3 {
            let (mut app, window) = pointer_app(Screen::Mines);
            app.world_mut()
                .resource_mut::<ViewLayout>()
                .hits
                .push(presentation::Hit {
                    rect: Rect::from_corners(Vec2::ZERO, Vec2::splat(100.0)),
                    action: Action::FlagMode,
                });
            touch_event(&mut app, window, 1, Started, 30.0, 30.0);
            app.update();
            match interruption {
                0 => {
                    app.world_mut().write_message(WindowEvent::WindowResized(
                        bevy::window::WindowResized {
                            window,
                            width: 1200.0,
                            height: 800.0,
                        },
                    ));
                }
                1 => {
                    app.world_mut()
                        .write_message(WindowEvent::WindowFocused(WindowFocused {
                            window,
                            focused: false,
                        }));
                }
                _ => {
                    app.world_mut().resource_mut::<Arcade>().lifecycle += 1;
                }
            }
            touch_event(&mut app, window, 1, Ended, 30.0, 30.0);
            app.update();
            assert!(!app.world().resource::<Arcade>().flag_mode);
            // A second interruption can lose the end event entirely. A fresh
            // contact must still work when the page resumes.
            touch_event(&mut app, window, 2, Started, 30.0, 30.0);
            app.update();
            if interruption == 0 {
                app.world_mut().write_message(WindowEvent::WindowResized(
                    bevy::window::WindowResized {
                        window,
                        width: 1200.0,
                        height: 800.0,
                    },
                ));
                app.update();
            } else if interruption == 1 {
                app.world_mut()
                    .write_message(WindowEvent::WindowFocused(WindowFocused {
                        window,
                        focused: false,
                    }));
                app.update();
            } else {
                app.world_mut().resource_mut::<Arcade>().lifecycle += 1;
            }
            touch_event(&mut app, window, 3, Started, 30.0, 30.0);
            touch_event(&mut app, window, 3, Ended, 30.0, 30.0);
            app.update();
            assert!(app.world().resource::<Arcade>().flag_mode);
        }
    }

    #[test]
    fn queued_clicks_keep_their_own_cells_in_one_update() {
        let (mut app, window) = pointer_app(Screen::Mines);
        let mut layout = app.world_mut().resource_mut::<ViewLayout>();
        let clip = Rect::from_corners(Vec2::ZERO, Vec2::splat(288.0));
        layout.board = Some(presentation::BoardLayout {
            origin: Vec2::ZERO,
            clip,
            cell: 32.0,
        });
        layout.hits.push(presentation::Hit {
            rect: clip,
            action: Action::Board,
        });
        for x in [16.0, 80.0] {
            move_pointer(&mut app, window, x, 16.0);
            mouse_event(&mut app, window, MouseButton::Right, ButtonState::Pressed);
            mouse_event(&mut app, window, MouseButton::Right, ButtonState::Released);
        }
        // A later move must not redirect either queued click.
        move_pointer(&mut app, window, 200.0, 200.0);
        app.update();
        let state = app.world().resource::<Arcade>();
        assert!(state.save.minesweeper.game.cells[0].flagged);
        assert!(state.save.minesweeper.game.cells[2].flagged);
        assert_eq!(
            state
                .save
                .minesweeper
                .game
                .cells
                .iter()
                .filter(|c| c.flagged)
                .count(),
            2
        );
    }

    #[test]
    fn queued_drag_and_release_commit_without_a_held_frame() {
        let (mut app, window) = pointer_app(Screen::Cards);
        let mut layout = app.world_mut().resource_mut::<ViewLayout>();
        layout.cards[0] = Rect::from_corners(Vec2::new(10.0, 400.0), Vec2::new(110.0, 540.0));
        layout.card_hits = layout.cards;
        layout.card_order = vec![0];
        layout.slots[0] = Rect::from_corners(Vec2::new(200.0, 100.0), Vec2::new(300.0, 240.0));
        move_pointer(&mut app, window, 30.0, 420.0);
        mouse_event(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        move_pointer(&mut app, window, 230.0, 150.0);
        mouse_event(&mut app, window, MouseButton::Left, ButtonState::Released);
        app.update();
        let state = app.world().resource::<Arcade>();
        assert_eq!(state.lab.slots, [Some(0), None, None]);
        assert_eq!(state.lab.drag, None);
    }

    #[test]
    fn resize_during_drag_cancels_instead_of_dropping_at_stale_slots() {
        let (mut app, window) = pointer_app(Screen::Cards);
        app.world_mut().resource_mut::<Arcade>().lab.pick(0);
        app.world_mut().resource_mut::<ViewLayout>().slots[0] =
            Rect::from_corners(Vec2::ZERO, Vec2::splat(100.0));
        move_pointer(&mut app, window, 30.0, 30.0);
        app.world_mut()
            .write_message(WindowEvent::WindowResized(bevy::window::WindowResized {
                window,
                width: 900.0,
                height: 700.0,
            }));
        mouse_event(&mut app, window, MouseButton::Left, ButtonState::Released);
        app.update();
        let state = app.world().resource::<Arcade>();
        assert_eq!(state.lab.drag, None);
        assert_eq!(state.lab.slots, [None; 3]);
    }

    #[test]
    fn secondary_click_flips_the_topmost_card_once() {
        let (mut app, window) = pointer_app(Screen::Cards);
        let mut layout = app.world_mut().resource_mut::<ViewLayout>();
        layout.cards[0] = Rect::from_corners(Vec2::splat(10.0), Vec2::splat(110.0));
        layout.cards[1] = layout.cards[0];
        layout.card_hits = layout.cards;
        layout.card_order = vec![1, 0];
        move_pointer(&mut app, window, 30.0, 30.0);
        mouse_event(&mut app, window, MouseButton::Right, ButtonState::Pressed);
        mouse_event(&mut app, window, MouseButton::Right, ButtonState::Released);
        app.update();
        let state = app.world().resource::<Arcade>();
        assert!(!state.lab.cards[0].face_up);
        assert!(state.lab.cards[1].face_up);
        assert_eq!(state.lab.drag, None);
    }

    #[test]
    fn navigation_does_not_reuse_the_previous_screens_hits() {
        let (mut app, window) = pointer_app(Screen::Lobby);
        app.world_mut()
            .resource_mut::<ViewLayout>()
            .hits
            .push(presentation::Hit {
                rect: Rect::from_corners(Vec2::ZERO, Vec2::splat(100.0)),
                action: Action::Open(Screen::Cards),
            });
        for _ in 0..2 {
            move_pointer(&mut app, window, 30.0, 30.0);
            mouse_event(&mut app, window, MouseButton::Left, ButtonState::Pressed);
            mouse_event(&mut app, window, MouseButton::Left, ButtonState::Released);
        }
        app.update();
        let state = app.world().resource::<Arcade>();
        assert_eq!(state.screen, Screen::Cards);
        assert_eq!(state.settings_return, Screen::Lobby);
        assert_eq!(state.lab.selected, None);
    }

    #[test]
    fn casino_mix_has_headroom_at_full_volume_with_all_voices_active() {
        let mut peak = 0.0_f32;
        for cue in casino_audio::ALL {
            let wav = casino_audio::synthesize(cue);
            for bytes in wav[44..].as_chunks::<2>().0 {
                peak = peak.max(i16::from_le_bytes(*bytes).unsigned_abs() as f32 / 32768.0);
            }
        }
        assert!(MAX_AUDIO_VOICES as f32 * (peak * CASINO_GAIN).max(0.12) < 0.95);
    }

    #[test]
    fn casino_audio_requires_gesture_caches_samples_caps_voices_and_mutes_immediately() {
        let mut app = App::new();
        let mut state = test_arcade();
        state.screen = Screen::Baccarat;
        app.insert_resource(state)
            .insert_resource(Assets::<AudioSource>::default())
            .init_resource::<SoundBank>()
            .add_systems(Update, play_sound);
        app.world_mut()
            .resource_mut::<Arcade>()
            .act(Action::Baccarat(baccarat::Command::Select(
                baccarat::BetSide::Banker,
            )));
        app.update();
        assert_eq!(app.world().resource::<SoundBank>().played, 0);
        app.world_mut().resource_mut::<Arcade>().audio_started = true;
        app.update();
        assert_eq!(app.world().resource::<SoundBank>().played, 0);
        for i in 0..20 {
            let side = if i % 2 == 0 {
                baccarat::BetSide::Player
            } else {
                baccarat::BetSide::Banker
            };
            app.world_mut()
                .resource_mut::<Arcade>()
                .act(Action::Baccarat(baccarat::Command::Select(side)));
            app.update();
        }
        assert_eq!(
            app.world_mut()
                .query::<&SoundVoice>()
                .iter(app.world())
                .count(),
            4
        );
        assert_eq!(app.world().resource::<SoundBank>().casino.len(), 1);
        assert_eq!(app.world().resource::<Assets<AudioSource>>().len(), 1);
        let played = app.world().resource::<SoundBank>().played;
        app.world_mut().resource_mut::<Arcade>().save.settings.muted = true;
        app.world_mut()
            .resource_mut::<Arcade>()
            .act(Action::Baccarat(baccarat::Command::Select(
                baccarat::BetSide::Tie,
            )));
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&SoundVoice>()
                .iter(app.world())
                .count(),
            0
        );
        assert_eq!(app.world().resource::<SoundBank>().played, played);
        {
            let mut state = app.world_mut().resource_mut::<Arcade>();
            state.save.settings.muted = false;
            state.save.settings.volume = 0.0;
            state.act(Action::Baccarat(baccarat::Command::Select(
                baccarat::BetSide::Player,
            )));
        }
        app.update();
        assert_eq!(app.world().resource::<SoundBank>().played, played);
        assert_eq!(
            app.world_mut()
                .query::<&SoundVoice>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn table_audio_obeys_gesture_mute_volume_and_does_not_replay_dropped_cues() {
        for screen in [Screen::FreeCell, Screen::Mahjong] {
            let mut app = App::new();
            let mut state = test_arcade();
            state.screen = screen;
            app.insert_resource(state)
                .insert_resource(Assets::<AudioSource>::default())
                .init_resource::<SoundBank>()
                .add_systems(Update, play_sound);
            let emit = |app: &mut App, kind| {
                let mut s = app.world_mut().resource_mut::<Arcade>();
                if screen == Screen::Mahjong {
                    s.mahjong_ui
                        .feedback
                        .emit(kind, &[table_feedback::Anchor::Tile(0)]);
                } else {
                    s.patience_ui.feedback.emit(
                        kind,
                        &[table_feedback::Anchor::Pile(patience::Pile::Cell(0))],
                    );
                }
            };
            emit(&mut app, table_feedback::EffectKind::Move);
            app.update();
            assert_eq!(app.world().resource::<SoundBank>().played, 0);
            app.world_mut().resource_mut::<Arcade>().audio_started = true;
            app.update();
            assert_eq!(app.world().resource::<SoundBank>().played, 0);
            emit(&mut app, table_feedback::EffectKind::Move);
            app.update();
            assert_eq!(app.world().resource::<SoundBank>().played, 1);
            app.world_mut().resource_mut::<Arcade>().save.settings.muted = true;
            emit(&mut app, table_feedback::EffectKind::Win);
            app.update();
            assert_eq!(app.world().resource::<SoundBank>().played, 1);
            assert_eq!(
                app.world_mut()
                    .query::<&SoundVoice>()
                    .iter(app.world())
                    .count(),
                0
            );
            app.world_mut().resource_mut::<Arcade>().save.settings.muted = false;
            app.update();
            assert_eq!(app.world().resource::<SoundBank>().played, 1);
            app.world_mut()
                .resource_mut::<Arcade>()
                .save
                .settings
                .volume = 0.0;
            emit(&mut app, table_feedback::EffectKind::Hint);
            app.update();
            assert_eq!(app.world().resource::<SoundBank>().played, 1);
        }
    }

    #[test]
    fn finishing_blackjack_animation_keeps_focus_until_controls_are_rebuilt() {
        let (mut app, _) = pointer_app(Screen::Blackjack);
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .add_message::<MouseWheel>()
            .add_systems(Update, handle_input);
        app.world_mut().resource_mut::<Arcade>().focus = 7;
        // The previous animation frame contained only the three header hits.
        app.world_mut().resource_mut::<ViewLayout>().hits = (0..3)
            .map(|_| presentation::Hit {
                rect: Rect::default(),
                action: Action::Open(Screen::Settings),
            })
            .collect();
        app.update();
        assert_eq!(app.world().resource::<Arcade>().focus, 7);
        let mut layout = app.world_mut().resource_mut::<ViewLayout>();
        for _ in 3..8 {
            layout.hits.push(presentation::Hit {
                rect: Rect::default(),
                action: Action::Blackjack(blackjack::Command::Deal),
            });
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();
        let state = app.world().resource::<Arcade>();
        assert_eq!(state.screen, Screen::Blackjack);
        assert_ne!(state.save.blackjack.phase, blackjack::Phase::Betting);
    }

    fn sudoku_test_state() -> Arcade {
        let mut state = test_arcade();
        state.save.sudoku.game = sudoku::Sudoku::new(4, sudoku::Difficulty::Easy, 5);
        state.open(Screen::Sudoku);
        state.sudoku_ui.selected = state
            .save
            .sudoku
            .game
            .board
            .iter()
            .position(|v| *v == 0)
            .unwrap();
        state
    }

    #[test]
    fn canvas_metric_resize_cancels_drag_without_a_window_resize_event() {
        let (mut app, window) = pointer_app(Screen::Cards);
        app.world_mut().resource_mut::<Arcade>().lab.pick(0);
        app.world_mut()
            .entity_mut(window)
            .get_mut::<Window>()
            .unwrap()
            .resolution
            .set(900.0, 700.0);
        app.update();
        let state = app.world().resource::<Arcade>();
        assert!(state.lab.drag.is_none());
        assert_eq!(state.lab.slots, [None; 3]);
    }

    #[test]
    fn sudoku_notes_undo_assistance_and_navigation_preserve_only_domain_state() {
        let mut state = sudoku_test_state();
        let i = state.sudoku_ui.selected;
        let original = state.save.sudoku.game.clone();
        state.act(Action::SudokuPencil);
        state.act(Action::Sudoku(sudoku::Command::Enter {
            index: 0,
            value: 1,
            pencil: false,
        }));
        assert_eq!(state.save.sudoku.game.notes[i], 1);
        assert_eq!(state.sudoku_ui.history.len(), 1);
        state.open(Screen::Settings);
        state.act(Action::Back);
        assert_eq!(state.sudoku_ui.history.len(), 1);
        state.act(Action::Sudoku(sudoku::Command::Undo));
        assert_eq!(state.save.sudoku.game, original);
        state.act(Action::Sudoku(sudoku::Command::Hint));
        assert!(state.save.sudoku.game.assisted);
        state.act(Action::Sudoku(sudoku::Command::Undo));
        assert!(state.save.sudoku.game.assisted);
        assert_eq!(state.save.sudoku.game.board, original.board);
        state.act(Action::Sudoku(sudoku::Command::Hint));
        let saved = state.save.sudoku.clone();
        state.open(Screen::Lobby);
        assert!(state.sudoku_ui.history.is_empty());
        state.open(Screen::Sudoku);
        assert_eq!(state.save.sudoku, saved);
        assert!(state.sudoku_ui.completion_age.is_none());
        assert_eq!(
            Save::from_json(&state.save.to_json().unwrap()).unwrap(),
            state.save
        );
    }

    #[test]
    fn sudoku_mouse_edit_actions_return_keyboard_focus_and_undo_reveals_the_edited_cell() {
        let mut state = sudoku_test_state();
        let original = state.sudoku_ui.selected;
        state.focus = 9;
        state.act(Action::SudokuPencil);
        assert_eq!(state.focus, 3);
        state.focus = 9;
        state.act(Action::Sudoku(sudoku::Command::Enter {
            index: original,
            value: 2,
            pencil: true,
        }));
        assert_eq!(state.focus, 3);
        state.sudoku_ui.selected = (original + 1) % state.save.sudoku.game.board.len();
        state.focus = 10;
        state.act(Action::Sudoku(sudoku::Command::Undo));
        assert_eq!(state.focus, 3);
        assert_eq!(state.sudoku_ui.selected, original);
        assert_eq!(state.save.sudoku.game.notes[original], 0);
        assert!(state.sudoku_ui.follow);
    }

    #[test]
    fn sudoku_setup_stages_choices_without_touching_the_live_save() {
        let mut state = sudoku_test_state();
        state.act(Action::Sudoku(sudoku::Command::Hint));
        state.focus = 7;
        let before = state.save.clone();
        let selected = state.sudoku_ui.selected;
        let undo = state.sudoku_ui.history.len();
        state.act(Action::SudokuSetup);
        assert!(matches!(state.modal, Some(Pending::SudokuSetup)));
        state.act(Action::SudokuChooseSize(16));
        state.act(Action::SudokuChooseDifficulty(sudoku::Difficulty::Expert));
        assert_eq!(
            state.sudoku_ui.setup,
            Some((16, sudoku::Difficulty::Expert))
        );
        assert_eq!(state.save, before);
        state.act(Action::SudokuChooseSize(7));
        assert_eq!(
            state.sudoku_ui.setup,
            Some((16, sudoku::Difficulty::Expert))
        );
        state.act(Action::Cancel);
        assert!(state.modal.is_none());
        assert!(state.sudoku_ui.setup.is_none());
        assert_eq!(state.focus, 7);
        assert_eq!(state.sudoku_ui.selected, selected);
        assert_eq!(state.sudoku_ui.history.len(), undo);
        assert_eq!(state.save, before);
        state.act(Action::SudokuSetup);
        state.act(Action::SudokuChooseSize(6));
        state.act(Action::SudokuChooseDifficulty(sudoku::Difficulty::Hard));
        state.act(Action::SudokuStart);
        assert!(matches!(
            state.modal,
            Some(Pending::NewSudoku(6, sudoku::Difficulty::Hard))
        ));
        assert_eq!(state.save, before);
        state.act(Action::Cancel);
        assert_eq!(state.focus, 7);
        assert_eq!(state.save, before);
        state.act(Action::SudokuSetup);
        state.act(Action::SudokuChooseSize(6));
        state.act(Action::SudokuChooseDifficulty(sudoku::Difficulty::Hard));
        state.act(Action::SudokuStart);
        state.act(Action::Confirm);
        assert_eq!(state.save.sudoku.game.size, 6);
        assert_eq!(state.save.sudoku.game.difficulty, sudoku::Difficulty::Hard);
        assert!(!state.save.sudoku.game.assisted);
        assert_eq!(state.save.sudoku.game.givens[state.sudoku_ui.selected], 0);
        assert_eq!(state.save.minesweeper, before.minesweeper);
        assert_eq!(state.save.blackjack, before.blackjack);
        assert_eq!(state.save.baccarat, before.baccarat);
    }

    #[test]
    fn sudoku_setup_keyboard_activates_the_focused_option_and_keeps_cancel_safe() {
        let (mut app, _) = pointer_app(Screen::Sudoku);
        app.insert_resource(ButtonInput::<KeyCode>::default())
            .add_message::<MouseWheel>()
            .add_systems(Update, handle_input);
        app.world_mut()
            .resource_mut::<Arcade>()
            .act(Action::SudokuSetup);
        {
            let mut layout = app.world_mut().resource_mut::<ViewLayout>();
            layout.modal = true;
            layout.hits = [
                Action::Cancel,
                Action::SudokuChooseSize(16),
                Action::SudokuChooseDifficulty(sudoku::Difficulty::Expert),
                Action::SudokuStart,
            ]
            .into_iter()
            .map(|action| presentation::Hit {
                rect: Rect::default(),
                action,
            })
            .collect();
        }
        app.world_mut().resource_mut::<Arcade>().focus = 1;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();
        assert_eq!(app.world().resource::<Arcade>().save.sudoku.game.size, 9);
        assert_eq!(
            app.world().resource::<Arcade>().sudoku_ui.setup.unwrap().0,
            16
        );
        assert!(matches!(
            app.world().resource::<Arcade>().modal,
            Some(Pending::SudokuSetup)
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut().resource_mut::<Arcade>().focus = 0;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();
        assert!(app.world().resource::<Arcade>().modal.is_none());
        assert_eq!(app.world().resource::<Arcade>().save.sudoku.game.size, 9);
    }

    #[test]
    fn replacing_sudoku_requires_confirmation_and_preserves_other_games() {
        let mut state = sudoku_test_state();
        state.act(Action::Sudoku(sudoku::Command::Hint));
        let before = state.save.clone();
        state.request_sudoku(16, sudoku::Difficulty::Expert);
        assert!(matches!(state.modal, Some(Pending::NewSudoku(16, _))));
        state.act(Action::Cancel);
        assert_eq!(state.save, before);
        state.request_sudoku(16, sudoku::Difficulty::Expert);
        state.act(Action::Confirm);
        assert!(state.modal.is_none());
        assert_eq!(state.save.sudoku.game.size, 16);
        assert!(!state.save.sudoku.game.assisted);
        assert_eq!(state.save.sudoku.game.elapsed_ms, 0);
        assert!(state.sudoku_ui.history.is_empty());
        assert_eq!(state.save.blackjack, before.blackjack);
        assert_eq!(state.save.baccarat, before.baccarat);
        assert_eq!(state.save.minesweeper, before.minesweeper);
        state.reset_saved_data();
        assert_eq!(state.save.sudoku.game.size, 9);
        assert!(!state.save.sudoku.game.assisted);
    }

    #[test]
    fn sudoku_keyboard_values_are_scoped_and_hex_letters_remain_values() {
        let mut state = sudoku_test_state();
        let mut layout = ViewLayout {
            hits: [
                Action::SudokuSetup,
                Action::SudokuSetup,
                Action::SudokuSetup,
                Action::SudokuCell,
            ]
            .into_iter()
            .map(|action| presentation::Hit {
                rect: Rect::default(),
                action,
            })
            .collect(),
            ..default()
        };
        state.focus = 3;
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Digit2);
        sudoku_app::board_keys(&mut state, &keys, &layout);
        let i = state.sudoku_ui.selected;
        assert_eq!(state.save.sudoku.game.board[i], 2);
        layout.hits[3].action = Action::SudokuPencil;
        keys.reset_all();
        keys.press(KeyCode::Digit3);
        sudoku_app::board_keys(&mut state, &keys, &layout);
        assert_eq!(state.save.sudoku.game.board[i], 2);
        state.save.sudoku.game = sudoku::Sudoku::new(16, sudoku::Difficulty::Easy, 2);
        state.sudoku_ui.selected = state
            .save
            .sudoku
            .game
            .board
            .iter()
            .position(|v| *v == 0)
            .unwrap();
        layout.hits[3].action = Action::SudokuCell;
        keys.reset_all();
        keys.press(KeyCode::KeyA);
        sudoku_app::board_keys(&mut state, &keys, &layout);
        assert_eq!(state.save.sudoku.game.board[state.sudoku_ui.selected], 10);
        keys.reset_all();
        keys.press(KeyCode::KeyG);
        sudoku_app::board_keys(&mut state, &keys, &layout);
        assert_eq!(state.save.sudoku.game.board[state.sudoku_ui.selected], 16);
        state.sudoku_ui.selected = 0;
        keys.reset_all();
        keys.press(KeyCode::ArrowLeft);
        sudoku_app::board_keys(&mut state, &keys, &layout);
        assert_eq!(state.sudoku_ui.selected, 0);
    }

    #[test]
    fn sudoku_clock_pauses_in_settings_and_does_not_advance_minesweeper() {
        use std::time::Duration;
        let mut app = App::new();
        app.insert_resource(sudoku_test_state())
            .insert_resource(Time::<Real>::default())
            .add_message::<WindowFocused>()
            .add_message::<WindowCloseRequested>()
            .add_systems(Update, lifecycle);
        app.world_mut().spawn((
            Window {
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ));
        let tick = |app: &mut App, ms| {
            app.world_mut()
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_millis(ms));
            app.update();
        };
        tick(&mut app, 100);
        tick(&mut app, 125);
        assert_eq!(
            app.world().resource::<Arcade>().save.sudoku.game.elapsed_ms,
            125
        );
        app.world_mut()
            .resource_mut::<Arcade>()
            .open(Screen::Settings);
        tick(&mut app, 200);
        tick(&mut app, 200);
        assert_eq!(
            app.world().resource::<Arcade>().save.sudoku.game.elapsed_ms,
            125
        );
        app.world_mut().resource_mut::<Arcade>().act(Action::Back);
        tick(&mut app, 200);
        tick(&mut app, 125);
        let state = app.world().resource::<Arcade>();
        assert_eq!(state.save.sudoku.game.elapsed_ms, 250);
        assert_eq!(state.save.minesweeper.game.elapsed_ms, 0);
    }

    #[test]
    fn sudoku_controls_fit_supported_layouts_and_selection_can_pan_into_view() {
        for (width, height) in [(1280, 720), (390, 844), (1000, 560), (390, 560)] {
            for size in sudoku::SIZES {
                for show_highlights in [false, true] {
                    let mut app = App::new();
                    let mut state = sudoku_test_state();
                    state.save.sudoku.game = sudoku::Sudoku::new(size, sudoku::Difficulty::Easy, 3);
                    state.sudoku_ui.selected = usize::from(size).pow(2) - 1;
                    state.sudoku_ui.follow = true;
                    state.sudoku_ui.show_highlights = show_highlights;
                    app.insert_resource(state)
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
                    let layout = app.world().resource::<ViewLayout>();
                    for hit in &layout.hits {
                        assert!(
                            hit.rect.min.x >= 0.0
                                && hit.rect.min.y >= 0.0
                                && hit.rect.max.x <= width as f32 + 0.1
                                && hit.rect.max.y <= height as f32 - 33.0,
                            "{width}x{height} size{size}: {:?}",
                            hit.rect
                        );
                    }
                    let board = layout.sudoku.as_ref().unwrap();
                    assert!(board.cell >= 32.0);
                    let n = usize::from(size);
                    let p = board.origin + Vec2::splat((n as f32 - 0.5) * board.cell);
                    assert!(
                        board.clip.contains(p),
                        "selected last cell must be reachable"
                    );
                    assert_eq!(board.index(p, n, n), Some(n * n - 1));
                    for (i, hit) in layout.hits.iter().enumerate() {
                        for other in &layout.hits[i + 1..] {
                            let overlap = hit.rect.intersect(other.rect);
                            assert!(
                                overlap.width() <= 0.0 || overlap.height() <= 0.0,
                                "overlapping Sudoku controls at {width}x{height} size{size}: {:?} {:?}",
                                hit.rect,
                                other.rect
                            );
                        }
                    }
                    app.world_mut()
                        .resource_mut::<Arcade>()
                        .act(Action::SudokuSetup);
                    app.update();
                    let layout = app.world().resource::<ViewLayout>();
                    assert!(matches!(
                        layout.hits.first().unwrap().action,
                        Action::Cancel
                    ));
                    assert!(matches!(
                        layout.hits.last().unwrap().action,
                        Action::SudokuStart
                    ));
                    for hit in &layout.hits {
                        assert!(
                            hit.rect.min.x >= 0.0
                                && hit.rect.min.y >= 0.0
                                && hit.rect.max.x <= width as f32 + 0.1
                                && hit.rect.max.y <= height as f32
                                && hit.rect.height() >= 32.0,
                            "setup control out of bounds {width}x{height}: {:?}",
                            hit.rect
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn baccarat_settles_before_animation_and_settings_preserve_the_round() {
        use baccarat::{BetSide, Command as Bc, Phase};
        let mut state = test_arcade();
        state.open(Screen::Baccarat);
        state.act(Action::Baccarat(Bc::Select(BetSide::Banker)));
        state.act(Action::Baccarat(Bc::Deal));
        assert_eq!(state.save.baccarat.phase, Phase::Settled);
        assert!(state.baccarat_motion.busy());
        let round = state.save.baccarat.clone();
        state.act(Action::Baccarat(Bc::NextRound));
        assert_eq!(state.save.baccarat, round);
        state.open(Screen::Settings);
        state.act(Action::Back);
        assert_eq!(state.screen, Screen::Baccarat);
        assert_eq!(state.save.baccarat, round);
        state.open(Screen::Lobby);
        assert!(state.modal.is_none());
        assert!(!state.baccarat_motion.busy());
        state.open(Screen::Baccarat);
        assert_eq!(state.save.baccarat, round);
        assert_eq!(
            Save::from_json(&state.save.to_json().unwrap()).unwrap(),
            state.save
        );
    }

    #[test]
    fn blackjack_leave_requires_confirmation_and_settings_preserve_the_hand() {
        let mut state = test_arcade();
        state.save.blackjack = blackjack::Blackjack::with_shoe(
            1,
            gamevex_core::cards::Shoe::from_cards(
                [10, 7, 10, 8]
                    .into_iter()
                    .map(|rank| {
                        gamevex_core::cards::Card::new(
                            rank,
                            gamevex_core::cards::Suit::Spades,
                            true,
                        )
                    })
                    .collect(),
            ),
        );
        state.open(Screen::Blackjack);
        state.act(Action::Blackjack(blackjack::Command::Deal));
        assert!(state.save.blackjack.legal(blackjack::Command::Abandon));
        {
            let round = state.save.blackjack.clone();
            assert!(state.blackjack_motion.busy());
            state.act(Action::Blackjack(blackjack::Command::Hit));
            assert_eq!(state.save.blackjack, round);
            state.open(Screen::Settings);
            state.act(Action::Back);
            assert_eq!(state.screen, Screen::Blackjack);
            assert_eq!(state.save.blackjack, round);
            state.open(Screen::Lobby);
            assert!(matches!(
                state.modal,
                Some(Pending::LeaveBlackjack(Screen::Lobby))
            ));
            state.act(Action::Cancel);
            assert_eq!(state.save.blackjack, round);
            state.open(Screen::Lobby);
            state.act(Action::Confirm);
            assert_eq!(state.screen, Screen::Lobby);
            assert_eq!(state.save.blackjack.balance, round.balance);
            assert!(!state.save.blackjack.legal(blackjack::Command::Abandon));
        }
    }

    #[test]
    fn modal_cancel_restores_focus_and_confirm_starts_new_game_on_board() {
        let mut state = test_arcade();
        state.screen = Screen::Mines;
        state.focus = 9;
        state.save.minesweeper.game.command(GameCommand::Reveal(40));
        let original_game = state.save.minesweeper.game.clone();

        state.act(Action::New(Difficulty::Intermediate, false));
        assert!(matches!(state.modal, Some(Pending::New(_, _))));
        assert_eq!(state.focus, 0); // Cancel is the first modal control.
        state.act(Action::Cancel);
        assert!(state.modal.is_none());
        assert_eq!(state.focus, 9);
        assert_eq!(state.save.minesweeper.game, original_game);

        state.act(Action::New(Difficulty::Intermediate, false));
        state.act(Action::Confirm);
        assert!(state.modal.is_none());
        assert_eq!(
            state.save.minesweeper.game.difficulty,
            Difficulty::Intermediate
        );
        assert_eq!(state.save.minesweeper.game.status, Status::Ready);
        assert_eq!(state.focus, 9);
    }

    #[test]
    fn returning_from_settings_restores_the_previous_screen_focus() {
        let mut state = test_arcade();
        state.screen = Screen::Mines;
        state.focus = 9;
        state.act(Action::Open(Screen::Settings));
        assert_eq!(state.settings_return, Screen::Mines);
        assert_eq!(state.settings_return_focus, 9);
        assert_eq!(state.screen, Screen::Settings);

        state.act(Action::Back);
        assert_eq!(state.screen, Screen::Mines);
        assert_eq!(state.focus, 9);
    }

    #[test]
    fn saved_data_reset_clears_game_records_settings_and_transient_state() {
        let mut state = test_arcade();
        state.save.settings.color_scheme = ColorScheme::Amber;
        state.save.minesweeper.records.beginner = Some(5000);
        state.save.minesweeper.game.command(GameCommand::Reveal(40));
        state.save.blackjack.command(blackjack::Command::Deal);
        state
            .save
            .baccarat
            .command(baccarat::Command::Select(baccarat::BetSide::Tie));
        state.save.baccarat.command(baccarat::Command::Deal);
        state.lab.pick(0);
        state.lab.drop_in(Some(0));
        state.flag_mode = true;
        state.reset_saved_data();
        assert_eq!(state.save.settings, Settings::default());
        assert_eq!(state.save.minesweeper.records, Records::default());
        assert_eq!(state.save.minesweeper.game.status, Status::Ready);
        assert_eq!(state.save.blackjack.balance, 1000);
        assert_eq!(state.save.blackjack.phase, blackjack::Phase::Betting);
        assert_eq!(state.save.baccarat.balance, 1000);
        assert_eq!(state.save.baccarat.phase, baccarat::Phase::Betting);
        assert_eq!(state.screen, Screen::Lobby);
        assert_eq!(state.lab, CardLab::default());
        assert!(!state.flag_mode);
        assert!(state.save.validate().is_ok());
    }

    #[test]
    fn play_clock_skips_transitions_pauses_and_long_frames_with_fractional_ms() {
        let mut fractional = 0.0;
        assert_eq!(
            account_play_time(0.25, true, true, false, &mut fractional),
            250
        );
        assert_eq!(
            account_play_time(0.0006, true, true, false, &mut fractional),
            0
        );
        assert_eq!(
            account_play_time(0.0006, true, true, false, &mut fractional),
            1
        );
        assert!((fractional - 0.2).abs() < 1e-9);

        assert_eq!(
            account_play_time(0.2, false, true, false, &mut fractional),
            0
        );
        assert_eq!(account_play_time(0.2, true, true, true, &mut fractional), 0);
        assert_eq!(
            account_play_time(0.5, true, true, false, &mut fractional),
            0
        );
        assert_eq!(
            account_play_time(f64::NAN, true, true, false, &mut fractional),
            0
        );
        assert!((fractional - 0.2).abs() < 1e-9);
    }

    #[test]
    fn mahjong_replacement_cancel_restart_and_reset_preserve_other_cabinets() {
        let mut state = test_arcade();
        state.open(Screen::Mahjong);
        let initial = state.save.mahjong.clone();
        let [a, b] = initial.available_pairs()[0];
        state.mahjong_tile(a);
        state.mahjong_tile(b);
        assert_eq!(state.save.mahjong.remaining(), 142);
        let before = state.save.clone();
        state.request_mahjong(mahjong::Layout::Arena, 88);
        assert!(matches!(state.modal, Some(Pending::NewMahjong(..))));
        state.act(Action::Cancel);
        assert_eq!(state.save, before);
        state.act(Action::MahjongRestart);
        state.act(Action::Confirm);
        assert_eq!(state.save.mahjong, initial);
        assert_eq!(state.save.sudoku, before.sudoku);
        assert_eq!(state.save.blackjack, before.blackjack);
        assert_eq!(state.save.baccarat, before.baccarat);
        assert_eq!(state.save.minesweeper, before.minesweeper);
        state.save.mahjong_records[0] = Some(1000);
        state.mahjong_hint();
        state.reset_saved_data();
        assert_eq!(state.save.mahjong.remaining(), 144);
        assert!(!state.save.mahjong.assisted);
        assert_eq!(state.save.mahjong_records, [None; 3]);
        assert!(state.mahjong_ui.hint.is_none());
    }

    #[test]
    fn mahjong_completion_records_only_unassisted_runs_and_undo_keeps_assistance() {
        for (assisted, zen) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut state = test_arcade();
            state.open(Screen::Mahjong);
            let game = &mut state.save.mahjong;
            game.zen = zen;
            game.elapsed_ms = if zen { 0 } else { 12_345 };
            let solution = mahjong::solution_pairs(game.layout, game.seed);
            if assisted {
                state.mahjong_hint();
            }
            for [a, b] in solution {
                state.mahjong_tile(a);
                state.mahjong_tile(b);
            }
            assert!(state.save.mahjong.won());
            assert_eq!(
                state.save.mahjong_records[0],
                (!assisted && !zen).then_some(12_345)
            );
            state.act(Action::MahjongUndo);
            assert_eq!(state.save.mahjong.remaining(), 2);
            assert_eq!(state.save.mahjong.assisted, assisted);
            assert!(state.save.validate().is_ok());
        }
    }

    #[test]
    fn mahjong_modes_stage_replacement_preserve_other_games_and_survive_restart() {
        let mut state = test_arcade();
        state.open(Screen::Mahjong);
        state.act(Action::MahjongZen(true));
        assert!(state.modal.is_none());
        assert!(state.save.mahjong.zen);
        assert_eq!(state.save.mahjong.elapsed_ms, 0);
        let [a, b] = state.save.mahjong.available_pairs()[0];
        state.mahjong_tile(a);
        state.mahjong_tile(b);
        let before = state.save.clone();
        state.act(Action::MahjongZen(false));
        assert!(matches!(
            state.modal,
            Some(Pending::NewMahjong(_, _, false))
        ));
        assert_eq!(state.save, before);
        state.act(Action::Cancel);
        assert_eq!(state.save, before);
        state.act(Action::MahjongRestart);
        state.act(Action::Confirm);
        assert!(state.save.mahjong.zen);
        assert_eq!(state.save.mahjong.seed, before.mahjong.seed);
        assert_eq!(state.save.mahjong.remaining(), 144);
        state.act(Action::MahjongNew(mahjong::Layout::Monument));
        assert!(state.save.mahjong.zen);
        assert_eq!(state.save.mahjong.layout, mahjong::Layout::Monument);
        state.act(Action::MahjongZen(false));
        assert!(!state.save.mahjong.zen);
        assert_eq!(state.save.sudoku, before.sudoku);
        assert_eq!(state.save.blackjack, before.blackjack);
        assert_eq!(state.save.baccarat, before.baccarat);
        assert_eq!(state.save.minesweeper, before.minesweeper);
        assert_eq!(
            Save::from_json(&state.save.to_json().unwrap()).unwrap(),
            state.save
        );
    }

    #[test]
    fn mahjong_rejects_blocked_tiles_and_scopes_keyboard_shortcuts() {
        let mut state = test_arcade();
        state.open(Screen::Mahjong);
        let blocked = (0..144)
            .find(|i| !state.save.mahjong.selectable(*i))
            .unwrap();
        let before = state.save.clone();
        state.mahjong_tile(blocked);
        assert_eq!(state.save, before);
        assert!(state.mahjong_ui.selected.is_none());
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyH);
        state.open(Screen::Lobby);
        mahjong_app::keys(&mut state, &keys, &ViewLayout::default());
        assert!(!state.save.mahjong.assisted);
        state.open(Screen::Mahjong);
        state.open_modal(Pending::NewMahjong(mahjong::Layout::Arena, 2, false));
        mahjong_app::keys(&mut state, &keys, &ViewLayout::default());
        assert!(!state.save.mahjong.assisted);
        state.act(Action::Cancel);
        mahjong_app::keys(&mut state, &keys, &ViewLayout::default());
        assert!(state.save.mahjong.assisted);
        assert!(state.mahjong_ui.reveal_hint);
    }

    #[test]
    fn mahjong_clock_pauses_in_settings_and_dialogs_without_advancing_other_games() {
        use std::time::Duration;
        let mut state = test_arcade();
        state.open(Screen::Mahjong);
        let mut app = App::new();
        app.insert_resource(state)
            .insert_resource(Time::<Real>::default())
            .add_message::<WindowFocused>()
            .add_message::<WindowCloseRequested>()
            .add_systems(Update, lifecycle);
        app.world_mut().spawn((
            Window {
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ));
        let tick = |app: &mut App| {
            app.world_mut()
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_millis(125));
            app.update();
        };
        tick(&mut app);
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Arcade>().save.mahjong.elapsed_ms,
            125
        );
        app.world_mut()
            .resource_mut::<Arcade>()
            .open(Screen::Settings);
        tick(&mut app);
        tick(&mut app);
        app.world_mut().resource_mut::<Arcade>().act(Action::Back);
        tick(&mut app);
        tick(&mut app);
        app.world_mut()
            .resource_mut::<Arcade>()
            .open_modal(Pending::NewMahjong(mahjong::Layout::Arena, 2, false));
        tick(&mut app);
        tick(&mut app);
        let state = app.world().resource::<Arcade>();
        assert_eq!(state.save.mahjong.elapsed_ms, 250);
        assert_eq!(state.save.minesweeper.game.elapsed_ms, 0);
        assert_eq!(state.save.sudoku.game.elapsed_ms, 0);
    }

    #[test]
    fn mahjong_controls_and_clipped_tile_hits_fit_all_supported_layouts() {
        for (width, height) in [(1280, 720), (390, 844), (1000, 560), (390, 560), (360, 560)] {
            for board in mahjong::Layout::ALL {
                let mut state = test_arcade();
                state.open(Screen::Mahjong);
                state.save.mahjong = mahjong::Game::new(board, 3);
                state.mahjong_hint();
                let first = state.mahjong_ui.hint.unwrap()[0];
                let mut app = App::new();
                app.insert_resource(state)
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
                let layout = app.world().resource::<ViewLayout>();
                if width >= 1280 {
                    assert!(
                        layout.mahjong_max_pan.max_element() < 0.001,
                        "desktop board should fit"
                    );
                }
                for hit in &layout.hits {
                    assert!(
                        hit.rect.min.x >= 0.0
                            && hit.rect.min.y >= 0.0
                            && hit.rect.max.x <= width as f32 + 0.1
                            && hit.rect.max.y <= height as f32 - 33.0,
                        "{width}x{height} {board:?}: {:?}",
                        hit.rect
                    );
                    if matches!(hit.action, Action::MahjongTile(_)) {
                        let clip = layout.mahjong_clip.unwrap();
                        assert!(clip.contains(hit.rect.center()));
                    }
                }
                assert!(
                    layout
                        .hits
                        .iter()
                        .any(|h| matches!(h.action, Action::MahjongTile(i) if i == first)),
                    "hinted first tile reachable"
                );
                assert!(!app.world().resource::<Arcade>().mahjong_ui.reveal_hint);
            }
        }
    }

    #[test]
    fn patience_family_isolation_cancel_restart_and_reset() {
        let mut state = test_arcade();
        state.open(Screen::Solitaire);
        state.act(Action::PatienceDeal);
        let before = state.save.clone();
        state.act(Action::PatienceNew(patience::Variant::Klondike3, false));
        assert!(matches!(state.modal, Some(Pending::NewPatience(..))));
        state.act(Action::Cancel);
        assert_eq!(state.save, before);
        state.open(Screen::FreeCell);
        let hint = state.patience_game_mut().hint().unwrap();
        state.patience_pick(hint.0, hint.1);
        state.patience_target(hint.2);
        assert!(state.save.freecell.moves > 0);
        assert_eq!(state.save.solitaire, before.solitaire);
        assert_eq!(state.save.spider, before.spider);
        let moves = state.save.freecell.moves;
        state.open(Screen::Settings);
        state.act(Action::Back);
        assert_eq!(state.save.freecell.moves, moves);
        state.reset_saved_data();
        assert_eq!(state.save.freecell.moves, 0);
        assert!(state.patience_ui.selected.is_none());
        assert!(state.save.validate().is_ok());
    }

    #[test]
    fn patience_stock_deal_feedback_is_scoped_to_affected_piles() {
        use table_feedback::{Anchor, EffectKind};

        let mut state = test_arcade();
        state.open(Screen::Solitaire);
        state.act(Action::PatienceDeal);
        let event = state.patience_ui.feedback.events.last().unwrap();
        assert_eq!(event.kind, EffectKind::Deal);
        assert_eq!(event.anchors, vec![Anchor::Pile(patience::Pile::Stock)]);
        assert_eq!(
            state.patience_ui.feedback.take_sounds(),
            vec![casino_audio::SoundCue::TableDeal]
        );

        state.open(Screen::Spider);
        state.save.spider = patience::Game::new(patience::Variant::Spider4, 44);
        state.act(Action::PatienceDeal);
        let event = state.patience_ui.feedback.events.last().unwrap();
        assert_eq!(event.kind, EffectKind::Deal);
        assert_eq!(
            event.anchors,
            (0..10)
                .map(|i| Anchor::Pile(patience::Pile::Tableau(i)))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            state.patience_ui.feedback.take_sounds(),
            vec![casino_audio::SoundCue::TableDeal]
        );
    }
    #[test]
    fn patience_tables_fit_and_all_foundations_are_reachable() {
        for (width, height) in [(1280, 720), (390, 844), (360, 560)] {
            for variant in patience::Variant::ALL {
                let mut state = test_arcade();
                state.screen = match variant {
                    patience::Variant::FreeCell | patience::Variant::FreeCellRelaxed => {
                        Screen::FreeCell
                    }
                    patience::Variant::Spider1
                    | patience::Variant::Spider2
                    | patience::Variant::Spider4 => Screen::Spider,
                    _ => Screen::Solitaire,
                };
                *state.patience_game_mut() = patience::Game::new(variant, 42);
                let mut app = App::new();
                app.insert_resource(state)
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
                assert!(l.patience_clip.is_some());
                for hit in &l.hits {
                    assert!(
                        hit.rect.min.x >= 0.0
                            && hit.rect.min.y >= 0.0
                            && hit.rect.max.x <= width as f32 + 0.1
                            && hit.rect.max.y <= height as f32 - 33.0,
                        "{variant:?} {width}x{height} {:?}",
                        hit.rect
                    );
                }
                if width >= 1280 && variant == patience::Variant::FreeCell {
                    for i in 0..4 {
                        assert!(l.hits.iter().any(|h|matches!(h.action,Action::PatienceTarget(patience::Pile::Foundation(j)) if i==j)));
                    }
                }
            }
        }
    }
    #[test]
    fn finish_commits_before_animation_undo_restores_one_action_and_relaxed_records_stay_separate()
    {
        use cards::{Card, Suit};
        use patience::{Pile, Variant};
        let mut s = test_arcade();
        s.screen = Screen::FreeCell;
        let mut game = patience::Game::new(Variant::FreeCellRelaxed, 77);
        game.tableau = vec![vec![]; 8];
        game.foundations = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds]
            .into_iter()
            .enumerate()
            .map(|(i, suit)| {
                std::iter::once(14)
                    .chain(2..=if i == 3 { 11 } else { 13 })
                    .map(|rank| Card::new(rank, suit, true))
                    .collect()
            })
            .collect();
        game.tableau[0] = vec![
            Card::new(13, Suit::Diamonds, true),
            Card::new(12, Suit::Diamonds, true),
        ];
        game.moves = 1;
        assert!(game.validate().is_ok());
        s.save.freecell = game.clone();
        s.save.patience_records[2] = Some(9_000);
        s.open(Screen::FreeCell);
        assert!(s.patience_ui.can_finish);
        s.act(Action::PatienceFinish);
        assert!(s.patience_game().won());
        assert!(s.patience_ui.motion.busy());
        assert!(!s.patience_ui.can_finish);
        assert_eq!(s.save.patience_records[2], Some(9_000));
        assert_eq!(s.save.patience_records[6], Some(1000));
        assert!(
            Save::from_json(&s.save.to_json().unwrap())
                .unwrap()
                .freecell
                .won()
        );
        s.act(Action::PatienceUndo);
        assert_eq!(s.patience_game(), &game);
        assert!(s.patience_ui.can_finish);
        s.save.settings.reduced_motion = true;
        s.act(Action::PatienceFinish);
        assert!(s.patience_game().won());
        assert!(!s.patience_ui.motion.busy());
        s.open(Screen::Lobby);
        s.open(Screen::FreeCell);
        assert!(!s.patience_ui.motion.busy());
        assert_eq!(s.patience_game().cards(Pile::Foundation(3)).len(), 13);
    }

    #[test]
    fn patience_win_is_transient_and_undo_does_not_replay_it() {
        use cards::{Card, Suit};
        use patience::Pile;
        use table_feedback::EffectKind;
        let mut s = test_arcade();
        s.screen = Screen::Solitaire;
        let game = &mut s.save.solitaire;
        game.stock.clear();
        game.waste.clear();
        game.tableau = vec![vec![]; 7];
        game.foundations = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds]
            .into_iter()
            .map(|suit| {
                std::iter::once(14)
                    .chain(2..=13)
                    .map(|rank| Card::new(rank, suit, true))
                    .collect()
            })
            .collect();
        game.tableau[0].push(game.foundations[3].pop().unwrap());
        assert!(s.save.validate().is_ok());
        s.patience_pick(Pile::Tableau(0), 0);
        s.patience_target(Pile::Foundation(3));
        assert!(s.patience_game().won());
        assert!(s.save.validate().is_ok());
        assert_eq!(s.patience_ui.feedback.events.len(), 1);
        assert_eq!(s.patience_ui.feedback.events[0].kind, EffectKind::Win);
        assert_eq!(
            s.patience_ui.feedback.take_sounds(),
            vec![casino_audio::SoundCue::TableWin]
        );
        s.act(Action::PatienceUndo);
        assert!(!s.patience_game().won());
        assert!(
            s.patience_ui
                .feedback
                .events
                .iter()
                .all(|e| e.kind != EffectKind::Win)
        );
        s.open(Screen::Lobby);
        s.open(Screen::Solitaire);
        assert!(s.patience_ui.feedback.events.is_empty());
        assert!(s.patience_ui.feedback.take_sounds().is_empty());
    }
    #[test]
    fn mahjong_final_pair_emits_win_and_navigation_clears_feedback() {
        use table_feedback::EffectKind;
        let mut s = test_arcade();
        s.screen = Screen::Mahjong;
        let [a, b] = s.save.mahjong.available_pairs()[0];
        for (i, tile) in s.save.mahjong.tiles.iter_mut().enumerate() {
            tile.removed = i != a && i != b;
        }
        s.mahjong_tile(a);
        s.mahjong_tile(b);
        assert!(s.save.mahjong.won());
        assert!(
            s.mahjong_ui
                .feedback
                .events
                .iter()
                .any(|e| e.kind == EffectKind::Win)
        );
        assert!(
            s.mahjong_ui
                .feedback
                .take_sounds()
                .contains(&casino_audio::SoundCue::TableWin)
        );
        let saved = s.save.mahjong.clone();
        s.open(Screen::Lobby);
        s.open(Screen::Mahjong);
        assert_eq!(s.save.mahjong, saved);
        assert!(s.mahjong_ui.feedback.events.is_empty());
        assert!(s.mahjong_ui.feedback.take_sounds().is_empty());
    }

    #[test]
    fn lobby_scroll_preserves_all_games_and_reveals_keyboard_focus() {
        for (width, height) in [(1280, 720), (390, 740), (360, 560)] {
            let mut app = App::new();
            app.insert_resource(test_arcade())
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
            let layout = app.world().resource::<ViewLayout>();
            let games: Vec<_> = layout
                .hits
                .iter()
                .enumerate()
                .filter(
                    |(_, h)| matches!(h.action, Action::Open(screen) if !matches!(screen, Screen::Settings | Screen::Lobby)),
                )
                .collect();
            assert_eq!(games.len(), 14);
            let first = games[0].0;
            let last = games.last().unwrap().0;
            assert!(matches!(
                games.last().unwrap().1.action,
                Action::Open(Screen::Backgammon)
            ));
            assert_eq!(games.last().unwrap().1.rect.height(), 0.0);
            {
                let mut state = app.world_mut().resource_mut::<Arcade>();
                state.keyboard = true;
                state.focus = last;
            }
            app.update();
            let layout = app.world().resource::<ViewLayout>();
            let clip = layout.lobby_clip.unwrap();
            assert_eq!(layout.hits[last].rect.height(), 76.0);
            for hit in &layout.hits[first..] {
                if hit.rect.height() > 0.0 {
                    assert!(hit.rect.min.y >= clip.min.y && hit.rect.max.y <= clip.max.y);
                }
            }
            assert!(app.world().resource::<Arcade>().lobby_scroll > 0.0);
            app.world_mut().resource_mut::<Arcade>().focus = first;
            app.update();
            assert_eq!(app.world().resource::<Arcade>().lobby_scroll, 0.0);
        }
    }
}

// Developer keys capture the actual rendered surface and frame timings.
#[derive(Resource, Default)]
struct PerformanceCapture {
    frames: Vec<f64>,
    active: bool,
}

fn capture_tools(
    _commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    state: Res<Arcade>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut capture: ResMut<PerformanceCapture>,
) {
    if keys.just_pressed(KeyCode::F12) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use bevy::render::view::screenshot::{Screenshot, save_to_disk};
            let mut commands = _commands;
            let name = match state.screen {
                Screen::Lobby => "lobby",
                Screen::Mines => "minesweeper",
                Screen::Cards => "card-lab",
                Screen::Settings => "settings",
                Screen::Blackjack => "blackjack",
                Screen::Baccarat => "baccarat",
                Screen::Sudoku => "sudoku",
                Screen::Mahjong => "mahjong",
                Screen::Solitaire => "solitaire",
                Screen::FreeCell => "freecell",
                Screen::Spider => "spider",
                Screen::Hearts => "hearts",
                Screen::Cribbage => "cribbage",
                Screen::Holdem => "holdem",
                Screen::Roulette => "roulette",
                Screen::Backgammon => "backgammon",
            };
            let folder = std::path::Path::new("docs/screenshots");
            if std::fs::create_dir_all(folder).is_ok() {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(folder.join(format!("{name}.png"))));
            }
        }
    }
    if keys.just_pressed(KeyCode::F9) {
        capture.frames.clear();
        capture.active = true;
        info!("GameVex measurement started; keep the playfield foreground for 300 frames");
    }
    if capture.active {
        capture.frames.push(time.delta_secs_f64() * 1000.0);
        if capture.frames.len() >= 300 {
            capture.active = false;
            capture.frames.sort_by(f64::total_cmp);
            let avg = capture.frames.iter().sum::<f64>() / capture.frames.len() as f64;
            let p95 = capture.frames[284];
            if let Ok(window) = windows.single() {
                info!(
                    "GameVex PERF backend={} logical={}x{} dpi={} preset={:?} effects={} bloom={} avg_ms={avg:.2} p95_ms={p95:.2} fps={:.1}",
                    state.backend,
                    window.width(),
                    window.height(),
                    window.scale_factor(),
                    state.save.settings.display_preset,
                    state.save.settings.crt,
                    state.save.settings.bloom,
                    1000.0 / avg
                );
            }
        }
    }
}
