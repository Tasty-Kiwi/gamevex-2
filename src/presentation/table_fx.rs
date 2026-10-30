//! Short-lived table feedback; every primitive stays inside the playing surface.
use crate::{
    rendering::Frame,
    table_feedback::{Anchor, EffectKind as K, TableFeedback},
};
use bevy::prelude::*;
use std::f32::consts::TAU;

pub(super) fn draw(
    f: &mut Frame,
    clip: Rect,
    feedback: &TableFeedback,
    locate: impl Fn(Anchor) -> Option<Rect>,
    reduced: bool,
) {
    for event in &feedback.events {
        let progress = (event.age / event.duration()).clamp(0.0, 1.0);
        let alpha = (1.0 - progress).powi(2);
        let rgb = match event.kind {
            K::Invalid | K::Stuck => [1.0, 0.24, 0.32],
            K::Win | K::Run | K::Hint => [1.0, 0.78, 0.15],
            K::Move => [0.12, 1.0, 0.68],
            K::Undo | K::Shuffle => [0.9, 0.28, 1.0],
            _ => [0.0, 0.88, 1.0],
        };
        let color = Color::srgba(rgb[0], rgb[1], rgb[2], alpha);
        let areas: Vec<_> = event
            .anchors
            .iter()
            .filter_map(|&a| locate(a))
            .filter(|r| {
                r.max.x > clip.min.x
                    && r.min.x < clip.max.x
                    && r.max.y > clip.min.y
                    && r.min.y < clip.max.y
            })
            .collect();
        for (i, area) in areas.iter().enumerate() {
            let center = area.center();
            let expand = if reduced { 2.0 } else { progress * 14.0 + 2.0 };
            outline(
                f,
                clip,
                Rect {
                    min: area.min - Vec2::splat(expand),
                    max: area.max + Vec2::splat(expand),
                },
                color,
            );
            if reduced {
                continue;
            }
            let radius = area.width().min(area.height()) * 0.28 + 35.0 * progress;
            let rays = if matches!(event.kind, K::Move | K::Run | K::Win) {
                12
            } else {
                6
            };
            for j in 0..rays {
                let angle = j as f32 / rays as f32 * TAU + i as f32 * 0.4;
                let direction = Vec2::new(angle.cos(), angle.sin());
                let p = center + direction * radius;
                line(
                    f,
                    clip,
                    p,
                    p + direction * (3.0 + 8.0 * (1.0 - progress)),
                    color,
                );
            }
            if event.kind == K::Invalid {
                let x = area.min.x + area.width() * 0.5;
                let y = area.min.y + area.height() * 0.5;
                line(
                    f,
                    clip,
                    Vec2::new(x - 8.0, y - 8.0),
                    Vec2::new(x + 8.0, y + 8.0),
                    color,
                );
                line(
                    f,
                    clip,
                    Vec2::new(x + 8.0, y - 8.0),
                    Vec2::new(x - 8.0, y + 8.0),
                    color,
                );
            }
        }
        if !reduced && areas.len() >= 2 {
            let from = areas[0].center();
            let to = areas[1].center();
            for j in 0..8 {
                let t = (progress * 1.6 - j as f32 * 0.055).clamp(0.0, 1.0);
                let p = from.lerp(to, t) + Vec2::new(0.0, -(t * std::f32::consts::PI).sin() * 20.0);
                dot(f, clip, p, (3.0 - j as f32 * 0.25).max(1.0), color);
            }
        }
        if matches!(
            event.kind,
            K::Win | K::Run | K::Shuffle | K::Deal | K::Stuck
        ) && areas.is_empty()
        {
            let inset = if reduced { 3.0 } else { 3.0 + progress * 18.0 };
            outline(
                f,
                clip,
                Rect {
                    min: clip.min + Vec2::splat(inset),
                    max: clip.max - Vec2::splat(inset),
                },
                color,
            );
        }
        if event.kind == K::Win && !reduced {
            // Three deterministic fountains, at most 72 particles, no retained emitters.
            for fountain in 0..3 {
                let origin = Vec2::new(
                    clip.min.x + clip.width() * (fountain as f32 + 0.5) / 3.0,
                    clip.max.y - 12.0,
                );
                for j in 0..24 {
                    let phase = j as f32 / 24.0;
                    let speed = 65.0 + (j % 5) as f32 * 17.0;
                    let angle = -std::f32::consts::PI * (0.18 + phase * 0.64);
                    let t = event.age;
                    let p = origin
                        + Vec2::new(
                            angle.cos() * speed * t,
                            angle.sin() * speed * t + 40.0 * t * t,
                        );
                    let tint = if j % 3 == 0 {
                        Color::srgba(1.0, 0.2, 0.65, alpha)
                    } else {
                        color
                    };
                    dot(f, clip, p, 2.0, tint);
                }
            }
        }
        let caption = match event.kind {
            K::Run => Some("RUN COMPLETE"),
            K::Win => Some("TABLE CLEARED!"),
            K::Shuffle => Some("SHUFFLED"),
            K::Undo => Some("RESTORED"),
            K::Stuck => Some("NO MOVES"),
            _ => None,
        };
        if let Some(caption) = caption {
            let size = if event.kind == K::Win { 20.0 } else { 13.0 };
            let width = caption.len() as f32 * size * 0.62 + 20.0;
            let r = Rect::from_center_size(clip.center(), Vec2::new(width, size + 20.0));
            if r.min.x >= clip.min.x
                && r.max.x <= clip.max.x
                && r.min.y >= clip.min.y
                && r.max.y <= clip.max.y
            {
                f.rect(r.min, r.size(), Color::srgba(0.015, 0.035, 0.05, 0.95));
                outline(f, clip, r, color);
                f.text(r.min + Vec2::splat(10.0), caption, size, color);
            }
        }
    }
}
fn dot(f: &mut Frame, clip: Rect, p: Vec2, r: f32, color: Color) {
    let min = (p - Vec2::splat(r)).max(clip.min);
    let max = (p + Vec2::splat(r)).min(clip.max);
    f.rect(min, max - min, color);
}
fn outline(f: &mut Frame, clip: Rect, r: Rect, color: Color) {
    for (a, b) in [
        (r.min, Vec2::new(r.max.x, r.min.y)),
        (Vec2::new(r.max.x, r.min.y), r.max),
        (r.max, Vec2::new(r.min.x, r.max.y)),
        (Vec2::new(r.min.x, r.max.y), r.min),
    ] {
        line(f, clip, a, b, color);
    }
}
fn line(f: &mut Frame, clip: Rect, a: Vec2, b: Vec2, color: Color) {
    let d = b - a;
    let mut lo: f32 = 0.0;
    let mut hi: f32 = 1.0;
    for (p, q) in [
        (-d.x, a.x - clip.min.x),
        (d.x, clip.max.x - a.x),
        (-d.y, a.y - clip.min.y),
        (d.y, clip.max.y - a.y),
    ] {
        if p.abs() < f32::EPSILON {
            if q < 0.0 {
                return;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                lo = lo.max(t);
            } else {
                hi = hi.min(t);
            }
            if lo > hi {
                return;
            }
        }
    }
    f.line(a + d * lo, a + d * hi, 1.4, color);
}
