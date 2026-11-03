use crate::{
    Arcade, Pending, platform,
    rendering::{Frame, RenderOptions},
};
use bevy::{prelude::*, window::PrimaryWindow};
use gamevex_core::{blackjack::Command, cards::Suit, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Lobby,
    Manual,
    Mines,
    Cards,
    Settings,
    Blackjack,
    Baccarat,
    Sudoku,
    Mahjong,
    Solitaire,
    FreeCell,
    Spider,
    Hearts,
    Cribbage,
    Holdem,
    Roulette,
    Backgammon,
}
#[derive(Clone, Copy)]
pub enum Action {
    LobbyCategory(Option<LobbyCategory>),
    ManualTopic(Option<usize>),
    Open(Screen),
    Back,
    New(Difficulty, bool),
    Confirm,
    Cancel,
    FlagMode,
    Board,
    Preset,
    SelectTheme(ColorScheme),
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
    MahjongTile(usize),
    MahjongHint,
    MahjongUndo,
    MahjongShuffle,
    MahjongRestart,
    MahjongNew(gamevex_core::mahjong::Layout),
    MahjongZen(bool),
    BackgammonPoint(u8),
    BackgammonRoll,
    BackgammonNew,
    BackgammonDifficulty,
    RouletteBet(gamevex_core::roulette::Bet),
    RouletteStake(i64),
    RouletteStakeMin,
    RouletteStakeMax,
    RouletteSpin,
    RouletteRestart,
    HoldemStart,
    HoldemPlay(gamevex_core::holdem::Action),
    HoldemStake(i32),
    HoldemStakeMax,
    HoldemStakeMin,
    HoldemOpponents(i32),
    HoldemRaise(i32),
    HoldemRaiseMin,
    HoldemRaisePot,
    HoldemRestart,
    HoldemSkip,
    CribbageCard(usize),
    CribbageDiscard,
    CribbageGo,
    CribbageContinue,
    CribbageNew,
    HeartsCard(usize),
    HeartsPass,
    HeartsContinue,
    HeartsNew,
    PatiencePick(gamevex_core::patience::Pile, usize),
    PatienceTarget(gamevex_core::patience::Pile),
    PatienceDeal,
    PatienceUndo,
    PatienceFinish,
    PatienceHint,
    PatienceNew(gamevex_core::patience::Variant, bool),
}
pub(crate) use screens::LobbyCategory;

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
    pub lobby_clip: Option<Rect>,
    pub(crate) lobby_category: Option<LobbyCategory>,
    pub(crate) manual_topic: Option<usize>,
    pub lobby_max_scroll: f32,
    pub sudoku: Option<BoardLayout>,
    pub sudoku_max_pan: Vec2,
    pub mahjong_clip: Option<Rect>,
    pub mahjong_max_pan: Vec2,
    pub patience_clip: Option<Rect>,
    pub patience_max_pan: Vec2,
    pub roulette_clip: Option<Rect>,
    pub roulette_max_pan: f32,
    pub patience_origins: Vec<(gamevex_core::patience::Pile, usize, Vec2)>,
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
    layout.mahjong_clip = None;
    layout.patience_clip = None;
    layout.patience_origins.clear();
    layout.patience_max_pan = Vec2::ZERO;
    layout.roulette_clip = None;
    layout.roulette_max_pan = 0.0;
    layout.mahjong_max_pan = Vec2::ZERO;
    layout.card_order.clear();
    layout.settings_max_scroll = 0.0;
    layout.settings_clip = None;
    layout.lobby_clip = None;
    layout.lobby_category = state.lobby_category;
    layout.manual_topic = state.manual_topic;
    layout.lobby_max_scroll = 0.0;
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
    let t = frame.theme();
    frame.rect(Vec2::ZERO, size, t.bg);
    let margin = (size.x * 0.04).clamp(16.0, 52.0);
    let w = size.x - 2.0 * margin;
    // A quiet cabinet grid surrounds the flat play surfaces.
    let grid_color = Color::srgba(
        t.edge.to_srgba().red,
        t.edge.to_srgba().green,
        t.edge.to_srgba().blue,
        0.22,
    );
    for x in (0..size.x as usize).step_by(64) {
        frame.line(
            Vec2::new(x as f32, 0.0),
            Vec2::new(x as f32, size.y),
            0.5,
            grid_color,
        );
    }
    for y in (0..size.y as usize).step_by(64) {
        frame.line(
            Vec2::new(0.0, y as f32),
            Vec2::new(size.x, y as f32),
            0.5,
            grid_color,
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
        Screen::Manual => manual::manual(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Lobby => lobby(&mut frame, &mut state, &mut layout, margin, w),
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
        Screen::Backgammon => {
            backgammon::backgammon(&mut frame, &mut state, &mut layout, margin, w)
        }
        Screen::Roulette => roulette::roulette(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Blackjack => blackjack(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Holdem => holdem::holdem(
            &mut frame,
            &mut state,
            &mut layout,
            margin,
            w,
            time.delta_secs(),
        ),
        Screen::Cribbage => cribbage::cribbage(
            &mut frame,
            &mut state,
            &mut layout,
            margin,
            w,
            time.delta_secs(),
        ),
        Screen::Hearts => hearts::hearts(
            &mut frame,
            &mut state,
            &mut layout,
            margin,
            w,
            time.delta_secs(),
        ),
        Screen::Baccarat => baccarat::baccarat(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Sudoku => sudoku::sudoku(&mut frame, &mut state, &mut layout, margin, w),
        Screen::Solitaire | Screen::FreeCell | Screen::Spider => {
            patience::patience(&mut frame, &mut state, &mut layout, margin, w)
        }
        Screen::Mahjong => mahjong::mahjong(&mut frame, &mut state, &mut layout, margin, w),
    }
    if state.screen == Screen::Baccarat
        && let Some(target) = state.baccarat_focus
        && let Some(index) = layout
            .hits
            .iter()
            .position(|hit| matches!(hit.action, Action::Baccarat(command) if command == target))
    {
        state.focus = index;
        state.baccarat_focus = None;
    }
    publish_diagnostics(&state, &layout, size);
    if state.screen == Screen::Cards {
        platform::publish_cards(Some(
            layout
                .card_hits
                .map(|r| [r.min.x, r.min.y, r.width(), r.height()]),
        ));
    }
    if state.protected_save || state.persistence.starts_with("Playing") {
        let footer = compact(&state.persistence, ((size.x - 2.0 * margin) / 7.5) as usize);
        let t = frame.theme();
        let edge = t.edge;
        let gold = t.gold;
        frame.line(
            Vec2::new(margin, size.y - 33.0),
            Vec2::new(size.x - margin, size.y - 33.0),
            1.0,
            edge,
        );
        frame.text(Vec2::new(margin, size.y - 25.0), footer, 12.0, gold);
    }
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
mod backgammon;
mod blackjack;
pub(crate) mod blackjack_motion;
pub(crate) mod casino_fx;
mod cribbage;
pub(crate) mod game_palette;
mod hearts;
mod holdem;
mod mahjong;
pub(crate) mod manual;
mod patience;
mod roulette;
mod screens;
mod sudoku;
mod table_fx;

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
        "focusIndex": state.focus,
        "focusCount": layout.hits.len(),
        "modal": match state.modal {
            Some(Pending::NewHearts) => Some("hearts-replace"),
            Some(Pending::NewCribbage) => Some("cribbage-replace"),
            Some(Pending::LeaveHoldem(_)) => Some("holdem-leave"),
            Some(Pending::BackgammonReplace(_)) => Some("backgammon-replace"),
            Some(Pending::SudokuSetup) => Some("sudoku-setup"),
            Some(Pending::NewSudoku(..)) => Some("sudoku-replace"),
            Some(Pending::NewMahjong(..)) => Some("mahjong-replace"),
            Some(Pending::NewPatience(..))=>Some("patience-replace"),
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
    if state.screen == Screen::Mahjong || crate::patience_app::is_family(state.screen) {
        let feedback = if state.screen == Screen::Mahjong {
            &state.mahjong_ui.feedback
        } else {
            &state.patience_ui.feedback
        };
        platform::diagnostics_attribute("data-table-feedback", &serde_json::json!({
            "events":feedback.events.iter().map(|e|serde_json::json!({"kind":format!("{:?}",e.kind),"age":e.age})).collect::<Vec<_>>(),
            "reduced":state.save.settings.reduced_motion
        }).to_string());
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
                Action::Open(candidate)
                    if candidate == screen
                        && hit.rect.width() > 0.0
                        && hit.rect.height() >= 30.0 =>
                {
                    Some([
                        hit.rect.min.x,
                        hit.rect.min.y,
                        hit.rect.width(),
                        hit.rect.height(),
                    ])
                }
                _ => None,
            })
        };
        let mut categories = serde_json::Map::new();
        let mut categories_back = None;
        for hit in &layout.hits {
            if let Action::LobbyCategory(category) = hit.action {
                let bounds = [
                    hit.rect.min.x,
                    hit.rect.min.y,
                    hit.rect.width(),
                    hit.rect.height(),
                ];
                if let Some(category) = category {
                    categories.insert(category.key().into(), serde_json::json!(bounds));
                } else {
                    categories_back = Some(bounds);
                }
            }
        }
        let bounds = serde_json::json!({
            "categories": categories,
            "categoriesBack": categories_back,
            "lobbyCategory": state.lobby_category.map(LobbyCategory::key),
            "manual": action_bounds(Screen::Manual),
            "cards": action_bounds(Screen::Cards),
            "mines": action_bounds(Screen::Mines),
            "blackjack": action_bounds(Screen::Blackjack),
            "baccarat": action_bounds(Screen::Baccarat),
            "sudoku": action_bounds(Screen::Sudoku),
            "mahjong": action_bounds(Screen::Mahjong),
            "solitaire":action_bounds(Screen::Solitaire),
            "freecell":action_bounds(Screen::FreeCell),
            "spider":action_bounds(Screen::Spider),
            "hearts":action_bounds(Screen::Hearts),
            "cribbage":action_bounds(Screen::Cribbage),
            "holdem":action_bounds(Screen::Holdem),
            "roulette":action_bounds(Screen::Roulette),
            "backgammon":action_bounds(Screen::Backgammon),
            "lobbyClip":layout.lobby_clip.map(|r| [r.min.x,r.min.y,r.width(),r.height()]),
            "lobbyScroll":state.lobby_scroll,
            "lobbyMaxScroll":layout.lobby_max_scroll,
        });
        if let Ok(json) = serde_json::to_string(&bounds) {
            platform::diagnostics_attribute("data-action-bounds", &json);
        }
    }
    if state.screen == Screen::Manual {
        let mut topics = serde_json::Map::new();
        let mut contents = None;
        let mut lobby = None;
        for hit in &layout.hits {
            let bounds = [
                hit.rect.min.x,
                hit.rect.min.y,
                hit.rect.width(),
                hit.rect.height(),
            ];
            match hit.action {
                Action::ManualTopic(Some(i)) => {
                    topics.insert(manual::TOPICS[i].0.into(), serde_json::json!(bounds));
                }
                Action::ManualTopic(None) => contents = Some(bounds),
                Action::Open(Screen::Lobby) => lobby = Some(bounds),
                _ => {}
            }
        }
        platform::diagnostics_attribute(
            "data-action-bounds",
            &serde_json::json!({
                "manualTopics": topics, "manualContents": contents, "lobby": lobby,
                "manualTopic": state.manual_topic.map(|i| manual::TOPICS[i].0),
                "manualScroll": state.lobby_scroll, "manualMaxScroll": layout.lobby_max_scroll
            })
            .to_string(),
        );
    }
    if state.screen == Screen::Backgammon {
        let mut bounds = serde_json::Map::new();
        let ui = &state.backgammon_ui;
        let g = &state.save.backgammon;
        for hit in &layout.hits {
            let key = match hit.action {
                Action::BackgammonPoint(p)
                    if !ui.busy(state.save.settings.reduced_motion)
                        && (ui.legal.iter().any(|m| m.from == p)
                            || ui.selected.is_some_and(|source| {
                                ui.legal.iter().any(|m| m.from == source && m.to == p)
                            })) =>
                {
                    format!("backgammonPoint{p}")
                }
                Action::BackgammonPoint(_) => continue,
                Action::BackgammonRoll => "backgammonRoll".into(),
                Action::BackgammonNew => "backgammonNew".into(),
                Action::BackgammonDifficulty => "backgammonDifficulty".into(),
                Action::Cancel => "cancel".into(),
                Action::Confirm => "confirm".into(),
                Action::Open(Screen::Lobby) | Action::Back => "lobby".into(),
                Action::Open(Screen::Settings) => "settings".into(),
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
        platform::diagnostics_attribute("data-backgammon-view",&serde_json::json!({"turn":g.turn,"phase":g.phase,"board":g.board,"dice":g.dice,"rolled":g.rolled,"legal":ui.legal,"selected":ui.selected,"animating":ui.busy(state.save.settings.reduced_motion),"wins":state.save.backgammon_wins,"elapsedMs":g.elapsed_ms,"difficulty":g.difficulty}).to_string());
    }
    if state.screen == Screen::Roulette {
        let mut bounds = serde_json::Map::new();
        for hit in &layout.hits {
            if hit.rect.width() <= 0.0 || hit.rect.height() <= 0.0 {
                continue;
            }
            let key = match hit.action {
                Action::RouletteBet(gamevex_core::roulette::Bet::Number(n)) => {
                    format!("rouletteBetNumber{n}")
                }
                Action::RouletteBet(b) => format!("rouletteBet{b:?}"),
                Action::RouletteStake(d) => if d < 0 {
                    "rouletteStakeDown"
                } else {
                    "rouletteStakeUp"
                }
                .into(),
                Action::RouletteStakeMin => "rouletteStakeMin".into(),
                Action::RouletteStakeMax => "rouletteStakeMax".into(),
                Action::RouletteSpin => "rouletteSpin".into(),
                Action::RouletteRestart => "rouletteRestart".into(),
                Action::Open(Screen::Lobby) | Action::Back => "lobby".into(),
                Action::Open(Screen::Settings) => "settings".into(),
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
        let ui = &state.roulette_ui;
        platform::diagnostics_attribute("data-roulette-view", &serde_json::json!({
            "bankroll":state.save.roulette_balance,"displayBalance":ui.display_balance(state.save.roulette_balance),
            "stake":ui.stake,"bet":ui.bet,"spinning":ui.busy(),
            "result":if ui.busy() { None } else { ui.result },"history":ui.history,
            "tablePan":ui.table_pan,"maxPan":layout.roulette_max_pan,"tableClip":layout.roulette_clip.map(|r|[r.min.x,r.min.y,r.width(),r.height()]),
            "stripPosition":ui.strip_position(),"stripTarget":ui.strip_to
        }).to_string());
    }
    if state.screen == Screen::Holdem || matches!(state.modal, Some(Pending::LeaveHoldem(_))) {
        use gamevex_core::holdem::{Action as P, Phase};
        let mut bounds = serde_json::Map::new();
        for hit in &layout.hits {
            let key = match hit.action {
                Action::HoldemStart => "holdemStart".into(),
                Action::HoldemPlay(p) => match p {
                    P::Fold => "holdemFold",
                    P::Check => "holdemCheck",
                    P::Call => "holdemCall",
                    P::Bet(_) | P::Raise(_) => "holdemBet",
                    P::AllIn => "holdemAllIn",
                }
                .into(),
                Action::HoldemStake(d) => if d < 0 {
                    "holdemStakeDown"
                } else {
                    "holdemStakeUp"
                }
                .into(),
                Action::HoldemStakeMin => "holdemStakeMin".into(),
                Action::HoldemStakeMax => "holdemStakeMax".into(),
                Action::HoldemOpponents(d) => if d < 0 {
                    "holdemOpponentsDown"
                } else {
                    "holdemOpponentsUp"
                }
                .into(),
                Action::HoldemRaise(d) => if d < 0 {
                    "holdemRaiseDown"
                } else {
                    "holdemRaiseUp"
                }
                .into(),
                Action::HoldemRaiseMin => "holdemRaiseMin".into(),
                Action::HoldemRaisePot => "holdemRaisePot".into(),
                Action::HoldemRestart => "holdemRestart".into(),
                Action::HoldemSkip => "holdemSkip".into(),
                Action::Cancel => "cancel".into(),
                Action::Confirm => "confirm".into(),
                Action::Open(Screen::Lobby) | Action::Back => "lobby".into(),
                Action::Open(Screen::Settings) => "settings".into(),
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
        let g = &state.holdem_ui.game;
        platform::diagnostics_attribute("data-holdem-view",&serde_json::json!({
            "phase":g.phase,"turn":g.turn,"board":g.board,"bankroll":state.save.holdem_balance,
            "buyIn":state.holdem_ui.buy_in,"opponents":g.seats.len()-1,"raiseTarget":state.holdem_ui.raise_target,
            "animating":state.holdem_ui.busy(state.save.settings.reduced_motion),
            "handNumber":g.hand_number,"dealer":g.dealer,"pot":g.pot(),"payouts":g.payouts,
            "legal":g.legal_actions(0).iter().map(|a|match a{P::Fold=>"fold",P::Check=>"check",P::Call=>"call",P::Bet(_)=>"bet",P::Raise(_)=>"raise",P::AllIn=>"all_in"}).collect::<Vec<_>>(),
            "seats":g.seats.iter().enumerate().map(|(i,s)|serde_json::json!({"stack":s.stack,"folded":s.folded,"committed":s.committed,"hole":if i==0||(g.phase==Phase::Showdown&&!s.folded){Some(&s.hole)}else{None}})).collect::<Vec<_>>()
        }).to_string());
    }
    if state.screen == Screen::Cribbage {
        let mut bounds = serde_json::Map::new();
        for hit in &layout.hits {
            let key = match hit.action {
                Action::CribbageCard(i)
                    if (state.save.cribbage.phase == gamevex_core::cribbage::Phase::Discard
                        && (state.cribbage_ui.selected.len() < 2
                            || state.cribbage_ui.selected.contains(&i)))
                        || state.save.cribbage.legal_indices(0).contains(&i) =>
                {
                    format!("cribbageCard{i}")
                }
                Action::CribbageDiscard => "cribbageDiscard".into(),
                Action::CribbageGo => "cribbageGo".into(),
                Action::CribbageContinue => "cribbageContinue".into(),
                Action::CribbageNew => "cribbageNew".into(),
                Action::Cancel => "cancel".into(),
                Action::Confirm => "confirm".into(),
                Action::Open(Screen::Lobby) | Action::Back => "lobby".into(),
                Action::Open(Screen::Settings) => "settings".into(),
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
        let g = &state.save.cribbage;
        platform::diagnostics_attribute("data-cribbage-view", &serde_json::json!({
            "phase":g.phase,"turn":g.turn,"selected":state.cribbage_ui.selected,
            "legal":g.legal_indices(0),"round":g.round,"count":g.count,"dealer":g.dealer,"scores":g.scores,"show_step":g.show_step,
            "hovered":state.cribbage_ui.hovered,"wins":state.save.cribbage_wins,
        }).to_string());
    }
    if state.screen == Screen::Hearts {
        let mut bounds = serde_json::Map::new();
        for hit in &layout.hits {
            let key = match hit.action {
                Action::HeartsCard(i)
                    if !state.hearts_ui.motion.busy()
                        && ((state.save.hearts.phase == gamevex_core::hearts::Phase::Passing
                            && (state.hearts_ui.selected.len() < 3
                                || state.hearts_ui.selected.contains(&i)))
                            || state.save.hearts.legal_indices(0).contains(&i)) =>
                {
                    format!("heartsCard{i}")
                }
                Action::HeartsPass => "heartsPass".into(),
                Action::HeartsContinue => "heartsContinue".into(),
                Action::HeartsNew => "heartsNew".into(),
                Action::Cancel => "cancel".into(),
                Action::Confirm => "confirm".into(),
                Action::Open(Screen::Lobby) | Action::Back => "lobby".into(),
                Action::Open(Screen::Settings) => "settings".into(),
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
        let g = &state.save.hearts;
        platform::diagnostics_attribute("data-hearts-view", &serde_json::json!({
            "phase":g.phase,"turn":g.turn,"selected":state.hearts_ui.selected,
            "legal":g.legal_indices(0),"handNumber":g.hand_number,"heartsBroken":g.hearts_broken,
            "animating":state.hearts_ui.motion.busy(),"motion":state.hearts_ui.motion.kind().map(|k|format!("{k:?}")),"motionCards":state.hearts_ui.motion.flights().len(),"motionTime":state.hearts_ui.motion.flights().first().map(|f|f.elapsed),
            "hovered":state.hearts_ui.hovered,"handPositions":state.hearts_ui.card_positions.iter().map(|p|[p.x,p.y]).collect::<Vec<_>>(),
            "totals":g.totals,"handPoints":g.hand_points,
        }).to_string());
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
    if crate::patience_app::is_family(state.screen) {
        let mut bounds = serde_json::Map::new();
        for hit in &layout.hits {
            let key = match hit.action {
                Action::PatiencePick(p, i) => format!("pick{p:?}_{i}"),
                Action::PatienceTarget(p) => format!("target{p:?}"),
                Action::PatienceDeal => "deal".into(),
                Action::PatienceUndo => "undo".into(),
                Action::PatienceFinish => "finish".into(),
                Action::PatienceHint => "hint".into(),
                Action::PatienceNew(_, true) => "restart".into(),
                Action::PatienceNew(v, false) if v == state.patience_game().variant => "new".into(),
                Action::PatienceNew(v, false) => format!("mode{v:?}"),
                Action::Open(Screen::Lobby) => "lobby".into(),
                Action::Cancel => "cancel".into(),
                Action::Confirm => "confirm".into(),
                _ => continue,
            };
            if let Action::PatienceNew(v, false) = hit.action {
                bounds.insert(
                    format!("mode{v:?}"),
                    serde_json::json!([
                        hit.rect.min.x,
                        hit.rect.min.y,
                        hit.rect.width(),
                        hit.rect.height()
                    ]),
                );
            }
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
        let g = state.patience_game();
        platform::diagnostics_attribute("data-patience-view",&serde_json::json!({"variant":g.variant,"modeLabel":match g.variant {gamevex_core::patience::Variant::FreeCell=>"Standard",gamevex_core::patience::Variant::FreeCellRelaxed=>"Relaxed",v=>v.label()},"canFinish":state.patience_ui.can_finish,"animating":state.patience_ui.motion.busy(),"selected":state.patience_ui.selected.map(|(p,i)|(format!("{p:?}"),i)),"dragActive":state.patience_ui.drag.is_some_and(|d|d.active),"dragSource":state.patience_ui.drag.map(|d|(format!("{:?}",d.source.0),d.source.1)),"pan":[state.patience_ui.pan.x,state.patience_ui.pan.y],"won":g.won(),"clip":layout.patience_clip.map(|r|[r.min.x,r.min.y,r.width(),r.height()]),"maxPan":[layout.patience_max_pan.x,layout.patience_max_pan.y]}).to_string());
    }
    if state.screen == Screen::Mahjong {
        let mut bounds = serde_json::Map::new();
        for hit in &layout.hits {
            let key = match hit.action {
                Action::MahjongTile(i) => format!("tile{i}"),
                Action::MahjongHint => "hint".into(),
                Action::MahjongZen(true) => "zen".into(),
                Action::MahjongZen(false) => "classic".into(),
                Action::MahjongUndo => "undo".into(),
                Action::MahjongShuffle => "shuffle".into(),
                Action::MahjongRestart => "restart".into(),
                Action::MahjongNew(layout) => format!("new{}", layout.label().to_ascii_lowercase()),
                Action::Cancel => "cancel".into(),
                Action::Confirm => "confirm".into(),
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
        platform::diagnostics_attribute("data-mahjong-view", &serde_json::json!({"selected":state.mahjong_ui.selected,"hint":state.mahjong_ui.hint,"pan":[state.mahjong_ui.pan.x,state.mahjong_ui.pan.y],"remaining":state.save.mahjong.remaining(),"zen":state.save.mahjong.zen,"shuffles":state.save.mahjong.shuffles,"won":state.save.mahjong.won(),"stuck":state.save.mahjong.stuck(),"clip":layout.mahjong_clip.map(|r|[r.min.x,r.min.y,r.width(),r.height()]),"maxPan":[layout.mahjong_max_pan.x,layout.mahjong_max_pan.y]}).to_string());
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
    let t = f.theme();
    if (s.keyboard && s.focus == index) || s.cursor.is_some_and(|p| logo.contains(p)) {
        f.rounded_outline(
            logo.min,
            logo.size(),
            4.0,
            1.0,
            if s.keyboard { t.gold } else { t.accent },
        );
    }
    f.text(Vec2::new(m, 24.0), "GAMEVEX", 25.0, t.accent);
    l.hits.push(Hit {
        rect: logo,
        action: Action::Open(Screen::Lobby),
    });
    let width = f.size.x;
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
    if s.screen == Screen::Lobby {
        button(
            f,
            s,
            l,
            rect(x - 88.0, 24.0, 88.0, 34.0),
            "Manual",
            Action::Open(Screen::Manual),
            false,
        );
    }
    if matches!(s.screen, Screen::Settings | Screen::Manual) {
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
    f.line(
        Vec2::new(m, 76.0),
        Vec2::new(width - m, 76.0),
        1.0,
        t.accent,
    );
    f.rect(Vec2::new(m, 74.0), Vec2::new(78.0, 4.0), t.gold);
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
