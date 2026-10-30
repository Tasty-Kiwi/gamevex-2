use crate::{
    Arcade, Pending, platform,
    rendering::{Frame, RenderOptions},
};
use bevy::{prelude::*, window::PrimaryWindow};
use gamevex_core::{blackjack::Command, cards::Suit, *};

pub const BG: Color = Color::srgb(0.008, 0.016, 0.031);
pub const PANEL: Color = Color::srgb(0.025, 0.055, 0.085);
pub const CYAN: Color = Color::srgb(0.0, 0.83, 1.0);
pub const MAGENTA: Color = Color::srgb(1.0, 0.0, 0.67);
pub const GOLD: Color = Color::srgb(1.0, 0.67, 0.0);
pub const INK: Color = Color::srgb(0.87, 0.94, 1.0);
pub const MUTED: Color = Color::srgb(0.39, 0.58, 0.68);
pub const EDGE: Color = Color::srgb(0.08, 0.22, 0.29);
pub const GREEN: Color = Color::srgb(0.0, 1.0, 0.72);
pub const RED: Color = Color::srgb(1.0, 0.2, 0.4);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Lobby,
    Mines,
    Cards,
    Settings,
    Blackjack,
    Baccarat,
    Sudoku,
}
#[derive(Clone, Copy)]
pub enum Action {
    Open(Screen),
    Back,
    New(Difficulty, bool),
    Confirm,
    Cancel,
    FlagMode,
    Board,
    Preset,
    Palette,
    Curvature(f32),
    Vignette(f32),
    ChromaticAberration(f32),
    Bloom(f32),
    Scanlines(f32),
    Volume(f32),
    Crt,
    Motion,
    Mute,
    Retry,
    Recover,
    Flip,
    Deal,
    ReturnCard,
    Blackjack(gamevex_core::blackjack::Command),
    Baccarat(gamevex_core::baccarat::Command),
    Sudoku(gamevex_core::sudoku::Command),
    SudokuCell,
    SudokuPencil,
    SudokuErrors,
    SudokuSame,
    SudokuSetup,
    SudokuChooseSize(u8),
    SudokuChooseDifficulty(gamevex_core::sudoku::Difficulty),
    SudokuStart,
    SudokuHighlights,
}
pub struct Hit {
    pub rect: Rect,
    pub action: Action,
}
pub struct BoardLayout {
    pub origin: Vec2,
    pub clip: Rect,
    pub cell: f32,
}
impl BoardLayout {
    pub fn index(&self, point: Vec2, cols: usize, rows: usize) -> Option<usize> {
        if !self.clip.contains(point) || point.x >= self.clip.max.x || point.y >= self.clip.max.y {
            return None;
        }
        let local = (point - self.origin) / self.cell;
        if local.x < 0.0 || local.y < 0.0 {
            return None;
        }
        let col = local.x.floor() as usize;
        let row = local.y.floor() as usize;
        (col < cols && row < rows).then_some(row * cols + col)
    }
}
#[derive(Resource, Default)]
pub struct ViewLayout {
    pub size: Vec2,
    pub surface: Option<Screen>,
    pub modal: bool,
    pub hits: Vec<Hit>,
    pub board: Option<BoardLayout>,
    pub cards: [Rect; 5],
    pub card_hits: [Rect; 5],
    pub slots: [Rect; 3],
    pub card_order: Vec<usize>,
    pub settings_max_scroll: f32,
    pub settings_clip: Option<Rect>,
    pub sudoku: Option<BoardLayout>,
    pub sudoku_max_pan: Vec2,
}

pub fn paint(
    mut frame: ResMut<Frame>,
    mut options: ResMut<RenderOptions>,
    mut state: ResMut<Arcade>,
    mut layout: ResMut<ViewLayout>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let metrics = platform::canvas_metrics();
    let size = metrics
        .map(|[w, h, _]| Vec2::new(w, h))
        .unwrap_or(Vec2::new(window.width(), window.height()));
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }
    frame.clear(size);
    layout.size = size;
    layout.surface = Some(state.screen);
    layout.modal = state.modal.is_some();
    layout.hits.clear();
    layout.board = None;
    layout.sudoku = None;
    layout.sudoku_max_pan = Vec2::ZERO;
    layout.card_order.clear();
    layout.settings_max_scroll = 0.0;
    layout.settings_clip = None;
    options.scale_factor = metrics.map(|[_, _, d]| d).unwrap_or(window.scale_factor());
    options.pixel = state.save.settings.display_preset == DisplayPreset::PixelCrt;
    frame.min_stroke = if options.pixel { 2.0 } else { 0.0 };
    options.bloom = state.save.settings.bloom;
    options.scanlines = state.save.settings.scanlines;
    options.crt = state.save.settings.crt;
    options.curvature = state.save.settings.curvature;
    options.vignette = state.save.settings.vignette;
    options.chromatic_aberration = state.save.settings.chromatic_aberration;
    frame.palette = state.save.settings.color_scheme;
    platform::publish_monitor(if options.crt {
        options.curvature * 0.12
    } else {
        0.0
    });
    frame.rect(Vec2::ZERO, size, BG);
    let margin = (size.x * 0.04).clamp(16.0, 52.0);
    let w = size.x - 2.0 * margin;
    // A quiet cabinet grid surrounds the flat play surfaces.
    for x in (0..size.x as usize).step_by(64) {
        frame.line(
            Vec2::new(x as f32, 0.0),
            Vec2::new(x as f32, size.y),
            0.5,
            Color::srgb(0.015, 0.045, 0.065),
        );
    }
    for y in (0..size.y as usize).step_by(64) {
        frame.line(
            Vec2::new(0.0, y as f32),
            Vec2::new(size.x, y as f32),
            0.5,
            Color::srgb(0.015, 0.045, 0.065),
        );
    }
    platform::publish_cards(None);
    if state.modal.is_some() {
        dialog(&mut frame, &state, &mut layout, margin);
        platform::publish_board(None);
        publish_diagnostics(&state, &layout, size);
        return;
    }
    header(&mut frame, &state, &mut layout, margin);
    match state.screen {
        Screen::Lobby => lobby(&mut frame, &state, &mut layout, margin, w),
        Screen::Mines => mines(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Cards => draw_cards(
            &mut frame,
            &mut state,
            &mut layout,
            margin,
            w,
            time.delta_secs(),
        ),
        Screen::Settings => settings(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Blackjack => blackjack(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Baccarat => baccarat::baccarat(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Sudoku => sudoku::sudoku(&mut frame, &mut state, &mut layout, margin, w),
    }
    publish_diagnostics(&state, &layout, size);
    if state.screen == Screen::Cards {
        platform::publish_cards(Some(
            layout
                .card_hits
                .map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
        ));
    }
    let status = if state.protected_save || state.persistence.starts_with("Playing") {
        GOLD
    } else {
        MUTED
    };
    let footer = compact(&state.persistence, ((size.x - 2.0 * margin) / 7.5) as usize);
    frame.line(
        Vec2::new(margin, size.y - 33.0),
        Vec2::new(size.x - margin, size.y - 33.0),
        1.0,
        EDGE,
    );
    frame.text(Vec2::new(margin, size.y - 25.0), footer, 12.0, status);
    if let Some(board) = &layout.board {
        platform::publish_board(Some([
            board.clip.min.x,
            board.clip.min.y,
            board.clip.width(),
            board.clip.height(),
        ]));
    } else {
        platform::publish_board(None);
    }
}

mod baccarat;
pub(crate) mod baccarat_motion;
mod blackjack;
pub(crate) mod blackjack_motion;
pub(crate) mod casino_fx;
mod screens;
mod sudoku;

use blackjack::blackjack;
use screens::{button, cards as draw_cards, dialog, lobby, mines, rects_overlap, settings};

#[cfg(test)]
use screens::card_hit_bounds;

fn publish_diagnostics(state: &Arcade, layout: &ViewLayout, size: Vec2) {
    if !platform::diagnostics_enabled() {
        return;
    }
    platform::diagnostics_attribute("data-feedback", &serde_json::json!({
        "result": state.casino_feedback.result.as_ref().map(|r| format!("{:?}", r.kind)),
        "age": state.casino_feedback.result_age,
        "action": state.casino_feedback.action.filter(|_| state.casino_feedback.action_age < 1.2),
    }).to_string());
    let mut interaction = serde_json::json!({
        "screen": format!("{:?}", state.screen),
        "modal": match state.modal {
            Some(Pending::SudokuSetup) => Some("sudoku-setup"),
            Some(Pending::NewSudoku(..)) => Some("sudoku-replace"),
            Some(_) => Some("dialog"), None => None,
        },
        "selectedCard": state.lab.selected,
        // The browser harness uses true to mean face-down.
        "flipped": std::array::from_fn::<_, 5, _>(|i| {
            state.lab.cards.get(i).is_some_and(|card| !card.face_up)
        }),
        "slots": state.lab.slots,
        "dragging": state.lab.drag,
        "pan": [state.pan.x, state.pan.y],
        "viewport": [size.x, size.y],
        "difficulty": format!("{:?}", state.save.minesweeper.game.difficulty),
    });
    if state.screen == Screen::Blackjack {
        interaction["phase"] = serde_json::json!(format!("{:?}", state.save.blackjack.phase));
        interaction["activeHand"] = serde_json::json!(state.save.blackjack.active_hand);
        interaction["balance"] = serde_json::json!(state.save.blackjack.balance);
    }
    if state.screen == Screen::Baccarat {
        interaction["phase"] = serde_json::json!(format!("{:?}", state.save.baccarat.phase));
        interaction["balance"] = serde_json::json!(state.save.baccarat.balance);
    }
    if let Ok(json) = serde_json::to_string(&interaction) {
        platform::diagnostics_attribute("data-interaction", &json);
    }
    if state.screen == Screen::Cards {
        let bounds = layout
            .slots
            .map(|r| [r.min.x, r.min.y, r.width(), r.height()]);
        if let Ok(json) = serde_json::to_string(&bounds) {
            platform::diagnostics_attribute("data-slot-bounds", &json);
        }
    }
    if state.screen == Screen::Lobby {
        let action_bounds = |screen| {
            layout.hits.iter().find_map(|hit| match hit.action {
                Action::Open(candidate) if candidate == screen => Some([
                    hit.rect.min.x,
                    hit.rect.min.y,
                    hit.rect.width(),
                    hit.rect.height(),
                ]),
                _ => None,
            })
        };
        let bounds = serde_json::json!({
            "cards": action_bounds(Screen::Cards),
            "mines": action_bounds(Screen::Mines),
            "blackjack": action_bounds(Screen::Blackjack),
            "baccarat": action_bounds(Screen::Baccarat),
            "sudoku": action_bounds(Screen::Sudoku),
        });
        if let Ok(json) = serde_json::to_string(&bounds) {
            platform::diagnostics_attribute("data-action-bounds", &json);
        }
    }
    if state.screen == Screen::Blackjack {
        let game = &state.save.blackjack;
        let action_bounds = |command| {
            layout.hits.iter().find_map(|hit| match hit.action {
                Action::Blackjack(candidate) if candidate == command => Some([
                    hit.rect.min.x,
                    hit.rect.min.y,
                    hit.rect.width(),
                    hit.rect.height(),
                ]),
                _ => None,
            })
        };
        let bounds = serde_json::json!({
            "deal": action_bounds(Command::Deal),
            "hit": action_bounds(Command::Hit),
            "stand": action_bounds(Command::Stand),
            "double": action_bounds(Command::Double),
            "split": action_bounds(Command::Split),
            "surrender": action_bounds(Command::Surrender),
            "insure": action_bounds(Command::Insure),
            "decline": action_bounds(Command::DeclineInsurance),
            "next": action_bounds(Command::NextRound),
            "restart": action_bounds(Command::Restart),
            "betDown": action_bounds(Command::SetBet(game.bet.saturating_sub(10).max(1))),
            "betUp": action_bounds(Command::SetBet(game.bet.saturating_add(10))),
            "handsDown": action_bounds(Command::SetHandCount(game.hand_count.saturating_sub(1).max(1))),
            "handsUp": action_bounds(Command::SetHandCount((game.hand_count + 1).min(5))),
        });
        if let Ok(json) = serde_json::to_string(&bounds) {
            platform::diagnostics_attribute("data-action-bounds", &json);
        }
    }
    if state.screen == Screen::Baccarat {
        use gamevex_core::baccarat::{BetSide, Command as Bc, MAX_BET};
        let game = &state.save.baccarat;
        let bounds = |command| {
            layout.hits.iter().find_map(|hit| match hit.action {
                Action::Baccarat(candidate) if candidate == command => Some([
                    hit.rect.min.x,
                    hit.rect.min.y,
                    hit.rect.width(),
                    hit.rect.height(),
                ]),
                _ => None,
            })
        };
        platform::diagnostics_attribute(
            "data-action-bounds",
            &serde_json::json!({
                "player": bounds(Bc::Select(BetSide::Player)),
                "banker": bounds(Bc::Select(BetSide::Banker)),
                "tie": bounds(Bc::Select(BetSide::Tie)),
                "betDown": bounds(Bc::SetBet(game.bet.saturating_sub(10).max(10))),
                "betUp": bounds(Bc::SetBet(game.bet.saturating_add(10).min(MAX_BET))),
                "max": bounds(Bc::MaxBet), "deal": bounds(Bc::Deal),
                "next": bounds(Bc::NextRound), "restart": bounds(Bc::Restart),
            })
            .to_string(),
        );
    }
    if state.screen == Screen::Sudoku {
        let mut bounds = serde_json::Map::new();
        for hit in &layout.hits {
            let key = match hit.action {
                Action::SudokuCell => "board".to_owned(),
                Action::Sudoku(gamevex_core::sudoku::Command::Enter { value, .. }) => {
                    format!("value{value}")
                }
                Action::Sudoku(gamevex_core::sudoku::Command::Erase(_)) => "erase".into(),
                Action::Sudoku(gamevex_core::sudoku::Command::Undo) => "undo".into(),
                Action::Sudoku(gamevex_core::sudoku::Command::Hint) => "hint".into(),
                Action::SudokuPencil => "pencil".into(),
                Action::SudokuSetup => "setup".into(),
                Action::SudokuHighlights => "highlights".into(),
                Action::SudokuChooseSize(size) => format!("size{size}"),
                Action::SudokuChooseDifficulty(difficulty) => {
                    difficulty.label().to_ascii_lowercase()
                }
                Action::SudokuStart => "start".into(),
                Action::Cancel => "cancel".into(),
                Action::Confirm => "confirm".into(),
                Action::SudokuErrors => "errors".into(),
                Action::SudokuSame => "same".into(),
                _ => continue,
            };
            bounds.insert(
                key,
                serde_json::json!([
                    hit.rect.min.x,
                    hit.rect.min.y,
                    hit.rect.width(),
                    hit.rect.height()
                ]),
            );
        }
        platform::diagnostics_attribute(
            "data-action-bounds",
            &serde_json::Value::Object(bounds).to_string(),
        );
        let board = layout.sudoku.as_ref();
        platform::diagnostics_attribute("data-sudoku-view", &serde_json::json!({
            "origin": board.map(|b| [b.origin.x,b.origin.y]),
            "clip": board.map(|b| [b.clip.min.x,b.clip.min.y,b.clip.width(),b.clip.height()]),
            "cell": board.map(|b| b.cell), "selected":state.sudoku_ui.selected,
            "pencil":state.sudoku_ui.pencil, "undo":state.sudoku_ui.history.len(),
            "message":state.sudoku_ui.message,
            "highlights":state.sudoku_ui.show_highlights,
            "setup":state.sudoku_ui.setup.map(|(size,difficulty)| serde_json::json!({"size":size,"difficulty":difficulty})),
        }).to_string());
    }

    if state.screen == Screen::Mines {
        let bounds = Difficulty::ALL.map(|difficulty| {
            layout
                .hits
                .iter()
                .find_map(|hit| match hit.action {
                    Action::New(candidate, false) if candidate == difficulty => Some([
                        hit.rect.min.x,
                        hit.rect.min.y,
                        hit.rect.width(),
                        hit.rect.height(),
                    ]),
                    _ => None,
                })
                .unwrap_or([0.0; 4])
        });
        if let Ok(json) = serde_json::to_string(&bounds) {
            platform::diagnostics_attribute("data-difficulty-bounds", &json);
        }
    }
}

fn header(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32) {
    let logo = rect(m - 4.0, 20.0, 128.0, 42.0);
    let index = l.hits.len();
    if (s.keyboard && s.focus == index) || s.cursor.is_some_and(|p| logo.contains(p)) {
        f.rounded_outline(
            logo.min,
            logo.size(),
            4.0,
            1.0,
            if s.keyboard { GOLD } else { CYAN },
        );
    }
    f.text(Vec2::new(m, 24.0), "GAMEVEX", 25.0, CYAN);
    l.hits.push(Hit {
        rect: logo,
        action: Action::Open(Screen::Lobby),
    });
    let width = f.size.x;
    if width > 720.0 {
        f.text(
            Vec2::new(m + 166.0, 32.0),
            "LOCAL ARCADE / RUST DEMO",
            12.0,
            MUTED,
        );
    }
    let mut x = width - m;
    if s.screen != Screen::Settings {
        x -= 100.0;
        button(
            f,
            s,
            l,
            rect(x, 24.0, 100.0, 34.0),
            "Settings",
            Action::Open(Screen::Settings),
            false,
        );
        x -= 12.0;
    }
    if s.screen != Screen::Lobby {
        x -= 84.0;
        button(
            f,
            s,
            l,
            rect(x, 24.0, 84.0, 34.0),
            "Lobby",
            Action::Open(Screen::Lobby),
            false,
        );
    }
    f.line(Vec2::new(m, 76.0), Vec2::new(width - m, 76.0), 1.0, CYAN);
    f.rect(Vec2::new(m, 74.0), Vec2::new(78.0, 4.0), GOLD);
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_corners(Vec2::new(x, y), Vec2::new(x + w, y + h))
}
pub fn difficulty_name(d: Difficulty) -> &'static str {
    match d {
        Difficulty::Beginner => "Beginner",
        Difficulty::Intermediate => "Intermediate",
        Difficulty::Expert => "Expert",
    }
}
pub fn format_time(ms: u64) -> String {
    let seconds = ms / 1000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
fn compact(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}...",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn board_picking_respects_pan_and_clipped_edges() {
        let b = BoardLayout {
            origin: Vec2::new(-12.0, 50.0),
            clip: rect(20.0, 50.0, 200.0, 200.0),
            cell: 32.0,
        };
        assert_eq!(b.index(Vec2::new(20.0, 51.0), 30, 16), Some(1));
        assert_eq!(b.index(Vec2::new(19.0, 51.0), 30, 16), None);
        assert_eq!(b.index(Vec2::new(219.0, 81.0), 30, 16), Some(7));
        assert_eq!(b.index(Vec2::new(220.0, 81.0), 30, 16), None);
    }

    #[test]
    fn card_hover_bounds_keep_the_lifted_bottom_edge_stable() {
        let resting = rect(20.0, 48.0, 80.0, 112.0);
        let lifted = rect(20.0, 30.0, 80.0, 112.0);
        let hit = card_hit_bounds(lifted, resting);
        assert!(hit.contains(Vec2::new(40.0, 159.0)));
        assert!(hit.contains(Vec2::new(40.0, 31.0)));
        assert!(!hit.contains(Vec2::new(40.0, 161.0)));
        assert_eq!(card_hit_bounds(resting, resting), resting);
    }

    #[test]
    fn card_hover_uses_the_topmost_painted_card() {
        let cards = [rect(20.0, 30.0, 80.0, 112.0), rect(55.0, 30.0, 80.0, 112.0)];
        let order = [0, 1];
        let point = Vec2::new(65.0, 50.0);
        let hit = order
            .iter()
            .rev()
            .copied()
            .find(|&i| card_hit_bounds(cards[i], cards[i]).contains(point));
        assert_eq!(hit, Some(1));
    }

    #[test]
    fn rank_is_hidden_when_a_later_card_covers_any_part_of_it() {
        let rank = rect(32.0, 41.0, 32.0, 27.0);
        let later = rect(60.0, 30.0, 80.0, 112.0);
        assert!(rects_overlap(rank, later));
    }
}
