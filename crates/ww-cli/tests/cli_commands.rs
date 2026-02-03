#![allow(deprecated)] // Command::cargo_bin – macro replacement not yet stable

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Create a temp directory with a complete test world.
fn test_world() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("world.ww"),
        r#"world "Test World" {
    genre "fantasy"
    setting "A test world for integration tests"
}

the Iron Citadel is a fortress {
    climate arid
    population 5000

    """
    An ancient fortress carved from iron ore.
    """
}
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("characters.ww"),
        r#"Kael Stormborn is a character {
    species human
    occupation knight
    status alive
    traits [brave, loyal]
    located at the Iron Citadel
    member of the Order of Dawn

    """
    A brave knight sworn to protect the realm.
    """
}
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("factions.ww"),
        r#"the Order of Dawn is a faction {
    type military_order
    values [honor, duty]
    based at the Iron Citadel
}
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("events.ww"),
        r#"the Great Sundering is an event {
    date year -1247, month 3, day 15
    type cataclysm

    """
    The day the world broke.
    """
}
"#,
    )
    .unwrap();
    dir
}

fn ww() -> Command {
    Command::cargo_bin("ww").unwrap()
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

#[test]
fn init_creates_world_directory() {
    let parent = TempDir::new().unwrap();
    ww().args(["init", "myworld"])
        .current_dir(parent.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created world 'myworld'"));

    assert!(parent.path().join("myworld/world.ww").exists());
}

#[test]
fn init_fails_if_dir_exists() {
    let parent = TempDir::new().unwrap();
    fs::create_dir(parent.path().join("myworld")).unwrap();

    ww().args(["init", "myworld"])
        .current_dir(parent.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

// ---------------------------------------------------------------------------
// build
// ---------------------------------------------------------------------------

#[test]
fn build_succeeds_with_valid_world() {
    let dir = test_world();
    ww().args(["build", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Test World")
                .and(predicate::str::contains("entities"))
                .and(predicate::str::contains("relationships")),
        );
}

#[test]
fn build_fails_with_invalid_syntax() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("bad.ww"), "this is not valid { { {").unwrap();

    ww().args(["build", "-d", dir.path().to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn build_empty_dir() {
    let dir = TempDir::new().unwrap();
    ww().args(["build", "-d", dir.path().to_str().unwrap()])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// check
// ---------------------------------------------------------------------------

#[test]
fn check_passes_valid_world() {
    let dir = test_world();
    ww().args(["check", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("All checks passed"));
}

#[test]
fn check_fails_invalid_world() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("bad.ww"), "this is not valid { { {").unwrap();

    ww().args(["check", "-d", dir.path().to_str().unwrap()])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

#[test]
fn list_shows_all_entities() {
    let dir = test_world();
    ww().args(["list", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Kael Stormborn")
                .and(predicate::str::contains("Iron Citadel"))
                .and(predicate::str::contains("Order of Dawn"))
                .and(predicate::str::contains("Great Sundering")),
        );
}

#[test]
fn list_filters_by_kind() {
    let dir = test_world();
    ww().args(["list", "character", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Kael Stormborn")
                .and(predicate::str::contains("Iron Citadel").not()),
        );
}

#[test]
fn list_no_matches() {
    let dir = test_world();
    ww().args(["list", "lore", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No entities found"));
}

// ---------------------------------------------------------------------------
// show
// ---------------------------------------------------------------------------

#[test]
fn show_displays_entity() {
    let dir = test_world();
    ww().args(["show", "Kael Stormborn", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Kael Stormborn")
                .and(predicate::str::contains("character"))
                .and(predicate::str::contains("human"))
                .and(predicate::str::contains("knight")),
        );
}

#[test]
fn show_with_relationships() {
    let dir = test_world();
    ww().args([
        "show",
        "Kael Stormborn",
        "-r",
        "-d",
        dir.path().to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(
        predicate::str::contains("Kael Stormborn")
            .and(predicate::str::contains("Iron Citadel"))
            .and(predicate::str::contains("Order of Dawn")),
    );
}

#[test]
fn show_fails_unknown_entity() {
    let dir = test_world();
    ww().args(["show", "Nobody", "-d", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("entity not found"));
}

// ---------------------------------------------------------------------------
// search
// ---------------------------------------------------------------------------

#[test]
fn search_finds_matching_entities() {
    let dir = test_world();
    ww().args(["search", "knight", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Kael Stormborn"));
}

#[test]
fn search_no_results() {
    let dir = test_world();
    ww().args(["search", "zzzznothing", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results"));
}

// ---------------------------------------------------------------------------
// graph
// ---------------------------------------------------------------------------

#[test]
fn graph_shows_relationships() {
    let dir = test_world();
    ww().args(["graph", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Relationship graph"));
}

#[test]
fn graph_focused_entity() {
    let dir = test_world();
    ww().args([
        "graph",
        "--focus",
        "Kael Stormborn",
        "-d",
        dir.path().to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(
        predicate::str::contains("Kael Stormborn").and(predicate::str::contains("Iron Citadel")),
    );
}

#[test]
fn graph_focused_unknown_entity() {
    let dir = test_world();
    ww().args([
        "graph",
        "--focus",
        "Nobody",
        "-d",
        dir.path().to_str().unwrap(),
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("entity not found"));
}

// ---------------------------------------------------------------------------
// timeline
// ---------------------------------------------------------------------------

#[test]
fn timeline_shows_events() {
    let dir = test_world();
    ww().args(["timeline", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Great Sundering").and(predicate::str::contains("-1247")));
}

#[test]
fn timeline_no_events_in_range() {
    let dir = test_world();
    ww().args([
        "timeline",
        "--from",
        "9000",
        "--to",
        "9999",
        "-d",
        dir.path().to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("No events"));
}

// ---------------------------------------------------------------------------
// export
// ---------------------------------------------------------------------------

#[test]
fn export_json_valid_output() {
    let dir = test_world();
    let output = ww()
        .args(["export", "json", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");
    assert_eq!(json["world"]["name"], "Test World");
    assert!(json["entities"].as_array().unwrap().len() >= 4);
}

#[test]
fn export_markdown() {
    let dir = test_world();
    ww().args(["export", "markdown", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("# Test World")
                .and(predicate::str::contains("## Characters"))
                .and(predicate::str::contains("### Kael Stormborn")),
        );
}

#[test]
fn export_html() {
    let dir = test_world();
    ww().args(["export", "html", "-d", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("<!DOCTYPE html")
                .and(predicate::str::contains("<title>Test World</title>")),
        );
}

#[test]
fn export_to_file() {
    let dir = test_world();
    let out_file = dir.path().join("export.json");
    ww().args([
        "export",
        "json",
        "-o",
        out_file.to_str().unwrap(),
        "-d",
        dir.path().to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Exported to"));

    let content = fs::read_to_string(&out_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON in file");
    assert_eq!(json["world"]["name"], "Test World");
}

#[test]
fn export_unsupported_format() {
    let dir = test_world();
    ww().args(["export", "xml", "-d", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported format"));
}

// ---------------------------------------------------------------------------
// new
// ---------------------------------------------------------------------------

#[test]
fn new_creates_entity_file() {
    let dir = TempDir::new().unwrap();
    ww().args(["new", "character", "Elara Nightwhisper"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Added character"));

    let content = fs::read_to_string(dir.path().join("characters.ww")).unwrap();
    assert!(content.contains("Elara Nightwhisper is a character"));
    assert!(content.contains("species"));
}

#[test]
fn new_appends_to_existing_file() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("characters.ww"),
        "Kael is a character { species human }\n",
    )
    .unwrap();

    ww().args(["new", "character", "Elara Nightwhisper"])
        .current_dir(dir.path())
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("characters.ww")).unwrap();
    assert!(content.contains("Kael is a character"));
    assert!(content.contains("Elara Nightwhisper is a character"));
}

#[test]
fn new_custom_file() {
    let dir = TempDir::new().unwrap();
    ww().args(["new", "location", "The Ashlands", "-f", "places.ww"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("places.ww"));

    assert!(dir.path().join("places.ww").exists());
    let content = fs::read_to_string(dir.path().join("places.ww")).unwrap();
    assert!(content.contains("The Ashlands is a location"));
}
