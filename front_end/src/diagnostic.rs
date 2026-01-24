use proc_macro2::Span;

use crate::spanned_ast::*;

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

pub type SID = Spanned<String>;
pub type SPlayerExpr = Spanned<PlayerExpr>;
pub type SIntExpr = Spanned<IntExpr>;
pub type SOp = Spanned<Op>;
pub type SCollection = Spanned<Collection>;
pub type SIntCollection = Spanned<IntCollection>;
pub type SStringCollection = Spanned<StringCollection>;
pub type SLocationCollection = Spanned<LocationCollection>;
pub type SPlayerCollection = Spanned<PlayerCollection>;
pub type STeamCollection = Spanned<TeamCollection>;
pub type SStringExpr = Spanned<StringExpr>;
pub type SCardPosition = Spanned<CardPosition>;
pub type SBoolExpr = Spanned<BoolExpr>;
pub type SIntCmpOp = Spanned<IntCmpOp>;
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
pub type SRule = Spanned<Rule>;
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

