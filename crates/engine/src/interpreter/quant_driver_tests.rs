//! Unit tests for the quantifier resume arms. These reach `pub(super)`
//! functions by living inside the `interpreter` module tree.

use crate::game_data::GameData;
use crate::interpreter::{Interpreter, StepResult};
use front_end::ir::{Ir, LoweredPayLoad, StateID};
use std::collections::HashMap;

fn synth_state_id(n: u32) -> StateID {
    // `StateID(u32)` is `#[repr(transparent)]`; transmute is a test-only
    // shortcut (matches the helper in interpreter/tests.rs).
    unsafe { std::mem::transmute(n) }
}

#[test]
fn take_quant_resume_returns_none_when_no_pending_quant() {
    // When pending_quant is None, take_quant_resume must return None and
    // must not mutate state.
    let ir = Ir::<LoweredPayLoad>::default();
    let mut interp = Interpreter {
        ir,
        game_data: GameData::new(),
        input_buffer: Vec::new(),
        current_state: synth_state_id(0),
        trace_sender: None,
        pending_overlay: HashMap::new(),
        next_synth: u32::MAX - 1,
        pending_quant: None,
    };
    let before_state = interp.current_state;
    let resumed = interp.take_quant_resume();
    assert!(resumed.is_none(), "no pending quant => None");
    assert_eq!(interp.current_state, before_state, "state unchanged");
    // No extra assertions on resume-arm coverage here: the integration
    // tests in tests/quantifier_test.rs already cover the end-to-end
    // resume behavior (quantifier_deal_any_moves_chosen_card,
    // quantifier_range_rejects_zero_then_moves_two,
    // quantifier_dest_any_deals_to_chosen_player). Add direct unit tests
    // for those arms here when the resume state machine is refactored.
    let _ = std::mem::discriminant(&StepResult::Ok); // keep import used
}
