use super::*;

pub(crate) fn settings(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let t = f.theme();
    f.text(
        Vec2::new(m, 102.0),
        "Tune the cabinet",
        if w < 600.0 { 29.0 } else { 42.0 },
        t.ink,
    );
    let top = 184.0;
    let bottom = f.size.y - 94.0;
    l.settings_clip = Some(rect(m, top, w, bottom - top));

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

    let is_wide = w >= 600.0;
    let theme_cols = if is_wide { 3 } else { 2 };
    let theme_rows = crate::theme::THEME_REGISTRY.len().div_ceil(theme_cols);
    let tile_h = 36.0;
    let theme_gap_x = 8.0;
    let theme_gap_y = 6.0;
    let label_width = if is_wide { 230.0 } else { w * 0.43 };
    let right = m + label_width;
    let usable = (w - label_width).min(440.0);
    let grid_w = if is_wide {
        (w - label_width).min(560.0)
    } else {
        w
    };
    let tile_w = (grid_w - (theme_cols - 1) as f32 * theme_gap_x) / theme_cols as f32;
    let grid_h = theme_rows as f32 * tile_h + (theme_rows - 1) as f32 * theme_gap_y;
    let theme_label_h = if is_wide { 0.0 } else { 24.0 };
    let theme_row_h = if is_wide {
        grid_h + 16.0
    } else {
        theme_label_h + 4.0 + grid_h + 10.0
    };

    let row_count = labels.len();
    let row_h_at = |idx: usize| -> f32 { if idx == 1 { theme_row_h } else { 46.0 } };
    let mut y_offsets = vec![0.0; row_count];
    let mut acc_h = 0.0;
    for (idx, offset) in y_offsets.iter_mut().enumerate() {
        *offset = acc_h;
        acc_h += row_h_at(idx);
    }
    let total_rows_h = acc_h;
    let total_content_height = total_rows_h + 30.0;

    l.settings_max_scroll = (total_content_height - (bottom - top)).max(0.0);
    s.settings_scroll = s.settings_scroll.clamp(0.0, l.settings_max_scroll);
    if l.settings_max_scroll > 0.0 {
        f.text(
            Vec2::new(m, 154.0),
            "Scroll or PgUp/PgDn for more settings.",
            if w < 400.0 { 11.0 } else { 13.0 },
            t.muted,
        );
    }

    for (i, label) in labels.iter().enumerate() {
        let y = top + y_offsets[i] - s.settings_scroll;
        let h = row_h_at(i);

        if i == 1 {
            // Color scheme section with responsive preview grid
            if is_wide {
                if y + 8.0 >= top - 0.5 && y + 30.0 <= bottom + 0.5 {
                    f.text(Vec2::new(m, y + 8.0), *label, 16.0, t.ink);
                }
            } else if y + 4.0 >= top - 0.5 && y + theme_label_h <= bottom + 0.5 {
                f.text(
                    Vec2::new(m, y + 4.0),
                    *label,
                    if w < 400.0 { 12.0 } else { 14.0 },
                    t.ink,
                );
            }

            if y + h - 5.0 >= top - 0.5 && y + h - 5.0 <= bottom + 0.5 {
                f.line(
                    Vec2::new(m, y + h - 5.0),
                    Vec2::new(m + w, y + h - 5.0),
                    0.75,
                    t.edge,
                );
            }

            for (tile_idx, meta) in crate::theme::THEME_REGISTRY.iter().enumerate() {
                let col = tile_idx % theme_cols;
                let row = tile_idx / theme_cols;
                let tile_x = if is_wide {
                    right + col as f32 * (tile_w + theme_gap_x)
                } else {
                    m + col as f32 * (tile_w + theme_gap_x)
                };
                let tile_y = if is_wide {
                    y + 8.0 + row as f32 * (tile_h + theme_gap_y)
                } else {
                    y + theme_label_h + 4.0 + row as f32 * (tile_h + theme_gap_y)
                };
                let tile_rect = rect(tile_x, tile_y, tile_w, tile_h);
                theme_tile(f, s, l, tile_rect, meta, top, bottom);
            }
            continue;
        }

        if y < top - 0.5 || y + h > bottom + 0.5 {
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
            t.ink,
        );
        f.line(
            Vec2::new(m, y + h - 5.0),
            Vec2::new(m + w, y + h - 5.0),
            0.75,
            t.edge,
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
        let thumb = (track * (track / total_content_height)).max(16.0);
        let y = top + (track - thumb) * s.settings_scroll / l.settings_max_scroll;
        f.rect(Vec2::new(m + w - 2.0, y), Vec2::new(2.0, thumb), t.accent);
    }
    let storage_y = top + total_rows_h - s.settings_scroll;
    if storage_y >= top && storage_y + 22.0 <= bottom {
        f.text(
            Vec2::new(m, storage_y + 4.0),
            compact(&platform::description(), (w / 7.4) as usize),
            11.0,
            t.muted,
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

fn theme_tile(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    meta: &crate::theme::ThemeMeta,
    top: f32,
    bottom: f32,
) {
    if r.min.y < top - 0.5 || r.max.y > bottom + 0.5 {
        return;
    }
    let t = f.theme();
    let tile_theme = crate::theme::Theme::for_scheme(meta.scheme);
    let idx = l.hits.len();
    let focused = s.keyboard && s.focus == idx;
    let hovered = s.cursor.is_some_and(|p| r.contains(p));
    let is_selected = s.save.settings.color_scheme == meta.scheme;

    // Theme's own surface
    let fill = if hovered || focused {
        tile_theme.panel_hover
    } else {
        tile_theme.panel
    };
    f.rect(r.min, r.size(), fill);

    // Obvious keyboard focus and border outlines
    if focused {
        f.outline(r.min, r.size(), 2.5, t.gold);
    } else if is_selected {
        f.outline(r.min, r.size(), 2.0, tile_theme.accent);
    } else if hovered {
        f.outline(r.min, r.size(), 1.0, tile_theme.ink);
    } else {
        f.outline(r.min, r.size(), 1.0, tile_theme.edge);
    }

    // Selected indicator: left accent bar
    if is_selected {
        f.rect(r.min, Vec2::new(3.5, r.height()), tile_theme.accent);
    }

    // Selected indicator: pip
    let pip_size = 7.0;
    let pip_pos = Vec2::new(r.min.x + 9.0, r.min.y + (r.height() - pip_size) * 0.5);
    if is_selected {
        f.rect(pip_pos, Vec2::splat(pip_size), tile_theme.accent);
    } else {
        f.outline(pip_pos, Vec2::splat(pip_size), 1.0, tile_theme.edge);
    }

    // Theme's own text: name
    let font = if r.width() < 130.0 { 11.0 } else { 12.0 };
    let text_color = if is_selected || focused {
        tile_theme.ink
    } else {
        tile_theme.control_text
    };
    let text_x = r.min.x + 22.0;
    let text_y = r.min.y + (r.height() - font * 1.2) * 0.5;
    f.text(Vec2::new(text_x, text_y), meta.name, font, text_color);

    // Accent, success, and danger swatches
    let swatch_size = 8.0;
    let swatch_gap = 3.5;
    let swatches = [tile_theme.accent, tile_theme.success, tile_theme.danger];
    let total_swatches_w =
        swatches.len() as f32 * swatch_size + (swatches.len() - 1) as f32 * swatch_gap;
    let swatch_start_x = r.max.x - total_swatches_w - 7.0;
    let swatch_y = r.min.y + (r.height() - swatch_size) * 0.5;

    for (si, &color) in swatches.iter().enumerate() {
        let sp = Vec2::new(
            swatch_start_x + si as f32 * (swatch_size + swatch_gap),
            swatch_y,
        );
        f.rect(sp, Vec2::splat(swatch_size), color);
        f.outline(sp, Vec2::splat(swatch_size), 0.5, tile_theme.edge);
    }

    l.hits.push(Hit {
        rect: r,
        action: Action::SelectTheme(meta.scheme),
    });
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
        f.theme().accent,
    );
    button(f, s, l, rect(x + w - 34.0, y, 34.0, 31.0), "+", up, false);
}
