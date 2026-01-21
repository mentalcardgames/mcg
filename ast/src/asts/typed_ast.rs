use crate::asts::game_type::GameType;

// IDs
#[derive(Debug, PartialEq, Clone)]
pub struct TypedID {
    pub id: String,
    ty: GameType,
}

impl TypedID {
    pub fn new(id: String, ty: GameType) -> TypedID {
        TypedID {
            id: id,
            ty: ty
        }
    }
}

// Structs + Enums
#[derive(Debug, PartialEq, Clone)]
pub enum PlayerExpr {
    PlayerName(TypedID),
    Current,
    Next,
    Previous,
    Competitor,
    Turnorder(IntExpr),
    OwnerOf(Box<CardPosition>),
    OwnerOfHighest(TypedID),
    OwnerOfLowest(TypedID),
}

#[derive(Debug, PartialEq, Clone)]
pub enum IntExpr {
    Int(i32),
    IntOp(Box<IntExpr>, Op, Box<IntExpr>),
    IntCollectionAt(Box<IntExpr>),
    SizeOf(Collection),
    SumOfIntCollection(IntCollection),
    SumOfCardSet(Box<CardSet>, TypedID),
    MinOf(Box<CardSet>, TypedID),
    MaxOf(Box<CardSet>, TypedID),
    MinIntCollection(IntCollection),
    MaxIntCollection(IntCollection),
    StageRoundCounter,
    PlayRoundCounter,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Plus,
    Minus,
    Mul,
    Div,
    Mod
}

#[derive(Debug, PartialEq, Clone)]
pub enum Collection {
    IntCollection(IntCollection),
    StringCollection(StringCollection),
    LocationCollection(LocationCollection),
    PlayerCollection(PlayerCollection),
    TeamCollection(TeamCollection),
    CardSet(Box<CardSet>),
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

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerCollection {
    Player(Vec<PlayerExpr>),
    Others,
    Quantifier(Quantifier),
    PlayersOut,
    PlayersIn,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TeamCollection {
    Team(Vec<TeamExpr>),
    OtherTeams,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringExpr {
    KeyOf(TypedID, CardPosition),
    StringCollectionAt(StringCollection, IntExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CardPosition {
    At(TypedID, IntExpr),
    Top(TypedID),
    Bottom(TypedID),
    MaxPrecedence (Box<CardSet>, TypedID),
    MinPrecedence (Box<CardSet>, TypedID),
    MaxPointMap (Box<CardSet>, TypedID),
    MinPointMap (Box<CardSet>, TypedID),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoolExpr {
    IntCmp(IntExpr, IntCmpOp, IntExpr),
    CardSetIsEmpty(CardSet),
    CardSetIsNotEmpty(CardSet),
    CardSetEq(CardSet, CardSet),
    CardSetNeq(CardSet, CardSet),
    StringEq (StringExpr, StringExpr),
    StringNeq(StringExpr, StringExpr),
    PlayerEq (PlayerExpr, PlayerExpr),
    PlayerNeq(PlayerExpr, PlayerExpr),
    TeamEq (TeamExpr, TeamExpr),
    TeamNeq(TeamExpr, TeamExpr),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
    OutOfStagePlayer(PlayerExpr),
    OutOfGamePlayer(PlayerExpr),
    OutOfStageCollection(PlayerCollection),
    OutOfGameCollection(PlayerCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum IntCmpOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le
}

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    FaceUp,
    FaceDown,
    Private
}

#[derive(Debug, PartialEq, Clone)]
pub enum TeamExpr {
    TeamName(TypedID),
    TeamOf(PlayerExpr)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Quantity {
    Int(IntExpr),
    Quantifier(Quantifier),
    IntRange(IntRange),
}

#[derive(Debug, PartialEq, Clone)]
pub struct IntRange { pub op: IntCmpOp, pub int: IntExpr}

#[derive(Debug, PartialEq, Clone)]
pub enum Quantifier {
    All,
    Any
}

#[derive(Debug, PartialEq, Clone)]
pub enum CardSet {
    Group(Group),
    GroupOfPlayer(Group, PlayerExpr),
    GroupOfPlayerCollection(Group, PlayerCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Group {
    Location(TypedID),
    LocationWhere(TypedID, FilterExpr),
    LocationCollection(LocationCollection),
    LocationCollectionWhere(LocationCollection, FilterExpr),
    ComboInLocation(TypedID, TypedID),
    ComboInLocationCollection(TypedID, LocationCollection),
    NotComboInLocation(TypedID, TypedID),
    NotComboInLocationCollection(TypedID, LocationCollection),
    CardPosition(CardPosition),
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterExpr {
    Same(TypedID),
    Distinct(TypedID),
    Adjacent(TypedID, TypedID),
    Higher(TypedID, TypedID),
    Lower(TypedID, TypedID),
    Size (IntCmpOp, Box<IntExpr>),
    KeyEqString  (TypedID, Box<StringExpr>),
    KeyNeqString (TypedID, Box<StringExpr>),
    KeyEqValue  (TypedID, TypedID),
    KeyNeqValue (TypedID, TypedID),
    NotCombo(TypedID),
    Combo(TypedID),
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Game {
    pub flows: Vec<FlowComponent>
}

#[derive(Debug, PartialEq, Clone)]
pub enum FlowComponent {
    Stage(SeqStage),
    Rule(Rule),
    IfRule(IfRule),
    ChoiceRule(ChoiceRule),
    OptionalRule(OptionalRule),
}

#[derive(Debug, PartialEq, Clone)]
pub enum EndCondition {
    UntilBool(BoolExpr),
    UntilBoolAndRep(BoolExpr, Repititions),
    UntilBoolOrRep(BoolExpr, Repititions),
    UntilRep(Repititions),
    UntilEnd
}   

#[derive(Debug, PartialEq, Clone)]
pub struct Repititions {
    pub times: IntExpr
}

#[derive(Debug, PartialEq, Clone)]
pub enum Rule {
    // Creations
    CreatePlayer(Vec<TypedID>),
    CreateTeam(TypedID, Vec<TypedID>),
    CreateTurnorder(Vec<TypedID>),
    CreateTurnorderRandom(Vec<TypedID>),
    CreateLocationOnPlayerCollection(TypedID, PlayerCollection),
    CreateLocationOnTeamCollection(TypedID, TeamCollection),
    CreateLocationOnTable(TypedID),
    CreateLocationCollectionOnPlayerCollection(LocationCollection, PlayerCollection),
    CreateLocationCollectionOnTeamCollection(LocationCollection, TeamCollection),
    CreateLocationCollectionOnTable(LocationCollection),
    CreateCardOnLocation(TypedID, Types),
    CreateTokenOnLocation(IntExpr, TypedID, TypedID),
    CreatePrecedence(TypedID, Vec<(TypedID, TypedID)>),
    CreateCombo(TypedID, FilterExpr),
    CreateMemoryIntPlayerCollection(TypedID, IntExpr, PlayerCollection),
    CreateMemoryStringPlayerCollection(TypedID, StringExpr, PlayerCollection),
    CreateMemoryIntTable(TypedID, IntExpr),
    CreateMemoryStringTable(TypedID, StringExpr),
    CreateMemoryPlayerCollection(TypedID, PlayerCollection),
    CreateMemoryTable(TypedID),
    CreatePointMap(TypedID, Vec<(TypedID, TypedID, IntExpr)>),
    // Actions
    FlipAction(CardSet, Status),
    ShuffleAction(CardSet),
    PlayerOutOfStageAction(PlayerExpr),
    PlayerOutOfGameSuccAction(PlayerExpr),
    PlayerOutOfGameFailAction(PlayerExpr),
    PlayerCollectionOutOfStageAction(PlayerCollection),
    PlayerCollectionOutOfGameSuccAction(PlayerCollection),
    PlayerCollectionOutOfGameFailAction(PlayerCollection),
    SetMemoryInt(TypedID, IntExpr),
    SetMemoryString(TypedID, StringExpr),
    SetMemoryCollection(TypedID, Collection),
    CycleAction(PlayerExpr),
    BidAction(Quantity),
    BidActionMemory(TypedID, Quantity),
    EndTurn,
    EndStage,
    EndGameWithWinner(PlayerExpr),
    DemandCardPositionAction(CardPosition),
    DemandStringAction(StringExpr),
    DemandIntAction(IntExpr),
    // Move-Actions
    ClassicMove(ClassicMove),
    DealMove(DealMove),
    ExchangeMove(ExchangeMove),
    TokenMove(TokenMove),
    // Score + Winner Rule
    ScoreRule(ScoreRule),
    WinnerRule(WinnerRule)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Types {
    pub types: Vec<(TypedID, Vec<TypedID>)>
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
pub enum ClassicMove {
    Move(CardSet, Status, CardSet),
    MoveQuantity(Quantity, CardSet, Status, CardSet),
}

#[derive(Debug, PartialEq, Clone)]
pub enum DealMove {
    Deal(CardSet, Status, CardSet),
    DealQuantity(Quantity, CardSet, Status, CardSet),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExchangeMove {
    Exchange(CardSet, Status, CardSet),
    ExchangeQuantity(Quantity, CardSet, Status, CardSet),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenMove {
    Place(TypedID, TokenLocExpr, TokenLocExpr),
    PlaceQuantity(Quantity, TypedID, TokenLocExpr, TokenLocExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenLocExpr {
    Location(TypedID),
    LocationCollection(LocationCollection),
    LocationPlayer(TypedID, PlayerExpr),
    LocationCollectionPlayer(LocationCollection, PlayerExpr),
    LocationPlayerCollection(TypedID, PlayerCollection),
    LocationCollectionPlayerCollection(LocationCollection, PlayerCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ScoreRule {
    ScorePlayer(IntExpr, PlayerExpr),
    ScorePlayerMemory(IntExpr, TypedID, PlayerExpr),
    ScorePlayerCollection(IntExpr, PlayerCollection),
    ScorePlayerCollectionMemory(IntExpr, TypedID, PlayerCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum WinnerRule {
    WinnerPlayer(PlayerExpr),
    WinnerPlayerCollection(PlayerCollection),
    WinnerLowestScore,
    WinnerHighestScore,
    WinnerLowestMemory(TypedID),
    WinnerHighestMemory(TypedID),
    WinnerLowestPosition,
    WinnerHighestPosition,   
}