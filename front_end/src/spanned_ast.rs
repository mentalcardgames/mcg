use crate::diagnostic::*;

// IDs
pub struct ID(pub String);

impl ToString for ID {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub type PlayerName = SID;
pub type TeamName = SID;
pub type Location = SID;
pub type Precedence = SID;
pub type PointMap = SID;
pub type Combo = SID;
pub type Key = SID;
pub type Value = SID;
pub type Memory = SID;
pub type Token = SID;
pub type Stage = SID;

// Structs + Enums
#[derive(Debug, Clone)]
pub enum PlayerExpr {
    PlayerName(PlayerName),
    Current,
    Next,
    Previous,
    Competitor,
    Turnorder(SIntExpr),
    OwnerOf(Box<SCardPosition>),
    OwnerOfHighest(Memory),
    OwnerOfLowest(Memory),
}

#[derive(Debug, Clone)]
pub enum IntExpr {
    Int(i32),
    IntOp(Box<SIntExpr>, SOp, Box<SIntExpr>),
    IntCollectionAt(Box<SIntExpr>),
    SizeOf(SCollection),
    SumOfIntCollection(SIntCollection),
    SumOfCardSet(Box<SCardSet>, PointMap),
    MinOf(Box<SCardSet>, PointMap),
    MaxOf(Box<SCardSet>, PointMap),
    MinIntCollection(SIntCollection),
    MaxIntCollection(SIntCollection),
    StageRoundCounter,
    // PlayRoundCounter,
}

#[derive(Debug, Clone)]
pub enum Op {
    Plus,
    Minus,
    Mul,
    Div,
    Mod
}


// TODO:
// Collection is only being used for size of.
// Maybe building a Collection type that is just used
// for this specific case.
// Because Collection is very ambiguous.
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
    pub locations: Vec<Location>
}

#[derive(Debug, Clone)]
pub enum PlayerCollection {
    Player(Vec<SPlayerExpr>),
    Others,
    Quantifier(SQuantifier),
    PlayersOut,
    PlayersIn,
}

#[derive(Debug, Clone)]
pub enum TeamCollection {
    Team(Vec<STeamExpr>),
    OtherTeams,
}

#[derive(Debug, Clone)]
pub enum StringExpr {
    // I think StringExpr ID has no value.
    // It has no type and therefore
    // can't be checked or use securely.
    // ID(ID),
    KeyOf(Key, SCardPosition),
    StringCollectionAt(SStringCollection, SIntExpr),
}

#[derive(Debug, Clone)]
pub enum CardPosition {
    At(Location, SIntExpr),
    Top(Location),
    Bottom(Location),
    // Analyzer decides afterwards if it is Precedence or PointMap
    Max (Box<SCardSet>, SID),
    Min (Box<SCardSet>, SID),
}

#[derive(Debug, Clone)]
pub enum BoolExpr {
    IntCmp(SIntExpr, SIntCmpOp, SIntExpr),
    CardSetIsEmpty(SCardSet),
    CardSetIsNotEmpty(SCardSet),
    CardSetEq(SCardSet, SCardSet),
    CardSetNeq(SCardSet, SCardSet),
    StringEq (SStringExpr, SStringExpr),
    StringNeq(SStringExpr, SStringExpr),
    PlayerEq (SPlayerExpr, SPlayerExpr),
    PlayerNeq(SPlayerExpr, SPlayerExpr),
    TeamEq (STeamExpr, STeamExpr),
    TeamNeq(STeamExpr, STeamExpr),
    And(Box<SBoolExpr>, Box<SBoolExpr>),
    Or(Box<SBoolExpr>, Box<SBoolExpr>),
    Not(Box<SBoolExpr>),
    OutOfStagePlayer(SPlayerExpr),
    OutOfGamePlayer(SPlayerExpr),
    OutOfStageCollection(SPlayerCollection),
    OutOfGameCollection(SPlayerCollection),
    // Catch case if we have something like P1 == P2 or T1 == T2
    // Matching IDs like P1 == P2 should not be done and will not be handled.
    // This will directly be interpreted as CardSet! 
    // AmbiguousEq (String, String),
    // AmbiguousNeq(String, String),
}

#[derive(Debug, Clone)]
pub enum IntCmpOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le
}

#[derive(Debug, Clone)]
pub enum Status {
    FaceUp,
    FaceDown,
    Private
}

#[derive(Debug, Clone)]
pub enum TeamExpr {
    TeamName(TeamName),
    TeamOf(SPlayerExpr)
}

#[derive(Debug, Clone)]
pub enum Quantity {
    Int(SIntExpr),
    Quantifier(SQuantifier),
    IntRange(SIntRange),
}

#[derive(Debug, Clone)]
pub struct IntRange { pub op: SIntCmpOp, pub int: SIntExpr}

#[derive(Debug, Clone)]
pub enum Quantifier {
    All,
    Any
}

#[derive(Debug, Clone)]
pub enum CardSet {
    Group(SGroup),
    GroupOfPlayer(SGroup, SPlayerExpr),
    GroupOfPlayerCollection(SGroup, SPlayerCollection),
}

#[derive(Debug, Clone)]
pub enum Group {
    Location(Location),
    LocationWhere(Location, SFilterExpr),
    LocationCollection(SLocationCollection),
    LocationCollectionWhere(SLocationCollection, SFilterExpr),
    ComboInLocation(Combo, Location),
    ComboInLocationCollection(Combo, SLocationCollection),
    NotComboInLocation(Combo, Location),
    NotComboInLocationCollection(Combo, SLocationCollection),
    CardPosition(SCardPosition),
}

#[derive(Debug, Clone)]
pub enum FilterExpr {
    Same(Key),
    Distinct(Key),
    Adjacent(Key, Precedence),
    Higher(Key, Precedence),
    Lower(Key, Precedence),
    Size (SIntCmpOp, Box<SIntExpr>),
    KeyEqString  (Key, Box<SStringExpr>),
    KeyNeqString (Key, Box<SStringExpr>),
    KeyEqValue  (Key, Value),
    KeyNeqValue (Key, Value),
    NotCombo(Combo),
    Combo(Combo),
    And(Box<SFilterExpr>, Box<SFilterExpr>),
    Or (Box<SFilterExpr>, Box<SFilterExpr>),
}

#[derive(Debug, Clone)]
pub struct Game {
    pub flows: Vec<SFlowComponent>
}

#[derive(Debug, Clone)]
pub enum FlowComponent {
    Stage(SSeqStage),
    Rule(SRule),
    IfRule(SIfRule),
    ChoiceRule(SChoiceRule),
    OptionalRule(SOptionalRule),
}

#[derive(Debug, Clone)]
pub enum EndCondition {
    UntilBool(SBoolExpr),
    UntilBoolAndRep(SBoolExpr, SRepititions),
    UntilBoolOrRep(SBoolExpr, SRepititions),
    UntilRep(SRepititions),
    UntilEnd
}   

#[derive(Debug, Clone)]
pub struct Repititions {
    pub times: SIntExpr
}

#[derive(Debug, Clone)]
pub enum Rule {
    // Creations
    CreatePlayer(Vec<PlayerName>),
    CreateTeam(TeamName, Vec<PlayerName>),
    CreateTurnorder(Vec<PlayerName>),
    CreateTurnorderRandom(Vec<PlayerName>),
    CreateLocationOnPlayerCollection(Location, SPlayerCollection),
    CreateLocationOnTeamCollection(Location, STeamCollection),
    CreateLocationOnTable(Location),
    CreateLocationCollectionOnPlayerCollection(SLocationCollection, SPlayerCollection),
    CreateLocationCollectionOnTeamCollection(SLocationCollection, STeamCollection),
    CreateLocationCollectionOnTable(SLocationCollection),
    CreateCardOnLocation(Location, STypes),
    CreateTokenOnLocation(SIntExpr, Token, Location),
    CreatePrecedence(Precedence, Vec<(Key, Value)>),
    CreateCombo(Combo, SFilterExpr),
    CreateMemoryIntPlayerCollection(Memory, SIntExpr, SPlayerCollection),
    CreateMemoryStringPlayerCollection(Memory, SStringExpr, SPlayerCollection),
    CreateMemoryIntTable(Memory, SIntExpr),
    CreateMemoryStringTable(Memory, SStringExpr),
    CreateMemoryPlayerCollection(Memory, SPlayerCollection),
    CreateMemoryTable(Memory),
    CreatePointMap(PointMap, Vec<(Key, Value, SIntExpr)>),
    // Actions
    FlipAction(SCardSet, SStatus),
    ShuffleAction(SCardSet),
    PlayerOutOfStageAction(SPlayerExpr),
    PlayerOutOfGameSuccAction(SPlayerExpr),
    PlayerOutOfGameFailAction(SPlayerExpr),
    PlayerCollectionOutOfStageAction(SPlayerCollection),
    PlayerCollectionOutOfGameSuccAction(SPlayerCollection),
    PlayerCollectionOutOfGameFailAction(SPlayerCollection),
    SetMemoryInt(Memory, SIntExpr),
    SetMemoryString(Memory, SStringExpr),
    SetMemoryCollection(Memory, SCollection),
    CycleAction(SPlayerExpr),
    BidAction(SQuantity),
    BidActionMemory(Memory, SQuantity),
    EndTurn,
    EndStage,
    EndGameWithWinner(SPlayerExpr),
    DemandCardPositionAction(SCardPosition),
    DemandStringAction(SStringExpr),
    DemandIntAction(SIntExpr),
    // Move-Actions
    ClassicMove(SClassicMove),
    DealMove(SDealMove),
    ExchangeMove(SExchangeMove),
    TokenMove(STokenMove),
    // Score + Winner Rule
    ScoreRule(SScoreRule),
    WinnerRule(SWinnerRule)
}

#[derive(Debug, Clone)]
pub struct Types {
    pub types: Vec<(Key, Vec<Value>)>
}

#[derive(Debug, Clone)]
pub struct SeqStage {
    pub stage: Stage,
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
pub enum ClassicMove {
    Move(SCardSet, SStatus, SCardSet),
    MoveQuantity(SQuantity, SCardSet, SStatus, SCardSet),
}

#[derive(Debug, Clone)]
pub enum DealMove {
    Deal(SCardSet, SStatus, SCardSet),
    DealQuantity(SQuantity, SCardSet, SStatus, SCardSet),
}

#[derive(Debug, Clone)]
pub enum ExchangeMove {
    Exchange(SCardSet, SStatus, SCardSet),
    ExchangeQuantity(SQuantity, SCardSet, SStatus, SCardSet),
}

#[derive(Debug, Clone)]
pub enum TokenMove {
    Place(Token, STokenLocExpr, STokenLocExpr),
    PlaceQuantity(SQuantity, Token, STokenLocExpr, STokenLocExpr),
}

#[derive(Debug, Clone)]
pub enum TokenLocExpr {
    Location(Location),
    LocationCollection(SLocationCollection),
    LocationPlayer(Location, SPlayerExpr),
    LocationCollectionPlayer(SLocationCollection, SPlayerExpr),
    LocationPlayerCollection(Location, SPlayerCollection),
    LocationCollectionPlayerCollection(SLocationCollection, SPlayerCollection),
}

#[derive(Debug, Clone)]
pub enum ScoreRule {
    ScorePlayer(SIntExpr, SPlayerExpr),
    ScorePlayerMemory(SIntExpr, Memory, SPlayerExpr),
    ScorePlayerCollection(SIntExpr, SPlayerCollection),
    ScorePlayerCollectionMemory(SIntExpr, Memory, SPlayerCollection),
}

#[derive(Debug, Clone)]
pub enum WinnerRule {
    WinnerPlayer(SPlayerExpr),
    WinnerPlayerCollection(SPlayerCollection),
    WinnerLowestScore,
    WinnerHighestScore,
    WinnerLowestMemory(Memory),
    WinnerHighestMemory(Memory),
    WinnerLowestPosition,
    WinnerHighestPosition,   
}