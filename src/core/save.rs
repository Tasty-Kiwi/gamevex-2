use crate::{Difficulty, Game, MAX_ELAPSED_MS, Status};
use serde::{Deserialize, Serialize};

/// Maximum accepted serialized save size. This also bounds parsing work on untrusted storage.
pub const MAX_SAVE_BYTES: usize = 256 * 1024;
pub const SAVE_VERSION: u32 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayPreset {
    VectorCrt,
    PixelCrt,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    #[default]
    Cyan,
    Amber,
}

fn default_curvature() -> f32 {
    0.5
}
fn default_vignette() -> f32 {
    0.45
}
fn default_chromatic_aberration() -> f32 {
    0.35
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub display_preset: DisplayPreset,
    #[serde(default)]
    pub color_scheme: ColorScheme,
    #[serde(default = "default_curvature")]
    pub curvature: f32,
    #[serde(default = "default_vignette")]
    pub vignette: f32,
    #[serde(default = "default_chromatic_aberration")]
    pub chromatic_aberration: f32,
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
            color_scheme: ColorScheme::Cyan,
            curvature: default_curvature(),
            vignette: default_vignette(),
            chromatic_aberration: default_chromatic_aberration(),
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
            ("curvature", self.curvature),
            ("vignette", self.vignette),
            ("chromatic_aberration", self.chromatic_aberration),
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
pub struct MinesweeperSave {
    pub game: Game,
    pub records: Records,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    pub version: u32,
    pub settings: Settings,
    pub minesweeper: MinesweeperSave,
    pub blackjack: crate::blackjack::Blackjack,
    pub baccarat: crate::baccarat::Baccarat,
    pub sudoku: crate::sudoku::Session,
    pub mahjong: crate::mahjong::Game,
    pub mahjong_records: [Option<u64>; 3],
    #[serde(default)]
    pub patience_records: [Option<u64>; 7],
    pub solitaire: crate::patience::Game,
    pub freecell: crate::patience::Game,
    pub spider: crate::patience::Game,
}

#[derive(Deserialize)]
struct LegacySave {
    settings: Settings,
    game: Game,
    records: Records,
}

#[derive(Deserialize)]
struct LegacyV2 {
    settings: Settings,
    minesweeper: MinesweeperSave,
    blackjack: crate::blackjack::Blackjack,
}

#[derive(Deserialize)]
struct LegacyV3 {
    settings: Settings,
    minesweeper: MinesweeperSave,
    blackjack: crate::blackjack::Blackjack,
    baccarat: crate::baccarat::Baccarat,
}

#[derive(Deserialize)]
struct LegacyV4 {
    settings: Settings,
    minesweeper: MinesweeperSave,
    blackjack: crate::blackjack::Blackjack,
    baccarat: crate::baccarat::Baccarat,
    sudoku: crate::sudoku::Session,
}

#[derive(Deserialize)]
struct LegacyV5 {
    settings: Settings,
    minesweeper: MinesweeperSave,
    blackjack: crate::blackjack::Blackjack,
    baccarat: crate::baccarat::Baccarat,
    sudoku: crate::sudoku::Session,
    mahjong: crate::mahjong::Game,
    mahjong_records: [Option<u64>; 3],
}

#[derive(Deserialize)]
struct LegacyV6 {
    settings: Settings,
    minesweeper: MinesweeperSave,
    blackjack: crate::blackjack::Blackjack,
    baccarat: crate::baccarat::Baccarat,
    sudoku: crate::sudoku::Session,
    mahjong: crate::mahjong::Game,
    mahjong_records: [Option<u64>; 3],
    patience_records: [Option<u64>; 6],
    solitaire: crate::patience::Game,
    freecell: crate::patience::Game,
    spider: crate::patience::Game,
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
            minesweeper: MinesweeperSave {
                game: Game::new(Difficulty::Beginner, seed),
                records: Records::default(),
            },
            blackjack: crate::blackjack::Blackjack::new(seed ^ 0x424c_4143_4b4a_4143),
            baccarat: crate::baccarat::Baccarat::new(seed ^ 0x4241_4343_4152_4154),
            sudoku: crate::sudoku::Session::new(seed ^ 0x5355_444f_4b55),
            mahjong_records: [None; 3],
            mahjong: crate::mahjong::Game::new(
                crate::mahjong::Layout::Turtle,
                seed ^ 0x004d_4148_4a4f_4e47,
            ),
            patience_records: [None; 7],
            solitaire: crate::patience::Game::new(
                crate::patience::Variant::Klondike1,
                seed ^ 0x534f_4c49_5441_4952,
            ),
            freecell: crate::patience::Game::new(
                crate::patience::Variant::FreeCell,
                seed ^ 0x4652_4545_4345_4c4c,
            ),
            spider: crate::patience::Game::new(
                crate::patience::Variant::Spider1,
                seed ^ 0x5350_4944_4552_0001,
            ),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version != SAVE_VERSION {
            return Err(format!("unsupported save version {}", self.version));
        }
        self.settings.validate()?;
        self.minesweeper.game.validate()?;
        self.minesweeper.records.validate()?;
        self.blackjack.validate()?;
        self.baccarat.validate()?;
        self.sudoku.validate()?;
        self.mahjong.validate()?;
        self.solitaire.validate()?;
        if self
            .patience_records
            .iter()
            .flatten()
            .any(|t| *t == 0 || *t > MAX_ELAPSED_MS)
        {
            return Err("invalid patience record".into());
        }
        self.freecell.validate()?;
        self.spider.validate()?;
        if !matches!(
            self.solitaire.variant,
            crate::patience::Variant::Klondike1 | crate::patience::Variant::Klondike3
        ) || !self.freecell.variant.is_freecell()
            || !matches!(
                self.spider.variant,
                crate::patience::Variant::Spider1
                    | crate::patience::Variant::Spider2
                    | crate::patience::Variant::Spider4
            )
        {
            return Err("patience save has an unexpected variant".into());
        }
        if self
            .mahjong_records
            .iter()
            .flatten()
            .any(|t| *t == 0 || *t > MAX_ELAPSED_MS)
        {
            return Err("invalid Mahjong record".into());
        }
        Ok(())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        if json.len() > MAX_SAVE_BYTES {
            return Err("save exceeds 256 KiB limit".into());
        }
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("invalid save JSON: {e}"))?;
        let version = value
            .get("version")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "missing or invalid save version".to_owned())?;
        let save = match version {
            1 => {
                let old: LegacySave = serde_json::from_value(value)
                    .map_err(|e| format!("invalid version-1 save: {e}"))?;
                let mut migrated = Self::new(old.game.seed);
                migrated.settings = old.settings;
                migrated.minesweeper = MinesweeperSave {
                    game: old.game,
                    records: old.records,
                };
                migrated
            }
            2 => {
                let old: LegacyV2 = serde_json::from_value(value)
                    .map_err(|e| format!("invalid version-2 save: {e}"))?;
                let mut migrated = Self::new(old.minesweeper.game.seed);
                migrated.settings = old.settings;
                migrated.minesweeper = old.minesweeper;
                migrated.blackjack = old.blackjack;
                migrated
            }
            3 => {
                let old: LegacyV3 = serde_json::from_value(value)
                    .map_err(|e| format!("invalid version-3 save: {e}"))?;
                let mut migrated = Self::new(old.minesweeper.game.seed);
                migrated.settings = old.settings;
                migrated.minesweeper = old.minesweeper;
                migrated.blackjack = old.blackjack;
                migrated.baccarat = old.baccarat;
                migrated
            }
            4 => {
                let old: LegacyV4 = serde_json::from_value(value)
                    .map_err(|e| format!("invalid version-4 save: {e}"))?;
                let mut migrated = Self::new(old.minesweeper.game.seed);
                migrated.settings = old.settings;
                migrated.minesweeper = old.minesweeper;
                migrated.blackjack = old.blackjack;
                migrated.baccarat = old.baccarat;
                migrated.sudoku = old.sudoku;
                migrated
            }
            5 => {
                let old: LegacyV5 = serde_json::from_value(value)
                    .map_err(|e| format!("invalid version-5 save: {e}"))?;
                let mut migrated = Self::new(old.minesweeper.game.seed);
                migrated.settings = old.settings;
                migrated.minesweeper = old.minesweeper;
                migrated.blackjack = old.blackjack;
                migrated.baccarat = old.baccarat;
                migrated.sudoku = old.sudoku;
                migrated.mahjong = old.mahjong;
                migrated.mahjong_records = old.mahjong_records;
                migrated
            }
            6 => {
                let old: LegacyV6 = serde_json::from_value(value)
                    .map_err(|e| format!("invalid version-6 save: {e}"))?;
                let mut migrated = Self::new(old.minesweeper.game.seed);
                migrated.settings = old.settings;
                migrated.minesweeper = old.minesweeper;
                migrated.blackjack = old.blackjack;
                migrated.baccarat = old.baccarat;
                migrated.sudoku = old.sudoku;
                migrated.mahjong = old.mahjong;
                migrated.mahjong_records = old.mahjong_records;
                migrated.patience_records[..6].copy_from_slice(&old.patience_records);
                migrated.solitaire = old.solitaire;
                migrated.freecell = old.freecell;
                migrated.spider = old.spider;
                migrated
            }
            7 => serde_json::from_value(value).map_err(|e| format!("invalid save JSON: {e}"))?,
            _ => return Err(format!("unsupported save version {version}")),
        };
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
    #[test]
    fn all_patience_modes_persist_in_their_cabinet() {
        for variant in crate::patience::Variant::ALL {
            let mut save = Save::new(42);
            let game = crate::patience::Game::new(variant, 99);
            match variant {
                crate::patience::Variant::Klondike1 | crate::patience::Variant::Klondike3 => {
                    save.solitaire = game
                }
                variant if variant.is_freecell() => save.freecell = game,
                _ => save.spider = game,
            }
            assert_eq!(Save::from_json(&save.to_json().unwrap()).unwrap(), save);
        }
    }
    use super::*;

    #[test]
    fn default_save_round_trips_and_starts_beginner() {
        let save = Save::default();
        assert_eq!(save.minesweeper.game.difficulty, Difficulty::Beginner);
        assert_eq!(save.minesweeper.game.status, Status::Ready);
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
    fn previous_v1_saves_load_with_monitor_and_palette_defaults() {
        let seed_save = Save::default();
        let mut old = serde_json::json!({"version":1,"settings":seed_save.settings,
            "game":seed_save.minesweeper.game,"records":seed_save.minesweeper.records});
        for key in [
            "color_scheme",
            "curvature",
            "vignette",
            "chromatic_aberration",
        ] {
            old["settings"].as_object_mut().unwrap().remove(key);
        }
        let loaded = Save::from_json(&old.to_string()).unwrap();
        assert_eq!(loaded.settings, Settings::default());
        let mut amber = loaded;
        amber.settings.color_scheme = ColorScheme::Amber;
        amber.settings.curvature = 0.8;
        amber.settings.vignette = 0.6;
        amber.settings.chromatic_aberration = 0.8;
        assert_eq!(Save::from_json(&amber.to_json().unwrap()).unwrap(), amber);
        amber.settings.curvature = 1.1;
        assert!(amber.validate().is_err());
        amber.settings.curvature = 0.5;
        amber.settings.vignette = -0.1;
        assert!(amber.validate().is_err());
        amber.settings.vignette = 0.45;
        for invalid in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
            amber.settings.chromatic_aberration = invalid;
            assert!(amber.validate().is_err());
        }
    }

    #[test]
    fn migration_preserves_progress_and_v2_keeps_both_cabinets() {
        let mut save = Save::default();
        save.minesweeper.game.command(crate::Command::Reveal(40));
        save.minesweeper.records.beginner = Some(1234);
        save.settings.color_scheme = ColorScheme::Amber;
        let legacy = serde_json::json!({"version":1,"settings":save.settings,
            "game":save.minesweeper.game,"records":save.minesweeper.records});
        let mut migrated = Save::from_json(&legacy.to_string()).unwrap();
        assert_eq!(migrated.minesweeper, save.minesweeper);
        assert_eq!(migrated.settings, save.settings);
        assert_eq!(migrated.blackjack.balance, 1000);
        migrated.blackjack.command(crate::blackjack::Command::Deal);
        let restored = Save::from_json(&migrated.to_json().unwrap()).unwrap();
        assert_eq!(restored, migrated);
        assert_eq!(restored.version, SAVE_VERSION);
        let mut corrupt = serde_json::to_value(restored).unwrap();
        corrupt["blackjack"]["hand_count"] = 0.into();
        assert!(Save::from_json(&corrupt.to_string()).is_err());
    }

    #[test]
    fn version_two_migration_preserves_both_cabinets_and_initializes_baccarat() {
        let mut old = Save::new(44);
        old.blackjack.command(crate::blackjack::Command::Deal);
        old.minesweeper.game.command(crate::Command::Reveal(40));
        old.settings.color_scheme = ColorScheme::Amber;
        let value = serde_json::json!({"version":2,"settings":old.settings,
            "minesweeper":old.minesweeper,"blackjack":old.blackjack});
        let mut migrated = Save::from_json(&value.to_string()).unwrap();
        assert_eq!(migrated.settings, old.settings);
        assert_eq!(migrated.minesweeper, old.minesweeper);
        assert_eq!(migrated.blackjack, old.blackjack);
        assert_eq!(migrated.baccarat.balance, 1000);
        migrated.baccarat.command(crate::baccarat::Command::Select(
            crate::baccarat::BetSide::Banker,
        ));
        migrated.baccarat.command(crate::baccarat::Command::Deal);
        assert_eq!(
            Save::from_json(&migrated.to_json().unwrap()).unwrap(),
            migrated
        );
        let mut corrupt = serde_json::to_value(migrated).unwrap();
        corrupt["baccarat"]["bet"] = 0.into();
        assert!(Save::from_json(&corrupt.to_string()).is_err());
    }

    #[test]
    fn version_three_migration_preserves_every_existing_cabinet_and_sudoku_round_trips() {
        let mut old = Save::new(55);
        old.minesweeper.game.command(crate::Command::Reveal(40));
        old.blackjack.command(crate::blackjack::Command::Deal);
        old.baccarat.command(crate::baccarat::Command::Select(
            crate::baccarat::BetSide::Tie,
        ));
        old.baccarat.command(crate::baccarat::Command::Deal);
        old.settings.color_scheme = ColorScheme::Amber;
        let value = serde_json::json!({"version":3,"settings":old.settings,
            "minesweeper":old.minesweeper,"blackjack":old.blackjack,"baccarat":old.baccarat});
        let mut migrated = Save::from_json(&value.to_string()).unwrap();
        assert_eq!(migrated.version, SAVE_VERSION);
        assert_eq!(migrated.settings, old.settings);
        assert_eq!(migrated.minesweeper, old.minesweeper);
        assert_eq!(migrated.blackjack, old.blackjack);
        assert_eq!(migrated.baccarat, old.baccarat);
        let mut history = crate::sudoku::History::default();
        assert!(
            migrated
                .sudoku
                .command(crate::sudoku::Command::Hint, &mut history)
        );
        let empty = migrated
            .sudoku
            .game
            .board
            .iter()
            .position(|v| *v == 0)
            .unwrap();
        assert!(migrated.sudoku.command(
            crate::sudoku::Command::Enter {
                index: empty,
                value: 2,
                pencil: true
            },
            &mut history
        ));
        migrated.sudoku.game.advance_time(1234);
        let json = migrated.to_json().unwrap();
        assert_eq!(Save::from_json(&json).unwrap(), migrated);
        assert!(!json.contains("snapshots"));
        let mut corrupt = serde_json::to_value(migrated).unwrap();
        corrupt["sudoku"]["game"]["notes"][empty] = 1_000_000.into();
        assert!(Save::from_json(&corrupt.to_string()).is_err());
    }

    #[test]
    fn version_four_migration_preserves_all_existing_state_and_round_trips() {
        let mut old = Save::new(0x1234_5678);
        old.minesweeper.game.command(crate::Command::Reveal(40));
        old.minesweeper.records.beginner = Some(4321);
        old.blackjack.command(crate::blackjack::Command::Deal);
        old.baccarat.command(crate::baccarat::Command::Select(
            crate::baccarat::BetSide::Player,
        ));
        old.baccarat.command(crate::baccarat::Command::Deal);
        let mut history = crate::sudoku::History::default();
        old.sudoku
            .command(crate::sudoku::Command::Hint, &mut history);
        old.sudoku.game.advance_time(987);
        old.settings.color_scheme = ColorScheme::Amber;
        old.settings.volume = 0.23;

        let value = serde_json::json!({"version":4,"settings":old.settings,
            "minesweeper":old.minesweeper,"blackjack":old.blackjack,
            "baccarat":old.baccarat,"sudoku":old.sudoku});
        let migrated = Save::from_json(&value.to_string()).unwrap();
        assert_eq!(migrated.version, SAVE_VERSION);
        assert_eq!(migrated.settings, old.settings);
        assert_eq!(migrated.minesweeper, old.minesweeper);
        assert_eq!(migrated.blackjack, old.blackjack);
        assert_eq!(migrated.baccarat, old.baccarat);
        assert_eq!(migrated.sudoku, old.sudoku);
        assert_eq!(
            migrated.mahjong,
            Save::new(old.minesweeper.game.seed).mahjong
        );
        assert_eq!(
            Save::from_json(&migrated.to_json().unwrap()).unwrap(),
            migrated
        );
    }

    #[test]
    fn current_schema_requires_mahjong_and_new_save_uses_turtle_default() {
        let save = Save::default();
        assert_eq!(save.mahjong, Save::new(0x4756_585f_5255_5354).mahjong);
        let mut value = serde_json::to_value(&save).unwrap();
        value.as_object_mut().unwrap().remove("mahjong");
        assert!(Save::from_json(&value.to_string()).is_err());
    }

    #[test]
    fn version_five_migration_preserves_every_legacy_cabinet_and_zen_history() {
        let mut old = Save::new(0x1234_5678);
        old.minesweeper.game.command(crate::Command::Reveal(40));
        old.minesweeper.records.beginner = Some(4321);
        old.blackjack.command(crate::blackjack::Command::Deal);
        old.baccarat.command(crate::baccarat::Command::Select(
            crate::baccarat::BetSide::Player,
        ));
        old.baccarat.command(crate::baccarat::Command::Deal);
        let mut history = crate::sudoku::History::default();
        old.sudoku
            .command(crate::sudoku::Command::Hint, &mut history);
        old.sudoku.game.advance_time(987);
        old.mahjong.zen = true;
        let pair = old.mahjong.available_pairs()[0];
        assert!(old.mahjong.match_pair(pair[0], pair[1]));
        old.mahjong_records = [Some(1200), Some(1300), Some(1400)];
        old.settings.volume = 0.23;

        let value = serde_json::json!({"version":5,"settings":old.settings,
            "minesweeper":old.minesweeper,"blackjack":old.blackjack,
            "baccarat":old.baccarat,"sudoku":old.sudoku,"mahjong":old.mahjong,
            "mahjong_records":old.mahjong_records});
        let migrated = Save::from_json(&value.to_string()).unwrap();
        assert_eq!(migrated.version, SAVE_VERSION);
        assert_eq!(migrated.settings, old.settings);
        assert_eq!(migrated.minesweeper, old.minesweeper);
        assert_eq!(migrated.blackjack, old.blackjack);
        assert_eq!(migrated.baccarat, old.baccarat);
        assert_eq!(migrated.sudoku, old.sudoku);
        assert_eq!(migrated.mahjong, old.mahjong);
        assert!(migrated.mahjong.zen);
        assert_eq!(migrated.mahjong_records, old.mahjong_records);
        assert_eq!(
            migrated.solitaire,
            Save::new(old.minesweeper.game.seed).solitaire
        );
        assert_eq!(
            migrated.freecell,
            Save::new(old.minesweeper.game.seed).freecell
        );
        assert_eq!(migrated.spider, Save::new(old.minesweeper.game.seed).spider);
        assert_eq!(
            Save::from_json(&migrated.to_json().unwrap()).unwrap(),
            migrated
        );
    }

    #[test]
    fn version_six_requires_all_patience_saves_and_round_trips() {
        let save = Save::default();
        assert_eq!(save.solitaire.variant, crate::patience::Variant::Klondike1);
        assert_eq!(save.freecell.variant, crate::patience::Variant::FreeCell);
        assert_eq!(save.spider.variant, crate::patience::Variant::Spider1);
        assert_eq!(Save::from_json(&save.to_json().unwrap()).unwrap(), save);
        for field in ["solitaire", "freecell", "spider"] {
            let mut value = serde_json::to_value(&save).unwrap();
            value.as_object_mut().unwrap().remove(field);
            assert!(Save::from_json(&value.to_string()).is_err(), "{field}");
        }
    }

    #[test]
    fn version_six_migration_preserves_cabinets_and_appends_relaxed_record_slot() {
        let mut old = Save::new(61);
        old.settings.color_scheme = ColorScheme::Amber;
        old.solitaire.advance_time(1234);
        old.solitaire.deal();
        old.freecell = crate::patience::Game::new(crate::patience::Variant::FreeCell, 62);
        let top = old.freecell.tableau[0].len() - 1;
        old.freecell.move_cards(
            crate::patience::Pile::Tableau(0),
            top,
            crate::patience::Pile::Cell(0),
        );
        old.spider = crate::patience::Game::new(crate::patience::Variant::Spider4, 63);
        old.patience_records[..6].copy_from_slice(&[
            Some(11),
            Some(22),
            Some(33),
            Some(44),
            Some(55),
            Some(66),
        ]);
        let mut value = serde_json::to_value(&old).unwrap();
        value["version"] = 6.into();
        value["patience_records"]
            .as_array_mut()
            .unwrap()
            .truncate(6);
        let migrated = Save::from_json(&value.to_string()).unwrap();
        assert_eq!(migrated.version, SAVE_VERSION);
        assert_eq!(migrated.settings, old.settings);
        assert_eq!(migrated.solitaire, old.solitaire);
        assert_eq!(migrated.freecell, old.freecell);
        assert_eq!(migrated.spider, old.spider);
        assert_eq!(&migrated.patience_records[..6], &old.patience_records[..6]);
        assert_eq!(migrated.patience_records[6], None);
        assert_eq!(
            Save::from_json(&migrated.to_json().unwrap()).unwrap(),
            migrated
        );
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
