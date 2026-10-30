use serde::{Deserialize, Serialize};

const HISTORY_LIMIT: usize = 100;
const ZEN_HISTORY_LIMIT: usize = 80;
const ZEN_SNAPSHOT_LIMIT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    Turtle,
    Arena,
    Monument,
}

impl Layout {
    pub const ALL: [Self; 3] = [Self::Turtle, Self::Arena, Self::Monument];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Turtle => "Turtle",
            Self::Arena => "Arena",
            Self::Monument => "Monument",
        }
    }
    fn coordinates(self) -> Vec<(i16, i16, u8)> {
        let mut c = Vec::with_capacity(144);
        match self {
            Self::Turtle => {
                let rows: &[(i16, &[i16])] = &[
                    (0, &[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24]),
                    (2, &[6, 8, 10, 12, 14, 16, 18, 20]),
                    (4, &[4, 6, 8, 10, 12, 14, 16, 18, 20, 22]),
                    (6, &[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24]),
                    (8, &[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24]),
                    (10, &[4, 6, 8, 10, 12, 14, 16, 18, 20, 22]),
                    (12, &[6, 8, 10, 12, 14, 16, 18, 20]),
                    (14, &[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24]),
                ];
                for (y, xs) in rows {
                    for x in *xs {
                        c.push((*x, *y, 0));
                    }
                }
                c.extend([(0, 7, 0), (26, 7, 0), (28, 7, 0)]);
                for y in [2, 4, 6, 8, 10, 12] {
                    for x in [8, 10, 12, 14, 16, 18] {
                        c.push((x, y, 1));
                    }
                }
                for y in [4, 6, 8, 10] {
                    for x in [10, 12, 14, 16] {
                        c.push((x, y, 2));
                    }
                }
                for y in [6, 8] {
                    for x in [12, 14] {
                        c.push((x, y, 3));
                    }
                }
                c.push((13, 7, 4));
            }
            Self::Arena => {
                for x in (0..=18).step_by(2) {
                    for y in (0..=18).step_by(2) {
                        if x == 0 || x == 18 || y == 0 || y == 18 {
                            c.push((x, y, 0));
                        }
                    }
                }
                for x in (2..=16).step_by(2) {
                    for y in (2..=16).step_by(2) {
                        c.push((x, y, 0));
                    }
                }
                for x in (4..=14).step_by(2) {
                    for y in (4..=14).step_by(2) {
                        c.push((x, y, 1));
                    }
                }
                for z in [2, 3] {
                    for x in (8..=10).step_by(2) {
                        for y in (8..=10).step_by(2) {
                            c.push((x, y, z));
                        }
                    }
                }
            }
            Self::Monument => {
                for x in (0..=22).step_by(2) {
                    for y in (0..=10).step_by(2) {
                        c.push((x, y, 0));
                    }
                }
                for x in (2..=20).step_by(2) {
                    for y in (2..=8).step_by(2) {
                        c.push((x, y, 1));
                    }
                }
                for x in (4..=18).step_by(2) {
                    for y in (4..=6).step_by(2) {
                        c.push((x, y, 2));
                    }
                }
                for x in (6..=16).step_by(2) {
                    for y in (4..=6).step_by(2) {
                        c.push((x, y, 3));
                    }
                }
                for x in (8..=14).step_by(2) {
                    c.push((x, 5, 4));
                }
            }
        }
        c
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub face: u8,
    pub x: i16,
    pub y: i16,
    pub z: u8,
    pub removed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ZenUndoSnapshot {
    history_index: usize,
    faces_before: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub layout: Layout,
    pub seed: u64,
    pub tiles: Vec<Tile>,
    pub elapsed_ms: u64,
    pub assisted: bool,
    pub shuffles: u32,
    pub history: Vec<[usize; 2]>,
    #[serde(default)]
    pub zen: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    zen_undo_snapshots: Vec<ZenUndoSnapshot>,
}

impl Game {
    pub fn new(layout: Layout, seed: u64) -> Self {
        let coords = layout.coordinates();
        assert_eq!(coords.len(), 144, "invalid canonical Mahjong geometry");
        let mut rng = seed;
        let (solution, mut faces) = construct_trace(&coords, &mut rng)
            .expect("canonical Mahjong layout must have a removal trace");
        let mut buckets: Vec<Vec<u8>> = (0..42).map(|_| Vec::new()).collect();
        for (f, bucket) in buckets.iter_mut().enumerate() {
            let copies = if f < 34 { 4 } else { 1 };
            bucket.extend(std::iter::repeat_n(f as u8, copies));
        }
        for bucket in &mut buckets {
            shuffle(bucket, &mut rng);
        }
        let mut pairs = Vec::new();
        for bucket in &buckets {
            for pair in bucket.as_chunks::<2>().0 {
                pairs.push(*pair);
            }
        }
        // The four flowers and four seasons each form two cross-face pairs.
        // Rebuild these special pairs as family buckets.
        let mut specials: Vec<u8> = (34..38).collect();
        shuffle(&mut specials, &mut rng);
        let mut seasons: Vec<u8> = (38..42).collect();
        shuffle(&mut seasons, &mut rng);
        pairs.retain(|p| p[0] < 34 && p[1] < 34);
        for group in [specials, seasons] {
            pairs.push([group[0], group[1]]);
            pairs.push([group[2], group[3]]);
        }
        shuffle(&mut pairs, &mut rng);
        for (i, p) in solution.iter().enumerate() {
            faces[p[0]] = pairs[i][0];
            faces[p[1]] = pairs[i][1];
        }
        let tiles = coords
            .into_iter()
            .enumerate()
            .map(|(i, (x, y, z))| Tile {
                face: faces[i],
                x,
                y,
                z,
                removed: false,
            })
            .collect();
        Self {
            layout,
            seed,
            tiles,
            elapsed_ms: 0,
            assisted: false,
            shuffles: 0,
            history: Vec::new(),
            zen: false,
            zen_undo_snapshots: Vec::new(),
        }
    }
    pub fn remaining(&self) -> usize {
        self.tiles.iter().filter(|t| !t.removed).count()
    }
    pub fn won(&self) -> bool {
        self.remaining() == 0
    }
    fn selectable_from(tiles: &[Tile], i: usize) -> bool {
        if i >= tiles.len() || tiles[i].removed {
            return false;
        }
        let t = &tiles[i];
        let above = tiles
            .iter()
            .any(|o| !o.removed && o.z > t.z && (o.x - t.x).abs() <= 1 && (o.y - t.y).abs() <= 1);
        if above {
            return false;
        }
        let left = tiles.iter().any(|o| {
            !o.removed && o.z == t.z && (-2..=-1).contains(&(o.x - t.x)) && (o.y - t.y).abs() <= 1
        });
        let right = tiles.iter().any(|o| {
            !o.removed && o.z == t.z && (1..=2).contains(&(o.x - t.x)) && (o.y - t.y).abs() <= 1
        });
        !(left && right)
    }
    pub fn selectable(&self, index: usize) -> bool {
        Self::selectable_from(&self.tiles, index)
    }
    fn matches(a: u8, b: u8) -> bool {
        if a >= 34 && b >= 34 {
            (a < 38) == (b < 38)
        } else {
            a == b
        }
    }
    pub fn available_pairs(&self) -> Vec<[usize; 2]> {
        let open: Vec<_> = (0..self.tiles.len())
            .filter(|&i| self.selectable(i))
            .collect();
        let mut out = Vec::new();
        for ai in 0..open.len() {
            for bi in ai + 1..open.len() {
                let (a, b) = (open[ai], open[bi]);
                if Self::matches(self.tiles[a].face, self.tiles[b].face) {
                    out.push([a, b]);
                }
            }
        }
        out
    }
    pub fn hint(&mut self) -> Option<[usize; 2]> {
        let p = self.available_pairs().into_iter().next()?;
        self.assisted = true;
        Some(p)
    }
    pub fn match_pair(&mut self, a: usize, b: usize) -> bool {
        if a == b
            || a >= self.tiles.len()
            || b >= self.tiles.len()
            || !self.selectable(a)
            || !self.selectable(b)
            || !Self::matches(self.tiles[a].face, self.tiles[b].face)
        {
            return false;
        }
        let faces_before = self
            .zen
            .then(|| self.tiles.iter().map(|tile| tile.face).collect::<Vec<_>>());
        self.tiles[a].removed = true;
        self.tiles[b].removed = true;
        self.history.push([a, b]);
        let history_limit = if self.zen {
            ZEN_HISTORY_LIMIT
        } else {
            HISTORY_LIMIT
        };
        if self.history.len() > history_limit {
            self.drop_history_prefix(self.history.len() - history_limit);
        }
        if self.zen && self.stuck() {
            let faces_before = faces_before.expect("Zen match captured its pre-move faces");
            self.zen_undo_snapshots.push(ZenUndoSnapshot {
                history_index: self.history.len() - 1,
                faces_before,
            });
            while self.zen_undo_snapshots.len() > ZEN_SNAPSHOT_LIMIT {
                let lost_prefix = self.zen_undo_snapshots.remove(0).history_index + 1;
                self.drop_history_prefix(lost_prefix);
            }
            let auto_seed = self
                .seed
                .wrapping_add(0x9e37_79b9_7f4a_7c15u64.wrapping_mul(u64::from(self.shuffles) + 1));
            self.shuffle_remaining(auto_seed);
        }
        true
    }
    fn drop_history_prefix(&mut self, count: usize) {
        let count = count.min(self.history.len());
        self.history.drain(0..count);
        self.zen_undo_snapshots.retain_mut(|snapshot| {
            if snapshot.history_index < count {
                false
            } else {
                snapshot.history_index -= count;
                true
            }
        });
    }
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }
    pub fn undo(&mut self) -> bool {
        let Some([a, b]) = self.history.pop() else {
            return false;
        };
        if let Some(snapshot) = self
            .zen_undo_snapshots
            .pop_if(|snapshot| snapshot.history_index == self.history.len())
        {
            for (tile, face) in self.tiles.iter_mut().zip(snapshot.faces_before) {
                tile.face = face;
            }
        }
        self.tiles[a].removed = false;
        self.tiles[b].removed = false;
        true
    }
    pub fn shuffle(&mut self, seed: u64) -> bool {
        if !self.shuffle_remaining(seed) {
            return false;
        }
        self.history.clear();
        self.zen_undo_snapshots.clear();
        self.assisted = true;
        true
    }
    fn shuffle_remaining(&mut self, seed: u64) -> bool {
        let ids: Vec<usize> = (0..self.tiles.len())
            .filter(|&i| !self.tiles[i].removed)
            .collect();
        if ids.len() < 2 {
            return false;
        }
        let mut faces: Vec<u8> = ids.iter().map(|&i| self.tiles[i].face).collect();
        let mut rng = seed;
        shuffle(&mut faces, &mut rng);
        for (i, f) in ids.iter().zip(faces.iter()) {
            self.tiles[*i].face = *f;
        }
        if self.available_pairs().is_empty() {
            let open: Vec<_> = ids
                .iter()
                .copied()
                .filter(|&i| self.selectable(i))
                .collect();
            let matching = (0..faces.len()).find_map(|i| {
                (i + 1..faces.len())
                    .find(|&j| Self::matches(faces[i], faces[j]))
                    .map(|j| (i, j))
            });
            if let (Some((fi, fj)), Some((&a, &b))) =
                (matching, open.iter().zip(open.iter().skip(1)).next())
            {
                self.tiles[a].face = faces[fi];
                self.tiles[b].face = faces[fj];
                let rest = faces
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, face)| (i != fi && i != fj).then_some(face));
                for (i, v) in ids.iter().copied().filter(|i| *i != a && *i != b).zip(rest) {
                    self.tiles[i].face = v;
                }
            }
        }
        self.shuffles = self.shuffles.saturating_add(1);
        true
    }
    pub fn advance_time(&mut self, ms: u64) {
        if self.zen || self.won() {
            return;
        }
        self.elapsed_ms = self
            .elapsed_ms
            .saturating_add(ms)
            .min(7 * 24 * 60 * 60 * 1000);
    }
    pub fn stuck(&self) -> bool {
        !self.won() && self.available_pairs().is_empty()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.tiles.len() != 144
            || self.history.len()
                > if self.zen {
                    ZEN_HISTORY_LIMIT
                } else {
                    HISTORY_LIMIT
                }
            || self.zen_undo_snapshots.len() > ZEN_SNAPSHOT_LIMIT
            || (!self.zen && !self.zen_undo_snapshots.is_empty())
            || self.tiles.iter().any(|t| t.face >= 42)
            || self.elapsed_ms > 7 * 24 * 60 * 60 * 1000
            || self
                .history
                .iter()
                .flatten()
                .any(|&i| i >= self.tiles.len())
            || self
                .history
                .iter()
                .any(|p| p[0] == p[1] || !self.tiles[p[0]].removed || !self.tiles[p[1]].removed)
        {
            return Err("invalid Mahjong tile count, face, history, or elapsed time".into());
        }
        let coords = self.layout.coordinates();
        if self
            .tiles
            .iter()
            .map(|t| (t.x, t.y, t.z))
            .collect::<Vec<_>>()
            != coords
        {
            return Err("Mahjong coordinates do not match the selected layout".into());
        }
        let mut history_coverage = vec![false; self.tiles.len()];
        for [a, b] in &self.history {
            if history_coverage[*a]
                || history_coverage[*b]
                || !Self::matches(self.tiles[*a].face, self.tiles[*b].face)
            {
                return Err("Mahjong history contains duplicate or nonmatching removals".into());
            }
            history_coverage[*a] = true;
            history_coverage[*b] = true;
        }
        let mut inventory = [0u8; 42];
        for t in &self.tiles {
            inventory[t.face as usize] += 1;
        }
        if (0..34).all(|f| inventory[f] == 4) && (34..42).all(|f| inventory[f] == 1) {
            let mut previous_snapshot_index = None;
            for snapshot in &self.zen_undo_snapshots {
                if snapshot.faces_before.len() != 144
                    || snapshot.faces_before.iter().any(|&face| face >= 42)
                    || snapshot.history_index >= self.history.len()
                    || previous_snapshot_index.is_some_and(|index| index >= snapshot.history_index)
                {
                    return Err("invalid Zen undo snapshot shape or order".into());
                }
                let [a, b] = self.history[snapshot.history_index];
                if !Self::matches(snapshot.faces_before[a], snapshot.faces_before[b]) {
                    return Err("Zen undo snapshot does not preserve the matched pair".into());
                }
                let mut snapshot_inventory = [0u8; 42];
                for &face in &snapshot.faces_before {
                    snapshot_inventory[face as usize] += 1;
                }
                if snapshot_inventory != inventory {
                    return Err("Zen undo snapshot face inventory is invalid".into());
                }
                let mut rewindable_at_snapshot = vec![false; self.tiles.len()];
                for [a, b] in self.history.iter().skip(snapshot.history_index) {
                    rewindable_at_snapshot[*a] = true;
                    rewindable_at_snapshot[*b] = true;
                }
                if self.tiles.iter().enumerate().any(|(index, tile)| {
                    tile.removed
                        && !rewindable_at_snapshot[index]
                        && snapshot.faces_before[index] != tile.face
                }) {
                    return Err("Zen undo snapshot changes an earlier removed tile".into());
                }
                previous_snapshot_index = Some(snapshot.history_index);
            }
            let mut remaining = [0u8; 42];
            for tile in self.tiles.iter().filter(|tile| !tile.removed) {
                remaining[tile.face as usize] += 1;
            }
            let normal_faces_even = (0..34).all(|face| remaining[face] % 2 == 0);
            let flowers_even = (34..38).map(|face| remaining[face]).sum::<u8>() % 2 == 0;
            let seasons_even = (38..42).map(|face| remaining[face]).sum::<u8>() % 2 == 0;
            if normal_faces_even && flowers_even && seasons_even {
                Ok(())
            } else {
                Err("remaining Mahjong tiles cannot be removed in matching pairs".into())
            }
        } else {
            Err("Mahjong face inventory is invalid".into())
        }
    }
}

fn shuffle<T>(v: &mut [T], rng: &mut u64) {
    for i in (1..v.len()).rev() {
        let j = (next(rng) % (i as u64 + 1)) as usize;
        v.swap(i, j);
    }
}
fn next(s: &mut u64) -> u64 {
    *s = s.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *s;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
fn construct_trace(c: &[(i16, i16, u8)], rng: &mut u64) -> Option<(Vec<[usize; 2]>, Vec<u8>)> {
    let mut attempt_seed = *rng;
    for attempt in 0..9 {
        let mut remain = vec![true; c.len()];
        let mut trace = Vec::new();
        let mut local = attempt_seed;
        attempt_seed = next(&mut attempt_seed);
        if attempt == 8 {
            // The last bounded attempt is the deterministic geometry fallback.
            local = 0;
        }
        while remain.iter().any(|x| *x) {
            let mut open = Vec::new();
            for i in 0..c.len() {
                if !remain[i] {
                    continue;
                }
                let (x, y, z) = c[i];
                let above = (0..c.len()).any(|j| {
                    remain[j] && c[j].2 > z && (c[j].0 - x).abs() <= 1 && (c[j].1 - y).abs() <= 1
                });
                let left = (0..c.len()).any(|j| {
                    remain[j]
                        && c[j].2 == z
                        && (-2..=-1).contains(&(c[j].0 - x))
                        && (c[j].1 - y).abs() <= 1
                });
                let right = (0..c.len()).any(|j| {
                    remain[j]
                        && c[j].2 == z
                        && (1..=2).contains(&(c[j].0 - x))
                        && (c[j].1 - y).abs() <= 1
                });
                if !above && !(left && right) {
                    open.push(i);
                }
            }
            if open.len() < 2 {
                break;
            }
            shuffle(&mut open, &mut local);
            let a = open[0];
            let b = open[1];
            remain[a] = false;
            remain[b] = false;
            trace.push([a, b]);
        }
        if trace.len() * 2 == c.len() {
            // Keep the seed-derived stream for face assignment, including when
            // the successful geometry trace came from the deterministic fallback.
            *rng = attempt_seed;
            let faces = (0..c.len()).map(|i| (i % 42) as u8).collect();
            return Some((trace, faces));
        }
    }
    None
}

/// Return the deterministic full-removal trace for a seeded canonical deal.
/// The trace is derived on demand and is never stored in or serialized with a game.
pub fn solution_pairs(layout: Layout, seed: u64) -> Vec<[usize; 2]> {
    let mut rng = seed;
    construct_trace(&layout.coordinates(), &mut rng)
        .expect("canonical Mahjong layout must have a removal trace")
        .0
}

#[cfg(test)]
#[path = "mahjong_parity.rs"]
mod mahjong_parity;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_geometry_and_constructive_solutions() {
        for layout in Layout::ALL {
            for seed in [0, 1, 7, 42, u64::MAX] {
                let mut g = Game::new(layout, seed);
                assert!(g.validate().is_ok());
                assert_eq!(g.tiles.len(), 144);
                for [a, b] in solution_pairs(layout, seed) {
                    assert!(
                        g.match_pair(a, b),
                        "{} seed {} pair {:?}",
                        layout.label(),
                        seed,
                        [a, b]
                    );
                }
                assert!(g.won());
            }
        }
    }
    #[test]
    fn history_hint_shuffle_and_limits() {
        let mut g = Game::new(Layout::Turtle, 9);
        assert!(!g.available_pairs().is_empty());
        let p = g.hint().unwrap();
        assert!(g.assisted);
        assert!(g.match_pair(p[0], p[1]));
        assert!(g.undo());
        let p = g.available_pairs()[0];
        assert!(g.match_pair(p[0], p[1]));
        assert_eq!(g.history.len(), 1);
        let before = g.remaining();
        assert!(g.shuffle(88));
        assert_eq!(before, g.remaining());
        assert!(g.history.is_empty());
        assert!(g.assisted);
        assert!(!g.undo());
        assert!(!g.available_pairs().is_empty());
        g.advance_time(u64::MAX);
        assert!(g.elapsed_ms <= 7 * 24 * 60 * 60 * 1000);
        assert!(g.validate().is_ok());
    }
    #[test]
    fn seeded_restart_and_save_roundtrip_are_stable() {
        let first = Game::new(Layout::Monument, 0x1234_5678);
        let restarted = Game::new(first.layout, first.seed);
        assert_eq!(first, restarted);
        assert_eq!(
            solution_pairs(first.layout, first.seed),
            solution_pairs(first.layout, first.seed)
        );
        let encoded = serde_json::to_string(&first).unwrap();
        let decoded: Game = serde_json::from_str(&encoded).unwrap();
        assert_eq!(first, decoded);
    }
    #[test]
    fn deals_remain_diverse_after_deterministic_trace_fallback() {
        for layout in Layout::ALL {
            let fingerprints: std::collections::HashSet<Vec<u8>> = (0..33u64)
                .map(|seed| {
                    Game::new(layout, seed)
                        .tiles
                        .into_iter()
                        .map(|tile| tile.face)
                        .collect()
                })
                .collect();
            assert_eq!(fingerprints.len(), 33, "layout {}", layout.label());
        }
    }
    #[test]
    fn fixed_seed_face_fingerprints_are_portable() {
        let actual: Vec<u64> = Layout::ALL
            .into_iter()
            .map(|layout| {
                Game::new(layout, 0x004d_4148_4a4f_4e47)
                    .tiles
                    .iter()
                    .fold(1_469_598_103_934_665_603u64, |hash, tile| {
                        (hash ^ u64::from(tile.face)).wrapping_mul(1_099_511_628_211)
                    })
            })
            .collect();
        assert_eq!(
            actual,
            [
                3_053_092_321_328_382_435,
                12_418_315_544_204_051_777,
                12_443_921_217_389_399_115,
            ]
        );
    }
    fn find_zen_shuffle_trigger() -> (Game, [usize; 2]) {
        for seed in 0..256u64 {
            let mut game = Game::new(Layout::Turtle, seed);
            let mut choice_rng = seed ^ 0x5a45_4e5f_5452_4947;
            for _ in 0..72 {
                let pairs = game.available_pairs();
                for pair in pairs.iter().copied() {
                    let mut after = game.clone();
                    after.tiles[pair[0]].removed = true;
                    after.tiles[pair[1]].removed = true;
                    if after.stuck() && !game.history.is_empty() {
                        return (game, pair);
                    }
                }
                if pairs.is_empty() {
                    break;
                }
                let index = (next(&mut choice_rng) % pairs.len() as u64) as usize;
                let [a, b] = pairs[index];
                assert!(game.match_pair(a, b));
            }
        }
        panic!("bounded deal search found no legal move that leaves a stuck board");
    }
    fn find_five_auto_shuffle_state() -> (Game, Game, usize) {
        for seed in 0..512u64 {
            for layout in Layout::ALL {
                let mut game = Game::new(layout, seed);
                game.zen = true;
                let mut choice_rng = seed ^ 0x4155_544f_5f53_4855;
                for _ in 0..72 {
                    let pairs = game.available_pairs();
                    if pairs.is_empty() {
                        break;
                    }
                    let mut chosen = None;
                    for pair in pairs.iter().copied() {
                        let mut candidate = game.clone();
                        candidate.match_pair(pair[0], pair[1]);
                        if candidate.shuffles > game.shuffles {
                            chosen = Some(pair);
                            break;
                        }
                    }
                    let pair = chosen.unwrap_or_else(|| {
                        let index = (next(&mut choice_rng) % pairs.len() as u64) as usize;
                        pairs[index]
                    });
                    let before = game.clone();
                    assert!(game.match_pair(pair[0], pair[1]));
                    if game.shuffles >= 5 {
                        let dropped_prefix = before
                            .zen_undo_snapshots
                            .first()
                            .expect("fifth auto-shuffle has four older snapshots")
                            .history_index
                            + 1;
                        return (before, game, dropped_prefix);
                    }
                }
            }
        }
        panic!("bounded deal search found no five-auto-shuffle legal-play sequence");
    }
    #[test]
    fn zen_auto_shuffle_undo_restores_exact_board_across_reload() {
        let (mut game, pair) = find_zen_shuffle_trigger();
        let expected_tiles = game.tiles.clone();
        let expected_history = game.history.clone();
        game.zen = true;
        assert!(game.validate().is_ok());
        assert!(game.match_pair(pair[0], pair[1]));
        assert_eq!(game.shuffles, 1);
        assert!(!game.stuck());
        assert_eq!(game.zen_undo_snapshots.len(), 1);
        assert!(game.validate().is_ok());

        let saved = serde_json::to_string(&game).unwrap();
        let mut loaded: Game = serde_json::from_str(&saved).unwrap();
        assert!(loaded.zen);
        assert!(loaded.can_undo());
        assert!(loaded.undo());
        assert_eq!(loaded.tiles, expected_tiles);
        assert_eq!(loaded.history, expected_history);
        assert_eq!(loaded.shuffles, 1);
        assert!(loaded.validate().is_ok());
    }
    #[test]
    fn zen_timer_classic_stuck_behavior_and_legacy_defaults() {
        let mut zen = Game::new(Layout::Arena, 81);
        zen.zen = true;
        zen.advance_time(60_000);
        assert_eq!(zen.elapsed_ms, 0);

        let mut classic = Game::new(Layout::Arena, 81);
        classic.advance_time(60_000);
        assert_eq!(classic.elapsed_ms, 60_000);
        assert!(!classic.zen);

        let (mut candidate, pair) = find_zen_shuffle_trigger();
        let previous_shuffles = candidate.shuffles;
        assert!(candidate.match_pair(pair[0], pair[1]));
        assert!(candidate.stuck());
        assert_eq!(candidate.shuffles, previous_shuffles);

        let mut legacy = serde_json::to_value(Game::new(Layout::Arena, 42)).unwrap();
        legacy.as_object_mut().unwrap().remove("zen");
        legacy.as_object_mut().unwrap().remove("zen_undo_snapshots");
        let legacy: Game = serde_json::from_value(legacy).unwrap();
        assert!(!legacy.zen);
        assert!(legacy.zen_undo_snapshots.is_empty());
        assert!(legacy.validate().is_ok());
    }
    #[test]
    fn zen_snapshot_metadata_rejects_corruption() {
        let (mut game, pair) = find_zen_shuffle_trigger();
        game.zen = true;
        assert!(game.match_pair(pair[0], pair[1]));
        assert!(game.validate().is_ok());

        let mut bad_shape = game.clone();
        bad_shape.zen_undo_snapshots[0].faces_before.pop();
        assert!(bad_shape.validate().is_err());

        let mut bad_index = game.clone();
        bad_index.zen_undo_snapshots[0].history_index = bad_index.history.len();
        assert!(bad_index.validate().is_err());

        let mut classic_with_zen_metadata = game;
        classic_with_zen_metadata.zen = false;
        assert!(classic_with_zen_metadata.validate().is_err());
    }
    #[test]
    fn zen_history_and_snapshot_limits_are_enforced() {
        let (mut game, pair) = find_zen_shuffle_trigger();
        game.zen = true;
        assert!(game.match_pair(pair[0], pair[1]));
        let snapshot = game.zen_undo_snapshots[0].clone();

        let mut too_many_snapshots = game.clone();
        too_many_snapshots
            .zen_undo_snapshots
            .extend(std::iter::repeat_n(snapshot.clone(), ZEN_SNAPSHOT_LIMIT));
        assert!(too_many_snapshots.validate().is_err());

        let mut unordered_snapshots = game.clone();
        unordered_snapshots
            .zen_undo_snapshots
            .push(snapshot.clone());
        assert!(unordered_snapshots.validate().is_err());

        let mut bad_snapshot_inventory = game.clone();
        bad_snapshot_inventory.zen_undo_snapshots[0].faces_before[0] = 41;
        assert!(bad_snapshot_inventory.validate().is_err());

        let mut earlier_removed_swap = game.clone();
        let snapshot = &mut earlier_removed_swap.zen_undo_snapshots[0];
        assert!(snapshot.history_index > 0);
        let earlier_removed = earlier_removed_swap.history[0][0];
        let [boundary_a, boundary_b] = earlier_removed_swap.history[snapshot.history_index];
        let other = (0..144)
            .find(|&index| {
                index != boundary_a
                    && index != boundary_b
                    && snapshot.faces_before[index] != snapshot.faces_before[earlier_removed]
            })
            .unwrap();
        snapshot.faces_before.swap(earlier_removed, other);
        assert!(earlier_removed_swap.validate().is_err());

        let mut too_much_history = game;
        too_much_history.history.resize(ZEN_HISTORY_LIMIT + 1, pair);
        assert!(too_much_history.validate().is_err());
    }
    #[test]
    fn fifth_auto_shuffle_truncates_undo_prefix_and_latest_undo_stays_exact() {
        let (before, after, dropped_prefix) = find_five_auto_shuffle_state();
        assert_eq!(after.zen_undo_snapshots.len(), ZEN_SNAPSHOT_LIMIT);
        assert!(dropped_prefix > 0);
        assert!(after.validate().is_ok());

        let mut loaded: Game =
            serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
        assert!(loaded.undo());
        assert_eq!(loaded.tiles, before.tiles);
        assert_eq!(loaded.history, before.history[dropped_prefix..]);
        assert_eq!(loaded.zen_undo_snapshots.len(), ZEN_SNAPSHOT_LIMIT - 1);
        assert!(loaded.validate().is_ok());
    }
    #[test]
    fn manual_shuffle_keeps_classic_semantics_in_zen_mode() {
        let mut game = Game::new(Layout::Monument, 9001);
        game.zen = true;
        let [a, b] = game.available_pairs()[0];
        assert!(game.match_pair(a, b));
        assert_eq!(game.history.len(), 1);
        assert!(game.shuffle(42));
        assert!(game.history.is_empty());
        assert!(game.zen_undo_snapshots.is_empty());
        assert!(game.assisted);
        assert!(!game.can_undo());
        assert!(game.validate().is_ok());
    }
    #[test]
    fn won_game_time_does_not_advance() {
        let mut game = Game::new(Layout::Arena, 17);
        for [a, b] in solution_pairs(game.layout, game.seed) {
            assert!(game.match_pair(a, b));
        }
        let elapsed = game.elapsed_ms;
        game.advance_time(5000);
        assert_eq!(game.elapsed_ms, elapsed);
    }
    #[test]
    fn rejects_malformed_persisted_state() {
        let game = Game::new(Layout::Arena, 123);
        let mut odd_removed = game.clone();
        odd_removed.tiles[0].removed = true;
        assert!(odd_removed.validate().is_err());
        let mut bad_face = game.clone();
        bad_face.tiles[0].face = 42;
        assert!(bad_face.validate().is_err());
        let mut bad_inventory = game.clone();
        bad_inventory.tiles[0].face = bad_inventory.tiles[1].face;
        assert!(bad_inventory.validate().is_err());
        let mut bad_geometry = game.clone();
        bad_geometry.tiles[0].x += 1;
        assert!(bad_geometry.validate().is_err());
        let mut bad_history = game;
        bad_history.history.push([0, usize::MAX]);
        assert!(bad_history.validate().is_err());
        let mut duplicate_history = Game::new(Layout::Arena, 321);
        let [a, b] = duplicate_history.available_pairs()[0];
        assert!(duplicate_history.match_pair(a, b));
        duplicate_history.history.push([a, b]);
        assert!(duplicate_history.validate().is_err());
    }
    #[test]
    fn family_pair_removals_and_post_shuffle_state_validate() {
        let mut game = Game::new(Layout::Turtle, 777);
        let flowers: Vec<usize> = game
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(i, tile)| (34..38).contains(&tile.face).then_some(i))
            .collect();
        let (first, second) = (flowers[0], flowers[1]);
        assert_ne!(game.tiles[first].face, game.tiles[second].face);
        game.tiles[first].removed = true;
        game.tiles[second].removed = true;
        game.history.push([first, second]);
        assert!(game.validate().is_ok());
        assert!(game.undo());
        assert!(game.validate().is_ok());
        assert!(game.shuffle(0x9876));
        assert!(game.validate().is_ok());
    }
}
