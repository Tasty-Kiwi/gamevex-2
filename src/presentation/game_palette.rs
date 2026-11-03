//! Authored Cyan and Amber palettes for game presentation.
//!
//! Replaces hardcoded per-game colors (felt, cards, chips, roulette wheel, etc.)
//! with explicit roles tailored for Cyan (cool dark arcade) and Amber
//! (warm charcoal/cream/amber with jade greens and coral reds).

use bevy::prelude::Color;
use gamevex_core::{ColorScheme, cards::Suit};

/// Authored card styling for face, back, outline, and suits.
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
    };

    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &Self::CYAN,
            ColorScheme::Amber => &Self::AMBER,
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
}

impl TablePalette {
    pub const CYAN: Self = Self {
        casino_felt: Color::srgb(0.008, 0.045, 0.041),
        patience_felt: Color::srgb(0.012, 0.055, 0.047),
        round_surface: Color::srgb(0.008, 0.018, 0.032),
        round_inset: Color::srgb(0.012, 0.028, 0.046),
    };

    pub const AMBER: Self = Self {
        casino_felt: Color::srgb(0.042, 0.032, 0.020),
        patience_felt: Color::srgb(0.045, 0.034, 0.022),
        round_surface: Color::srgb(0.032, 0.024, 0.016),
        round_inset: Color::srgb(0.044, 0.034, 0.022),
    };

    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &Self::CYAN,
            ColorScheme::Amber => &Self::AMBER,
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

    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &Self::CYAN,
            ColorScheme::Amber => &Self::AMBER,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_preserve_the_red_black_suit_groups_used_by_card_games() {
        for meta in crate::theme::THEME_REGISTRY {
            let palette = card(meta.scheme);
            for a in [Suit::Hearts, Suit::Diamonds, Suit::Spades, Suit::Clubs] {
                for b in [Suit::Hearts, Suit::Diamonds, Suit::Spades, Suit::Clubs] {
                    assert_eq!(
                        palette.suit_color(a) == palette.suit_color(b),
                        a.is_red() == b.is_red(),
                        "{:?}: card colors must reflect the rules' suit groups",
                        meta.scheme
                    );
                }
            }
        }
    }
}
