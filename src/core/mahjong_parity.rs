use super::{Game, Layout, Tile};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Fixture {
    oracle: String,
    layouts: BTreeMap<String, Vec<Coordinate>>,
    match_matrix: Vec<Vec<bool>>,
    selectability_cases: Vec<SelectabilityCase>,
}

#[derive(Deserialize)]
struct SelectabilityCase {
    name: String,
    tiles: Vec<Coordinate>,
    expected: bool,
}

#[derive(Deserialize)]
struct Coordinate {
    x: i16,
    y: i16,
    z: i16,
}

const FIXTURE: &str = include_str!("../../tests/fixtures/mahjong-oracle.json");

#[test]
fn canonical_layouts_match_typescript_oracle() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("valid oracle fixture");
    assert!(fixture.oracle.ends_with("games/mahjong/engine.ts"));
    for layout in Layout::ALL {
        let key = layout.label().to_ascii_lowercase();
        let expected = fixture.layouts.get(&key).expect("oracle layout present");
        let actual: Vec<(i16, i16, i16)> = layout
            .coordinates()
            .into_iter()
            .map(|(x, y, z)| (x, y, z as i16))
            .collect();
        let expected: Vec<_> = expected.iter().map(|c| (c.x, c.y, c.z)).collect();
        assert_eq!(actual, expected, "layout {key}");
    }
}

#[test]
fn all_face_pairs_match_typescript_oracle() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("valid oracle fixture");
    assert_eq!(fixture.match_matrix.len(), 42);
    for (a, row) in fixture.match_matrix.iter().enumerate() {
        assert_eq!(row.len(), 42);
        for (b, &expected) in row.iter().enumerate() {
            assert_eq!(Game::matches(a as u8, b as u8), expected, "faces {a}, {b}");
        }
    }
}

#[test]
fn half_offset_blockers_match_typescript_oracle() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("valid oracle fixture");
    for case in fixture.selectability_cases {
        let tiles: Vec<Tile> = case
            .tiles
            .iter()
            .enumerate()
            .map(|(i, coordinate)| Tile {
                face: if i == 0 { 0 } else { 1 },
                x: coordinate.x,
                y: coordinate.y,
                z: coordinate.z as u8,
                removed: false,
            })
            .collect();
        assert_eq!(
            Game::selectable_from(&tiles, 0),
            case.expected,
            "case {}",
            case.name
        );
    }
}
