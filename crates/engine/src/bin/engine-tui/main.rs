//! Engine Test Harness TUI - Interactive engine testing interface
//!
//! Usage: cargo run --bin engine-tui -- <path-to-game.cgdsl>
//!
//! Structure: this file is the thin driver (setup, channel plumbing, render
//! loop, shutdown). Key handling lives in `keys.rs`; panels live in `ui/`.

mod keys;
mod trace;
mod ui;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use cgdsl_engine::{
    run_game_with, EngineError, GameData, Input, InputKind, InputSource, InputType, RunOptions,
    TraceEntry,
};
use crossbeam_channel::{bounded, Receiver, Sender};
use front_end::validation::parse_document;

use keys::handle_key;
use ui::{
    AppLayout, ControlsPanel, EngineStatus, GameStatePanel, InputPanel, PanelFocus, TraceLogPanel,
    TuiState,
};

/// Driver stack size: the `front_end` parser's recursion cost grows with the
/// number of flow components, and large games (e.g. Go Fish's 13-option asks)
/// exceed the OS default 1 MiB main-thread stack. The whole driver (parse +
/// TUI loop) runs on a dedicated thread with a generous stack. See
/// `docs/NEXT_STEPS.md` (parser stack scaling).
const DRIVER_STACK_BYTES: usize = 16 * 1024 * 1024;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let handle = std::thread::Builder::new()
        .stack_size(DRIVER_STACK_BYTES)
        .spawn(driver)?;
    handle
        .join()
        .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> {
            "engine-tui driver panicked".into()
        })?
}

fn driver() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let game_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("Usage: engine-tui <path-to-game.cgdsl>");

    let source = std::fs::read_to_string(&game_path)?;
    let game = parse_document(&source)?;
    let ir = game.to_lowered_graph();

    // Engine <-> UI plumbing: all engine-side events travel over channels.
    let (input_tx, input_rx): (Sender<Input>, Receiver<Input>) = bounded(1);
    let (input_type_tx, input_type_rx): (Sender<InputType>, Receiver<InputType>) = bounded(1);
    let (trace_tx, trace_rx): (Sender<TraceEntry>, Receiver<TraceEntry>) = bounded(100);
    let (state_tx, state_rx): (Sender<GameData>, Receiver<GameData>) = bounded(100);
    // Panics anywhere in the process (any thread) notify the UI via this
    // channel instead of the old AtomicBool + Mutex<String> pair.
    let (panic_tx, panic_rx): (Sender<String>, Receiver<String>) = bounded(1);
    // `run_game_with`'s `Result` lands here when the run completes.
    let (outcome_tx, outcome_rx): (Sender<Result<GameData, EngineError>>, Receiver<_>) = bounded(1);

    let trace_sender: Sender<TraceEntry> = trace_tx.clone();
    let trace_sender = Box::new(move |entry: TraceEntry| {
        let _ = trace_sender.send(entry);
    }) as Box<dyn Fn(TraceEntry) + Send>;

    let state_sender = Box::new(move |gd: &GameData| {
        let _ = state_tx.send(gd.clone());
    }) as Box<dyn Fn(&GameData) + Send>;

    let input_source = InputSource::Player(Box::new(move |it: cgdsl_engine::InputType| {
        let _ = input_type_tx.send(it);
        input_rx.recv().unwrap_or(Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })
    }));

    let engine_ir = ir;
    thread::spawn(move || {
        // Surface any panic (engine or otherwise) in the UI; `capture_panics`
        // below turns engine-internal panics into a recoverable error, and the
        // hook is the belt-and-suspenders for everything else.
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info: &std::panic::PanicHookInfo| {
            let _ = panic_tx.send(panic_info.to_string());
            prev_hook(panic_info);
        }));
        let result = run_game_with(
            engine_ir,
            GameData::new(),
            input_source,
            RunOptions::new()
                .with_event_sender(state_sender)
                .with_trace_sender(trace_sender)
                .capture_panics(true),
        );
        let _ = outcome_tx.send(result);
    });

    let mut tui_state = TuiState::new();
    tui_state.input_tx = Some(input_tx);

    let mut terminal = ratatui::try_init()?;

    loop {
        while let Ok(gd) = state_rx.try_recv() {
            tui_state.current_player_name = gd
                .get_current_player()
                .map(|p| p.name.clone())
                .unwrap_or_default();
            tui_state.current_state = Some(gd);
        }

        tui_state.detect_turn_change();

        while let Ok(entry) = trace_rx.try_recv() {
            tui_state.push_trace(entry);
        }

        while let Ok(it) = input_type_rx.try_recv() {
            match &it {
                InputType::ChooseCards { display, .. } => {
                    tui_state.choose_cursor = 0;
                    tui_state.choose_selected = vec![false; display.len()];
                }
                InputType::ChoosePlayer { candidates: _, .. } | InputType::Choice { .. } => {
                    tui_state.choose_cursor = 0;
                    tui_state.choose_selected = Vec::new();
                }
                _ => {}
            }
            tui_state.pending_input = Some(it);
            tui_state.waiting_for_input = true;
        }

        // Life-cycle status: a panic wins over the (possibly never-arriving)
        // run outcome; the outcome only updates a still-running status.
        if let Ok(msg) = panic_rx.try_recv() {
            tui_state.engine_status = EngineStatus::Panicked(msg);
        }
        if let Ok(result) = outcome_rx.try_recv() {
            if tui_state.engine_status == EngineStatus::Running {
                tui_state.engine_status = match result {
                    Ok(gd) => {
                        tui_state.current_state = Some(gd);
                        EngineStatus::Finished
                    }
                    Err(e) => EngineStatus::Errored(e.to_string()),
                };
            }
        }

        terminal.draw(|f| {
            let size = f.area();
            let layout = AppLayout::new(size);

            if let Some(ref gd) = tui_state.current_state {
                let game_state_focused = tui_state.focus == PanelFocus::GameState;
                let panel = GameStatePanel::new(tui_state.state_detail);
                let gh = panel.render(
                    f,
                    gd,
                    layout.game_state_area,
                    tui_state.game_state_scroll,
                    tui_state.game_state_auto_scroll,
                    game_state_focused,
                );
                tui_state.game_state_inner_height = gh;

                let player_names: Vec<String> = gd.players.iter().map(|p| p.name.clone()).collect();
                if tui_state.perspective_idx >= player_names.len() {
                    tui_state.perspective_idx = 0;
                }

                let input_panel = InputPanel::new(tui_state.perspective_idx, player_names);
                input_panel.render(
                    f,
                    tui_state.waiting_for_input,
                    tui_state.pending_input.as_ref(),
                    layout.input_area,
                    tui_state.choose_cursor,
                    &tui_state.choose_selected,
                    &tui_state.current_player_name,
                );
            }

            let trace_focused = tui_state.focus == PanelFocus::TraceLog;
            let trace_panel = TraceLogPanel::new(tui_state.trace_detail, tui_state.trace_raw);
            let th = trace_panel.render(
                f,
                &tui_state.trace_entries,
                layout.trace_log_area,
                tui_state.trace_scroll,
                tui_state.trace_auto_scroll,
                trace_focused,
            );
            tui_state.trace_inner_height = th;

            let controls_panel = ControlsPanel::new();
            controls_panel.render(f, layout.controls_area, &tui_state.engine_status);
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match crossterm::event::read() {
                Ok(crossterm::event::Event::Key(key)) => {
                    if handle_key(key.code, &mut tui_state) {
                        break;
                    }
                }
                Err(_) => {}
                _ => {}
            }
        }

        thread::sleep(Duration::from_millis(16));
    }

    ratatui::restore();

    // Report the final state on the plain terminal (the UI already showed it
    // live via the CONTROLS panel status line).
    match &tui_state.engine_status {
        EngineStatus::Panicked(msg) => {
            eprintln!();
            eprintln!("========================================");
            eprintln!("ENGINE PANIC:");
            eprintln!("{msg}");
            eprintln!("========================================");
        }
        EngineStatus::Errored(msg) => eprintln!("Engine error: {msg}"),
        EngineStatus::Finished => println!("Engine terminated."),
        EngineStatus::Running => {
            // User quit mid-run; the engine thread (blocked on input) is
            // dropped with the process.
        }
    }

    Ok(())
}
