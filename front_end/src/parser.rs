/*
    This is the Parsing logic. We use a "direct to AST"-parsing method.
    The library used are pest and especially pest_consume.

    The Grammar is in some parts ambiguous so a Symbol-Table is added (is doable with pest_consume)
    to resolve the ambiguous Rules (e.g. Collection: "(" ident ("," ident)* ")" is matched by multiple
    collections: LocationCollection, PlayerCollection, TeamCollection)

    It might make sense to use (or return to) a Sigil-style type naming in the future to get rid of ambiguity:
    - Players start with "P"
    - Teams start with "T"
    - Combos start with "C"
    - ...
    - Memory-Type:
    > Player-Memory starts with "P:"
    > ...

    This would make the parsing very dumb and easy to extend.

    However if no more ambiguity comes forth that we can keep this approach.
*/


use std::cell::RefCell;
use std::collections::HashMap;

use pest_consume::{Parser, match_nodes};

use crate::{spans::*};
use crate::ast::ast_spanned::*;


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CGDSLParser;

use pest_consume::Error;
pub type Result<T> = std::result::Result<T, Error<Rule>>;
pub type Node<'i> = pest_consume::Node<'i, Rule, RefCell<SymbolTable>>;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum MemType {
    Int,
    String,
    PlayerCollection,
    StringCollection,
    IntCollection,
    TeamCollection,
    LocationCollection,
    CardSet,    
}

use crate::symbols::GameType;

#[derive(Clone, Debug)]
pub struct SymbolTable {
    pub symbols: HashMap<String, GameType>,
    pub memories: HashMap<String, MemType>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        SymbolTable { symbols: HashMap::new(), memories: HashMap::new() }
    }
}


#[pest_consume::parser]
impl CGDSLParser {
    pub(crate) fn ident(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub fn file(input: Node) -> Result<SGame> {
        Ok(
            match_nodes!(input.into_children();
                [game_flow(g), _] => g,
            )
        )
    }

    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn game_flow(input: Node) -> Result<SGame> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
                [flow_component(flow_components)..] => flow_components.collect(),
        );
        
        Ok(
            SGame {
                node: Game { flows: node },
                span
            }
        )
    }

    pub(crate) fn flow_component(input: Node) -> Result<SFlowComponent> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
                [seq_stage(n)] => FlowComponent::SeqStage { stage: n },
                [sim_stage(n)] => FlowComponent::SimStage { stage: n },
                [if_rule(s)] => FlowComponent::IfRule { if_rule: s },
                [choice_rule(t)] => FlowComponent::ChoiceRule { choice_rule: t },
                [optional_rule(l)] => FlowComponent::OptionalRule { optional_rule: l },
                [trigger_rule(l)] => FlowComponent::TriggerRule { trigger_rule: l },
                [game_rule(k)] => FlowComponent::Rule { game_rule: k },
                [cond_rule(k)] => FlowComponent::Conditional { conditional: k },
        );

        Ok(
            SFlowComponent {
                node: node,
                span
            }
        )
    }

    pub(crate) fn seq_stage(input: Node) -> Result<SSeqStage> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_stage(_), stage(s), kw_for(_), player_expr(p), end_condition(e), flow_component(f)..] => SeqStage {
                stage: s,
                player: p,
                end_condition: e,
                flows: f.collect(), // Collects the trailing flow_components into a Vec
            },
        );

        Ok(
            SSeqStage {
                node: node,
                span
            }
        )
    }

    pub(crate) fn sim_stage(input: Node) -> Result<SSimStage> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_stage(_), stage(s), kw_for(_), player_collection(p), end_condition(e), flow_component(f)..] => SimStage {
                stage: s,
                players: p,
                end_condition: e,
                flows: f.collect(), // Collects the trailing flow_components into a Vec
            },
        );

        Ok(
            SSimStage {
                node: node,
                span
            }
        )
    }


    pub(crate) fn kw_case(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_conditional(input: Node) -> Result<()> {
        Ok(())
    }
    
    pub(crate) fn case(input: Node) -> Result<SCase> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_case(_), bool_expr(n), flow_component(f)..] => Case::Bool { bool_expr: n, flows: f.collect() },
            [kw_case(_), flow_component(f)..] => Case::NoBool { flows: f.collect() },
        );

        Ok(
            SCase {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_else(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn else_case(input: Node) -> Result<SCase> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_case(_), kw_else(_), flow_component(f)..] => Case::NoBool { flows: f.collect() },
        );

        Ok(
            SCase {
                node: node,
                span
            }
        )
    }

    pub(crate) fn cond_rule(input: Node) -> Result<SConditional> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_conditional(_), case(c).., else_case(e)] => {
                let mut result: Vec<SCase> = c.collect();
                result.push(e);
                Conditional { cases: result }
            },
            [kw_conditional(_), case(c)..] => Conditional { cases: c.collect() },
        );

        Ok(
            SConditional {
                node: node,
                span
            }
        )
    }
    
    pub(crate) fn if_rule(input: Node) -> Result<SIfRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_if(_), bool_expr(n), flow_component(f)..] => IfRule {
                condition: n,
                flows: f.collect(), // Collects the trailing flow_components into a Vec
            }
        );

        Ok(
            SIfRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn choice_rule(input: Node) -> Result<SChoiceRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_choose(_), flow_component(f)..] => ChoiceRule {
                options: f.collect(), // Collects the trailing flow_components into a Vec
            }
        );

        Ok(
            SChoiceRule {
                node: node,
                span
            }
        )

    }

    pub(crate) fn kw_trigger(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn trigger_rule(input: Node) -> Result<STriggerRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_trigger(_), flow_component(f)..] => TriggerRule {
                flows: f.collect(), // Collects the trailing flow_components into a Vec
            }
        );

        Ok(
            STriggerRule {
                node: node,
                span
            }
        )

    }

    pub(crate) fn optional_rule(input: Node) -> Result<SOptionalRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_optional(_), flow_component(f)..] => OptionalRule {
                flows: f.collect(), // Collects the trailing flow_components into a Vec
            }
        );

        Ok(
            SOptionalRule {
                node: node,
                span
            }
        )

    }

    pub(crate) fn kw_choose(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_optional(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_if(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_times(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn repetitions(input: Node) -> Result<SRepititions> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [int_expr(n), kw_times(_)] => Repititions { times: n},
        );

        Ok(
            SRepititions {
                node: node,
                span
            }
        )
    }

    pub(crate) fn logical_compare(input: Node) -> Result<SLogicBinOp> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_and(_)] => LogicBinOp::And,
            [kw_or(_)] => LogicBinOp::Or,
        );

        Ok(
            SLogicBinOp {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_until(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_end(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn until_bool(input: Node) -> Result<SEndCondition> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_until(_), bool_expr(b)] => EndCondition::UntilBool { bool_expr: b },
        );

        Ok(
            SEndCondition {
                node: node,
                span
            }
        )
    }

    pub(crate) fn until_bool_repetitions(input: Node) -> Result<SEndCondition> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_until(_), bool_expr(b), logical_compare(l), repetitions(r)] => EndCondition::UntilBoolRep { bool_expr: b, logic: l, reps: r},
        );

        Ok(
            SEndCondition {
                node: node,
                span
            }
        )
    }

    pub(crate) fn until_repetitions(input: Node) -> Result<SEndCondition> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [repetitions(r)] => EndCondition::UntilRep { reps: r},
        );

        Ok(
            SEndCondition {
                node: node,
                span
            }
        )
    }

    pub(crate) fn until_end(input: Node) -> Result<SEndCondition> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_until(_), kw_end(_)] => EndCondition::UntilEnd,
        );

        Ok(
            SEndCondition {
                node: node,
                span
            }
        )
    }

    pub(crate) fn end_condition(input: Node) -> Result<SEndCondition> {
        Ok(
            match_nodes!(input.into_children();
                [until_end(e)] => e,
                [until_bool_repetitions(br)] => br,
                [until_bool(b)] => b,
                [until_repetitions(r)] => r,
            )
        )
    }

    pub(crate) fn stage(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }
    
    pub(crate) fn key(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
       )
    }

    pub(crate) fn combo(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn playername(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn teamname(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn precedence(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn pointmap(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn token(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn memory(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn value(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn kw_current(input: Node) -> Result<RuntimePlayer> {
        Ok(RuntimePlayer::Current)
    }

    pub(crate) fn kw_next(input: Node) -> Result<RuntimePlayer> {
        Ok(RuntimePlayer::Next)
    }

    pub(crate) fn kw_previous(input: Node) -> Result<RuntimePlayer> {
        Ok(RuntimePlayer::Previous)
    }

    pub(crate) fn kw_competitor(input: Node) -> Result<RuntimePlayer> {
        Ok(RuntimePlayer::Competitor)
    }

    pub(crate) fn location(input: Node) -> Result<SID> {
        Ok(
            SID {
                node: input.as_str().to_string(),
                span: OwnedSpan::from(input.as_span())
            }
        )
    }

    pub(crate) fn kw_top(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_bottom(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn location_top(input: Node) -> Result<SCardPosition> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [kw_top(_), location(n)] => squery_card_position(QueryCardPosition::Top { location: n }, span), 
            )
        )
    }

    pub(crate) fn location_bottom(input: Node) -> Result<SCardPosition> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [kw_bottom(_), location(n)] => squery_card_position(QueryCardPosition::Bottom { location: n }, span), 
            )
        )
    }

    pub(crate) fn kw_min(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_max(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_lowest(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_highest(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn extrema(input: Node) -> Result<SExtrema> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
                [kw_min(_)] => Extrema::Min,
                [kw_max(_)] => Extrema::Max,
                [kw_lowest(_)] => Extrema::Min,
                [kw_highest(_)] => Extrema::Max,
        );

        Ok(
            SExtrema {
                node: node,
                span
            }
        )

    }

    pub(crate) fn kw_for(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_using(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn extrema_of_card_set(input: Node) -> Result<SCardPosition> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [extrema(n), kw_of(_), card_set(s), kw_using(_), kw_points(_), pointmap(t)] => saggregate_card_position(AggregateCardPosition::ExtremaPointMap { extrema: n, card_set: Box::new(s), pointmap: t }, span), 
                [extrema(n), kw_of(_), card_set(s), kw_using(_), precedence(t)] => saggregate_card_position(AggregateCardPosition::ExtremaPrecedence { extrema: n, card_set: Box::new(s), precedence: t }, span), 
            )
        )
    }

    pub(crate) fn location_int_expr(input: Node) -> Result<SCardPosition> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [location(n), int_expr(s)] => squery_card_position(QueryCardPosition::At {location: n, int_expr: s }, span), 
            )
        )
    }

    pub(crate) fn card_position(input: Node) -> Result<SCardPosition> {
        Ok(
            match_nodes!(input.into_children();
                [location_top(n)] => n,
                [location_bottom(n)] => n,
                [location_int_expr(n)] => n,
                [extrema_of_card_set(n)] => n,
            )
        )
    }

    pub(crate) fn kw_owner(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_of(input: Node) -> Result<()> {
        Ok(())
    }
    
    pub(crate) fn owner_of_player(input: Node) -> Result<AggregatePlayer> {
        Ok(
            match_nodes!(input.into_children();
                [kw_owner(_), kw_of(_), extrema(n), memory(s)] => AggregatePlayer::OwnerOfMemory { extrema: n, memory: s },
                [kw_owner(_), kw_of(_), card_position(n)] => AggregatePlayer::OwnerOfCardPostion { card_position: Box::new(n) },
            )
        )
    }

    pub(crate) fn turnorder_at(input: Node) -> Result<QueryPlayer> {
        Ok(
            match_nodes!(input.into_children();
                [kw_turnorder(_), int_expr(i)] => QueryPlayer::Turnorder { int: i },
            )
        )
    }

    pub(crate) fn players_at(input: Node) -> Result<QueryPlayer> {
        Ok(
            match_nodes!(input.into_children();
                [player_collection(pc), int_expr(i)] => QueryPlayer::CollectionAt { players: pc, int: i },
            )
        )
    }

    pub(crate) fn player_expr(input: Node) -> Result<SPlayerExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [players_at(p)] => SPlayerExpr { node: PlayerExpr::Query { query: SQueryPlayer { node: p, span: span.clone() } }, span },
                [kw_current(n)] => sruntime_player(n, span),
                [kw_previous(s)] => sruntime_player(s, span),
                [kw_competitor(t)] => sruntime_player(t, span),
                [kw_next(r)] => sruntime_player(r, span),
                [owner_of_player(q)] => saggregate_player(q, span),
                [turnorder_at(q)] => SPlayerExpr { node: PlayerExpr::Query { query: SQueryPlayer { node: q, span: span.clone() } }, span },
                [playername(p)] => SPlayerExpr { node: PlayerExpr::Literal { name: p }, span }
            )
        )
    }

    pub(crate) fn plus(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn minus(input: Node) -> Result<()> {
        Ok(())
    }
    
    pub(crate) fn div(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn mul(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn modulo(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn int_op(input: Node) -> Result<SIntOp> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [plus(_)] => IntOp::Plus,
            [minus(_)] => IntOp::Minus,
            [mul(_)] => IntOp::Mul,
            [div(_)] => IntOp::Div,
            [modulo(_)] => IntOp::Mod,
        );

        Ok(
            SIntOp {
                node: node,
                span
            }
        )
    }

    pub(crate) fn gt(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn lt(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn ge(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn le(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn int_compare(input: Node) -> Result<SIntCompare> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [eq(_)] =>  IntCompare::Eq,
            [neq(_)] => IntCompare::Neq,
            [gt(_)] =>  IntCompare::Gt,
            [lt(_)] =>  IntCompare::Lt,
            [ge(_)] =>  IntCompare::Ge,
            [le(_)] =>  IntCompare::Le,
        );

        Ok(
            SIntCompare {
                node: node,
                span
            }
        )
    }

    pub(crate) fn bin_int_op(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [int_expr(n), int_op(s), int_expr(t)] => IntExpr::Binary { int: Box::new(n), op: s, int1: Box::new(t) }
        );

        Ok(
            SIntExpr {
                node: node,
                span
            }
        )   
    }

    pub(crate) fn int_collection_at(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [int_collection(n), int_expr(s)] => squery_int(QueryInt::IntCollectionAt { int_collection: Box::new(n), int_expr: Box::new(s)}, span),
            )
        )
    }

    pub(crate) fn kw_size(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn size_of_collection(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [kw_size(_), collection(n)] => saggregate_int(AggregateInt::SizeOf { collection: n }, span),
            )
        )
    }

    pub(crate) fn kw_sum(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn sum_of_int_collection(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [kw_sum(_), int_collection(n)] => saggregate_int(AggregateInt::SumOfIntCollection { int_collection: n }, span),
            )
        )
    }

    pub(crate) fn sum_of_card_set(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [kw_sum(_), kw_of(_), card_set(n), kw_using(_), pointmap(s)] => saggregate_int(AggregateInt::SumOfCardSet { card_set: Box::new(n), pointmap: s }, span),
            )
        )
    }

    pub(crate) fn int_extrema_of_card_set(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [extrema(n), kw_of(_), card_set(s), kw_using(_), pointmap(p)] => saggregate_int(AggregateInt::ExtremaCardset { extrema: n, card_set: Box::new(s), pointmap: p }, span),
            )
        )
    }


    pub(crate) fn extrema_of_int_collection(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [extrema(n), int_collection(s)] => saggregate_int(AggregateInt::ExtremaIntCollection { extrema: n, int_collection: s}, span),
            )
        )
    }

    pub(crate) fn int(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        let s = input.as_str();
        let int = s.parse::<i32>().map_err(|e| {
            // This attaches the error to the specific span in the source code
            input.error(format!("'{}' is not a valid i32: {}", s, e))
        });

        Ok(
            SIntExpr {
                node: IntExpr::Literal { int:
                    SInt {
                        node: int?,
                        span: span.clone()
                    }
                },
                span
            }
        )
    }

    pub(crate) fn runtime_int(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_stageroundcounter(_)] => {
                sruntime_int(RuntimeInt::StageRoundCounter, span.clone())
            },
            [kw_playroundcounter(_)] => {
                sruntime_int(RuntimeInt::PlayRoundCounter, span.clone())
            }
        );

        Ok(
            node
        )
    }

    pub(crate) fn int_expr(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [bin_int_op(n)] => n,
                [int_collection_at(n)] => n,
                [size_of_collection(n)] => n,
                [sum_of_card_set(n)] => n,
                [sum_of_int_collection(n)] => n,
                [int_extrema_of_card_set(n)] => n,
                [extrema_of_int_collection(n)] => n,
                [runtime_int(n)] => n,
                [int(n)] => n,
                [use_memory(m)] => SIntExpr { node: IntExpr::Memory { memory: m }, span },
            )
        )
    }

    pub(crate) fn key_of_card_position(input: Node) -> Result<SStringExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [key(n), kw_of(_), card_position(s)] => squery_string(QueryString::KeyOf { key: n, card_position: s}, span),
            )
        )
    }

    pub(crate) fn string_collection_at(input: Node) -> Result<SStringExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [string_collection(n), int_expr(s)] => squery_string(QueryString::StringCollectionAt { string_collection: n, int_expr: s }, span),
            )
        )
    }

    pub(crate) fn string_expr(input: Node) -> Result<SStringExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [key_of_card_position(n)] => n,
                [string_collection_at(n)] => n,
                [value(v)] => SStringExpr { node: StringExpr::Literal { value: v }, span: span },
                [use_memory(m)] => SStringExpr { node: StringExpr::Memory { memory: m }, span },
            )
        )
    }

    pub(crate) fn kw_team(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn team_of_player(input: Node) -> Result<STeamExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [kw_team(_), kw_of(_), player_expr(n)] => saggregate_team(AggregateTeam::TeamOf { player: n }, span),
            )
        )
    }

    pub(crate) fn team_expr(input: Node) -> Result<STeamExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [team_of_player(n)] => n,
                [teamname(n)] => STeamExpr { node: TeamExpr::Literal { name: n }, span},
            )
        )
    }

    pub(crate) fn int_collection(input: Node) -> Result<SIntCollection> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [int_expr(int_exprs)..] => IntCollection::Literal { ints: int_exprs.collect() },
            [use_memory(m)] => IntCollection::Memory { memory: m },
        );

        Ok(
            SIntCollection {
                node: node,
                span
            }
        )
        
    }

    pub(crate) fn string_collection(input: Node) -> Result<SStringCollection> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [string_expr(string_exprs)..] => StringCollection::Literal { strings: string_exprs.collect() },
            [use_memory(m)] => StringCollection::Memory { memory: m },
        );

        Ok(
            SStringCollection {
                node: node,
                span
            }
        )
    }

    pub(crate) fn eq(input: Node) -> Result<OwnedSpan> {
        Ok(OwnedSpan::from(input.as_span()))
    }

    pub(crate) fn neq(input: Node) -> Result<OwnedSpan> {
        Ok(OwnedSpan::from(input.as_span()))
    }

    pub(crate) fn kw_and(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_or(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_not(input: Node) -> Result<bool> {
        Ok(true)
    }

    pub(crate) fn card_set_compare(input: Node) -> Result<SCardSetCompare> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [eq(_)] => CardSetCompare::Eq,
            [neq(_)] => CardSetCompare::Neq,
        );

        Ok(
            SCardSetCompare {
                node: node,
                span
            }
        )
    }

    pub(crate) fn player_expr_compare(input: Node) -> Result<SPlayerCompare> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [eq(_)] => PlayerCompare::Eq,
            [neq(_)] => PlayerCompare::Neq,
        );

        Ok(
            SPlayerCompare {
                node: node,
                span
            }
        )
    }

    pub(crate) fn team_expr_compare(input: Node) -> Result<STeamCompare> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [eq(_)] => TeamCompare::Eq,
            [neq(_)] => TeamCompare::Neq,
        );

        Ok(
            STeamCompare {
                node: node,
                span
            }
        )
    }

    pub(crate) fn string_expr_compare(input: Node) -> Result<SStringCompare> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [eq(_)] => StringCompare::Eq,
            [neq(_)] => StringCompare::Neq,
        );

        Ok(
            SStringCompare {
                node: node,
                span
            }
        )
    }

    pub(crate) fn bool_op(input: Node) -> Result<SBoolOp> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_and(_)] => BoolOp::And,
            [kw_or(_)] => BoolOp::Or,
        );

        Ok(
            SBoolOp {
                node: node,
                span
            }
        )
    }

    pub(crate) fn unary_op(input: Node) -> Result<SUnaryOp> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_not(_)] => UnaryOp::Not,
        );

        Ok(
            SUnaryOp {
                node: node,
                span
            }
        )
    }

    pub(crate) fn card_set_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [card_set(n), card_set_compare(s), card_set(t)] => saggregate_compare_bool(CompareBool::CardSet { card_set: n, cmp: s, card_set1: t }, span),
            )
        )
    }

    pub(crate) fn string_expr_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [string_expr(n), string_expr_compare(s), string_expr(t)] => saggregate_compare_bool(CompareBool::String { string: n, cmp: s, string1: t }, span),
            )
        )
    }

    pub(crate) fn player_expr_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [player_expr(n), player_expr_compare(s), player_expr(t)] => saggregate_compare_bool(CompareBool::Player{ player: n, cmp: s, player1: t}, span),
            )
        )
    }

    pub(crate) fn team_expr_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [team_expr(n), team_expr_compare(s), team_expr(t)] => saggregate_compare_bool(CompareBool::Team { team: n, cmp: s, team1: t }, span),
            )
        )
    }

    pub(crate) fn int_expr_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [int_expr(n), int_compare(s), int_expr(t)] => saggregate_compare_bool(CompareBool::Int { int: n, cmp: s, int1: t }, span),
            )
        )
    }

    pub(crate) fn kw_is(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_empty(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn card_set_not_empty(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [card_set(n), kw_not(_), kw_empty(_)] => saggregate_bool(AggregateBool::CardSetNotEmpty { card_set: n}, span),
            )
        )
    }

    pub(crate) fn card_set_empty(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [card_set(n), kw_empty(_)] => saggregate_bool(AggregateBool::CardSetEmpty { card_set: n }, span),
            )
        )
    }

    pub(crate) fn bool_expr_binary(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [bool_expr(n), bool_op(s), bool_expr(t)] => SBoolExpr { node: BoolExpr::Binary { bool_expr: Box::new(n), op: s, bool_expr1: Box::new(t) }, span },
            )
        )
    }

    pub(crate) fn bool_expr_unary(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [unary_op(n), bool_expr(s)] => SBoolExpr { node: BoolExpr::Unary { op: n, bool_expr: Box::new(s) }, span },
            )
        )
    }

    pub(crate) fn players(input: Node) -> Result<SPlayers> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [player_expr(n)] => SPlayers { node: Players::Player { player: n } , span },
                [player_collection(n)] => SPlayers { node: Players::PlayerCollection { player_collection: n }, span },
            )
        )
    }

    pub(crate) fn kw_all(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_any(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn quantifier(input: Node) -> Result<SQuantifier> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_all(_)] => Quantifier::All,
            [kw_any(_)] => Quantifier::Any,
        );
        
        Ok(
            SQuantifier {
                node: node,
                span
            }
        )
    }

    pub(crate) fn player_expr_collection(input: Node) -> Result<SPlayerCollection> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [player_expr(players)..] => SPlayerCollection { node: PlayerCollection::Literal { players: players.collect() }, span },
            )
        )
    }

    pub(crate) fn kw_playroundcounter(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_others(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_other(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_playersin(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_playersout(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn player_collection(input: Node) -> Result<SPlayerCollection> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [player_expr_collection(n)] => n,
                [quantifier(n)] => saggregate_player_collection(AggregatePlayerCollection::Quantifier { quantifier: n }, span),
                [kw_others(_)] => sruntime_player_collection(RuntimePlayerCollection::Others, span),
                [kw_playersin(_)] => sruntime_player_collection(RuntimePlayerCollection::PlayersIn, span),
                [kw_playersout(_)] => sruntime_player_collection(RuntimePlayerCollection::PlayersOut, span),
                [use_memory(m)] => SPlayerCollection { node: PlayerCollection::Memory { memory: m }, span },
            )
        )
    }

    pub(crate) fn kw_out(input: Node) ->  Result<()> {
        Ok(())
    }

    pub(crate) fn kw_stage(input: Node) ->  Result<()> {
        Ok(())
    }
    
    pub(crate) fn kw_stageroundcounter(input: Node) ->  Result<()> {
        Ok(())
    }
    
    pub(crate) fn kw_game(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn out_of(input: Node) -> Result<SOutOf> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_stage(_)] => OutOf::CurrentStage,
            [stage(n)] => OutOf::Stage { name: n },
            [kw_game(_)] => OutOf::Game,
        );
        
        Ok(
            SOutOf {
                node: node,
                span
            }
        )
    }

    pub(crate) fn players_out_of(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [players(n), kw_out(_), kw_of(_), out_of(s)] => saggregate_bool(AggregateBool::OutOfPlayer { players: n, out_of: s }, span),
            )
        )
    }

    pub(crate) fn cmp_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.clone().into_children();
            [players(n), kw_out(_), kw_of(_), out_of(s)] => saggregate_bool(AggregateBool::OutOfPlayer { players: n, out_of: s }, span),
            [use_memory(m1), eq(e), use_memory(m2)] =>
                resolve_use_memory_eq_use_memory(m1, e, m2, span, input),
            [use_memory(m1), neq(ne), use_memory(m2)] =>
                resolve_use_memory_neq_use_memory(m1, ne, m2, span, input),
            [card_set_bool(n)] => n,
            [string_expr_bool(n)] => n,
            [player_expr_bool(n)] => n,
            [team_expr_bool(n)] => n,
            [int_expr_bool(n)] => n,
        );

        Ok(
            node
        )
    }

    pub(crate) fn string_card_set(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [string_expr(s), kw_in(_), card_set(c)] => 
                    saggregate_bool(AggregateBool::StringInCardSet { string: s, card_set: c }, span),
                [string_expr(s), kw_not(_), kw_in(_), card_set(c)] => 
                    saggregate_bool(AggregateBool::StringNotInCardSet { string: s, card_set: c }, span),
            )
        )
    }

    pub(crate) fn bool_expr(input: Node) -> Result<SBoolExpr> {
        Ok(
            match_nodes!(input.into_children();
                [string_card_set(n)] => n,
                [cmp_bool(n)] => n,
                [card_set_not_empty(n)] => n,
                [card_set_empty(n)] => n,
                [bool_expr_binary(n)] => n,
                [bool_expr_unary(n)] => n,
                [players_out_of(n)] => n,
            )
        )
    }

    pub(crate) fn kw_same(input: Node) ->  Result<()> {
        Ok(())
    }

    pub(crate) fn key_distinct(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [kw_distinct(_), key(key)] => saggregate_filter(AggregateFilter::Same { key: key }, span),
            )
        )
    }

    pub(crate) fn kw_distinct(input: Node) ->  Result<()> {
        Ok(())
    }

    pub(crate) fn kw_adjacent(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn key_adjacent(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [kw_adjacent(_), key(key), kw_using(_), precedence(prec)] => saggregate_filter(AggregateFilter::Adjacent { key: key, precedence: prec } , span),
            )
        )
    }

    pub(crate) fn kw_higher(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn key_higher(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [key(key), kw_higher(_), kw_than(_), string_expr(s), kw_using(_), precedence(prec)] => saggregate_filter(AggregateFilter::Higher{ key: key, value: s, precedence: prec }, span),
            )
        )
    }

    pub(crate) fn use_memory(input: Node) -> Result<SUseMemory> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.children();
            [memory(m)] => UseMemory::Memory { memory: m },
            [memory(m), kw_of(_), owner(o)] => UseMemory::WithOwner { memory: m, owner: Box::new(o) },
        );

        Ok(
            SUseMemory {
                node: node,
                span
            }
        )
        
    }

    pub(crate) fn kw_lower(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_than(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn key_lower(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [key(key), kw_lower(_), kw_than(_), string_expr(s), kw_using(_), precedence(prec)] => saggregate_filter(AggregateFilter::Lower{ key: key, value: s, precedence: prec }, span),
            )
        )
    }

    pub(crate) fn key_same(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [kw_same(_), key(key)] => saggregate_filter(AggregateFilter::Same { key: key }, span),
            )
        )
    }

    pub(crate) fn size_int(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [kw_size(_), int_compare(n), int_expr(s)] => saggregate_filter(AggregateFilter::Size { cmp: n, int_expr: Box::new(s) }, span),
            )
        )
    }

    pub(crate) fn key_string(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [key(key), kw_is(_), string_expr(s)] => saggregate_filter(AggregateFilter::KeyIsString { key: key, string: Box::new(s) }, span),
                [key(key), kw_is(_), kw_not(_), string_expr(s)] => saggregate_filter(AggregateFilter::KeyIsNotString { key: key, string: Box::new(s) }, span),
            )
        )
    }

    pub(crate) fn filter_combo(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [kw_not(_), combo(combo)] => saggregate_filter(AggregateFilter::NotCombo { combo: combo } , span),
                [combo(combo)] => saggregate_filter(AggregateFilter::Combo { combo: combo }, span),
            )
        )
    }

    pub(crate) fn filter_op(input: Node) -> Result< SFilterOp> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.children();
            [kw_and(_)] => FilterOp::And,
            [kw_or(_)] => FilterOp::Or,
        );

        Ok(
            SFilterOp {
                node: node,
                span
            }
        )
    }

    pub(crate) fn filter_bin_op(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.children();
            [filter_expr(n), filter_op(s), filter_expr(t)] => FilterExpr::Binary { filter: Box::new(n), op: s, filter1: Box::new(t) },
        );
        
        Ok(
            SFilterExpr {
                node: node,
                span
            }
        )
    }

    pub(crate) fn filter_expr(input: Node) -> Result<SFilterExpr> {
        Ok(
            match_nodes!(input.into_children();
                [key_same(s)] => s,
                [key_distinct(d)] => d,
                [key_adjacent(a)] => a,
                [key_higher(h)] => h,
                [key_lower(l)] => l,
                [size_int(si)] => si,
                [key_string(st)] => st,
                [filter_combo(fc)] => fc,
                [filter_bin_op(fb)] => fb,
            )
        )
    }

    pub(crate) fn face_up(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn face_down(input: Node) -> Result<()> {
        Ok(())
    }
    
    pub(crate) fn private(input: Node) -> Result<()> {
        Ok(())
    }
    
    pub(crate) fn status(input: Node) -> Result<SStatus> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [face_down(_)] => Status::FaceDown,
            [face_up(_)] => Status::FaceUp,
            [private(_)] => Status::Private,
        );
        
        Ok(
            SStatus {
                node: node,
                span
            }
        )
    }

    pub(crate) fn int_range_logic(input: Node) -> Result<SIntRangeOperator> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_and(_)] => IntRangeOperator::And,
            [kw_or(_)] => IntRangeOperator::Or,
        );

        Ok(
            SIntRangeOperator {
                node: node,
                span
            }
        )
    }

    pub(crate) fn int_range_helper(input: Node) -> Result<(SIntRangeOperator, SIntCompare, SIntExpr)> {
        Ok(
            match_nodes!(input.into_children();
                [int_range_logic(c), int_compare(n), int_expr(s)] => (c, n, s),
            )
        )
    }

    pub(crate) fn int_range(input: Node) -> Result<SIntRange> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [int_compare(n), int_expr(s)] => IntRange { start: (n, s), op_int: vec![] },
            [int_compare(n), int_expr(s), int_range_helper(irhs)..] => 
                IntRange { start: (n, s), op_int: irhs.collect() },
        );

        Ok(
            SIntRange {
                node: node,
                span
            }
        )
    }

    pub(crate) fn quantity(input: Node) -> Result<SQuantity> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [int_expr(n)] => Quantity::Int { int: n },
            [quantifier(n)] => Quantity::Quantifier { qunatifier: n },
            [int_range(n)] => Quantity::IntRange {int_range: n },
        );

        Ok(
            SQuantity {
                node: node,
                span
            }
        )

    }

    pub(crate) fn kw_where(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn groupable(input: Node) -> Result<SGroupable> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [location_collection(n)] => Groupable::LocationCollection { location_collection: n },
            [location(n)] => Groupable::Location { name: n },
        );

        Ok(
            SGroupable {
                node: node,
                span
            }
        )
    }

    pub(crate) fn groupable_where_filter(input: Node) -> Result<SGroup> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [groupable(n), kw_where(_), filter_expr(s)] => Group::Where { groupable: n, filter: s },
        );

        Ok(
            SGroup {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_in(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn combo_in_groupable(input: Node) -> Result<SGroup> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [combo(combo), kw_in(_), groupable(s)] => Group::Combo { combo: combo, groupable: s },
            [kw_not(_), combo(combo), kw_in(_), groupable(s)] => Group::NotCombo { combo: combo, groupable: s },
        );

        Ok(
            SGroup {
                node: node,
                span
            }
        )
    }

    pub(crate) fn group(input: Node) -> Result<SGroup> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [groupable_where_filter(n)] => n,
                [groupable(n)] => SGroup { node: Group::Groupable { groupable: n }, span },
                [combo_in_groupable(n)] => n,
                [card_position(n)] => SGroup { node: Group::CardPosition { card_position: n }, span },
            )
        )
    }

    pub(crate) fn kw_table(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn owner(input: Node) -> Result<SOwner> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.clone().into_children();
            [player_expr(n)] => Owner::Player { player: n },
            [player_collection(n)] => Owner::PlayerCollection { player_collection: n },
            [team_expr(n)] => Owner::Team { team: n },
            [team_collection(n)] => Owner::TeamCollection { team_collection: n },
            [kw_table(_)] => Owner::Table,
            [use_memory(m)] => resolve_owner_memory(m, span.clone(), input),
            [ident(ids)..] => resolve_owner_idents(ids.collect(), span.clone(), input),
        );

        Ok(
            SOwner {
                node: node,
                span
            }
        )
    }

    pub(crate) fn group_of_owner(input: Node) -> Result<SCardSet> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [group(n), kw_of(_), owner(s)] => CardSet::GroupOwner { group: n, owner: s },
        );

        Ok(
            SCardSet {
                node: node,
                span
            }
        )       
    }

    pub(crate) fn kw_cards(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_players(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn card_set(input: Node) -> Result<SCardSet> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [group_of_owner(n)] => n,
                [group(n)] => SCardSet { node: CardSet::Group { group: n }, span },
                [use_memory(n)] => SCardSet { node: CardSet::Memory { memory: n }, span },
            )
        )
    }

    pub(crate) fn collection(input: Node) -> Result<SCollection> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.clone().into_children();
                [use_memory(memory)] => resolve_collection_memory(memory, span, input),
                [ident(ids)..] => resolve_collection_idents(ids.collect(), span, input),
                [use_memory(mis)..] => resolve_collection_use_memory(mis.collect(), span, input),
                [int_collection(n)] => SCollection {node: Collection::IntCollection { int: n }, span},
                [string_collection(n)] => SCollection {node: Collection::StringCollection { string: n }, span},
                [player_collection(n)] => SCollection {node: Collection::PlayerCollection { player: n }, span},
                [team_collection(n)] => SCollection {node: Collection::TeamCollection { team: n }, span},
                [location_collection(n)] => SCollection {node: Collection::LocationCollection { location: n }, span},
                [kw_cards(_), card_set(n)] => SCollection {node: Collection::CardSet {card_set: Box::new(n)}, span},
            )
        )
    }

    pub(crate) fn location_collection(input: Node) -> Result<SLocationCollection> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
                [location(ids)..] => LocationCollection::Literal { locations: ids.collect() },
                [use_memory(m)] => LocationCollection::Memory { memory: m },
        );
        
        Ok(
            SLocationCollection {
                node: node,
                span
            }
        )
    }

    pub(crate) fn team_expr_collection(input: Node) -> Result<STeamCollection> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [team_expr(teams)..] => STeamCollection { node: TeamCollection::Literal { teams: teams.collect() }, span },
            )
        )
    }

    pub(crate) fn kw_teams(input: Node) -> Result<()> {
        Ok(())
    }
    
    pub(crate) fn other_teams(input: Node) -> Result<()> {
        match_nodes!(input.into_children();
            [kw_other(_), kw_teams(_)] => (),
        );
        Ok(())
    }

    pub(crate) fn team_collection(input: Node) -> Result<STeamCollection> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [team_expr_collection(n)] => n,
                [other_teams(_)] => sruntime_team_collection(RuntimeTeamCollection::OtherTeams, span),
                [use_memory(m)] => STeamCollection { node: TeamCollection::Memory { memory: m }, span },
            )
        )
    }

    pub(crate) fn game_rule(input: Node) -> Result<SGameRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [setup_rule(s)] => GameRule::SetUp { setup: s },
            [action_rule(s)] => GameRule::Action { action: s },
            [scoring_rule(s)] => GameRule::Scoring { scoring: s },
        );

        Ok(
            SGameRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn setup_rule(input: Node) -> Result<SSetUpRule> {
        Ok(
            match_nodes!(input.into_children();
                [create_player(s)] => s,
                [create_team(s)] => s,
                [create_turnorder(s)] => s,
                [create_location(s)] => s,
                [create_card(s)] => s,
                [create_token(s)] => s,
                [create_precedence(s)] => s,
                [create_pointmap(s)] => s,
                [create_combo(s)] => s,
                [create_memory(s)] => s,
            )
        )
    }

    pub(crate) fn kw_player(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn player_name_list(input: Node) -> Result<Vec<SID>> {
        Ok(
            match_nodes!(input.into_children();
                [playername(players)..] => players.collect(),
            )
        )
    }

    pub(crate) fn create_player_names(input: Node) -> Result<Vec<SID>> {

        Ok(
            // TODO: Resolve Cloning later if possibler
            match_nodes!(input.clone().into_children();
                [playername(players)..] => {
                    let ps: Vec<SID> = players.collect();
                    for p in ps.iter() {
                        input.user_data().borrow_mut().symbols.insert(p.node.clone(), GameType::Player);
                    }

                    ps
                },
            )
        )
    }

    pub(crate) fn team_name_with_player_collection(input: Node) -> Result<(SID, SPlayerCollection)> {
        Ok(
            // TODO: Resolve Cloning later
            match_nodes!(input.clone().into_children();
                [teamname(teamname), kw_with(_), player_collection(p)] => {
                    input.user_data().borrow_mut().symbols.insert(teamname.node.clone(), GameType::Team);    
                    (teamname, p)
                },
            )
        )
    }

    pub(crate) fn create_player(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_player(_), create_player_names(p)] => SetUpRule::CreatePlayer { players: p },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn create_team(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_team(_), team_name_with_player_collection(twps)..] => SetUpRule::CreateTeams { teams: twps.collect() },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_turnorder(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_random(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn create_turnorder(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_turnorder(_), player_collection(p), kw_random(_)] => SetUpRule::CreateTurnorderRandom {player_collection: p},
            [kw_turnorder(_), player_collection(p)] => SetUpRule::CreateTurnorder {player_collection: p},
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_location(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_card(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_token(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_precedence(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_points(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_combo(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_memory(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn location_list(input: Node) -> Result<Vec<SID>> {
        Ok(
            match_nodes!(input.into_children();
               [location(locations)..] => locations.collect(),
            )
        )
    }

    pub(crate) fn create_location(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.clone().into_children();
            [kw_location(_), location_list(l), kw_on(_), owner(o)] => {
                for loc in l.iter() {
                    input.user_data().borrow_mut().symbols.insert(loc.node.clone(), GameType::Location);
                }
                SetUpRule::CreateLocation { locations: l, owner: o }
            },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn values(input: Node) -> Result<Vec<SID>> {
        Ok(
            match_nodes!(input.into_children();
               [value(values)..] => values.collect(),
            )
        )
    }

    pub(crate) fn for_key_values(input: Node) -> Result<(SID, Vec<SID>)> {
        Ok(
            match_nodes!(input.into_children();
               [kw_for(_), key(key), values(vs)] => (key, vs),
            )
        )
    }

    pub(crate) fn types(input: Node) -> Result<STypes> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [key(key), values(vs), for_key_values(kvs)..] => {
                let types = vec![vec![(key, vs)], kvs.collect()].concat();
                Types { types: types }
            },
            [key(key), values(vs)] => {
                let types = vec![(key, vs)];
                Types { types: types }
            },
        );

        Ok(
            STypes {
                node: node,
                span
            }
        )
    } 

    pub(crate) fn cards(input: Node) -> Result<Vec<STypes>> {
        Ok(
            match_nodes!(input.into_children();
                [types(ts)..] => ts.collect()
            )
        )
    } 

    pub(crate) fn create_card(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_card(_), kw_on(_), location(location), cards(t)] => SetUpRule::CreateCardOnLocation { location: location, cards: t },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn create_token(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_token(_), int_expr(i), token(token), kw_on(_), location(location)] => SetUpRule::CreateTokenOnLocation { int: i, token: token, location: location },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn key_value(input: Node) -> Result<(SID, SID)> {
        let res = match_nodes!(input.into_children();
            [key(k), value(v)] => (k, v),
        );

        Ok(res)
    }

    pub(crate) fn key_value_list(input: Node) -> Result<Vec<(SID, SID)>> {
        let res = match_nodes!(input.into_children();
            [key_value(kv)..] => kv.collect(),
        );

        Ok(res)
    }

    pub(crate) fn create_precedence(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_precedence(_), precedence(precedence), kw_on(_), key(key), values(vs)] => {
                let key_value: Vec<(SID, SID)> = vs.into_iter().map(|v| (key.clone(), v)).collect();
                SetUpRule::CreatePrecedence { precedence: precedence, kvs: key_value }
            },
            [kw_precedence(_), precedence(precedence), key_value_list(kvs)] => {
                SetUpRule::CreatePrecedence { precedence: precedence, kvs: kvs }
            },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn create_combo(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_combo(_), combo(combo), kw_where(_), filter_expr(f)] => SetUpRule::CreateCombo { combo: combo, filter: f },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn create_memory(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.clone().into_children();
            [kw_memory(_), memory(memory), memory_type(mt), kw_on(_), owner(o)] => {
                match &mt.node {
                    MemoryType::String { string: _ } => {
                        input.user_data().borrow_mut().memories.insert(memory.node.clone(), MemType::String);
                    },
                    MemoryType::Int { int: _ } => {
                        input.user_data().borrow_mut().memories.insert(memory.node.clone(), MemType::Int);
                    },
                    MemoryType::TeamCollection { teams: _ } => {
                        input.user_data().borrow_mut().memories.insert(memory.node.clone(), MemType::TeamCollection);
                    },
                    MemoryType::PlayerCollection { players: _ } => {
                        input.user_data().borrow_mut().memories.insert(memory.node.clone(), MemType::PlayerCollection);
                    },
                    MemoryType::LocationCollection { locations: _ } => {
                        input.user_data().borrow_mut().memories.insert(memory.node.clone(), MemType::LocationCollection);
                    },
                    MemoryType::StringCollection { strings: _ } => {
                        input.user_data().borrow_mut().memories.insert(memory.node.clone(), MemType::StringCollection);
                    },
                    MemoryType::IntCollection { ints: _ } => {
                        input.user_data().borrow_mut().memories.insert(memory.node.clone(), MemType::IntCollection);
                    },
                    MemoryType::CardSet { card_set: _ } => {
                        input.user_data().borrow_mut().memories.insert(memory.node.clone(), MemType::CardSet);
                    },      
                }
                SetUpRule::CreateMemoryWithMemoryType { memory: memory, memory_type: mt, owner: o }
            },
            [kw_memory(_), memory(memory), kw_on(_), owner(o)] => SetUpRule::CreateMemory { memory: memory, owner: o },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }
    
    pub(crate) fn value_int(input: Node) -> Result<(SID, SIntExpr)> {
        Ok(
            match_nodes!(input.children();
                [value(v), int_expr(i)] => (v, i)
            )
        )
    }

    pub(crate) fn value_int_list(input: Node) -> Result<Vec<(SID, SIntExpr)>> {
        Ok(
            match_nodes!(input.children();
                [value_int(vi)..] => vi.collect()
            )
        )
    }

    pub(crate) fn key_value_int(input: Node) -> Result<(SID, SID, SIntExpr)> {
        Ok(
            match_nodes!(input.children();
                [key(k), value(v), int_expr(i)] => (k, v, i)
            )
        )
    }

    pub(crate) fn key_value_int_list(input: Node) -> Result<Vec<(SID, SID, SIntExpr)>> {
        Ok(
            match_nodes!(input.children();
                [key_value_int(kvi)..] => kvi.collect()
            )
        )
    }

    pub(crate) fn create_pointmap(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_points(_), pointmap(pointmap), kw_on(_), key(key), value_int_list(vis)] => {
                let key_value_int: Vec<(SID, SID, SIntExpr)> = vis.into_iter().map(|(v, i)| (key.clone(), v, i)).collect();
                SetUpRule::CreatePointMap { pointmap: pointmap, kvis: key_value_int }
            },
            [kw_points(_), pointmap(pointmap), key_value_int_list(kvis)] => {
                SetUpRule::CreatePointMap { pointmap: pointmap, kvis: kvis }
            },
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn action_rule(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [move_action(a)] => SActionRule { node: ActionRule::Move { move_type: a }, span: span },
                [flip_action(b)] => b,
                [shuffle_action(c)] => c,
                [out_action(d)] => d,
                [reset_memory(f)] => f,
                [cycle_action(g)] => g,
                [bid_action(h)] => h,
                [end_action(i)] => i,
                [demand_action(j)] => j,
                [set_memory(e)] => e,
            )
        )
    }

    pub(crate) fn kw_flip(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn flip_action(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_flip(_), card_set(c), kw_to(_), status(s)] => ActionRule::FlipAction { card_set: c, status: s },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_shuffle(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn shuffle_action(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_shuffle(_), card_set(c)] => ActionRule::ShuffleAction { card_set: c },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_set(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_reset(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_successful(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_fail(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn memory_type(input: Node) -> Result<SMemoryType> {
        let span = OwnedSpan::from(input.as_span());
        // TODO: Resolve Cloning later!
        let node = match_nodes!(input.clone().into_children();
            // Resolving Memory
            [use_memory(memory)] => resolve_memory_type(memory, span.clone(), input),
            [int_expr(i)] => MemoryType::Int { int: i },
            [string_expr(s)] => MemoryType::String { string: s },
            [collection(c)] => {
                match c.node {
                    Collection::IntCollection { int } =>  MemoryType::IntCollection { ints: int },
                    Collection::StringCollection { string } => MemoryType::StringCollection { strings: string },
                    Collection::LocationCollection { location } => MemoryType::LocationCollection { locations: location },
                    Collection::PlayerCollection { player } => MemoryType::PlayerCollection { players: player },
                    Collection::TeamCollection { team } => MemoryType::TeamCollection { teams: team },
                    Collection::CardSet { card_set } => MemoryType::CardSet { card_set: *card_set },
                }
            },
        );

        Ok(
            SMemoryType {
                node: node,
                span
            }
        )
    }

    pub(crate) fn out_action(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_set(_), players(p), kw_out(_), kw_of(_), kw_stage(_)] => ActionRule::PlayerOutOfStageAction { players: p },
            [kw_set(_), players(p), kw_out(_), kw_of(_), kw_game(_), kw_successful(_)] => ActionRule::PlayerOutOfGameSuccAction { players: p },
            [kw_set(_), players(p), kw_out(_), kw_of(_), kw_game(_), kw_fail(_)] => ActionRule::PlayerOutOfGameFailAction { players: p },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn set_memory(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [memory(memory), kw_is(_), memory_type(m)] => ActionRule::SetMemory { memory: memory, memory_type: m },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn reset_memory(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_reset(_), memory(memory)] => ActionRule::ResetMemory { memory: memory },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }
    
    pub(crate) fn kw_cycle(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn cycle_action(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_cycle(_), kw_to(_), player_expr(p)] => ActionRule::CycleAction { player: p },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_bid(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_on(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_turn(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_with(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn bid_action(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_bid(_), quantity(q), kw_on(_), memory(memory), kw_of(_), owner(o)] => ActionRule::BidMemoryAction { memory: memory, quantity: q, owner: o },
            [kw_bid(_), quantity(q)] => ActionRule::BidAction { quantitiy: q },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn end_type(input: Node) -> Result<SEndType> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_turn(_)] => EndType::Turn,
            [kw_stage(_)] => EndType::CurrentStage,
            [stage(s)] => EndType::Stage { stage: s },
            [kw_game(_), kw_with(_), kw_winner(_), players(p)] => EndType::GameWithWinner { players: p },

        );

        Ok(
            SEndType {
                node: node,
                span
            }
        )
    }

    pub(crate) fn end_action(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_end(_), end_type(e)] => ActionRule::EndAction { end_type: e },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_demand(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_as(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn demand_type(input: Node) -> Result<SDemandType> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [card_position(c)] => DemandType::CardPosition { card_position: c },
            [string_expr(s)] => DemandType::String { string: s },
            [int_expr(i)] => DemandType::Int { int: i },
        );

        Ok(
            SDemandType {
                node: node,
                span
            }
        )
    }

    pub(crate) fn demand_action(input: Node) -> Result<SActionRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_demand(_), demand_type(d)] => ActionRule::DemandAction { demand_type: d },
            [kw_demand(_), demand_type(d), kw_as(_), memory(memory)] => ActionRule::DemandMemoryAction {demand_type: d, memory: memory },
        );

        Ok(
            SActionRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_from(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn card_set_to_card_set(input: Node) -> Result<SMoveCardSet> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [quantity(q), kw_from(_), card_set(c1), status(s), kw_to(_), card_set(c2)] => MoveCardSet::MoveQuantity { quantity: q, from: c1, status: s, to: c2 },
            [card_set(c1), status(s), kw_to(_), card_set(c2)] => MoveCardSet::Move { from: c1, status: s, to: c2 },
        );

        Ok(
            SMoveCardSet {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_move(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn classic_move(input: Node) -> Result<SClassicMove> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_move(_), card_set_to_card_set(m)] => ClassicMove::MoveCardSet { move_cs: m },
        );

        Ok(
            SClassicMove {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_deal(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn deal_move(input: Node) -> Result<SDealMove> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_deal(_), card_set_to_card_set(m)] => DealMove::MoveCardSet { deal_cs: m },
        );

        Ok(
            SDealMove {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_exchange(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn exchange_move(input: Node) -> Result<SExchangeMove> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_exchange(_), card_set_to_card_set(m)] => ExchangeMove::MoveCardSet { exchange_cs: m },
        );

        Ok(
            SExchangeMove {
                node: node,
                span
            }
        )
    }

    pub(crate) fn token_loc(input: Node) -> Result<STokenLocExpr> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [groupable(g)] => TokenLocExpr::Groupable { groupable: g },
            [groupable(g), kw_of(_), players(p)] => TokenLocExpr::GroupablePlayers { groupable: g, players: p },
        );

        Ok(
            STokenLocExpr {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_place(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn token_move(input: Node) -> Result<STokenMove> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_place(_), quantity(q), token(token), kw_from(_), token_loc(c1), kw_to(_), token_loc(c2)] => TokenMove::PlaceQuantity {quantity: q, token: token, from_loc: c1, to_loc: c2 },
            [kw_place(_), token(token), kw_from(_), token_loc(c1), kw_to(_), token_loc(c2)] => TokenMove::Place { token: token, from_loc: c1, to_loc: c2 },
        );

        Ok(
            STokenMove {
                node: node,
                span
            }
        )
    }

    pub(crate) fn move_action(input: Node) -> Result<SMoveType> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [classic_move(c)] => MoveType::Classic { classic: c },
            [deal_move(d)] => MoveType::Deal { deal: d },
            [exchange_move(e)] => MoveType::Exchange { exchange: e },
            [token_move(t)] => MoveType::Place { token: t },
        );

        Ok(
            SMoveType {
                node: node,
                span
            }
        )
    }

    pub(crate) fn scoring_rule(input: Node) -> Result<SScoringRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [score_rule(s)] => ScoringRule::ScoreRule { score_rule: s },
            [winner_rule(w)] => ScoringRule::WinnerRule { winner_rule: w },
        );

        Ok(
            SScoringRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_score(input: Node) -> Result<()> {
        Ok(())
    }
    
    pub(crate) fn kw_to(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_position(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn winner_type(input: Node) -> Result<SWinnerType> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_position(_)] => WinnerType::Position,
            [memory(m)] => WinnerType::Memory { memory: m },
            [kw_score(_)] => WinnerType::Score,
        );

        Ok(
            SWinnerType {
                node: node,
                span
            }
        )
    }

    pub(crate) fn score_rule(input: Node) -> Result<SScoreRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_score(_), int_expr(n), kw_to(_), memory(memory), kw_of(_), players(o)] => ScoreRule::ScoreMemory { int: n, memory: memory, players: o },
            [kw_score(_), int_expr(n), kw_to(_), players(o)] => ScoreRule::Score{int: n, players: o},
        );

        Ok(
            SScoreRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn kw_winner(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn winner_rule(input: Node) -> Result<SWinnerRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_winner(_), kw_is(_), extrema(e), winner_type(w)] => WinnerRule::WinnerWith { extrema: e, winner_type: w },
            [kw_winner(_), kw_is(_), players(p)] => WinnerRule::Winner { players: p },
        );

        Ok(
            SWinnerRule {
                node: node,
                span
            }
        )
    }
}

// ===========================================================================
// Helper
// ===========================================================================
#[allow(dead_code)]
pub(crate) fn saggregate_player(aggr: AggregatePlayer, span: OwnedSpan) -> SPlayerExpr {
    SPlayerExpr {
        node: PlayerExpr::Aggregate {
                aggregate: SAggregatePlayer {
                    node: aggr,
                    span: span.clone()
                },
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn saggregate_filter(aggr: AggregateFilter, span: OwnedSpan) -> SFilterExpr {
    SFilterExpr {
        node: FilterExpr::Aggregate {      
            aggregate: SAggregateFilter {
                node: aggr,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn saggregate_player_collection(aggr: AggregatePlayerCollection, span: OwnedSpan) -> SPlayerCollection {
    SPlayerCollection {
        node: PlayerCollection::Aggregate {    
            aggregate: SAggregatePlayerCollection {
                node: aggr,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn sruntime_player_collection(runt: RuntimePlayerCollection, span: OwnedSpan) -> SPlayerCollection {
    SPlayerCollection {
        node: PlayerCollection::Runtime {       
            runtime: SRuntimePlayerCollection {
                node: runt,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn sruntime_team_collection(runt: RuntimeTeamCollection, span: OwnedSpan) -> STeamCollection {
    STeamCollection {
        node: TeamCollection::Runtime {     
            runtime: SRuntimeTeamCollection {
                node: runt,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn saggregate_team(aggr: AggregateTeam, span: OwnedSpan) -> STeamExpr {
    STeamExpr {
        node: TeamExpr::Aggregate {        
            aggregate: SAggregateTeam {
                node: aggr,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn sruntime_player(runt: RuntimePlayer, span: OwnedSpan) -> SPlayerExpr {
    SPlayerExpr {
        node: PlayerExpr::Runtime {        
            runtime: SRuntimePlayer {
                node: runt,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn squery_card_position(quer: QueryCardPosition, span: OwnedSpan) -> SCardPosition {
    SCardPosition {
        node: CardPosition::Query {
            query: SQueryCardPosition {
                node: quer,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn saggregate_card_position(aggr: AggregateCardPosition, span: OwnedSpan) -> SCardPosition {
    SCardPosition {
        node: CardPosition::Aggregate {
            aggregate: SAggregateCardPosition {
                node: aggr,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn squery_int(quer: QueryInt, span: OwnedSpan) -> SIntExpr {
    SIntExpr {
        node: IntExpr::Query {
            query: SQueryInt {
                node: quer,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn saggregate_int(aggr: AggregateInt, span: OwnedSpan) -> SIntExpr {
    SIntExpr {
        node: IntExpr::Aggregate {
            aggregate: SAggregateInt {
                node: aggr,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn sruntime_int(runt: RuntimeInt, span: OwnedSpan) -> SIntExpr {
    SIntExpr {
        node: IntExpr::Runtime {   
            runtime: SRuntimeInt {
                node: runt,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn squery_string(quer: QueryString, span: OwnedSpan) -> SStringExpr {
    SStringExpr {
        node: StringExpr::Query {
            query: SQueryString {
                node: quer,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn saggregate_compare_bool(cmp: CompareBool, span: OwnedSpan) -> SBoolExpr {
    SBoolExpr {
        node: BoolExpr::Aggregate {
            aggregate: SAggregateBool {
                node: AggregateBool::Compare {
                    cmp_bool: SCompareBool {
                        node: cmp,
                        span: span.clone()
                    }
                },
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

#[allow(dead_code)]
pub(crate) fn saggregate_bool(aggr: AggregateBool, span: OwnedSpan) -> SBoolExpr {
    SBoolExpr {
        node: BoolExpr::Aggregate {
            aggregate: SAggregateBool {
                node: aggr,
                span: span.clone()
            }
        },
        span: span.clone()
    }
}

// ===========================================================================
// ===========================================================================
// Resolving-Logic
// ===========================================================================
// ===========================================================================

pub(crate) fn resolve_collection_memory(memory: SUseMemory, span: OwnedSpan, input: Node) -> SCollection {
    let mem = match &memory.node {
        UseMemory::Memory { memory } => &memory.node,
        UseMemory::WithOwner { memory, owner: _ } => &memory.node,
    };
    if let Some(m) = input.user_data().borrow().memories.get(mem) {
        let node = match m {
            MemType::StringCollection => {
                Collection::StringCollection { 
                    string: SStringCollection {
                        node: StringCollection::Memory { memory: memory.clone() },
                        span: span.clone()
                    }
                }
            },
            MemType::PlayerCollection => {
                Collection::PlayerCollection { 
                    player: SPlayerCollection {
                        node: PlayerCollection::Memory { memory: memory.clone() },
                        span: span.clone()
                    }
                }
            },
            MemType::TeamCollection => {
                Collection::TeamCollection { 
                    team: STeamCollection {
                        node: TeamCollection::Memory { memory: memory.clone() },
                        span: span.clone()
                    }
                }
            },
            MemType::LocationCollection => {
                Collection::LocationCollection { 
                    location: SLocationCollection {
                        node: LocationCollection::Memory { memory: memory.clone() },
                        span: span.clone()
                    }
                }
            },
            MemType::CardSet => {
                Collection::CardSet { 
                    card_set: Box::new(
                        SCardSet {
                            node: CardSet::Memory { memory: memory.clone() },
                            span: span.clone()
                        }
                    )
                }
            },
            // FallBack
            _ => {
                Collection::IntCollection { 
                    int: SIntCollection {
                        node: IntCollection::Memory { memory: memory.clone() },
                        span: span.clone()
                    }
                }
            }
        };

        return SCollection {
            node: node,
            span
        }
    }
    
    // FallBack
    SCollection {
        node: Collection::IntCollection { 
            int: SIntCollection {
                node: IntCollection::Memory { memory: memory.clone() },
                span: span.clone()
            }
        },
        span: span
    }
}

pub(crate) fn resolve_collection_idents(ids: Vec<SID>, span: OwnedSpan, input: Node) -> SCollection {
    let names: Vec<SID> = ids;
    for name in names.iter() {
        if let Some(game_type) = input.user_data().borrow().symbols.get(&name.node) {
            let result = match game_type {
                GameType::Player => {
                    SCollection {
                        node: Collection::PlayerCollection { 
                            player: SPlayerCollection { 
                                node: PlayerCollection::Literal {
                                    players: names.iter().map(|n| 
                                        {
                                            SPlayerExpr {
                                                node: PlayerExpr::Literal { name: n.clone()},
                                                span: span.clone()
                                            }   
                                        }
                                    ).collect()
                                },
                                span: span.clone()
                            }
                        },
                        span
                    }
                },
                GameType::Team => {
                    SCollection {
                        node: Collection::TeamCollection { 
                            team: STeamCollection { 
                                node: TeamCollection::Literal {
                                    teams: names.iter().map(|n| 
                                        {
                                            STeamExpr {
                                                node: TeamExpr::Literal { name: n.clone()},
                                                span: span.clone()
                                            }   
                                        }
                                    ).collect()
                                },
                                span: span.clone()
                            }
                        },
                        span
                    }
                },
                GameType::Location => {
                    SCollection {
                        node: Collection::LocationCollection { 
                            location: SLocationCollection { 
                                node: LocationCollection::Literal {
                                    locations: names
                                },
                                span: span.clone()
                            }
                        },
                        span
                    }
                },
                // FallBack
                _ => {
                    SCollection {
                        node: Collection::PlayerCollection { 
                            player: SPlayerCollection { 
                                node: PlayerCollection::Literal {
                                    players: names.iter().map(|n| 
                                        {
                                            SPlayerExpr {
                                                node: PlayerExpr::Literal { name: n.clone()},
                                                span: span.clone()
                                            }   
                                        }
                                    ).collect()
                                },
                                span: span.clone()
                            }
                        },
                        span
                    }         
                },
            };

            return result
        }
    }

    // FallBack
    SCollection {
        node: Collection::PlayerCollection { 
            player: SPlayerCollection { 
                node: PlayerCollection::Literal {
                    players: names.iter().map(|n| 
                        {
                            SPlayerExpr {
                                node: PlayerExpr::Literal { name: n.clone()},
                                span: span.clone()
                            }   
                        }
                    ).collect()
                },
                span: span.clone()
            }
        },
        span
    }
}

pub(crate) fn resolve_collection_use_memory(ids: Vec<SUseMemory>, span: OwnedSpan, input: Node) -> SCollection {
    // ints
    // checking the first type
    let id = ids.first().unwrap();
    match &id.node {
        UseMemory::Memory { memory} => {
            if let Some(ty) = input.user_data().borrow().memories.get(&memory.node) {
                match ty {
                    MemType::Int => {
                        return SCollection {
                            node: Collection::IntCollection {
                                int: SIntCollection {
                                    node: IntCollection::Literal { ints: 
                                        ids.iter().map(|i|
                                            SIntExpr {
                                                node: IntExpr::Memory {
                                                    memory: i.clone(),
                                                },
                                                span: span.clone()
                                            }
                                        ).collect()
                                    },
                                    span: span.clone()
                                }
                            },
                            span
                        }
                    },
                    MemType::String => {
                        return SCollection {
                            node: Collection::StringCollection {
                                string: SStringCollection {
                                    node: StringCollection::Literal { strings: 
                                        ids.iter().map(|i|
                                            SStringExpr {
                                                node: StringExpr::Memory {
                                                    memory: i.clone(),
                                                },
                                                span: span.clone()
                                            }
                                        ).collect()
                                    },
                                    span: span.clone()
                                }
                            },
                            span
                        }
                    },
                    // FallBack is IntCollection
                    _ => {
                        return SCollection {
                            node: Collection::IntCollection {
                                int: SIntCollection {
                                    node: IntCollection::Literal { ints: 
                                        ids.iter().map(|i|
                                            SIntExpr {
                                                node: IntExpr::Memory {
                                                    memory: i.clone(),
                                                },
                                                span: span.clone()
                                            }
                                        ).collect()
                                    },
                                    span: span.clone()
                                }
                            },
                            span
                        }
                    }
                }
            } else {
                // FallBack is IntCollection
                return SCollection {
                    node: Collection::IntCollection {
                        int: SIntCollection {
                            node: IntCollection::Literal { ints: 
                                ids.iter().map(|i|
                                    SIntExpr {
                                        node: IntExpr::Memory {
                                            memory: i.clone(),
                                        },
                                        span: span.clone()
                                    }
                                ).collect()
                            },
                            span: span.clone()
                        }
                    },
                    span
                }
            }
        },
        UseMemory::WithOwner { memory, owner: _} => {
            if let Some(ty) = input.user_data().borrow().memories.get(&memory.node) {
                match ty {
                    MemType::Int => {
                        return SCollection {
                            node: Collection::IntCollection {
                                int: SIntCollection {
                                    node: IntCollection::Literal { ints: 
                                        ids.iter().map(|i|
                                            SIntExpr {
                                                node: IntExpr::Memory {
                                                    memory: i.clone(),
                                                },
                                                span: span.clone()
                                            }
                                        ).collect()
                                    },
                                    span: span.clone()
                                }
                            },
                            span
                        }
                    },
                    MemType::String => {
                        return SCollection {
                            node: Collection::StringCollection {
                                string: SStringCollection {
                                    node: StringCollection::Literal { strings: 
                                        ids.iter().map(|i|
                                            SStringExpr {
                                                node: StringExpr::Memory {
                                                    memory: i.clone(),
                                                },
                                                span: span.clone()
                                            }
                                        ).collect()
                                    },
                                    span: span.clone()
                                }
                            },
                            span
                        }
                    },
                    // FallBack is IntCollection
                    _ => {
                        return SCollection {
                            node: Collection::IntCollection {
                                int: SIntCollection {
                                    node: IntCollection::Literal { ints: 
                                        ids.iter().map(|i|
                                            SIntExpr {
                                                node: IntExpr::Memory {
                                                    memory: i.clone(),
                                                },
                                                span: span.clone()
                                            }
                                        ).collect()
                                    },
                                    span: span.clone()
                                }
                            },
                            span
                        }
                    }
                }
            } else {
                // FallBack is IntCollection
                return SCollection {
                    node: Collection::IntCollection {
                        int: SIntCollection {
                            node: IntCollection::Literal { ints: 
                                ids.iter().map(|i|
                                    SIntExpr {
                                        node: IntExpr::Memory {
                                            memory: i.clone(),
                                        },
                                        span: span.clone()
                                    }
                                ).collect()
                            },
                            span: span.clone()
                        }
                    },
                    span
                }
            }
        },
    }
}

pub(crate) fn resolve_memory_type(memory: SUseMemory, span: OwnedSpan, input: Node) -> MemoryType {
    let mem = match &memory.node {
        UseMemory::Memory { memory } => &memory.node,
        UseMemory::WithOwner { memory, owner: _ } => &memory.node,
    };
    if let Some(mt) = input.user_data().borrow().memories.get(mem) {
        match mt {
            MemType::Int => {
                return MemoryType::Int { 
                    int: SIntExpr {
                        node: IntExpr::Memory { memory },
                        span: span.clone(),
                    } 
                }
            },
            MemType::String => {
                return MemoryType::String { 
                    string: SStringExpr {
                        node: StringExpr::Memory { memory },
                        span: span.clone(),
                    } 
                }
            },
            _ => {
                let col = resolve_collection_memory(memory, span.clone(), input.clone());
                let node = match col.node {
                    Collection::IntCollection { int } => {
                        MemoryType::IntCollection { ints: int }
                    },
                    Collection::StringCollection { string } => {
                        MemoryType::StringCollection { strings: string }
                    },
                    Collection::LocationCollection { location } => {
                        MemoryType::LocationCollection { locations: location }
                    },
                    Collection::PlayerCollection { player } => {
                        MemoryType::PlayerCollection { players: player }
                    },
                    Collection::TeamCollection { team } => {
                        MemoryType::TeamCollection { teams: team }
                    },
                    Collection::CardSet { card_set } => {
                        MemoryType::CardSet { card_set: *card_set }
                    },
                };

                return node
            },
        }
    }

    // FallBack
    MemoryType::Int { 
        int: SIntExpr {
            node: IntExpr::Memory { memory },
            span: span.clone(),
        } 
    }
}

pub(crate) fn resolve_owner_idents(ids: Vec<SID>, span: OwnedSpan, input: Node) -> Owner {
    let names: Vec<SID> = ids;
    for name in names.iter() {
        if let Some(game_type) = input.user_data().borrow().symbols.get(&name.node) {
            let result = match game_type {
                GameType::Player => {
                    Owner::PlayerCollection { 
                        player_collection: SPlayerCollection { 
                            node: PlayerCollection::Literal {
                                players: names.iter().map(|n| 
                                    {
                                        SPlayerExpr {
                                            node: PlayerExpr::Literal { name: n.clone()},
                                            span: span.clone()
                                        }   
                                    }
                                ).collect()
                            },
                            span: span.clone()
                        }
                    }
                },
                GameType::Team => {
                    Owner::TeamCollection { 
                        team_collection: STeamCollection { 
                            node: TeamCollection::Literal {
                                teams: names.iter().map(|n| 
                                    {
                                        STeamExpr {
                                            node: TeamExpr::Literal { name: n.clone()},
                                            span: span.clone()
                                        }   
                                    }
                                ).collect()
                            },
                            span: span.clone()
                        }
                    }
                },

                // FallBack
                _ => {
                    Owner::PlayerCollection { 
                        player_collection: SPlayerCollection { 
                            node: PlayerCollection::Literal {
                                players: names.iter().map(|n| 
                                    {
                                        SPlayerExpr {
                                            node: PlayerExpr::Literal { name: n.clone()},
                                            span: span.clone()
                                        }   
                                    }
                                ).collect()
                            },
                            span: span.clone()
                        }
                    }         
                },
            };

            return result
        }
    }

    // FallBack
    Owner::PlayerCollection { 
        player_collection: SPlayerCollection { 
            node: PlayerCollection::Literal {
                players: names.iter().map(|n| 
                    {
                        SPlayerExpr {
                            node: PlayerExpr::Literal { name: n.clone()},
                            span: span.clone()
                        }   
                    }
                ).collect()
            },
            span: span.clone()
        }
    }
}


pub(crate) fn resolve_owner_memory(memory: SUseMemory, span: OwnedSpan, input: Node) -> Owner {
    let mem = match &memory.node {
        UseMemory::Memory { memory } => &memory.node,
        UseMemory::WithOwner { memory, owner: _ } => &memory.node,
    };
    if let Some(m) = input.user_data().borrow().memories.get(mem) {
        let node = match m {
            MemType::PlayerCollection => {
                Owner::PlayerCollection { 
                    player_collection: SPlayerCollection {
                        node: PlayerCollection::Memory { memory: memory.clone() },
                        span: span
                    }
                }
            },
            MemType::TeamCollection => {
                Owner::TeamCollection { 
                    team_collection: STeamCollection {
                        node: TeamCollection::Memory { memory: memory.clone() },
                        span: span
                    }
                }
            },
            // FallBack
            _ => {
                Owner::PlayerCollection { 
                    player_collection: SPlayerCollection {
                        node: PlayerCollection::Memory { memory: memory.clone() },
                        span: span
                    }
                }
            }
        };

        return node
    }
    
    // FallBack
    Owner::PlayerCollection { 
        player_collection: SPlayerCollection {
            node: PlayerCollection::Memory { memory: memory.clone() },
            span: span
        }
    }
}

fn resolve_use_memory_eq_use_memory(m1: Spanned<UseMemory>, e: OwnedSpan, m2: Spanned<UseMemory>, span: OwnedSpan, input: Node) -> SBoolExpr {
    let mid = match &m1.node {
        UseMemory::Memory { memory } => &memory.node,
        UseMemory::WithOwner { memory, owner: _ } => &memory.node,
    };
    let mid1 = match &m2.node {
        UseMemory::Memory { memory } => &memory.node,
        UseMemory::WithOwner { memory, owner: _ } => &memory.node,
    };

    if let Some(ty1) = input.user_data().borrow().memories.get(mid) {
        if let Some(ty2) = input.user_data().borrow().memories.get(mid1) {
            if ty1 == ty2 {
                match ty1 {
                    MemType::Int => {
                        let int = SIntExpr {
                            node: IntExpr::Memory { memory: m1 },
                            span: span.clone()
                        };
                        let int1 = SIntExpr {
                            node: IntExpr::Memory { memory: m2 },
                            span: span.clone()
                        };
                        let int_cmp = SIntCompare {
                            node: IntCompare::Eq,
                            span: e
                        };

                        return saggregate_compare_bool(CompareBool::Int { int , cmp: int_cmp, int1 }, span)
                    },
                    MemType::String => {
                        let string = SStringExpr {
                            node: StringExpr::Memory { memory: m1 },
                            span: span.clone()
                        };
                        let string1 = SStringExpr {
                            node: StringExpr::Memory { memory: m2 },
                            span: span.clone()
                        };
                        let string_cmp = SStringCompare {
                            node: StringCompare::Eq,
                            span: e
                        };

                        return saggregate_compare_bool(CompareBool::String { string , cmp: string_cmp, string1 }, span)
                    },
                    MemType::CardSet => {
                        let card_set = SCardSet {
                            node: CardSet::Memory { memory: m1 },
                            span: span.clone()
                        };
                        let card_set1 = SCardSet {
                            node: CardSet::Memory { memory: m2 },
                            span: span.clone()
                        };
                        let card_set_cmp = SCardSetCompare {
                            node: CardSetCompare::Eq,
                            span: e
                        };

                        return saggregate_compare_bool(CompareBool::CardSet { card_set , cmp: card_set_cmp, card_set1 }, span)
                    },
                    // FallBack
                    _ => {}
                }
            }
        }
    }

    // FallBack
    let int = SIntExpr {
        node: IntExpr::Memory { memory: m1 },
        span: span.clone()
    };
    let int1 = SIntExpr {
        node: IntExpr::Memory { memory: m2 },
        span: span.clone()
    };
    let int_cmp = SIntCompare {
        node: IntCompare::Eq,
        span: e
    };

    saggregate_compare_bool(CompareBool::Int { int , cmp: int_cmp, int1 }, span)
}


fn resolve_use_memory_neq_use_memory(m1: Spanned<UseMemory>, ne: OwnedSpan, m2: Spanned<UseMemory>, span: OwnedSpan, input: Node) -> SBoolExpr {
    let mid = match &m1.node {
        UseMemory::Memory { memory } => &memory.node,
        UseMemory::WithOwner { memory, owner: _ } => &memory.node,
    };
    let mid1 = match &m2.node {
        UseMemory::Memory { memory } => &memory.node,
        UseMemory::WithOwner { memory, owner: _ } => &memory.node,
    };

    if let Some(ty1) = input.user_data().borrow().memories.get(mid) {
        if let Some(ty2) = input.user_data().borrow().memories.get(mid1) {
            if ty1 == ty2 {
                match ty1 {
                    MemType::Int => {
                        let int = SIntExpr {
                            node: IntExpr::Memory { memory: m1 },
                            span: span.clone()
                        };
                        let int1 = SIntExpr {
                            node: IntExpr::Memory { memory: m2 },
                            span: span.clone()
                        };
                        let int_cmp = SIntCompare {
                            node: IntCompare::Neq,
                            span: ne
                        };

                        return saggregate_compare_bool(CompareBool::Int { int , cmp: int_cmp, int1 }, span)
                    },
                    MemType::String => {
                        let string = SStringExpr {
                            node: StringExpr::Memory { memory: m1 },
                            span: span.clone()
                        };
                        let string1 = SStringExpr {
                            node: StringExpr::Memory { memory: m2 },
                            span: span.clone()
                        };
                        let string_cmp = SStringCompare {
                            node: StringCompare::Neq,
                            span: ne
                        };

                        return saggregate_compare_bool(CompareBool::String { string , cmp: string_cmp, string1 }, span)
                    },
                    MemType::CardSet => {
                        let card_set = SCardSet {
                            node: CardSet::Memory { memory: m1 },
                            span: span.clone()
                        };
                        let card_set1 = SCardSet {
                            node: CardSet::Memory { memory: m2 },
                            span: span.clone()
                        };
                        let card_set_cmp = SCardSetCompare {
                            node: CardSetCompare::Neq,
                            span: ne
                        };

                        return saggregate_compare_bool(CompareBool::CardSet { card_set , cmp: card_set_cmp, card_set1 }, span)
                    },
                    // FallBack
                    _ => {}
                }
            }
        }
    }

    // FallBack
    let int = SIntExpr {
        node: IntExpr::Memory { memory: m1 },
        span: span.clone()
    };
    let int1 = SIntExpr {
        node: IntExpr::Memory { memory: m2 },
        span: span.clone()
    };
    let int_cmp = SIntCompare {
        node: IntCompare::Neq,
        span: ne
    };

    saggregate_compare_bool(CompareBool::Int { int , cmp: int_cmp, int1 }, span)
}