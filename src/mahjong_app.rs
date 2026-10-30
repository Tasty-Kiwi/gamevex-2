//! Interaction state for the layered Mahjong solitaire cabinet.
use crate::*;

#[derive(Default)]
pub(crate) struct MahjongUi {
    pub feedback: crate::table_feedback::TableFeedback,
    pub selected: Option<usize>,
    pub hint: Option<[usize; 2]>,
    pub reveal_hint: bool,
    pub completion_age: Option<f32>,
    pub pan: Vec2,
    pub message: String,
}

impl Arcade {
    pub(crate) fn request_mahjong(&mut self, layout: mahjong::Layout, seed: u64) {
        self.request_mahjong_run(layout, seed, self.save.mahjong.zen);
    }
    pub(crate) fn request_mahjong_mode(&mut self, zen: bool) {
        if zen != self.save.mahjong.zen {
            self.request_mahjong_run(self.save.mahjong.layout, fresh_seed(), zen);
        }
    }
    fn request_mahjong_run(&mut self, layout: mahjong::Layout, seed: u64, zen: bool) {
        if self.save.mahjong.remaining() < 144 && !self.save.mahjong.won() {
            self.open_modal(Pending::NewMahjong(layout, seed, zen));
        } else {
            self.new_mahjong(layout, seed, zen);
        }
    }
    pub(crate) fn new_mahjong(&mut self, layout: mahjong::Layout, seed: u64, zen: bool) {
        self.save.mahjong = mahjong::Game::new(layout, seed);
        self.save.mahjong.zen = zen;
        self.mahjong_ui = Default::default();
        self.mahjong_ui
            .feedback
            .emit(crate::table_feedback::EffectKind::Shuffle, &[]);
        self.modal = None;
        self.focus = 3;
        self.fractional_ms = 0.0;
        self.last_active = false;
        self.persist();
    }
    pub(crate) fn mahjong_tile(&mut self, index: usize) {
        if !self.save.mahjong.selectable(index) {
            self.mahjong_ui.feedback.emit(
                crate::table_feedback::EffectKind::Invalid,
                &[crate::table_feedback::Anchor::Tile(index)],
            );
            self.mahjong_ui.message = "Blocked: clear above and at least one side.".into();
            return;
        }
        self.mahjong_ui.hint = None;
        if self.mahjong_ui.selected == Some(index) {
            self.mahjong_ui.selected = None;
        } else if let Some(first) = self.mahjong_ui.selected {
            let before_shuffles = self.save.mahjong.shuffles;
            if self.save.mahjong.match_pair(first, index) {
                self.mahjong_ui.feedback.emit(
                    crate::table_feedback::EffectKind::Move,
                    &[
                        crate::table_feedback::Anchor::Tile(first),
                        crate::table_feedback::Anchor::Tile(index),
                    ],
                );
                self.mahjong_ui.selected = None;
                if self.save.mahjong.won() {
                    self.mahjong_ui.completion_age = Some(0.0);
                    self.mahjong_ui.feedback.emit(
                        crate::table_feedback::EffectKind::Win,
                        &[
                            crate::table_feedback::Anchor::Tile(first),
                            crate::table_feedback::Anchor::Tile(index),
                        ],
                    );
                }
                if self.save.mahjong.won() && !self.save.mahjong.assisted && !self.save.mahjong.zen
                {
                    let index = mahjong::Layout::ALL
                        .iter()
                        .position(|l| *l == self.save.mahjong.layout)
                        .unwrap();
                    let elapsed = self.save.mahjong.elapsed_ms.max(1000);
                    if self.save.mahjong_records[index].is_none_or(|t| elapsed < t) {
                        self.save.mahjong_records[index] = Some(elapsed);
                    }
                }
                if self.save.mahjong.shuffles > before_shuffles {
                    self.mahjong_ui.feedback.emit(
                        crate::table_feedback::EffectKind::Shuffle,
                        &[
                            crate::table_feedback::Anchor::Tile(first),
                            crate::table_feedback::Anchor::Tile(index),
                        ],
                    );
                }
                if self.save.mahjong.stuck() && !self.save.mahjong.won() {
                    self.mahjong_ui
                        .feedback
                        .emit(crate::table_feedback::EffectKind::Stuck, &[]);
                }
                self.mahjong_ui.message = if self.save.mahjong.won() {
                    "Board cleared!"
                } else if self.save.mahjong.shuffles > before_shuffles {
                    "Auto-shuffled. Undo also restores the old faces."
                } else if self.save.mahjong.stuck() {
                    "No pairs. Undo, shuffle, or start again."
                } else {
                    "Pair removed."
                }
                .into();
                self.persist();
            } else {
                self.mahjong_ui.selected = Some(index);
                self.mahjong_ui.feedback.emit(
                    crate::table_feedback::EffectKind::Invalid,
                    &[
                        crate::table_feedback::Anchor::Tile(first),
                        crate::table_feedback::Anchor::Tile(index),
                    ],
                );
                self.mahjong_ui.message = "Different faces. New tile selected.".into();
            }
        } else {
            self.mahjong_ui.selected = Some(index);
            self.mahjong_ui.message.clear();
        }
        if self.mahjong_ui.selected == Some(index) {
            self.mahjong_ui.feedback.emit(
                crate::table_feedback::EffectKind::Pick,
                &[crate::table_feedback::Anchor::Tile(index)],
            );
        }
    }
    pub(crate) fn mahjong_hint(&mut self) {
        let previous = self.mahjong_ui.hint;
        self.mahjong_ui.hint = self.save.mahjong.hint();
        if let (Some(old), Some(pair)) = (previous, &mut self.mahjong_ui.hint)
            && old == *pair
        {
            pair.reverse();
        }
        self.mahjong_ui.selected = None;
        if self.mahjong_ui.hint.is_some() {
            if let Some([a, b]) = self.mahjong_ui.hint {
                self.mahjong_ui.feedback.emit(
                    crate::table_feedback::EffectKind::Hint,
                    &[
                        crate::table_feedback::Anchor::Tile(a),
                        crate::table_feedback::Anchor::Tile(b),
                    ],
                );
            }
            self.mahjong_ui.reveal_hint = true;
            self.mahjong_ui.message = "Hint pair. Press again to find the other tile.".into();
        } else {
            self.mahjong_ui
                .feedback
                .emit(crate::table_feedback::EffectKind::Stuck, &[]);
            self.mahjong_ui.message = "No pairs. Undo, shuffle, or start again.".into();
        }
        self.persist();
    }
}

pub(crate) fn keys(s: &mut Arcade, keys: &ButtonInput<KeyCode>, l: &ViewLayout) {
    if s.screen != Screen::Mahjong || s.modal.is_some() {
        return;
    }
    for (key, action) in [
        (KeyCode::KeyH, Action::MahjongHint),
        (KeyCode::KeyU, Action::MahjongUndo),
        (KeyCode::KeyS, Action::MahjongShuffle),
        (KeyCode::KeyR, Action::MahjongRestart),
    ] {
        if keys.just_pressed(key) {
            s.act(action);
        }
    }
    if s.modal.is_some() {
        return;
    }
    let dx = i32::from(keys.just_pressed(KeyCode::ArrowRight))
        - i32::from(keys.just_pressed(KeyCode::ArrowLeft));
    let dy = i32::from(keys.just_pressed(KeyCode::ArrowDown))
        - i32::from(keys.just_pressed(KeyCode::ArrowUp));
    s.mahjong_ui.pan = (s.mahjong_ui.pan + Vec2::new(dx as f32, dy as f32) * 48.0)
        .clamp(Vec2::ZERO, l.mahjong_max_pan);
}
