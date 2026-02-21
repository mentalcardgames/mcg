use code_gen::*;

#[spanned_ast]
pub mod ast {
    use arbitrary::Arbitrary;
    use serde::{Serialize, Deserialize};
    use crate::arbitrary::{
        gen_vec_min_1,
        gen_team_name, 
        gen_vec_strings, 
        gen_vec_players_prefixed, 
        gen_ident, 
        gen_vec_teams_with_players,
        gen_types_and_subtypes,
        gen_vec_min_1_kvs,
        gen_vec_min_1_kvis,
        gen_vec_min_1_ints,
        gen_flows_safe,
    };

    // Operator
    // ===========================================================================
    // ===========================================================================
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum BinCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum LogicBinOp {
        And,
        Or,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum IntOp {
        Plus,
        Minus,
        Mul,
        Div,
        Mod,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
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
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum UseMemory {
        Memory { 
            #[arbitrary(with = gen_ident)]
            memory: String
        },
        WithOwner { 
            #[arbitrary(with = gen_ident)]
            memory: String,
            owner: Box<Owner>
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum Extrema {
        Min,
        Max,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum OutOf {
        CurrentStage,
        Stage { 
            #[arbitrary(with = gen_ident)]
            name: String 
        },
        Game,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum Groupable {
        Location { 
            #[arbitrary(with = gen_ident)]
            name: String
        },
        LocationCollection { location_collection: LocationCollection },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum Owner {
        Player { player: PlayerExpr },
        PlayerCollection { player_collection: PlayerCollection},
        Team{ team: TeamExpr},
        TeamCollection {team_collection: TeamCollection},
        Table,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum Quantity {
        Int {int: IntExpr},
        Quantifier {qunatifier: Quantifier},
        IntRange {int_range: IntRange},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum IntRangeOperator {
        And,
        Or
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct IntRange {
        pub start: (IntCompare, IntExpr),
        #[arbitrary(with = gen_vec_min_1)]
        pub op_int: Vec<(IntRangeOperator, IntCompare, IntExpr)>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum Quantifier {
        All,
        Any,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum EndCondition {
        UntilBool {bool_expr: BoolExpr},
        UntilBoolRep {bool_expr: BoolExpr, logic: LogicBinOp, reps: Repititions},
        UntilRep {reps: Repititions},
        UntilEnd,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct Repititions {
        pub times: IntExpr,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum MemoryType {
        Int {int: IntExpr},
        String {string: StringExpr },
        PlayerCollection { players: PlayerCollection},
        StringCollection { strings: StringCollection},
        TeamCollection { teams: TeamCollection},
        IntCollection { ints: IntCollection},
        LocationCollection { locations: LocationCollection},
        CardSet { card_set: CardSet },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum Players {
        Player { player: PlayerExpr},
        PlayerCollection {player_collection: PlayerCollection},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum EndType {
        Turn,
        Stage,
        GameWithWinner {players: Players},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum DemandType {
        CardPosition {card_position: CardPosition},
        String {string: StringExpr},
        Int{ int: IntExpr},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct Types {
        #[arbitrary(with = gen_types_and_subtypes)]
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
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum RuntimePlayer {
        Current,
        Next,
        Previous,
        Competitor,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum QueryPlayer {
        Turnorder {int: IntExpr},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum AggregatePlayer {
        OwnerOfCardPostion {card_position: Box<CardPosition>},
        OwnerOfMemory {
            extrema: Extrema, 
            #[arbitrary(with = gen_ident)]
            memory: String 
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum PlayerExpr {
        Literal { 
            name: String 
        },
        Runtime {runtime: RuntimePlayer},
        Aggregate {aggregate: AggregatePlayer},
        Query {query: QueryPlayer},
    }
    // ===========================================================================

    // IntExpr
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum QueryInt {
        IntCollectionAt { int_collection: Box<IntCollection>, int_expr: Box<IntExpr> },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum AggregateInt {
        SizeOf {collection: Collection},
        SumOfIntCollection {int_collection: IntCollection},
        SumOfCardSet{ 
            card_set:Box<CardSet>, 
            #[arbitrary(with = gen_ident)]
            pointmap: String 
        },
        ExtremaCardset {
            extrema: Extrema, 
            card_set: Box<CardSet>, 
            #[arbitrary(with = gen_ident)]
            pointmap: String 
        },
        ExtremaIntCollection {extrema: Extrema, int_collection: IntCollection},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum RuntimeInt {
        StageRoundCounter,
        PlayRoundCounter,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum IntExpr {
        Literal { int: i32},
        Binary {int: Box<IntExpr>, op: IntOp, int1: Box<IntExpr>},
        Query {query: QueryInt},
        Aggregate {aggregate: AggregateInt},
        Runtime {runtime: RuntimeInt },
        Memory { 
            memory: UseMemory,
        },
    }
    // ===========================================================================

    // String
    // ===========================================================================

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum QueryString {
        KeyOf{ 
            #[arbitrary(with = gen_ident)]
            key: String, 
            card_position:CardPosition
        },
        StringCollectionAt { string_collection: StringCollection, int_expr: IntExpr },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum StringExpr {
        Literal {
            // #[arbitrary(with = gen_ident)]
            value: String
        },
        Query {query: QueryString},
        Memory { memory: UseMemory },
    }
    // ===========================================================================

    // Bool
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum CardSetCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum StringCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum PlayerCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum TeamCompare {
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum BoolOp {
        And,
        Or,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum UnaryOp {
        Not,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum CompareBool {
        Int { int: IntExpr, cmp: IntCompare, int1: IntExpr },
        CardSet{card_set: CardSet, cmp: CardSetCompare, card_set1: CardSet},
        String{string: StringExpr, cmp: StringCompare, string1: StringExpr},
        Player{player: PlayerExpr, cmp: PlayerCompare, player1: PlayerExpr},
        Team{team: TeamExpr, cmp: TeamCompare, team1: TeamExpr},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum AggregateBool {
        Compare { cmp_bool: CompareBool},
        CardSetEmpty{card_set: CardSet},
        CardSetNotEmpty{card_set: CardSet},
        OutOfPlayer{players: Players, out_of: OutOf},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum BoolExpr {
        Binary{ bool_expr: Box<BoolExpr>, op: BoolOp, bool_expr1: Box<BoolExpr>},
        Unary { op: UnaryOp, bool_expr: Box<BoolExpr> },
        Aggregate{aggregate: AggregateBool},
    }
    // ===========================================================================

    // Team
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum AggregateTeam {
        TeamOf { player: PlayerExpr },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum TeamExpr {
        Literal {
            #[arbitrary(with = gen_team_name)]
            name: String
        },
        Aggregate{ aggregate: AggregateTeam},
    }
    // ===========================================================================

    // CardPosition
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum QueryCardPosition {
        At {
            #[arbitrary(with = gen_ident)]
            location: String, 
            int_expr:IntExpr
        },
        Top{ 
            #[arbitrary(with = gen_ident)]
            location: String
        },
        Bottom {
            #[arbitrary(with = gen_ident)]
            location: String
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum AggregateCardPosition {
        ExtremaPointMap { extrema: Extrema, card_set: Box<CardSet>, 
            #[arbitrary(with = gen_ident)]
            pointmap: String 
        },
        ExtremaPrecedence { extrema: Extrema, card_set: Box<CardSet>, 
            #[arbitrary(with = gen_ident)]
            precedence: String 
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum CardPosition {
        Query { query: QueryCardPosition },
        Aggregate { aggregate: AggregateCardPosition },
    }

    // Stauts
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
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
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum Collection {
        IntCollection { int: IntCollection },
        StringCollection { string: StringCollection },
        LocationCollection { location: LocationCollection},
        PlayerCollection {player: PlayerCollection },
        TeamCollection { team: TeamCollection },
        CardSet { card_set: Box<CardSet> },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum IntCollection {
        Literal {     
            #[arbitrary(with = gen_vec_min_1_ints)]
            ints: Vec<IntExpr>
        },
        Memory { 
            memory: UseMemory
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum StringCollection {
        Literal { 
            #[arbitrary(with = gen_vec_min_1)]
            strings: Vec<StringExpr>
        },
        Memory { 
            memory: UseMemory
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum LocationCollection {
        Literal { 
            #[arbitrary(with = gen_vec_strings)]
            locations: Vec<String>
        },
        Memory { 
            memory: UseMemory
        },
    }

    // PlayerCollection
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum RuntimePlayerCollection {
        PlayersOut,
        PlayersIn,
        Others,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum AggregatePlayerCollection {
        Quantifier { quantifier: Quantifier },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum PlayerCollection {
        Literal {
            players: Vec<PlayerExpr> 
        },
        Aggregate { aggregate: AggregatePlayerCollection },
        Runtime { runtime: RuntimePlayerCollection },
        Memory { memory: UseMemory },
    }
    // ===========================================================================

    // TeamCollection
    // ===========================================================================

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum RuntimeTeamCollection {
        OtherTeams,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum TeamCollection {
        Literal { 
            teams: Vec<TeamExpr> 
        },
        Runtime {runtime: RuntimeTeamCollection },
        Memory { memory: UseMemory },
    }

    // ===========================================================================
    // ===========================================================================
    // ===========================================================================

    // CardSet
    // ===========================================================================
    // ===========================================================================
    // ===========================================================================

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum CardSet {
        Group { group: Group },
        GroupOwner { group: Group, owner: Owner},
        // CardSet is already inside of Collection!
        Memory { 
            memory: UseMemory
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum Group {
        Groupable { groupable: Groupable },
        Where { groupable: Groupable, filter: FilterExpr},
        NotCombo { combo: String, groupable: Groupable },
        Combo{ combo: String, groupable: Groupable },
        CardPosition{ card_position: CardPosition},
    }

    // FilterExpr
    // ===========================================================================

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum AggregateFilter {
        Size { cmp: IntCompare, int_expr: Box<IntExpr> },
        Same {
            #[arbitrary(with = gen_ident)]
            key: String
        },
        Distinct{ 
            #[arbitrary(with = gen_ident)]
            key: String
        },
        Adjacent {
            #[arbitrary(with = gen_ident)]
            key: String, 
            #[arbitrary(with = gen_ident)]
            precedence: String },
        Higher{ 
            #[arbitrary(with = gen_ident)]
            key: String,
            value: StringExpr,
            #[arbitrary(with = gen_ident)]
            precedence: String },
        Lower{
            #[arbitrary(with = gen_ident)]
            key: String, 
            value: StringExpr,
            #[arbitrary(with = gen_ident)]
            precedence: String},
        KeyString{ 
            #[arbitrary(with = gen_ident)]
            key: String, 
            cmp: StringCompare, string: Box<StringExpr>},
        Combo {
            #[arbitrary(with = gen_ident)]
            combo: String
        },
        NotCombo {
            #[arbitrary(with = gen_ident)]
            combo: String
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum FilterOp {
        And,
        Or,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum FilterExpr {
        Aggregate {aggregate: AggregateFilter},
        Binary { filter: Box<FilterExpr>, op: FilterOp, filter1: Box<FilterExpr>},
    }
    // ===========================================================================

    // ===========================================================================
    // ===========================================================================
    // ===========================================================================

    // Game + Stage + FlowComponent + Rule
    // ===========================================================================
    // ===========================================================================
    // ===========================================================================
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct Game {
        #[arbitrary(with = gen_flows_safe)]
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum FlowComponent {
        SeqStage {stage: SeqStage},
        SimStage {stage: SimStage},
        Rule{game_rule: GameRule},
        IfRule{if_rule: IfRule},
        ChoiceRule {choice_rule: ChoiceRule},
        OptionalRule{ optional_rule: OptionalRule},
        Conditional {conditional: Conditional},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum SetUpRule {
        // Creations
        CreatePlayer { 
            #[arbitrary(with = gen_vec_players_prefixed)]
            players: Vec<String> 
        },
        CreateTeams {
            #[arbitrary(with = gen_vec_teams_with_players)]
            teams: Vec<(String, PlayerCollection)>
        },
        CreateTurnorder {player_collection: PlayerCollection},
        CreateTurnorderRandom { player_collection: PlayerCollection},
        CreateLocation { 
            #[arbitrary(with = gen_vec_strings)]
            locations: Vec<String>, 
            owner: Owner 
        },
        CreateCardOnLocation { 
            #[arbitrary(with = gen_ident)]
            location: String, 
            types: Types 
        },
        CreateTokenOnLocation { int: IntExpr, 
            #[arbitrary(with = gen_ident)]
            token: String,
            #[arbitrary(with = gen_ident)]
            location: String 
        },
        CreateCombo {
            #[arbitrary(with = gen_ident)]
            combo: String, 
            filter: FilterExpr
        },
        CreateMemoryWithMemoryType {
            #[arbitrary(with = gen_ident)]
            memory: String,
            memory_type: MemoryType, 
            owner: Owner
        },
        CreateMemory { 
            #[arbitrary(with = gen_ident)]
            memory: String,
            owner: Owner 
        },
        CreatePrecedence {
            #[arbitrary(with = gen_ident)]
            precedence: String, 
            #[arbitrary(with = gen_vec_min_1_kvs)]
            kvs: Vec<(String, String)>
        },
        CreatePointMap { 
            #[arbitrary(with = gen_ident)]
            pointmap: String, 
            #[arbitrary(with = gen_vec_min_1_kvis)]
            kvis: Vec<(String, String, IntExpr)> 
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum ActionRule {
        // Actions
        FlipAction { card_set: CardSet, status: Status },
        ShuffleAction {card_set: CardSet},
        PlayerOutOfStageAction {players: Players},
        PlayerOutOfGameSuccAction {players: Players},
        PlayerOutOfGameFailAction{players: Players},
        SetMemory{
            #[arbitrary(with = gen_ident)]
            memory: String,
            memory_type: MemoryType},
        ResetMemory {
            #[arbitrary(with = gen_ident)]
            memory: String
        },
        CycleAction {player:PlayerExpr},
        BidAction{quantitiy: Quantity},
        BidMemoryAction{ 
            #[arbitrary(with = gen_ident)]
            memory: String,
            quantity: Quantity,
            owner: Owner,
        },
        EndAction{end_type: EndType},
        DemandAction{demand_type: DemandType},
        DemandMemoryAction{demand_type: DemandType, 
            #[arbitrary(with = gen_ident)]
            memory: String
        },
        Move{move_type: MoveType},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum ScoringRule {
        // Score + Winner Rule
        ScoreRule{score_rule: ScoreRule},
        WinnerRule{ winner_rule: WinnerRule},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum GameRule {
        SetUp { setup: SetUpRule},
        Action{ action: ActionRule},
        Scoring{scoring: ScoringRule},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct SeqStage {
        #[arbitrary(with = gen_ident)]
        pub stage: String,
        pub player: PlayerExpr,
        pub end_condition: EndCondition,
        #[arbitrary(with = gen_flows_safe)]
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct SimStage {
        #[arbitrary(with = gen_ident)]
        pub stage: String,
        pub players: PlayerCollection,
        pub end_condition: EndCondition,
        #[arbitrary(with = gen_flows_safe)]
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum Case {
        // Else {
        //     #[arbitrary(with = gen_vec_min_1)]
        //     flows: Vec<FlowComponent>
        // },
        NoBool{
            #[arbitrary(with = gen_flows_safe)]
            flows: Vec<FlowComponent>
        },
        Bool{
            bool_expr: BoolExpr, 
            #[arbitrary(with = gen_flows_safe)]
            flows: Vec<FlowComponent>
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct Conditional {
        #[arbitrary(with = gen_vec_min_1)]
        pub cases: Vec<Case>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct IfRule {
        pub condition: BoolExpr,
        #[arbitrary(with = gen_flows_safe)]
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct OptionalRule {
        #[arbitrary(with = gen_flows_safe)]
        pub flows: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub struct ChoiceRule {
        #[arbitrary(with = gen_flows_safe)]
        pub options: Vec<FlowComponent>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum MoveType {
        Deal { deal: DealMove },
        Exchange { exchange: ExchangeMove},
        Classic{ classic: ClassicMove},
        Place{ token: TokenMove},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum MoveCardSet {
        Move { from: CardSet, status: Status, to: CardSet },
        MoveQuantity { quantity: Quantity, from: CardSet, status: Status, to: CardSet},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum ClassicMove {
        MoveCardSet {move_cs: MoveCardSet},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum DealMove {
        MoveCardSet {deal_cs: MoveCardSet},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum ExchangeMove {
        MoveCardSet {exchange_cs: MoveCardSet},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum TokenMove {
        Place {
            #[arbitrary(with = gen_ident)]
            token: String,
            from_loc: TokenLocExpr, to_loc: TokenLocExpr},
        PlaceQuantity {quantity: Quantity, 
            #[arbitrary(with = gen_ident)]
            token: String,
            from_loc: TokenLocExpr, to_loc: TokenLocExpr},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum TokenLocExpr {
        Groupable{ groupable: Groupable},
        GroupablePlayers { groupable: Groupable, players: Players},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum ScoreRule {
        Score {int: IntExpr, players: Players},
        ScoreMemory {int: IntExpr, 
            #[arbitrary(with = gen_ident)]
            memory: String,
            players: Players},
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum WinnerType {
        Score,
        Memory{ 
            #[arbitrary(with = gen_ident)]
            memory: String
        },
        Position,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Arbitrary)]
    pub enum WinnerRule {
        Winner {players: Players},
        WinnerWith {extrema: Extrema, winner_type: WinnerType},
    }

    // ===========================================================================
    // ===========================================================================
    // ===========================================================================
}