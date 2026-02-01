#[derive(Debug, PartialEq, Clone)]
pub enum GameType {
  Player,
  Team,
  Location,
  Precedence,
  PointMap,
  Combo,
  Key,
  Value,
  Memory,
  Token,
  Stage,
  String,
  NoType,
}

// IDs
#[derive(Debug, PartialEq, Clone)]
pub struct TypedID {
    pub id: String,
    pub ty: GameType,
}

impl TypedID {
    pub fn new(id: String, ty: GameType) -> TypedID {
        TypedID {
            id: id,
            ty: ty
        }
    }
}

use crate::diagnostic::*;


// Operator
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, PartialEq, Clone)]
pub enum BinCompare {
    Eq,
    Neq
}

#[derive(Debug, PartialEq, Clone)]
pub enum LogicBinOp {
    And,
    Or
}

#[derive(Debug, PartialEq, Clone)]
pub enum IntOp {
    Plus,
    Minus,
    Mul,
    Div,
    Mod
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum Extrema {
    Min,
    Max
}


#[derive(Debug, PartialEq, Clone)]
pub enum OutOf {
    CurrentStage,
    Stage(TypedID),
    Game,
    Play
}

#[derive(Debug, PartialEq, Clone)]
pub enum Groupable {
    Location(TypedID),
    LocationCollection(LocationCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Owner {
    Player(PlayerExpr),
    PlayerCollection(PlayerCollection),
    Team(TeamExpr),
    TeamCollection(TeamCollection),
    Table
}

#[derive(Debug, PartialEq, Clone)]
pub enum Quantity {
    Int(IntExpr),
    Quantifier(Quantifier),
    IntRange(IntRange),
}

#[derive(Debug, PartialEq, Clone)]
pub struct IntRange { 
    pub op_int: Vec<(IntCompare, IntExpr)>, 
}

#[derive(Debug, PartialEq, Clone)]
pub enum Quantifier {
    All,
    Any
}

#[derive(Debug, PartialEq, Clone)]
pub enum EndCondition {
    UntilBool(BoolExpr),
    UntilBoolRep(BoolExpr, LogicBinOp, Repititions),
    UntilRep(Repititions),
    UntilEnd
}   

#[derive(Debug, PartialEq, Clone)]
pub struct Repititions {
    pub times: IntExpr
}

#[derive(Debug, PartialEq, Clone)]
pub enum MemoryType {
    Int(IntExpr),
    String(StringExpr),
    CardSet(CardSet),
    Collection(Collection)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Players {
    Player(PlayerExpr),
    PlayerCollection(PlayerCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum EndType {
    Turn,
    Stage,
    GameWithWinner(Players),
}

#[derive(Debug, PartialEq, Clone)]
pub enum DemandType {
    CardPosition(CardPosition),
    String(StringExpr),
    Int(IntExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Types {
    pub types: Vec<(TypedID, Vec<TypedID>)>
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
#[derive(Debug, PartialEq, Clone)]
pub enum RuntimePlayer {
    Current,
    Next,
    Previous,
    Competitor,
}

#[derive(Debug, PartialEq, Clone)]
pub enum QueryPlayer {
    Turnorder(IntExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AggregatePlayer {
    OwnerOfCardPostion(Box<CardPosition>),
    OwnerOfMemory(Extrema, TypedID),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerExpr {
    Literal(TypedID),
    Runtime(RuntimePlayer),
    Aggregate(AggregatePlayer),
    Query(QueryPlayer)
}
// ===========================================================================


// IntExpr
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
pub enum QueryInt {
    IntCollectionAt(Box<IntCollection>, Box<IntExpr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AggregateInt {
    SizeOf(Collection),
    SumOfIntCollection(IntCollection),
    SumOfCardSet(Box<CardSet>, TypedID),
    ExtremaCardset(Extrema, Box<CardSet>, TypedID),
    ExtremaIntCollection(Extrema, IntCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeInt {
    StageRoundCounter,
    PlayRoundCounter,
}

#[derive(Debug, PartialEq, Clone)]
pub enum IntExpr {
    Literal(i32),
    Binary(Box<IntExpr>, IntOp, Box<IntExpr>),
    Query(QueryInt),
    Aggregate(AggregateInt),
    Runtime(RuntimeInt),   
}
// ===========================================================================


// String
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
pub enum QueryString {
    KeyOf(TypedID, CardPosition),
    StringCollectionAt(StringCollection, IntExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringExpr {
    Literal(TypedID),
    Query(QueryString),
}
// ===========================================================================

// Bool
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
pub enum CardSetCompare {
    Eq,
    Neq
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringCompare {
    Eq,
    Neq
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerCompare {
    Eq,
    Neq
}

#[derive(Debug, PartialEq, Clone)]
pub enum TeamCompare {
    Eq,
    Neq
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoolOp {
    And,
    Or
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CompareBool {
    Int(IntExpr, IntCompare, IntExpr),
    CardSet(CardSet, CardSetCompare, CardSet),
    String(StringExpr, StringCompare, StringExpr),
    Player(PlayerExpr, PlayerCompare, PlayerExpr),
    Team(TeamExpr, TeamCompare, TeamExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AggregateBool {
    Compare(CompareBool),
    CardSetEmpty(CardSet),
    CardSetNotEmpty(CardSet),
    OutOfPlayer(Players, OutOf),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoolExpr {
    Binary(Box<BoolExpr>, BoolOp, Box<BoolExpr>),
    Unary(UnaryOp, Box<BoolExpr>),
    Aggregate(AggregateBool),
}
// ===========================================================================


// Team
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
pub enum AggregateTeam {
    TeamOf(PlayerExpr)
}

#[derive(Debug, PartialEq, Clone)]
pub enum TeamExpr {
    Literal(TypedID),
    Aggregate(AggregateTeam),
}
// ===========================================================================

// CardPosition
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
pub enum QueryCardPosition {
    At(TypedID, IntExpr),
    Top(TypedID),
    Bottom(TypedID),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AggregateCardPosition {
    Extrema(Extrema, Box<CardSet>, TypedID),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CardPosition {
    Query(QueryCardPosition),
    Aggregate(AggregateCardPosition)
}

// Stauts
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum Collection {
    IntCollection(IntCollection),
    StringCollection(StringCollection),
    LocationCollection(LocationCollection),
    PlayerCollection(PlayerCollection),
    TeamCollection(TeamCollection),
    CardSet(Box<CardSet>),
    Ambiguous(Vec<TypedID>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct IntCollection {
    pub ints: Vec<IntExpr>
}

#[derive(Debug, PartialEq, Clone)]
pub struct StringCollection {
    pub strings: Vec<StringExpr>
}

#[derive(Debug, PartialEq, Clone)]
pub struct LocationCollection {
    pub locations: Vec<TypedID>
}

// PlayerCollection
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
pub enum RuntimePlayerCollection {
    PlayersOut,
    PlayersIn,
    Others,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AggregatePlayerCollection {
    Quantifier(Quantifier),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerCollection {
    Literal(Vec<PlayerExpr>),
    Aggregate(AggregatePlayerCollection),
    Runtime(RuntimePlayerCollection),
}
// ===========================================================================

// TeamCollection
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeTeamCollection {
    OtherTeams,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TeamCollection {
    Literal(Vec<TeamExpr>),
    Runtime(RuntimeTeamCollection)
}

// ===========================================================================
// ===========================================================================
// ===========================================================================

// CardSet
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, PartialEq, Clone)]
pub enum CardSet {
    Group(Group),
    GroupOwner(Group, Owner),
}


#[derive(Debug, PartialEq, Clone)]
pub enum Group {
    Groupable(Groupable),
    Where(Groupable, FilterExpr),
    NotCombo(TypedID, Groupable),
    Combo(TypedID, Groupable),
    CardPosition(CardPosition),
}

// FilterExpr
// ===========================================================================
#[derive(Debug, PartialEq, Clone)]
pub enum AggregateFilter {
    Size(IntCompare, Box<IntExpr>),
    Same(TypedID),
    Distinct(TypedID),
    Adjacent(TypedID, TypedID),
    Higher(TypedID, TypedID),
    Lower(TypedID, TypedID),
    KeyString(TypedID, StringCompare, Box<StringExpr>),
    Combo(TypedID),
    NotCombo(TypedID),
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterOp {
    And,
    Or
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterExpr {
    Aggregate(AggregateFilter),
    Binary(Box<FilterExpr>, FilterOp, Box<FilterExpr>),
}
// ===========================================================================


// ===========================================================================
// ===========================================================================
// ===========================================================================

// Game + Stage + FlowComponent + Rule
// ===========================================================================
// ===========================================================================
// ===========================================================================

#[derive(Debug, PartialEq, Clone)]
pub struct Game {
    pub flows: Vec<FlowComponent>
}

#[derive(Debug, PartialEq, Clone)]
pub enum FlowComponent {
    Stage(SeqStage),
    Rule(GameRule),
    IfRule(IfRule),
    ChoiceRule(ChoiceRule),
    OptionalRule(OptionalRule),
}

#[derive(Debug, PartialEq, Clone)]
pub enum SetUpRule {
    // Creations
    CreatePlayer(Vec<TypedID>),
    CreateTeams(Vec<(TypedID, PlayerCollection)>),
    CreateTurnorder(PlayerCollection),
    CreateTurnorderRandom(PlayerCollection),
    CreateLocation(Vec<TypedID>, Owner),
    CreateCardOnLocation(TypedID, Types),
    CreateTokenOnLocation(IntExpr, TypedID, TypedID),
    CreateCombo(TypedID, FilterExpr),
    CreateMemory(TypedID, MemoryType, Owner),
    CreatePrecedence(TypedID, Vec<(TypedID, TypedID)>),
    CreatePointMap(TypedID, Vec<(TypedID, TypedID, IntExpr)>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ActionRule {
    // Actions
    FlipAction (CardSet, Status),
    ShuffleAction(CardSet),
    PlayerOutOfStageAction(Players),
    PlayerOutOfGameSuccAction(Players),
    PlayerOutOfGameFailAction(Players),
    SetMemory(TypedID, MemoryType),
    ResetMemory(TypedID),
    CycleAction(PlayerExpr),
    BidAction(Quantity),
    BidMemoryAction(TypedID, Quantity),
    EndAction(EndType),
    DemandAction(DemandType),
    DemandMemoryAction(DemandType, TypedID),
    Move(MoveType),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ScoringRule {
    // Score + Winner Rule
    ScoreRule(ScoreRule),
    WinnerRule(WinnerRule)
}

#[derive(Debug, PartialEq, Clone)]
pub enum GameRule {
    SetUp(SetUpRule),
    Action(ActionRule),
    Scoring(ScoringRule)
}

#[derive(Debug, PartialEq, Clone)]
pub struct SeqStage {
    pub stage: TypedID,
    pub player: PlayerExpr,
    pub end_condition: EndCondition,
    pub flows: Vec<FlowComponent>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfRule {
    pub condition: BoolExpr,
    pub flows: Vec<FlowComponent>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OptionalRule {
    pub flows: Vec<FlowComponent>
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChoiceRule {
    pub options: Vec<FlowComponent>
}

#[derive(Debug, PartialEq, Clone)]
pub enum MoveType {
    Deal(DealMove),
    Exchange(ExchangeMove),
    Classic(ClassicMove),
    Place(TokenMove),
}

#[derive(Debug, PartialEq, Clone)]
pub enum MoveCardSet {
    Move(CardSet, Status, CardSet),
    MoveQuantity(Quantity, CardSet, Status, CardSet)
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClassicMove {
    MoveCardSet(MoveCardSet),
}

#[derive(Debug, PartialEq, Clone)]
pub enum DealMove {
    MoveCardSet(MoveCardSet),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExchangeMove {
    MoveCardSet(MoveCardSet),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenMove {
    Place(TypedID, TokenLocExpr, TokenLocExpr),
    PlaceQuantity(Quantity, TypedID, TokenLocExpr, TokenLocExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenLocExpr {
    Groupable(Groupable),
    GroupablePlayers(Groupable, Players),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ScoreRule {
    Score(IntExpr, Players),
    ScoreMemory(IntExpr, TypedID, Players),
}

#[derive(Debug, PartialEq, Clone)]
pub enum WinnerType {
    Score,
    Memory(TypedID),
    Position
}

#[derive(Debug, PartialEq, Clone)]
pub enum WinnerRule {
    Winner(Players),
    WinnerWith(Extrema, WinnerType),
}

// ===========================================================================
// ===========================================================================
// ===========================================================================
