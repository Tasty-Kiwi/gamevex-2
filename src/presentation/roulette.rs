use super::screens::button;
use super::*;
use gamevex_core::roulette::{Bet, PocketColor, WHEEL_ORDER};

fn pocket_color(n: u8) -> Color {
    match gamevex_core::roulette::color(n) {
        PocketColor::Red => RED,
        PocketColor::Black => Color::srgb(0.12, 0.38, 0.46),
        PocketColor::Green => GREEN,
    }
}
fn centered(f: &mut Frame, r: Rect, text: &str, size: f32, color: Color) {
    f.text(
        Vec2::new(
            r.center().x - text.len() as f32 * size * 0.30,
            r.center().y - size * 0.60,
        ),
        text,
        size,
        color,
    );
}
fn intersection(a: Rect, b: Rect) -> Rect {
    let min = a.min.max(b.min).min(b.max);
    Rect::from_corners(min, a.max.min(b.max).max(min))
}
fn strip(f: &mut Frame, s: &Arcade, r: Rect) {
    let position = s.roulette_ui.strip_position();
    let pitch = if r.width() < 620.0 { 54.0 } else { 72.0 };
    let center = r.center().x;
    let first = position.floor() as i32 - (r.width() / pitch * 0.5).ceil() as i32 - 1;
    let count = (r.width() / pitch).ceil() as i32 + 4;
    f.rect(r.min, r.size(), BG);
    for index in first..first + count {
        let n = WHEEL_ORDER[index.rem_euclid(37) as usize];
        let full = rect(
            center + (index as f32 - position) * pitch - pitch * 0.5,
            r.min.y,
            pitch - 3.0,
            r.height(),
        );
        let cell = intersection(full, r);
        if cell.width() <= 0.0 {
            continue;
        }
        let color = pocket_color(n);
        let c = color.to_srgba();
        let palette = f.palette;
        if n == 0 {
            f.palette = gamevex_core::ColorScheme::Cyan;
        }
        f.rect(
            cell.min,
            cell.size(),
            if n == 0 {
                Color::srgb(0.0, 0.23, 0.13)
            } else if c.red > 0.5 {
                Color::srgb(0.18, 0.018, 0.05)
            } else {
                PANEL
            },
        );
        f.outline(cell.min, cell.size(), 1.5, color);
        f.palette = palette;
        if full.min.x >= r.min.x && full.max.x <= r.max.x {
            centered(
                f,
                full,
                &n.to_string(),
                if r.height() < 55.0 { 23.0 } else { 30.0 },
                INK,
            );
        }
    }
    let marker = rect(
        center - pitch * 0.5 - 2.0,
        r.min.y - 2.0,
        pitch + 1.0,
        r.height() + 4.0,
    );
    f.outline(marker.min, marker.size(), 2.5, GOLD);
    f.polygon(
        &[
            Vec2::new(center - 7.0, r.min.y - 9.0),
            Vec2::new(center + 7.0, r.min.y - 9.0),
            Vec2::new(center, r.min.y - 2.0),
        ],
        GOLD,
    );
    if let Some(event) = s.roulette_ui.feedback.events.iter().find(|e| {
        matches!(
            e.kind,
            crate::table_feedback::EffectKind::Win | crate::table_feedback::EffectKind::Stuck
        )
    }) {
        let c = if event.kind == crate::table_feedback::EffectKind::Win {
            GOLD
        } else {
            RED
        }
        .to_srgba();
        f.outline(
            r.min - Vec2::splat(4.0),
            r.size() + Vec2::splat(8.0),
            2.0,
            Color::srgba(c.red, c.green, c.blue, 1.0 - event.age / event.duration()),
        );
    }
}

/// Reference layout: zero, three number rows and column bets; dozens, then even-money bets.
fn bet_rects(table: Rect) -> Vec<(Rect, Bet, String)> {
    let gap = 3.0;
    let zero_w = 36.0;
    let column_w = 44.0;
    let x = table.min.x + zero_w + gap;
    let width = table.width() - zero_w - column_w - 2.0 * gap;
    let cell_w = (width - 11.0 * gap) / 12.0;
    let outside_h = 32.0;
    let row_h = (table.height() - 2.0 * outside_h - 4.0 * gap) / 3.0;
    let mut bets = vec![(
        rect(table.min.x, table.min.y, zero_w, 3.0 * row_h + 2.0 * gap),
        Bet::Number(0),
        "0".into(),
    )];
    for n in 1..=36u8 {
        let row = 2 - ((n - 1) % 3) as usize;
        let col = ((n - 1) / 3) as usize;
        bets.push((
            rect(
                x + col as f32 * (cell_w + gap),
                table.min.y + row as f32 * (row_h + gap),
                cell_w,
                row_h,
            ),
            Bet::Number(n),
            n.to_string(),
        ));
    }
    for (row, bet) in [Bet::Column3, Bet::Column2, Bet::Column1]
        .into_iter()
        .enumerate()
    {
        bets.push((
            rect(
                x + width + gap,
                table.min.y + row as f32 * (row_h + gap),
                column_w,
                row_h,
            ),
            bet,
            "2:1".into(),
        ));
    }
    let y = table.min.y + 3.0 * (row_h + gap);
    for (i, bet) in [Bet::Dozen1, Bet::Dozen2, Bet::Dozen3]
        .into_iter()
        .enumerate()
    {
        bets.push((
            rect(
                x + i as f32 * (width + gap) / 3.0,
                y,
                (width - 2.0 * gap) / 3.0,
                outside_h,
            ),
            bet,
            format!("{} 12", ["1st", "2nd", "3rd"][i]),
        ));
    }
    for (i, bet) in [
        Bet::Low,
        Bet::Even,
        Bet::Red,
        Bet::Black,
        Bet::Odd,
        Bet::High,
    ]
    .into_iter()
    .enumerate()
    {
        bets.push((
            rect(
                x + i as f32 * (width + gap) / 6.0,
                y + outside_h + gap,
                (width - 5.0 * gap) / 6.0,
                outside_h,
            ),
            bet,
            ["1-18", "EVEN", "RED", "BLACK", "ODD", "19-36"][i].into(),
        ));
    }
    bets
}
fn paint_bet(
    f: &mut Frame,
    s: &Arcade,
    l: &mut ViewLayout,
    entry: &(Rect, Bet, String),
    clip: Rect,
) {
    let (full, bet, label) = entry;
    let r = intersection(*full, clip);
    let selected = s.roulette_ui.bet == *bet;
    let enabled = !s.roulette_ui.busy() && !s.roulette_ui.rules;
    if enabled {
        l.hits.push(Hit {
            rect: r,
            action: Action::RouletteBet(*bet),
        });
    }
    if r.width() <= 0.0 || r.height() <= 0.0 {
        return;
    }
    let focused = s.keyboard && s.focus == l.hits.len().saturating_sub(1) && enabled;
    let hovered = s.cursor.is_some_and(|p| r.contains(p));
    let color = match bet {
        Bet::Number(n) => pocket_color(*n),
        Bet::Red => RED,
        Bet::Black => MAGENTA,
        _ => CYAN,
    };
    let fill = if *bet == Bet::Number(0) {
        Color::srgb(0.0, 0.23, 0.13)
    } else if selected || hovered || focused {
        Color::srgb(0.045, 0.12, 0.17)
    } else {
        PANEL
    };
    let palette = f.palette;
    if *bet == Bet::Number(0) {
        f.palette = gamevex_core::ColorScheme::Cyan;
    }
    f.rect(r.min, r.size(), fill);
    f.outline(
        r.min,
        r.size(),
        if selected || focused { 2.5 } else { 1.0 },
        if focused { GOLD } else { color },
    );
    f.palette = palette;
    if selected {
        f.rect(r.min, Vec2::new(3.0, r.height()), GOLD);
    }
    if full.min.x >= clip.min.x && full.max.x <= clip.max.x {
        let text_color = if enabled { INK } else { MUTED };
        if matches!(bet, Bet::Red | Bet::Black) {
            let center = full.center();
            let half = Vec2::new(11.0, 9.0);
            let points = [
                center - Vec2::new(half.x, 0.0),
                center - Vec2::new(0.0, half.y),
                center + Vec2::new(half.x, 0.0),
                center + Vec2::new(0.0, half.y),
            ];
            f.polygon(&points, if *bet == Bet::Red { RED } else { BG });
            for i in 0..4 {
                f.line(points[i], points[(i + 1) % 4], 1.0, text_color);
            }
        } else {
            centered(
                f,
                *full,
                label,
                if matches!(bet, Bet::Number(_)) {
                    18.0
                } else {
                    12.0
                },
                text_color,
            );
        }
    }
}

pub(super) fn roulette(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let small = w < 620.0;
    let short = f.size.y < 640.0;
    let title_y = if short { 72.0 } else { 84.0 };
    let busy = s.roulette_ui.busy();
    if !busy {
        let (min, max) = gamevex_core::roulette::stake_bounds(s.save.roulette_balance);
        s.roulette_ui.stake = s.roulette_ui.stake.clamp(min, max);
    }
    let balance = s.roulette_ui.display_balance(s.save.roulette_balance);
    f.text(
        Vec2::new(m, title_y),
        "ROULETTE",
        if small { 28.0 } else { 38.0 },
        INK,
    );
    f.text(
        Vec2::new(m, title_y + 36.0),
        format!("CREDITS ${balance}"),
        if small { 12.0 } else { 14.0 },
        GOLD,
    );
    f.text(
        Vec2::new(m + w * 0.53, title_y + 36.0),
        format!("STAKE ${}", s.roulette_ui.stake),
        if small { 12.0 } else { 14.0 },
        CYAN,
    );
    button(
        f,
        s,
        l,
        rect(m + w - 82.0, title_y, 82.0, 30.0),
        "Rules",
        Action::RouletteRules,
        s.roulette_ui.rules,
    );
    let strip_r = rect(m, title_y + 64.0, w, if short { 46.0 } else { 66.0 });
    strip(f, s, strip_r);
    let info_y = strip_r.max.y + 12.0;
    f.text(
        Vec2::new(m, info_y),
        format!(
            "{} / {}:1",
            s.roulette_ui.bet.label(),
            s.roulette_ui.bet.multiplier()
        ),
        if small { 10.0 } else { 13.0 },
        CYAN,
    );
    if small {
        f.text(Vec2::new(m + w - 92.0, info_y), "Swipe table", 10.0, MUTED);
    } else {
        for (i, n) in s.roulette_ui.history.iter().enumerate() {
            f.text(
                Vec2::new(m + w - 260.0 + i as f32 * 25.0, info_y),
                n.to_string(),
                12.0,
                if i == 0 { GOLD } else { pocket_color(*n) },
            );
        }
    }
    let controls_y = f.size.y - if short { 86.0 } else { 100.0 };
    let controls_h = if short { 34.0 } else { 40.0 };
    let clip = rect(m, info_y + 24.0, w, controls_y - info_y - 42.0);
    let content_w = w.max(560.0);
    l.roulette_clip = Some(clip);
    l.roulette_max_pan = (content_w - w).max(0.0);
    s.roulette_ui.table_pan = s.roulette_ui.table_pan.clamp(0.0, l.roulette_max_pan);
    let unshifted = bet_rects(rect(clip.min.x, clip.min.y, content_w, clip.height()));
    if s.keyboard
        && !busy
        && !s.roulette_ui.rules
        && let Some(index) = s.focus.checked_sub(l.hits.len())
        && let Some((r, _, _)) = unshifted.get(index)
    {
        if r.min.x - s.roulette_ui.table_pan < clip.min.x {
            s.roulette_ui.table_pan = r.min.x - clip.min.x;
        } else if r.max.x - s.roulette_ui.table_pan > clip.max.x {
            s.roulette_ui.table_pan = r.max.x - clip.max.x;
        }
        s.roulette_ui.table_pan = s.roulette_ui.table_pan.clamp(0.0, l.roulette_max_pan);
    }
    for (r, bet, label) in unshifted {
        paint_bet(
            f,
            s,
            l,
            &(
                Rect::from_corners(
                    r.min - Vec2::new(s.roulette_ui.table_pan, 0.0),
                    r.max - Vec2::new(s.roulette_ui.table_pan, 0.0),
                ),
                bet,
                label,
            ),
            clip,
        );
    }
    if l.roulette_max_pan > 0.0 {
        let y = clip.max.y + 6.0;
        f.rect(Vec2::new(m, y), Vec2::new(w, 2.0), EDGE);
        let thumb = w * w / content_w;
        f.rect(
            Vec2::new(
                m + (w - thumb) * s.roulette_ui.table_pan / l.roulette_max_pan,
                y,
            ),
            Vec2::new(thumb, 2.0),
            CYAN,
        );
    }
    let widths = [54.0, 42.0, 42.0, 52.0, w - 210.0];
    let actions = [
        Action::RouletteStakeMin,
        Action::RouletteStake(-5),
        Action::RouletteStake(5),
        Action::RouletteStakeMax,
        if balance == 0 {
            Action::RouletteRestart
        } else {
            Action::RouletteSpin
        },
    ];
    let labels = [
        "Min",
        "-5",
        "+5",
        "Max",
        if busy {
            "Spinning"
        } else if balance == 0 {
            "Restart"
        } else {
            "Spin"
        },
    ];
    let mut x = m;
    for i in 0..5 {
        let r = rect(x, controls_y, widths[i], controls_h);
        if !busy && !s.roulette_ui.rules {
            button(f, s, l, r, labels[i], actions[i], i == 4);
        } else {
            f.rect(r.min, r.size(), PANEL);
            f.outline(r.min, r.size(), 1.0, EDGE);
            centered(f, r, labels[i], 12.0, MUTED);
        }
        x += widths[i] + 5.0;
    }
    if let Some(result) = s.roulette_ui.result.filter(|_| !busy) {
        f.text(
            Vec2::new(m, controls_y + controls_h + 6.0),
            format!(
                "{} / {} / {}${}",
                if result.won { "WIN" } else { "LOST" },
                result.number,
                if result.net >= 0 { "+" } else { "-" },
                result.net.unsigned_abs()
            ),
            if short { 10.0 } else { 12.0 },
            if result.won { GOLD } else { RED },
        );
    }
    if s.roulette_ui.rules {
        l.hits.clear();
        l.roulette_clip = None;
        l.roulette_max_pan = 0.0;
        let r = rect(
            m + w * 0.06,
            title_y + 20.0,
            w * 0.88,
            (f.size.y - title_y - 44.0).max(180.0),
        );
        f.rect(r.min, r.size(), BG);
        f.outline(r.min, r.size(), 2.0, MAGENTA);
        f.text(r.min + Vec2::new(14.0, 16.0), "TABLE RULES", 20.0, GOLD);
        let lines = [
            "European single-zero roulette",
            "Number 35:1; dozen 2:1.",
            "Red, black, odd, even: 1:1.",
            "Low/high 1:1; columns 2:1.",
            "Choose stake and one bet, then spin.",
            "A zero loses every outside bet.",
        ];
        for (i, line) in lines.iter().enumerate() {
            f.text(
                r.min + Vec2::new(14.0, 54.0 + i as f32 * 25.0),
                *line,
                if small { 12.0 } else { 15.0 },
                INK,
            );
        }
        button(
            f,
            s,
            l,
            rect(r.max.x - 94.0, r.max.y - 44.0, 78.0, 30.0),
            "CLOSE",
            Action::RouletteRules,
            true,
        );
    }
}
