use super::*;

/// Preserve the OS event order and each press/release position. ButtonInput's
/// per-frame sets can collapse fast clicks and pair them with a later cursor move.
pub(super) fn handle_pointer_input(
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
        state.connect4_ui.settle_motion();
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
            && layout.lobby_category == state.lobby_category
            && layout.manual_topic == state.manual_topic
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

pub(super) fn pointer_pressed(state: &mut Arcade, layout: &ViewLayout, button: MouseButton) {
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

pub(super) fn handle_input(
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
    if layout.surface != Some(state.screen)
        || layout.modal != state.modal.is_some()
        || layout.lobby_category != state.lobby_category
        || layout.manual_topic != state.manual_topic
    {
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
        } else if state.patience_ui.drag.is_some() {
            state.patience_ui.drag = None;
            state.patience_ui.motion.clear();
        } else if state.lab.drag.is_some() {
            state.lab.cancel_drag();
            state.patience_ui.drag = None;
            state.patience_ui.motion.clear();
        } else if state.screen == Screen::Manual && state.manual_topic.is_some() {
            state.act(Action::ManualTopic(None));
        } else if state.screen == Screen::Lobby && state.lobby_category.is_some() {
            state.act(Action::LobbyCategory(None));
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
    if state.screen == Screen::Connect4 && state.connect4_key(&keys, &layout) {
        wheel.clear();
        return;
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
    } else if state.baccarat_focus.is_none()
        && (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
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
    if matches!(state.screen, Screen::Lobby | Screen::Manual) && state.modal.is_none() {
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
        if matches!(state.screen, Screen::Lobby | Screen::Manual) && state.modal.is_none() {
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
