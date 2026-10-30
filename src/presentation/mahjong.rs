use super::*;
use gamevex_core::mahjong::Layout;

const FELT: Color = Color::srgb(0.027, 0.035, 0.055);
const FACE: Color = Color::srgb(0.086, 0.102, 0.133);
const NEON_CYAN: Color = Color::srgb(0.0, 1.0, 1.0);
const NEON_PINK: Color = Color::srgb(1.0, 0.0, 0.60);
const NEON_GREEN: Color = Color::srgb(0.0, 1.0, 0.60);
const NEON_GOLD: Color = Color::srgb(1.0, 0.77, 0.0);

pub(super) fn mahjong(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let game = &s.save.mahjong;
    let small = w < 620.0;
    let top = 96.0;
    f.text(
        Vec2::new(m, top),
        "MAHJONG SOLITAIRE",
        if small { 24.0 } else { 31.0 },
        INK,
    );
    f.text(
        Vec2::new(m, top + 31.0),
        format!("{}  /  {} tiles", game.layout.label(), game.remaining()),
        12.0,
        NEON_CYAN,
    );
    f.text(
        Vec2::new(m + if small { 190.0 } else { 240.0 }, top + 31.0),
        if game.zen {
            "UNTIMED".into()
        } else {
            format_time(game.elapsed_ms)
        },
        12.0,
        NEON_GOLD,
    );
    let status = if game.won() {
        "SOLVED"
    } else if game.stuck() {
        "NO MOVES"
    } else {
        "MATCH FREE ENDS"
    };
    f.text(
        Vec2::new(m, top + 51.0),
        status,
        10.0,
        if game.won() {
            GREEN
        } else if game.stuck() {
            GOLD
        } else {
            MUTED
        },
    );
    let layout_index = Layout::ALL
        .iter()
        .position(|&layout| layout == game.layout)
        .unwrap_or(0);
    if !game.zen {
        let best = s.save.mahjong_records[layout_index]
            .map(format_time)
            .unwrap_or_else(|| "--:--".into());
        f.text(
            Vec2::new(m + if small { 150.0 } else { 210.0 }, top + 51.0),
            format!("BEST {best}"),
            10.0,
            MUTED,
        );
    } else {
        f.text(
            Vec2::new(m + if small { 150.0 } else { 210.0 }, top + 51.0),
            "ZEN / AUTO-SHUFFLE",
            10.0,
            NEON_CYAN,
        );
    }

    let control_y = top + 68.0;
    let control_h = 32.0;
    let gap = 5.0;
    let labels = ["Hint", "Undo", "Shuffle", "Restart", "New"];
    let actions = [
        Action::MahjongHint,
        Action::MahjongUndo,
        Action::MahjongShuffle,
        Action::MahjongRestart,
        Action::MahjongNew(game.layout),
    ];
    let bw = ((w - gap * 4.0) / 5.0).clamp(58.0, 104.0);
    for i in 0..5 {
        let enabled = match i {
            0 => !game.won(),
            1 => !game.history.is_empty(),
            2 => game.stuck() && game.remaining() > 0,
            _ => true,
        };
        let r = rect(m + i as f32 * (bw + gap), control_y, bw, control_h);
        if enabled {
            button(f, s, l, r, labels[i], actions[i], false);
        } else {
            f.rect(r.min, r.size(), PANEL);
            f.outline(r.min, r.size(), 1.0, EDGE);
            let fs = if r.width() < 150.0 { 13.0 } else { 14.0 };
            f.text(
                r.min
                    + Vec2::new(
                        (r.width() - labels[i].len() as f32 * fs * 0.6) * 0.5,
                        (r.height() - fs * 1.2) * 0.5,
                    ),
                labels[i],
                fs,
                MUTED,
            );
        }
    }
    // Layout choices sit on a second compact row and stay explicit on phones.
    let layout_y = control_y + control_h + 5.0;
    let layout_w = ((w - gap * (Layout::ALL.len().saturating_sub(1) as f32))
        / Layout::ALL.len().max(1) as f32)
        .min(116.0);
    let total =
        Layout::ALL.len() as f32 * layout_w + gap * Layout::ALL.len().saturating_sub(1) as f32;
    let layout_x = m + (w - total).max(0.0) * 0.5;
    for (i, layout) in Layout::ALL.iter().copied().enumerate() {
        button(
            f,
            s,
            l,
            rect(
                layout_x + i as f32 * (layout_w + gap),
                layout_y,
                layout_w,
                27.0,
            ),
            layout.label(),
            Action::MahjongNew(layout),
            game.layout == layout,
        );
    }
    let mode_y = layout_y + 31.0;
    let mode_w = ((w - gap) * 0.5).min(150.0);
    let mode_total = mode_w * 2.0 + gap;
    let mode_x = m + (w - mode_total).max(0.0) * 0.5;
    button(
        f,
        s,
        l,
        rect(mode_x, mode_y, mode_w, 26.0),
        "Timed",
        Action::MahjongZen(false),
        !game.zen,
    );
    button(
        f,
        s,
        l,
        rect(mode_x + mode_w + gap, mode_y, mode_w, 26.0),
        "Zen",
        Action::MahjongZen(true),
        game.zen,
    );

    let footer = f.size.y - 31.0;
    let board_top = mode_y + 33.0;
    let board_bottom = footer - 48.0;
    let available = rect(m, board_top, w, (board_bottom - board_top).max(32.0));
    let (min_x, _) = game
        .tiles
        .iter()
        .fold((0i16, 0i16), |(lo, hi), t| (lo.min(t.x), hi.max(t.x)));
    let (min_y, _) = game
        .tiles
        .iter()
        .fold((0i16, 0i16), |(lo, hi), t| (lo.min(t.y), hi.max(t.y)));
    // Core coordinates use half-tile steps for side edges and layered stacks.
    let unit_positions: Vec<Vec2> = game
        .tiles
        .iter()
        .map(|tile| {
            Vec2::new(
                (tile.x - min_x) as f32 * 0.5 - tile.z as f32 * 4.0 / 60.0,
                (tile.y - min_y) as f32 * 0.5 - tile.z as f32 * 6.0 / 60.0,
            )
        })
        .collect();
    let pos_min = unit_positions
        .iter()
        .copied()
        .reduce(Vec2::min)
        .unwrap_or(Vec2::ZERO);
    let pos_max = unit_positions
        .iter()
        .copied()
        .reduce(Vec2::max)
        .unwrap_or(Vec2::ZERO);
    let depth = 6.0 / 60.0;
    let board_units = pos_max - pos_min + Vec2::ONE * (1.0 + depth);
    let cell_w = (available.width() / board_units.x)
        .min(available.height() / board_units.y)
        .clamp(if small { 40.0 } else { 32.0 }, 66.0);
    let cell_h = cell_w;
    let board_size = board_units * cell_w;
    let board_w = board_size.x;
    let board_h = board_size.y;
    let clip_size = Vec2::new(
        board_w.min(available.width()),
        board_h.min(available.height()),
    );
    let clip = rect(
        available.min.x + (available.width() - clip_size.x) * 0.5,
        available.min.y,
        clip_size.x,
        clip_size.y,
    );
    l.mahjong_clip = Some(clip);
    l.mahjong_max_pan = Vec2::new(
        (board_w - clip.width()).max(0.0),
        (board_h - clip.height()).max(0.0),
    );
    if s.mahjong_ui.reveal_hint {
        if let Some([first, second]) = s.mahjong_ui.hint
            && let (Some(a), Some(b)) = (game.tiles.get(first), game.tiles.get(second))
        {
            let tile_pos = |tile: &gamevex_core::mahjong::Tile| {
                Vec2::new((tile.x - min_x) as f32 * 0.5, (tile.y - min_y) as f32 * 0.5)
                    - Vec2::new(tile.z as f32 * 4.0 / 60.0, tile.z as f32 * 6.0 / 60.0)
                    - pos_min
            };
            let tile_pos = |tile: &gamevex_core::mahjong::Tile| tile_pos(tile) * cell_w;
            let a_pos = tile_pos(a);
            let b_pos = tile_pos(b);
            let both_min = a_pos.min(b_pos);
            let both_max =
                (a_pos + Vec2::new(cell_w, cell_h)).max(b_pos + Vec2::new(cell_w, cell_h));
            let pair_fits =
                both_max.x - both_min.x <= clip.width() && both_max.y - both_min.y <= clip.height();
            let center = if pair_fits {
                (both_min + both_max) * 0.5
            } else {
                a_pos + Vec2::new(cell_w, cell_h) * 0.5
            };
            s.mahjong_ui.pan = (center - clip.size() * 0.5).clamp(Vec2::ZERO, l.mahjong_max_pan);
        }
        s.mahjong_ui.reveal_hint = false;
    }
    let pan = s.mahjong_ui.pan.clamp(Vec2::ZERO, l.mahjong_max_pan);
    s.mahjong_ui.pan = pan;
    let origin = clip.min - pan;
    f.rect(clip.min, clip.size(), FELT);

    let mut order: Vec<usize> = (0..game.tiles.len())
        .filter(|&i| !game.tiles[i].removed)
        .collect();
    order.sort_by_key(|&i| (game.tiles[i].z, game.tiles[i].y, game.tiles[i].x));
    let mut drawn = Vec::with_capacity(order.len());
    for &i in &order {
        let t = &game.tiles[i];
        let p = origin + unit_positions[i] * cell_w - pos_min * cell_w;
        let r = rect(p.x, p.y, cell_w, cell_h);
        if r.max.x <= clip.min.x
            || r.min.x >= clip.max.x
            || r.max.y <= clip.min.y
            || r.min.y >= clip.max.y
        {
            continue;
        }
        let selectable = game.selectable(i);
        let selected = s.mahjong_ui.selected == Some(i);
        let hinted = s.mahjong_ui.hint.is_some_and(|pair| pair.contains(&i));
        draw_tile(f, r, clip, cell_w, selectable, selected, hinted);
        draw_face_art(f, r, clip, t.face, cell_w, selectable);
        if s.mahjong_ui.selected == Some(i) {
            rounded_outline_clipped(
                f,
                r.min + Vec2::splat(2.0),
                r.size() - Vec2::splat(4.0),
                5.0,
                2.0,
                NEON_PINK,
                clip,
            );
        }
        if hinted {
            rounded_outline_clipped(
                f,
                r.min + Vec2::splat(4.0),
                r.size() - Vec2::splat(8.0),
                4.0,
                2.0,
                NEON_GOLD,
                clip,
            );
        }
        // Register every visible surface, including covered tiles, in painter order.
        let hit_bounds = rect(
            r.min.x,
            r.min.y,
            cell_w + cell_w * 0.1,
            cell_h + cell_h * 0.1,
        );
        let hit = rect(
            hit_bounds.min.x.max(clip.min.x),
            hit_bounds.min.y.max(clip.min.y),
            hit_bounds.max.x.min(clip.max.x) - hit_bounds.min.x.max(clip.min.x),
            hit_bounds.max.y.min(clip.max.y) - hit_bounds.min.y.max(clip.min.y),
        );
        l.hits.push(Hit {
            rect: hit,
            action: Action::MahjongTile(i),
        });
        let focused = s.keyboard
            && matches!(l.hits.get(s.focus).map(|hit| hit.action), Some(Action::MahjongTile(j)) if j == i);
        drawn.push((i, t.face, r, focused));
    }
    // Text is a separate layer in Frame. Suppress it only when a later tile face
    // or its protruding bevel actually covers the text's drawn bounds.
    for (at, &(_, face, r, focused)) in drawn.iter().enumerate() {
        let label = face_label(r, face, cell_w);
        let covered = drawn[at + 1..].iter().any(|(_, _, later, _)| {
            let bevel = cell_w * (6.0 / 60.0);
            label_overlaps_tile(label.bounds, *later, bevel)
        });
        if !covered {
            draw_face_labels(f, label, clip);
        }
        if focused {
            rounded_outline_clipped(f, r.min, r.size(), 6.0, 2.0, NEON_GOLD, clip);
        }
    }
    super::table_fx::draw(
        f,
        clip,
        &s.mahjong_ui.feedback,
        |anchor| match anchor {
            crate::table_feedback::Anchor::Tile(index) => game.tiles.get(index).map(|_| {
                let p = origin + unit_positions[index] * cell_w - pos_min * cell_w;
                rect(p.x, p.y, cell_w, cell_h)
            }),
            crate::table_feedback::Anchor::Pile(_) => None,
        },
        s.save.settings.reduced_motion,
    );
    if game.won() {
        let center = clip.center();
        let panel = rect(
            center.x - (clip.width() - 20.0).min(380.0) * 0.5,
            center.y - 52.0,
            (clip.width() - 20.0).min(380.0),
            104.0,
        );
        f.rect(panel.min, panel.size(), PANEL);
        f.outline(panel.min, panel.size(), 2.0, GREEN);
        if !s.save.settings.reduced_motion
            && let Some(age) = s.mahjong_ui.completion_age.filter(|a| *a < 1.2)
        {
            let spread = (age / 1.2) * 12.0;
            f.outline(
                panel.min - Vec2::splat(spread),
                panel.size() + Vec2::splat(spread * 2.0),
                1.0,
                GREEN.with_alpha(1.0 - age / 1.2),
            );
        }
        f.text(
            panel.min + Vec2::new(14.0, 14.0),
            "BOARD CLEARED",
            24.0,
            GREEN,
        );
        f.text(
            panel.min + Vec2::new(14.0, 50.0),
            if game.zen {
                format!("{} / ZEN UNTIMED", game.layout.label())
            } else {
                format!("{} / {}", game.layout.label(), format_time(game.elapsed_ms))
            },
            14.0,
            INK,
        );
        f.text(
            panel.min + Vec2::new(14.0, 77.0),
            if game.assisted {
                "ASSISTED RUN"
            } else {
                "UNASSISTED RUN"
            },
            11.0,
            GOLD,
        );
    }
    if game.zen {
        f.text(
            Vec2::new(m, footer - 43.0),
            "ZEN / AUTO-SHUFFLE",
            10.0,
            NEON_CYAN,
        );
    } else if game.assisted {
        f.text(
            Vec2::new(m, footer - 43.0),
            "ASSISTED / NO BEST",
            10.0,
            GOLD,
        );
    }
    if !game.zen && game.shuffles > 0 {
        f.text(Vec2::new(m + 150.0, footer - 43.0), "SHUFFLED", 10.0, MUTED);
    }
    if !s.mahjong_ui.message.is_empty() {
        f.text(
            Vec2::new(m, footer - 20.0),
            compact(&s.mahjong_ui.message, (w / 6.6) as usize),
            11.0,
            INK,
        );
    } else {
        f.text(
            Vec2::new(m, footer - 20.0),
            if small {
                "Match free ends. Swipe to pan. H: hint U: undo"
            } else {
                "Match free ends. H hint / U undo / S shuffle / R restart / Arrows pan"
            },
            if small { 9.0 } else { 11.0 },
            MUTED,
        );
    }
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}

fn label_overlaps_tile(label: Rect, face: Rect, bevel: f32) -> bool {
    let painted = rect(
        face.min.x,
        face.min.y,
        face.width() + bevel,
        face.height() + bevel,
    );
    rects_overlap(label, painted)
}

fn clipped_rect(f: &mut Frame, pos: Vec2, size: Vec2, color: Color, clip: Rect) {
    let min = pos.max(clip.min);
    let max = (pos + size).min(clip.max);
    if max.x > min.x && max.y > min.y {
        f.rect(min, max - min, color);
    }
}

fn rounded_fill_clipped(
    f: &mut Frame,
    pos: Vec2,
    size: Vec2,
    radius: f32,
    color: Color,
    clip: Rect,
) {
    if size.x <= 0.0 || size.y <= 0.0 {
        return;
    }
    let radius = radius.clamp(0.0, size.min_element() * 0.5);
    if radius <= 0.5 {
        clipped_rect(f, pos, size, color, clip);
        return;
    }
    let steps = 8;
    let strip = radius / steps as f32;
    for row in 0..steps {
        let y0 = row as f32 * strip;
        let y1 = (row + 1) as f32 * strip;
        let y = (y0 + y1) * 0.5;
        let dy = (radius - y).abs();
        let inset = radius - (radius * radius - dy * dy).max(0.0).sqrt();
        clipped_rect(
            f,
            pos + Vec2::new(inset, y0),
            Vec2::new(size.x - inset * 2.0, strip + 0.25),
            color,
            clip,
        );
        let bottom_y0 = size.y - y1;
        clipped_rect(
            f,
            pos + Vec2::new(inset, bottom_y0),
            Vec2::new(size.x - inset * 2.0, strip + 0.25),
            color,
            clip,
        );
    }
    clipped_rect(
        f,
        pos + Vec2::new(0.0, radius),
        Vec2::new(size.x, size.y - radius * 2.0),
        color,
        clip,
    );
}

fn clipped_line(f: &mut Frame, a: Vec2, b: Vec2, width: f32, color: Color, clip: Rect) {
    let delta = b - a;
    let mut low: f32 = 0.0;
    let mut high: f32 = 1.0;
    for (p, q) in [
        (-delta.x, a.x - clip.min.x),
        (delta.x, clip.max.x - a.x),
        (-delta.y, a.y - clip.min.y),
        (delta.y, clip.max.y - a.y),
    ] {
        if p.abs() < f32::EPSILON {
            if q < 0.0 {
                return;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                low = low.max(t);
            } else {
                high = high.min(t);
            }
            if low > high {
                return;
            }
        }
    }
    f.line(a + delta * low, a + delta * high, width, color);
}

fn polyline(f: &mut Frame, points: &[Vec2], width: f32, color: Color, clip: Rect) {
    for pair in points.windows(2) {
        clipped_line(f, pair[0], pair[1], width, color, clip);
    }
}

fn circle(f: &mut Frame, center: Vec2, radius: f32, width: f32, color: Color, clip: Rect) {
    let points: Vec<Vec2> = (0..=32)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 32.0;
            center + Vec2::new(a.cos(), a.sin()) * radius
        })
        .collect();
    polyline(f, &points, width, color, clip);
}

fn rounded_outline_clipped(
    f: &mut Frame,
    pos: Vec2,
    size: Vec2,
    radius: f32,
    width: f32,
    color: Color,
    clip: Rect,
) {
    if size.x <= 0.0 || size.y <= 0.0 {
        return;
    }
    let radius = radius.clamp(0.0, size.min_element() * 0.5);
    let mut points = Vec::new();
    for (center, start) in [
        (pos + Vec2::new(size.x - radius, radius), -90.0_f32),
        (pos + size - Vec2::splat(radius), 0.0),
        (pos + Vec2::new(radius, size.y - radius), 90.0),
        (pos + Vec2::splat(radius), 180.0),
    ] {
        for step in 0..=5 {
            let angle = (start + 90.0 * step as f32 / 5.0).to_radians();
            points.push(center + Vec2::new(angle.cos(), angle.sin()) * radius);
        }
    }
    points.push(points[0]);
    polyline(f, &points, width, color, clip);
}

fn draw_tile(
    f: &mut Frame,
    r: Rect,
    clip: Rect,
    cell: f32,
    selectable: bool,
    selected: bool,
    hinted: bool,
) {
    let scale = cell / 60.0;
    let bevel = 6.0 * scale;
    let rim = if selected {
        NEON_PINK
    } else if hinted {
        NEON_GOLD
    } else if selectable {
        Color::srgb(0.28, 0.49, 0.56)
    } else {
        Color::srgb(0.31, 0.35, 0.39)
    };
    clipped_rect(
        f,
        r.min + Vec2::new(3.0 * scale, 4.0 * scale),
        r.size() + Vec2::splat(bevel),
        Color::srgba(0.0, 0.0, 0.0, 0.62),
        clip,
    );
    clipped_rect(
        f,
        r.min + Vec2::new(cell, 4.0 * scale),
        Vec2::new(bevel, cell - 2.0 * scale),
        Color::srgb(0.09, 0.11, 0.15),
        clip,
    );
    clipped_rect(
        f,
        r.min + Vec2::new(4.0 * scale, cell - 2.0 * scale),
        Vec2::new(cell - 4.0 * scale, bevel + 2.0 * scale),
        Color::srgb(0.10, 0.12, 0.16),
        clip,
    );
    rounded_fill_clipped(
        f,
        r.min,
        r.size(),
        7.0 * scale,
        if selectable {
            FACE
        } else {
            Color::srgb(0.063, 0.071, 0.090)
        },
        clip,
    );
    rounded_outline_clipped(f, r.min, r.size(), 7.0 * scale, 1.3 * scale, rim, clip);
    rounded_outline_clipped(
        f,
        r.min + Vec2::splat(4.0 * scale),
        r.size() - Vec2::splat(8.0 * scale),
        4.0 * scale,
        0.8 * scale,
        Color::srgba(0.72, 0.82, 0.91, 0.20),
        clip,
    );
    if !selectable {
        clipped_rect(
            f,
            r.min,
            Vec2::new(cell, 2.0 * scale),
            Color::srgba(0.02, 0.03, 0.04, 0.28),
            clip,
        );
    }
}

fn draw_face_art(f: &mut Frame, r: Rect, clip: Rect, face: u8, cell: f32, selectable: bool) {
    let family = if face < 9 {
        0
    } else if face < 18 {
        1
    } else if face < 27 {
        2
    } else if face < 31 {
        3
    } else if face < 34 {
        4
    } else if face < 38 {
        5
    } else {
        6
    };
    let rank = match family {
        0..=2 => face % 9 + 1,
        3 => face - 26,
        4 => face - 30,
        5 => face - 33,
        _ => face - 37,
    };
    let color = match family {
        0 => NEON_CYAN,
        1 => NEON_GREEN,
        2 => NEON_PINK,
        3 => NEON_GOLD,
        4 => match rank {
            1 => Color::srgb(1.0, 0.18, 0.24),
            2 => NEON_GREEN,
            _ => Color::srgb(0.85, 0.91, 0.94),
        },
        5 => NEON_PINK,
        6 => NEON_CYAN,
        _ => INK,
    };
    let alpha = if selectable { 1.0 } else { 0.52 };
    let color = color.with_alpha(alpha);
    let center = r.center() - Vec2::new(0.0, cell * 0.04);
    let unit = cell / 60.0;
    let sw = 1.6 * unit;
    match family {
        0 => {
            circle(f, center, cell * 0.23, sw * 1.3, color, clip);
        }
        1 => {
            for dx in [-0.15, 0.15] {
                let x = center.x + dx * cell;
                let stalk = rect(
                    x - cell * 0.055,
                    center.y - cell * 0.25,
                    cell * 0.11,
                    cell * 0.5,
                );
                clipped_rect(
                    f,
                    stalk.min,
                    stalk.size(),
                    Color::srgb(0.035, 0.10, 0.10),
                    clip,
                );
                rounded_outline_clipped(f, stalk.min, stalk.size(), 2.0 * unit, sw, color, clip);
                for dy in [-0.16, 0.0, 0.16] {
                    clipped_line(
                        f,
                        Vec2::new(x - 0.07 * cell, center.y + dy * cell),
                        Vec2::new(x + 0.07 * cell, center.y + dy * cell),
                        sw,
                        color,
                        clip,
                    );
                }
            }
            clipped_rect(
                f,
                center - Vec2::new(cell * 0.11, cell * 0.16),
                Vec2::new(cell * 0.22, cell * 0.32),
                Color::srgb(0.055, 0.071, 0.09),
                clip,
            );
        }
        2 => {
            // Hand-drawn 萬 structure: grass strokes, field frame, and lower sweeping legs.
            let p = [
                Vec2::new(-0.20, -0.22),
                Vec2::new(0.20, -0.22),
                Vec2::new(-0.12, -0.30),
                Vec2::new(-0.12, -0.15),
                Vec2::new(0.12, -0.30),
                Vec2::new(0.12, -0.15),
                Vec2::new(-0.18, -0.11),
                Vec2::new(0.18, -0.11),
                Vec2::new(-0.18, 0.05),
                Vec2::new(0.18, 0.05),
                Vec2::new(0.0, -0.11),
                Vec2::new(0.0, 0.18),
                Vec2::new(-0.24, 0.27),
                Vec2::new(0.24, 0.27),
            ];
            for (a, b) in [
                (0, 1),
                (2, 3),
                (4, 5),
                (6, 7),
                (6, 8),
                (7, 9),
                (8, 9),
                (10, 11),
                (11, 12),
                (11, 13),
            ] {
                clipped_line(
                    f,
                    center + p[a] * cell,
                    center + p[b] * cell,
                    sw,
                    color,
                    clip,
                );
            }
        }
        3 => {
            let half = cell * 0.25;
            let dash = cell * 0.08;
            let gap = cell * 0.055;
            let mut offset = 0.0;
            while offset < half * 2.0 {
                let end = (offset + dash).min(half * 2.0);
                let a = offset - half;
                let b = end - half;
                clipped_line(
                    f,
                    Vec2::new(center.x + a, center.y - half),
                    Vec2::new(center.x + b, center.y - half),
                    sw,
                    color,
                    clip,
                );
                clipped_line(
                    f,
                    Vec2::new(center.x + a, center.y + half),
                    Vec2::new(center.x + b, center.y + half),
                    sw,
                    color,
                    clip,
                );
                clipped_line(
                    f,
                    Vec2::new(center.x - half, center.y + a),
                    Vec2::new(center.x - half, center.y + b),
                    sw,
                    color,
                    clip,
                );
                clipped_line(
                    f,
                    Vec2::new(center.x + half, center.y + a),
                    Vec2::new(center.x + half, center.y + b),
                    sw,
                    color,
                    clip,
                );
                offset += dash + gap;
            }
        }
        4 => {
            let d = cell * 0.25;
            let points = [
                center + Vec2::new(0.0, -d),
                center + Vec2::new(d, 0.0),
                center + Vec2::new(0.0, d),
                center + Vec2::new(-d, 0.0),
                center + Vec2::new(0.0, -d),
            ];
            polyline(f, &points, sw * 1.5, color, clip);
        }
        5 => draw_flower(f, center, cell, rank, color, clip, sw),
        _ => draw_season(f, center, cell, rank, color, clip, sw),
    }
}

fn draw_flower(f: &mut Frame, c: Vec2, cell: f32, rank: u8, color: Color, clip: Rect, sw: f32) {
    match rank {
        1 => {
            for i in 0..5 {
                let a = i as f32 * std::f32::consts::TAU / 5.0;
                circle(
                    f,
                    c + Vec2::new(a.cos(), a.sin()) * cell * 0.12,
                    cell * 0.075,
                    sw,
                    color,
                    clip,
                );
            }
        }
        2 => {
            clipped_line(
                f,
                c + Vec2::new(-cell * 0.20, cell * 0.19),
                c + Vec2::new(cell * 0.14, -cell * 0.20),
                sw,
                color,
                clip,
            );
            for p in [
                Vec2::new(-0.06, 0.03),
                Vec2::new(0.07, -0.08),
                Vec2::new(0.16, -0.17),
            ] {
                circle(f, c + p * cell, cell * 0.045, sw, color, clip);
            }
        }
        3 => {
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let d = Vec2::new(a.cos(), a.sin());
                clipped_line(f, c, c + d * cell * 0.24, sw, color, clip);
                circle(f, c + d * cell * 0.24, cell * 0.035, sw, color, clip);
            }
        }
        _ => {
            for dx in [-0.12, 0.12] {
                clipped_line(
                    f,
                    c + Vec2::new(dx * cell, cell * 0.23),
                    c + Vec2::new(dx * cell, -cell * 0.22),
                    sw,
                    color,
                    clip,
                );
                for dy in [-0.12, 0.0, 0.12] {
                    clipped_line(
                        f,
                        c + Vec2::new(dx * cell - cell * 0.06, dy * cell),
                        c + Vec2::new(dx * cell + cell * 0.06, dy * cell),
                        sw,
                        color,
                        clip,
                    );
                }
            }
        }
    }
}

fn draw_season(f: &mut Frame, c: Vec2, cell: f32, rank: u8, color: Color, clip: Rect, sw: f32) {
    match rank {
        1 => {
            clipped_line(
                f,
                c + Vec2::new(0.0, cell * 0.23),
                c + Vec2::new(0.0, -cell * 0.05),
                sw,
                color,
                clip,
            );
            clipped_line(
                f,
                c + Vec2::new(0.0, cell * 0.06),
                c + Vec2::new(-cell * 0.17, -cell * 0.12),
                sw,
                color,
                clip,
            );
            clipped_line(
                f,
                c + Vec2::new(0.0, 0.0),
                c + Vec2::new(cell * 0.17, -cell * 0.18),
                sw,
                color,
                clip,
            );
            circle(
                f,
                c + Vec2::new(-cell * 0.17, -cell * 0.12),
                cell * 0.06,
                sw,
                color,
                clip,
            );
            circle(
                f,
                c + Vec2::new(cell * 0.17, -cell * 0.18),
                cell * 0.06,
                sw,
                color,
                clip,
            );
        }
        2 => {
            circle(f, c, cell * 0.13, sw, color, clip);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let d = Vec2::new(a.cos(), a.sin());
                clipped_line(f, c + d * cell * 0.18, c + d * cell * 0.25, sw, color, clip);
            }
        }
        3 => {
            let pts = [
                Vec2::new(0.0, -0.22),
                Vec2::new(0.18, -0.10),
                Vec2::new(0.13, 0.12),
                Vec2::new(0.0, 0.22),
                Vec2::new(-0.13, 0.12),
                Vec2::new(-0.18, -0.10),
                Vec2::new(0.0, -0.22),
            ];
            let pts: Vec<Vec2> = pts.iter().map(|p| c + *p * cell).collect();
            polyline(f, &pts, sw, color, clip);
            clipped_line(
                f,
                c + Vec2::new(0.0, -cell * 0.18),
                c + Vec2::new(0.0, cell * 0.18),
                sw,
                color,
                clip,
            );
        }
        _ => {
            for a in [
                0.0_f32,
                std::f32::consts::FRAC_PI_3,
                -std::f32::consts::FRAC_PI_3,
            ] {
                let d = Vec2::new(a.cos(), a.sin()) * cell * 0.24;
                clipped_line(f, c - d, c + d, sw, color, clip);
                for p in [-d, d] {
                    let n = Vec2::new(-p.y, p.x) * 0.32;
                    clipped_line(f, c + p, c + p + n, sw, color, clip);
                    clipped_line(f, c + p, c + p - n, sw, color, clip);
                }
            }
        }
    }
}

struct FaceLabel {
    text: String,
    color: Color,
    size: f32,
    position: Vec2,
    bounds: Rect,
}

fn face_label(r: Rect, face: u8, cell: f32) -> FaceLabel {
    let family = if face < 9 {
        0
    } else if face < 18 {
        1
    } else if face < 27 {
        2
    } else if face < 31 {
        3
    } else if face < 34 {
        4
    } else if face < 38 {
        5
    } else {
        6
    };
    let rank = match family {
        0..=2 => face % 9 + 1,
        3 => face - 26,
        4 => face - 30,
        5 => face - 33,
        _ => face - 37,
    };
    let color = match family {
        0 => NEON_CYAN,
        1 => NEON_GREEN,
        2 => NEON_PINK,
        3 => NEON_GOLD,
        4 => match rank {
            1 => Color::srgb(1.0, 0.18, 0.24),
            2 => NEON_GREEN,
            _ => Color::srgb(0.85, 0.91, 0.94),
        },
        5 => NEON_PINK,
        _ => NEON_CYAN,
    };
    let label = match family {
        0..=2 => format!("{rank}"),
        3 => ["E", "S", "W", "N"][(rank - 1) as usize].to_string(),
        4 => ["RED", "GRN", "WHT"][(rank - 1) as usize].to_string(),
        5 => ["PLM", "ORC", "CHR", "BAM"][(rank - 1) as usize].to_string(),
        _ => ["SPR", "SUM", "AUT", "WIN"][(rank - 1) as usize].to_string(),
    };
    let fs = match family {
        0 | 1 => (cell * 0.24).clamp(9.0, 16.0),
        3 => (cell * 0.34).clamp(12.0, 22.0),
        _ => (cell * 0.17).clamp(8.0, 13.0),
    };
    let y = match family {
        0 | 1 => r.center().y - fs * 0.62,
        3 => r.center().y - fs * 0.62,
        2 | 4 | 5 | 6 => r.max.y - fs - cell * 0.08,
        _ => r.min.y + cell * 0.08,
    };
    let position = Vec2::new(r.center().x - label.len() as f32 * fs * 0.28, y);
    let bounds = rect(
        position.x,
        position.y,
        label.chars().count() as f32 * fs * 0.56,
        fs * 1.2,
    );
    FaceLabel {
        text: label,
        color,
        size: fs,
        position,
        bounds,
    }
}

fn draw_face_labels(f: &mut Frame, label: FaceLabel, clip: Rect) {
    if label.bounds.min.x < clip.min.x
        || label.bounds.max.x > clip.max.x
        || label.bounds.min.y < clip.min.y
        || label.bounds.max.y > clip.max.y
    {
        return;
    }
    f.text(label.position, label.text, label.size, label.color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_wind_label_survives_unrelated_corner_occlusion() {
        let face = rect(0.0, 0.0, 60.0, 60.0);
        let label = face_label(face, 27, 60.0);
        let corner_tile = rect(0.0, 0.0, 15.0, 15.0);
        assert!(!label_overlaps_tile(label.bounds, corner_tile, 6.0));
    }

    #[test]
    fn label_is_suppressed_when_later_tile_covers_its_text_bounds() {
        let face = rect(0.0, 0.0, 60.0, 60.0);
        let label = face_label(face, 27, 60.0);
        let covering_tile = rect(24.0, 18.0, 20.0, 28.0);
        assert!(label_overlaps_tile(label.bounds, covering_tile, 6.0));
    }
}
