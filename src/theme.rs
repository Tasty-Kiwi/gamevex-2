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

pub const EMERALD_THEME: Theme = Theme {
    bg: Color::srgb(0.012, 0.032, 0.020),
    panel: Color::srgb(0.026, 0.068, 0.044),
    panel_hover: Color::srgb(0.055, 0.140, 0.092),
    accent: Color::srgb(0.08, 0.86, 0.52),
    secondary: Color::srgb(0.38, 0.94, 0.72),
    gold: Color::srgb(0.96, 0.82, 0.30),
    ink: Color::srgb(0.88, 0.98, 0.92),
    muted: Color::srgb(0.40, 0.64, 0.50),
    control_text: Color::srgb(0.66, 0.86, 0.75),
    edge: Color::srgb(0.09, 0.24, 0.16),
    success: Color::srgb(0.18, 0.94, 0.60),
    danger: Color::srgb(1.0, 0.34, 0.26),
};

pub const VIOLET_THEME: Theme = Theme {
    bg: Color::srgb(0.025, 0.012, 0.034),
    panel: Color::srgb(0.058, 0.032, 0.078),
    panel_hover: Color::srgb(0.125, 0.070, 0.165),
    accent: Color::srgb(0.74, 0.36, 1.0),
    secondary: Color::srgb(0.18, 0.88, 0.88),
    gold: Color::srgb(1.0, 0.78, 0.28),
    ink: Color::srgb(0.95, 0.90, 0.98),
    muted: Color::srgb(0.58, 0.48, 0.68),
    control_text: Color::srgb(0.78, 0.70, 0.88),
    edge: Color::srgb(0.20, 0.12, 0.28),
    success: Color::srgb(0.18, 0.92, 0.80),
    danger: Color::srgb(1.0, 0.28, 0.44),
};

pub const PAPER_THEME: Theme = Theme {
    bg: Color::srgb(0.95, 0.93, 0.88),
    panel: Color::srgb(0.89, 0.86, 0.80),
    panel_hover: Color::srgb(0.80, 0.76, 0.68),
    accent: Color::srgb(0.55, 0.10, 0.20),
    secondary: Color::srgb(0.14, 0.44, 0.24),
    gold: Color::srgb(0.60, 0.40, 0.05),
    ink: Color::srgb(0.12, 0.12, 0.14),
    muted: Color::srgb(0.38, 0.36, 0.34),
    control_text: Color::srgb(0.18, 0.18, 0.20),
    edge: Color::srgb(0.72, 0.68, 0.60),
    success: Color::srgb(0.12, 0.50, 0.22),
    danger: Color::srgb(0.68, 0.12, 0.16),
};

impl Theme {
    /// Return the static `Theme` definition corresponding to `scheme`.
    #[must_use]
    pub const fn for_scheme(scheme: ColorScheme) -> &'static Self {
        match scheme {
            ColorScheme::Cyan => &CYAN_THEME,
            ColorScheme::Amber => &AMBER_THEME,
            ColorScheme::Emerald => &EMERALD_THEME,
            ColorScheme::Violet => &VIOLET_THEME,
            ColorScheme::Paper => &PAPER_THEME,
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

/// Registry of available themes in cabinet display order.
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
    ThemeMeta {
        scheme: ColorScheme::Emerald,
        name: "Emerald",
        description: "Deep forest cabinet with vibrant mint, emerald, and coral",
    },
    ThemeMeta {
        scheme: ColorScheme::Violet,
        name: "Violet",
        description: "Dark plum arcade with electric violet, lavender, and turquoise",
    },
    ThemeMeta {
        scheme: ColorScheme::Paper,
        name: "Paper",
        description: "Light ivory and charcoal print with burgundy and forest green",
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
    fn new_themes_provide_distinct_palettes_and_contrast() {
        // Emerald
        let emerald = &EMERALD_THEME;
        let em_succ = emerald.success.to_srgba();
        let em_dang = emerald.danger.to_srgba();
        assert!(em_succ.green > em_succ.red * 2.0);
        assert!(em_dang.red > em_dang.green * 2.0);

        // Violet
        let violet = &VIOLET_THEME;
        let vi_acc = violet.accent.to_srgba();
        assert!(vi_acc.blue > 0.9 && vi_acc.red > 0.6);

        // Paper: light background, dark ink and control text for high contrast
        let paper = &PAPER_THEME;
        let paper_bg = paper.bg.to_srgba();
        let paper_ink = paper.ink.to_srgba();
        let paper_ctrl = paper.control_text.to_srgba();
        assert!(paper_bg.red > 0.9 && paper_bg.green > 0.9 && paper_bg.blue > 0.8);
        assert!(paper_ink.red < 0.2 && paper_ink.green < 0.2 && paper_ink.blue < 0.2);
        assert!(paper_ctrl.red < 0.25 && paper_ctrl.green < 0.25 && paper_ctrl.blue < 0.25);
    }

    #[test]
    fn scheme_name_matches_registry_for_all_five_themes() {
        assert_eq!(scheme_name(ColorScheme::Cyan), "Cyan");
        assert_eq!(scheme_name(ColorScheme::Amber), "Amber");
        assert_eq!(scheme_name(ColorScheme::Emerald), "Emerald");
        assert_eq!(scheme_name(ColorScheme::Violet), "Violet");
        assert_eq!(scheme_name(ColorScheme::Paper), "Paper");
        assert_eq!(THEME_REGISTRY.len(), 5);
    }
}
