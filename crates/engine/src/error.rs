//! The engine's unified error type.
//!
//! Every fallible engine operation returns `Result<_, EngineError>`:
//! evaluator reads, action mutations, interpreter steps, quantifier
//! validation, and controller input handling. The only exceptions are the
//! intentional internal-invariant panics documented in
//! `docs/error-handling.md` §2 (none reachable from well-formed DSL input).
//!
//! The `Display` messages are the public diagnostic surface — they appear in
//! TUI error panels, `cgdsl-play` output, trace-log footers, and re-prompt
//! messages (e.g. the `IntRange` re-prompt path builds its prompt from
//! [`SelectionDoesNotSatisfyRange`]'s message). They are stable strings; do
//! not change them without updating `docs/error-handling.md` §1 and the
//! tests that assert them.
//!
//! For coarse programmatic handling without matching every variant, use
//! [`EngineError::kind`] / [`ErrorKind`].

use front_end::ast::{CardSet, IntExpr, IntRange, Owner, PlayerExpr};

/// Coarse classification of an [`EngineError`], for hosts that want to group
/// or handle errors without matching every variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Evaluator failures (`crates::engine::query`) — missing state, type
    /// mismatches, out-of-range reads, etc.
    Query,
    /// Action-mutation failures (`crates::engine::action`).
    Action,
    /// FSM/step failures (`crates::engine::interpreter`).
    Interpreter,
    /// Quantifier fan-out / selection-validation failures.
    Quantifier,
    /// Controller / test-input failures.
    Input,
    /// An internal-invariant panic caught and converted
    /// (`EngineError::InternalPanic`).
    Internal,
}

/// The single error type for the whole engine crate.
///
/// Variants are grouped by the module that raises them (query / action /
/// interpreter / quantifier / controller). Wrapping variants carry a
/// `source: EngineError` so a failure's origin can be inspected
/// programmatically; the `Display` message embeds the same context the
/// previous string-typed errors did.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    // =====================================================================
    // Query / evaluator errors (`crates::engine::query`)
    // =====================================================================
    /// Integer division by zero.
    #[error("Division by zero")]
    DivisionByZero,

    /// `RuntimePlayer::Current` / `RuntimePlayer::Next` etc. with no current
    /// player set.
    #[error("No current player")]
    NoCurrentPlayer,

    /// `RuntimeInt::CurrentStageRoundCounter` / `RuntimePlayer::Next` etc.
    /// with no stage on the stage stack.
    #[error("No current stage")]
    NoCurrentStage,

    /// `RuntimePlayer::Next` with no eligible *other* player (I-13).
    #[error("No next player available")]
    NoNextPlayerAvailable,

    /// `RuntimePlayer::Previous` with an empty `turn_order`.
    #[error("Previous player not found")]
    PreviousPlayerNotFound,

    /// `RuntimePlayer::Competitor` with no teammate to return.
    #[error("No competitor found")]
    NoCompetitorFound,

    /// `AggregatePlayer::OwnerOfCardPostion` — card not in any owned location.
    #[error("Owner of card position not found")]
    CardOwnerNotFound,

    /// `AggregatePlayer::OwnerOfMemory` — no player holds a score for the
    /// memory slot.
    #[error("No player found for OwnerOfMemory")]
    OwnerOfMemoryNoPlayer,

    /// `PlayerExpr::Memory` — a `PlayerCollection` slot with no entries.
    #[error("PlayerCollection memory is empty")]
    EmptyPlayerCollectionMemory,

    /// `AggregateTeam::TeamOf` — the player is not a member of any team.
    #[error("Player {name} not found in any team")]
    PlayerNotInAnyTeam { name: String },

    /// A memory slot (owner-prefixed key) does not exist.
    #[error("Memory {key} not found")]
    MemoryNotFound { key: String },

    /// A memory slot holds a different type than the reader expects.
    #[error("Memory value is not an Int")]
    MemoryNotInt,

    /// As [`MemoryNotInt`], for an aggregate-memory read where the key is
    /// part of the message.
    #[error("Memory value is not an Int ({key})")]
    MemoryNotIntFor { key: String },

    #[error("Memory value is not an IntCollection")]
    MemoryNotIntCollection,

    #[error("Memory value is not a LocationCollection")]
    MemoryNotLocationCollection,

    #[error("Memory value is not a Team")]
    MemoryNotTeam,

    #[error("Memory value is not a Team ({key})")]
    MemoryNotTeamFor { key: String },

    #[error("Memory value is not a CardSet")]
    MemoryNotCardSet,

    #[error("Memory value is not a String")]
    MemoryNotString,

    #[error("Memory value is not a String ({key})")]
    MemoryNotStringFor { key: String },

    #[error("Memory value is not a StringCollection")]
    MemoryNotStringCollection,

    #[error("Memory value is not a PlayerCollection")]
    MemoryNotPlayerCollection,

    #[error("Memory value is not a PlayerCollection ({key})")]
    MemoryNotPlayerCollectionFor { key: String },

    /// A `PlayerExpr::Memory` slot holding something other than a player
    /// name / player collection.
    #[error("Memory value is not a valid player")]
    MemoryNotValidPlayer,

    /// A memory reference without an explicit `of <owner>` clause.
    #[error("memory access requires an explicit owner; use &M:{key} of <owner>")]
    MemoryRequiresExplicitOwner { key: String },

    /// `QueryInt::IntCollectionAt` index out of bounds.
    #[error("No int at index {idx}")]
    IntCollectionAtOutOfRange { idx: usize },

    /// `QueryString::StringCollectionAt` index out of bounds.
    #[error("No string at index {idx}")]
    StringCollectionAtOutOfRange { idx: usize },

    /// A named `PointMap` is not defined in `GameData`.
    #[error("PointMap {name} not found")]
    PointMapNotFound { name: String },

    /// `AggregateInt::ExtremaCardset` over an empty set.
    #[error("No card found for extrema")]
    NoCardForExtrema,

    /// `AggregateInt::ExtremaIntCollection` over an empty collection.
    #[error("No value found in IntCollection")]
    NoValueInIntCollection,

    /// `QueryPlayer::Turnorder` index out of bounds.
    #[error("No player at turn order index {idx}")]
    TurnOrderIndexOutOfRange { idx: usize },

    /// A player slot resolved by index does not exist.
    #[error("Player at index {idx} not found")]
    PlayerIndexNotFound { idx: usize },

    /// `QueryPlayer::CollectionAt` index out of bounds.
    #[error("No player at index {idx} in player collection")]
    PlayerCollectionAtOutOfRange { idx: usize },

    /// A player slot resolved from a collection does not exist.
    #[error("Player at collection index {idx} not found")]
    PlayerCollectionIndexNotFound { idx: usize },

    /// `Players::Player` names a player that is not in `GameData::players`.
    #[error("resolve_players: player {name} not found in game_data")]
    ResolvePlayersPlayerNotFound { name: String },

    /// A literal entry of a `PlayerCollection` is not in `GameData::players`.
    #[error("resolve_player_collection: player {name} not found in game_data")]
    ResolvePlayerCollectionPlayerNotFound { name: String },

    /// `resolve_owner_to_name` on a multi-player owner (cannot yield one name).
    #[error("resolve_owner_to_name: PlayerCollection cannot resolve to a single name")]
    OwnerNameFromPlayerCollection,

    /// `resolve_owner_to_name` on a multi-team owner (cannot yield one name).
    #[error("resolve_owner_to_name: TeamCollection cannot resolve to a single name")]
    OwnerNameFromTeamCollection,

    /// `resolve_owner_to_names` on a team owner — team-owned locations and
    /// memories are not in the data model.
    #[error(
        "resolve_owner_to_names: team '{name}' cannot own a location or memory (team-owned locations are not in the data model)"
    )]
    TeamCannotOwn { name: String },

    /// `resolve_owner_to_names` on a multi-team owner.
    #[error("resolve_owner_to_names: TeamCollection cannot resolve to owner names")]
    OwnerNamesFromTeamCollection,

    /// `QueryString::KeyOf` — the card id does not exist.
    #[error("Card {card_id} not found")]
    CardNotFound { card_id: usize },

    /// `QueryString::KeyOf` — the card lacks the requested attribute key.
    #[error("Key {key} not found in card {card_id}")]
    CardKeyNotFound { key: String, card_id: usize },

    /// A groupable location with an explicit owner does not exist for that
    /// owner.
    #[error("Location {name} not found for owner {owner}")]
    LocationNotFoundForOwner { name: String, owner: String },

    /// A groupable location does not exist.
    #[error("Location {name} not found")]
    LocationNotFound { name: String },

    /// A `CardPosition::Query` references a location that does not exist.
    #[error("Location '{name}' not found for card position")]
    LocationNotFoundForCardPosition { name: String },

    /// A `CardPosition::Aggregate` card cannot be found in any location.
    #[error("Card position not found in any location")]
    CardPositionNotFound,

    /// A named `Precedence` is not defined in `GameData`.
    #[error("Precedence {name} not found")]
    PrecedenceNotFound { name: String },

    /// A `Higher`/`Lower` filter value is not in the precedence's value list.
    #[error("Value {value} not found in precedence {precedence}")]
    ValueNotFoundInPrecedence { value: String, precedence: String },

    /// A named `Combo` is not defined in `GameData`.
    #[error("Combo {name} not found")]
    ComboNotFound { name: String },

    /// `QueryCardPosition::At` — no card at the requested index.
    #[error("No card at index {idx} in location {location}")]
    CardAtIndexNotFound { idx: usize, location: String },

    /// `QueryCardPosition::Top` — the location has no cards.
    #[error("No card at top of location {location}")]
    CardAtTopNotFound { location: String },

    /// `QueryCardPosition::Bottom` — the location has no cards.
    #[error("No card at bottom of location {location}")]
    CardAtBottomNotFound { location: String },

    /// `AggregateCardPosition::ExtremaPointMap` over an empty set.
    #[error("No card found for ExtremaPointMap")]
    NoCardForExtremaPointMap,

    /// `AggregateCardPosition::ExtremaPrecedence` over an empty set.
    #[error("No card found for ExtremaPrecedence")]
    NoCardForExtremaPrecedence,

    // =====================================================================
    // Action errors (`crates::engine::action`)
    // =====================================================================
    /// `CreateLocation` — the `owner` could not be resolved to names.
    #[error("CreateLocation: failed to resolve owner {owner:?}: {source}")]
    CreateLocationOwnerResolution {
        owner: Box<Owner>,
        source: Box<EngineError>,
    },

    /// `CreateCardOnLocation` — the target location does not exist.
    #[error("CreateCardOnLocation: location {location:?} not found")]
    CreateCardOnLocationLocationNotFound { location: String },

    /// `CreateMemory` — the `owner` could not be resolved to names.
    #[error("CreateMemory: failed to resolve owner {owner:?}: {source}")]
    CreateMemoryOwnerResolution {
        owner: Box<Owner>,
        source: Box<EngineError>,
    },

    /// `CreateMemoryWithMemoryType` — the `owner` could not be resolved.
    #[error("CreateMemoryWithMemoryType: failed to resolve owner {owner:?}: {source}")]
    CreateMemoryWithTypeOwnerResolution {
        owner: Box<Owner>,
        source: Box<EngineError>,
    },

    /// `CreatePointMap` — a points expression could not be evaluated.
    #[error("CreatePointMap: failed to eval int {int_expr:?} for key {key}:{value}: {source}")]
    CreatePointMapIntEval {
        int_expr: Box<IntExpr>,
        key: String,
        value: String,
        source: Box<EngineError>,
    },

    /// `ShuffleAction` — the cardset could not be evaluated.
    #[error("ShuffleAction failed: {source}")]
    ShuffleActionEval { source: Box<EngineError> },

    /// `SetMemory` — the `Int` value expression failed to evaluate.
    #[error("SetMemory Int eval failed: {source}")]
    SetMemoryIntEval { source: Box<EngineError> },

    /// `SetMemory` — the `String` value expression failed to evaluate.
    #[error("SetMemory String eval failed: {source}")]
    SetMemoryStringEval { source: Box<EngineError> },

    /// `SetMemory` — the `Player` value expression failed to evaluate.
    #[error("SetMemory Player eval failed: {source}")]
    SetMemoryPlayerEval { source: Box<EngineError> },

    /// `SetMemory` — the `Team` value expression failed to evaluate.
    #[error("SetMemory Team eval failed: {source}")]
    SetMemoryTeamEval { source: Box<EngineError> },

    /// `SetMemory` — no current player to prefix the memory key with
    /// (the grammar has no owner clause; see `action.rs`).
    #[error("SetMemory requires a current player")]
    SetMemoryNoCurrentPlayer,

    /// `ResetMemory` — no current player to prefix the memory key with.
    #[error("ResetMemory requires a current player")]
    ResetMemoryNoCurrentPlayer,

    /// `CycleAction` — the player expression failed to evaluate.
    #[error("CycleAction: failed to eval player {player:?}: {source}")]
    CycleActionPlayerEval {
        player: Box<PlayerExpr>,
        source: Box<EngineError>,
    },

    /// `CycleAction` — the resolved player is not in `GameData::players`.
    #[error("CycleAction: player {name} not found in game_data.players")]
    CycleActionPlayerNotFound { name: String },

    /// `CycleAction` — the player index is not in `turn_order`.
    #[error("CycleAction: player_idx {player_idx} not in turn_order {turn_order:?}")]
    CycleActionTurnOrderNotFound {
        player_idx: usize,
        turn_order: Vec<usize>,
    },

    /// `ScoreRule::Score` — the value expression failed to evaluate.
    #[error("Score: failed to eval int {int_expr:?}: {source}")]
    ScoreIntEval {
        int_expr: Box<IntExpr>,
        source: Box<EngineError>,
    },

    /// `ScoreRule::ScoreMemory` — the value expression failed to evaluate.
    #[error("ScoreMemory: failed to eval int {int_expr:?}: {source}")]
    ScoreMemoryIntEval {
        int_expr: Box<IntExpr>,
        source: Box<EngineError>,
    },

    /// A move's `from` cardset failed to evaluate.
    #[error("execute_cardset_move: failed to eval from cardset {cardset:?}: {source}")]
    MoveFromCardsetEval {
        cardset: Box<CardSet>,
        source: Box<EngineError>,
    },

    /// A move's `to` cardset failed to evaluate.
    #[error("execute_cardset_move: failed to eval dest cardset {cardset:?}: {source}")]
    MoveDestCardsetEval {
        cardset: Box<CardSet>,
        source: Box<EngineError>,
    },

    /// A move's resolved destination location index is out of bounds.
    #[error(
        "execute_cardset_move: dest_loc_idx {dest_loc_idx} >= locations.len() {locations_len} (cardset expr: {cardset:?})"
    )]
    MoveDestLocationOutOfRange {
        dest_loc_idx: usize,
        locations_len: usize,
        cardset: Box<CardSet>,
    },

    // =====================================================================
    // Interpreter errors (`crates::engine::interpreter`)
    // =====================================================================
    /// `step()` on a `StateID` that is absent from the IR.
    #[error("Current state {state} not found in IR")]
    CurrentStateNotFoundInIr { state: u32 },

    /// A state with no outgoing edges that is not the goal state.
    #[error("No outgoing edges from state {state} and not at goal state")]
    NoOutgoingEdges { state: u32 },

    /// A non-goal state with no edges at all.
    #[error("No edges found in state {state}")]
    NoEdgesFound { state: u32 },

    /// A `Condition` state that does not have exactly two outgoing edges
    /// (invariant I-9).
    #[error("Condition state {state} must have exactly 2 edges, found {found}")]
    ConditionEdgeCount { state: u32, found: usize },

    /// An `EndCondition` state that does not have exactly two outgoing edges.
    #[error("EndCondition state {state} must have exactly 2 edges, found {found}")]
    EndConditionEdgeCount { state: u32, found: usize },

    /// A `Condition` state whose `took_else` branch has no edge.
    #[error("Failed to get condition edge")]
    ConditionEdgeMissing,

    /// An `EndCondition` state whose exit branch has no edge.
    #[error("Failed to get end condition edge")]
    EndConditionEdgeMissing,

    /// An `Optional` state received a non-accept/decline input kind.
    #[error("Unexpected input for Optional")]
    UnexpectedInputForOptional,

    // =====================================================================
    // Quantifier errors (`crates::engine::quantifier`, `quant_driver`)
    // =====================================================================
    /// A `DestPlayerAll` fan-out exceeds [`crate::quantifier::FANOUT_CAP`].
    #[error("dest-player fan-out {n} exceeds cap {cap}")]
    DestPlayerFanoutExceedsCap { n: usize, cap: usize },

    /// A `ChooseCards` selection count violates an `IntRange`; the resume
    /// path re-prompts with this message.
    #[error("selected {count} does not satisfy range {range:?}")]
    SelectionDoesNotSatisfyRange { count: usize, range: Box<IntRange> },

    /// `validate_int_range` fallback: the selection exceeds the available
    /// cards (non-literal range expressions are accepted up to `available`).
    #[error("selected {count} exceeds available {available}")]
    SelectionExceedsAvailable { count: usize, available: usize },

    /// Resume of `DestPlayerAny` with an out-of-range candidate index
    /// (invariant I-8).
    #[error("ChoosePlayer idx {idx} out of range ({len})")]
    ChoosePlayerIdxOutOfRange { idx: usize, len: usize },

    /// Resume of `CardsAnyOrRange` / `DestAllThenCards` with a `selected`
    /// entry out of bounds (invariant I-8).
    #[error("ChooseCards index out of range")]
    ChooseCardsIndexOutOfRange,

    // =====================================================================
    // Controller / test-input errors (`crates::engine::controller`)
    // =====================================================================
    /// `InputSource::TestFile` — the file could not be opened.
    #[error("Failed to open test file {path}: {source}")]
    TestFileOpen {
        path: String,
        source: std::io::Error,
    },

    /// `InputSource::TestFile` — a line could not be read.
    #[error("Failed to read test file {path}: {source}")]
    TestFileRead {
        path: String,
        source: std::io::Error,
    },

    /// `InputSource::TestFile` — consumed more inputs than the file holds.
    #[error("Test input file exhausted (input #{input_sequence})")]
    TestInputExhausted { input_sequence: usize },

    /// `InputSource::TestFile` — malformed `p <N>` line.
    #[error("Invalid test input #{input_sequence}: expected 'p <N>', got '{line}'")]
    InvalidTestInputP { input_sequence: usize, line: String },

    /// `InputSource::TestFile` — `p 0` (player indices are 1-based).
    #[error("Invalid test input #{input_sequence}: player indices start at 1, got 0")]
    InvalidTestInputPlayerZero { input_sequence: usize },

    /// `InputSource::TestFile` — malformed `c <csv>` line.
    #[error("Invalid test input #{input_sequence}: expected 'c <csv>', got '{line}'")]
    InvalidTestInputC { input_sequence: usize, line: String },

    /// `InputSource::TestFile` — `c` list containing 0 (card indices are
    /// 1-based).
    #[error("Invalid test input #{input_sequence}: card indices start at 1, got 0")]
    InvalidTestInputCardZero { input_sequence: usize },

    /// `InputSource::TestFile` — a line that is none of `y`/`n`/`p <N>`/
    /// `c <csv>`/`<N>`.
    #[error(
        "Invalid test input #{input_sequence}: expected number, 'y', 'n', 'p <N>', or 'c <csv>', got '{line}'"
    )]
    InvalidTestInputNumber { input_sequence: usize, line: String },

    /// `InputSource::TestFile` — a bare `0` (choice indices are 1-based).
    #[error("Invalid test input #{input_sequence}: choice indices start at 1, got 0")]
    InvalidTestInputChoiceZero { input_sequence: usize },

    /// An internal-invariant panic was caught and converted to an error by
    /// `run_game_with` with `RunOptions::capture_panics(true)`. The remaining
    /// panic sites are unreachable from well-formed DSL input (see
    /// `docs/error-handling.md` §2); this variant lets a host turn an engine
    /// bug into a reportable `Err` instead of a process abort.
    #[error("internal engine panic: {message}")]
    InternalPanic { message: String },
}

impl EngineError {
    /// Coarse classification of the error, mirroring the module that raised
    /// it. Use this for grouping/handling without matching every variant.
    pub fn kind(&self) -> ErrorKind {
        match self {
            EngineError::DivisionByZero
            | EngineError::NoCurrentPlayer
            | EngineError::NoCurrentStage
            | EngineError::NoNextPlayerAvailable
            | EngineError::PreviousPlayerNotFound
            | EngineError::NoCompetitorFound
            | EngineError::CardOwnerNotFound
            | EngineError::OwnerOfMemoryNoPlayer
            | EngineError::EmptyPlayerCollectionMemory
            | EngineError::PlayerNotInAnyTeam { .. }
            | EngineError::MemoryNotFound { .. }
            | EngineError::MemoryNotInt
            | EngineError::MemoryNotIntFor { .. }
            | EngineError::MemoryNotIntCollection
            | EngineError::MemoryNotLocationCollection
            | EngineError::MemoryNotTeam
            | EngineError::MemoryNotTeamFor { .. }
            | EngineError::MemoryNotCardSet
            | EngineError::MemoryNotString
            | EngineError::MemoryNotStringFor { .. }
            | EngineError::MemoryNotStringCollection
            | EngineError::MemoryNotPlayerCollection
            | EngineError::MemoryNotPlayerCollectionFor { .. }
            | EngineError::MemoryNotValidPlayer
            | EngineError::MemoryRequiresExplicitOwner { .. }
            | EngineError::IntCollectionAtOutOfRange { .. }
            | EngineError::StringCollectionAtOutOfRange { .. }
            | EngineError::PointMapNotFound { .. }
            | EngineError::NoCardForExtrema
            | EngineError::NoValueInIntCollection
            | EngineError::TurnOrderIndexOutOfRange { .. }
            | EngineError::PlayerIndexNotFound { .. }
            | EngineError::PlayerCollectionAtOutOfRange { .. }
            | EngineError::PlayerCollectionIndexNotFound { .. }
            | EngineError::ResolvePlayersPlayerNotFound { .. }
            | EngineError::ResolvePlayerCollectionPlayerNotFound { .. }
            | EngineError::OwnerNameFromPlayerCollection
            | EngineError::OwnerNameFromTeamCollection
            | EngineError::TeamCannotOwn { .. }
            | EngineError::OwnerNamesFromTeamCollection
            | EngineError::CardNotFound { .. }
            | EngineError::CardKeyNotFound { .. }
            | EngineError::LocationNotFoundForOwner { .. }
            | EngineError::LocationNotFound { .. }
            | EngineError::LocationNotFoundForCardPosition { .. }
            | EngineError::CardPositionNotFound
            | EngineError::PrecedenceNotFound { .. }
            | EngineError::ValueNotFoundInPrecedence { .. }
            | EngineError::ComboNotFound { .. }
            | EngineError::CardAtIndexNotFound { .. }
            | EngineError::CardAtTopNotFound { .. }
            | EngineError::CardAtBottomNotFound { .. }
            | EngineError::NoCardForExtremaPointMap
            | EngineError::NoCardForExtremaPrecedence => ErrorKind::Query,

            EngineError::CreateLocationOwnerResolution { .. }
            | EngineError::CreateCardOnLocationLocationNotFound { .. }
            | EngineError::CreateMemoryOwnerResolution { .. }
            | EngineError::CreateMemoryWithTypeOwnerResolution { .. }
            | EngineError::CreatePointMapIntEval { .. }
            | EngineError::ShuffleActionEval { .. }
            | EngineError::SetMemoryIntEval { .. }
            | EngineError::SetMemoryStringEval { .. }
            | EngineError::SetMemoryPlayerEval { .. }
            | EngineError::SetMemoryTeamEval { .. }
            | EngineError::SetMemoryNoCurrentPlayer
            | EngineError::ResetMemoryNoCurrentPlayer
            | EngineError::CycleActionPlayerEval { .. }
            | EngineError::CycleActionPlayerNotFound { .. }
            | EngineError::CycleActionTurnOrderNotFound { .. }
            | EngineError::ScoreIntEval { .. }
            | EngineError::ScoreMemoryIntEval { .. }
            | EngineError::MoveFromCardsetEval { .. }
            | EngineError::MoveDestCardsetEval { .. }
            | EngineError::MoveDestLocationOutOfRange { .. } => ErrorKind::Action,

            EngineError::CurrentStateNotFoundInIr { .. }
            | EngineError::NoOutgoingEdges { .. }
            | EngineError::NoEdgesFound { .. }
            | EngineError::ConditionEdgeCount { .. }
            | EngineError::EndConditionEdgeCount { .. }
            | EngineError::ConditionEdgeMissing
            | EngineError::EndConditionEdgeMissing
            | EngineError::UnexpectedInputForOptional => ErrorKind::Interpreter,

            EngineError::DestPlayerFanoutExceedsCap { .. }
            | EngineError::SelectionDoesNotSatisfyRange { .. }
            | EngineError::SelectionExceedsAvailable { .. }
            | EngineError::ChoosePlayerIdxOutOfRange { .. }
            | EngineError::ChooseCardsIndexOutOfRange => ErrorKind::Quantifier,

            EngineError::TestFileOpen { .. }
            | EngineError::TestFileRead { .. }
            | EngineError::TestInputExhausted { .. }
            | EngineError::InvalidTestInputP { .. }
            | EngineError::InvalidTestInputPlayerZero { .. }
            | EngineError::InvalidTestInputC { .. }
            | EngineError::InvalidTestInputCardZero { .. }
            | EngineError::InvalidTestInputNumber { .. }
            | EngineError::InvalidTestInputChoiceZero { .. } => ErrorKind::Input,

            EngineError::InternalPanic { .. } => ErrorKind::Internal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_classifies_representative_variants() {
        assert_eq!(EngineError::DivisionByZero.kind(), ErrorKind::Query);
        assert_eq!(
            EngineError::MemoryNotFound {
                key: "k".to_string()
            }
            .kind(),
            ErrorKind::Query
        );
        assert_eq!(
            EngineError::CycleActionPlayerNotFound {
                name: "x".to_string()
            }
            .kind(),
            ErrorKind::Action
        );
        assert_eq!(
            EngineError::CurrentStateNotFoundInIr { state: 3 }.kind(),
            ErrorKind::Interpreter
        );
        assert_eq!(
            EngineError::ChooseCardsIndexOutOfRange.kind(),
            ErrorKind::Quantifier
        );
        assert_eq!(
            EngineError::TestInputExhausted { input_sequence: 1 }.kind(),
            ErrorKind::Input
        );
        assert_eq!(
            EngineError::InternalPanic {
                message: "boom".to_string()
            }
            .kind(),
            ErrorKind::Internal
        );
    }

    #[test]
    fn every_variant_has_a_kind() {
        // The classifier must be exhaustive; a representative per group is
        // asserted above. This test guards against a variant being added to
        // the enum without a `kind()` arm (compile error if one is missing).
        let samples = [
            EngineError::NoNextPlayerAvailable,
            EngineError::ScoreIntEval {
                int_expr: Box::new(IntExpr::Literal { int: 1 }),
                source: Box::new(EngineError::DivisionByZero),
            },
            EngineError::NoEdgesFound { state: 0 },
            EngineError::SelectionExceedsAvailable {
                count: 5,
                available: 2,
            },
            EngineError::InvalidTestInputC {
                input_sequence: 0,
                line: "x".to_string(),
            },
            EngineError::InternalPanic {
                message: "x".to_string(),
            },
        ];
        for e in samples {
            let _ = e.kind();
        }
    }
}
