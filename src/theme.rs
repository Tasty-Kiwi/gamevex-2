//! # Theme System
//!
//! Explicit semantic color palettes for GameVex.
//!
//! Rather than relying on RGB post-processing heuristics which distort explicit
//! colors (such as card suits, alert flags, and transparency), GameVex uses an explicit
//! semantic theme role model.
//!
//! ## Semantic Roles
//! Every theme provides 12 standard semantic roles:
//! - `bg`: Cabinet backdrop / play area canvas
//! - `panel`: Surfaces, cards, and modal dialog backgrounds
//! - `panel_hover`: Hovered and keyboard-focused control surfaces
//! - `accent`: Primary interactive highlight / brand color (e.g. Cyan or Amber)
//! - `secondary`: Complementary highlight (e.g. Magenta or Terracotta)
//! - `gold`: High-value milestones, coins, chips, and keyboard focus outlines
//! - `ink`: Primary high-contrast readable text and markings
//! - `muted`: Secondary captions, instructions, and inactive labels
//! - `control_text`: Resting button labels
//! - `edge`: Subtle divider lines, slot borders, and structural outlines
//! - `success`: Win states, safe indicators, and confirmations (e.g. Jade)
//! - `danger`: Loss states, mine explosions, and destructive actions (e.g. Coral)
//!
//! ## Adding a New Theme
//! To add a new theme to GameVex:
//! 1. Add a new variant to `ColorScheme` in `src/core/save.rs` (e.g. `Emerald`).
//! 2. Define a complete static literal `pub const EMERALD_THEME: Theme = ...` in `src/theme.rs`.
//! 3. Add the mapping to `Theme::for_scheme(scheme)`.
//! 4. Add a `ThemeMeta` entry to `THEME_REGISTRY` in `src/theme.rs`.
//! 5. Supply the new variant in each game palette `for_scheme` match (the compiler
//!    identifies these), including board markings, cards, tables, and effects.
//! 6. Review the lobby and every game with the new theme and CRT effects enabled.

use bevy::color::Color;
use gamevex_core::ColorScheme;

/// A complete, explicit semantic color palette for the GameVex arcade interface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub bg: Color,
    pub panel: Color,
    pub panel_hover: Color,
    pub accent: Color,
    pub secondary: Color,
    pub gold: Color,
    pub ink: Color,
    pub muted: Color,
    pub control_text: Color,
    pub edge: Color,
    pub success: Color,
    pub danger: Color,
}

/// The classic neon-vector cyan/magenta theme.
/// Preserves exact existing shared constant values.
pub const CYAN_THEME: Theme = Theme {
    bg: Color::srgb(0.008, 0.016, 0.031),
    panel: Color::srgb(0.025, 0.055, 0.085),
    panel_hover: Color::srgb(0.055, 0.14, 0.18),
    accent: Color::srgb(0.0, 0.83, 1.0),
    secondary: Color::srgb(1.0, 0.0, 0.67),
    gold: Color::srgb(1.0, 0.67, 0.0),
    ink: Color::srgb(0.87, 0.94, 1.0),
    muted: Color::srgb(0.39, 0.58, 0.68),
    control_text: Color::srgb(0.6, 0.78, 0.85),
    edge: Color::srgb(0.08, 0.22, 0.29),
    success: Color::srgb(0.0, 1.0, 0.72),
    danger: Color::srgb(1.0, 0.2, 0.4),
};

/// Warm phosphor amber theme.
/// Features warm charcoal backgrounds, cream typography, vibrant amber accents,
/// warm terracotta secondary accents, with distinct jade (success) and coral (danger) feedback.
pub const AMBER_THEME: Theme = Theme {
    bg: Color::srgb(0.028, 0.022, 0.016),
    panel: Color::srgb(0.065, 0.052, 0.038),
    panel_hover: Color::srgb(0.189, 0.148, 0.104),
    accent: Color::srgb(1.0, 0.73, 0.20),
    secondary: Color::srgb(1.0, 0.52, 0.16),
    gold: Color::srgb(1.0, 0.88, 0.52),
    ink: Color::srgb(0.96, 0.90, 0.78),
    muted: Color::srgb(0.62, 0.52, 0.38),
    control_text: Color::srgb(0.84, 0.75, 0.60),
    edge: Color::srgb(0.24, 0.18, 0.12),
    success: Color::srgb(0.18, 0.84, 0.52),
    danger: Color::srgb(1.0, 0.33, 0.20),
};

impl Theme {
    /// Return the static `Theme` definition corresponding to `scheme`.
    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &CYAN_THEME,
            ColorScheme::Amber => &AMBER_THEME,
        }
    }
}

/// Metadata describing a selectable theme in the arcade cabinet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeMeta {
    pub scheme: ColorScheme,
    pub name: &'static str,
    pub description: &'static str,
}

/// Registry of available themes in cycling order.
pub const THEME_REGISTRY: &[ThemeMeta] = &[
    ThemeMeta {
        scheme: ColorScheme::Cyan,
        name: "Cyan",
        description: "Classic neon vector arcade with cyan and magenta accents",
    },
    ThemeMeta {
        scheme: ColorScheme::Amber,
        name: "Amber",
        description: "Warm phosphor palette with charcoal, cream, jade, and coral",
    },
];

/// Returns the user-facing display name for a given color scheme.
pub fn scheme_name(scheme: ColorScheme) -> &'static str {
    THEME_REGISTRY
        .iter()
        .find(|meta| meta.scheme == scheme)
        .map(|meta| meta.name)
        .unwrap_or("Custom")
}

/// Advance to the next available theme in the registry cycle.
pub fn cycle_scheme(current: ColorScheme) -> ColorScheme {
    let index = THEME_REGISTRY
        .iter()
        .position(|meta| meta.scheme == current)
        .unwrap_or(0);
    THEME_REGISTRY[(index + 1) % THEME_REGISTRY.len()].scheme
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_scheme_returns_valid_themes_for_all_schemes() {
        for meta in THEME_REGISTRY {
            let theme = Theme::for_scheme(meta.scheme);
            for color in [
                theme.bg,
                theme.panel,
                theme.panel_hover,
                theme.accent,
                theme.secondary,
                theme.gold,
                theme.ink,
                theme.muted,
                theme.control_text,
                theme.edge,
                theme.success,
                theme.danger,
            ] {
                let srgba = color.to_srgba();
                assert!(srgba.red.is_finite() && (0.0..=1.0).contains(&srgba.red));
                assert!(srgba.green.is_finite() && (0.0..=1.0).contains(&srgba.green));
                assert!(srgba.blue.is_finite() && (0.0..=1.0).contains(&srgba.blue));
                assert!(srgba.alpha.is_finite() && (0.0..=1.0).contains(&srgba.alpha));
            }
        }
    }

    #[test]
    fn amber_theme_provides_distinct_feedback_colors() {
        let amber = &AMBER_THEME;
        let success = amber.success.to_srgba();
        let danger = amber.danger.to_srgba();
        let accent = amber.accent.to_srgba();

        // Jade success must have high green, significantly higher than red
        assert!(success.green > 0.7);
        assert!(success.green > success.red * 2.0);

        // Coral danger must have high red, significantly higher than green
        assert!(danger.red > 0.8);
        assert!(danger.red > danger.green * 2.0);

        // Accent must be distinctly amber (high red, high green, low blue)
        assert!(accent.red > 0.9);
        assert!(accent.green > 0.6 && accent.green < 0.85);
        assert!(accent.blue < 0.3);
    }

    #[test]
    fn cycle_scheme_advances_and_wraps_cleanly() {
        assert_eq!(cycle_scheme(ColorScheme::Cyan), ColorScheme::Amber);
        assert_eq!(cycle_scheme(ColorScheme::Amber), ColorScheme::Cyan);
    }

    #[test]
    fn scheme_name_matches_registry() {
        assert_eq!(scheme_name(ColorScheme::Cyan), "Cyan");
        assert_eq!(scheme_name(ColorScheme::Amber), "Amber");
    }
}
