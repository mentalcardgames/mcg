// IDs
pub struct ID(pub String);

impl ToString for ID {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub type PlayerName = String;
pub type TeamName = String;
pub type Location = String;
pub type Precedence = String;
pub type PointMap = String;
pub type Combo = String;
pub type Key = String;
pub type Value = String;
pub type Memory = String;
pub type Token = String;
pub type Stage = String;

// Structs + Enums
#[derive(Debug, PartialEq, Clone)]
pub enum PlayerExpr {
    PlayerName(PlayerName),
    Current,
    Next,
    Previous,
    Competitor,
    Turnorder(IntExpr),
    OwnerOf(Box<CardPosition>),
    OwnerOfHighest(Memory),
    OwnerOfLowest(Memory),
}

#[derive(Debug, PartialEq, Clone)]
pub enum IntExpr {
    Int(i32),
    IntOp(Box<IntExpr>, Op, Box<IntExpr>),
    IntCollectionAt(Box<IntExpr>),
    SizeOf(Collection),
    SumOfIntCollection(IntCollection),
    SumOfCardSet(Box<CardSet>, PointMap),
    MinOf(Box<CardSet>, PointMap),
    MaxOf(Box<CardSet>, PointMap),
    MinIntCollection(IntCollection),
    MaxIntCollection(IntCollection),
    StageRoundCounter,
    // PlayRoundCounter,
}

#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
pub enum Collection {
    IntCollection(IntCollection),
    StringCollection(StringCollection),
    LocationCollection(LocationCollection),
    PlayerCollection(PlayerCollection),
    TeamCollection(TeamCollection),
    CardSet(Box<CardSet>),
    Ambiguous(Vec<String>),
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
    pub locations: Vec<Location>
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
    // I think StringExpr ID has no value.
    // It has no type and therefore
    // can't be checked or use securely.
    // ID(ID),
    KeyOf(Key, CardPosition),
    StringCollectionAt(StringCollection, IntExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CardPosition {
    At(Location, IntExpr),
    Top(Location),
    Bottom(Location),
    // Analyzer decides afterwards if it is Precedence or PointMap
    Max (Box<CardSet>, String),
    Min (Box<CardSet>, String),
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
    // Catch case if we have something like P1 == P2 or T1 == T2
    // Matching IDs like P1 == P2 should not be done and will not be handled.
    // This will directly be interpreted as CardSet! 
    // AmbiguousEq (String, String),
    // AmbiguousNeq(String, String),
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
    TeamName(TeamName),
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
    Location(Location),
    LocationWhere(Location, FilterExpr),
    LocationCollection(LocationCollection),
    LocationCollectionWhere(LocationCollection, FilterExpr),
    ComboInLocation(Combo, Location),
    ComboInLocationCollection(Combo, LocationCollection),
    NotComboInLocation(Combo, Location),
    NotComboInLocationCollection(Combo, LocationCollection),
    CardPosition(CardPosition),
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterExpr {
    Same(Key),
    Distinct(Key),
    Adjacent(Key, Precedence),
    Higher(Key, Precedence),
    Lower(Key, Precedence),
    Size (IntCmpOp, Box<IntExpr>),
    KeyEqString  (Key, Box<StringExpr>),
    KeyNeqString (Key, Box<StringExpr>),
    KeyEqValue  (Key, Value),
    KeyNeqValue (Key, Value),
    NotCombo(Combo),
    Combo(Combo),
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
    CreatePlayer(Vec<PlayerName>),
    CreateTeam(TeamName, Vec<PlayerName>),
    CreateTurnorder(Vec<PlayerName>),
    CreateTurnorderRandom(Vec<PlayerName>),
    CreateLocationOnPlayerCollection(Location, PlayerCollection),
    CreateLocationOnTeamCollection(Location, TeamCollection),
    CreateLocationOnTable(Location),
    CreateLocationCollectionOnPlayerCollection(LocationCollection, PlayerCollection),
    CreateLocationCollectionOnTeamCollection(LocationCollection, TeamCollection),
    CreateLocationCollectionOnTable(LocationCollection),
    CreateCardOnLocation(Location, Types),
    CreateTokenOnLocation(IntExpr, Token, Location),
    CreatePrecedence(Precedence, Vec<(Key, Value)>),
    CreateCombo(Combo, FilterExpr),
    CreateMemoryIntPlayerCollection(Memory, IntExpr, PlayerCollection),
    CreateMemoryStringPlayerCollection(Memory, StringExpr, PlayerCollection),
    CreateMemoryIntTable(Memory, IntExpr),
    CreateMemoryStringTable(Memory, StringExpr),
    CreateMemoryPlayerCollection(Memory, PlayerCollection),
    CreateMemoryTable(Memory),
    CreatePointMap(PointMap, Vec<(Key, Value, IntExpr)>),
    // Actions
    FlipAction(CardSet, Status),
    ShuffleAction(CardSet),
    PlayerOutOfStageAction(PlayerExpr),
    PlayerOutOfGameSuccAction(PlayerExpr),
    PlayerOutOfGameFailAction(PlayerExpr),
    PlayerCollectionOutOfStageAction(PlayerCollection),
    PlayerCollectionOutOfGameSuccAction(PlayerCollection),
    PlayerCollectionOutOfGameFailAction(PlayerCollection),
    SetMemoryInt(Memory, IntExpr),
    SetMemoryString(Memory, StringExpr),
    SetMemoryCollection(Memory, Collection),
    CycleAction(PlayerExpr),
    BidAction(Quantity),
    BidActionMemory(Memory, Quantity),
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
    pub types: Vec<(Key, Vec<Value>)>
}

#[derive(Debug, PartialEq, Clone)]
pub struct SeqStage {
    pub stage: Stage,
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
    Place(Token, TokenLocExpr, TokenLocExpr),
    PlaceQuantity(Quantity, Token, TokenLocExpr, TokenLocExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenLocExpr {
    Location(Location),
    LocationCollection(LocationCollection),
    LocationPlayer(Location, PlayerExpr),
    LocationCollectionPlayer(LocationCollection, PlayerExpr),
    LocationPlayerCollection(Location, PlayerCollection),
    LocationCollectionPlayerCollection(LocationCollection, PlayerCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ScoreRule {
    ScorePlayer(IntExpr, PlayerExpr),
    ScorePlayerMemory(IntExpr, Memory, PlayerExpr),
    ScorePlayerCollection(IntExpr, PlayerCollection),
    ScorePlayerCollectionMemory(IntExpr, Memory, PlayerCollection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum WinnerRule {
    WinnerPlayer(PlayerExpr),
    WinnerPlayerCollection(PlayerCollection),
    WinnerLowestScore,
    WinnerHighestScore,
    WinnerLowestMemory(Memory),
    WinnerHighestMemory(Memory),
    WinnerLowestPosition,
    WinnerHighestPosition,   
}