use crate::diagnostic::*;


// Operator
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, Clone)]
pub enum BinCompare {
    Eq,
    Neq
}

#[derive(Debug, Clone)]
pub enum LogicBinOp {
    And,
    Or
}

#[derive(Debug, Clone)]
pub enum IntOp {
    Plus,
    Minus,
    Mul,
    Div,
    Mod
}

#[derive(Debug, Clone)]
pub enum IntCompare {
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le
}

// ===========================================================================
// ===========================================================================
// ===========================================================================


// Utility
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, Clone)]
pub enum Extrema {
    Min,
    Max
}


#[derive(Debug, Clone)]
pub enum OutOf {
    CurrentStage,
    Stage(SID),
    Game,
    Play
}

#[derive(Debug, Clone)]
pub enum Groupable {
    Location(SID),
    LocationCollection(SLocationCollection),
}

#[derive(Debug, Clone)]
pub enum Owner {
    Player(SPlayerExpr),
    PlayerCollection(SPlayerCollection),
    Team(STeamExpr),
    TeamCollection(STeamCollection),
    Table
}

#[derive(Debug, Clone)]
pub enum Quantity {
    Int(SIntExpr),
    Quantifier(SQuantifier),
    IntRange(SIntRange),
}

#[derive(Debug, Clone)]
pub struct IntRange { 
    pub op_int: Vec<(SIntCompare, SIntExpr)>
}

#[derive(Debug, Clone)]
pub enum Quantifier {
    All,
    Any
}

#[derive(Debug, Clone)]
pub enum EndCondition {
    UntilBool(SBoolExpr),
    UntilBoolRep(SBoolExpr, SLogicBinOp, SRepititions),
    UntilRep(SRepititions),
    UntilEnd
}   

#[derive(Debug, Clone)]
pub struct Repititions {
    pub times: SIntExpr
}

#[derive(Debug, Clone)]
pub enum MemoryType {
    Int(SIntExpr),
    String(SStringExpr),
    CardSet(SCardSet),
    Collection(SCollection)
}

#[derive(Debug, Clone)]
pub enum Players {
    Player(SPlayerExpr),
    PlayerCollection(SPlayerCollection),
}

#[derive(Debug, Clone)]
pub enum EndType {
    Turn,
    Stage,
    GameWithWinner(SPlayers),
}

#[derive(Debug, Clone)]
pub enum DemandType {
    CardPosition(SCardPosition),
    String(SStringExpr),
    Int(SIntExpr),
}

#[derive(Debug, Clone)]
pub struct Types {
    pub types: Vec<(SID, Vec<SID>)>
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
#[derive(Debug, Clone)]
pub enum RuntimePlayer {
    Current,
    Next,
    Previous,
    Competitor,
}

#[derive(Debug, Clone)]
pub enum QueryPlayer {
    Turnorder(SIntExpr),
}

#[derive(Debug, Clone)]
pub enum AggregatePlayer {
    OwnerOfCardPostion(Box<SCardPosition>),
    OwnerOfMemory(SExtrema, SID),
}

#[derive(Debug, Clone)]
pub enum PlayerExpr {
    Literal(SID),
    Runtime(SRuntimePlayer),
    Aggregate(SAggregatePlayer),
    Query(SQueryPlayer)
}
// ===========================================================================


// IntExpr
// ===========================================================================
#[derive(Debug, Clone)]
pub enum QueryInt {
    IntCollectionAt(Box<SIntCollection>, Box<SIntExpr>),
}

#[derive(Debug, Clone)]
pub enum AggregateInt {
    SizeOf(SCollection),
    SumOfIntCollection(SIntCollection),
    SumOfCardSet(Box<SCardSet>, SID),
    ExtremaCardset(SExtrema, Box<SCardSet>, SID),
    ExtremaIntCollection(SExtrema, SIntCollection),
}

#[derive(Debug, Clone)]
pub enum RuntimeInt {
    StageRoundCounter,
    PlayRoundCounter,
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum QueryString {
    KeyOf(SID, SCardPosition),
    StringCollectionAt(SStringCollection, SIntExpr),
}

#[derive(Debug, Clone)]
pub enum StringExpr {
    Literal(SID),
    Query(SQueryString),
}
// ===========================================================================

// Bool
// ===========================================================================
#[derive(Debug, Clone)]
pub enum CardSetCompare {
    Eq,
    Neq
}

#[derive(Debug, Clone)]
pub enum StringCompare {
    Eq,
    Neq
}

#[derive(Debug, Clone)]
pub enum PlayerCompare {
    Eq,
    Neq
}

#[derive(Debug, Clone)]
pub enum TeamCompare {
    Eq,
    Neq
}

#[derive(Debug, Clone)]
pub enum BoolOp {
    And,
    Or
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, Clone)]
pub enum CompareBool {
    Int(SIntExpr, SIntCompare, SIntExpr),
    CardSet(SCardSet, SCardSetCompare, SCardSet),
    String(SStringExpr, SStringCompare, SStringExpr),
    Player(SPlayerExpr, SPlayerCompare, SPlayerExpr),
    Team(STeamExpr, STeamCompare, STeamExpr),
}

#[derive(Debug, Clone)]
pub enum AggregateBool {
    Compare(SCompareBool),
    CardSetEmpty(SCardSet),
    CardSetNotEmpty(SCardSet),
    OutOfPlayer(SPlayers,     SOutOf),
}

#[derive(Debug, Clone)]
pub enum BoolExpr {
    Binary(Box<SBoolExpr>, SBoolOp, Box<SBoolExpr>),
    Unary(SUnaryOp, Box<SBoolExpr>),
    Aggregate(SAggregateBool),
}
// ===========================================================================


// Team
// ===========================================================================
#[derive(Debug, Clone)]
pub enum AggregateTeam {
    TeamOf(SPlayerExpr)
}

#[derive(Debug, Clone)]
pub enum TeamExpr {
    Literal(SID),
    Aggregate(SAggregateTeam),
}
// ===========================================================================

// CardPosition
// ===========================================================================
#[derive(Debug, Clone)]
pub enum QueryCardPosition {
    At(SID, SIntExpr),
    Top(SID),
    Bottom(SID),
}

#[derive(Debug, Clone)]
pub enum AggregateCardPosition {
    Extrema(SExtrema, Box<SCardSet>, SID),
}

#[derive(Debug, Clone)]
pub enum CardPosition {
    Query(SQueryCardPosition),
    Aggregate(SAggregateCardPosition)
}

// Stauts
// ===========================================================================
#[derive(Debug, Clone)]
pub enum Status {
    FaceUp,
    FaceDown,
    Private
}
// ===========================================================================

// ===========================================================================
// ===========================================================================
// ===========================================================================


// Collections
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, Clone)]
pub enum Collection {
    IntCollection(SIntCollection),
    StringCollection(SStringCollection),
    LocationCollection(SLocationCollection),
    PlayerCollection(SPlayerCollection),
    TeamCollection(STeamCollection),
    CardSet(Box<SCardSet>),
    Ambiguous(Vec<SID>),
}

#[derive(Debug, Clone)]
pub struct IntCollection {
    pub ints: Vec<SIntExpr>
}

#[derive(Debug, Clone)]
pub struct StringCollection {
    pub strings: Vec<SStringExpr>
}

#[derive(Debug, Clone)]
pub struct LocationCollection {
    pub locations: Vec<SID>
}

// PlayerCollection
// ===========================================================================
#[derive(Debug, Clone)]
pub enum RuntimePlayerCollection {
    PlayersOut,
    PlayersIn,
    Others,
}

#[derive(Debug, Clone)]
pub enum AggregatePlayerCollection {
    Quantifier(SQuantifier),
}

#[derive(Debug, Clone)]
pub enum PlayerCollection {
    Literal(Vec<SPlayerExpr>),
    Aggregate(SAggregatePlayerCollection),
    Runtime(SRuntimePlayerCollection),
}
// ===========================================================================

// TeamCollection
// ===========================================================================
#[derive(Debug, Clone)]
pub enum RuntimeTeamCollection {
    OtherTeams,
}

#[derive(Debug, Clone)]
pub enum TeamCollection {
    Literal(Vec<STeamExpr>),
    Runtime(SRuntimeTeamCollection)
}

// ===========================================================================
// ===========================================================================
// ===========================================================================

// CardSet
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, Clone)]
pub enum CardSet {
    Group(SGroup),
    GroupOwner(SGroup, SOwner),
}


#[derive(Debug, Clone)]
pub enum Group {
    Groupable(SGroupable),
    Where(SGroupable, SFilterExpr),
    NotCombo(SID, SGroupable),
    Combo(SID, SGroupable),
    CardPosition(SCardPosition),
}

// FilterExpr
// ===========================================================================
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum FilterOp {
    And,
    Or
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Game {
    pub flows: Vec<SFlowComponent>
}

#[derive(Debug, Clone)]
pub enum FlowComponent {
    Stage(SSeqStage),
    Rule(SGameRule),
    IfRule(SIfRule),
    ChoiceRule(SChoiceRule),
    OptionalRule(SOptionalRule),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum ActionRule {
    // Actions
    FlipAction (SCardSet, SStatus),
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

#[derive(Debug, Clone)]
pub enum ScoringRule {
    // Score + Winner Rule
    ScoreRule(SScoreRule),
    WinnerRule(SWinnerRule)
}

#[derive(Debug, Clone)]
pub enum GameRule {
    SetUp(SSetUpRule),
    Action(SActionRule),
    Scoring(SScoringRule)
}

#[derive(Debug, Clone)]
pub struct SeqStage {
    pub stage: SID,
    pub player: SPlayerExpr,
    pub end_condition: SEndCondition,
    pub flows: Vec<SFlowComponent>,
}

#[derive(Debug, Clone)]
pub struct IfRule {
    pub condition: SBoolExpr,
    pub flows: Vec<SFlowComponent>,
}

#[derive(Debug, Clone)]
pub struct OptionalRule {
    pub flows: Vec<SFlowComponent>
}

#[derive(Debug, Clone)]
pub struct ChoiceRule {
    pub options: Vec<SFlowComponent>
}

#[derive(Debug, Clone)]
pub enum MoveType {
    Deal(SDealMove),
    Exchange(SExchangeMove),
    Classic(SClassicMove),
    Place(STokenMove),
}

#[derive(Debug, Clone)]
pub enum MoveCardSet {
    Move(SCardSet, SStatus, SCardSet),
    MoveQuantity(SQuantity, SCardSet, SStatus, SCardSet),
}

#[derive(Debug, Clone)]
pub enum ClassicMove {
    MoveCardSet(SMoveCardSet),
}

#[derive(Debug, Clone)]
pub enum DealMove {
    MoveCardSet(SMoveCardSet),
}

#[derive(Debug, Clone)]
pub enum ExchangeMove {
    MoveCardSet(SMoveCardSet),
}

#[derive(Debug, Clone)]
pub enum TokenMove {
    Place(SID, STokenLocExpr, STokenLocExpr),
    PlaceQuantity(SQuantity, SID, STokenLocExpr, STokenLocExpr),
}

#[derive(Debug, Clone)]
pub enum TokenLocExpr {
    Groupable(SGroupable),
    GroupablePlayers(SGroupable, SPlayers),
}

#[derive(Debug, Clone)]
pub enum ScoreRule {
    Score(SIntExpr, SPlayers),
    ScoreMemory(SIntExpr, SID, SPlayers),
}

#[derive(Debug, Clone)]
pub enum WinnerType {
    Score,
    Memory(SID),
    Position
}

#[derive(Debug, Clone)]
pub enum WinnerRule {
    Winner(SPlayers),
    WinnerWith(SExtrema, SWinnerType),
}

// ===========================================================================
// ===========================================================================
// ===========================================================================
