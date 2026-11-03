use super::*;

impl Arcade {
    pub(super) fn open(&mut self, screen: Screen) {
        self.baccarat_focus = None;
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
        if screen == Screen::Lobby && self.screen != Screen::Settings {
            self.lobby_category = None;
            self.lobby_scroll = 0.0;
        }
        if screen == Screen::Manual && self.screen != Screen::Settings {
            self.manual_topic = None;
            self.lobby_scroll = 0.0;
        }
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
            Screen::Baccarat => {
                self.baccarat_focus = Some(match self.save.baccarat.phase {
                    baccarat::Phase::Betting => {
                        baccarat::Command::Select(baccarat::BetSide::Player)
                    }
                    baccarat::Phase::Settled => baccarat::Command::NextRound,
                    baccarat::Phase::GameOver => baccarat::Command::Restart,
                });
                0
            }
            Screen::Sudoku => sudoku_app::BOARD_FOCUS,
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

    pub(super) fn open_modal(&mut self, pending: Pending) {
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

    pub(super) fn restore_modal_focus(&mut self) {
        self.modal = None;
        self.focus = self.modal_return_focus;
    }
}
