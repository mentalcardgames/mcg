use crate::spanned_ast::*;
use crate::spans::*;


pub trait AstPass {
    fn enter_node<T: Walker>(&mut self, node: &T)
    where
        Self: Sized;
    fn exit_node<T: Walker>(&mut self, node: &T)
    where
        Self: Sized;
}

pub trait Walker {
    fn walk<V: AstPass>(&self, visitor: &mut V);
    fn kind(&self) -> NodeKind<'_>;
}

impl<T> Walker for Vec<T> 
where 
    T: Walker 
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        // We iterate through the vector and call walk on every element.
        // If T is Spanned<something>, it uses your Spanned implementation.
        for item in self {
            item.walk(visitor);
        }
    }

    fn kind(&self) -> NodeKind<'_> {
        // Usually, vectors aren't considered a single "node" in the AST 
        // sense for kind-tracking, so we often return a generic tag.
        NodeKind::None
    }
}

impl<T, S> Walker for (T, S) 
where 
    T: Walker,
    S: Walker,
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        // We iterate through the vector and call walk on every element.
        // If T is Spanned<something>, it uses your Spanned implementation.
        self.0.walk(visitor);
        self.1.walk(visitor);  
    }

    fn kind(&self) -> NodeKind<'_> {
        // Usually, vectors aren't considered a single "node" in the AST 
        // sense for kind-tracking, so we often return a generic tag.
        NodeKind::None
    }
}

impl<T, S, P> Walker for (T, S, P) 
where 
    T: Walker,
    S: Walker,
    P: Walker,
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        // We iterate through the vector and call walk on every element.
        // If T is Spanned<something>, it uses your Spanned implementation.
        self.0.walk(visitor);
        self.1.walk(visitor);  
        self.2.walk(visitor);  
    }

    fn kind(&self) -> NodeKind<'_> {
        // Usually, vectors aren't considered a single "node" in the AST 
        // sense for kind-tracking, so we often return a generic tag.
        NodeKind::None
    }
}


impl Walker for i32 {
    fn walk<V: AstPass>(&self, _: &mut V) {
        
    }

    fn kind(&self) -> NodeKind<'_> {
        NodeKind::None
    }
}


impl<T> Walker for Spanned<T>
where
    T: Walker,
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        self.node.walk(visitor);
    }

    fn kind(&self) -> NodeKind<'_> {
        self.node.kind()
    }
}

impl Walker for String {
    fn walk<V: AstPass>(&self, _: &mut V) {}

    fn kind(&self) -> NodeKind<'_> {
        NodeKind::String
    }
}

pub enum NodeKind<'a> {
    BinCompare(&'a BinCompare),
    LogicBinOp(&'a LogicBinOp),
    IntOp(&'a IntOp),
    IntCompare(&'a IntCompare),
    Extrema(&'a Extrema),
    OutOf(&'a OutOf),
    Groupable(&'a Groupable),
    Owner(&'a Owner),
    Quantity(&'a Quantity),
    IntRange(&'a IntRange),
    Quantifier(&'a Quantifier),
    EndCondition(&'a EndCondition),
    Repititions(&'a Repititions),
    MemoryType(&'a MemoryType),
    Players(&'a Players),
    EndType(&'a EndType),
    DemandType(&'a DemandType),
    Types(&'a Types),
    RuntimePlayer(&'a RuntimePlayer),
    QueryPlayer(&'a QueryPlayer),
    AggregatePlayer(&'a AggregatePlayer),
    PlayerExpr(&'a PlayerExpr),
    QueryInt(&'a QueryInt),
    AggregateInt(&'a AggregateInt),
    RuntimeInt(&'a RuntimeInt),
    IntExpr(&'a IntExpr),
    QueryString(&'a QueryString),
    StringExpr(&'a StringExpr),
    CardSetCompare(&'a CardSetCompare),
    StringCompare(&'a StringCompare),
    PlayerCompare(&'a PlayerCompare),
    TeamCompare(&'a TeamCompare),
    BoolOp(&'a BoolOp),
    UnaryOp(&'a UnaryOp),
    CompareBool(&'a CompareBool),
    AggregateBool(&'a AggregateBool),
    BoolExpr(&'a BoolExpr),
    AggregateTeam(&'a AggregateTeam),
    TeamExpr(&'a TeamExpr),
    QueryCardPosition(&'a QueryCardPosition),
    AggregateCardPosition(&'a AggregateCardPosition),
    CardPosition(&'a CardPosition),
    Status(&'a Status),
    Collection(&'a Collection),
    IntCollection(&'a IntCollection),
    StringCollection(&'a StringCollection),
    LocationCollection(&'a LocationCollection),
    RuntimePlayerCollection(&'a RuntimePlayerCollection),
    AggregatePlayerCollection(&'a AggregatePlayerCollection),
    PlayerCollection(&'a PlayerCollection),
    RuntimeTeamCollection(&'a RuntimeTeamCollection),
    TeamCollection(&'a TeamCollection),
    CardSet(&'a CardSet),
    Group(&'a Group),
    AggregateFilter(&'a AggregateFilter),
    FilterOp(&'a FilterOp),
    FilterExpr(&'a FilterExpr),
    Game(&'a Game),
    FlowComponent(&'a FlowComponent),
    FlowComponents(&'a Vec<FlowComponent>),
    SetUpRule(&'a SetUpRule),
    ActionRule(&'a ActionRule),
    ScoringRule(&'a ScoringRule),
    GameRule(&'a GameRule),
    SeqStage(&'a SeqStage),
    IfRule(&'a IfRule),
    OptionalRule(&'a OptionalRule),
    ChoiceRule(&'a ChoiceRule),
    MoveType(&'a MoveType),
    MoveCardSet(&'a MoveCardSet),
    ClassicMove(&'a ClassicMove),
    DealMove(&'a DealMove),
    ExchangeMove(&'a ExchangeMove),
    TokenMove(&'a TokenMove),
    TokenLocExpr(&'a TokenLocExpr),
    ScoreRule(&'a ScoreRule),
    WinnerType(&'a WinnerType),
    WinnerRule(&'a WinnerRule),
    String,

    // Spanned
    SBinCompare(&'a SBinCompare),
    SLogicBinOp(&'a SLogicBinOp),
    SIntOp(&'a SIntOp),
    SIntCompare(&'a SIntCompare),
    SExtrema(&'a SExtrema),
    SOutOf(&'a SOutOf),
    SGroupable(&'a SGroupable),
    SOwner(&'a SOwner),
    SQuantity(&'a SQuantity),
    SIntRange(&'a SIntRange),
    SQuantifier(&'a SQuantifier),
    SEndCondition(&'a SEndCondition),
    SRepititions(&'a SRepititions),
    SMemoryType(&'a SMemoryType),
    SPlayers(&'a SPlayers),
    SEndType(&'a SEndType),
    SDemandType(&'a SDemandType),
    STypes(&'a STypes),
    SRuntimePlayer(&'a SRuntimePlayer),
    SQueryPlayer(&'a SQueryPlayer),
    SAggregatePlayer(&'a SAggregatePlayer),
    SPlayerExpr(&'a SPlayerExpr),
    SQueryInt(&'a SQueryInt),
    SAggregateInt(&'a SAggregateInt),
    SRuntimeInt(&'a SRuntimeInt),
    SIntExpr(&'a SIntExpr),
    SQueryString(&'a SQueryString),
    SStringExpr(&'a SStringExpr),
    SCardSetCompare(&'a SCardSetCompare),
    SStringCompare(&'a SStringCompare),
    SPlayerCompare(&'a SPlayerCompare),
    STeamCompare(&'a STeamCompare),
    SBoolOp(&'a SBoolOp),
    SUnaryOp(&'a SUnaryOp),
    SCompareBool(&'a SCompareBool),
    SAggregateBool(&'a SAggregateBool),
    SBoolExpr(&'a SBoolExpr),
    SAggregateTeam(&'a SAggregateTeam),
    STeamExpr(&'a STeamExpr),
    SQueryCardPosition(&'a SQueryCardPosition),
    SAggregateCardPosition(&'a SAggregateCardPosition),
    SCardPosition(&'a SCardPosition),
    SStatus(&'a SStatus),
    SCollection(&'a SCollection),
    SIntCollection(&'a SIntCollection),
    SStringCollection(&'a SStringCollection),
    SLocationCollection(&'a SLocationCollection),
    SRuntimePlayerCollection(&'a SRuntimePlayerCollection),
    SAggregatePlayerCollection(&'a SAggregatePlayerCollection),
    SPlayerCollection(&'a SPlayerCollection),
    SRuntimeTeamCollection(&'a STeamCollection),
    STeamCollection(&'a STeamCollection),
    SCardSet(&'a SCardSet),
    SGroup(&'a SGroup),
    SAggregateFilter(&'a SAggregateFilter),
    SFilterOp(&'a SFilterOp),
    SFilterExpr(&'a SFilterExpr),
    SGame(&'a SGame),
    SFlowComponent(&'a SFlowComponent),
    SFlowComponents(&'a Vec<SFlowComponent>),
    SSetUpRule(&'a SSetUpRule),
    SActionRule(&'a SActionRule),
    SScoringRule(&'a SScoringRule),
    SGameRule(&'a SGameRule),
    SSeqStage(&'a SSeqStage),
    SIfRule(&'a SIfRule),
    SOptionalRule(&'a SOptionalRule),
    SChoiceRule(&'a SChoiceRule),
    SMoveType(&'a SMoveType),
    SMoveCardSet(&'a SMoveCardSet),
    SClassicMove(&'a SClassicMove),
    SDealMove(&'a SDealMove),
    SExchangeMove(&'a SExchangeMove),
    STokenMove(&'a STokenMove),
    STokenLocExpr(&'a STokenLocExpr),
    SScoreRule(&'a SScoreRule),
    SWinnerType(&'a SWinnerType),
    SWinnerRule(&'a SWinnerRule),
    SString(&'a String),

    None
}
