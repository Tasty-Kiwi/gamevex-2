use super::cards::{Card, Suit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Variant {
    Klondike1,
    Klondike3,
    FreeCell,
    Spider1,
    Spider2,
    Spider4,
}
impl Variant {
    pub const ALL: [Self; 6] = [
        Self::Klondike1,
        Self::Klondike3,
        Self::FreeCell,
        Self::Spider1,
        Self::Spider2,
        Self::Spider4,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Klondike1 => "Klondike Draw 1",
            Self::Klondike3 => "Klondike Draw 3",
            Self::FreeCell => "FreeCell",
            Self::Spider1 => "Spider 1 Suit",
            Self::Spider2 => "Spider 2 Suits",
            Self::Spider4 => "Spider 4 Suits",
        }
    }
    pub const fn columns(self) -> usize {
        match self {
            Self::Klondike1 | Self::Klondike3 => 7,
            Self::FreeCell => 8,
            _ => 10,
        }
    }
    fn decks(self) -> usize {
        match self {
            Self::Spider1 | Self::Spider2 | Self::Spider4 => 2,
            _ => 1,
        }
    }
    fn is_spider(self) -> bool {
        matches!(self, Self::Spider1 | Self::Spider2 | Self::Spider4)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pile {
    Stock,
    Waste,
    Foundation(usize),
    Cell(usize),
    Tableau(usize),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Snapshot {
    stock: Vec<Card>,
    waste: Vec<Card>,
    foundations: Vec<Vec<Card>>,
    cells: Vec<Option<Card>>,
    tableau: Vec<Vec<Card>>,
    completed: Vec<Vec<Card>>,
    moves: u32,
    assisted: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub variant: Variant,
    pub seed: u64,
    pub stock: Vec<Card>,
    pub waste: Vec<Card>,
    pub foundations: Vec<Vec<Card>>,
    pub cells: Vec<Option<Card>>,
    pub tableau: Vec<Vec<Card>>,
    pub completed: Vec<Vec<Card>>,
    pub elapsed_ms: u64,
    pub moves: u32,
    pub assisted: bool,
    history: Vec<Snapshot>,
}
fn snapshot(g: &Game) -> Snapshot {
    Snapshot {
        stock: g.stock.clone(),
        waste: g.waste.clone(),
        foundations: g.foundations.clone(),
        cells: g.cells.clone(),
        tableau: g.tableau.clone(),
        completed: g.completed.clone(),
        moves: g.moves,
        assisted: g.assisted,
    }
}
fn color(c: Card) -> bool {
    c.suit.is_red()
}
fn ace_rank(c: Card) -> u8 {
    if c.rank == 14 { 1 } else { c.rank }
}
fn suits() -> [Suit; 4] {
    [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds]
}
impl Game {
    pub fn new(variant: Variant, seed: u64) -> Self {
        let mut deck = Vec::new();
        let suit_order: Vec<Suit> = match variant {
            Variant::Spider1 => vec![Suit::Spades; 8],
            Variant::Spider2 => vec![Suit::Spades; 4]
                .into_iter()
                .chain(vec![Suit::Hearts; 4])
                .collect(),
            Variant::Spider4 => vec![
                Suit::Spades,
                Suit::Spades,
                Suit::Hearts,
                Suit::Hearts,
                Suit::Diamonds,
                Suit::Diamonds,
                Suit::Clubs,
                Suit::Clubs,
            ],
            _ => suits().to_vec(),
        };
        for s in suit_order {
            for rank in 2..=14 {
                deck.push(Card::new(rank, s, false));
            }
        }
        let mut state = seed.max(1);
        for i in (1..deck.len()).rev() {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            let j = (state.wrapping_mul(0x2545_F491_4F6C_DD1D) % ((i + 1) as u64)) as usize;
            deck.swap(i, j);
        }
        let (mut stock, mut tableau, mut cells) = (
            Vec::new(),
            vec![Vec::new(); variant.columns()],
            vec![None; 4],
        );
        if variant.is_spider() {
            for (i, mut c) in deck.into_iter().enumerate() {
                if i < 54 {
                    c.face_up = false;
                    tableau[i % 10].push(c)
                } else {
                    c.face_up = false;
                    stock.push(c)
                }
            }
            for col in &mut tableau {
                if let Some(top) = col.last_mut() {
                    top.face_up = true;
                }
            }
        } else if variant == Variant::FreeCell {
            for (i, mut c) in deck.into_iter().enumerate() {
                c.face_up = true;
                tableau[i % 8].push(c);
            }
        } else {
            for col in 0..7 {
                for row in 0..=col {
                    let mut c = deck[col * (col + 1) / 2 + row];
                    c.face_up = row == col;
                    tableau[col].push(c);
                }
            }
            stock = deck[28..].iter().rev().copied().collect();
        }
        if variant != Variant::FreeCell {
            cells.clear();
        }
        let foundations = vec![Vec::new(); 4];
        Self {
            variant,
            seed,
            stock,
            waste: Vec::new(),
            foundations,
            cells,
            tableau,
            completed: Vec::new(),
            elapsed_ms: 0,
            moves: 0,
            assisted: false,
            history: Vec::new(),
        }
    }
    pub fn cards(&self, p: Pile) -> Vec<Card> {
        match p {
            Pile::Stock => self.stock.clone(),
            Pile::Waste => self.waste.clone(),
            Pile::Foundation(i) => self.foundations.get(i).cloned().unwrap_or_default(),
            Pile::Cell(i) => self.cells.get(i).and_then(|x| *x).into_iter().collect(),
            Pile::Tableau(i) => self.tableau.get(i).cloned().unwrap_or_default(),
        }
    }
    pub fn pile(&self, p: Pile) -> &[Card] {
        match p {
            Pile::Stock => &self.stock,
            Pile::Waste => &self.waste,
            Pile::Foundation(i) => self.foundations.get(i).map(Vec::as_slice).unwrap_or(&[]),
            Pile::Tableau(i) => self.tableau.get(i).map(Vec::as_slice).unwrap_or(&[]),
            Pile::Cell(_) => &[],
        }
    }
    fn moving(&self, p: Pile, index: usize) -> Option<Vec<Card>> {
        match p {
            Pile::Waste if index + 1 == self.waste.len() => Some(vec![*self.waste.last()?]),
            Pile::Foundation(i) if index + 1 == self.foundations.get(i)?.len() => {
                Some(vec![*self.foundations[i].last()?])
            }
            Pile::Cell(i) if index == 0 => Some(vec![self.cells.get(i).copied().flatten()?]),
            Pile::Tableau(i) => {
                let col = self.tableau.get(i)?;
                let tail = col.get(index..)?;
                if tail.is_empty() || tail.iter().any(|c| !c.face_up) {
                    return None;
                }
                if self.variant == Variant::FreeCell {
                    if tail
                        .windows(2)
                        .any(|w| color(w[0]) == color(w[1]) || ace_rank(w[0]) != ace_rank(w[1]) + 1)
                    {
                        return None;
                    }
                } else if self.variant.is_spider() {
                    if tail
                        .windows(2)
                        .any(|w| w[0].suit != w[1].suit || ace_rank(w[0]) != ace_rank(w[1]) + 1)
                    {
                        return None;
                    }
                } else if tail
                    .windows(2)
                    .any(|w| color(w[0]) == color(w[1]) || ace_rank(w[0]) != ace_rank(w[1]) + 1)
                {
                    return None;
                }
                Some(tail.to_vec())
            }
            _ => None,
        }
    }
    /// Whether this card and its trailing stack can be picked up.
    pub fn can_pick(&self, from: Pile, index: usize) -> bool {
        !self.won() && self.moving(from, index).is_some()
    }

    pub fn legal_move(&self, from: Pile, index: usize, to: Pile) -> bool {
        let Some(cs) = self.moving(from, index) else {
            return false;
        };
        if from == to {
            return false;
        }
        if self.variant == Variant::FreeCell
            && matches!((from, to), (Pile::Tableau(_), Pile::Tableau(_)))
        {
            let (Pile::Tableau(src), Pile::Tableau(dst)) = (from, to) else {
                unreachable!()
            };
            let free_cells = self.cells.iter().filter(|cell| cell.is_none()).count();
            let empty_columns = self
                .tableau
                .iter()
                .enumerate()
                .filter(|(i, col)| col.is_empty() && *i != src && *i != dst)
                .count();
            let capacity = (free_cells + 1).saturating_mul(
                1usize
                    .checked_shl(empty_columns.min(usize::BITS as usize - 1) as u32)
                    .unwrap_or(usize::MAX),
            );
            if cs.len() > capacity {
                return false;
            }
        }
        let c = cs[0];
        match (self.variant, to) {
            (Variant::FreeCell, Pile::Cell(i)) => {
                cs.len() == 1
                    && self.cells.get(i).is_some_and(Option::is_none)
                    && !matches!(from,Pile::Cell(j) if i==j)
            }
            (Variant::FreeCell | Variant::Klondike1 | Variant::Klondike3, Pile::Foundation(i)) => {
                cs.len() == 1
                    && self.foundations.get(i).is_some_and(|f| {
                        f.last().map_or(ace_rank(c) == 1, |top| {
                            top.suit == c.suit && ace_rank(c) == ace_rank(*top) + 1
                        })
                    })
            }
            (_, Pile::Tableau(i)) => {
                let Some(dst) = self.tableau.get(i) else {
                    return false;
                };
                if matches!(from,Pile::Tableau(j) if i==j) {
                    return false;
                }
                let Some(top) = dst.last() else {
                    return self.variant.is_spider()
                        || self.variant == Variant::FreeCell
                        || c.rank == 13;
                };
                if self.variant.is_spider() {
                    ace_rank(*top) == ace_rank(c) + 1
                } else {
                    color(*top) != color(c) && ace_rank(*top) == ace_rank(c) + 1
                }
            }
            _ => false,
        }
    }
    pub fn move_cards(&mut self, from: Pile, index: usize, to: Pile) -> bool {
        if !self.legal_move(from, index, to) {
            return false;
        }
        self.push_history();
        let cs = self.moving(from, index).unwrap();
        match from {
            Pile::Waste => {
                self.waste.pop();
            }
            Pile::Foundation(i) => {
                self.foundations[i].pop();
            }
            Pile::Cell(i) => self.cells[i] = None,
            Pile::Tableau(i) => {
                self.tableau[i].truncate(index);
                self.flip(i);
            }
            Pile::Stock => return false,
        }
        match to {
            Pile::Foundation(i) => self.foundations[i].extend(cs),
            Pile::Cell(i) => self.cells[i] = cs.first().copied(),
            Pile::Tableau(i) => self.tableau[i].extend(cs),
            _ => return false,
        }
        self.moves = self.moves.saturating_add(1);
        if self.variant.is_spider() {
            self.extract_runs()
        }
        true
    }
    fn push_history(&mut self) {
        if self.history.len() == 16 {
            self.history.remove(0);
        }
        self.history.push(snapshot(self));
    }
    fn flip(&mut self, i: usize) {
        if let Some(c) = self.tableau.get_mut(i).and_then(|v| v.last_mut()) {
            c.face_up = true
        }
    }
    fn extract_runs(&mut self) {
        loop {
            let mut found = None;
            for (i, col) in self.tableau.iter().enumerate() {
                if col.len() < 13 {
                    continue;
                }
                let start = col.len() - 13;
                let tail = &col[start..];
                if tail.iter().all(|c| c.face_up)
                    && tail
                        .windows(2)
                        .all(|w| w[0].suit == w[1].suit && ace_rank(w[0]) == ace_rank(w[1]) + 1)
                    && tail[0].rank == 13
                    && tail[12].rank == 14
                {
                    found = Some((i, start));
                    break;
                }
            }
            let Some((i, start)) = found else { break };
            let run = self.tableau[i].split_off(start);
            self.completed.push(run);
            self.flip(i);
        }
    }
    pub fn deal(&mut self) -> bool {
        match self.variant {
            Variant::Klondike1 | Variant::Klondike3 => {
                if !self.stock.is_empty() {
                    self.push_history();
                    let n = if self.variant == Variant::Klondike1 {
                        1
                    } else {
                        3
                    };
                    for _ in 0..n {
                        if let Some(mut c) = self.stock.pop() {
                            c.face_up = true;
                            self.waste.push(c)
                        }
                    }
                } else if !self.waste.is_empty() {
                    self.push_history();
                    self.stock = self
                        .waste
                        .drain(..)
                        .rev()
                        .map(|mut c| {
                            c.face_up = false;
                            c
                        })
                        .collect()
                } else {
                    return false;
                }
            }
            v if v.is_spider() => {
                if self.stock.len() < 10 || self.tableau.iter().any(Vec::is_empty) {
                    return false;
                }
                self.push_history();
                let start = self.stock.len() - 10;
                let dealt = self.stock.split_off(start);
                for (i, mut c) in dealt.into_iter().enumerate() {
                    c.face_up = true;
                    self.tableau[i].push(c)
                }
                self.extract_runs()
            }
            _ => return false,
        }
        self.moves = self.moves.saturating_add(1);
        true
    }
    pub fn undo(&mut self) -> bool {
        let Some(s) = self.history.pop() else {
            return false;
        };
        self.stock = s.stock;
        self.waste = s.waste;
        self.foundations = s.foundations;
        self.cells = s.cells;
        self.tableau = s.tableau;
        self.completed = s.completed;
        self.moves = s.moves;
        self.assisted |= s.assisted;
        true
    }
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }
    pub fn hint(&mut self) -> Option<(Pile, usize, Pile)> {
        let sources = [
            Pile::Waste,
            Pile::Cell(0),
            Pile::Cell(1),
            Pile::Cell(2),
            Pile::Cell(3),
        ];
        for p in sources {
            let n = self.cards(p).len();
            if n > 0 {
                for i in (0..n).rev() {
                    for j in 0..4 {
                        let q = Pile::Foundation(j);
                        if self.legal_move(p, i, q) {
                            self.assisted = true;
                            return Some((p, i, q));
                        }
                    }
                }
            }
        }
        for foundation in 0..self.foundations.len() {
            if let Some(index) = self.foundations[foundation].len().checked_sub(1) {
                let source = Pile::Foundation(foundation);
                for destination in 0..self.tableau.len() {
                    let target = Pile::Tableau(destination);
                    if self.legal_move(source, index, target) {
                        self.assisted = true;
                        return Some((source, index, target));
                    }
                }
            }
        }
        for (i, col) in self.tableau.iter().enumerate() {
            for k in (0..col.len()).rev() {
                let p = Pile::Tableau(i);
                if !self.variant.is_spider()
                    && self.moving(p, k).is_some_and(|cards| cards.len() == 1)
                {
                    for foundation in 0..self.foundations.len() {
                        let q = Pile::Foundation(foundation);
                        if self.legal_move(p, k, q) {
                            self.assisted = true;
                            return Some((p, k, q));
                        }
                    }
                }
                let destinations: Vec<_> = (0..self.tableau.len()).map(Pile::Tableau).collect();
                for q in destinations {
                    if self.legal_move(p, k, q) {
                        self.assisted = true;
                        return Some((p, k, q));
                    }
                }
            }
        }
        None
    }
    pub fn won(&self) -> bool {
        if self.variant.is_spider() {
            self.completed.len() == 8
        } else {
            self.foundations.iter().all(|p| p.len() == 13)
        }
    }
    pub fn advance_time(&mut self, ms: u64) {
        self.elapsed_ms = self
            .elapsed_ms
            .saturating_add(ms)
            .min(7 * 24 * 60 * 60 * 1000)
    }
    pub fn has_progress(&self) -> bool {
        self.moves > 0 || self.elapsed_ms > 0 || self.assisted
    }
    pub fn validate(&self) -> Result<(), String> {
        let n = self.variant.decks();
        if self.tableau.len() != self.variant.columns()
            || self.foundations.len() != 4
            || self.cells.len()
                != if self.variant == Variant::FreeCell {
                    4
                } else {
                    0
                }
        {
            return Err("invalid pile counts".into());
        }
        if self.elapsed_ms > 7 * 24 * 60 * 60 * 1000 || self.history.len() > 16 {
            return Err("invalid timer or history".into());
        }
        match self.variant {
            Variant::FreeCell
                if !self.stock.is_empty()
                    || !self.waste.is_empty()
                    || !self.completed.is_empty() =>
            {
                return Err("invalid FreeCell piles".into());
            }
            Variant::Klondike1 | Variant::Klondike3 if !self.completed.is_empty() => {
                return Err("invalid Klondike completed piles".into());
            }
            Variant::Spider1 | Variant::Spider2 | Variant::Spider4
                if self.foundations.iter().any(|pile| !pile.is_empty()) =>
            {
                return Err("Spider has no foundation piles".into());
            }
            _ => {}
        }
        let mut all = Vec::new();
        for p in [&self.stock, &self.waste] {
            all.extend(p.iter().copied())
        }
        for p in self
            .foundations
            .iter()
            .chain(self.tableau.iter())
            .chain(self.completed.iter())
        {
            all.extend(p.iter().copied())
        }
        all.extend(self.cells.iter().flatten().copied());
        let mut counts = std::collections::HashMap::new();
        for c in all {
            if !(2..=14).contains(&c.rank) {
                return Err("invalid rank".into());
            }
            *counts.entry((c.rank, c.suit)).or_insert(0usize) += 1;
        }
        let expected = n * 52;
        let multiplicity = match self.variant {
            Variant::Spider1 => 8,
            Variant::Spider2 => 4,
            _ => n,
        };
        if counts.values().sum::<usize>() != expected || counts.values().any(|&v| v > multiplicity)
        {
            return Err("invalid deck inventory".into());
        }
        for rank in 2..=14 {
            for suit in suits() {
                let wanted = match self.variant {
                    Variant::Spider1 => {
                        if suit == Suit::Spades {
                            8
                        } else {
                            0
                        }
                    }
                    Variant::Spider2 => {
                        if matches!(suit, Suit::Spades | Suit::Hearts) {
                            4
                        } else {
                            0
                        }
                    }
                    Variant::Spider4 => 2,
                    _ => 1,
                };
                if counts.get(&(rank, suit)).copied().unwrap_or(0) != wanted {
                    return Err("invalid deck inventory".into());
                }
            }
        }
        if self.variant.is_spider()
            && self.completed.iter().any(|r| {
                r.len() != 13
                    || r[0].rank != 13
                    || r[12].rank != 14
                    || r.windows(2)
                        .any(|w| w[0].suit != w[1].suit || ace_rank(w[0]) != ace_rank(w[1]) + 1)
            })
        {
            return Err("invalid completed run".into());
        }
        if self.foundations.iter().any(|foundation| {
            foundation.len() > 13
                || foundation.iter().enumerate().any(|(index, card)| {
                    !card.face_up
                        || ace_rank(*card) != index as u8 + 1
                        || (index > 0 && card.suit != foundation[0].suit)
                })
        }) {
            return Err("invalid foundation sequence".into());
        }
        if self.stock.iter().any(|card| card.face_up)
            || self.waste.iter().any(|card| !card.face_up)
            || self.cells.iter().flatten().any(|card| !card.face_up)
            || self.completed.iter().flatten().any(|card| !card.face_up)
        {
            return Err("card has invalid face-up state for its pile".into());
        }
        for col in &self.tableau {
            let mut seen_up = false;
            let mut face_up = Vec::new();
            for c in col {
                if c.face_up {
                    seen_up = true;
                    face_up.push(*c);
                } else if seen_up {
                    return Err("face-down card beneath face-up card".into());
                }
            }
            if self.variant == Variant::FreeCell && face_up.len() != col.len() {
                return Err("FreeCell tableau cards must all be face up".into());
            }
            if matches!(self.variant, Variant::Klondike1 | Variant::Klondike3)
                && !col.is_empty()
                && !seen_up
            {
                return Err("Klondike tableau column must expose its top card".into());
            }
            if matches!(self.variant, Variant::Klondike1 | Variant::Klondike3)
                && face_up.windows(2).any(|pair| {
                    ace_rank(pair[0]) != ace_rank(pair[1]) + 1 || color(pair[0]) == color(pair[1])
                })
            {
                return Err("invalid tableau sequence".into());
            }
        }
        if self.variant.is_spider() && self.completed.len() > 8 {
            return Err("too many completed Spider runs".into());
        }
        for saved in &self.history {
            let previous = Game {
                variant: self.variant,
                seed: self.seed,
                stock: saved.stock.clone(),
                waste: saved.waste.clone(),
                foundations: saved.foundations.clone(),
                cells: saved.cells.clone(),
                tableau: saved.tableau.clone(),
                completed: saved.completed.clone(),
                elapsed_ms: 0,
                moves: saved.moves,
                assisted: saved.assisted,
                history: Vec::new(),
            };
            previous
                .validate()
                .map_err(|error| format!("invalid history snapshot: {error}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn assistance_stays_set_after_undo() {
        let mut game = Game::new(Variant::Klondike1, 42);
        assert!(game.deal());
        game.assisted = true;
        assert!(game.undo());
        assert!(game.assisted);
    }
    use super::*;

    fn oracle() -> serde_json::Value {
        serde_json::from_str(include_str!("../../tests/fixtures/patience-oracle.json")).unwrap()
    }
    fn oracle_card(value: &serde_json::Value) -> Card {
        let rank = match value["rank"].as_str().unwrap() {
            "A" => 14,
            "K" => 13,
            "Q" => 12,
            "J" => 11,
            n => n.parse().unwrap(),
        };
        let suit = match value["suit"].as_str().unwrap() {
            "spades" => Suit::Spades,
            "hearts" => Suit::Hearts,
            "clubs" => Suit::Clubs,
            "diamonds" => Suit::Diamonds,
            _ => panic!("unknown fixture suit"),
        };
        Card::new(rank, suit, value["faceUp"].as_bool().unwrap())
    }
    fn oracle_cards(value: &serde_json::Value, name: &str, key: &str) -> Vec<Card> {
        value["states"][name][key]
            .as_array()
            .unwrap()
            .iter()
            .map(oracle_card)
            .collect()
    }
    fn oracle_piles(value: &serde_json::Value, name: &str, key: &str) -> Vec<Vec<Card>> {
        value["states"][name][key]
            .as_array()
            .unwrap()
            .iter()
            .map(|pile| pile.as_array().unwrap().iter().map(oracle_card).collect())
            .collect()
    }

    #[test]
    fn typescript_klondike_oracle_covers_stock_waste_runs_and_foundation_back_moves() {
        let fixture = oracle();
        let mut draw = Game::new(Variant::Klondike3, 1);
        draw.stock = oracle_cards(&fixture, "klondike_draw3", "stock");
        let input = &fixture["states"]["klondike_draw3"]["stock"];
        let by_id: std::collections::HashMap<_, _> = input
            .as_array()
            .unwrap()
            .iter()
            .map(|card| (card["id"].as_str().unwrap().to_owned(), oracle_card(card)))
            .collect();
        assert!(draw.deal());
        let dealt: Vec<_> = fixture["expected"]["klondike_draw3"]["waste"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| {
                let mut card = by_id[id.as_str().unwrap()];
                card.face_up = true;
                card
            })
            .collect();
        assert_eq!(draw.waste, dealt);
        let mut expected_top = by_id[fixture["expected"]["klondike_draw3"]["waste_top"]
            .as_str()
            .unwrap()];
        expected_top.face_up = true;
        assert_eq!(draw.waste.last().unwrap(), &expected_top);

        let mut recycle = Game::new(Variant::Klondike1, 2);
        recycle.stock.clear();
        let waste = &fixture["states"]["klondike_recycle"]["waste"];
        recycle.waste = waste.as_array().unwrap().iter().map(oracle_card).collect();
        assert!(recycle.deal());
        let stock_ids: Vec<_> = fixture["expected"]["klondike_recycle"]["stock"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap())
            .collect();
        let by_id: std::collections::HashMap<_, _> = waste
            .as_array()
            .unwrap()
            .iter()
            .map(|card| (card["id"].as_str().unwrap(), oracle_card(card)))
            .collect();
        assert_eq!(
            recycle.stock,
            stock_ids
                .iter()
                .map(|id| {
                    let mut card = by_id[id];
                    card.face_up = false;
                    card
                })
                .collect::<Vec<_>>()
        );
        let mut expected_next = by_id[fixture["expected"]["klondike_recycle"]["next_draw"]
            .as_str()
            .unwrap()];
        expected_next.face_up = false;
        assert_eq!(recycle.stock.last().unwrap(), &expected_next);

        let expected = &fixture["expected"]["klondike_stack"];
        let mut klondike = Game::new(Variant::Klondike1, 3);
        klondike.tableau = oracle_piles(&fixture, "klondike_stack", "tableau");
        assert_eq!(
            klondike.legal_move(Pile::Tableau(0), 1, Pile::Tableau(1)),
            expected["valid_substack"]
        );
        assert_eq!(
            klondike.legal_move(Pile::Tableau(0), 0, Pile::Tableau(2)),
            expected["valid_whole_stack"]
        );
        assert_eq!(
            klondike.legal_move(Pile::Tableau(0), 1, Pile::Tableau(2)),
            expected["invalid_same_color"]
        );
        let mut foundation = Game::new(Variant::Klondike1, 4);
        foundation.foundations = oracle_piles(&fixture, "klondike_foundation_back", "foundations");
        foundation.tableau = oracle_piles(&fixture, "klondike_foundation_back", "tableau");
        assert_eq!(
            foundation.legal_move(Pile::Foundation(0), 1, Pile::Tableau(0)),
            fixture["expected"]["klondike_foundation_back"]
        );
    }

    #[test]
    fn typescript_freecell_oracle_covers_sequences_capacity_and_foundation_back_moves() {
        let fixture = oracle();
        let mut valid = Game::new(Variant::FreeCell, 1);
        valid.tableau = oracle_piles(&fixture, "freecell_valid_stack", "cascades");
        assert_eq!(
            valid.legal_move(Pile::Tableau(0), 0, Pile::Tableau(1)),
            fixture["expected"]["freecell_valid_stack"]
        );
        let mut capacity = Game::new(Variant::FreeCell, 2);
        capacity.tableau = oracle_piles(&fixture, "freecell_empty_target_capacity", "cascades");
        capacity.cells = fixture["states"]["freecell_empty_target_capacity"]["freeCells"]
            .as_array()
            .unwrap()
            .iter()
            .map(|cell| {
                if cell.is_null() {
                    None
                } else {
                    Some(oracle_card(cell))
                }
            })
            .collect();
        assert_eq!(
            capacity.legal_move(Pile::Tableau(0), 0, Pile::Tableau(1)),
            fixture["expected"]["freecell_empty_target_capacity"]["strict_accepts"]
        );
        assert!(!capacity.legal_move(Pile::Tableau(0), 0, Pile::Tableau(1)));
        let mut foundation = Game::new(Variant::FreeCell, 3);
        foundation.foundations = oracle_piles(&fixture, "freecell_foundation_back", "foundations");
        foundation.tableau = oracle_piles(&fixture, "freecell_foundation_back", "cascades");
        assert_eq!(
            foundation.legal_move(Pile::Foundation(0), 1, Pile::Tableau(0)),
            fixture["expected"]["freecell_foundation_back"]
        );
    }

    #[test]
    fn typescript_spider_oracle_covers_mixed_suit_runs_deals_and_run_extraction() {
        let fixture = oracle();
        let mut mixed = Game::new(Variant::Spider4, 1);
        mixed.tableau = oracle_piles(&fixture, "spider_mixed_stack", "tableau");
        assert_eq!(
            mixed.legal_move(Pile::Tableau(0), 0, Pile::Tableau(1)),
            fixture["expected"]["spider_mixed_stack"]["mixed_suit_run_movable"]
        );
        let mut singleton = Game::new(Variant::Spider4, 2);
        singleton.tableau = oracle_piles(&fixture, "spider_mixed_suit_placement", "tableau");
        assert_eq!(
            singleton.legal_move(Pile::Tableau(0), 0, Pile::Tableau(1)),
            fixture["expected"]["spider_mixed_suit_placement"]
        );

        let mut denied = Game::new(Variant::Spider1, 3);
        denied.stock = oracle_cards(&fixture, "spider_stock_empty_column", "stock");
        denied.tableau = oracle_piles(&fixture, "spider_stock_empty_column", "tableau");
        assert_eq!(
            denied.deal(),
            fixture["expected"]["spider_stock_empty_column"]["deal_succeeds"]
        );
        let mut auto = Game::new(Variant::Spider1, 4);
        auto.stock = oracle_cards(&fixture, "spider_autocomplete_on_deal", "stock");
        auto.tableau = oracle_piles(&fixture, "spider_autocomplete_on_deal", "tableau");
        assert!(auto.deal());
        assert_eq!(
            auto.completed.len(),
            fixture["expected"]["spider_autocomplete_on_deal"]["completed_runs"]
        );
        assert_eq!(
            auto.tableau[0].len(),
            fixture["expected"]["spider_autocomplete_on_deal"]["first_column_length"]
        );
        assert_eq!(
            auto.stock.len(),
            fixture["expected"]["spider_autocomplete_on_deal"]["stock_length"]
        );

        let mut chain = Game::new(Variant::Spider1, 5);
        chain.tableau = oracle_piles(&fixture, "spider_chained_runs_after_move", "tableau");
        assert!(chain.move_cards(Pile::Tableau(1), 0, Pile::Tableau(2)));
        assert_eq!(
            chain.completed.len(),
            fixture["expected"]["spider_chained_runs_after_move"]["completed_runs"]
        );
        assert_eq!(
            chain.tableau[0].len(),
            fixture["expected"]["spider_chained_runs_after_move"]["first_column_length"]
        );
    }

    #[test]
    fn seeded_deals_are_repeatable_and_valid() {
        for v in Variant::ALL {
            let a = Game::new(v, u64::MAX);
            let b = Game::new(v, u64::MAX);
            assert_eq!(a, b);
            a.validate().unwrap();
        }
    }
    #[test]
    fn klondike_deal_and_recycle_undo_exactly() {
        let mut g = Game::new(Variant::Klondike3, 7);
        let before = snapshot(&g);
        assert!(g.deal());
        assert_eq!(g.waste.len(), 3);
        assert!(g.undo());
        assert_eq!(snapshot(&g), before);
        while !g.stock.is_empty() {
            g.deal();
        }
        let waste = g.waste.clone();
        assert!(g.deal());
        assert_eq!(g.stock.len(), waste.len());
        assert!(g.undo());
        assert_eq!(g.waste, waste);
    }
    #[test]
    fn tableau_rules_and_face_flip() {
        let mut g = Game::new(Variant::Klondike1, 22);
        g.tableau = vec![
            vec![Card::new(7, Suit::Hearts, true)],
            vec![
                Card::new(8, Suit::Spades, true),
                Card::new(6, Suit::Clubs, false),
            ],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        g.stock.clear();
        assert!(!g.legal_move(Pile::Tableau(0), 0, Pile::Tableau(1)));
        assert!(!g.legal_move(Pile::Tableau(0), 0, Pile::Tableau(2)));
        assert!(!g.move_cards(Pile::Tableau(1), 1, Pile::Tableau(0)));
    }
    #[test]
    fn freecell_capacity_and_single_card_cells() {
        let mut g = Game::new(Variant::FreeCell, 55);
        g.tableau = vec![
            vec![
                Card::new(9, Suit::Spades, true),
                Card::new(8, Suit::Hearts, true),
                Card::new(7, Suit::Clubs, true),
            ],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        g.cells = vec![None; 4];
        assert!(g.legal_move(Pile::Tableau(0), 1, Pile::Tableau(1)));
        assert!(!g.legal_move(Pile::Tableau(0), 0, Pile::Cell(0)));
        assert!(g.move_cards(Pile::Tableau(0), 2, Pile::Cell(0)));
        assert!(g.undo());
    }
    #[test]
    fn spider_extracts_complete_runs_and_reports_win() {
        let mut g = Game::new(Variant::Spider1, 9);
        g.tableau = vec![Vec::new(); 10];
        g.stock.clear();
        let run: Vec<Card> = (2..=13)
            .rev()
            .chain(std::iter::once(14))
            .map(|rank| Card::new(rank, Suit::Spades, true))
            .collect();
        g.tableau[0] = run;
        g.extract_runs();
        assert_eq!(g.completed.len(), 1);
        assert!(g.tableau[0].is_empty());
        assert!(!g.won());
        g.completed = vec![g.completed[0].clone(); 8];
        assert!(g.won());
    }

    #[test]
    fn klondike_moves_only_alternating_descending_runs() {
        let mut g = Game::new(Variant::Klondike1, 5);
        g.tableau = vec![
            vec![
                Card::new(9, Suit::Spades, true),
                Card::new(8, Suit::Hearts, true),
            ],
            vec![Card::new(9, Suit::Clubs, true)],
            vec![Card::new(10, Suit::Hearts, true)],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        assert!(g.legal_move(Pile::Tableau(0), 1, Pile::Tableau(1)));
        assert!(g.legal_move(Pile::Tableau(0), 0, Pile::Tableau(2)));
        assert!(!g.legal_move(Pile::Tableau(0), 1, Pile::Tableau(2)));
        let before = snapshot(&g);
        assert!(g.move_cards(Pile::Tableau(0), 0, Pile::Tableau(2)));
        assert!(g.undo());
        assert_eq!(snapshot(&g), before);
    }

    #[test]
    fn maximum_history_for_all_three_saves_fits_the_global_save_limit() {
        let mut save = crate::save::Save::default();
        for game in [&mut save.solitaire, &mut save.freecell, &mut save.spider] {
            game.history = vec![snapshot(game); 16];
        }
        let json = serde_json::to_vec(&save).unwrap();
        assert!(
            json.len() <= crate::save::MAX_SAVE_BYTES,
            "serialized save was {} bytes",
            json.len()
        );
        save.validate().unwrap();
    }

    #[test]
    fn validation_checks_history_snapshots() {
        let mut game = Game::new(Variant::Klondike1, 12);
        let mut old = snapshot(&game);
        old.foundations[0].push(Card::new(7, Suit::Spades, true));
        game.history.push(old);
        assert!(game.validate().unwrap_err().contains("history snapshot"));
    }

    #[test]
    fn timer_and_assistance_count_as_progress() {
        let mut game = Game::new(Variant::Klondike1, 91);
        assert!(!game.has_progress());
        game.advance_time(1);
        assert!(game.has_progress());
        let mut hinted = Game::new(Variant::Klondike1, 92);
        hinted.tableau[0] = vec![Card::new(14, Suit::Spades, true)];
        assert!(hinted.hint().is_some());
        assert!(hinted.assisted && hinted.has_progress());
    }

    #[test]
    fn timer_saturates_and_history_is_bounded() {
        let mut g = Game::new(Variant::Klondike1, 1);
        g.advance_time(u64::MAX);
        assert_eq!(g.elapsed_ms, 604_800_000);
        for _ in 0..30 {
            g.deal();
        }
        assert!(g.history.len() <= 16);
    }
}
