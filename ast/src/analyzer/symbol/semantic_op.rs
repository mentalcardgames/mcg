use crate::{ast::*, dsl_types::DSLType};

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticOp {
  Initialize {
    id: String,
    ty: DSLType,
  },
  InitializeVec {
    vec_id: Vec<String>,
    ty: DSLType,
  },
  Use {
    id: String,
    ty: DSLType,
  },
  UseVec {
    vec_id: Vec<String>,
    ty: DSLType,
  },
  UseCollection {
    collection: Collection,
  },
  InitializeKeyValue {
    key: String,
    value: String, 
  },
  UseFilter {
    filter: FilterExpr,
  },
  UsePlayer {
    player: PlayerExpr,
  },
  UseTeam {
    team: TeamExpr,
  },
  UseCardPosition {
    card_position: CardPosition
  },
  UseCardSet {
    card_set: CardSet, 
  },
  UseGroup {
    group: Group, 
  },
  UseInt {
    int_expr: IntExpr, 
  },
  UseString {
    string_expr: StringExpr, 
  },
  UseTokenLoc {
    token_loc_expr: TokenLocExpr,
  },
  UseClassicMove {
    classic_move: ClassicMove,
  },
  UseDealMove {
    deal_move: DealMove,
  },
  UseExchangeMove {
    exchange_move: ExchangeMove,
  },
  UseTokenMove {
    token_move: TokenMove,
  },
  UseScoreRule {
    score_rule: ScoreRule,
  },
  UseWinnerRule {
    winner_rule: WinnerRule,
  },
  UseEndCondition {
    end_condition: EndCondition,
  },
  UseBoolExpr {
    bool_expr: BoolExpr,
  },
}
