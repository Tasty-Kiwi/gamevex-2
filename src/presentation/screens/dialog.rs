use super::*;

pub(crate) fn dialog(f: &mut Frame, s: &Arcade, l: &mut ViewLayout, m: f32) {
    if matches!(s.modal, Some(Pending::SudokuSetup)) {
        super::super::sudoku::setup_dialog(f, s, l, m);
        return;
    }
    let width = (f.size.x - 2.0 * m).min(590.0);
    let r = rect(
        (f.size.x - width) * 0.5,
        (f.size.y - 254.0) * 0.5,
        width,
        254.0,
    );
    f.outline(
        r.min - Vec2::splat(6.0),
        r.size() + Vec2::splat(12.0),
        1.0,
        EDGE,
    );
    f.rect(r.min, r.size(), PANEL);
    f.rounded_outline(r.min, r.size(), 8.0, 2.0, GOLD);
    let recover = matches!(s.modal, Some(Pending::Recover));
    let sudoku = matches!(s.modal, Some(Pending::NewSudoku(..)));
    let leave_holdem = matches!(s.modal, Some(Pending::LeaveHoldem(_)));
    let leave_blackjack = matches!(s.modal, Some(Pending::LeaveBlackjack(_)));
    f.text(
        r.min + Vec2::new(22.0, 24.0),
        if leave_blackjack || leave_holdem {
            "Leave this round?"
        } else if recover {
            "Reset saved data?"
        } else {
            "Replace this run?"
        },
        if width < 400.0 { 23.0 } else { 29.0 },
        INK,
    );
    f.text(
        r.min + Vec2::new(22.0, 83.0),
        if leave_holdem {
            "Chips already in the pot will be forfeited.\nYour unspent stack stays in your bankroll."
        } else if leave_blackjack {
            "Your current Blackjack bets will be forfeited.\nUnspent credits will be kept."
        } else if recover {
            "Saved settings, records, and progress\nwill be replaced with a fresh session."
        } else if matches!(s.modal, Some(Pending::NewMahjong(..))) {
            "Your Mahjong progress will be replaced.\nCancel to keep the current board."
        } else if matches!(s.modal, Some(Pending::NewPatience(..))) {
            "Your card-game progress will be replaced.\nCancel to keep this deal."
        } else if matches!(s.modal, Some(Pending::NewCribbage)) {
            "Your Cribbage match will be replaced.\nCancel to keep the current round."
        } else if matches!(s.modal, Some(Pending::BackgammonReplace(_))) {
            "Your Backgammon game will be replaced.\nCancel to keep the current board."
        } else if matches!(s.modal, Some(Pending::NewHearts)) {
            "Your Hearts match will be replaced.\nCancel to keep the current hand."
        } else if sudoku {
            "Your Sudoku entries and notes will be replaced.\nCancel to keep playing this puzzle."
        } else {
            "Your unfinished field will be replaced.\nCancel to keep playing this run."
        },
        if width < 400.0 { 12.0 } else { 16.0 },
        MUTED,
    );
    let bw = (width - 58.0) * 0.5;
    button(
        f,
        s,
        l,
        rect(r.min.x + 22.0, r.max.y - 65.0, bw, 40.0),
        "Cancel",
        Action::Cancel,
        false,
    );
    button(
        f,
        s,
        l,
        rect(r.min.x + 36.0 + bw, r.max.y - 65.0, bw, 40.0),
        if leave_blackjack || leave_holdem {
            "Leave round"
        } else if recover {
            "Reset data"
        } else {
            "Replace run"
        },
        Action::Confirm,
        true,
    );
}
