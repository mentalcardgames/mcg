//! Fixture-hygiene guard: every `.cgdsl` in `test_games/` must be referenced
//! by at least one test (via `load_game("...")` in `tests/` or
//! `test_games/...` path joins in `src/`) or be allow-listed as a
//! manual/TUI-only fixture, and every referenced fixture must exist.
//! Prevents dead-fixture drift (e.g. the former `probe_*` files).

use std::collections::HashSet;
use std::path::PathBuf;

/// Fixtures used only for interactive play (TUI / `cgdsl-play`), never by tests.
const TUI_ONLY: &[&str] = &["location_resolution.cgdsl", "test.cgdsl"];

#[test]
fn every_fixture_is_referenced_or_allow_listed() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = manifest.join("test_games");

    let mut referenced: HashSet<String> = HashSet::new();
    for dir in [manifest.join("tests"), manifest.join("src")] {
        for entry in walk_rs_files(&dir) {
            let src = std::fs::read_to_string(&entry).expect("source file readable");
            let mut remaining = src.as_str();
            while let Some(start) = remaining.find("load_game(\"") {
                let after = &remaining[start + "load_game(\"".len()..];
                let end = after.find('"').expect("closing quote after load_game(\"");
                referenced.insert(after[..end].to_string());
                remaining = &after[end..];
            }
            let mut remaining = src.as_str();
            while let Some(start) = remaining.find("test_games/") {
                let after = &remaining[start + "test_games/".len()..];
                let end = after.find('"').expect("closing quote after test_games/");
                let name = &after[..end];
                if name.ends_with(".cgdsl") {
                    referenced.insert(name.to_string());
                }
                remaining = &after[end..];
            }
        }
    }

    let mut orphans: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&fixtures_dir).expect("test_games/ readable") {
        let path = entry.expect("test_games/ entry").path();
        if matches!(path.extension().and_then(|e| e.to_str()), Some("cgdsl")) {
            let name = path
                .file_name()
                .expect("fixture file name")
                .to_string_lossy()
                .into_owned();
            if !referenced.contains(&name) && !TUI_ONLY.contains(&name.as_str()) {
                orphans.push(name);
            }
        }
    }
    assert!(
        orphans.is_empty(),
        "orphaned fixtures (referenced by no test; delete or allow-list): {orphans:?}"
    );

    for name in &referenced {
        assert!(
            fixtures_dir.join(name).is_file(),
            "test references missing fixture: {name}"
        );
    }
}

/// Recursively list `.rs` files under `dir`, skipping this guard test itself
/// (its allow-list constants would otherwise count as references).
fn walk_rs_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).expect("dir readable") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                out.extend(walk_rs_files(&path));
            } else if matches!(path.extension().and_then(|e| e.to_str()), Some("rs"))
                && path.file_name().and_then(|n| n.to_str()) != Some("hygiene_test.rs")
            {
                out.push(path);
            }
        }
    }
    out
}
