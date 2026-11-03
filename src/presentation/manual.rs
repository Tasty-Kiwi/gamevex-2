use super::*;

pub(crate) const TOPICS: &[(&str, &[&str])] = &[
    (
        "Minesweeper",
        &[
            "Clear every safe cell without revealing a mine. Numbers count mines in the eight neighboring cells. The first reveal and its neighbors are safe.",
            "Click or tap to reveal. Right-click or use Flag mode to mark a mine. Click an open number to reveal its unflagged neighbors when the number of flags matches it.",
            "Beginner: 9 by 9, 10 mines. Intermediate: 16 by 16, 40 mines. Expert: 30 by 16, 99 mines. Restart repeats the field seed. New field changes it.",
            "Use arrows to select a cell, Space to reveal, F to flag, and Enter to chord. Scroll or swipe large fields to pan.",
        ],
    ),
    (
        "Sudoku",
        &[
            "Fill each row, column, and box with every symbol once. Given clues cannot be edited. Choose a size and difficulty in Setup.",
            "Select a cell, then type a symbol or tap the keypad. Pencil mode records candidates. Erase removes the entry or notes, and Undo restores the previous edit.",
            "Hint helps with the next deduction. Using hints marks the run assisted and excludes it from best times. Highlights and error display can be adjusted in the cabinet.",
        ],
    ),
    (
        "Mahjong",
        &[
            "Clear the board by matching two free tiles. A free tile has no tile above it and at least one open side. Flower tiles match other flowers, and season tiles match other seasons.",
            "Hint highlights a free pair. Undo restores the last pair. Shuffle rearranges remaining faces when needed. Hint and Shuffle mark the run assisted.",
            "Timed runs track elapsed time. Zen turns off the timer and automatically shuffles a stuck board. Restart repeats the deal. New changes the layout or deal.",
        ],
    ),
    (
        "Solitaire",
        &[
            "Build four suit foundations from Ace to King. On the tableau, build downward in alternating colors. Move a correctly ordered face-up stack together. Empty columns take a King.",
            "Deal draws from stock to waste in Draw 1 or Draw 3 mode. Move the top waste card or an exposed tableau card. Tap a source and destination, or drag with mouse or touch.",
            "Hint shows a legal move. Undo restores the prior state. Finish appears when the remaining moves can safely complete the game.",
        ],
    ),
    (
        "FreeCell",
        &[
            "Build four suit foundations from Ace to King. Tableau columns build downward in alternating colors. Each of the four free cells holds one card.",
            "Empty tableau columns can take any card. Standard limits sequence moves by the available free cells and empty columns. Relaxed lets you move a valid sequence together without that limit.",
            "Tap a source and destination or drag cards. Hint shows a legal move. Undo restores the previous move. Finish appears when the deal can safely complete.",
        ],
    ),
    (
        "Spider",
        &[
            "Remove eight complete King-to-Ace sequences of the same suit. Build downward in rank. Only a same-suit sequence moves together.",
            "Deal adds one card to every tableau column. Fill all empty columns before dealing. An empty column can accept any legal card or sequence.",
            "Choose one, two, or four suits. Tap a source and destination or drag cards. Hint and Undo help recover from difficult positions.",
        ],
    ),
    (
        "Blackjack",
        &[
            "Beat the dealer by getting closer to 21 without going over. Face cards count 10. Aces count 1 or 11. A two-card 21 is a blackjack.",
            "Choose a wager and how many hands to play, then Deal. Hit draws a card. Stand keeps your total. Double adds a matching wager and draws one final card. Split separates a qualifying pair into two hands.",
            "Insurance is a side wager offered against a dealer Ace. Surrender, when offered, forfeits half the wager. A tie returns the wager. The dealer stands on soft 17 and plays after all player hands finish. A natural blackjack pays 3:2. Insurance pays 2:1 when the dealer has blackjack.",
            "Strategy cheat sheet: On hard 17 or higher, stand. On hard 13-16, stand against dealer 2-6 and hit otherwise. On hard 12, stand against 4-6 and hit otherwise.",
            "For hard 11, double against 2-10 when offered. For hard 10, double against 2-9. For hard 9, double against 3-6. Hit these totals when doubling is unavailable or outside those ranges. Always split Aces and eights when allowed. Keep pairs of tens together. Skip insurance as a basic strategy default.",
            "Soft totals: A soft hand counts an Ace as 11. Stand on soft 19 or higher. With soft 18, double against 3-6 if offered, stand against 2-8 otherwise, and hit against 9, 10, or Ace.",
            "Leaving an active round asks for confirmation. Committed wagers are forfeited if you leave. Unspent credits are saved.",
        ],
    ),
    (
        "Baccarat",
        &[
            "Choose Player, Banker, Tie, Player Pair, or Banker Pair before dealing. Choose one wager per round. The side closest to nine wins. Aces count one, face cards and tens count zero, and only the last digit of a hand's total counts.",
            "Draw cheat sheet: A natural 8 or 9 on either side stops play. Otherwise Player draws on 0-5 and stands on 6-7. If Player stands, Banker draws on 0-5.",
            "If Player draws, Banker draws on 0-2 always, on 3 except against an 8, on 4 against 2-7, on 5 against 4-7, and on 6 against 6-7. Banker stands on 7. These ranges refer to the value of Player’s third card.",
            "Player Pair and Banker Pair pay 11:1 if the named hand’s first two cards match in rank. The final total, third card, and winning hand do not affect a pair wager.",
            "Player pays even money. Banker pays even money less a 5% commission. A tie pays 8:1 on a Tie bet and returns Player or Banker wagers.",
        ],
    ),
    (
        "Texas Hold'em",
        &[
            "Make the best five-card hand from two private cards and five community cards. Each deal is one hand with a fresh buy-in against one to four CPUs.",
            "Blinds are $5 and $10. Check when nothing is owed, call to match the bet, raise, go all in, or fold. Raises must meet the last full raise size. Short all-ins do not reopen a closed raise.",
            "The board reveals the flop, turn, and river between betting rounds. The game settles side pots and split pots automatically. After folding, Skip to result advances the CPU play.",
            "Your bankroll persists. Leaving an active hand forfeits committed chips and keeps your unspent stack.",
        ],
    ),
    (
        "Roulette",
        &[
            "Choose one wager and a stake, then Spin. The single-zero European strip contains pockets 0 through 36. Green zero loses every outside bet.",
            "A straight number pays 35:1. Dozens and columns pay 2:1. Red or black, odd or even, and low (1-18) or high (19-36) pay 1:1.",
            "On a narrow screen, swipe the betting table sideways. Your bankroll saves after each spin. A zero bankroll offers a fresh session.",
        ],
    ),
    (
        "Hearts",
        &[
            "Aim for the lowest score. Each heart is one point and the queen of spades is 13. Take all 26 points to give each opponent 26 instead.",
            "Pass three cards as directed. Passes rotate left, right, across, and hold. The two of clubs starts the first trick. Follow the led suit whenever possible. Avoid point cards on the first trick if you have an alternative.",
            "The highest card in the led suit wins the trick and leads next. Lead hearts only after they break, unless you hold only hearts. The match ends when anyone reaches 100.",
        ],
    ),
    (
        "Cribbage",
        &[
            "Discard two cards to the dealer's crib. The non-dealer pegs first. Play without exceeding 31. Fifteen and 31 score two. Pairs score 2, 6, or 12. Runs score their length. Go or last card scores one.",
            "After pegging, score hands and the crib for fifteens, pairs, runs, and flushes. A hand flush can score four, or five with the cut. A crib flush requires all five cards.",
            "Nobs scores one for a Jack matching the cut suit. A Jack cut gives the dealer two. First to 121 wins.",
        ],
    ),
    (
        "Backgammon",
        &[
            "Be first to bear off all 15 checkers. Each die is a separate move. Doubles give four moves. Use as many dice as possible. If only one of two dice can be used, use the higher die when legal.",
            "A point with two or more opposing checkers is blocked. Landing on a single opposing checker hits it to the bar. Checkers on the bar must re-enter before moving others.",
            "Bear off once all your remaining checkers are in your home board. Use an exact die, or a higher one when no checker is farther from the exit.",
            "Roll, select a highlighted source, then a destination or the off tray. The Easy or Hard control changes CPU difficulty. Active games and wins are saved.",
        ],
    ),
    (
        "Card Lab",
        &[
            "Card Lab is a sandbox for the shared card controls. Select or drag a card into an empty slot. Occupied slots reject another card.",
            "Right-click a card to flip it, or use the Flip button. Deal resets the sample hand. Return card puts the selected card back in the hand.",
        ],
    ),
];

pub(super) fn manual(f: &mut Frame, s: &mut Arcade, l: &mut ViewLayout, m: f32, w: f32) {
    let narrow = w < 620.0;
    let title_size = if narrow { 26.0 } else { 40.0 };
    f.text(Vec2::new(m, 101.0), "Game manual", title_size, INK);
    if let Some(topic) = s.manual_topic {
        let (title, paragraphs) = TOPICS[topic];
        button(
            f,
            s,
            l,
            rect(m + w - 86.0, 151.0, 86.0, 34.0),
            "Contents",
            Action::ManualTopic(None),
            false,
        );
        f.text(
            Vec2::new(m, 155.0),
            title,
            if narrow { 16.0 } else { 23.0 },
            CYAN,
        );
        let clip = rect(m, 202.0, w.min(780.0), (f.size.y - 246.0).max(1.0));
        let size = if narrow { 14.0 } else { 17.0 };
        let line_height = size + 8.0;
        let columns = ((clip.width() - 32.0) / (size * 0.62)).floor().max(1.0) as usize;
        let mut lines = Vec::new();
        let mut height = 12.0;
        for paragraph in paragraphs {
            let mut line = String::new();
            for word in paragraph.split_whitespace() {
                if !line.is_empty() && line.len() + 1 + word.len() > columns {
                    lines.push((height, std::mem::take(&mut line)));
                    height += line_height;
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
            lines.push((height, line));
            height += line_height + 14.0;
        }
        l.lobby_clip = Some(clip);
        l.lobby_max_scroll = (height - clip.height()).max(0.0);
        s.lobby_scroll = s.lobby_scroll.clamp(0.0, l.lobby_max_scroll);
        f.rounded_rect(clip, 8.0, PANEL);
        for (y, line) in lines {
            let y = clip.min.y + y - s.lobby_scroll;
            if y >= clip.min.y && y + line_height <= clip.max.y {
                f.text(Vec2::new(clip.min.x + 14.0, y), line, size, INK);
            }
        }
        if l.lobby_max_scroll > 0.0 {
            let length = clip.height() * clip.height() / height;
            let top = clip.min.y + (clip.height() - length) * s.lobby_scroll / l.lobby_max_scroll;
            f.rect(
                Vec2::new(clip.max.x - 4.0, top),
                Vec2::new(2.0, length),
                CYAN,
            );
        }
    } else {
        f.text(Vec2::new(m, 151.0), "Choose a game", 14.0, MUTED);
        let columns = if narrow { 2 } else { 3 };
        let rows = TOPICS.len().div_ceil(columns);
        let gap = 8.0;
        let width = (w - (columns - 1) as f32 * gap) / columns as f32;
        let height = ((f.size.y - 218.0 - (rows - 1) as f32 * gap) / rows as f32).clamp(33.0, 56.0);
        for (i, (title, _)) in TOPICS.iter().enumerate() {
            let r = rect(
                m + (i % columns) as f32 * (width + gap),
                178.0 + (i / columns) as f32 * (height + gap),
                width,
                height,
            );
            button(f, s, l, r, title, Action::ManualTopic(Some(i)), false);
        }
    }
}
