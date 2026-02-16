use code_gen::*;

#[spanned_ast]
pub mod ast {
    use serde::{Serialize, Deserialize};

    // Operator
    // ===========================================================================
    // ===========================================================================
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum BinCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum LogicBinOp {
        And,
        Or,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum IntOp {
        Plus,
        Minus,
        Mul,
        Div,
        Mod,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Extrema {
        Min,
        Max,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum OutOf {
        CurrentStage,
        Stage(String),
        Game,
        Play,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Groupable {
        Location(String),
        LocationCollection(LocationCollection),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Owner {
        Player(PlayerExpr),
        PlayerCollection(PlayerCollection),
        Team(TeamExpr),
        TeamCollection(TeamCollection),
        Table,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Quantity {
        Int(IntExpr),
        Quantifier(Quantifier),
        IntRange(IntRange),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IntRange {
        pub op_int: Vec<(IntCompare, IntExpr)>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Quantifier {
        All,
        Any,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum EndCondition {
        UntilBool(BoolExpr),
        UntilBoolRep(BoolExpr, LogicBinOp, Repititions),
        UntilRep(Repititions),
        UntilEnd,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Repititions {
        pub times: IntExpr,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MemoryType {
        Int(IntExpr),
        String(StringExpr),
        CardSet(CardSet),
        Collection(Collection),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Players {
        Player(PlayerExpr),
        PlayerCollection(PlayerCollection),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum EndType {
        Turn,
        Stage,
        GameWithWinner(Players),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DemandType {
        CardPosition(CardPosition),
        String(StringExpr),
        Int(IntExpr),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Types {
        pub types: Vec<(String, Vec<String>)>,
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
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RuntimePlayer {
        Current,
        Next,
        Previous,
        Competitor,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryPlayer {
        Turnorder(IntExpr),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregatePlayer {
        OwnerOfCardPostion(Box<CardPosition>),
        OwnerOfMemory(Extrema, String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum PlayerExpr {
        Literal(String),
        Runtime(RuntimePlayer),
        Aggregate(AggregatePlayer),
        Query(QueryPlayer),
    }
    // ===========================================================================

    // IntExpr
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryInt {
        IntCollectionAt(Box<IntCollection>, Box<IntExpr>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateInt {
        SizeOf(Collection),
        SumOfIntCollection(IntCollection),
        SumOfCardSet(Box<CardSet>, String),
        ExtremaCardset(Extrema, Box<CardSet>, String),
        ExtremaIntCollection(Extrema, IntCollection),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RuntimeInt {
        StageRoundCounter,
        PlayRoundCounter,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryString {
        KeyOf(String, CardPosition),
        StringCollectionAt(StringCollection, IntExpr),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum StringExpr {
        Literal(String),
        Query(QueryString),
    }
    // ===========================================================================

    // Bool
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CardSetCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum StringCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum PlayerCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TeamCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum BoolOp {
        And,
        Or,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum UnaryOp {
        Not,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CompareBool {
        Int(IntExpr, IntCompare, IntExpr),
        CardSet(CardSet, CardSetCompare, CardSet),
        String(StringExpr, StringCompare, StringExpr),
        Player(PlayerExpr, PlayerCompare, PlayerExpr),
        Team(TeamExpr, TeamCompare, TeamExpr),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateBool {
        Compare(CompareBool),
        CardSetEmpty(CardSet),
        CardSetNotEmpty(CardSet),
        OutOfPlayer(Players, OutOf),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum BoolExpr {
        Binary(Box<BoolExpr>, BoolOp, Box<BoolExpr>),
        Unary(UnaryOp, Box<BoolExpr>),
        Aggregate(AggregateBool),
    }
    // ===========================================================================

    // Team
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateTeam {
        TeamOf(PlayerExpr),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TeamExpr {
        Literal(String),
        Aggregate(AggregateTeam),
    }
    // ===========================================================================

    // CardPosition
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryCardPosition {
        At(String, IntExpr),
        Top(String),
        Bottom(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateCardPosition {
        ExtremaPointMap(Extrema, Box<CardSet>, String),
        ExtremaPrecedence(Extrema, Box<CardSet>, String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CardPosition {
        Query(QueryCardPosition),
        Aggregate(AggregateCardPosition),
    }

    // Stauts
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Collection {
        IntCollection(IntCollection),
        StringCollection(StringCollection),
        LocationCollection(LocationCollection),
        PlayerCollection(PlayerCollection),
        TeamCollection(TeamCollection),
        CardSet(Box<CardSet>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IntCollection {
        pub ints: Vec<IntExpr>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StringCollection {
        pub strings: Vec<StringExpr>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LocationCollection {
        pub locations: Vec<String>,
    }

    // PlayerCollection
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RuntimePlayerCollection {
        PlayersOut,
        PlayersIn,
        Others,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregatePlayerCollection {
        Quantifier(Quantifier),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum PlayerCollection {
        Literal(Vec<PlayerExpr>),
        Aggregate(AggregatePlayerCollection),
        Runtime(RuntimePlayerCollection),
    }
    // ===========================================================================

    // TeamCollection
    // ===========================================================================

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RuntimeTeamCollection {
        OtherTeams,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TeamCollection {
        Literal(Vec<TeamExpr>),
        Runtime(RuntimeTeamCollection),
    }

    // ===========================================================================
    // ===========================================================================
    // ===========================================================================

    // CardSet
    // ===========================================================================
    // ===========================================================================
    // ===========================================================================

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CardSet {
        Group(Group),
        GroupOwner(Group, Owner),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Group {
        Groupable(Groupable),
        Where(Groupable, FilterExpr),
        NotCombo(String, Groupable),
        Combo(String, Groupable),
        CardPosition(CardPosition),
    }

    // FilterExpr
    // ===========================================================================

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateFilter {
        Size(IntCompare, Box<IntExpr>),
        Same(String),
        Distinct(String),
        Adjacent(String, String),
        Higher(String, String),
        Lower(String, String),
        KeyString(String, StringCompare, Box<StringExpr>),
        Combo(String),
        NotCombo(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FilterOp {
        And,
        Or,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Game {
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FlowComponent {
        Stage(SeqStage),
        Rule(GameRule),
        IfRule(IfRule),
        ChoiceRule(ChoiceRule),
        OptionalRule(OptionalRule),
        Conditional(Conditional),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum SetUpRule {
        // Creations
        CreatePlayer(Vec<String>),
        CreateTeams(Vec<(String, PlayerCollection)>),
        CreateTurnorder(PlayerCollection),
        CreateTurnorderRandom(PlayerCollection),
        CreateLocation(Vec<String>, Owner),
        CreateCardOnLocation(String, Types),
        CreateTokenOnLocation(IntExpr, String, String),
        CreateCombo(String, FilterExpr),
        CreateMemoryWithMemoryType(String, MemoryType, Owner),
        CreateMemory(String, Owner),
        CreatePrecedence(String, Vec<(String, String)>),
        CreatePointMap(String, Vec<(String, String, IntExpr)>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ActionRule {
        // Actions
        FlipAction(CardSet, Status),
        ShuffleAction(CardSet),
        PlayerOutOfStageAction(Players),
        PlayerOutOfGameSuccAction(Players),
        PlayerOutOfGameFailAction(Players),
        SetMemory(String, MemoryType),
        ResetMemory(String),
        CycleAction(PlayerExpr),
        BidAction(Quantity),
        BidMemoryAction(String, Quantity),
        EndAction(EndType),
        DemandAction(DemandType),
        DemandMemoryAction(DemandType, String),
        Move(MoveType),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ScoringRule {
        // Score + Winner Rule
        ScoreRule(ScoreRule),
        WinnerRule(WinnerRule),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum GameRule {
        SetUp(SetUpRule),
        Action(ActionRule),
        Scoring(ScoringRule),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SeqStage {
        pub stage: String,
        pub player: PlayerExpr,
        pub end_condition: EndCondition,
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Case {
        Else(Vec<FlowComponent>),
        NoBool(Vec<FlowComponent>),
        Bool(BoolExpr, Vec<FlowComponent>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Conditional {
        pub cases: Vec<Case>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IfRule {
        pub condition: BoolExpr,
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OptionalRule {
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChoiceRule {
        pub options: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MoveType {
        Deal(DealMove),
        Exchange(ExchangeMove),
        Classic(ClassicMove),
        Place(TokenMove),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MoveCardSet {
        Move(CardSet, Status, CardSet),
        MoveQuantity(Quantity, CardSet, Status, CardSet),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ClassicMove {
        MoveCardSet(MoveCardSet),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DealMove {
        MoveCardSet(MoveCardSet),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ExchangeMove {
        MoveCardSet(MoveCardSet),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TokenMove {
        Place(String, TokenLocExpr, TokenLocExpr),
        PlaceQuantity(Quantity, String, TokenLocExpr, TokenLocExpr),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TokenLocExpr {
        Groupable(Groupable),
        GroupablePlayers(Groupable, Players),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ScoreRule {
        Score(IntExpr, Players),
        ScoreMemory(IntExpr, String, Players),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum WinnerType {
        Score,
        Memory(String),
        Position,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum WinnerRule {
        Winner(Players),
        WinnerWith(Extrema, WinnerType),
    }

    // ===========================================================================
    // ===========================================================================
    // ===========================================================================
}