//! Engine Test Harness TUI - Interactive engine testing interface
//!
//! Usage: cargo run --bin engine-tui -- <path-to-game.cgdsl>

mod trace;
mod ui;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use cgdsl_engine::{run_game, GameData, Input, InputKind, InputSource, InputType, TraceEntry};
use crossbeam_channel::{bounded, Receiver, Sender};
use front_end::validation::parse_document;

use ui::{
    AppLayout, ControlsPanel, GameStatePanel, InputPanel, PanelFocus, TraceLogPanel, TuiState,
};

use std::sync::atomic::{AtomicBool, Ordering};

fn adjust_focused_scroll(state: &mut TuiState, delta: i32) {
    match state.focus {
        PanelFocus::GameState => {
            if delta < 0 {
                state.game_state_scroll = state.game_state_scroll.saturating_sub((-delta) as u16);
            } else {
                state.game_state_scroll = state.game_state_scroll.saturating_add(delta as u16);
            }
            state.game_state_auto_scroll = false;
        }
        PanelFocus::TraceLog => {
            if delta < 0 {
                state.trace_scroll = state.trace_scroll.saturating_sub((-delta) as u16);
            } else {
                state.trace_scroll = state.trace_scroll.saturating_add(delta as u16);
            }
            state.trace_auto_scroll = false;
        }
    }
}

fn page_focused_scroll(state: &mut TuiState, up: bool) {
    match state.focus {
        PanelFocus::GameState => {
            if up {
                state.game_state_scroll = state
                    .game_state_scroll
                    .saturating_sub(state.game_state_inner_height);
            } else {
                state.game_state_scroll = state
                    .game_state_scroll
                    .saturating_add(state.game_state_inner_height);
            }
            state.game_state_auto_scroll = false;
        }
        PanelFocus::TraceLog => {
            if up {
                state.trace_scroll = state.trace_scroll.saturating_sub(state.trace_inner_height);
            } else {
                state.trace_scroll = state.trace_scroll.saturating_add(state.trace_inner_height);
            }
            state.trace_auto_scroll = false;
        }
    }
}

fn home_focused_scroll(state: &mut TuiState) {
    match state.focus {
        PanelFocus::GameState => {
            state.game_state_scroll = 0;
            state.game_state_auto_scroll = false;
        }
        PanelFocus::TraceLog => {
            state.trace_scroll = 0;
            state.trace_auto_scroll = false;
        }
    }
}

fn end_focused_scroll(state: &mut TuiState) {
    match state.focus {
        PanelFocus::GameState => {
            state.game_state_auto_scroll = true;
        }
        PanelFocus::TraceLog => {
            state.trace_auto_scroll = true;
        }
    }
}

fn is_current_player(state: &TuiState) -> bool {
    let perspective_name = state
        .current_state
        .as_ref()
        .and_then(|gd| gd.players.get(state.perspective_idx))
        .map(|p| p.name.as_str())
        .unwrap_or("");
    !state.waiting_for_input
        || state.current_player_name.is_empty()
        || perspective_name == state.current_player_name
}

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
        input_rx.recv().unwrap_or(Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })
    }));

    let engine_panic = std::sync::Arc::new(AtomicBool::new(false));
    let engine_panic_msg = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let engine_panic_clone = engine_panic.clone();
    let engine_panic_msg_clone = engine_panic_msg.clone();

    let engine_ir = ir;
    let engine_handle = thread::spawn(move || {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info: &std::panic::PanicHookInfo| {
            *engine_panic_msg_clone.lock().unwrap() = panic_info.to_string();
            engine_panic_clone.store(true, Ordering::SeqCst);
            prev_hook(panic_info);
        }));
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
                InputType::ChoosePlayer { candidates: _, .. } => {
                    tui_state.choose_cursor = 0;
                    tui_state.choose_selected = Vec::new();
                }
                _ => {}
            }
            tui_state.pending_input = Some(it);
            tui_state.waiting_for_input = true;
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
            let trace_panel = TraceLogPanel::new(tui_state.trace_detail);
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
            controls_panel.render(f, layout.controls_area);
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match crossterm::event::read() {
                Ok(crossterm::event::Event::Key(key)) => {
                    let is_choosing = tui_state.waiting_for_input
                        && matches!(
                            &tui_state.pending_input,
                            Some(InputType::ChooseCards { .. })
                                | Some(InputType::ChoosePlayer { .. })
                        );
                    match key.code {
                        crossterm::event::KeyCode::Char('q') => break,
                        crossterm::event::KeyCode::F(10) => break,
                        crossterm::event::KeyCode::Char('l') => {
                            tui_state.cycle_state_detail();
                        }
                        crossterm::event::KeyCode::Char('t') => {
                            tui_state.cycle_trace_detail();
                        }
                        crossterm::event::KeyCode::Char('p') => {
                            if let Some(ref gd) = tui_state.current_state {
                                let player_count = gd.players.len();
                                if player_count > 0 {
                                    tui_state.perspective_idx =
                                        (tui_state.perspective_idx + 1) % player_count;
                                }
                            }
                        }
                        crossterm::event::KeyCode::Tab => {
                            tui_state.focus = match tui_state.focus {
                                PanelFocus::GameState => PanelFocus::TraceLog,
                                PanelFocus::TraceLog => PanelFocus::GameState,
                            };
                        }
                        crossterm::event::KeyCode::Up => {
                            if is_choosing && is_current_player(&tui_state) {
                                if tui_state.choose_cursor > 0 {
                                    tui_state.choose_cursor -= 1;
                                }
                            } else {
                                adjust_focused_scroll(&mut tui_state, -1);
                            }
                        }
                        crossterm::event::KeyCode::Down => {
                            if is_choosing && is_current_player(&tui_state) {
                                let max = match &tui_state.pending_input {
                                    Some(InputType::ChooseCards { display, .. }) => display.len(),
                                    Some(InputType::ChoosePlayer { candidates, .. }) => {
                                        candidates.len()
                                    }
                                    _ => 1,
                                };
                                if tui_state.choose_cursor + 1 < max {
                                    tui_state.choose_cursor += 1;
                                }
                            } else {
                                adjust_focused_scroll(&mut tui_state, 1);
                            }
                        }
                        crossterm::event::KeyCode::PageUp => {
                            page_focused_scroll(&mut tui_state, true);
                        }
                        crossterm::event::KeyCode::PageDown => {
                            page_focused_scroll(&mut tui_state, false);
                        }
                        crossterm::event::KeyCode::Home => {
                            home_focused_scroll(&mut tui_state);
                        }
                        crossterm::event::KeyCode::End => {
                            end_focused_scroll(&mut tui_state);
                        }
                        crossterm::event::KeyCode::Char(' ') => {
                            if tui_state.waiting_for_input && is_current_player(&tui_state) {
                                if let Some(InputType::ChooseCards { .. }) =
                                    &tui_state.pending_input
                                {
                                    if tui_state.choose_cursor < tui_state.choose_selected.len() {
                                        tui_state.choose_selected[tui_state.choose_cursor] ^= true;
                                    }
                                }
                            }
                        }
                        crossterm::event::KeyCode::Char(n) => {
                            if tui_state.waiting_for_input && is_current_player(&tui_state) {
                                match &tui_state.pending_input {
                                    Some(InputType::ChooseCards { .. })
                                    | Some(InputType::ChoosePlayer { .. }) => {
                                        // ignored: use arrows/space/enter for these
                                    }
                                    _ => {
                                        let player_name = tui_state
                                            .current_state
                                            .as_ref()
                                            .and_then(|gd| {
                                                gd.players.get(tui_state.perspective_idx)
                                            })
                                            .map(|p| p.name.clone())
                                            .unwrap_or_else(|| {
                                                format!("Player{}", tui_state.perspective_idx)
                                            });
                                        if n == 'y' || n == 'Y' {
                                            if let Some(ref tx) = tui_state.input_tx {
                                                let _ = tx.send(Input {
                                                    player_id: player_name,
                                                    kind: InputKind::OptionalAccept,
                                                });
                                                tui_state.waiting_for_input = false;
                                                tui_state.pending_input = None;
                                            }
                                        } else if n == 'n' || n == 'N' {
                                            if let Some(ref tx) = tui_state.input_tx {
                                                let _ = tx.send(Input {
                                                    player_id: player_name,
                                                    kind: InputKind::OptionalDecline,
                                                });
                                                tui_state.waiting_for_input = false;
                                                tui_state.pending_input = None;
                                            }
                                        } else if let Some(digit) = n.to_digit(10) {
                                            if (1..=9).contains(&digit) {
                                                let idx = digit as usize - 1;
                                                if let Some(ref tx) = tui_state.input_tx {
                                                    let _ = tx.send(Input {
                                                        player_id: player_name,
                                                        kind: InputKind::Choice { idx },
                                                    });
                                                    tui_state.waiting_for_input = false;
                                                    tui_state.pending_input = None;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        crossterm::event::KeyCode::Enter => {
                            if tui_state.waiting_for_input && is_current_player(&tui_state) {
                                let player_name = tui_state
                                    .current_state
                                    .as_ref()
                                    .and_then(|gd| gd.players.get(tui_state.perspective_idx))
                                    .map(|p| p.name.clone())
                                    .unwrap_or_else(|| {
                                        format!("Player{}", tui_state.perspective_idx)
                                    });
                                match &tui_state.pending_input {
                                    Some(InputType::ChooseCards { min, max, .. }) => {
                                        let selected: Vec<usize> = tui_state
                                            .choose_selected
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, &s)| s)
                                            .map(|(i, _)| i)
                                            .collect();
                                        if selected.len() >= *min && selected.len() <= *max {
                                            if let Some(ref tx) = tui_state.input_tx {
                                                let _ = tx.send(Input {
                                                    player_id: player_name,
                                                    kind: InputKind::ChooseCards { selected },
                                                });
                                                tui_state.waiting_for_input = false;
                                                tui_state.pending_input = None;
                                            }
                                        }
                                    }
                                    Some(InputType::ChoosePlayer { .. }) => {
                                        if let Some(ref tx) = tui_state.input_tx {
                                            let _ = tx.send(Input {
                                                player_id: player_name,
                                                kind: InputKind::ChoosePlayer {
                                                    idx: tui_state.choose_cursor,
                                                },
                                            });
                                            tui_state.waiting_for_input = false;
                                            tui_state.pending_input = None;
                                        }
                                    }
                                    _ => {
                                        if let Some(ref tx) = tui_state.input_tx {
                                            let _ = tx.send(Input {
                                                player_id: player_name,
                                                kind: InputKind::Choice { idx: 0 },
                                            });
                                            tui_state.waiting_for_input = false;
                                            tui_state.pending_input = None;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {}
                _ => {}
            }
        }

        thread::sleep(Duration::from_millis(16));
    }

    ratatui::restore();

    if engine_panic.load(Ordering::SeqCst) {
        eprintln!();
        eprintln!("========================================");
        eprintln!("ENGINE PANIC:");
        eprintln!("{}", engine_panic_msg.lock().unwrap());
        eprintln!("========================================");
    } else {
        match engine_handle.join() {
            Ok(Ok(_)) => println!("Engine terminated."),
            Ok(Err(e)) => eprintln!("Engine error: {e}"),
            Err(_) => {} // thread already reported via engine_panic
        }
    }

    Ok(())
}
