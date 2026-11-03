use super::*;

pub(crate) fn settings(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    f.text(
        Vec2::new(m, 102.0),
        "Tune the cabinet",
        if w < 600.0 { 29.0 } else { 42.0 },
        INK,
    );
    let top = 184.0;
    let bottom = f.size.y - 94.0;
    l.settings_clip = Some(rect(m, top, w, bottom - top));
    let row_h = 46.0;
    let labels = [
        "Display",
        "Color scheme",
        "Bloom",
        "Scanlines",
        "Curvature",
        "Vignette",
        "Chromatic aberration",
        "CRT effects",
        "Reduced motion",
        "Volume",
        "Sound",
    ];
    l.settings_max_scroll = (labels.len() as f32 * row_h + 30.0 - (bottom - top)).max(0.0);
    s.settings_scroll = s.settings_scroll.clamp(0.0, l.settings_max_scroll);
    if l.settings_max_scroll > 0.0 {
        f.text(
            Vec2::new(m, 154.0),
            "Scroll or PgUp/PgDn for more settings.",
            if w < 400.0 { 11.0 } else { 13.0 },
            MUTED,
        );
    }
    let label_width = if w < 600.0 { w * 0.43 } else { 230.0 };
    let right = m + label_width;
    let usable = (w - label_width).min(440.0);
    for (i, label) in labels.iter().enumerate() {
        let y = top + i as f32 * row_h - s.settings_scroll;
        if y < top - 0.5 || y + row_h > bottom + 0.5 {
            continue;
        }
        f.text(
            Vec2::new(m, y + 8.0),
            *label,
            if w < 400.0 {
                if i == 6 { 11.0 } else { 12.0 }
            } else {
                16.0
            },
            INK,
        );
        f.line(
            Vec2::new(m, y + row_h - 5.0),
            Vec2::new(m + w, y + row_h - 5.0),
            0.75,
            EDGE,
        );
        match i {
            0 => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.display_preset == DisplayPreset::VectorCrt {
                    "Vector CRT"
                } else {
                    "Pixel CRT"
                },
                Action::Preset,
                true,
            ),
            1 => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.color_scheme == ColorScheme::Cyan {
                    "Cyan"
                } else {
                    "Amber"
                },
                Action::Palette,
                true,
            ),
            2 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.bloom,
                Action::Bloom(-0.1),
                Action::Bloom(0.1),
            ),
            3 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.scanlines,
                Action::Scanlines(-0.1),
                Action::Scanlines(0.1),
            ),
            4 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.curvature,
                Action::Curvature(-0.1),
                Action::Curvature(0.1),
            ),
            5 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.vignette,
                Action::Vignette(-0.1),
                Action::Vignette(0.1),
            ),
            6 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.chromatic_aberration,
                Action::ChromaticAberration(-0.1),
                Action::ChromaticAberration(0.1),
            ),
            7 => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.crt {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Action::Crt,
                s.save.settings.crt,
            ),
            8 => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.reduced_motion {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Action::Motion,
                s.save.settings.reduced_motion,
            ),
            9 => stepper(
                f,
                s,
                l,
                right,
                y,
                usable,
                s.save.settings.volume,
                Action::Volume(-0.1),
                Action::Volume(0.1),
            ),
            _ => button(
                f,
                s,
                l,
                rect(right, y, usable, 31.0),
                if s.save.settings.muted { "Muted" } else { "On" },
                Action::Mute,
                !s.save.settings.muted,
            ),
        }
    }
    if l.settings_max_scroll > 0.0 {
        let track = bottom - top;
        let thumb = track * (track / (labels.len() as f32 * row_h + 30.0));
        let y = top + (track - thumb) * s.settings_scroll / l.settings_max_scroll;
        f.rect(Vec2::new(m + w - 2.0, y), Vec2::new(2.0, thumb), CYAN);
    }
    let storage_y = top + labels.len() as f32 * row_h - s.settings_scroll;
    if storage_y >= top && storage_y + 22.0 <= bottom {
        f.text(
            Vec2::new(m, storage_y + 4.0),
            compact(&platform::description(), (w / 7.4) as usize),
            11.0,
            MUTED,
        );
    }
    let y = f.size.y - 80.0;
    button(f, s, l, rect(m, y, 70.0, 34.0), "Back", Action::Back, false);
    button(
        f,
        s,
        l,
        rect(m + 78.0, y, 90.0, 34.0),
        "Retry save",
        Action::Retry,
        false,
    );
    button(
        f,
        s,
        l,
        rect(m + 176.0, y, (w - 176.0).min(190.0), 34.0),
        "Reset saved data",
        Action::Recover,
        false,
    );
}

#[allow(clippy::too_many_arguments)]
fn stepper(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    x: f32,
    y: f32,
    w: f32,
    value: f32,
    down: Action,
    up: Action,
) {
    button(f, s, l, rect(x, y, 34.0, 31.0), "-", down, false);
    f.text(
        Vec2::new(x + w * 0.5 - 20.0, y + 6.0),
        format!("{:.0}%", value * 100.0),
        15.0,
        CYAN,
    );
    button(f, s, l, rect(x + w - 34.0, y, 34.0, 31.0), "+", up, false);
}
