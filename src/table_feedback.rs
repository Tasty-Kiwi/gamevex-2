//! Bounded, transient feedback shared by the patience and tile tables.
use crate::casino_audio::SoundCue;
use gamevex_core::patience::Pile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectKind {
    Pick,
    Move,
    Deal,
    Undo,
    Hint,
    Shuffle,
    Invalid,
    Run,
    Win,
    Stuck,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Pile(Pile),
    Tile(usize),
}
pub struct TableEvent {
    pub kind: EffectKind,
    pub age: f32,
    pub anchors: Vec<Anchor>,
}
impl TableEvent {
    pub fn duration(&self) -> f32 {
        match self.kind {
            EffectKind::Win => 2.2,
            EffectKind::Run => 1.3,
            EffectKind::Pick | EffectKind::Invalid => 0.4,
            _ => 0.8,
        }
    }
}
#[derive(Default)]
pub struct TableFeedback {
    pub events: Vec<TableEvent>,
    sounds: Vec<SoundCue>,
}
impl TableFeedback {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn emit(&mut self, kind: EffectKind, anchors: &[Anchor]) {
        if kind == EffectKind::Win {
            self.clear();
        } else if matches!(kind, EffectKind::Move | EffectKind::Deal) {
            self.events
                .retain(|e| matches!(e.kind, EffectKind::Move | EffectKind::Run));
        }
        // Replace rapid repeated picks/errors; keep simultaneous move and run effects.
        if matches!(
            kind,
            EffectKind::Pick | EffectKind::Invalid | EffectKind::Hint
        ) {
            self.events.retain(|event| event.kind != kind);
        }
        if self.events.len() >= 12 {
            self.events.remove(0);
        }
        self.events.push(TableEvent {
            kind,
            age: 0.0,
            anchors: anchors.iter().copied().take(12).collect(),
        });
        let tile = anchors.iter().any(|a| matches!(a, Anchor::Tile(_)));
        use EffectKind as K;
        let cue = match kind {
            K::Pick if tile => SoundCue::TilePick,
            K::Move if tile => SoundCue::TileMatch,
            K::Pick => SoundCue::TablePick,
            K::Move => SoundCue::TableMove,
            K::Deal => SoundCue::TableDeal,
            K::Undo => SoundCue::TableUndo,
            K::Hint => SoundCue::TableHint,
            K::Shuffle => SoundCue::TableShuffle,
            K::Invalid | K::Stuck => SoundCue::TableInvalid,
            K::Run => SoundCue::TableRun,
            K::Win => SoundCue::TableWin,
        };
        if self.sounds.len() < 8 && !self.sounds.contains(&cue) {
            self.sounds.push(cue);
        }
    }
    pub fn tick(&mut self, dt: f32) {
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        for event in &mut self.events {
            event.age = (event.age + dt).min(event.duration());
        }
        self.events.retain(|e| e.age < e.duration());
    }
    pub fn take_sounds(&mut self) -> Vec<SoundCue> {
        std::mem::take(&mut self.sounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rapid_actions_are_bounded_and_expire_without_replaying_audio() {
        let mut f = TableFeedback::default();
        for _ in 0..100 {
            f.emit(EffectKind::Move, &[Anchor::Tile(1), Anchor::Tile(2)]);
        }
        assert_eq!(f.events.len(), 12);
        assert_eq!(f.take_sounds(), vec![SoundCue::TileMatch]);
        assert!(f.take_sounds().is_empty());
        f.tick(f32::NAN);
        assert_eq!(f.events[0].age, 0.0);
        for _ in 0..30 {
            f.tick(0.1);
        }
        assert!(f.events.is_empty());
    }
    #[test]
    fn expiry_uses_elapsed_seconds_even_when_frames_are_slow() {
        let mut f = TableFeedback::default();
        f.emit(EffectKind::Undo, &[]);
        f.tick(0.6);
        assert!((f.events[0].age - 0.6).abs() < 0.001);
        f.tick(0.6);
        assert!(f.events.is_empty());
        f.emit(EffectKind::Win, &[]);
        f.tick(5.0);
        assert!(f.events.is_empty());
        f.emit(EffectKind::Move, &[]);
        f.tick(-1.0);
        f.tick(f32::NAN);
        assert_eq!(f.events[0].age, 0.0);
    }

    #[test]
    fn different_table_actions_have_distinct_audio_and_clear_drops_pending_events() {
        let mut f = TableFeedback::default();
        f.emit(EffectKind::Pick, &[Anchor::Tile(1)]);
        f.emit(EffectKind::Pick, &[Anchor::Pile(Pile::Cell(0))]);
        assert_eq!(f.events.len(), 1);
        assert_eq!(
            f.take_sounds(),
            vec![SoundCue::TilePick, SoundCue::TablePick]
        );
        f.emit(EffectKind::Win, &[]);
        f.clear();
        assert!(f.events.is_empty() && f.take_sounds().is_empty());
    }
}
