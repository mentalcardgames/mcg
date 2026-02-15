use code_gen::*;

#[ast]
pub mod ast {
    use crate::spans::*;
    use crate::walker::*;
    use crate::lower::*;
    
    use serde::{Serialize, Deserialize};

    pub type SID = Spanned<String>;

    // Operator
    // ===========================================================================
    // ===========================================================================
    // ===========================================================================

    pub type SBinCompare = Spanned<BinCompare>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum BinCompare {
        Eq,
        Neq,
    }


    pub type SLogicBinOp = Spanned<LogicBinOp>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum LogicBinOp {
        And,
        Or,
    }


    pub type SIntOp = Spanned<IntOp>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum IntOp {
        Plus,
        Minus,
        Mul,
        Div,
        Mod,
    }


    pub type SIntCompare = Spanned<IntCompare>;

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

    pub type SExtrema = Spanned<Extrema>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Extrema {
        Min,
        Max,
    }


    pub type SOutOf = Spanned<OutOf>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum OutOf {
        CurrentStage,
        Stage(SID),
        Game,
        Play,
    }


    pub type SGroupable = Spanned<Groupable>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Groupable {
        Location(SID),
        LocationCollection(SLocationCollection),
    }


    pub type SOwner = Spanned<Owner>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Owner {
        Player(SPlayerExpr),
        PlayerCollection(SPlayerCollection),
        Team(STeamExpr),
        TeamCollection(STeamCollection),
        Table,
    }


    pub type SQuantity = Spanned<Quantity>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Quantity {
        Int(SIntExpr),
        Quantifier(SQuantifier),
        IntRange(SIntRange),
    }


    pub type SIntRange = Spanned<IntRange>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IntRange {
        pub op_int: Vec<(SIntCompare, SIntExpr)>,
    }


    pub type SQuantifier = Spanned<Quantifier>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Quantifier {
        All,
        Any,
    }


    pub type SEndCondition = Spanned<EndCondition>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum EndCondition {
        UntilBool(SBoolExpr),
        UntilBoolRep(SBoolExpr, SLogicBinOp, SRepititions),
        UntilRep(SRepititions),
        UntilEnd,
    }


    pub type SRepititions = Spanned<Repititions>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Repititions {
        pub times: SIntExpr,
    }


    pub type SMemoryType = Spanned<MemoryType>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MemoryType {
        Int(SIntExpr),
        String(SStringExpr),
        CardSet(SCardSet),
        Collection(SCollection),
    }


    pub type SPlayers = Spanned<Players>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Players {
        Player(SPlayerExpr),
        PlayerCollection(SPlayerCollection),
    }


    pub type SEndType = Spanned<EndType>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum EndType {
        Turn,
        Stage,
        GameWithWinner(SPlayers),
    }


    pub type SDemandType = Spanned<DemandType>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DemandType {
        CardPosition(SCardPosition),
        String(SStringExpr),
        Int(SIntExpr),
    }


    pub type STypes = Spanned<Types>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub type SRuntimePlayer = Spanned<RuntimePlayer>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RuntimePlayer {
        Current,
        Next,
        Previous,
        Competitor,
    }


    pub type SQueryPlayer = Spanned<QueryPlayer>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryPlayer {
        Turnorder(SIntExpr),
    }


    pub type SAggregatePlayer = Spanned<AggregatePlayer>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregatePlayer {
        OwnerOfCardPostion(Box<SCardPosition>),
        OwnerOfMemory(SExtrema, SID),
    }


    pub type SPlayerExpr = Spanned<PlayerExpr>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum PlayerExpr {
        Literal(SID),
        Runtime(SRuntimePlayer),
        Aggregate(SAggregatePlayer),
        Query(SQueryPlayer),
    }
    // ===========================================================================

    // IntExpr
    // ===========================================================================

    pub type SQueryInt = Spanned<QueryInt>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryInt {
        IntCollectionAt(Box<SIntCollection>, Box<SIntExpr>),
    }


    pub type SAggregateInt = Spanned<AggregateInt>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateInt {
        SizeOf(SCollection),
        SumOfIntCollection(SIntCollection),
        SumOfCardSet(Box<SCardSet>, SID),
        ExtremaCardset(SExtrema, Box<SCardSet>, SID),
        ExtremaIntCollection(SExtrema, SIntCollection),
    }


    pub type SRuntimeInt = Spanned<RuntimeInt>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RuntimeInt {
        StageRoundCounter,
        PlayRoundCounter,
    }


    pub type SIntExpr = Spanned<IntExpr>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub type SQueryString = Spanned<QueryString>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryString {
        KeyOf(SID, SCardPosition),
        StringCollectionAt(SStringCollection, SIntExpr),
    }


    pub type SStringExpr = Spanned<StringExpr>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum StringExpr {
        Literal(SID),
        Query(SQueryString),
    }
    // ===========================================================================

    // Bool
    // ===========================================================================

    pub type SCardSetCompare = Spanned<CardSetCompare>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CardSetCompare {
        Eq,
        Neq,
    }


    pub type SStringCompare = Spanned<StringCompare>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum StringCompare {
        Eq,
        Neq,
    }


    pub type SPlayerCompare = Spanned<PlayerCompare>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum PlayerCompare {
        Eq,
        Neq,
    }


    pub type STeamCompare = Spanned<TeamCompare>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TeamCompare {
        Eq,
        Neq,
    }


    pub type SBoolOp = Spanned<BoolOp>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum BoolOp {
        And,
        Or,
    }


    pub type SUnaryOp = Spanned<UnaryOp>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum UnaryOp {
        Not,
    }


    pub type SCompareBool = Spanned<CompareBool>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CompareBool {
        Int(SIntExpr, SIntCompare, SIntExpr),
        CardSet(SCardSet, SCardSetCompare, SCardSet),
        String(SStringExpr, SStringCompare, SStringExpr),
        Player(SPlayerExpr, SPlayerCompare, SPlayerExpr),
        Team(STeamExpr, STeamCompare, STeamExpr),
    }


    pub type SAggregateBool = Spanned<AggregateBool>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateBool {
        Compare(SCompareBool),
        CardSetEmpty(SCardSet),
        CardSetNotEmpty(SCardSet),
        OutOfPlayer(SPlayers, SOutOf),
    }


    pub type SBoolExpr = Spanned<BoolExpr>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum BoolExpr {
        Binary(Box<SBoolExpr>, SBoolOp, Box<SBoolExpr>),
        Unary(SUnaryOp, Box<SBoolExpr>),
        Aggregate(SAggregateBool),
    }
    // ===========================================================================

    // Team
    // ===========================================================================

    pub type SAggregateTeam = Spanned<AggregateTeam>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateTeam {
        TeamOf(SPlayerExpr),
    }


    pub type STeamExpr = Spanned<TeamExpr>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TeamExpr {
        Literal(SID),
        Aggregate(SAggregateTeam),
    }
    // ===========================================================================

    // CardPosition
    // ===========================================================================

    pub type SQueryCardPosition = Spanned<QueryCardPosition>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryCardPosition {
        At(SID, SIntExpr),
        Top(SID),
        Bottom(SID),
    }


    pub type SAggregateCardPosition = Spanned<AggregateCardPosition>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregateCardPosition {
        ExtremaPointMap(SExtrema, Box<SCardSet>, SID),
        ExtremaPrecedence(SExtrema, Box<SCardSet>, SID),
    }


    pub type SCardPosition = Spanned<CardPosition>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CardPosition {
        Query(SQueryCardPosition),
        Aggregate(SAggregateCardPosition),
    }

    // Stauts
    // ===========================================================================

    pub type SStatus = Spanned<Status>;

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


    pub type SCollection = Spanned<Collection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Collection {
        IntCollection(SIntCollection),
        StringCollection(SStringCollection),
        LocationCollection(SLocationCollection),
        PlayerCollection(SPlayerCollection),
        TeamCollection(STeamCollection),
        CardSet(Box<SCardSet>),
    }


    pub type SIntCollection = Spanned<IntCollection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IntCollection {
        pub ints: Vec<SIntExpr>,
    }


    pub type SStringCollection = Spanned<StringCollection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StringCollection {
        pub strings: Vec<SStringExpr>,
    }


    pub type SLocationCollection = Spanned<LocationCollection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LocationCollection {
        pub locations: Vec<SID>,
    }

    // PlayerCollection
    // ===========================================================================

    pub type SRuntimePlayerCollection = Spanned<RuntimePlayerCollection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RuntimePlayerCollection {
        PlayersOut,
        PlayersIn,
        Others,
    }


    pub type SAggregatePlayerCollection = Spanned<AggregatePlayerCollection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AggregatePlayerCollection {
        Quantifier(SQuantifier),
    }


    pub type SPlayerCollection = Spanned<PlayerCollection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum PlayerCollection {
        Literal(Vec<SPlayerExpr>),
        Aggregate(SAggregatePlayerCollection),
        Runtime(SRuntimePlayerCollection),
    }
    // ===========================================================================

    // TeamCollection
    // ===========================================================================

    pub type SRuntimeTeamCollection = Spanned<RuntimeTeamCollection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RuntimeTeamCollection {
        OtherTeams,
    }


    pub type STeamCollection = Spanned<TeamCollection>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
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


    pub type SCardSet = Spanned<CardSet>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CardSet {
        Group(SGroup),
        GroupOwner(SGroup, SOwner),
    }


    pub type SGroup = Spanned<Group>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Group {
        Groupable(SGroupable),
        Where(SGroupable, SFilterExpr),
        NotCombo(SID, SGroupable),
        Combo(SID, SGroupable),
        CardPosition(SCardPosition),
    }

    // FilterExpr
    // ===========================================================================

    pub type SAggregateFilter = Spanned<AggregateFilter>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
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


    pub type SFilterOp = Spanned<FilterOp>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FilterOp {
        And,
        Or,
    }


    pub type SFilterExpr = Spanned<FilterExpr>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
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


    pub type SGame = Spanned<Game>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Game {
        pub flows: Vec<SFlowComponent>,
    }


    pub type SFlowComponent = Spanned<FlowComponent>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FlowComponent {
        Stage(SSeqStage),
        Rule(SGameRule),
        IfRule(SIfRule),
        ChoiceRule(SChoiceRule),
        OptionalRule(SOptionalRule),
        Conditional(SConditional),
    }


    pub type SSetUpRule = Spanned<SetUpRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
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
        CreateMemoryWithMemoryType(SID, SMemoryType, SOwner),
        CreateMemory(SID, SOwner),
        CreatePrecedence(SID, Vec<(SID, SID)>),
        CreatePointMap(SID, Vec<(SID, SID, SIntExpr)>),
    }


    pub type SActionRule = Spanned<ActionRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
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


    pub type SScoringRule = Spanned<ScoringRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ScoringRule {
        // Score + Winner Rule
        ScoreRule(SScoreRule),
        WinnerRule(SWinnerRule),
    }


    pub type SGameRule = Spanned<GameRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum GameRule {
        SetUp(SSetUpRule),
        Action(SActionRule),
        Scoring(SScoringRule),
    }


    pub type SSeqStage = Spanned<SeqStage>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SeqStage {
        pub stage: SID,
        pub player: SPlayerExpr,
        pub end_condition: SEndCondition,
        pub flows: Vec<SFlowComponent>,
    }


    pub type SCase = Spanned<Case>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Case {
        Else(Vec<SFlowComponent>),
        NoBool(Vec<SFlowComponent>),
        Bool(SBoolExpr, Vec<SFlowComponent>),
    }


    pub type SConditional = Spanned<Conditional>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Conditional {
        pub cases: Vec<SCase>,
    }

    pub type SIfRule = Spanned<IfRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IfRule {
        pub condition: SBoolExpr,
        pub flows: Vec<SFlowComponent>,
    }


    pub type SOptionalRule = Spanned<OptionalRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OptionalRule {
        pub flows: Vec<SFlowComponent>,
    }


    pub type SChoiceRule = Spanned<ChoiceRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChoiceRule {
        pub options: Vec<SFlowComponent>,
    }


    pub type SMoveType = Spanned<MoveType>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MoveType {
        Deal(SDealMove),
        Exchange(SExchangeMove),
        Classic(SClassicMove),
        Place(STokenMove),
    }


    pub type SMoveCardSet = Spanned<MoveCardSet>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MoveCardSet {
        Move(SCardSet, SStatus, SCardSet),
        MoveQuantity(SQuantity, SCardSet, SStatus, SCardSet),
    }


    pub type SClassicMove = Spanned<ClassicMove>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ClassicMove {
        MoveCardSet(SMoveCardSet),
    }


    pub type SDealMove = Spanned<DealMove>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DealMove {
        MoveCardSet(SMoveCardSet),
    }


    pub type SExchangeMove = Spanned<ExchangeMove>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ExchangeMove {
        MoveCardSet(SMoveCardSet),
    }


    pub type STokenMove = Spanned<TokenMove>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TokenMove {
        Place(SID, STokenLocExpr, STokenLocExpr),
        PlaceQuantity(SQuantity, SID, STokenLocExpr, STokenLocExpr),
    }


    pub type STokenLocExpr = Spanned<TokenLocExpr>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TokenLocExpr {
        Groupable(SGroupable),
        GroupablePlayers(SGroupable, SPlayers),
    }


    pub type SScoreRule = Spanned<ScoreRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ScoreRule {
        Score(SIntExpr, SPlayers),
        ScoreMemory(SIntExpr, SID, SPlayers),
    }


    pub type SWinnerType = Spanned<WinnerType>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum WinnerType {
        Score,
        Memory(SID),
        Position,
    }


    pub type SWinnerRule = Spanned<WinnerRule>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum WinnerRule {
        Winner(SPlayers),
        WinnerWith(SExtrema, SWinnerType),
    }

    // ===========================================================================
    // ===========================================================================
    // ===========================================================================
}