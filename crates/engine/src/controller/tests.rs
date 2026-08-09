use super::*;
use crate::game_data::GameData;
use crate::interpreter::{Input, InputKind, InputType, Interpreter, TraceEntry};
use front_end::ir::{Ir, LoweredPayLoad};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

/// Verify that [`read_test_file`] correctly parses `y`, `n`, and numeric
/// lines in FIFO order into the corresponding [`Input`] variants.
#[test]
fn test_input_parsing() {
    let default_ir = Ir::<LoweredPayLoad>::default();
    let interpreter = Interpreter::new(
        Ir {
            states: std::collections::HashMap::new(),
            entry: default_ir.entry,
            goal: default_ir.goal,
        },
        GameData::new(),
        None,
    );
    let mut controller = Controller {
        interpreter,
        input_source: InputSource::TestFile(PathBuf::from("/nonexistent")),
        event_sender: None,
        line_buffer: VecDeque::from([
            "1".to_string(),
            "y".to_string(),
            "2".to_string(),
            "n".to_string(),
            "p 2".to_string(),
            "c 1,3".to_string(),
        ]),
        file_loaded: true,
        input_sequence: 0,
        step_count: Arc::new(std::sync::Mutex::new(0)),
    };

    let path = PathBuf::from("/nonexistent");
    assert_eq!(
        controller.read_test_file(&path).unwrap(),
        Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 }
        }
    );
    assert_eq!(
        controller.read_test_file(&path).unwrap(),
        Input {
            player_id: "P1".into(),
            kind: InputKind::OptionalAccept
        }
    );
    assert_eq!(
        controller.read_test_file(&path).unwrap(),
        Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 1 }
        }
    );
    assert_eq!(
        controller.read_test_file(&path).unwrap(),
        Input {
            player_id: "P1".into(),
            kind: InputKind::OptionalDecline
        }
    );
    assert_eq!(
        controller.read_test_file(&path).unwrap(),
        Input {
            player_id: "P1".into(),
            kind: InputKind::ChoosePlayer { idx: 1 }
        },
        "p 2 -> ChoosePlayer idx 1"
    );
    assert_eq!(
        controller.read_test_file(&path).unwrap(),
        Input {
            player_id: "P1".into(),
            kind: InputKind::ChooseCards {
                selected: vec![0, 2]
            }
        },
        "c 1,3 -> ChooseCards selected [0,2]"
    );
}

/// Ensure that popping from an empty `line_buffer` produces the expected
/// `"Test input file exhausted"` error.
#[test]
fn test_input_exhausted_error() {
    let default_ir = Ir::<LoweredPayLoad>::default();
    let interpreter = Interpreter::new(
        Ir {
            states: std::collections::HashMap::new(),
            entry: default_ir.entry,
            goal: default_ir.goal,
        },
        GameData::new(),
        None,
    );
    let mut controller = Controller {
        interpreter,
        input_source: InputSource::TestFile(PathBuf::from("test_input.txt")),
        event_sender: None,
        line_buffer: VecDeque::from(["1".to_string()]),
        file_loaded: true,
        input_sequence: 0,
        step_count: Arc::new(std::sync::Mutex::new(0)),
    };

    let path = PathBuf::from("test_input.txt");
    assert!(controller.read_test_file(&path).is_ok());
    let result = controller.read_test_file(&path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Test input file exhausted (input #0)");
}

/// Verify that [`read_test_file`] correctly parses a `Name:` prefix on
/// each line (e.g. `P2:y`, `P3:c 1,3`), setting the right `player_id`.
#[test]
fn test_input_parsing_name_prefix() {
    let default_ir = Ir::<LoweredPayLoad>::default();
    let interpreter = Interpreter::new(
        Ir {
            states: std::collections::HashMap::new(),
            entry: default_ir.entry,
            goal: default_ir.goal,
        },
        GameData::new(),
        None,
    );
    let mut controller = Controller {
        interpreter,
        input_source: InputSource::TestFile(PathBuf::from("/nonexistent")),
        event_sender: None,
        line_buffer: VecDeque::from(["P2:y".to_string(), "P3:c 1,3".to_string()]),
        file_loaded: true,
        input_sequence: 0,
        step_count: Arc::new(std::sync::Mutex::new(0)),
    };
    let path = PathBuf::from("/nonexistent");
    assert_eq!(
        controller.read_test_file(&path).unwrap(),
        Input {
            player_id: "P2".into(),
            kind: InputKind::OptionalAccept,
        }
    );
    assert_eq!(
        controller.read_test_file(&path).unwrap(),
        Input {
            player_id: "P3".into(),
            kind: InputKind::ChooseCards {
                selected: vec![0, 2],
            },
        }
    );
}

#[test]
fn test_input_file_ordering_and_validation() {
    use front_end::validation::parse_document;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_path = manifest_dir.join("test_games/ordering_test.cgdsl");
    let input_path = manifest_dir.join("test_games/ordering_test.txt");

    let source = std::fs::read_to_string(&game_path).expect("Failed to read test game file");
    let game = parse_document(&source).expect("Failed to parse game");
    let ir = game.to_lowered_graph();

    let game_data = GameData::new();
    let result = run_game(ir, game_data, InputSource::TestFile(input_path), None, None);

    assert!(result.is_ok(), "Game should complete successfully");
}

/// Shared setup for the two `ordering_test` debug-integration tests: load
/// `test_games/ordering_test.cgdsl`, run the game with a `TestFile` input
/// source, and capture every emitted `GameData` snapshot. Returns the
/// snapshots `Arc` and the run result so each test keeps its own unique
/// assertions. See Stage 6 / sub-task B5.
fn run_ordering_game_snapshots() -> (
    std::sync::Arc<std::sync::RwLock<Vec<GameData>>>,
    Result<GameData, String>,
) {
    use front_end::validation::parse_document;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_path = manifest_dir.join("test_games/ordering_test.cgdsl");
    let input_path = manifest_dir.join("test_games/ordering_test.txt");

    let source = std::fs::read_to_string(&game_path).expect("Failed to read test game file");
    let game = parse_document(&source).expect("Failed to parse game");
    let ir = game.to_lowered_graph();

    let snapshots = std::sync::Arc::new(std::sync::RwLock::new(Vec::new()));
    let snapshots_clone = snapshots.clone();
    let game_data = GameData::new();

    let result = run_game(
        ir,
        game_data,
        InputSource::TestFile(input_path),
        Some(Box::new(move |gd| {
            snapshots_clone.write().unwrap().push(gd.clone());
        })),
        None,
    );

    (snapshots, result)
}

#[test]
fn test_debug_integration_game_snapshots() {
    use crate::debug::{format_game_data, DebugLevel};

    let (snapshots, result) = run_ordering_game_snapshots();

    assert!(result.is_ok());
    assert!(!snapshots.read().unwrap().is_empty());

    let output_low = format_game_data(&snapshots.read().unwrap()[0], DebugLevel::Low);
    assert!(!output_low.is_empty());
    assert!(output_low.contains("GAME DATA (LOW)"));

    let output_medium = format_game_data(&snapshots.read().unwrap()[0], DebugLevel::Medium);
    assert!(!output_medium.is_empty());
    assert!(output_medium.contains("GAME DATA (MEDIUM)"));

    let output_high = format_game_data(&snapshots.read().unwrap()[0], DebugLevel::High);
    assert!(!output_high.is_empty());
    assert!(output_high.contains("GAME DATA (HIGH)"));
}

#[test]
fn test_debug_integration_verify_game_progression() {
    use crate::debug::{format_game_data, DebugLevel};

    let (snapshots, result) = run_ordering_game_snapshots();

    assert!(result.is_ok());

    let first_snapshot = &snapshots.read().unwrap()[0];
    let first_output = format_game_data(first_snapshot, DebugLevel::Low);

    assert!(first_output.contains("Players:"));

    if snapshots.read().unwrap().len() > 1 {
        let second_snapshot = &snapshots.read().unwrap()[1];
        let second_output = format_game_data(second_snapshot, DebugLevel::Low);

        assert!(
            second_output.contains("Players:"),
            "All snapshots should contain player info"
        );

        assert!(
            first_output != second_output,
            "Snapshots should show different game states"
        );
    }
}

#[test]
fn test_play_stage_advances_turn_and_runs_two_iterations() {
    use crate::interpreter::TraceEvent;
    use front_end::validation::parse_document;
    use std::sync::{Arc, Mutex};

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_path = manifest_dir.join("test_games/turn_switch.cgdsl");

    let source = std::fs::read_to_string(&game_path).expect("read turn_switch.cgdsl");
    let game = parse_document(&source).expect("parse turn_switch.cgdsl");
    let ir = game.to_lowered_graph();

    let snapshots: Arc<Mutex<Vec<GameData>>> = Arc::new(Mutex::new(Vec::new()));
    let snapshots_clone = snapshots.clone();
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();

    let result = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_input_type: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        Some(Box::new(move |gd: &GameData| {
            snapshots_clone.lock().unwrap().push(gd.clone());
        })),
        Some(Box::new(move |entry: TraceEntry| {
            trace_clone.lock().unwrap().push(entry);
        })),
    );

    assert!(result.is_ok(), "game should complete: {:?}", result.err());

    let trace_vec = trace.lock().unwrap().clone();
    let play_rounds = trace_vec
        .iter()
        .filter(|e| {
            if let TraceEntry::Step {
                event: TraceEvent::StageRoundCounter { stage, .. },
                ..
            } = *e
            {
                stage.as_str() == "Play"
            } else {
                false
            }
        })
        .count();
    assert_eq!(
        play_rounds, 2,
        "Play must run 2 iterations (one StageRoundCounter traversal each); got {}",
        play_rounds
    );

    let reached_p2 = snapshots
        .lock()
        .unwrap()
        .iter()
        .any(|gd: &GameData| gd.current_player == Some(1));
    assert!(
        reached_p2,
        "current_player must reach P2 (Some(1)) during Play — enter_stage must fire before the first cycle-to-next"
    );
}
