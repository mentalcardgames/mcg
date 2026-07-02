//! Engine Test Harness TUI - Interactive engine testing interface
//!
//! Usage: cargo run --bin engine-tui -- <path-to-game.cgdsl>

mod trace;
mod ui;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use cgdsl_engine::{run_game, GameData, Input, InputSource, InputType, TraceEntry};
use crossbeam_channel::{bounded, Receiver, Sender};
use front_end::validation::parse_document;

use ui::{AppLayout, ControlsPanel, GameStatePanel, InputPanel, TraceLogPanel, TuiState};

use std::sync::atomic::{AtomicBool, Ordering};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("Usage: engine-tui <path-to-game.cgdsl>");

    let source = std::fs::read_to_string(&game_path)?;
    let game = parse_document(&source)?;
    let ir = game.to_lowered_graph();

    let (input_tx, input_rx): (Sender<Input>, Receiver<Input>) = bounded(1);
    let (input_type_tx, input_type_rx): (Sender<InputType>, Receiver<InputType>) = bounded(1);
    let (trace_tx, trace_rx): (Sender<TraceEntry>, Receiver<TraceEntry>) = bounded(100);
    let (state_tx, state_rx): (Sender<GameData>, Receiver<GameData>) = bounded(100);

    let trace_sender: Sender<TraceEntry> = trace_tx.clone();
    let trace_sender = Some(Box::new(move |entry: TraceEntry| {
        let _ = trace_sender.send(entry);
    }) as Box<dyn Fn(TraceEntry) + Send>);

    let state_sender = Some(Box::new(move |gd: &GameData| {
        let _ = state_tx.send(gd.clone());
    }) as Box<dyn Fn(&GameData) + Send>);

    let input_rx = input_rx;
    let input_type_tx = input_type_tx;
    let input_source = InputSource::Player(Box::new(move |it: cgdsl_engine::InputType| {
        let _ = input_type_tx.send(it);
        input_rx.recv().unwrap_or(Input::Choice { idx: 0 })
    }));

    let engine_panic = std::sync::Arc::new(AtomicBool::new(false));
    let engine_panic_msg = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let engine_panic_clone = engine_panic.clone();
    let engine_panic_msg_clone = engine_panic_msg.clone();

    let engine_ir = ir;
    let engine_handle = thread::spawn(move || {
        let hook = Box::new(move |panic_info: &std::panic::PanicHookInfo| {
            *engine_panic_msg_clone.lock().unwrap() = panic_info.to_string();
            engine_panic_clone.store(true, Ordering::SeqCst);
        });
        std::panic::set_hook(hook);
        run_game(
            engine_ir,
            GameData::new(),
            input_source,
            state_sender,
            trace_sender,
        )
    });

    let mut tui_state = TuiState::new();
    tui_state.input_tx = Some(input_tx);

    let mut terminal = ratatui::init();

    loop {
        while let Ok(entry) = trace_rx.try_recv() {
            tui_state.push_trace(entry);
        }

        while let Ok(it) = input_type_rx.try_recv() {
            tui_state.pending_input = Some(it);
            tui_state.waiting_for_input = true;
        }

        let current_state = state_rx.try_recv().ok();

        terminal.draw(|f| {
            let size = f.area();
            let layout = AppLayout::new(size);

            if let Some(ref gd) = current_state {
                let panel = GameStatePanel::new(tui_state.state_detail);
                panel.render(f, gd, layout.game_state_area);

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
                );
            }

            let trace_panel = TraceLogPanel::new(tui_state.trace_detail);
            trace_panel.render(f, &tui_state.trace_entries, layout.trace_log_area);

            let controls_panel = ControlsPanel::new();
            controls_panel.render(f, layout.controls_area);
        })?;

        match crossterm::event::read() {
            Ok(crossterm::event::Event::Key(key)) => match key.code {
                crossterm::event::KeyCode::Char('q') => break,
                crossterm::event::KeyCode::F(10) => break,
                crossterm::event::KeyCode::Char('l') => {
                    tui_state.cycle_state_detail();
                }
                crossterm::event::KeyCode::Char('t') => {
                    tui_state.cycle_trace_detail();
                }
                crossterm::event::KeyCode::Char('p') => {
                    if let Some(ref gd) = current_state {
                        let player_count = gd.players.len();
                        if player_count > 0 {
                            tui_state.perspective_idx =
                                (tui_state.perspective_idx + 1) % player_count;
                        }
                    }
                }
                crossterm::event::KeyCode::Char(n) => {
                    if n == 'y' || n == 'Y' {
                        if tui_state.waiting_for_input {
                            if let Some(ref tx) = tui_state.input_tx {
                                let _ = tx.send(Input::OptionalAccept);
                                tui_state.waiting_for_input = false;
                                tui_state.pending_input = None;
                            }
                        }
                    } else if n == 'n' || n == 'N' {
                        if tui_state.waiting_for_input {
                            if let Some(ref tx) = tui_state.input_tx {
                                let _ = tx.send(Input::OptionalDecline);
                                tui_state.waiting_for_input = false;
                                tui_state.pending_input = None;
                            }
                        }
                    } else if let Some(digit) = n.to_digit(10) {
                        if digit >= 1 && digit <= 9 {
                            if tui_state.waiting_for_input {
                                let idx = digit as usize - 1;
                                if let Some(ref tx) = tui_state.input_tx {
                                    let _ = tx.send(Input::Choice { idx });
                                    tui_state.waiting_for_input = false;
                                    tui_state.pending_input = None;
                                }
                            }
                        }
                    }
                }
                crossterm::event::KeyCode::Enter => {
                    if tui_state.waiting_for_input {
                        if let Some(ref tx) = tui_state.input_tx {
                            let _ = tx.send(Input::Choice { idx: 0 });
                            tui_state.waiting_for_input = false;
                            tui_state.pending_input = None;
                        }
                    }
                }
                _ => {}
            },
            Err(_) => {}
            _ => {}
        }

        if engine_handle.is_finished() {
            break;
        }

        thread::sleep(Duration::from_millis(16));
    }

    if engine_panic.load(Ordering::SeqCst) {
        let _ = ratatui::restore();
        eprintln!();
        eprintln!("========================================");
        eprintln!("ENGINE PANIC:");
        eprintln!("{}", engine_panic_msg.lock().unwrap());
        eprintln!("========================================");
    } else {
        ratatui::restore();
        println!("Engine terminated.");
    }

    Ok(())
}
