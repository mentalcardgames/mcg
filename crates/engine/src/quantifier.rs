//! Runtime preprocessor that rewrites quantifier-bearing edges into concrete
//! `Edge<LoweredPayLoad>` instances the existing `action::execute` path can run.
//!
//! The engine cannot natively execute an edge whose `Payload::Action` subtree
//! contains a `Quantifier::All`/`Quantifier::Any` over an
//! `Owner::PlayerCollection` destination, nor an `Any`/`IntRange` `Quantity`.
//! Rather than mutating the frozen `front_end::ir` contract, this module builds
//! an **ephemeral overlay** of synthetic replacement edges that flow through
//! the *unchanged* `action::execute` → `execute_cardset_move` path. Where a
//! quantifier requires player choice, `Interpreter::step` returns one of the
//! new `InputType::{ChoosePlayer, ChooseCards}` variants; the controller
//! round-trips the answer back through `provide_input`, and the preprocessor
//! substitutes the answer into a single concrete replacement edge.
//!
//! `self.ir` is **never** mutated. Replacement edges live in
//! `Interpreter::pending_overlay`, keyed only by synthetic `StateID`s (never by
//! real IR ids). See `crates/engine/docs/invariants.md` (I-5, I-7, I-10).

use crate::game_data::GameData;
use front_end::ast::{
    AggregatePlayerCollection, CardSet, IntExpr, IntRange, IntRangeOperator, MoveCardSet, Owner,
    PlayerCollection, PlayerExpr, Quantifier, Quantity, UseMemory,
};
use front_end::ir::{Edge, LoweredPayLoad, Payload, StateID};

/// Namespaced synthetic memory slot used to carry the player-chosen card ids
/// from a `ChooseCards` round-trip into the replacement edge's `from`
/// (`CardSet::Memory`). Written into `game_data.memories` immediately before
/// dispatching the replacement edge and removed right after the quantifier
/// edge completes, so user `.cgdsl` programs that later `CreateMemory` with
/// the same key by coincidence are unaffected.
pub const SYNTH_MEMORY_KEY: &str = "__quantifier_overlay_cards";

/// Hard cap on a dest-player fan-out (e.g. `Hand of all` resolving to N
/// players). A safety valve against runaway dynamic sets; exceeding it yields
/// `StepResult::Error` (not a panic) so the controller can surface the failure.
/// Large real games can raise this via a const change.
pub const FANOUT_CAP: usize = 64;

/// Where a quantifier site sits on an edge, in the precedence order
/// `DestPlayerAll` > `DestPlayerAny` > `SrcCardsAnyOrRange`. `scan_edge`
/// returns the first site that matches; the resume branches in `step()`
/// handle any additional lower-precedence site.
#[allow(clippy::large_enum_variant)] // ephemeral, never stored in bulk
#[derive(Clone, Debug, PartialEq)]
pub enum QuantSite {
    None,
    /// `to` is `CardSet::GroupOwner { owner: Owner::PlayerCollection {
    /// PlayerCollection::Aggregate { Quantifier::All } } }` — fan-out target.
    DestPlayerAll {
        pc: PlayerCollection,
    },
    /// As above but `Quantifier::Any` — pick-one target.
    DestPlayerAny {
        pc: PlayerCollection,
    },
    /// `quantity` is `Quantifier::Any` or `IntRange` — pick cards / count.
    SrcCardsAnyOrRange {
        qty: Quantity,
        from: CardSet,
    },
}

/// State carried across the `NeedsInput` round-trip for a quantifier that
/// requires player input. `state` is the real IR `StateID` we were sitting on
/// when the prompt was issued, so the resume branch can confirm it is
/// resuming the right edge (the FSM does not advance while waiting).
#[derive(Clone, Debug)]
pub struct PendingQuant {
    pub state: StateID,
    pub kind: PendingKind,
}

/// Which flavour of pending quantifier is awaiting input. The original edge is
/// carried verbatim so the resume branch can rebuild the replacement edge from
/// scratch (the original edge is never partially mutated).
#[derive(Clone, Debug)]
pub enum PendingKind {
    /// `DestPlayerAny`: a single player must be chosen from `candidates`; on
    /// resume, the chosen name is substituted into the original edge.
    DestPlayerAny {
        candidates: Vec<String>,
        original: Edge<LoweredPayLoad>,
    },
    /// `SrcCardsAnyOrRange`: a card subset must be chosen from
    /// `candidate_ids`; on resume, the chosen ids are written into the
    /// synthetic memory slot and the original edge's `from`/`quantity` are
    /// rewritten to read it.
    CardsAnyOrRange {
        candidate_ids: Vec<usize>,
        original: Edge<LoweredPayLoad>,
    },
    /// `All`-of-`Any`: the per-player fan-out targets are pre-decided
    /// (`player_names`) but a single card choice is still pending. On resume,
    /// the chosen cards are written to the synthetic memory once and shared
    /// across every per-player edge in the fan-out chain.
    DestAllThenCards {
        player_names: Vec<String>,
        candidate_ids: Vec<usize>,
        original: Edge<LoweredPayLoad>,
    },
}

/// Allocate a synthetic [`StateID`] that cannot collide with real IR ids.
///
/// Real IR ids are allocated densely from 0 upward by the `front_end` IR
/// builder. We seed the allocator at `u32::MAX - 1` and decrement, so
/// synthetic ids live at the very top of the `u32` space and never shadow a
/// real state. `wrapping_sub` prevents overflow panics on pathological reuse —
/// the id space (2³²) is effectively unlimited for any realistic game.
///
/// `StateID`'s tuple field is private to `front_end::ir` (only `raw()` is
/// public), so — per the plan's sanctioned fallback — we construct one via
/// serde deserialisation: `StateID` derives `Serialize`/`Deserialize` as a
/// transparent newtype around `u32`, so deserialising a `u32` yields the
/// equivalent `StateID`. This stays inside the `/crates/engine` boundary
/// (the engine crate already depends on `serde`/`serde_json`).
pub fn alloc_synth(next_synth: &mut u32) -> StateID {
    let raw = *next_synth;
    *next_synth = next_synth.wrapping_sub(1);
    serde_json::from_value(serde_json::Value::from(raw))
        .expect("StateID deserialisation from a valid u32 cannot fail")
}

/// Resolve a dest-quantifier `PlayerCollection` to concrete player indices
/// *without* touching `Evaluator::resolve_player_collection`'s `Aggregate`
/// arm (which is a `todo!()` — see `query.rs`). For
/// `Aggregate { AggregatePlayerCollection::Quantifier }` we mirror
/// `RuntimePlayerCollection::PlayersIn`: every in-game player is a candidate
/// (the `All`/`Any` distinction — fan-out vs pick-one — is the caller's job).
/// For every other `PlayerCollection` variant we delegate to the existing
/// (working) resolver.
///
/// NOTE: this deviates from the plan's literal Task 19 instruction to call
/// `Evaluator::resolve_player_collection(&pc, ...)` directly, because that
/// would hit the `todo!()` panic for an `Aggregate`. The deviation is
/// consistent with the plan's anti-pattern guidance ("the preprocessor must
/// intercept `Aggregate` before any code path that reaches
/// `resolve_player_collection`").
pub fn resolve_player_candidates(pc: &PlayerCollection, game_data: &GameData) -> Vec<usize> {
    match pc {
        PlayerCollection::Aggregate { aggregate } => match aggregate {
            AggregatePlayerCollection::Quantifier { .. } => game_data
                .players
                .iter()
                .enumerate()
                .filter(|(_, p)| p.in_game)
                .map(|(i, _)| i)
                .collect(),
        },
        _ => crate::query::Evaluator::resolve_player_collection(pc, game_data),
    }
}

/// Immutably borrow the `MoveCardSet` carried by a Move edge (Deal/Exchange/
/// Classic), if any. `Place` and non-Move actions yield `None`.
fn move_cardset_ref(edge: &Edge<LoweredPayLoad>) -> Option<&MoveCardSet> {
    match &edge.payload {
        Payload::Action(front_end::ast::GameRule::Action {
            action: front_end::ast::ActionRule::Move { move_type },
        }) => match move_type {
            front_end::ast::MoveType::Deal { deal } => match deal {
                front_end::ast::DealMove::MoveCardSet { deal_cs } => Some(deal_cs),
            },
            front_end::ast::MoveType::Exchange { exchange } => match exchange {
                front_end::ast::ExchangeMove::MoveCardSet { exchange_cs } => Some(exchange_cs),
            },
            front_end::ast::MoveType::Classic { classic } => match classic {
                front_end::ast::ClassicMove::MoveCardSet { move_cs } => Some(move_cs),
            },
            front_end::ast::MoveType::Place { .. } => None,
        },
        _ => None,
    }
}

/// Mutably borrow the `MoveCardSet` carried by a Move edge (Deal/Exchange/
/// Classic), if any. `Place` and non-Move actions yield `None`.
fn move_cardset_mut(edge: &mut Edge<LoweredPayLoad>) -> Option<&mut MoveCardSet> {
    match &mut edge.payload {
        Payload::Action(front_end::ast::GameRule::Action {
            action: front_end::ast::ActionRule::Move { move_type },
        }) => match move_type {
            front_end::ast::MoveType::Deal { deal } => match deal {
                front_end::ast::DealMove::MoveCardSet { deal_cs } => Some(deal_cs),
            },
            front_end::ast::MoveType::Exchange { exchange } => match exchange {
                front_end::ast::ExchangeMove::MoveCardSet { exchange_cs } => Some(exchange_cs),
            },
            front_end::ast::MoveType::Classic { classic } => match classic {
                front_end::ast::ClassicMove::MoveCardSet { move_cs } => Some(move_cs),
            },
            front_end::ast::MoveType::Place { .. } => None,
        },
        _ => None,
    }
}

/// Mutably borrow the `to` card-set's `owner`, if the edge is a Move whose
/// `to` is a `CardSet::GroupOwner`. Returns `None` for any other shape
/// (defensive: callers only invoke this on edges `scan_edge` flagged as
/// dest-quantifier sites).
fn dest_owner_mut(edge: &mut Edge<LoweredPayLoad>) -> Option<&mut Owner> {
    let mcs = move_cardset_mut(edge)?;
    let to = match mcs {
        MoveCardSet::Move { to, .. } => to,
        MoveCardSet::MoveQuantity { to, .. } => to,
    };
    match to {
        front_end::ast::CardSet::GroupOwner { owner, .. } => Some(owner),
        _ => None,
    }
}

/// Immutably borrow the `to` card-set of a `MoveCardSet` (shared by both
/// variants).
fn mcs_to_ref(mcs: &MoveCardSet) -> &CardSet {
    match mcs {
        MoveCardSet::Move { to, .. } => to,
        MoveCardSet::MoveQuantity { to, .. } => to,
    }
}

/// Immutably borrow the `from` card-set of a `MoveCardSet` (shared by both
/// variants). Test-only: `from` access in production goes through the
/// `MoveQuantity` variant match in [`card_site`] / [`substitute_cardset_memory`].
#[cfg(test)]
fn mcs_from_ref(mcs: &MoveCardSet) -> &CardSet {
    match mcs {
        MoveCardSet::Move { from, .. } => from,
        MoveCardSet::MoveQuantity { from, .. } => from,
    }
}

/// Classify the destination card-set into a dest-quantifier site (or `None`).
fn dest_site_for(to: &CardSet) -> QuantSite {
    if let front_end::ast::CardSet::GroupOwner {
        owner:
            Owner::PlayerCollection {
                player_collection:
                    pc @ PlayerCollection::Aggregate {
                        aggregate: AggregatePlayerCollection::Quantifier { quantifier },
                    },
            },
        ..
    } = to
    {
        return match quantifier {
            Quantifier::All => QuantSite::DestPlayerAll { pc: pc.clone() },
            Quantifier::Any => QuantSite::DestPlayerAny { pc: pc.clone() },
        };
    }
    QuantSite::None
}

/// Detect the quantifier site on `edge`, in precedence order
/// `DestPlayerAll` > `DestPlayerAny` > `SrcCardsAnyOrRange`.
///
/// If `to` is a `CardSet::GroupOwner` whose `owner` is
/// `Owner::PlayerCollection { PlayerCollection::Aggregate { Quantifier::All } }`
/// → `DestPlayerAll`; `Quantifier::Any` → `DestPlayerAny`. These short-circuit
/// so that a combined `All`-of-`Any` edge (e.g. `deal any from Stock to Hand
/// of all`) scans as `DestPlayerAll` and the card choice is handled by the
/// resume branch via [`card_site`].
///
/// Otherwise, if the edge is a `MoveQuantity` whose `quantity` is
/// `Quantifier::Any` or `IntRange` → `SrcCardsAnyOrRange`.
pub fn scan_edge(edge: &Edge<LoweredPayLoad>) -> QuantSite {
    let Some(mcs) = move_cardset_ref(edge) else {
        return QuantSite::None;
    };

    // Destination quantifier (highest precedence).
    let dest = dest_site_for(mcs_to_ref(mcs));
    if dest != QuantSite::None {
        return dest;
    }

    // Card-amount quantifier (only MoveQuantity carries a quantity).
    if let front_end::ast::MoveCardSet::MoveQuantity { quantity, from, .. } = mcs {
        match quantity {
            Quantity::Quantifier {
                quantifier: Quantifier::Any,
            }
            | Quantity::IntRange { .. } => {
                return QuantSite::SrcCardsAnyOrRange {
                    qty: quantity.clone(),
                    from: from.clone(),
                };
            }
            _ => {}
        }
    }

    QuantSite::None
}

/// If `edge` also carries a card-amount quantifier site (`Quantifier::Any` or
/// `IntRange` on its `quantity`), return the `(quantity, from)` so the caller
/// can fire a `ChooseCards` prompt before fanning out. Used by the
/// `DestPlayerAll` arm to implement `All`-of-`Any` (single card choice shared
/// across the per-player fan-out).
pub fn card_site(edge: &Edge<LoweredPayLoad>) -> Option<(Quantity, CardSet)> {
    let mcs = move_cardset_ref(edge)?;
    let front_end::ast::MoveCardSet::MoveQuantity { quantity, from, .. } = mcs else {
        return None;
    };
    match quantity {
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        }
        | Quantity::IntRange { .. } => Some((quantity.clone(), from.clone())),
        _ => None,
    }
}

/// Clone `edge` and replace the `to` card-set's owner with a concrete
/// `Owner::Player { PlayerExpr::Literal { player_name } }`. If the edge's `to`
/// is not a `GroupOwner` (defensive), the edge is returned unchanged.
pub fn substitute_dest_player(
    edge: &Edge<LoweredPayLoad>,
    player_name: String,
) -> Edge<LoweredPayLoad> {
    let mut repl = edge.clone();
    if let Some(owner) = dest_owner_mut(&mut repl) {
        *owner = Owner::Player {
            player: PlayerExpr::Literal { name: player_name },
        };
    }
    repl
}

/// Clone `edge` and rewrite its `from` to read the chosen card ids from the
/// synthetic memory slot ([`SYNTH_MEMORY_KEY`]). The caller must have written
/// the chosen ids into `game_data.memories` under that key first.
///
/// For a `MoveQuantity`, the `quantity` is also rewritten to
/// `Quantity::Int { IntExpr::Literal { chosen.len() } }` so `resolve_quantity`
/// returns exactly the chosen count (consistent with the memory contents). For
/// a bare `Move` (no quantity), the executor moves *all* cards in `from` —
/// which is exactly the chosen set, since `from` now points at the synthetic
/// memory holding precisely those ids — so no variant surgery is needed.
pub fn substitute_cardset_memory(
    edge: &Edge<LoweredPayLoad>,
    chosen: &[usize],
) -> Edge<LoweredPayLoad> {
    let mut repl = edge.clone();
    if let Some(mcs) = move_cardset_mut(&mut repl) {
        let new_from = CardSet::Memory {
            memory: UseMemory::Memory {
                memory: SYNTH_MEMORY_KEY.to_string(),
            },
        };
        match mcs {
            front_end::ast::MoveCardSet::Move { from, .. } => {
                *from = new_from;
            }
            front_end::ast::MoveCardSet::MoveQuantity { quantity, from, .. } => {
                *from = new_from;
                *quantity = Quantity::Int {
                    int: IntExpr::Literal {
                        int: chosen.len() as i32,
                    },
                };
            }
        }
    }
    repl
}

/// Build the synthetic fan-out chain for a `DestPlayerAll` edge: one
/// per-player replacement edge per name, threaded through freshly-allocated
/// synthetic `StateID`s. The returned `Vec<(synth_id, edge)>` is keyed into
/// `Interpreter::pending_overlay` by the caller; `step()` then dispatches each
/// edge in turn (the first entry's `synth_id` is where `current_state` is
/// moved to). The last edge's `to` is the original `edge.to`.
///
/// Returns `Err` if `player_names.len()` exceeds [`FANOUT_CAP`]. Returns an
/// empty `Vec` for an empty player set (the caller no-ops and advances to
/// `edge.to`).
pub fn build_dest_all_chain(
    edge: &Edge<LoweredPayLoad>,
    player_names: Vec<String>,
    next_synth: &mut u32,
) -> Result<Vec<(StateID, Edge<LoweredPayLoad>)>, String> {
    let n = player_names.len();
    if n > FANOUT_CAP {
        return Err(format!(
            "dest-player fan-out {} exceeds cap {}",
            n, FANOUT_CAP
        ));
    }
    if n == 0 {
        return Ok(vec![]);
    }

    // Pre-allocate all n synthetic ids so each per-player edge's `to` can
    // point at the next synth (the last points at the original `edge.to`).
    let synth_ids: Vec<StateID> = (0..n).map(|_| alloc_synth(next_synth)).collect();

    let mut chain = Vec::with_capacity(n);
    for (i, name) in player_names.into_iter().enumerate() {
        let mut per_player = substitute_dest_player(edge, name);
        per_player.to = if i + 1 < n { synth_ids[i + 1] } else { edge.to };
        chain.push((synth_ids[i], per_player));
    }
    Ok(chain)
}

/// Validate a player-chosen `count` against an `IntRange`, mirroring the
/// comparison semantics of `Evaluator::resolve_quantity`'s `IntRange` arm but
/// parameterised by the proposed `count` rather than `available`.
///
/// The range is folded left-to-right (the grammar declares no operator
/// precedence): `start` is the first atom, each `op_int` entry combines the
/// running result with its atom via `And`/`Or`. Returns `Ok(())` if the final
/// result holds, `Err(message)` otherwise.
///
/// Non-literal `IntExpr`s (memory/runtime-backed) cannot be evaluated without
/// live `GameData`; per the plan's edge-case note, we fall back to "accept any
/// count in `[0, available]`" in that case.
pub fn validate_int_range(range: &IntRange, count: usize, available: usize) -> Result<(), String> {
    let gd = GameData::new();

    let (start_cmp, start_expr) = &range.start;
    let start_target = match crate::query::Evaluator::eval_int(start_expr, &gd) {
        Ok(t) => t,
        Err(_) => return validate_fallback(count, available),
    };
    let mut result =
        crate::query::Evaluator::eval_int_compare(count as i32, start_cmp, start_target);

    for (op, cmp, expr) in &range.op_int {
        let target = match crate::query::Evaluator::eval_int(expr, &gd) {
            Ok(t) => t,
            Err(_) => return validate_fallback(count, available),
        };
        let atom = crate::query::Evaluator::eval_int_compare(count as i32, cmp, target);
        result = match op {
            IntRangeOperator::And => result && atom,
            IntRangeOperator::Or => result || atom,
        };
    }

    if result {
        Ok(())
    } else {
        Err(format!(
            "selected {} does not satisfy range {:?}",
            count, range
        ))
    }
}

/// Fallback when an `IntRange` constraint uses a non-literal `IntExpr`: accept
/// any count that does not exceed `available`.
fn validate_fallback(count: usize, available: usize) -> Result<(), String> {
    if count <= available {
        Ok(())
    } else {
        Err(format!(
            "selected {} exceeds available {}",
            count, available
        ))
    }
}

/// Like [`build_dest_all_chain`], but each per-player edge is additionally
/// rewritten by [`substitute_cardset_memory`] so it reads the chosen card ids
/// from the synthetic memory slot. Used for `All`-of-`Any` (e.g. `deal any
/// from Stock to Hand of all`): one `ChooseCards` round-trip, then a fan-out
/// whose every per-player edge shares the same synthetic memory.
pub fn build_dest_all_chain_with_memory(
    edge: &Edge<LoweredPayLoad>,
    player_names: Vec<String>,
    chosen: &[usize],
    next_synth: &mut u32,
) -> Result<Vec<(StateID, Edge<LoweredPayLoad>)>, String> {
    let n = player_names.len();
    if n > FANOUT_CAP {
        return Err(format!(
            "dest-player fan-out {} exceeds cap {}",
            n, FANOUT_CAP
        ));
    }
    if n == 0 {
        return Ok(vec![]);
    }

    let synth_ids: Vec<StateID> = (0..n).map(|_| alloc_synth(next_synth)).collect();
    let mut chain = Vec::with_capacity(n);
    for (i, name) in player_names.into_iter().enumerate() {
        // Substitute the dest player, then rewrite from/quantity to read the
        // shared synthetic memory. The two substitutions touch disjoint fields
        // (owner vs from/quantity), so order is irrelevant.
        let mut per_player = substitute_dest_player(edge, name);
        per_player = substitute_cardset_memory(&per_player, chosen);
        per_player.to = if i + 1 < n { synth_ids[i + 1] } else { edge.to };
        chain.push((synth_ids[i], per_player));
    }
    Ok(chain)
}

/// Derive a `(min, max)` UI hint for a `ChooseCards` prompt from the quantity.
///
/// For `Quantifier::Any` the player must pick at least one card (or zero if
/// there are no candidates). For `IntRange` the hint is loose (`0..available`)
/// — the authoritative check is [`validate_int_range`] on resume, which can
/// re-issue the prompt if the chosen count violates the range. This keeps the
/// controller's bounds check permissive enough to let the interpreter's
/// `IntRange` validation (and its re-prompt path) actually exercise.
pub fn derive_min_max(qty: &Quantity, available: usize) -> (usize, usize) {
    match qty {
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        } => (if available == 0 { 0 } else { 1 }, available),
        // IntRange: validated authoritatively by validate_int_range on resume.
        _ => (0, available),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_data::MemoryValue;
    use crate::query::Evaluator;
    use front_end::ast::{
        ActionRule, ClassicMove, GameRule, Group, Groupable, IntCompare, MoveType, Status,
    };
    use front_end::ir::Ir;

    /// A throwaway `StateID` for the `to` of hand-built test edges. `Ir::default`
    /// has `entry == StateID(0)`, which is fine for unit tests (we never
    /// dispatch these edges through the real interpreter).
    fn dest_state() -> StateID {
        Ir::<LoweredPayLoad>::default().entry
    }

    fn loc_cardset(name: &str) -> CardSet {
        CardSet::Group {
            group: Group::Groupable {
                groupable: Groupable::Location {
                    name: name.to_string(),
                },
            },
        }
    }

    fn groupowner_cardset(name: &str, owner: Owner) -> CardSet {
        CardSet::GroupOwner {
            group: Group::Groupable {
                groupable: Groupable::Location {
                    name: name.to_string(),
                },
            },
            owner,
        }
    }

    fn aggregate_owner(quantifier: Quantifier) -> Owner {
        Owner::PlayerCollection {
            player_collection: PlayerCollection::Aggregate {
                aggregate: AggregatePlayerCollection::Quantifier { quantifier },
            },
        }
    }

    /// Build a `Classic`/`MoveQuantity` edge carrying the given quantity/from/to.
    fn move_qty_edge(quantity: Quantity, from: CardSet, to: CardSet) -> Edge<LoweredPayLoad> {
        Edge {
            to: dest_state(),
            payload: Payload::Action(GameRule::Action {
                action: ActionRule::Move {
                    move_type: MoveType::Classic {
                        classic: ClassicMove::MoveCardSet {
                            move_cs: MoveCardSet::MoveQuantity {
                                quantity,
                                from,
                                status: Status::Private,
                                to,
                            },
                        },
                    },
                },
            }),
            meta: None,
        }
    }

    #[test]
    fn alloc_synth_yields_valid_decreasing_stateids() {
        let mut counter = u32::MAX - 1;
        let mut prev_raw = u32::MAX;
        for _ in 0..1024 {
            let id = alloc_synth(&mut counter);
            let raw = id.raw();
            assert_ne!(raw, 0, "synthetic ids must never be 0");
            assert_eq!(raw, prev_raw - 1, "ids must decrease monotonically");
            prev_raw = raw;
        }
        assert_eq!(
            alloc_synth(&mut (u32::MAX - 1)).raw(),
            u32::MAX - 1,
            "first allocation from a fresh seed is u32::MAX - 1"
        );
    }

    #[test]
    fn alloc_synth_wraps_without_panicking() {
        let mut counter = 0u32;
        let _ = alloc_synth(&mut counter);
        let _ = alloc_synth(&mut counter);
    }

    #[test]
    fn scan_edge_dest_player_all() {
        let edge = move_qty_edge(
            Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            loc_cardset("Stock"),
            groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
        );
        assert!(matches!(scan_edge(&edge), QuantSite::DestPlayerAll { .. }));
    }

    #[test]
    fn scan_edge_dest_player_any() {
        let edge = move_qty_edge(
            Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            loc_cardset("Stock"),
            groupowner_cardset("Hand", aggregate_owner(Quantifier::Any)),
        );
        assert!(matches!(scan_edge(&edge), QuantSite::DestPlayerAny { .. }));
    }

    #[test]
    fn scan_edge_src_cards_any() {
        let edge = move_qty_edge(
            Quantity::Quantifier {
                quantifier: Quantifier::Any,
            },
            loc_cardset("Stock"),
            loc_cardset("Discard"),
        );
        assert!(matches!(
            scan_edge(&edge),
            QuantSite::SrcCardsAnyOrRange { .. }
        ));
    }

    #[test]
    fn scan_edge_src_cards_int_range() {
        let range = IntRange {
            start: (IntCompare::Ge, IntExpr::Literal { int: 1 }),
            op_int: vec![(
                IntRangeOperator::And,
                IntCompare::Le,
                IntExpr::Literal { int: 3 },
            )],
        };
        let edge = move_qty_edge(
            Quantity::IntRange { int_range: range },
            loc_cardset("Stock"),
            loc_cardset("Discard"),
        );
        assert!(matches!(
            scan_edge(&edge),
            QuantSite::SrcCardsAnyOrRange { .. }
        ));
    }

    #[test]
    fn scan_edge_none_for_concrete_move() {
        let edge = move_qty_edge(
            Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            loc_cardset("Stock"),
            loc_cardset("Discard"),
        );
        assert_eq!(scan_edge(&edge), QuantSite::None);
    }

    #[test]
    fn scan_edge_precedence_all_over_card_any() {
        // `deal any from Stock to Hand of all` — both a dest-all site and a
        // card-any site. scan_edge must report DestPlayerAll (the resume
        // branch handles the card choice via card_site).
        let edge = move_qty_edge(
            Quantity::Quantifier {
                quantifier: Quantifier::Any,
            },
            loc_cardset("Stock"),
            groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
        );
        assert!(matches!(scan_edge(&edge), QuantSite::DestPlayerAll { .. }));
        assert!(
            card_site(&edge).is_some(),
            "card_site must still detect the any-qty"
        );
    }

    #[test]
    fn substitute_dest_player_replaces_owner() {
        let edge = move_qty_edge(
            Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            loc_cardset("Stock"),
            groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
        );
        let repl = substitute_dest_player(&edge, "P2".to_string());
        let mcs = move_cardset_ref(&repl).expect("edge still a Move");
        match mcs_to_ref(mcs) {
            CardSet::GroupOwner {
                owner:
                    Owner::Player {
                        player: PlayerExpr::Literal { name },
                    },
                ..
            } => assert_eq!(name, "P2"),
            other => panic!("expected concrete Player owner, got {:?}", other),
        }
    }

    #[test]
    fn substitute_cardset_memory_round_trips_through_eval_cardset() {
        let edge = move_qty_edge(
            Quantity::Quantifier {
                quantifier: Quantifier::Any,
            },
            loc_cardset("Stock"),
            loc_cardset("Discard"),
        );
        let repl = substitute_cardset_memory(&edge, &[5, 7]);
        let mcs = move_cardset_ref(&repl).expect("edge still a Move");

        let mut gd = GameData::new();
        gd.memories.insert(
            SYNTH_MEMORY_KEY.to_string(),
            MemoryValue::CardSet(vec![5, 7]),
        );
        let (loc_idx, card_ids) =
            Evaluator::eval_cardset(mcs_from_ref(mcs), &gd).expect("eval_cardset ok");
        assert_eq!(card_ids, vec![5, 7]);
        // No location holds card 5 in this empty GameData, so the fallback
        // sentinel loc_idx 0 is returned (invariant I-14).
        assert_eq!(loc_idx, 0);
    }

    #[test]
    fn build_dest_all_chain_length_and_targets() {
        let edge = move_qty_edge(
            Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            loc_cardset("Stock"),
            groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
        );
        let mut next = u32::MAX - 1;
        let chain = build_dest_all_chain(
            &edge,
            vec!["P1".into(), "P2".into(), "P3".into()],
            &mut next,
        )
        .expect("chain builds");
        assert_eq!(chain.len(), 3);
        // Each per-player edge must target the next synth (or the original
        // `edge.to` for the last).
        assert_eq!(chain[1].0, chain[0].1.to, "edge 0 targets synth 1");
        assert_eq!(chain[2].0, chain[1].1.to, "edge 1 targets synth 2");
        assert_eq!(chain[2].1.to, edge.to, "last edge targets the original to");
        // Each per-player edge has a concrete Player owner.
        for (_, e) in &chain {
            let mcs = move_cardset_ref(e).expect("per-player edge is a Move");
            assert!(
                matches!(
                    mcs_to_ref(mcs),
                    CardSet::GroupOwner {
                        owner: Owner::Player { .. },
                        ..
                    }
                ),
                "per-player edge must have a concrete Player owner"
            );
        }
    }

    #[test]
    fn build_dest_all_chain_empty_is_noop() {
        let edge = move_qty_edge(
            Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            loc_cardset("Stock"),
            groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
        );
        let mut next = u32::MAX - 1;
        let chain = build_dest_all_chain(&edge, vec![], &mut next).expect("empty chain ok");
        assert!(chain.is_empty());
    }

    #[test]
    fn build_dest_all_chain_errors_over_cap() {
        let edge = move_qty_edge(
            Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            loc_cardset("Stock"),
            groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
        );
        let mut next = u32::MAX - 1;
        let names: Vec<String> = (0..=FANOUT_CAP).map(|i| format!("P{i}")).collect();
        let result = build_dest_all_chain(&edge, names, &mut next);
        assert!(result.is_err(), "fan-out > cap must error");
    }

    #[test]
    fn validate_int_range_accepts_in_range_count() {
        // `>= 1 and <= 3`
        let range = IntRange {
            start: (IntCompare::Ge, IntExpr::Literal { int: 1 }),
            op_int: vec![(
                IntRangeOperator::And,
                IntCompare::Le,
                IntExpr::Literal { int: 3 },
            )],
        };
        assert!(validate_int_range(&range, 1, 10).is_ok());
        assert!(validate_int_range(&range, 2, 10).is_ok());
        assert!(validate_int_range(&range, 3, 10).is_ok());
    }

    #[test]
    fn validate_int_range_rejects_out_of_range_count() {
        let range = IntRange {
            start: (IntCompare::Ge, IntExpr::Literal { int: 1 }),
            op_int: vec![(
                IntRangeOperator::And,
                IntCompare::Le,
                IntExpr::Literal { int: 3 },
            )],
        };
        assert!(validate_int_range(&range, 0, 10).is_err(), "0 < 1");
        assert!(validate_int_range(&range, 4, 10).is_err(), "4 > 3");
        assert!(validate_int_range(&range, 100, 10).is_err(), "100 > 3");
    }
}
