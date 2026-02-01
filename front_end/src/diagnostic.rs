use crate::spanned_ast::*;


#[derive(Debug, Clone)]
pub struct OwnedSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl From<pest::Span<'_>> for OwnedSpan {
    fn from(span: pest::Span) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: OwnedSpan,
}

pub type SExtrema = Spanned<Extrema>;
pub type SBinCompare = Spanned<BinCompare>;
pub type SLogicBinOp = Spanned<LogicBinOp>;
pub type SOutOf = Spanned<OutOf>;
pub type SGroupable = Spanned<Groupable>;
pub type SOwner = Spanned<Owner>;
pub type SPlayers = Spanned<Players>;
pub type SEndType = Spanned<EndType>;
pub type SDemandType = Spanned<DemandType>;
pub type SMoveType = Spanned<MoveType>;
pub type SWinnerType = Spanned<WinnerType>;
pub type SMemoryType = Spanned<MemoryType>;
pub type SSetUpRule = Spanned<SetUpRule>;
pub type SScoringRule = Spanned<ScoringRule>;
pub type SActionRule = Spanned<ActionRule>;
pub type SRuntimePlayer = Spanned<RuntimePlayer>;
pub type SAggregatePlayer = Spanned<AggregatePlayer>;
pub type SQueryPlayer = Spanned<QueryPlayer>;
pub type SRuntimeInt = Spanned<RuntimeInt>;
pub type SAggregateInt = Spanned<AggregateInt>;
pub type SQueryInt = Spanned<QueryInt>;
pub type SQueryString = Spanned<QueryString>;
pub type SBoolOp = Spanned<BoolOp>;
pub type SIntCompare = Spanned<IntCompare>;
pub type SStringCompare = Spanned<StringCompare>;
pub type SCardSetCompare = Spanned<CardSetCompare>;
pub type SPlayerCompare = Spanned<PlayerCompare>;
pub type STeamCompare = Spanned<TeamCompare>;
pub type SCompareBool = Spanned<CompareBool>;
pub type SUnaryOp = Spanned<UnaryOp>;
pub type SAggregateBool = Spanned<AggregateBool>;
pub type SAggregateTeam = Spanned<AggregateTeam>;
pub type SQueryCardPosition = Spanned<QueryCardPosition>;
pub type SAggregateCardPosition = Spanned<AggregateCardPosition>;
pub type SAggregatePlayerCollection = Spanned<AggregatePlayerCollection>;
pub type SRuntimePlayerCollection = Spanned<RuntimePlayerCollection>;
pub type SRuntimeTeamCollection = Spanned<RuntimeTeamCollection>;
pub type SAggregateFilter = Spanned<AggregateFilter>;
pub type SFilterOp = Spanned<FilterOp>;
pub type SMoveCardSet = Spanned<MoveCardSet>;

pub type SID = Spanned<String>;
pub type SPlayerExpr = Spanned<PlayerExpr>;
pub type SIntExpr = Spanned<IntExpr>;
pub type SIntOp = Spanned<IntOp>;
pub type SCollection = Spanned<Collection>;
pub type SIntCollection = Spanned<IntCollection>;
pub type SStringCollection = Spanned<StringCollection>;
pub type SLocationCollection = Spanned<LocationCollection>;
pub type SPlayerCollection = Spanned<PlayerCollection>;
pub type STeamCollection = Spanned<TeamCollection>;
pub type SStringExpr = Spanned<StringExpr>;
pub type SCardPosition = Spanned<CardPosition>;
pub type SBoolExpr = Spanned<BoolExpr>;
pub type SStatus = Spanned<Status>; 
pub type STeamExpr = Spanned<TeamExpr>;
pub type SQuantity = Spanned<Quantity>;
pub type SIntRange = Spanned<IntRange>;
pub type SQuantifier = Spanned<Quantifier>;
pub type SCardSet = Spanned<CardSet>;
pub type SGroup = Spanned<Group>;
pub type SFilterExpr = Spanned<FilterExpr>;
pub type SGame = Spanned<Game>;
pub type SFlowComponent = Spanned<FlowComponent>;
pub type SEndCondition = Spanned<EndCondition>;
pub type SRepititions = Spanned<Repititions>;
pub type SGameRule = Spanned<GameRule>;
pub type STypes = Spanned<Types>;
pub type SSeqStage = Spanned<SeqStage>;
pub type SIfRule = Spanned<IfRule>;
pub type SOptionalRule = Spanned<OptionalRule>;
pub type SChoiceRule = Spanned<ChoiceRule>;
pub type SClassicMove = Spanned<ClassicMove>;
pub type SDealMove = Spanned<DealMove>;
pub type SExchangeMove = Spanned<ExchangeMove>;
pub type STokenMove = Spanned<TokenMove>;
pub type STokenLocExpr = Spanned<TokenLocExpr>;
pub type SScoreRule = Spanned<ScoreRule>;
pub type SWinnerRule = Spanned<WinnerRule>;

