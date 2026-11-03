//! Shared, bounded visual effects for casino outcomes and player actions.
//!
//! These helpers draw into the immediate-mode frame only. They do not create
//! hit targets, retain state, or allocate per-particle collections.

use crate::rendering::Frame;
use bevy::prelude::{Color, Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultKind {
    Win,
    Natural,
    BigWin,
    Push,
    Loss,
    GameOver,
}

impl From<crate::casino_feedback::RoundKind> for ResultKind {
    fn from(kind: crate::casino_feedback::RoundKind) -> Self {
        use crate::casino_feedback::RoundKind as K;
        match kind {
            K::Win => Self::Win,
            K::Natural => Self::Natural,
            K::BigWin => Self::BigWin,
            K::Push => Self::Push,
            K::Loss => Self::Loss,
            K::GameOver => Self::GameOver,
        }
    }
}

/// Main tint for an outcome and its associated emphasis effects.
pub fn kind_color(kind: ResultKind, palette: gamevex_core::ColorScheme) -> Color {
    let pal = super::game_palette::casino_fx(palette);
    match kind {
        ResultKind::Win => pal.win,
        ResultKind::Natural | ResultKind::BigWin => pal.natural_or_big_win,
        ResultKind::Push => pal.push,
        ResultKind::Loss => pal.loss,
        ResultKind::GameOver => pal.game_over,
    }
}

/// Draw a persistent outcome panel with bounded, short-lived ornamental FX.
/// `age` is elapsed seconds since the result began; `None` draws a static panel.
pub fn banner(
    f: &mut Frame,
    r: Rect,
    kind: ResultKind,
    title: &str,
    detail: &str,
    age: Option<f32>,
    reduced: bool,
) {
    let Some(r) = clipped_rect(f, r) else { return };
    let tint = kind_color(kind, f.palette);
    let active = !reduced && age.is_some_and(|t| t.is_finite() && (0.0..2.0).contains(&t));
    let t = age.filter(|t| t.is_finite()).unwrap_or(2.0).clamp(0.0, 2.0);

    let pal = super::game_palette::casino_fx(f.palette);
    let bg = match kind {
        ResultKind::GameOver => pal.banner_bg_game_over,
        ResultKind::Loss => pal.banner_bg_loss,
        _ => pal.banner_bg_default,
    };
    f.rect(r.min, r.size(), bg);
    f.rect(r.min, Vec2::new(r.width(), 2.0_f32.min(r.height())), tint);
    f.outline(r.min, r.size(), 1.5, tint);
    terminal_corners(
        f,
        r,
        tint,
        8.0_f32.min(r.width() * 0.10).min(r.height() * 0.22),
    );

    if active {
        match kind {
            ResultKind::Win => winning_fx(f, r, t, false),
            ResultKind::Natural | ResultKind::BigWin => winning_fx(f, r, t, true),
            ResultKind::Push => push_fx(f, r, t),
            ResultKind::Loss => loss_fx(f, r, t),
            ResultKind::GameOver => loss_fx(f, r, t),
        }
    }

    let badge = badge(kind);
    let badge_w = (r.height() - 12.0)
        .clamp(26.0, 40.0)
        .min((r.width() * 0.20).max(20.0));
    let badge_min = r.min + Vec2::new(7.0, 7.0);
    let badge_r = Rect::from_corners(
        badge_min,
        badge_min + Vec2::new(badge_w, (r.height() - 14.0).max(1.0)),
    );
    if badge_r.max.x < r.max.x - 5.0 {
        let badge_bg = pal.badge_bg;
        f.rect(badge_r.min, badge_r.size(), badge_bg);
        f.outline(badge_r.min, badge_r.size(), 1.2, tint);
        let glyph_size = (badge_r.height() * 0.40).clamp(10.0, 16.0);
        let badge_color = pal.badge_text;
        f.text(
            Vec2::new(
                badge_r.min.x + (badge_r.width() - badge.len() as f32 * glyph_size * 0.61) * 0.5,
                badge_r.min.y + (badge_r.height() - glyph_size) * 0.46,
            ),
            badge,
            glyph_size,
            badge_color,
        );
    }

    let text_left = (badge_r.max.x + 9.0).min(r.max.x - 4.0);
    let available = (r.max.x - text_left - 8.0).max(0.0);
    let compact = r.height() < 66.0;
    let title_size = if compact { 14.0_f32 } else { 17.0 }.min((r.height() * 0.36).max(10.0));
    let detail_size = if compact { 11.0_f32 } else { 12.0 }.min((r.height() * 0.27).max(9.0));
    let title = fit_ascii(
        title,
        (available / (title_size * 0.61)).floor().max(0.0) as usize,
    );
    let detail = fit_ascii(
        detail,
        (available / (detail_size * 0.61)).floor().max(0.0) as usize,
    );
    let y_title = if compact {
        r.min.y + 7.0
    } else {
        r.min.y + 9.0
    };
    let title_color = pal.title_text;
    f.text(
        Vec2::new(text_left, y_title),
        title,
        title_size,
        title_color,
    );
    if !detail.is_empty() && r.height() >= 38.0 {
        let detail_y = if compact {
            r.min.y + r.height() * 0.56
        } else {
            r.min.y + 35.0
        };
        let detail_color = pal.detail_text;
        f.text(
            Vec2::new(text_left, detail_y.min(r.max.y - detail_size - 4.0)),
            detail,
            detail_size,
            detail_color,
        );
    }
}

/// Add a small outline and corner marks around a hand, lane, or result area.
pub fn highlight(f: &mut Frame, r: Rect, kind: ResultKind, age: Option<f32>, reduced: bool) {
    let Some(r) = clipped_rect(f, r) else { return };
    let tint = kind_color(kind, f.palette);
    let pulse = if reduced {
        0.0
    } else {
        age.filter(|v| v.is_finite()).unwrap_or(1.0).clamp(0.0, 2.0)
    };
    let width = if pulse < 0.55 { 2.4 } else { 1.5 };
    f.outline(r.min, r.size(), width, tint);
    let len = (r.width().min(r.height()) * 0.17).clamp(4.0, 12.0);
    for (x, y, sx, sy) in [
        (r.min.x, r.min.y, 1.0, 1.0),
        (r.max.x, r.min.y, -1.0, 1.0),
        (r.max.x, r.max.y, -1.0, -1.0),
        (r.min.x, r.max.y, 1.0, -1.0),
    ] {
        f.line(
            Vec2::new(x, y),
            Vec2::new(x + sx * len, y),
            width + 0.6,
            tint,
        );
        f.line(
            Vec2::new(x, y),
            Vec2::new(x, y + sy * len),
            width + 0.6,
            tint,
        );
    }
}

/// Draw a short floating chip marker for an action such as DOUBLE or SPLIT.
/// The marker expires after 1.2 seconds.
pub fn action(f: &mut Frame, center: Vec2, label: &str, age: f32, reduced: bool, color: Color) {
    if !center.is_finite() || !age.is_finite() || !(0.0..1.2).contains(&age) || !f.size.is_finite()
    {
        return;
    }
    let progress = age / 1.2;
    let rise = if reduced { 0.0 } else { -12.0 * progress };
    let alpha = if reduced {
        1.0
    } else {
        (1.0 - (progress * 0.72).powi(2)).clamp(0.2, 1.0)
    };
    let label = fit_ascii(label, 18);
    let width = (label.chars().count() as f32 * 7.6 + 28.0).clamp(62.0, 164.0);
    let height = 27.0;
    let margin = if reduced { 0.0 } else { 3.0 };
    let pos = Vec2::new(
        (center.x - width * 0.5).clamp(0.0, (f.size.x - width - margin * 2.0).max(0.0)) + margin,
        (center.y + rise - height * 0.5).clamp(0.0, (f.size.y - height - margin * 2.0).max(0.0))
            + margin,
    );
    let bounds = Rect::from_corners(pos, pos + Vec2::new(width, height));
    if clipped_rect(f, bounds).is_none() {
        return;
    }
    let pal = super::game_palette::casino_fx(f.palette);
    let tint = with_alpha(color, alpha);
    let chip_bg = with_alpha(pal.chip_bg, alpha);
    f.rect(pos, Vec2::new(width, height), chip_bg);
    f.outline(pos, Vec2::new(width, height), 1.4, tint);
    if !reduced {
        let pulse = (1.0 - progress).max(0.0) * 1.8;
        f.outline(
            pos - Vec2::splat(pulse),
            Vec2::new(width + pulse * 2.0, height + pulse * 2.0),
            0.8,
            with_alpha(color, alpha * 0.55),
        );
    }
    // A crisp diamond glyph avoids relying on a font symbol unsupported by the bitmap font.
    let c = Vec2::new(pos.x + 12.0, pos.y + height * 0.5);
    let d = 4.0;
    f.line(c + Vec2::new(0.0, -d), c + Vec2::new(d, 0.0), 1.4, tint);
    f.line(c + Vec2::new(d, 0.0), c + Vec2::new(0.0, d), 1.4, tint);
    f.line(c + Vec2::new(0.0, d), c + Vec2::new(-d, 0.0), 1.4, tint);
    f.line(c + Vec2::new(-d, 0.0), c + Vec2::new(0.0, -d), 1.4, tint);
    let chip_text_color = pal.chip_text;
    f.text(
        Vec2::new(pos.x + 22.0, pos.y + 6.0),
        label,
        12.0,
        chip_text_color,
    );
}

fn winning_fx(f: &mut Frame, r: Rect, t: f32, premium: bool) {
    let pal = super::game_palette::casino_fx(f.palette);
    let phase = (t / 2.0).clamp(0.0, 1.0);
    let center = r.center();
    let base = if premium {
        pal.win_base_premium
    } else {
        pal.win_base_regular
    };
    let rings = if premium { 3 } else { 2 };
    for i in 0..rings {
        let radius = (8.0 + phase * r.height().min(r.width()) * (0.30 + i as f32 * 0.16))
            .min((r.height().min(r.width()) * 0.5 - 3.0).max(1.0));
        circle(
            f,
            center,
            radius,
            1.0,
            with_alpha(base, 0.32 * (1.0 - phase)),
        );
    }
    // Deterministic bounded spark field, kept to 30 points in the banner interior.
    for i in 0..30 {
        let h = hash(i as u32 + 17);
        let x0 = r.min.x + 7.0 + (h & 0xffff) as f32 / 65535.0 * (r.width() - 14.0).max(0.0);
        let y0 =
            r.min.y + 5.0 + ((h >> 16) & 0xffff) as f32 / 65535.0 * (r.height() - 10.0).max(0.0);
        let speed = 7.0 + (i % 5) as f32 * 2.0;
        let drift = if premium { 0.0 } else { speed * phase };
        let x = (x0 + (i as f32 * 1.7).sin() * drift)
            .clamp(r.min.x + 3.0, (r.max.x - 9.0).max(r.min.x + 3.0));
        let y = (y0 - speed * phase).clamp(r.min.y + 3.0, r.max.y - 3.0);
        let particle_color = if premium && i % 3 == 0 {
            pal.win_sparkle
        } else {
            base
        };
        let alpha = (0.65 * (1.0 - phase * 0.7)).clamp(0.0, 0.65);
        f.rect(
            Vec2::new(x, y),
            Vec2::splat(if i % 4 == 0 { 2.5 } else { 1.5 }),
            with_alpha(particle_color, alpha),
        );
        if i % 6 == 0 {
            let len = if premium { 6.0 } else { 4.0 };
            f.line(
                Vec2::new(x - len, y),
                Vec2::new(x + len, y),
                0.8,
                with_alpha(particle_color, alpha * 0.7),
            );
        }
    }
    // Short shimmer streaks live in the border band, away from the text block.
    for i in 0..3 {
        let x = r.min.x + ((phase * r.width() + i as f32 * r.width() / 3.0) % r.width().max(1.0));
        f.line(
            Vec2::new(x, r.min.y + 4.0),
            Vec2::new((x + 10.0).min(r.max.x - 3.0), r.min.y + 4.0),
            1.0,
            with_alpha(base, 0.22 * (1.0 - phase)),
        );
    }
}

fn push_fx(f: &mut Frame, r: Rect, t: f32) {
    let alpha = 0.32 * (1.0 - (t / 2.0).clamp(0.0, 1.0));
    let center = r.center();
    let size = 9.0 + t * 5.0;
    let c = kind_color(ResultKind::Push, f.palette);
    f.line(
        center + Vec2::new(0.0, -size),
        center + Vec2::new(size, 0.0),
        1.5,
        with_alpha(c, alpha),
    );
    f.line(
        center + Vec2::new(size, 0.0),
        center + Vec2::new(0.0, size),
        1.5,
        with_alpha(c, alpha),
    );
    f.line(
        center + Vec2::new(0.0, size),
        center + Vec2::new(-size, 0.0),
        1.5,
        with_alpha(c, alpha),
    );
    f.line(
        center + Vec2::new(-size, 0.0),
        center + Vec2::new(0.0, -size),
        1.5,
        with_alpha(c, alpha),
    );
    let inset = 5.0 + (t * 5.0).sin().abs() * 2.0;
    f.line(
        Vec2::new(r.min.x + inset, r.min.y + 5.0),
        Vec2::new(r.max.x - inset, r.min.y + 5.0),
        1.0,
        with_alpha(c, alpha * 0.7),
    );
}

fn loss_fx(f: &mut Frame, r: Rect, t: f32) {
    let color = kind_color(ResultKind::Loss, f.palette);
    let pulse = (t * 5.0).sin().abs();
    f.rect(
        r.min,
        Vec2::new(3.0 + pulse * 1.5, r.height()),
        with_alpha(color, 0.12 + pulse * 0.08),
    );
    f.rect(
        Vec2::new(r.max.x - 3.0, r.min.y),
        Vec2::new(3.0, r.height()),
        with_alpha(color, 0.12 + pulse * 0.08),
    );
    let sweep_x = r.min.x + (t / 2.0) * r.width();
    let sweep_y = r.min.y + 4.0;
    f.line(
        Vec2::new(sweep_x, sweep_y),
        Vec2::new((sweep_x + 12.0).min(r.max.x), sweep_y),
        1.0,
        with_alpha(color, 0.35),
    );
    // Restrained cross sweep placed over the quiet border band.
    let cross = 5.0;
    f.line(
        Vec2::new(r.max.x - 12.0, r.min.y + 12.0),
        Vec2::new(r.max.x - 12.0 + cross, r.min.y + 12.0 + cross),
        1.0,
        with_alpha(color, 0.42),
    );
    f.line(
        Vec2::new(r.max.x - 12.0 + cross, r.min.y + 12.0),
        Vec2::new(r.max.x - 12.0, r.min.y + 12.0 + cross),
        1.0,
        with_alpha(color, 0.42),
    );
}

fn terminal_corners(f: &mut Frame, r: Rect, color: Color, len: f32) {
    if len <= 0.0 {
        return;
    }
    for (x, y, sx, sy) in [
        (r.min.x, r.min.y, 1.0, 1.0),
        (r.max.x, r.min.y, -1.0, 1.0),
        (r.max.x, r.max.y, -1.0, -1.0),
        (r.min.x, r.max.y, 1.0, -1.0),
    ] {
        f.line(Vec2::new(x, y), Vec2::new(x + sx * len, y), 2.0, color);
        f.line(Vec2::new(x, y), Vec2::new(x, y + sy * len), 2.0, color);
    }
}

fn circle(f: &mut Frame, center: Vec2, radius: f32, width: f32, color: Color) {
    if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
        return;
    }
    let mut prev = center + Vec2::new(radius, 0.0);
    for i in 1..=20 {
        let a = i as f32 * std::f32::consts::TAU / 20.0;
        let next = center + Vec2::new(a.cos(), a.sin()) * radius;
        f.line(prev, next, width, color);
        prev = next;
    }
}

fn clipped_rect(f: &Frame, r: Rect) -> Option<Rect> {
    if !r.min.is_finite() || !r.max.is_finite() || !f.size.is_finite() {
        return None;
    }
    let min = r.min.max(Vec2::ZERO);
    let max = r.max.min(f.size);
    (max.x > min.x && max.y > min.y).then(|| Rect::from_corners(min, max))
}

fn badge(kind: ResultKind) -> &'static str {
    match kind {
        ResultKind::Win => "WIN",
        ResultKind::Natural => "NAT",
        ResultKind::BigWin => "8:1",
        ResultKind::Push => "PUSH",
        ResultKind::Loss => "LOSS",
        ResultKind::GameOver => "END",
    }
}

fn fit_ascii(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let clean: String = text
        .chars()
        .map(|c| {
            if c.is_ascii() && !c.is_control() {
                c
            } else {
                ' '
            }
        })
        .collect();
    if clean.chars().count() <= max_chars {
        return clean;
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }
    let mut result: String = clean.chars().take(max_chars - 3).collect();
    result.push_str("...");
    result
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    let c = color.to_srgba();
    Color::srgba(c.red, c.green, c.blue, (c.alpha * alpha).clamp(0.0, 1.0))
}

fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamevex_core::ColorScheme;

    #[test]
    fn kind_color_provides_distinct_finite_colors_for_all_schemes() {
        for scheme in [
            ColorScheme::Cyan,
            ColorScheme::Amber,
            ColorScheme::Emerald,
            ColorScheme::Violet,
            ColorScheme::Paper,
        ] {
            for kind in [
                ResultKind::Win,
                ResultKind::Natural,
                ResultKind::BigWin,
                ResultKind::Push,
                ResultKind::Loss,
                ResultKind::GameOver,
            ] {
                let c = kind_color(kind, scheme).to_srgba();
                assert!(c.red.is_finite() && (0.0..=1.0).contains(&c.red));
                assert!(c.green.is_finite() && (0.0..=1.0).contains(&c.green));
                assert!(c.blue.is_finite() && (0.0..=1.0).contains(&c.blue));
            }
        }
    }
}
