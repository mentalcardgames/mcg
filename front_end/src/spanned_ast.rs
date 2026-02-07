use crate::spans::*;
use crate::walker::*;
use crate::lower::*;
use proc_generation::*;

// Operator
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, Clone, Walker, Lower)]
pub enum BinCompare {
    Eq,
    Neq,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum LogicBinOp {
    And,
    Or,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum IntOp {
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum IntCompare {
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le,
}

// ===========================================================================
// ===========================================================================
// ===========================================================================

// Utility
// ===========================================================================
// ===========================================================================
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum Extrema {
    Min,
    Max,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum OutOf {
    CurrentStage,
    Stage(SID),
    Game,
    Play,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum Groupable {
    Location(SID),
    LocationCollection(SLocationCollection),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum Owner {
    Player(SPlayerExpr),
    PlayerCollection(SPlayerCollection),
    Team(STeamExpr),
    TeamCollection(STeamCollection),
    Table,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum Quantity {
    Int(SIntExpr),
    Quantifier(SQuantifier),
    IntRange(SIntRange),
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct IntRange {
    pub op_int: Vec<(SIntCompare, SIntExpr)>,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum Quantifier {
    All,
    Any,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum EndCondition {
    UntilBool(SBoolExpr),
    UntilBoolRep(SBoolExpr, SLogicBinOp, SRepititions),
    UntilRep(SRepititions),
    UntilEnd,
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct Repititions {
    pub times: SIntExpr,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum MemoryType {
    Int(SIntExpr),
    String(SStringExpr),
    CardSet(SCardSet),
    Collection(SCollection),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum Players {
    Player(SPlayerExpr),
    PlayerCollection(SPlayerCollection),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum EndType {
    Turn,
    Stage,
    GameWithWinner(SPlayers),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum DemandType {
    CardPosition(SCardPosition),
    String(SStringExpr),
    Int(SIntExpr),
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct Types {
    pub types: Vec<(SID, Vec<SID>)>,
}

// ===========================================================================
// ===========================================================================
// ===========================================================================

// Base Types
// ===========================================================================
// ===========================================================================
// ===========================================================================

// Player
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum RuntimePlayer {
    Current,
    Next,
    Previous,
    Competitor,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum QueryPlayer {
    Turnorder(SIntExpr),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum AggregatePlayer {
    OwnerOfCardPostion(Box<SCardPosition>),
    OwnerOfMemory(SExtrema, SID),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum PlayerExpr {
    Literal(SID),
    Runtime(SRuntimePlayer),
    Aggregate(SAggregatePlayer),
    Query(SQueryPlayer),
}
// ===========================================================================

// IntExpr
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum QueryInt {
    IntCollectionAt(Box<SIntCollection>, Box<SIntExpr>),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum AggregateInt {
    SizeOf(SCollection),
    SumOfIntCollection(SIntCollection),
    SumOfCardSet(Box<SCardSet>, SID),
    ExtremaCardset(SExtrema, Box<SCardSet>, SID),
    ExtremaIntCollection(SExtrema, SIntCollection),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum RuntimeInt {
    StageRoundCounter,
    PlayRoundCounter,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum IntExpr {
    Literal(i32),
    Binary(Box<SIntExpr>, SIntOp, Box<SIntExpr>),
    Query(SQueryInt),
    Aggregate(SAggregateInt),
    Runtime(SRuntimeInt),
}
// ===========================================================================

// String
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum QueryString {
    KeyOf(SID, SCardPosition),
    StringCollectionAt(SStringCollection, SIntExpr),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum StringExpr {
    Literal(SID),
    Query(SQueryString),
}
// ===========================================================================

// Bool
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum CardSetCompare {
    Eq,
    Neq,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum StringCompare {
    Eq,
    Neq,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum PlayerCompare {
    Eq,
    Neq,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum TeamCompare {
    Eq,
    Neq,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum BoolOp {
    And,
    Or,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum CompareBool {
    Int(SIntExpr, SIntCompare, SIntExpr),
    CardSet(SCardSet, SCardSetCompare, SCardSet),
    String(SStringExpr, SStringCompare, SStringExpr),
    Player(SPlayerExpr, SPlayerCompare, SPlayerExpr),
    Team(STeamExpr, STeamCompare, STeamExpr),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum AggregateBool {
    Compare(SCompareBool),
    CardSetEmpty(SCardSet),
    CardSetNotEmpty(SCardSet),
    OutOfPlayer(SPlayers, SOutOf),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum BoolExpr {
    Binary(Box<SBoolExpr>, SBoolOp, Box<SBoolExpr>),
    Unary(SUnaryOp, Box<SBoolExpr>),
    Aggregate(SAggregateBool),
}
// ===========================================================================

// Team
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum AggregateTeam {
    TeamOf(SPlayerExpr),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum TeamExpr {
    Literal(SID),
    Aggregate(SAggregateTeam),
}
// ===========================================================================

// CardPosition
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum QueryCardPosition {
    At(SID, SIntExpr),
    Top(SID),
    Bottom(SID),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum AggregateCardPosition {
    ExtremaPointMap(SExtrema, Box<SCardSet>, SID),
    ExtremaPrecedence(SExtrema, Box<SCardSet>, SID),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum CardPosition {
    Query(SQueryCardPosition),
    Aggregate(SAggregateCardPosition),
}

// Stauts
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum Status {
    FaceUp,
    FaceDown,
    Private,
}
// ===========================================================================

// ===========================================================================
// ===========================================================================
// ===========================================================================

// Collections
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, Clone, Walker, Lower)]
pub enum Collection {
    IntCollection(SIntCollection),
    StringCollection(SStringCollection),
    LocationCollection(SLocationCollection),
    PlayerCollection(SPlayerCollection),
    TeamCollection(STeamCollection),
    CardSet(Box<SCardSet>),
    Ambiguous(Vec<SID>),
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct IntCollection {
    pub ints: Vec<SIntExpr>,
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct StringCollection {
    pub strings: Vec<SStringExpr>,
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct LocationCollection {
    pub locations: Vec<SID>,
}

// PlayerCollection
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum RuntimePlayerCollection {
    PlayersOut,
    PlayersIn,
    Others,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum AggregatePlayerCollection {
    Quantifier(SQuantifier),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum PlayerCollection {
    Literal(Vec<SPlayerExpr>),
    Aggregate(SAggregatePlayerCollection),
    Runtime(SRuntimePlayerCollection),
}
// ===========================================================================

// TeamCollection
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum RuntimeTeamCollection {
    OtherTeams,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum TeamCollection {
    Literal(Vec<STeamExpr>),
    Runtime(SRuntimeTeamCollection),
}

// ===========================================================================
// ===========================================================================
// ===========================================================================

// CardSet
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, Clone, Walker, Lower)]
pub enum CardSet {
    Group(SGroup),
    GroupOwner(SGroup, SOwner),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum Group {
    Groupable(SGroupable),
    Where(SGroupable, SFilterExpr),
    NotCombo(SID, SGroupable),
    Combo(SID, SGroupable),
    CardPosition(SCardPosition),
}

// FilterExpr
// ===========================================================================
#[derive(Debug, Clone, Walker, Lower)]
pub enum AggregateFilter {
    Size(SIntCompare, Box<SIntExpr>),
    Same(SID),
    Distinct(SID),
    Adjacent(SID, SID),
    Higher(SID, SID),
    Lower(SID, SID),
    KeyString(SID, SStringCompare, Box<SStringExpr>),
    Combo(SID),
    NotCombo(SID),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum FilterOp {
    And,
    Or,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum FilterExpr {
    Aggregate(SAggregateFilter),
    Binary(Box<SFilterExpr>, SFilterOp, Box<SFilterExpr>),
}
// ===========================================================================

// ===========================================================================
// ===========================================================================
// ===========================================================================

// Game + Stage + FlowComponent + Rule
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, Clone, Walker, Lower)]
pub struct Game {
    pub flows: Vec<SFlowComponent>,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum FlowComponent {
    Stage(SSeqStage),
    Rule(SGameRule),
    IfRule(SIfRule),
    ChoiceRule(SChoiceRule),
    OptionalRule(SOptionalRule),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum SetUpRule {
    // Creations
    CreatePlayer(Vec<SID>),
    CreateTeams(Vec<(SID, SPlayerCollection)>),
    CreateTurnorder(SPlayerCollection),
    CreateTurnorderRandom(SPlayerCollection),
    CreateLocation(Vec<SID>, SOwner),
    CreateCardOnLocation(SID, STypes),
    CreateTokenOnLocation(SIntExpr, SID, SID),
    CreateCombo(SID, SFilterExpr),
    CreateMemory(SID, SMemoryType, SOwner),
    CreatePrecedence(SID, Vec<(SID, SID)>),
    CreatePointMap(SID, Vec<(SID, SID, SIntExpr)>),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum ActionRule {
    // Actions
    FlipAction(SCardSet, SStatus),
    ShuffleAction(SCardSet),
    PlayerOutOfStageAction(SPlayers),
    PlayerOutOfGameSuccAction(SPlayers),
    PlayerOutOfGameFailAction(SPlayers),
    SetMemory(SID, SMemoryType),
    ResetMemory(SID),
    CycleAction(SPlayerExpr),
    BidAction(SQuantity),
    BidMemoryAction(SID, SQuantity),
    EndAction(SEndType),
    DemandAction(SDemandType),
    DemandMemoryAction(SDemandType, SID),
    Move(SMoveType),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum ScoringRule {
    // Score + Winner Rule
    ScoreRule(SScoreRule),
    WinnerRule(SWinnerRule),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum GameRule {
    SetUp(SSetUpRule),
    Action(SActionRule),
    Scoring(SScoringRule),
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct SeqStage {
    pub stage: SID,
    pub player: SPlayerExpr,
    pub end_condition: SEndCondition,
    pub flows: Vec<SFlowComponent>,
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct IfRule {
    pub condition: SBoolExpr,
    pub flows: Vec<SFlowComponent>,
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct OptionalRule {
    pub flows: Vec<SFlowComponent>,
}

#[derive(Debug, Clone, Walker, Lower)]
pub struct ChoiceRule {
    pub options: Vec<SFlowComponent>,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum MoveType {
    Deal(SDealMove),
    Exchange(SExchangeMove),
    Classic(SClassicMove),
    Place(STokenMove),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum MoveCardSet {
    Move(SCardSet, SStatus, SCardSet),
    MoveQuantity(SQuantity, SCardSet, SStatus, SCardSet),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum ClassicMove {
    MoveCardSet(SMoveCardSet),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum DealMove {
    MoveCardSet(SMoveCardSet),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum ExchangeMove {
    MoveCardSet(SMoveCardSet),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum TokenMove {
    Place(SID, STokenLocExpr, STokenLocExpr),
    PlaceQuantity(SQuantity, SID, STokenLocExpr, STokenLocExpr),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum TokenLocExpr {
    Groupable(SGroupable),
    GroupablePlayers(SGroupable, SPlayers),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum ScoreRule {
    Score(SIntExpr, SPlayers),
    ScoreMemory(SIntExpr, SID, SPlayers),
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum WinnerType {
    Score,
    Memory(SID),
    Position,
}

#[derive(Debug, Clone, Walker, Lower)]
pub enum WinnerRule {
    Winner(SPlayers),
    WinnerWith(SExtrema, SWinnerType),
}

// ===========================================================================
// ===========================================================================
// ===========================================================================
