use std::{collections::HashSet, mem::Discriminant};

use bevy::input::touch::{TouchInput, TouchPhase};

use crate::*;

const DRAG_THRESHOLD: f32 = 8.0;

#[derive(Clone, Copy)]
enum Pan {
    Lobby,
    Mines,
    Sudoku,
    Mahjong,
    Patience,
    Settings,
}

#[derive(PartialEq)]
struct Context {
    screen: Screen,
    modal: Option<Discriminant<Pending>>,
    size: Vec2,
    mines_seed: u64,
    sudoku_seed: u64,
    mahjong_seed: u64,
    patience_seed: u64,
}

impl Context {
    fn current(s: &Arcade, l: &ViewLayout) -> Self {
        Self {
            screen: s.screen,
            modal: s.modal.as_ref().map(std::mem::discriminant),
            size: l.size,
            mines_seed: s.save.minesweeper.game.seed,
            sudoku_seed: s.save.sudoku.game.seed,
            mahjong_seed: s.save.mahjong.seed,
            patience_seed: s.patience_game().seed,
        }
    }
}

struct Gesture {
    id: u64,
    start: Vec2,
    last: Vec2,
    moved: bool,
    pan: Option<Pan>,
    tap: Option<Rect>,
    patience_drag: bool,
    context: Context,
}

/// Winit sends touch separately from mouse input. Keep one owning finger until
/// all contacts lift; a second finger must never become a new press mid-gesture.
#[derive(Default)]
pub(crate) struct TouchPointer {
    contacts: HashSet<u64>,
    gesture: Option<Gesture>,
    lifecycle: Option<u64>,
}

impl TouchPointer {
    pub fn sync_lifecycle(&mut self, s: &mut Arcade, pointer: &mut Option<Vec2>) {
        if self.lifecycle != Some(s.lifecycle) {
            self.reset(s, pointer);
            self.lifecycle = Some(s.lifecycle);
        }
    }

    pub fn active(&self) -> bool {
        !self.contacts.is_empty()
    }

    pub fn cancel(&mut self, s: &mut Arcade, pointer: &mut Option<Vec2>) {
        if self.gesture.take().is_some() {
            s.lab.cancel_drag();
            s.patience_ui.drag = None;
            s.cursor = None;
            *pointer = None;
        }
    }

    pub fn reset(&mut self, s: &mut Arcade, pointer: &mut Option<Vec2>) {
        self.cancel(s, pointer);
        // A hidden/resized page may never receive the old finger's end event.
        self.contacts.clear();
    }

    pub fn handle(
        &mut self,
        event: &TouchInput,
        point: Vec2,
        valid: bool,
        s: &mut Arcade,
        l: &ViewLayout,
        pointer: &mut Option<Vec2>,
    ) {
        if !valid {
            self.reset(s, pointer);
            return;
        }
        if self
            .gesture
            .as_ref()
            .is_some_and(|g| g.context != Context::current(s, l))
        {
            self.reset(s, pointer);
        }
        let first = self.contacts.is_empty();
        if event.phase == TouchPhase::Started {
            self.contacts.insert(event.id);
        }
        if let Some(surface) = rendering::screen_to_surface(
            point,
            l.size,
            if s.save.settings.crt {
                s.save.settings.curvature
            } else {
                0.0
            },
        ) {
            if event.phase == TouchPhase::Started && first {
                *pointer = Some(point);
                s.cursor = Some(surface);
                s.keyboard = false;
                s.audio_started = true;
                let patience_drag =
                    s.modal.is_none() && patience_app::is_family(s.screen) && s.patience_press(l);
                let pan = if s.modal.is_some() || patience_drag {
                    None
                } else if l.lobby_clip.is_some_and(|r| r.contains(surface)) {
                    Some(Pan::Lobby)
                } else if l.board.as_ref().is_some_and(|b| b.clip.contains(surface)) {
                    Some(Pan::Mines)
                } else if l.sudoku.as_ref().is_some_and(|b| b.clip.contains(surface)) {
                    Some(Pan::Sudoku)
                } else if l.mahjong_clip.is_some_and(|r| r.contains(surface)) {
                    Some(Pan::Mahjong)
                } else if l.patience_clip.is_some_and(|r| r.contains(surface)) {
                    Some(Pan::Patience)
                } else if l.settings_clip.is_some_and(|r| r.contains(surface)) {
                    Some(Pan::Settings)
                } else {
                    None
                };
                let tap = l
                    .hits
                    .iter()
                    .rev()
                    .find(|h| h.rect.contains(surface))
                    .map(|h| h.rect);
                self.gesture = Some(Gesture {
                    id: event.id,
                    start: surface,
                    last: surface,
                    moved: false,
                    pan,
                    tap,
                    patience_drag,
                    context: Context::current(s, l),
                });
                // Cards need an immediate pick for dragging; buttons and boards
                // wait for release so swipes/cancelled touches cannot activate them.
                if s.screen == Screen::Cards && s.modal.is_none() && tap.is_none() {
                    pointer_pressed(s, l, MouseButton::Left);
                }
            } else if let Some(g) = self.gesture.as_mut().filter(|g| g.id == event.id) {
                *pointer = Some(point);
                s.cursor = Some(surface);
                if matches!(event.phase, TouchPhase::Moved | TouchPhase::Ended) {
                    let was_moved = g.moved;
                    g.moved |= surface.distance(g.start) > DRAG_THRESHOLD;
                    if g.moved {
                        let delta = surface - if was_moved { g.last } else { g.start };
                        match g.pan {
                            Some(Pan::Lobby) => {
                                s.lobby_scroll =
                                    (s.lobby_scroll - delta.y).clamp(0.0, l.lobby_max_scroll);
                            }
                            Some(Pan::Mines) => s.pan -= delta,
                            Some(Pan::Sudoku) => {
                                s.sudoku_ui.follow = false;
                                s.sudoku_ui.pan =
                                    (s.sudoku_ui.pan - delta).clamp(Vec2::ZERO, l.sudoku_max_pan);
                            }
                            Some(Pan::Mahjong) => {
                                s.mahjong_ui.pan =
                                    (s.mahjong_ui.pan - delta).clamp(Vec2::ZERO, l.mahjong_max_pan);
                            }
                            Some(Pan::Patience) => {
                                s.patience_ui.pan = (s.patience_ui.pan - delta)
                                    .clamp(Vec2::ZERO, l.patience_max_pan);
                            }
                            Some(Pan::Settings) => {
                                s.settings_scroll =
                                    (s.settings_scroll - delta.y).clamp(0.0, l.settings_max_scroll);
                            }
                            None => {}
                        }
                        if g.patience_drag {
                            s.patience_drag_update();
                        }
                    }
                    g.last = surface;
                    if event.phase == TouchPhase::Ended {
                        if g.patience_drag {
                            s.patience_release(l);
                        } else if s.lab.drag.is_some() {
                            let slot = l.slots.iter().position(|r| r.contains(surface));
                            s.lab.drop_in(slot);
                        } else if !g.moved && g.tap.is_some_and(|r| r.contains(surface)) {
                            pointer_pressed(s, l, MouseButton::Left);
                        }
                    }
                }
            }
        } else {
            self.cancel(s, pointer);
        }
        if matches!(event.phase, TouchPhase::Ended | TouchPhase::Canceled) {
            self.contacts.remove(&event.id);
            if self.gesture.as_ref().is_some_and(|g| g.id == event.id) {
                self.cancel(s, pointer);
            }
        }
    }
}
