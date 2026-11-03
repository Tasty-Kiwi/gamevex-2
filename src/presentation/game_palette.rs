//! Authored palettes for game presentation across all themes.
//!
//! Replaces hardcoded per-game colors (felt, cards, chips, roulette wheel, etc.)
//! with explicit roles tailored for:
//! - Cyan (cool dark neon arcade)
//! - Amber (warm phosphor charcoal/cream/amber/coral)
//! - Emerald (forest/mint/emerald/coral)
//! - Violet (plum/lavender/violet/turquoise)
//! - Ice (navy/silver/pale-blue/rose)
//! - Paper (light ivory/charcoal/burgundy/forest-green)

use bevy::prelude::Color;
use gamevex_core::{ColorScheme, cards::Suit};

/// Authored card styling for face, back, outline, suits, and dimming overlay.
#[derive(Clone, Copy, Debug)]
pub struct CardPalette {
    pub face_bg: Color,
    pub shared_face_bg: Color,
    pub back_bg: Color,
    pub shadow: Color,
    pub border_normal: Color,
    pub border_selected: Color,
    pub border_legal: Color,
    /// Hearts and diamonds share the red-suit role in every theme.
    pub red_suit: Color,
    /// Spades and clubs share the black-suit role in every theme.
    pub black_suit: Color,
    /// Overlay wash applied to dimmed/inactive cards.
    pub card_dim: Color,
}

impl CardPalette {
    pub const CYAN: Self = Self {
        face_bg: Color::srgb(0.025, 0.075, 0.095),
        shared_face_bg: Color::srgb(0.025, 0.055, 0.095),
        back_bg: Color::srgb(0.035, 0.12, 0.15),
        shadow: Color::srgb(0.0, 0.01, 0.025),
        border_normal: Color::srgb(0.0, 0.83, 1.0),
        border_selected: Color::srgb(1.0, 0.0, 0.67),
        border_legal: Color::srgb(0.0, 1.0, 0.72),
        red_suit: Color::srgb(1.0, 0.0, 0.67),
        black_suit: Color::srgb(0.0, 0.83, 1.0),
        card_dim: Color::srgba(0.008, 0.016, 0.031, 0.60),
    };

    pub const AMBER: Self = Self {
        face_bg: Color::srgb(0.070, 0.052, 0.032),
        shared_face_bg: Color::srgb(0.070, 0.052, 0.032),
        back_bg: Color::srgb(0.10, 0.075, 0.040),
        shadow: Color::srgb(0.020, 0.015, 0.008),
        border_normal: Color::srgb(1.0, 0.73, 0.20),
        border_selected: Color::srgb(1.0, 0.88, 0.52),
        border_legal: Color::srgb(0.18, 0.84, 0.52),
        red_suit: Color::srgb(0.94, 0.32, 0.22),
        black_suit: Color::srgb(0.96, 0.90, 0.78),
        card_dim: Color::srgba(0.030, 0.020, 0.010, 0.60),
    };

    pub const EMERALD: Self = Self {
        face_bg: Color::srgb(0.016, 0.042, 0.028),
        shared_face_bg: Color::srgb(0.016, 0.042, 0.028),
        back_bg: Color::srgb(0.024, 0.068, 0.042),
        shadow: Color::srgb(0.004, 0.012, 0.008),
        border_normal: Color::srgb(0.18, 0.85, 0.48),
        border_selected: Color::srgb(0.55, 1.0, 0.78),
        border_legal: Color::srgb(0.40, 0.98, 0.72),
        red_suit: Color::srgb(0.96, 0.38, 0.28),
        black_suit: Color::srgb(0.88, 0.98, 0.92),
        card_dim: Color::srgba(0.010, 0.028, 0.018, 0.60),
    };

    pub const VIOLET: Self = Self {
        face_bg: Color::srgb(0.045, 0.020, 0.055),
        shared_face_bg: Color::srgb(0.045, 0.020, 0.055),
        back_bg: Color::srgb(0.075, 0.032, 0.095),
        shadow: Color::srgb(0.012, 0.005, 0.018),
        border_normal: Color::srgb(0.72, 0.35, 0.98),
        border_selected: Color::srgb(0.90, 0.68, 1.0),
        border_legal: Color::srgb(0.15, 0.90, 0.85),
        red_suit: Color::srgb(0.96, 0.32, 0.52),
        black_suit: Color::srgb(0.94, 0.90, 0.98),
        card_dim: Color::srgba(0.025, 0.010, 0.030, 0.60),
    };

    pub const ICE: Self = Self {
        face_bg: Color::srgb(0.016, 0.030, 0.052),
        shared_face_bg: Color::srgb(0.016, 0.030, 0.052),
        back_bg: Color::srgb(0.028, 0.052, 0.088),
        shadow: Color::srgb(0.004, 0.008, 0.018),
        border_normal: Color::srgb(0.45, 0.80, 1.0),
        border_selected: Color::srgb(0.85, 0.94, 1.0),
        border_legal: Color::srgb(0.30, 0.95, 0.85),
        red_suit: Color::srgb(0.96, 0.38, 0.50),
        black_suit: Color::srgb(0.90, 0.95, 1.0),
        card_dim: Color::srgba(0.010, 0.018, 0.035, 0.60),
    };

    pub const PAPER: Self = Self {
        face_bg: Color::srgb(0.96, 0.95, 0.91),
        shared_face_bg: Color::srgb(0.96, 0.95, 0.91),
        back_bg: Color::srgb(0.86, 0.83, 0.76),
        shadow: Color::srgb(0.68, 0.66, 0.60),
        border_normal: Color::srgb(0.24, 0.24, 0.26),
        border_selected: Color::srgb(0.72, 0.14, 0.22),
        border_legal: Color::srgb(0.12, 0.52, 0.24),
        red_suit: Color::srgb(0.68, 0.12, 0.20),
        black_suit: Color::srgb(0.12, 0.12, 0.14),
        card_dim: Color::srgba(0.92, 0.90, 0.85, 0.65),
    };

    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &Self::CYAN,
            ColorScheme::Amber => &Self::AMBER,
            ColorScheme::Emerald => &Self::EMERALD,
            ColorScheme::Violet => &Self::VIOLET,
            ColorScheme::Ice => &Self::ICE,
            ColorScheme::Paper => &Self::PAPER,
        }
    }

    #[must_use]
    pub fn suit_color(&self, suit: Suit) -> Color {
        if suit.is_red() {
            self.red_suit
        } else {
            self.black_suit
        }
    }
}

/// Authored table surface felt and background colors.
#[derive(Clone, Copy, Debug)]
pub struct TablePalette {
    pub casino_felt: Color,
    pub patience_felt: Color,
    pub round_surface: Color,
    pub round_inset: Color,
    pub seat_active_bg: Color,
}

impl TablePalette {
    pub const CYAN: Self = Self {
        casino_felt: Color::srgb(0.008, 0.045, 0.041),
        patience_felt: Color::srgb(0.012, 0.055, 0.047),
        round_surface: Color::srgb(0.008, 0.018, 0.032),
        round_inset: Color::srgb(0.012, 0.028, 0.046),
        seat_active_bg: Color::srgb(0.035, 0.09, 0.13),
    };

    pub const AMBER: Self = Self {
        casino_felt: Color::srgb(0.042, 0.032, 0.020),
        patience_felt: Color::srgb(0.045, 0.034, 0.022),
        round_surface: Color::srgb(0.032, 0.024, 0.016),
        round_inset: Color::srgb(0.044, 0.034, 0.022),
        seat_active_bg: Color::srgba(0.85, 0.55, 0.15, 0.14),
    };

    pub const EMERALD: Self = Self {
        casino_felt: Color::srgb(0.010, 0.048, 0.028),
        patience_felt: Color::srgb(0.012, 0.056, 0.034),
        round_surface: Color::srgb(0.008, 0.028, 0.018),
        round_inset: Color::srgb(0.014, 0.040, 0.024),
        seat_active_bg: Color::srgba(0.18, 0.85, 0.48, 0.16),
    };

    pub const VIOLET: Self = Self {
        casino_felt: Color::srgb(0.038, 0.015, 0.048),
        patience_felt: Color::srgb(0.045, 0.018, 0.058),
        round_surface: Color::srgb(0.026, 0.010, 0.034),
        round_inset: Color::srgb(0.035, 0.014, 0.046),
        seat_active_bg: Color::srgba(0.72, 0.35, 0.98, 0.16),
    };

    pub const ICE: Self = Self {
        casino_felt: Color::srgb(0.010, 0.024, 0.048),
        patience_felt: Color::srgb(0.014, 0.028, 0.058),
        round_surface: Color::srgb(0.008, 0.016, 0.036),
        round_inset: Color::srgb(0.012, 0.022, 0.046),
        seat_active_bg: Color::srgba(0.45, 0.80, 1.0, 0.16),
    };

    pub const PAPER: Self = Self {
        casino_felt: Color::srgb(0.91, 0.89, 0.83),
        patience_felt: Color::srgb(0.93, 0.91, 0.86),
        round_surface: Color::srgb(0.89, 0.87, 0.81),
        round_inset: Color::srgb(0.85, 0.83, 0.77),
        seat_active_bg: Color::srgba(0.12, 0.52, 0.24, 0.15),
    };

    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &Self::CYAN,
            ColorScheme::Amber => &Self::AMBER,
            ColorScheme::Emerald => &Self::EMERALD,
            ColorScheme::Violet => &Self::VIOLET,
            ColorScheme::Ice => &Self::ICE,
            ColorScheme::Paper => &Self::PAPER,
        }
    }
}

/// Authored roulette colors for wheel pockets, strip numbers, and table betting cells.
#[derive(Clone, Copy, Debug)]
pub struct RoulettePalette {
    pub red_pocket: Color,
    pub black_pocket: Color,
    pub green_pocket: Color,
    pub zero_fill: Color,
    pub red_strip_fill: Color,
    pub black_strip_fill: Color,
    pub selection_fill: Color,
}

impl RoulettePalette {
    pub const CYAN: Self = Self {
        red_pocket: Color::srgb(1.0, 0.2, 0.4),
        black_pocket: Color::srgb(0.12, 0.38, 0.46),
        green_pocket: Color::srgb(0.0, 1.0, 0.72),
        zero_fill: Color::srgb(0.0, 0.23, 0.13),
        red_strip_fill: Color::srgb(0.18, 0.018, 0.05),
        black_strip_fill: Color::srgb(0.025, 0.055, 0.085),
        selection_fill: Color::srgb(0.045, 0.12, 0.17),
    };

    pub const AMBER: Self = Self {
        red_pocket: Color::srgb(0.94, 0.32, 0.22),
        black_pocket: Color::srgb(0.16, 0.13, 0.10),
        green_pocket: Color::srgb(0.22, 0.78, 0.50),
        zero_fill: Color::srgb(0.08, 0.20, 0.14),
        red_strip_fill: Color::srgb(0.22, 0.06, 0.04),
        black_strip_fill: Color::srgb(0.065, 0.052, 0.038),
        selection_fill: Color::srgb(0.18, 0.12, 0.05),
    };

    pub const EMERALD: Self = Self {
        red_pocket: Color::srgb(0.96, 0.38, 0.28),
        black_pocket: Color::srgb(0.08, 0.18, 0.12),
        green_pocket: Color::srgb(0.15, 0.92, 0.50),
        zero_fill: Color::srgb(0.04, 0.22, 0.12),
        red_strip_fill: Color::srgb(0.20, 0.05, 0.04),
        black_strip_fill: Color::srgb(0.03, 0.07, 0.05),
        selection_fill: Color::srgb(0.08, 0.25, 0.16),
    };

    pub const VIOLET: Self = Self {
        red_pocket: Color::srgb(0.96, 0.32, 0.52),
        black_pocket: Color::srgb(0.18, 0.10, 0.22),
        green_pocket: Color::srgb(0.15, 0.90, 0.85),
        zero_fill: Color::srgb(0.04, 0.18, 0.18),
        red_strip_fill: Color::srgb(0.20, 0.04, 0.08),
        black_strip_fill: Color::srgb(0.06, 0.03, 0.07),
        selection_fill: Color::srgb(0.18, 0.08, 0.24),
    };

    pub const ICE: Self = Self {
        red_pocket: Color::srgb(0.96, 0.38, 0.50),
        black_pocket: Color::srgb(0.10, 0.20, 0.32),
        green_pocket: Color::srgb(0.28, 0.95, 0.82),
        zero_fill: Color::srgb(0.03, 0.16, 0.18),
        red_strip_fill: Color::srgb(0.20, 0.04, 0.08),
        black_strip_fill: Color::srgb(0.03, 0.06, 0.10),
        selection_fill: Color::srgb(0.06, 0.16, 0.26),
    };

    pub const PAPER: Self = Self {
        red_pocket: Color::srgb(0.72, 0.12, 0.20),
        black_pocket: Color::srgb(0.15, 0.15, 0.18),
        green_pocket: Color::srgb(0.10, 0.50, 0.22),
        zero_fill: Color::srgb(0.85, 0.92, 0.86),
        red_strip_fill: Color::srgb(0.95, 0.87, 0.88),
        black_strip_fill: Color::srgb(0.88, 0.87, 0.86),
        selection_fill: Color::srgb(0.92, 0.84, 0.72),
    };

    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &Self::CYAN,
            ColorScheme::Amber => &Self::AMBER,
            ColorScheme::Emerald => &Self::EMERALD,
            ColorScheme::Violet => &Self::VIOLET,
            ColorScheme::Ice => &Self::ICE,
            ColorScheme::Paper => &Self::PAPER,
        }
    }

    #[must_use]
    pub fn pocket(&self, n: u8) -> Color {
        match gamevex_core::roulette::color(n) {
            gamevex_core::roulette::PocketColor::Red => self.red_pocket,
            gamevex_core::roulette::PocketColor::Black => self.black_pocket,
            gamevex_core::roulette::PocketColor::Green => self.green_pocket,
        }
    }
}

/// Authored casino visual feedback styling for banners, action chips, and celebratory particles.
#[derive(Clone, Copy, Debug)]
pub struct CasinoFxPalette {
    pub win: Color,
    pub natural_or_big_win: Color,
    pub push: Color,
    pub loss: Color,
    pub game_over: Color,
    pub banner_bg_default: Color,
    pub banner_bg_loss: Color,
    pub banner_bg_game_over: Color,
    pub badge_bg: Color,
    pub badge_text: Color,
    pub title_text: Color,
    pub detail_text: Color,
    pub chip_bg: Color,
    pub chip_text: Color,
    pub win_base_premium: Color,
    pub win_base_regular: Color,
    pub win_sparkle: Color,
}

impl CasinoFxPalette {
    pub const CYAN: Self = Self {
        win: Color::srgb(0.28, 1.0, 0.62),
        natural_or_big_win: Color::srgb(1.0, 0.79, 0.25),
        push: Color::srgb(0.18, 0.86, 1.0),
        loss: Color::srgb(1.0, 0.31, 0.38),
        game_over: Color::srgb(0.72, 0.16, 0.25),
        banner_bg_default: Color::srgba(0.025, 0.07, 0.075, 0.94),
        banner_bg_loss: Color::srgba(0.10, 0.025, 0.04, 0.94),
        banner_bg_game_over: Color::srgba(0.12, 0.025, 0.045, 0.96),
        badge_bg: Color::srgba(0.02, 0.035, 0.045, 0.92),
        badge_text: Color::srgb(0.96, 0.99, 1.0),
        title_text: Color::srgb(0.98, 1.0, 1.0),
        detail_text: Color::srgb(0.81, 0.91, 0.94),
        chip_bg: Color::srgba(0.015, 0.035, 0.045, 0.94),
        chip_text: Color::srgb(0.96, 0.99, 1.0),
        win_base_premium: Color::srgb(1.0, 0.78, 0.22),
        win_base_regular: Color::srgb(0.43, 1.0, 0.64),
        win_sparkle: Color::srgb(1.0, 0.95, 0.62),
    };

    pub const AMBER: Self = Self {
        win: Color::srgb(0.22, 0.82, 0.52),
        natural_or_big_win: Color::srgb(1.0, 0.80, 0.26),
        push: Color::srgb(0.96, 0.78, 0.40),
        loss: Color::srgb(0.94, 0.32, 0.22),
        game_over: Color::srgb(0.76, 0.18, 0.20),
        banner_bg_default: Color::srgba(0.065, 0.05, 0.035, 0.94),
        banner_bg_loss: Color::srgba(0.13, 0.035, 0.03, 0.94),
        banner_bg_game_over: Color::srgba(0.15, 0.035, 0.04, 0.96),
        badge_bg: Color::srgba(0.05, 0.038, 0.025, 0.92),
        badge_text: Color::srgb(0.96, 0.92, 0.84),
        title_text: Color::srgb(0.98, 0.95, 0.90),
        detail_text: Color::srgb(0.88, 0.82, 0.70),
        chip_bg: Color::srgba(0.05, 0.038, 0.025, 0.94),
        chip_text: Color::srgb(0.96, 0.92, 0.84),
        win_base_premium: Color::srgb(1.0, 0.80, 0.26),
        win_base_regular: Color::srgb(0.22, 0.82, 0.52),
        win_sparkle: Color::srgb(1.0, 0.90, 0.70),
    };

    pub const EMERALD: Self = Self {
        win: Color::srgb(0.35, 0.98, 0.65),
        natural_or_big_win: Color::srgb(1.0, 0.82, 0.30),
        push: Color::srgb(0.45, 0.90, 0.80),
        loss: Color::srgb(0.96, 0.38, 0.28),
        game_over: Color::srgb(0.78, 0.22, 0.18),
        banner_bg_default: Color::srgba(0.02, 0.065, 0.04, 0.94),
        banner_bg_loss: Color::srgba(0.12, 0.03, 0.03, 0.94),
        banner_bg_game_over: Color::srgba(0.14, 0.03, 0.03, 0.96),
        badge_bg: Color::srgba(0.02, 0.05, 0.03, 0.92),
        badge_text: Color::srgb(0.90, 0.98, 0.92),
        title_text: Color::srgb(0.94, 1.0, 0.96),
        detail_text: Color::srgb(0.72, 0.90, 0.80),
        chip_bg: Color::srgba(0.02, 0.05, 0.03, 0.94),
        chip_text: Color::srgb(0.90, 0.98, 0.92),
        win_base_premium: Color::srgb(1.0, 0.82, 0.30),
        win_base_regular: Color::srgb(0.25, 0.92, 0.55),
        win_sparkle: Color::srgb(0.85, 1.0, 0.90),
    };

    pub const VIOLET: Self = Self {
        win: Color::srgb(0.15, 0.90, 0.85),
        natural_or_big_win: Color::srgb(1.0, 0.82, 0.32),
        push: Color::srgb(0.82, 0.65, 0.98),
        loss: Color::srgb(0.96, 0.32, 0.52),
        game_over: Color::srgb(0.75, 0.16, 0.30),
        banner_bg_default: Color::srgba(0.055, 0.025, 0.070, 0.94),
        banner_bg_loss: Color::srgba(0.13, 0.03, 0.05, 0.94),
        banner_bg_game_over: Color::srgba(0.15, 0.03, 0.05, 0.96),
        badge_bg: Color::srgba(0.04, 0.02, 0.05, 0.92),
        badge_text: Color::srgb(0.96, 0.92, 1.0),
        title_text: Color::srgb(0.98, 0.95, 1.0),
        detail_text: Color::srgb(0.85, 0.78, 0.92),
        chip_bg: Color::srgba(0.04, 0.02, 0.05, 0.94),
        chip_text: Color::srgb(0.96, 0.92, 1.0),
        win_base_premium: Color::srgb(1.0, 0.82, 0.32),
        win_base_regular: Color::srgb(0.20, 0.92, 0.86),
        win_sparkle: Color::srgb(0.92, 0.85, 1.0),
    };

    pub const ICE: Self = Self {
        win: Color::srgb(0.28, 0.95, 0.82),
        natural_or_big_win: Color::srgb(1.0, 0.84, 0.40),
        push: Color::srgb(0.55, 0.85, 1.0),
        loss: Color::srgb(0.96, 0.38, 0.50),
        game_over: Color::srgb(0.74, 0.18, 0.28),
        banner_bg_default: Color::srgba(0.018, 0.045, 0.080, 0.94),
        banner_bg_loss: Color::srgba(0.12, 0.03, 0.05, 0.94),
        banner_bg_game_over: Color::srgba(0.14, 0.03, 0.05, 0.96),
        badge_bg: Color::srgba(0.015, 0.035, 0.06, 0.92),
        badge_text: Color::srgb(0.92, 0.96, 1.0),
        title_text: Color::srgb(0.95, 0.98, 1.0),
        detail_text: Color::srgb(0.75, 0.88, 0.96),
        chip_bg: Color::srgba(0.015, 0.035, 0.06, 0.94),
        chip_text: Color::srgb(0.92, 0.96, 1.0),
        win_base_premium: Color::srgb(1.0, 0.84, 0.40),
        win_base_regular: Color::srgb(0.30, 0.95, 0.85),
        win_sparkle: Color::srgb(0.85, 0.95, 1.0),
    };

    pub const PAPER: Self = Self {
        win: Color::srgb(0.12, 0.52, 0.24),
        natural_or_big_win: Color::srgb(0.72, 0.48, 0.10),
        push: Color::srgb(0.24, 0.42, 0.54),
        loss: Color::srgb(0.72, 0.12, 0.20),
        game_over: Color::srgb(0.60, 0.08, 0.14),
        banner_bg_default: Color::srgba(0.96, 0.94, 0.90, 0.96),
        banner_bg_loss: Color::srgba(0.95, 0.88, 0.88, 0.95),
        banner_bg_game_over: Color::srgba(0.94, 0.86, 0.86, 0.96),
        badge_bg: Color::srgba(0.90, 0.88, 0.82, 0.95),
        badge_text: Color::srgb(0.12, 0.12, 0.14),
        title_text: Color::srgb(0.10, 0.10, 0.12),
        detail_text: Color::srgb(0.32, 0.32, 0.35),
        chip_bg: Color::srgba(0.94, 0.92, 0.87, 0.95),
        chip_text: Color::srgb(0.12, 0.12, 0.14),
        win_base_premium: Color::srgb(0.72, 0.48, 0.10),
        win_base_regular: Color::srgb(0.12, 0.52, 0.24),
        win_sparkle: Color::srgb(0.60, 0.40, 0.08),
    };

    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &Self::CYAN,
            ColorScheme::Amber => &Self::AMBER,
            ColorScheme::Emerald => &Self::EMERALD,
            ColorScheme::Violet => &Self::VIOLET,
            ColorScheme::Ice => &Self::ICE,
            ColorScheme::Paper => &Self::PAPER,
        }
    }
}

/// Authored table feedback effect colors.
#[derive(Clone, Copy, Debug)]
pub struct TableFxPalette {
    pub invalid: [f32; 3],
    pub win: [f32; 3],
    pub move_color: [f32; 3],
    pub undo: [f32; 3],
    pub default: [f32; 3],
    pub fountain_tint: Color,
    pub caption_bg: Color,
}

impl TableFxPalette {
    pub const CYAN: Self = Self {
        invalid: [1.0, 0.24, 0.32],
        win: [1.0, 0.78, 0.15],
        move_color: [0.12, 1.0, 0.68],
        undo: [0.90, 0.28, 1.0],
        default: [0.0, 0.88, 1.0],
        fountain_tint: Color::srgb(1.0, 0.2, 0.65),
        caption_bg: Color::srgba(0.015, 0.035, 0.05, 0.95),
    };

    pub const AMBER: Self = Self {
        invalid: [0.94, 0.32, 0.22],
        win: [1.0, 0.80, 0.26],
        move_color: [0.22, 0.82, 0.52],
        undo: [0.96, 0.55, 0.18],
        default: [1.0, 0.73, 0.20],
        fountain_tint: Color::srgb(0.96, 0.55, 0.18),
        caption_bg: Color::srgba(0.05, 0.038, 0.025, 0.95),
    };

    pub const EMERALD: Self = Self {
        invalid: [0.96, 0.38, 0.28],
        win: [1.0, 0.82, 0.30],
        move_color: [0.35, 0.98, 0.65],
        undo: [0.95, 0.60, 0.25],
        default: [0.18, 0.85, 0.48],
        fountain_tint: Color::srgb(0.96, 0.38, 0.28),
        caption_bg: Color::srgba(0.02, 0.05, 0.035, 0.95),
    };

    pub const VIOLET: Self = Self {
        invalid: [0.96, 0.32, 0.52],
        win: [1.0, 0.82, 0.32],
        move_color: [0.15, 0.90, 0.85],
        undo: [0.82, 0.58, 0.98],
        default: [0.72, 0.35, 0.98],
        fountain_tint: Color::srgb(0.15, 0.90, 0.85),
        caption_bg: Color::srgba(0.045, 0.02, 0.06, 0.95),
    };

    pub const ICE: Self = Self {
        invalid: [0.96, 0.38, 0.50],
        win: [1.0, 0.84, 0.40],
        move_color: [0.28, 0.95, 0.82],
        undo: [0.50, 0.80, 1.0],
        default: [0.45, 0.80, 1.0],
        fountain_tint: Color::srgb(0.96, 0.38, 0.50),
        caption_bg: Color::srgba(0.015, 0.035, 0.065, 0.95),
    };

    pub const PAPER: Self = Self {
        invalid: [0.72, 0.12, 0.20],
        win: [0.72, 0.48, 0.10],
        move_color: [0.12, 0.52, 0.24],
        undo: [0.70, 0.32, 0.14],
        default: [0.24, 0.32, 0.40],
        fountain_tint: Color::srgb(0.72, 0.12, 0.20),
        caption_bg: Color::srgba(0.95, 0.93, 0.88, 0.96),
    };

    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &Self::CYAN,
            ColorScheme::Amber => &Self::AMBER,
            ColorScheme::Emerald => &Self::EMERALD,
            ColorScheme::Violet => &Self::VIOLET,
            ColorScheme::Ice => &Self::ICE,
            ColorScheme::Paper => &Self::PAPER,
        }
    }

    #[must_use]
    pub fn event_rgb(&self, kind: crate::table_feedback::EffectKind) -> [f32; 3] {
        use crate::table_feedback::EffectKind as K;
        match kind {
            K::Invalid | K::Stuck => self.invalid,
            K::Win | K::Run | K::Hint => self.win,
            K::Move => self.move_color,
            K::Undo | K::Shuffle => self.undo,
            _ => self.default,
        }
    }
}

/// Helper getters for game palettes by `ColorScheme`.
#[must_use]
pub fn card(scheme: ColorScheme) -> &'static CardPalette {
    CardPalette::for_scheme(scheme)
}

#[must_use]
pub fn table(scheme: ColorScheme) -> &'static TablePalette {
    TablePalette::for_scheme(scheme)
}

#[must_use]
pub fn roulette(scheme: ColorScheme) -> &'static RoulettePalette {
    RoulettePalette::for_scheme(scheme)
}

#[must_use]
pub fn casino_fx(scheme: ColorScheme) -> &'static CasinoFxPalette {
    CasinoFxPalette::for_scheme(scheme)
}

#[must_use]
pub fn table_fx(scheme: ColorScheme) -> &'static TableFxPalette {
    TableFxPalette::for_scheme(scheme)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_SCHEMES: [ColorScheme; 6] = [
        ColorScheme::Cyan,
        ColorScheme::Amber,
        ColorScheme::Emerald,
        ColorScheme::Violet,
        ColorScheme::Ice,
        ColorScheme::Paper,
    ];

    #[test]
    fn themes_preserve_the_red_black_suit_groups_used_by_card_games() {
        for scheme in ALL_SCHEMES {
            let palette = card(scheme);
            for a in [Suit::Hearts, Suit::Diamonds, Suit::Spades, Suit::Clubs] {
                for b in [Suit::Hearts, Suit::Diamonds, Suit::Spades, Suit::Clubs] {
                    assert_eq!(
                        palette.suit_color(a) == palette.suit_color(b),
                        a.is_red() == b.is_red(),
                        "{scheme:?}: card colors must reflect the rules' suit groups (Hearts+Diamonds vs Spades+Clubs)"
                    );
                }
            }
            assert_eq!(palette.suit_color(Suit::Hearts), palette.red_suit);
            assert_eq!(palette.suit_color(Suit::Diamonds), palette.red_suit);
            assert_eq!(palette.suit_color(Suit::Spades), palette.black_suit);
            assert_eq!(palette.suit_color(Suit::Clubs), palette.black_suit);
            assert_ne!(palette.red_suit, palette.black_suit);
        }
    }

    #[test]
    fn emerald_theme_does_not_use_success_green_for_club_or_black_suit() {
        let emerald_card = card(ColorScheme::Emerald);
        let black_suit = emerald_card.black_suit.to_srgba();
        // Spades and clubs use mint-cream, not a pure green (which would look like success feedback)
        assert!(
            black_suit.red > 0.7,
            "Emerald black suit must be mint-cream/ivory, not pure green"
        );
        assert!(
            black_suit.blue > 0.7,
            "Emerald black suit must have balanced blue for mint-cream"
        );
        assert!(black_suit.green > 0.85);
    }

    #[test]
    fn paper_theme_provides_readable_dark_markings_on_light_surfaces() {
        let paper_card = card(ColorScheme::Paper);
        let face = paper_card.face_bg.to_srgba();
        let red = paper_card.red_suit.to_srgba();
        let black = paper_card.black_suit.to_srgba();

        // Light card face surface
        assert!(face.red > 0.85 && face.green > 0.85 && face.blue > 0.80);

        // Dark markings on light card face
        assert!(black.red < 0.25 && black.green < 0.25 && black.blue < 0.25);
        assert!(red.green < 0.25 && red.blue < 0.30); // deep burgundy

        let paper_table = table(ColorScheme::Paper);
        let felt = paper_table.casino_felt.to_srgba();
        assert!(felt.red > 0.80 && felt.green > 0.80 && felt.blue > 0.75);

        let paper_roulette = roulette(ColorScheme::Paper);
        let r_red = paper_roulette.red_pocket.to_srgba();
        let r_green = paper_roulette.green_pocket.to_srgba();
        assert!(
            r_red.red > r_red.green * 2.0,
            "Roulette red pocket must be distinctly red"
        );
        assert!(
            r_green.green > r_green.red * 2.0,
            "Roulette green pocket must be distinctly green"
        );
    }

    #[test]
    fn all_palettes_provide_finite_normalized_colors_for_all_schemes() {
        for scheme in ALL_SCHEMES {
            let c_pal = card(scheme);
            for col in [
                c_pal.face_bg,
                c_pal.shared_face_bg,
                c_pal.back_bg,
                c_pal.shadow,
                c_pal.border_normal,
                c_pal.border_selected,
                c_pal.border_legal,
                c_pal.red_suit,
                c_pal.black_suit,
                c_pal.card_dim,
            ] {
                let s = col.to_srgba();
                assert!(s.red.is_finite() && (0.0..=1.0).contains(&s.red));
                assert!(s.green.is_finite() && (0.0..=1.0).contains(&s.green));
                assert!(s.blue.is_finite() && (0.0..=1.0).contains(&s.blue));
                assert!(s.alpha.is_finite() && (0.0..=1.0).contains(&s.alpha));
            }

            let t_pal = table(scheme);
            for col in [
                t_pal.casino_felt,
                t_pal.patience_felt,
                t_pal.round_surface,
                t_pal.round_inset,
                t_pal.seat_active_bg,
            ] {
                let s = col.to_srgba();
                assert!(s.red.is_finite() && (0.0..=1.0).contains(&s.red));
                assert!(s.green.is_finite() && (0.0..=1.0).contains(&s.green));
                assert!(s.blue.is_finite() && (0.0..=1.0).contains(&s.blue));
                assert!(s.alpha.is_finite() && (0.0..=1.0).contains(&s.alpha));
            }

            let r_pal = roulette(scheme);
            for col in [
                r_pal.red_pocket,
                r_pal.black_pocket,
                r_pal.green_pocket,
                r_pal.zero_fill,
                r_pal.red_strip_fill,
                r_pal.black_strip_fill,
                r_pal.selection_fill,
            ] {
                let s = col.to_srgba();
                assert!(s.red.is_finite() && (0.0..=1.0).contains(&s.red));
                assert!(s.green.is_finite() && (0.0..=1.0).contains(&s.green));
                assert!(s.blue.is_finite() && (0.0..=1.0).contains(&s.blue));
                assert!(s.alpha.is_finite() && (0.0..=1.0).contains(&s.alpha));
            }

            let fx_pal = casino_fx(scheme);
            for col in [
                fx_pal.win,
                fx_pal.natural_or_big_win,
                fx_pal.push,
                fx_pal.loss,
                fx_pal.game_over,
                fx_pal.banner_bg_default,
                fx_pal.banner_bg_loss,
                fx_pal.banner_bg_game_over,
                fx_pal.badge_bg,
                fx_pal.badge_text,
                fx_pal.title_text,
                fx_pal.detail_text,
                fx_pal.chip_bg,
                fx_pal.chip_text,
                fx_pal.win_base_premium,
                fx_pal.win_base_regular,
                fx_pal.win_sparkle,
            ] {
                let s = col.to_srgba();
                assert!(s.red.is_finite() && (0.0..=1.0).contains(&s.red));
                assert!(s.green.is_finite() && (0.0..=1.0).contains(&s.green));
                assert!(s.blue.is_finite() && (0.0..=1.0).contains(&s.blue));
                assert!(s.alpha.is_finite() && (0.0..=1.0).contains(&s.alpha));
            }

            let tfx_pal = table_fx(scheme);
            for col in [tfx_pal.fountain_tint, tfx_pal.caption_bg] {
                let s = col.to_srgba();
                assert!(s.red.is_finite() && (0.0..=1.0).contains(&s.red));
                assert!(s.green.is_finite() && (0.0..=1.0).contains(&s.green));
                assert!(s.blue.is_finite() && (0.0..=1.0).contains(&s.blue));
                assert!(s.alpha.is_finite() && (0.0..=1.0).contains(&s.alpha));
            }
        }
    }
}
