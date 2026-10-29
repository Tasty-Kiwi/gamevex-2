use super::*;

pub(crate) fn lobby(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let wide = w > 800.0;
    let title_size = if f.size.y < 640.0 {
        (w / 13.0).clamp(28.0, 50.0)
    } else if f.size.y < 720.0 {
        (w / 10.0).clamp(34.0, 70.0)
    } else {
        (w / 12.0).clamp(48.0, 96.0)
    };
    f.text(Vec2::new(m - 3.0, 106.0), "GAMEVEX", title_size, INK);
    let y = 124.0 + title_size;
    f.line(Vec2::new(m, y), Vec2::new(m + w, y), 2.0, MAGENTA);
    f.text(
        Vec2::new(m, y + 17.0),
        "One more round.",
        if f.size.y < 640.0 {
            24.0
        } else if wide {
            32.0
        } else {
            25.0
        },
        CYAN,
    );
    if wide {
        f.text(
            Vec2::new(m + w - 330.0, y + 23.0),
            "No account. No network. Just play.",
            15.0,
            MUTED,
        );
    }
    let compact_layout = f.size.y < 640.0;
    let row_gap = if compact_layout { 8.0 } else { 16.0 };
    let roster_y = y + if compact_layout { 50.0 } else { 76.0 };
    let reserved = if compact_layout { 52.0 } else { 88.0 };
    let row_h = ((f.size.y - roster_y - reserved - row_gap * 3.0) / 4.0).clamp(64.0, 96.0);
    let a = rect(m, roster_y, w, row_h);
    cabinet(
        f,
        s,
        l,
        a,
        "MINESWEEPER",
        "Read the field. Trust the numbers.",
        "PLAY",
        CYAN,
        Action::Open(Screen::Mines),
    );
    let b = rect(m, roster_y + row_h + row_gap, w, row_h);
    cabinet(
        f,
        s,
        l,
        b,
        "CARD LAB",
        "Deal, flip, drag. An interaction sandbox.",
        "OPEN",
        MAGENTA,
        Action::Open(Screen::Cards),
    );
    let c = rect(m, roster_y + (row_h + row_gap) * 2.0, w, row_h);
    cabinet(
        f,
        s,
        l,
        c,
        "BLACKJACK",
        "Five hands. One shoe. Your call.",
        "DEAL",
        GOLD,
        Action::Open(Screen::Blackjack),
    );
    f.text(
        Vec2::new(c.min.x + 24.0, c.min.y + row_h - 11.0),
        format!("{} credits", s.save.blackjack.balance),
        10.0,
        GOLD,
    );
    let d = rect(m, roster_y + (row_h + row_gap) * 3.0, w, row_h);
    cabinet(
        f,
        s,
        l,
        d,
        "BACCARAT",
        "Player. Banker. Closest to nine.",
        "DEAL",
        GREEN,
        Action::Open(Screen::Baccarat),
    );
    f.text(
        Vec2::new(d.min.x + 24.0, d.min.y + row_h - 11.0),
        format!("{} credits", s.save.baccarat.balance),
        10.0,
        GREEN,
    );
    let records_y = roster_y + row_h * 4.0 + row_gap * 3.0 + 23.0;
    if records_y + 75.0 < f.size.y - 40.0 {
        f.text(
            Vec2::new(m, records_y),
            "Personal bests / Minesweeper",
            15.0,
            INK,
        );
        for (i, d) in Difficulty::ALL.into_iter().enumerate() {
            let x = m + i as f32 * w / 3.0;
            let name = difficulty_name(d);
            let best = s
                .save
                .minesweeper
                .records
                .best(d)
                .map(format_time)
                .unwrap_or("--".into());
            f.text(
                Vec2::new(x, records_y + 30.0),
                format!("{name}  {best}"),
                if wide { 17.0 } else { 12.0 },
                GOLD,
            );
        }
        if records_y + 115.0 < f.size.y - 40.0 {
            f.text(
                Vec2::new(m, records_y + 82.0),
                compact(&s.backend, ((w - 150.0) / 7.5).max(8.0) as usize),
                12.0,
                MUTED,
            );
            f.text(
                Vec2::new(m + w - 135.0, records_y + 82.0),
                format!("{:.0} fps / local", s.fps),
                12.0,
                MUTED,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cabinet(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    r: Rect,
    title: &str,
    desc: &str,
    verb: &str,
    color: Color,
    action: Action,
) {
    let index = l.hits.len();
    let hot = s.cursor.is_some_and(|p| r.contains(p)) || (s.keyboard && s.focus == index);
    f.rect(
        r.min,
        r.size(),
        if hot {
            Color::srgb(0.04, 0.1, 0.14)
        } else {
            PANEL
        },
    );
    f.outline(
        r.min,
        r.size(),
        if hot { 2.0 } else { 1.0 },
        if hot { color } else { EDGE },
    );
    f.rect(r.min, Vec2::new(5.0, r.height()), color);
    let small = r.width() < 600.0;
    let short_row = r.height() < 90.0;
    f.text(
        r.min + Vec2::new(23.0, 17.0),
        title,
        if short_row {
            20.0
        } else if small {
            22.0
        } else {
            30.0
        },
        INK,
    );
    if !small && !short_row {
        f.text(r.min + Vec2::new(24.0, 61.0), desc, 15.0, MUTED);
    } else {
        let short_desc = match title {
            "MINESWEEPER" => "A safe first move. A new field.",
            "BLACKJACK" => "Five hands. One shoe. Play local.",
            "BACCARAT" => "Player, Banker or Tie. Closest to 9.",
            _ => "Card interaction sandbox.",
        };
        let desc_y = if short_row {
            (r.height() - 23.0).max(40.0)
        } else {
            53.0
        };
        f.text(
            r.min + Vec2::new(24.0, desc_y),
            short_desc,
            if short_row { 10.0 } else { 12.0 },
            MUTED,
        );
    }
    f.text(
        Vec2::new(r.max.x - if small { 64.0 } else { 112.0 }, r.min.y + 26.0),
        verb,
        if small { 15.0 } else { 20.0 },
        color,
    );
    l.hits.push(Hit { rect: r, action });
}
