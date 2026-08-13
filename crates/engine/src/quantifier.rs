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

use crate::error::EngineError;
use crate::game_data::GameData;
use front_end::ast::{
    AggregatePlayerCollection, CardSet, FilterExpr, Group, IntExpr, IntRange, IntRangeOperator,
    MoveCardSet, Owner, PlayerCollection, PlayerExpr, Quantifier, Quantity, SetUpRule, UseMemory,
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
    /// `move N from <non-positional>` (Classic/Exchange): pick **exactly N**
    /// cards (2026-08-10 verb semantics — `move` = choose, `deal` = top).
    SrcCardsExactN {
        qty: Quantity,
        from: CardSet,
    },
    /// `deal any` / `deal >= M and <= N from X`: prompt for **how many**
    /// cards to deal (an `InputType::Number` count), then deal that many
    /// from the top automatically (2026-08-10 verb semantics).
    DealCount {
        qty: Quantity,
        from: CardSet,
    },
    /// The move's `from` is a combo group (`<combo> in <pile>`): prompt the
    /// player to choose cards from the pile, then validate the chosen set
    /// against the combo's filter ("laying down", see D-16).
    ComboSource {
        combo: String,
        from: CardSet,
    },
    /// The move's `from` is a `CardSet::GroupOwner` whose owner is
    /// `Owner::PlayerCollection { Quantifier::Any }` — the *source* player is
    /// pick-one. Mirrors `DestPlayerAny` for the `from` side (e.g.
    /// `deal Hand where Rank is "Ace" of any …` — "ask any player").
    SourcePlayerAny {
        pc: PlayerCollection,
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
    /// `SourcePlayerAny`: like `DestPlayerAny`, but the chosen player is
    /// substituted into the original edge's *`from`* owner.
    SourcePlayerAny {
        candidates: Vec<String>,
        original: Edge<LoweredPayLoad>,
    },
    /// `SetupAny` (I-20, relaxed): a setup rule contains `Quantifier::Any`
    /// (e.g. `location Hand on any`); the chosen player is substituted into
    /// every any-site of the setup before dispatch.
    SetupAny {
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
    /// `SrcCardsExactN` (2026-08-10): pick exactly `expected` cards from
    /// `candidate_ids` (`min=max=expected`; `expected` is clamped to the
    /// candidate count). Resume writes the chosen ids into the synthetic
    /// memory slot.
    CardsExactN {
        candidate_ids: Vec<usize>,
        expected: usize,
        original: Edge<LoweredPayLoad>,
    },
    /// `DealCount` (2026-08-10): the player chose a count for a `deal`;
    /// on resume the original edge's quantity is substituted with the
    /// literal `value`. `min`/`max` are the prompt bounds (re-validated on
    /// resume; out-of-range answers re-prompt).
    DealCount {
        min: Option<i32>,
        max: Option<i32>,
        prompt: String,
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
    /// Combo lay-down: the player chose cards from `candidate_ids` (the whole
    /// source pile); on resume the choice must satisfy `filter` (the combo's
    /// filter), otherwise the prompt is re-issued.
    Combo {
        candidate_ids: Vec<usize>,
        filter: FilterExpr,
        original: Edge<LoweredPayLoad>,
    },
}

/// Allocate a synthetic [`StateID`] that cannot collide with real IR ids.
///
/// Real IR ids are allocated densely from 0 upward by the `front_end` IR
/// builder, and the IR is frozen by the time the engine runs. The allocator
/// counter is therefore seeded at `max(real id) + 1` (see
/// `Interpreter::new`) and incremented, so synthetic ids live directly above
/// the real ones and can never shadow a real state. `wrapping_add` prevents
/// overflow panics on pathological reuse — the id space (2³²) is effectively
/// unlimited for any realistic game.
pub fn alloc_synth(next_synth: &mut u32) -> StateID {
    let raw = *next_synth;
    *next_synth = next_synth.wrapping_add(1);
    StateID::from_raw(raw)
}

fn pc_is_any(pc: &PlayerCollection) -> bool {
    matches!(
        pc,
        PlayerCollection::Aggregate {
            aggregate: AggregatePlayerCollection::Quantifier {
                quantifier: Quantifier::Any
            }
        }
    )
}

fn owner_is_any(owner: &Owner) -> bool {
    matches!(owner, Owner::PlayerCollection { player_collection: pc } if pc_is_any(pc))
}

pub fn setup_contains_any(setup: &SetUpRule) -> bool {
    match setup {
        SetUpRule::CreatePlayer { .. } => false,
        SetUpRule::CreateTeams { teams } => teams.iter().any(|(_, pc)| pc_is_any(pc)),
        SetUpRule::CreateTurnorder { player_collection }
        | SetUpRule::CreateTurnorderRandom { player_collection } => pc_is_any(player_collection),
        SetUpRule::CreateLocation { owner, .. } => owner_is_any(owner),
        SetUpRule::CreateCardOnLocation { .. } => false,
        SetUpRule::CreateTokenOnLocation { .. } => false,
        SetUpRule::CreateCombo { .. } => false,
        SetUpRule::CreatePrecedence { .. } => false,
        SetUpRule::CreatePointMap { .. } => false,
        SetUpRule::CreateMemory { owner, .. }
        | SetUpRule::CreateMemoryWithMemoryType { owner, .. } => owner_is_any(owner),
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

/// If the move's `from` is a combo group (`<combo> in <groupable>`, with or
/// without an owner), return the combo name and the full `from` cardset.
fn combo_source(mcs: &MoveCardSet) -> Option<(String, CardSet)> {
    let from = match mcs {
        MoveCardSet::Move { from, .. } => from,
        MoveCardSet::MoveQuantity { from, .. } => from,
    };
    match from {
        CardSet::Group {
            group: Group::Combo { combo, .. },
        } => Some((combo.clone(), from.clone())),
        CardSet::GroupOwner {
            group: Group::Combo { combo, .. },
            ..
        } => Some((combo.clone(), from.clone())),
        _ => None,
    }
}

/// If `from` is a `CardSet::GroupOwner` whose owner is
/// `Owner::PlayerCollection { PlayerCollection::Aggregate { Quantifier::Any } }`,
/// return the collection — the candidate source for the `ChoosePlayer` prompt.
fn source_any_site(from: &CardSet) -> Option<PlayerCollection> {
    if let CardSet::GroupOwner {
        owner:
            Owner::PlayerCollection {
                player_collection:
                    pc @ PlayerCollection::Aggregate {
                        aggregate:
                            AggregatePlayerCollection::Quantifier {
                                quantifier: Quantifier::Any,
                            },
                    },
            },
        ..
    } = from
    {
        Some(pc.clone())
    } else {
        None
    }
}

/// Rebuild the *pile* cardset that a combo group filters — i.e. drop the
/// combo, keep the groupable (and owner). Used to prompt the player over the
/// whole pile rather than the pre-matched subset.
pub fn combo_pile_cardset(from: &CardSet) -> CardSet {
    match from {
        CardSet::Group {
            group: Group::Combo { groupable, .. },
        } => CardSet::Group {
            group: Group::Groupable {
                groupable: groupable.clone(),
            },
        },
        CardSet::GroupOwner {
            group: Group::Combo { groupable, .. },
            owner,
        } => CardSet::GroupOwner {
            group: Group::Groupable {
                groupable: groupable.clone(),
            },
            owner: owner.clone(),
        },
        _ => from.clone(),
    }
}

/// Detect the quantifier site on `edge`, in precedence order
/// `DestPlayerAll` > `DestPlayerAny` > `SourcePlayerAny` > `ComboSource` >
/// `SrcCardsAnyOrRange`.
///
/// If `to` is a `CardSet::GroupOwner` whose `owner` is
/// `Owner::PlayerCollection { PlayerCollection::Aggregate { Quantifier::All } }`
/// → `DestPlayerAll`; `Quantifier::Any` → `DestPlayerAny`. These short-circuit
/// so that a combined `All`-of-`Any` edge (e.g. `deal any from Stock to Hand
/// of all`) scans as `DestPlayerAll` and the card choice is handled by the
/// resume branch via [`card_site`].
///
/// Otherwise, if the move's `from` is owned by `any` → `SourcePlayerAny`
/// (resolved before the combo/quantity sites: a multi-player owner cannot be
/// evaluated, so the player must be substituted first — and this ordering is
/// what lets chained sites resolve sequentially, see
/// `Interpreter::quantify_or_dispatch`).
///
/// Otherwise, if the move's `from` is a combo group → `ComboSource` (lay-down
/// with validation). Otherwise, if the edge is a `MoveQuantity` whose
/// `quantity` is `Quantifier::Any` or `IntRange` → `SrcCardsAnyOrRange`.
pub fn scan_edge(edge: &Edge<LoweredPayLoad>) -> QuantSite {
    let Some(mcs) = move_cardset_ref(edge) else {
        return QuantSite::None;
    };

    // Destination quantifier (highest precedence).
    let dest = dest_site_for(mcs_to_ref(mcs));
    if dest != QuantSite::None {
        return dest;
    }

    // Source-player quantifier: the `from` is owned by `any` — prompt the
    // player, then substitute. Resolved *before* the combo/quantity sites: a
    // multi-player owner cannot be evaluated, so the card choice would fail
    // if the source were still unresolved.
    let from = match mcs {
        front_end::ast::MoveCardSet::Move { from, .. } => from,
        front_end::ast::MoveCardSet::MoveQuantity { from, .. } => from,
    };
    if let Some(pc) = source_any_site(from) {
        return QuantSite::SourcePlayerAny { pc };
    }

    // Combo-source (lay-down) — takes precedence over the card-amount site
    // so the chosen set is validated against the combo filter.
    if let Some((combo, from)) = combo_source(mcs) {
        return QuantSite::ComboSource { combo, from };
    }

    // Card-amount quantifier (only MoveQuantity carries a quantity).
    if let front_end::ast::MoveCardSet::MoveQuantity { quantity, from, .. } = mcs {
        if let Some(site) = quantity_site(edge, quantity, from) {
            return site;
        }
    }

    QuantSite::None
}

/// True if the edge's move verb is `deal`. Since 2026-08-10 the verb carries
/// the quantity semantics: `deal` = automatic (from the top), `move`/
/// `exchange` = the player picks the cards (2026-08-10 verb semantics).
pub fn is_deal_move(edge: &Edge<LoweredPayLoad>) -> bool {
    matches!(
        &edge.payload,
        Payload::Action(front_end::ast::GameRule::Action {
            action: front_end::ast::ActionRule::Move {
                move_type: front_end::ast::MoveType::Deal { .. },
            },
        })
    )
}

/// True if a move `from` cardset is *positional* — a `CardPosition` group
/// (`top(X)`, `bottom(X)`, `X[N]`, extrema). Positional sources are always
/// automatic: the position already determined the cards, so no quantity
/// prompt applies (2026-08-10 verb semantics).
fn is_positional(from: &CardSet) -> bool {
    matches!(
        from,
        CardSet::Group {
            group: Group::CardPosition { .. },
        } | CardSet::GroupOwner {
            group: Group::CardPosition { .. },
            ..
        }
    )
}

/// True if `from` is the quantifier's own synthetic-memory slot. After a
/// card-choice resume substitutes `CardSet::Memory` for `from`, the re-scan
/// must not re-issue a pick-exactly-N prompt for the already-chosen set.
fn is_synth_memory(from: &CardSet) -> bool {
    matches!(
        from,
        CardSet::Memory {
            memory: UseMemory::WithOwner { memory, .. }
        } if memory == SYNTH_MEMORY_KEY
    )
}

/// Classify a `MoveQuantity`'s quantity into a source card-amount site,
/// honouring the 2026-08-10 verb semantics:
///
/// - positional `from` → automatic (no site);
/// - `deal` + `any`/`IntRange` → [`QuantSite::DealCount`] (prompt the count);
/// - `deal` + `Int` → automatic (top N, unchanged);
/// - `move`/`exchange` + `any`/`IntRange` → [`QuantSite::SrcCardsAnyOrRange`]
///   (pick the cards, unchanged);
/// - `move`/`exchange` + `Int` → [`QuantSite::SrcCardsExactN`] (pick exactly N);
/// - `move`/`exchange` + `Int` over the synthetic memory slot → automatic
///   (the cards were already chosen).
fn quantity_site(
    edge: &Edge<LoweredPayLoad>,
    quantity: &Quantity,
    from: &CardSet,
) -> Option<QuantSite> {
    if is_positional(from) {
        return None;
    }
    match quantity {
        Quantity::Quantifier {
            quantifier: Quantifier::All,
        } => None, // `move all from X` — automatic, all cards (unchanged)
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        }
        | Quantity::IntRange { .. } => {
            if is_deal_move(edge) {
                Some(QuantSite::DealCount {
                    qty: quantity.clone(),
                    from: from.clone(),
                })
            } else {
                Some(QuantSite::SrcCardsAnyOrRange {
                    qty: quantity.clone(),
                    from: from.clone(),
                })
            }
        }
        Quantity::Int { .. } => {
            if is_deal_move(edge) || is_synth_memory(from) {
                None
            } else {
                Some(QuantSite::SrcCardsExactN {
                    qty: quantity.clone(),
                    from: from.clone(),
                })
            }
        }
    }
}

/// The `(quantity, from)` form of [`quantity_site`], for the `DestPlayerAll`
/// arm: a source card-amount site chained with a dest fan-out (e.g.
/// `move 1 from Hand of current to Table of all` prompts for the card(s)
/// first, then fans out — 2026-08-10).
pub fn src_card_choice_site(edge: &Edge<LoweredPayLoad>) -> Option<(Quantity, CardSet)> {
    let mcs = move_cardset_ref(edge)?;
    let front_end::ast::MoveCardSet::MoveQuantity { quantity, from, .. } = mcs else {
        return None;
    };
    quantity_site(edge, quantity, from).map(|site| match site {
        QuantSite::DealCount { qty, from }
        | QuantSite::SrcCardsAnyOrRange { qty, from }
        | QuantSite::SrcCardsExactN { qty, from } => (qty, from),
        _ => unreachable!("quantity_site only yields card-amount sites"),
    })
}

/// The `SourcePlayerAny` site of `edge`, if its `from` is owned by `any`.
/// Used by the `DestPlayerAll` arm to resolve a chained source player *before*
/// fanning out (the per-player replacement edges must carry a concrete owner).
pub fn edge_source_any(edge: &Edge<LoweredPayLoad>) -> Option<PlayerCollection> {
    let mcs = move_cardset_ref(edge)?;
    let from = match mcs {
        front_end::ast::MoveCardSet::Move { from, .. } => from,
        front_end::ast::MoveCardSet::MoveQuantity { from, .. } => from,
    };
    source_any_site(from)
}

/// Clone `edge` and replace every `Quantifier::Any` in its setup rule with the
/// chosen player's literal — the setup-`Any` counterpart of the move
/// substitutions (I-20, relaxed 2026-08-10). Any-sites can appear as an
/// `Owner` (`location X on any`, `memory M on any`) or as a player collection
/// (`team T1 with any`, `turnorder any`).
pub fn substitute_setup_any(
    edge: &Edge<LoweredPayLoad>,
    player_name: String,
) -> Edge<LoweredPayLoad> {
    let mut repl = edge.clone();
    if let Payload::Action(front_end::ast::GameRule::SetUp { setup }) = &mut repl.payload {
        substitute_setup_any_in_rule(setup, &player_name);
    }
    repl
}

fn substitute_setup_any_in_rule(setup: &mut SetUpRule, name: &str) {
    match setup {
        SetUpRule::CreateTeams { teams } => {
            for (_, pc) in teams {
                if is_any_player_collection(pc) {
                    *pc = PlayerCollection::Literal {
                        players: vec![PlayerExpr::Literal {
                            name: name.to_string(),
                        }],
                    };
                }
            }
        }
        SetUpRule::CreateTurnorder { player_collection }
        | SetUpRule::CreateTurnorderRandom { player_collection } => {
            if is_any_player_collection(player_collection) {
                *player_collection = PlayerCollection::Literal {
                    players: vec![PlayerExpr::Literal {
                        name: name.to_string(),
                    }],
                };
            }
        }
        SetUpRule::CreateLocation { owner, .. } => substitute_any_owner(owner, name),
        SetUpRule::CreateMemory { owner, .. }
        | SetUpRule::CreateMemoryWithMemoryType { owner, .. } => substitute_any_owner(owner, name),
        _ => {}
    }
}

fn is_any_player_collection(pc: &PlayerCollection) -> bool {
    matches!(
        pc,
        PlayerCollection::Aggregate {
            aggregate: AggregatePlayerCollection::Quantifier {
                quantifier: Quantifier::Any
            }
        }
    )
}

fn substitute_any_owner(owner: &mut Owner, name: &str) {
    if let Owner::PlayerCollection {
        player_collection: pc,
    } = owner
    {
        if is_any_player_collection(pc) {
            *owner = Owner::Player {
                player: PlayerExpr::Literal {
                    name: name.to_string(),
                },
            };
        }
    }
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

/// Clone `edge` and replace the `from` card-set's owner with a concrete
/// `Owner::Player { PlayerExpr::Literal { player_name } }` — the
/// `SourcePlayerAny` counterpart of [`substitute_dest_player`]. If the edge's
/// `from` is not a `GroupOwner` (defensive), the edge is returned unchanged.
pub fn substitute_source_player(
    edge: &Edge<LoweredPayLoad>,
    player_name: String,
) -> Edge<LoweredPayLoad> {
    let mut repl = edge.clone();
    if let Some(mcs) = move_cardset_mut(&mut repl) {
        let from = match mcs {
            front_end::ast::MoveCardSet::Move { from, .. } => from,
            front_end::ast::MoveCardSet::MoveQuantity { from, .. } => from,
        };
        if let CardSet::GroupOwner { owner, .. } = from {
            *owner = Owner::Player {
                player: PlayerExpr::Literal { name: player_name },
            };
        }
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
            memory: UseMemory::WithOwner {
                memory: SYNTH_MEMORY_KEY.to_string(),
                owner: Box::new(Owner::Table),
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

/// Clone `edge` and replace the `MoveQuantity`'s quantity with the literal
/// `value` (the `DealCount` resume: the player chose how many cards to deal).
/// Non-`MoveQuantity` edges are returned unchanged (defensive).
pub fn substitute_quantity(edge: &Edge<LoweredPayLoad>, value: i32) -> Edge<LoweredPayLoad> {
    let mut repl = edge.clone();
    if let Some(front_end::ast::MoveCardSet::MoveQuantity { quantity, .. }) =
        move_cardset_mut(&mut repl)
    {
        *quantity = Quantity::Int {
            int: IntExpr::Literal { int: value },
        };
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
) -> Result<Vec<(StateID, Edge<LoweredPayLoad>)>, EngineError> {
    let n = player_names.len();
    if n > FANOUT_CAP {
        return Err(EngineError::DestPlayerFanoutExceedsCap { n, cap: FANOUT_CAP });
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
/// result holds, `Err(EngineError)` otherwise.
///
/// Non-literal `IntExpr`s (memory/runtime-backed) cannot be evaluated without
/// live `GameData`; per the plan's edge-case note, we fall back to "accept any
/// count in `[0, available]`" in that case.
pub fn validate_int_range(
    range: &IntRange,
    count: usize,
    available: usize,
) -> Result<(), EngineError> {
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
        Err(EngineError::SelectionDoesNotSatisfyRange {
            count,
            range: Box::new(range.clone()),
        })
    }
}

/// Fallback when an `IntRange` constraint uses a non-literal `IntExpr`: accept
/// any count that does not exceed `available`.
fn validate_fallback(count: usize, available: usize) -> Result<(), EngineError> {
    if count <= available {
        Ok(())
    } else {
        Err(EngineError::SelectionExceedsAvailable { count, available })
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
) -> Result<Vec<(StateID, Edge<LoweredPayLoad>)>, EngineError> {
    let n = player_names.len();
    if n > FANOUT_CAP {
        return Err(EngineError::DestPlayerFanoutExceedsCap { n, cap: FANOUT_CAP });
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
#[path = "quantifier_tests.rs"]
mod tests;
