//! Keyboard handling for the TUI.
//!
//! Everything that reacts to a key press lives here, so `main.rs` stays a thin
//! loop: poll channels, render, delegate keys.

use crossterm::event::KeyCode;

use crate::ui::{PanelFocus, TuiState};
use cgdsl_engine::{Input, InputKind, InputType};

pub(super) fn adjust_focused_scroll(state: &mut TuiState, delta: i32) {
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

pub(super) fn page_focused_scroll(state: &mut TuiState, up: bool) {
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

pub(super) fn home_focused_scroll(state: &mut TuiState) {
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

pub(super) fn end_focused_scroll(state: &mut TuiState) {
    match state.focus {
        PanelFocus::GameState => {
            state.game_state_auto_scroll = true;
        }
        PanelFocus::TraceLog => {
            state.trace_auto_scroll = true;
        }
    }
}

pub(super) fn is_current_player(state: &TuiState) -> bool {
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

/// Map a digit-key shortcut to a `Choice` option index.
/// `1`..=`9` select options 1..=9; `0` selects option 10 (for long lists
/// such as Go Fish's 13-rank ask). Returns `None` when the option does not
/// exist (the digit is then simply ignored).
fn choice_shortcut_idx(digit: u32, max_index: usize) -> Option<usize> {
    let idx = if digit == 0 { 9 } else { digit as usize - 1 };
    (idx <= max_index).then_some(idx)
}

/// Handle one key event. Returns `true` when the user asked to quit.
pub(super) fn handle_key(code: KeyCode, state: &mut TuiState) -> bool {
    let is_choosing = state.waiting_for_input
        && matches!(
            &state.pending_input,
            Some(InputType::ChooseCards { .. })
                | Some(InputType::ChoosePlayer { .. })
                | Some(InputType::Choice { .. })
        );
    match code {
        KeyCode::Char('q') => return true,
        KeyCode::F(10) => return true,
        KeyCode::Char('l') => {
            state.cycle_state_detail();
        }
        KeyCode::Char('t') => {
            state.cycle_trace_detail();
        }
        KeyCode::Char('r') => {
            state.toggle_trace_raw();
        }
        KeyCode::Char('p') => {
            if let Some(ref gd) = state.current_state {
                let player_count = gd.players.len();
                if player_count > 0 {
                    state.perspective_idx = (state.perspective_idx + 1) % player_count;
                }
            }
        }
        KeyCode::Tab => {
            state.focus = match state.focus {
                PanelFocus::GameState => PanelFocus::TraceLog,
                PanelFocus::TraceLog => PanelFocus::GameState,
            };
        }
        KeyCode::Up => {
            if is_choosing && is_current_player(state) {
                if state.choose_cursor > 0 {
                    state.choose_cursor -= 1;
                }
            } else {
                adjust_focused_scroll(state, -1);
            }
        }
        KeyCode::Down => {
            if is_choosing && is_current_player(state) {
                let max = match &state.pending_input {
                    Some(InputType::ChooseCards { display, .. }) => display.len(),
                    Some(InputType::ChoosePlayer { candidates, .. }) => candidates.len(),
                    Some(InputType::Choice { options, .. }) => options.len(),
                    _ => 1,
                };
                if state.choose_cursor + 1 < max {
                    state.choose_cursor += 1;
                }
            } else {
                adjust_focused_scroll(state, 1);
            }
        }
        KeyCode::PageUp => {
            page_focused_scroll(state, true);
        }
        KeyCode::PageDown => {
            page_focused_scroll(state, false);
        }
        KeyCode::Home => {
            home_focused_scroll(state);
        }
        KeyCode::End => {
            end_focused_scroll(state);
        }
        KeyCode::Char(' ') => {
            if state.waiting_for_input && is_current_player(state) {
                if let Some(InputType::ChooseCards { .. }) = &state.pending_input {
                    if state.choose_cursor < state.choose_selected.len() {
                        state.choose_selected[state.choose_cursor] ^= true;
                    }
                }
            }
        }
        KeyCode::Char(n) => {
            if state.waiting_for_input && is_current_player(state) {
                let player_name = state
                    .current_state
                    .as_ref()
                    .and_then(|gd| gd.players.get(state.perspective_idx))
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| format!("Player{}", state.perspective_idx));
                match &state.pending_input {
                    Some(InputType::ChooseCards { .. }) | Some(InputType::ChoosePlayer { .. }) => {
                        // ignored: use arrows/space/enter for these
                    }
                    Some(InputType::Choice { max_index, .. }) => {
                        if let Some(digit) = n.to_digit(10) {
                            if let Some(idx) = choice_shortcut_idx(digit, *max_index) {
                                if let Some(ref tx) = state.input_tx {
                                    let _ = tx.send(Input {
                                        player_id: player_name,
                                        kind: InputKind::Choice { idx },
                                    });
                                    state.waiting_for_input = false;
                                    state.pending_input = None;
                                }
                            }
                        }
                    }
                    Some(InputType::Optional(_)) => {
                        if n == 'y' || n == 'Y' {
                            if let Some(ref tx) = state.input_tx {
                                let _ = tx.send(Input {
                                    player_id: player_name,
                                    kind: InputKind::OptionalAccept,
                                });
                                state.waiting_for_input = false;
                                state.pending_input = None;
                            }
                        } else if n == 'n' || n == 'N' {
                            if let Some(ref tx) = state.input_tx {
                                let _ = tx.send(Input {
                                    player_id: player_name,
                                    kind: InputKind::OptionalDecline,
                                });
                                state.waiting_for_input = false;
                                state.pending_input = None;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Enter => {
            if state.waiting_for_input && is_current_player(state) {
                let player_name = state
                    .current_state
                    .as_ref()
                    .and_then(|gd| gd.players.get(state.perspective_idx))
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| format!("Player{}", state.perspective_idx));
                match &state.pending_input {
                    Some(InputType::ChooseCards { min, max, .. }) => {
                        let selected: Vec<usize> = state
                            .choose_selected
                            .iter()
                            .enumerate()
                            .filter(|(_, &s)| s)
                            .map(|(i, _)| i)
                            .collect();
                        if selected.len() >= *min && selected.len() <= *max {
                            if let Some(ref tx) = state.input_tx {
                                let _ = tx.send(Input {
                                    player_id: player_name,
                                    kind: InputKind::ChooseCards { selected },
                                });
                                state.waiting_for_input = false;
                                state.pending_input = None;
                            }
                        }
                    }
                    Some(InputType::ChoosePlayer { .. }) => {
                        if let Some(ref tx) = state.input_tx {
                            let _ = tx.send(Input {
                                player_id: player_name,
                                kind: InputKind::ChoosePlayer {
                                    idx: state.choose_cursor,
                                },
                            });
                            state.waiting_for_input = false;
                            state.pending_input = None;
                        }
                    }
                    Some(InputType::Choice { .. }) => {
                        if let Some(ref tx) = state.input_tx {
                            let _ = tx.send(Input {
                                player_id: player_name,
                                kind: InputKind::Choice {
                                    idx: state.choose_cursor,
                                },
                            });
                            state.waiting_for_input = false;
                            state.pending_input = None;
                        }
                    }
                    Some(InputType::Optional(_)) => {
                        // Enter = accept (y/n remain the explicit
                        // controls; sending a non-Optional Input
                        // here would error the engine).
                        if let Some(ref tx) = state.input_tx {
                            let _ = tx.send(Input {
                                player_id: player_name,
                                kind: InputKind::OptionalAccept,
                            });
                            state.waiting_for_input = false;
                            state.pending_input = None;
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choice_shortcuts_cover_1_to_9() {
        assert_eq!(choice_shortcut_idx(1, 12), Some(0));
        assert_eq!(choice_shortcut_idx(9, 12), Some(8));
    }

    #[test]
    fn choice_shortcut_zero_selects_option_10() {
        assert_eq!(choice_shortcut_idx(0, 12), Some(9), "0 = option 10");
        assert_eq!(
            choice_shortcut_idx(0, 8),
            None,
            "no option 10 in a 9-option list"
        );
    }

    #[test]
    fn choice_shortcut_out_of_range_is_ignored() {
        assert_eq!(choice_shortcut_idx(7, 3), None, "option 7 does not exist");
    }

    #[test]
    fn quit_keys_return_true() {
        assert!(handle_key(KeyCode::Char('q'), &mut TuiState::new()));
        assert!(handle_key(KeyCode::F(10), &mut TuiState::new()));
        assert!(!handle_key(KeyCode::Char('l'), &mut TuiState::new()));
    }
}
