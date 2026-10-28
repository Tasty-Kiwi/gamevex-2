mod platform;
mod presentation;
mod rendering;

use bevy::{
    audio::{AudioSource, Volume},
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::{
        RenderPlugin,
        renderer::RenderAdapterInfo,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
    window::{PrimaryWindow, WindowCloseRequested, WindowFocused, WindowResolution},
};
use gamevex_core::{Command as GameCommand, cards::CardLab, *};
use presentation::{Action, Hit, Screen, ViewLayout};
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
    pub cursor: Option<Vec2>,
    pub card_positions: [Vec2; 5],
    pub drag_offset: Vec2,
    pub motion_time: f32,
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
        let message = match save.game.status {
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
            cursor: None,
            card_positions: [Vec2::splat(-1000.0); 5],
            drag_offset: Vec2::ZERO,
            motion_time: 0.0,
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
        self.save.game.status == Status::Playing
            || (self.save.game.status == Status::Ready
                && self.save.game.cells.iter().any(|c| c.flagged))
    }

    fn open(&mut self, screen: Screen) {
        if screen == Screen::Settings {
            self.settings_scroll = 0.0;
            self.settings_return = self.screen;
            self.settings_return_focus = self.focus;
        }
        self.screen = screen;
        self.focus = if screen == Screen::Mines { 9 } else { 0 };
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
            self.save.game.seed
        } else {
            fresh_seed()
        };
        self.save.game = Game::new(difficulty, seed);
        self.pan = Vec2::ZERO;
        self.cell = 0;
        self.fractional_ms = 0.0;
        self.focus = 9;
        self.message = "Ready. Reveal a cell to begin; the first area is safe.".into();
        self.modal = None;
        self.persist();
    }

    fn command(&mut self, command: GameCommand) {
        let events = self.save.game.command(command);
        if events.is_empty() {
            return;
        }
        self.sound = Some(650.0);
        for event in events {
            match event {
                DomainEvent::Won => {
                    let record = self.save.records.record(&self.save.game);
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
        if self.modal.is_some() && !matches!(action, Action::Confirm | Action::Cancel) {
            return;
        }
        self.sound = Some(440.0);
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
                Some(Pending::Recover) => {
                    self.reset_saved_data();
                    self.persist();
                }
                None => {}
            },
            Action::Cancel => self.restore_modal_focus(),
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
        .add_systems(Startup, diagnostics)
        .add_systems(Update, capture_tools)
        .add_systems(
            Update,
            (lifecycle, handle_input, presentation::paint, play_sound).chain(),
        )
        .run();
}

fn diagnostics(adapter: Option<Res<RenderAdapterInfo>>, mut state: ResMut<Arcade>) {
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
    let active = foreground && state.screen == Screen::Mines && state.modal.is_none();
    let delta = time.delta_secs_f64();
    let millis = account_play_time(
        delta,
        active,
        state.last_active,
        transitioned,
        &mut state.fractional_ms,
    );
    state.save.game.advance_elapsed(millis, active);
    if !foreground || transitioned {
        state.lab.cancel_drag();
    }
    state.save_age += delta.min(0.5) as f32;
    state.motion_time += delta.min(0.1) as f32;
    if (state.last_active && !active)
        || transitioned
        || close.read().count() > 0
        || (active && state.save.game.status == Status::Playing && state.save_age >= 5.0)
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

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut wheel: MessageReader<MouseWheel>,
    layout: Res<ViewLayout>,
    mut state: ResMut<Arcade>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    state.cursor = if let Some([_, _, density]) = platform::canvas_metrics() {
        window.physical_cursor_position().map(|p| p / density)
    } else {
        window.cursor_position()
    };
    let size = platform::canvas_metrics()
        .map(|[w, h, _]| Vec2::new(w, h))
        .unwrap_or(Vec2::new(window.width(), window.height()));
    state.cursor = state.cursor.and_then(|p| {
        rendering::screen_to_surface(
            p,
            size,
            if state.save.settings.crt {
                state.save.settings.curvature
            } else {
                0.0
            },
        )
    });
    if !window.focused || !platform::visible() {
        wheel.clear();
        return;
    }
    let gesture = mouse.just_pressed(MouseButton::Left)
        || mouse.just_pressed(MouseButton::Right)
        || keys.get_just_pressed().next().is_some();
    if gesture {
        state.audio_started = true;
    }
    if keys.get_just_pressed().next().is_some() {
        state.keyboard = true;
    }
    if keys.just_pressed(KeyCode::Escape) {
        if state.modal.is_some() {
            state.act(Action::Cancel);
        } else if state.lab.drag.is_some() {
            state.lab.cancel_drag();
        } else if state.screen != Screen::Lobby {
            state.act(Action::Back);
        }
        return;
    }
    let focus_count = if state.modal.is_some() {
        2
    } else {
        layout.hits.len()
    };
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
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
            let action = if state.focus == 0 {
                Action::Cancel
            } else {
                Action::Confirm
            };
            state.act(action);
        }
        if mouse.just_pressed(MouseButton::Left)
            && let Some(hit) = hit_at(&layout.hits, state.cursor)
        {
            state.act(hit.action);
        }
        wheel.clear();
        return;
    }
    if state.screen == Screen::Mines && board_focused {
        let cols = state.save.game.cols();
        let rows = state.save.game.rows();
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
        if let Some(board) = &layout.board {
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
    for event in wheel.read() {
        if state.screen == Screen::Settings {
            let scale = if event.unit == MouseScrollUnit::Line {
                36.0
            } else {
                1.0
            };
            state.settings_scroll =
                (state.settings_scroll - event.y * scale).clamp(0.0, layout.settings_max_scroll);
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
    if mouse.just_pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Right) {
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
                    && let Some(i) = board.index(
                        state.cursor.unwrap(),
                        state.save.game.cols(),
                        state.save.game.rows(),
                    )
                {
                    state.cell = i;
                    let command = if mouse.just_pressed(MouseButton::Right) || state.flag_mode {
                        GameCommand::ToggleFlag(i)
                    } else if state.save.game.cells[i].revealed {
                        GameCommand::Chord(i)
                    } else {
                        GameCommand::Reveal(i)
                    };
                    state.command(command);
                }
            } else if mouse.just_pressed(MouseButton::Left) {
                state.act(hit.action);
            }
        } else if state.screen == Screen::Cards
            && mouse.just_pressed(MouseButton::Left)
            && let Some(i) = layout
                .card_order
                .iter()
                .rev()
                .copied()
                .find(|i| state.cursor.is_some_and(|p| layout.cards[*i].contains(p)))
        {
            state.lab.pick(i);
            state.drag_offset = state.cursor.unwrap() - layout.cards[i].min;
        }
    }
    if state.lab.drag.is_some() && (!mouse.pressed(MouseButton::Left) || state.cursor.is_none()) {
        let slot = state
            .cursor
            .and_then(|p| layout.slots.iter().position(|r| r.contains(p)));
        let accepted = state.lab.drop_in(slot);
        debug!(
            "CardLab pointer drop slot={slot:?} accepted={accepted} at {:?}",
            state.cursor
        );
    }
}

fn hit_at(hits: &[Hit], cursor: Option<Vec2>) -> Option<&Hit> {
    hits.iter()
        .rev()
        .find(|hit| cursor.is_some_and(|p| hit.rect.contains(p)))
}

fn play_sound(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    mut state: ResMut<Arcade>,
) {
    let Some(frequency) = state.sound.take() else {
        return;
    };
    if !state.audio_started || state.save.settings.muted || state.save.settings.volume <= 0.0 {
        return;
    }
    let source = sources.add(AudioSource {
        bytes: tone(frequency).into(),
    });
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings {
            volume: Volume::Linear(state.save.settings.volume * 0.12),
            ..PlaybackSettings::DESPAWN
        },
    ));
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

    fn test_arcade() -> Arcade {
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
            cursor: None,
            card_positions: [Vec2::ZERO; 5],
            drag_offset: Vec2::ZERO,
            motion_time: 0.0,
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

    #[test]
    fn modal_cancel_restores_focus_and_confirm_starts_new_game_on_board() {
        let mut state = test_arcade();
        state.screen = Screen::Mines;
        state.focus = 9;
        state.save.game.command(GameCommand::Reveal(40));
        let original_game = state.save.game.clone();

        state.act(Action::New(Difficulty::Intermediate, false));
        assert!(matches!(state.modal, Some(Pending::New(_, _))));
        assert_eq!(state.focus, 0); // Cancel is the first modal control.
        state.act(Action::Cancel);
        assert!(state.modal.is_none());
        assert_eq!(state.focus, 9);
        assert_eq!(state.save.game, original_game);

        state.act(Action::New(Difficulty::Intermediate, false));
        state.act(Action::Confirm);
        assert!(state.modal.is_none());
        assert_eq!(state.save.game.difficulty, Difficulty::Intermediate);
        assert_eq!(state.save.game.status, Status::Ready);
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
        state.save.records.beginner = Some(5000);
        state.save.game.command(GameCommand::Reveal(40));
        state.lab.pick(0);
        state.lab.drop_in(Some(0));
        state.flag_mode = true;
        state.reset_saved_data();
        assert_eq!(state.save.settings, Settings::default());
        assert_eq!(state.save.records, Records::default());
        assert_eq!(state.save.game.status, Status::Ready);
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
