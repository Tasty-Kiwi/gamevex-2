use crate::{Difficulty, Game, MAX_ELAPSED_MS, Status};
use serde::{Deserialize, Serialize};

/// Maximum accepted serialized save size. This also bounds parsing work on untrusted storage.
pub const MAX_SAVE_BYTES: usize = 256 * 1024;
pub const SAVE_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayPreset {
    VectorCrt,
    PixelCrt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub display_preset: DisplayPreset,
    pub bloom: f32,
    pub scanlines: f32,
    pub crt: bool,
    pub reduced_motion: bool,
    pub volume: f32,
    pub muted: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display_preset: DisplayPreset::VectorCrt,
            bloom: 0.35,
            scanlines: 0.2,
            crt: true,
            reduced_motion: false,
            volume: 0.7,
            muted: false,
        }
    }
}

impl Settings {
    fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("bloom", self.bloom),
            ("scanlines", self.scanlines),
            ("volume", self.volume),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(format!("setting {name} must be between 0 and 1"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Records {
    pub beginner: Option<u64>,
    pub intermediate: Option<u64>,
    pub expert: Option<u64>,
}

impl Records {
    pub fn best(&self, difficulty: Difficulty) -> Option<u64> {
        match difficulty {
            Difficulty::Beginner => self.beginner,
            Difficulty::Intermediate => self.intermediate,
            Difficulty::Expert => self.expert,
        }
    }

    /// Records a winning game if it improves the best time for its difficulty.
    pub fn record(&mut self, game: &Game) -> bool {
        if game.status != Status::Won || game.elapsed_ms > MAX_ELAPSED_MS {
            return false;
        }
        let best = match game.difficulty {
            Difficulty::Beginner => &mut self.beginner,
            Difficulty::Intermediate => &mut self.intermediate,
            Difficulty::Expert => &mut self.expert,
        };
        if best.is_none_or(|old| game.elapsed_ms < old) {
            *best = Some(game.elapsed_ms);
            true
        } else {
            false
        }
    }

    fn validate(&self) -> Result<(), String> {
        if [self.beginner, self.intermediate, self.expert]
            .into_iter()
            .flatten()
            .any(|v| v > MAX_ELAPSED_MS)
        {
            return Err("record time exceeds seven day limit".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    pub version: u32,
    pub settings: Settings,
    pub game: Game,
    pub records: Records,
}

impl Default for Save {
    fn default() -> Self {
        Self::new(0x4756_585f_5255_5354)
    }
}

impl Save {
    pub fn new(seed: u64) -> Self {
        Self {
            version: SAVE_VERSION,
            settings: Settings::default(),
            game: Game::new(Difficulty::Beginner, seed),
            records: Records::default(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version != SAVE_VERSION {
            return Err(format!("unsupported save version {}", self.version));
        }
        self.settings.validate()?;
        self.game.validate()?;
        self.records.validate()?;
        Ok(())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        if json.len() > MAX_SAVE_BYTES {
            return Err("save exceeds 256 KiB limit".into());
        }
        let save: Self =
            serde_json::from_str(json).map_err(|e| format!("invalid save JSON: {e}"))?;
        save.validate()?;
        Ok(save)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        let json =
            serde_json::to_string(self).map_err(|e| format!("could not serialize save: {e}"))?;
        if json.len() > MAX_SAVE_BYTES {
            return Err("save exceeds 256 KiB limit".into());
        }
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_save_round_trips_and_starts_beginner() {
        let save = Save::default();
        assert_eq!(save.game.difficulty, Difficulty::Beginner);
        assert_eq!(save.game.status, Status::Ready);
        assert_eq!(Save::from_json(&save.to_json().unwrap()).unwrap(), save);
    }

    #[test]
    fn rejects_unsupported_version_and_invalid_settings() {
        let mut save = Save::default();
        save.version += 1;
        assert!(save.to_json().unwrap_err().contains("version"));
        let mut save = Save::default();
        save.settings.volume = f32::NAN;
        assert!(save.validate().unwrap_err().contains("volume"));
    }

    #[test]
    fn rejects_oversized_json() {
        assert!(
            Save::from_json(&" ".repeat(MAX_SAVE_BYTES + 1))
                .unwrap_err()
                .contains("256 KiB")
        );
    }

    #[test]
    fn records_only_improving_wins() {
        let mut records = Records::default();
        let mut game = Game::new(Difficulty::Beginner, 1);
        game.status = Status::Playing;
        game.elapsed_ms = 900;
        assert!(!records.record(&game));
        game.status = Status::Won;
        assert!(records.record(&game));
        game.elapsed_ms = 1_000;
        assert!(!records.record(&game));
        game.elapsed_ms = 800;
        assert!(records.record(&game));
        assert_eq!(records.best(Difficulty::Beginner), Some(800));
    }
}
