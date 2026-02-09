use pest_consume::{Parser, match_nodes};

use crate::{spans::*};
use crate::{ast::ast::*};


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CGDSLParser;

use pest_consume::Error;
pub type Result<T> = std::result::Result<T, Error<Rule>>;
pub type Node<'i> = pest_consume::Node<'i, Rule, ()>;

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
                [seq_stage(n)] => FlowComponent::Stage(n),
                [if_rule(s)] => FlowComponent::IfRule(s),
                [choice_rule(t)] => FlowComponent::ChoiceRule(t),
                [optional_rule(l)] => FlowComponent::OptionalRule(l),
                [game_rule(k)] => FlowComponent::Rule(k),
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
            [kw_until(_), bool_expr(b)] => EndCondition::UntilBool(b),
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
            [kw_until(_), bool_expr(b), logical_compare(l), repetitions(r)] => EndCondition::UntilBoolRep(b, l, r),
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
            [repetitions(r)] => EndCondition::UntilRep(r),
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
                [location(n), kw_top(_)] => squery_card_position(QueryCardPosition::Top(n), span), 
            )
        )
    }

    pub(crate) fn location_bottom(input: Node) -> Result<SCardPosition> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [location(n), kw_bottom(_)] => squery_card_position(QueryCardPosition::Bottom(n), span), 
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
                [extrema(n), kw_for(_), card_set(s), kw_using(_), kw_points(_), ident(t)] => saggregate_card_position(AggregateCardPosition::ExtremaPointMap(n, Box::new(s), t), span), 
                [extrema(n), kw_for(_), card_set(s), kw_using(_), ident(t)] => saggregate_card_position(AggregateCardPosition::ExtremaPrecedence(n, Box::new(s), t), span), 
            )
        )
    }

    pub(crate) fn location_int_expr(input: Node) -> Result<SCardPosition> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [location(n), int_expr(s)] => squery_card_position(QueryCardPosition::At(n, s), span), 
            )
        )
    }

    pub(crate) fn card_position(input: Node) -> Result<SCardPosition> {
        Ok(
            match_nodes!(input.into_children();
                [location_top(n)] => n, 
                [location_bottom(n)] => n, 
            )
        )
    }

    pub(crate) fn owner_of_card_position(input: Node) -> Result<AggregatePlayer> {
        Ok(
            match_nodes!(input.into_children();
                [kw_owner(_), kw_of(_), card_position(n)] => AggregatePlayer::OwnerOfCardPostion(Box::new(n)),
            )
        )
    }

    pub(crate) fn kw_owner(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_of(input: Node) -> Result<()> {
        Ok(())
    }
    
    pub(crate) fn owner_of_memory(input: Node) -> Result<AggregatePlayer> {
        Ok(
            match_nodes!(input.into_children();
                [kw_owner(_), kw_of(_), extrema(n), memory(s)] => AggregatePlayer::OwnerOfMemory(n, s),
            )
        )
    }

    pub(crate) fn player_expr(input: Node) -> Result<SPlayerExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [kw_current(n)] => sruntime_player(n, span),
                [kw_previous(s)] => sruntime_player(s, span),
                [kw_competitor(t)] => sruntime_player(t, span),
                [kw_next(r)] => sruntime_player(r, span),
                [owner_of_card_position(q)] => saggregate_player(q, span),
                [owner_of_memory(q)] => saggregate_player(q, span),
                [playername(p)] => SPlayerExpr { node: PlayerExpr::Literal(p), span }
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
            [int_expr(n), int_op(s), int_expr(t)] => IntExpr::Binary(Box::new(n), s, Box::new(t))
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
                [int_collection(n), int_expr(s)] => squery_int(QueryInt::IntCollectionAt(Box::new(n), Box::new(s)), span),
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
                [kw_size(_), kw_of(_), collection(n)] => saggregate_int(AggregateInt::SizeOf(n), span),
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
                [kw_sum(_), kw_of(_), int_collection(n)] => saggregate_int(AggregateInt::SumOfIntCollection(n), span),
            )
        )
    }

    pub(crate) fn sum_of_card_set(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [kw_sum(_), kw_of(_), card_set(n), kw_using(_), pointmap(s)] => saggregate_int(AggregateInt::SumOfCardSet(Box::new(n), s), span),
            )
        )
    }

    pub(crate) fn extrema_of_int_collection(input: Node) -> Result<SIntExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [extrema(n), kw_of(_), int_collection(s)] => saggregate_int(AggregateInt::ExtremaIntCollection(n, s), span),
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
                node: IntExpr::Literal(int?),
                span
            }
        )
    }

    pub(crate) fn int_expr(input: Node) -> Result<SIntExpr> {
        Ok(
            match_nodes!(input.into_children();
                [bin_int_op(n)] => n,
                [int_collection_at(n)] => n,
                [size_of_collection(n)] => n,
                [sum_of_card_set(n)] => n,
                [sum_of_int_collection(n)] => n,
                [extrema_of_int_collection(n)] => n,
                [int(n)] => n,
            )
        )
    }

    pub(crate) fn key_of_card_position(input: Node) -> Result<SStringExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [key(n), kw_of(_), card_position(s)] => squery_string(QueryString::KeyOf(n, s), span),
            )
        )
    }

    pub(crate) fn string_collection_at(input: Node) -> Result<SStringExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [string_collection(n), int_expr(s)] => squery_string(QueryString::StringCollectionAt(n, s), span),
            )
        )
    }

    pub(crate) fn string_expr(input: Node) -> Result<SStringExpr> {
        Ok(
            match_nodes!(input.into_children();
                [key_of_card_position(n)] => n,
                [string_collection_at(n)] => n,
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
                [kw_team(_), kw_of(_), player_expr(n)] => saggregate_team(AggregateTeam::TeamOf(n), span),
            )
        )
    }

    pub(crate) fn team_expr(input: Node) -> Result<STeamExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [team_of_player(n)] => n,
                [teamname(n)] => STeamExpr { node: TeamExpr::Literal(n), span},
            )
        )
    }

    pub(crate) fn int_collection(input: Node) -> Result<SIntCollection> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [int_expr(int_exprs)..] => IntCollection { ints: int_exprs.collect() },
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
            [string_expr(string_exprs)..] => StringCollection { strings: string_exprs.collect() },
        );

        Ok(
            SStringCollection {
                node: node,
                span
            }
        )
    }

    pub(crate) fn eq(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn neq(input: Node) -> Result<()> {
        Ok(())
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
                [card_set(n), card_set_compare(s), card_set(t)] => saggregate_compare_bool(CompareBool::CardSet(n, s, t), span),
            )
        )
    }

    pub(crate) fn string_expr_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [string_expr(n), string_expr_compare(s), string_expr(t)] => saggregate_compare_bool(CompareBool::String(n, s, t), span),
            )
        )
    }

    pub(crate) fn player_expr_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [player_expr(n), player_expr_compare(s), player_expr(t)] => saggregate_compare_bool(CompareBool::Player(n, s, t), span),
            )
        )
    }

    pub(crate) fn team_expr_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [team_expr(n), team_expr_compare(s), team_expr(t)] => saggregate_compare_bool(CompareBool::Team(n, s, t), span),
            )
        )
    }

    pub(crate) fn int_expr_bool(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [int_expr(n), int_compare(s), int_expr(t)] => saggregate_compare_bool(CompareBool::Int(n, s, t), span),
            )
        )
    }

    pub(crate) fn kw_is(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn kw_empty(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn card_set_empty(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [card_set(n), kw_is(_), kw_not(_), kw_empty(_)] => saggregate_bool(AggregateBool::CardSetNotEmpty(n), span),
                [card_set(n), kw_is(_), kw_empty(_)] => saggregate_bool(AggregateBool::CardSetEmpty(n), span),
            )
        )
    }

    pub(crate) fn bool_expr_binary(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [bool_expr(n), bool_op(s), bool_expr(t)] => SBoolExpr { node: BoolExpr::Binary(Box::new(n), s, Box::new(t)), span },
            )
        )
    }

    pub(crate) fn bool_expr_unary(input: Node) -> Result<SBoolExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [unary_op(n), bool_expr(s)] => SBoolExpr { node: BoolExpr::Unary(n, Box::new(s)), span },
            )
        )
    }

    pub(crate) fn players(input: Node) -> Result<SPlayers> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [player_expr(n)] => SPlayers { node: Players::Player(n), span },
                [player_collection(n)] => SPlayers { node: Players::PlayerCollection(n), span },
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
                [player_expr(players)..] => SPlayerCollection { node: PlayerCollection::Literal(players.collect()), span },
            )
        )
    }

    pub(crate) fn kw_others(input: Node) -> Result<()> {
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
                [quantifier(n)] => saggregate_player_collection(AggregatePlayerCollection::Quantifier(n), span),
                [kw_others(_)] => sruntime_player_collection(RuntimePlayerCollection::Others, span),
                [kw_playersin(_)] => sruntime_player_collection(RuntimePlayerCollection::PlayersIn, span),
                [kw_playersout(_)] => sruntime_player_collection(RuntimePlayerCollection::PlayersOut, span),
            )
        )
    }

    pub(crate) fn kw_out(input: Node) ->  Result<()> {
        Ok(())
    }

    pub(crate) fn kw_stage(input: Node) ->  Result<()> {
        Ok(())
    }
    
    pub(crate) fn kw_game(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn out_of(input: Node) -> Result<SOutOf> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_stage(_)] => OutOf::CurrentStage,
            [stage(n)] => OutOf::Stage(n),
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
                [players(n), kw_out(_), kw_of(_), out_of(s)] => saggregate_bool(AggregateBool::OutOfPlayer(n, s), span),
            )
        )
    }

    pub(crate) fn bool_expr(input: Node) -> Result<SBoolExpr> {
        Ok(
            match_nodes!(input.into_children();
                [card_set_bool(n)] => n,
                [string_expr_bool(n)] => n,
                [player_expr_bool(n)] => n,
                [team_expr_bool(n)] => n,
                [int_expr_bool(n)] => n,
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
                [key(key), kw_distinct(_)] => saggregate_filter(AggregateFilter::Same(key), span),
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
                [key(key), kw_adjacent(_), kw_using(_), precedence(prec)] => saggregate_filter(AggregateFilter::Adjacent(key, prec), span),
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
                [key(key), kw_higher(_), kw_using(_), precedence(prec)] => saggregate_filter(AggregateFilter::Higher(key, prec), span),
            )
        )
    }

    pub(crate) fn kw_lower(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn key_lower(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [key(key), kw_lower(_), kw_using(_), precedence(prec)] => saggregate_filter(AggregateFilter::Lower(key, prec), span),
            )
        )
    }

    pub(crate) fn key_same(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [key(key), kw_same(_)] => saggregate_filter(AggregateFilter::Same(key), span),
            )
        )
    }

    pub(crate) fn size_int(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [kw_size(_), int_compare(n), int_expr(s)] => saggregate_filter(AggregateFilter::Size(n, Box::new(s)), span),
            )
        )
    }

    pub(crate) fn key_string(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [key(key), string_expr_compare(n), string_expr(s)] => saggregate_filter(AggregateFilter::KeyString(key, n, Box::new(s)), span),
            )
        )
    }

    pub(crate) fn filter_combo(input: Node) -> Result<SFilterExpr> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.children();
                [kw_not(_), combo(combo)] => saggregate_filter(AggregateFilter::NotCombo(combo), span),
                [combo(combo)] => saggregate_filter(AggregateFilter::Combo(combo), span),
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
            [filter_expr(n), filter_op(s), filter_expr(t)] => FilterExpr::Binary(Box::new(n), s, Box::new(t)),
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

    pub(crate) fn int_range_helper(input: Node) -> Result<(SIntCompare, SIntExpr)> {
        Ok(
            match_nodes!(input.into_children();
                [int_compare(n), int_expr(s)] => (n, s),
            )
        )
    }

    pub(crate) fn int_range(input: Node) -> Result<SIntRange> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [int_range_helper(irhs)..] => IntRange { op_int: irhs.collect() },
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
            [int_expr(n)] => Quantity::Int(n),
            [quantifier(n)] => Quantity::Quantifier(n),
            [int_range(n)] => Quantity::IntRange(n),
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
            [location_collection(n)] => Groupable::LocationCollection(n),
            [location(n)] => Groupable::Location(n),
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
            [groupable(n), kw_where(_), filter_expr(s)] => Group::Where(n, s),
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
            [combo(combo), kw_in(_), groupable(s)] => Group::Combo(combo, s),
            [combo(combo), kw_not(_), kw_in(_), groupable(s)] => Group::NotCombo(combo, s),
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
                [groupable(n)] => SGroup { node: Group::Groupable(n), span },
                [combo_in_groupable(n)] => n,
                [card_position(n)] => SGroup { node: Group::CardPosition(n), span },
            )
        )
    }

    pub(crate) fn kw_table(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn owner(input: Node) -> Result<SOwner> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [player_expr(n)] => Owner::Player(n),
            [player_collection(n)] => Owner::PlayerCollection(n),
            [team_expr(n)] => Owner::Team(n),
            [team_collection(n)] => Owner::TeamCollection(n),
            [kw_table(_)] => Owner::Table,
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
            [group(n), kw_of(_), owner(s)] => CardSet::GroupOwner(n, s),
        );

        Ok(
            SCardSet {
                node: node,
                span
            }
        )       
    }

    pub(crate) fn card_set(input: Node) -> Result<SCardSet> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [group_of_owner(n)] => n,
                [group(n)] => SCardSet { node: CardSet::Group(n), span },
            )
        )
    }

    pub(crate) fn collection(input: Node) -> Result<SCollection> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [int_collection(n)] => SCollection {node: Collection::IntCollection(n), span},
                [string_collection(n)] => SCollection {node: Collection::StringCollection(n), span},
                [player_collection(n)] => SCollection {node: Collection::PlayerCollection(n), span},
                [team_collection(n)] => SCollection {node: Collection::TeamCollection(n), span},
                [location_collection(n)] => SCollection {node: Collection::LocationCollection(n), span},
                [card_set(n)] => SCollection {node: Collection::CardSet(Box::new(n)), span},
            )
        )
    }

    pub(crate) fn location_collection(input: Node) -> Result<SLocationCollection> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
                [location(ids)..] => LocationCollection { locations: ids.collect() },
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
                [team_expr(teams)..] => STeamCollection { node: TeamCollection::Literal(teams.collect()), span },
            )
        )
    }

    pub(crate) fn other_teams(input: Node) -> Result<()> {
        Ok(())
    }

    pub(crate) fn team_collection(input: Node) -> Result<STeamCollection> {
        let span = OwnedSpan::from(input.as_span());
        Ok(
            match_nodes!(input.into_children();
                [team_expr_collection(n)] => n,
                [other_teams(_)] => sruntime_team_collection(RuntimeTeamCollection::OtherTeams, span),
            )
        )
    }

    pub(crate) fn game_rule(input: Node) -> Result<SGameRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [setup_rule(s)] => GameRule::SetUp(s),
            [action_rule(s)] => GameRule::Action(s),
            [scoring_rule(s)] => GameRule::Scoring(s),
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

    pub(crate) fn team_name_with_player_collection(input: Node) -> Result<(SID, SPlayerCollection)> {
        Ok(
            match_nodes!(input.into_children();
                [teamname(teamname), kw_with(_), player_collection(p)] => (teamname, p),
            )
        )
    }

    pub(crate) fn create_player(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_player(_), player_name_list(p)] => SetUpRule::CreatePlayer(p),
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
            [kw_team(_), team_name_with_player_collection(twps)..] => SetUpRule::CreateTeams(twps.collect()),
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
            [kw_turnorder(_), player_collection(p), kw_random(_)] => SetUpRule::CreateTurnorderRandom(p),
            [kw_turnorder(_), player_collection(p)] => SetUpRule::CreateTurnorder(p),
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
        let node = match_nodes!(input.into_children();
            [kw_location(_), location_list(l), kw_on(_), owner(o)] => SetUpRule::CreateLocation(l, o),
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

    pub(crate) fn create_card(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_card(_), kw_on(_), location(location), types(t)] => SetUpRule::CreateCardOnLocation(location, t),
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
            [kw_token(_), int_expr(i), token(token), kw_on(_), location(location)] => SetUpRule::CreateTokenOnLocation(i, token, location),
        );

        Ok(
            SSetUpRule {
                node: node,
                span
            }
        )
    }

    pub(crate) fn create_precedence(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_precedence(_), precedence(precedence), kw_on(_), key(key), values(vs)] => {
                let key_value: Vec<(SID, SID)> = vs.into_iter().map(|v| (key.clone(), v)).collect();
                SetUpRule::CreatePrecedence(precedence, key_value)
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
            [kw_combo(_), combo(combo), kw_where(_), filter_expr(f)] => SetUpRule::CreateCombo(combo, f),
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
        let node = match_nodes!(input.into_children();
            [kw_memory(_), memory(memory), memory_type(mt), kw_on(_), owner(o)] => SetUpRule::CreateMemoryWithMemoryType(memory, mt, o),
            [kw_memory(_), memory(memory), kw_on(_), owner(o)] => SetUpRule::CreateMemory(memory, o),
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

    pub(crate) fn create_pointmap(input: Node) -> Result<SSetUpRule> {
        let span = OwnedSpan::from(input.as_span());
        let node = match_nodes!(input.into_children();
            [kw_points(_), pointmap(pointmap), kw_on(_), key(key), value_int_list(vis)] => {
                let key_value_int: Vec<(SID, SID, SIntExpr)> = vis.into_iter().map(|(v, i)| (key.clone(), v, i)).collect();
                SetUpRule::CreatePointMap(pointmap, key_value_int)
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
                [move_action(a)] => SActionRule { node: ActionRule::Move(a), span: span },
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
            [kw_flip(_), card_set(c), kw_to(_), status(s)] => ActionRule::FlipAction(c, s),
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
            [kw_shuffle(_), card_set(c)] => ActionRule::ShuffleAction(c),
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
        let node = match_nodes!(input.into_children();
            [int_expr(i)] => MemoryType::Int(i),
            [string_expr(s)] => MemoryType::String(s),
            [collection(c)] => MemoryType::Collection(c),
            [card_set(c)] => MemoryType::CardSet(c),
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
            [kw_set(_), players(p), kw_out(_), kw_of(_), kw_stage(_)] => ActionRule::PlayerOutOfStageAction(p),
            [kw_set(_), players(p), kw_out(_), kw_of(_), kw_game(_), kw_successful(_)] => ActionRule::PlayerOutOfGameSuccAction(p),
            [kw_set(_), players(p), kw_out(_), kw_of(_), kw_game(_), kw_fail(_)] => ActionRule::PlayerOutOfGameFailAction(p),
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
            [memory(memory), kw_is(_), memory_type(m)] => ActionRule::SetMemory(memory, m),
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
            [kw_reset(_), memory(memory)] => ActionRule::ResetMemory(memory),
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
            [kw_cycle(_), kw_to(_), player_expr(p)] => ActionRule::CycleAction(p),
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
            [kw_bid(_), quantity(q), kw_on(_), memory(memory)] => ActionRule::BidMemoryAction(memory, q),
            [kw_bid(_), quantity(q)] => ActionRule::BidAction(q),
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
            [kw_stage(_)] => EndType::Stage,
            [kw_game(_), kw_with(_), kw_winner(_), players(p)] => EndType::GameWithWinner(p),

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
            [kw_end(_), end_type(e)] => ActionRule::EndAction(e),
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
            [card_position(c)] => DemandType::CardPosition(c),
            [string_expr(s)] => DemandType::String(s),
            [int_expr(i)] => DemandType::Int(i),
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
            [kw_demand(_), demand_type(d)] => ActionRule::DemandAction(d),
            [kw_demand(_), demand_type(d), kw_as(_), memory(memory)] => ActionRule::DemandMemoryAction(d, memory),
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
            [quantity(q), kw_from(_), card_set(c1), status(s), kw_to(_), card_set(c2)] => MoveCardSet::MoveQuantity(q, c1, s, c2),
            [card_set(c1), status(s), kw_to(_), card_set(c2)] => MoveCardSet::Move(c1, s, c2),
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
            [kw_move(_), card_set_to_card_set(m)] => ClassicMove::MoveCardSet(m),
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
            [kw_deal(_), card_set_to_card_set(m)] => DealMove::MoveCardSet(m),
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
            [kw_exchange(_), card_set_to_card_set(m)] => ExchangeMove::MoveCardSet(m),
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
            [groupable(g)] => TokenLocExpr::Groupable(g),
            [groupable(g), kw_of(_), players(p)] => TokenLocExpr::GroupablePlayers(g, p),
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
            [kw_place(_), quantity(q), token(token), kw_from(_), token_loc(c1), kw_to(_), token_loc(c2)] => TokenMove::PlaceQuantity(q, token, c1, c2),
            [kw_place(_), token(token), kw_from(_), token_loc(c1), kw_to(_), token_loc(c2)] => TokenMove::Place(token, c1, c2),
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
            [classic_move(c)] => MoveType::Classic(c),
            [deal_move(d)] => MoveType::Deal(d),
            [exchange_move(e)] => MoveType::Exchange(e),
            [token_move(t)] => MoveType::Place(t),
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
            [score_rule(s)] => ScoringRule::ScoreRule(s),
            [winner_rule(w)] => ScoringRule::WinnerRule(w),
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
            [memory(m)] => WinnerType::Memory(m),
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
            [kw_score(_), int_expr(n), kw_to(_), memory(memory), kw_of(_), players(o)] => ScoreRule::ScoreMemory(n, memory, o),
            [kw_score(_), int_expr(n), kw_of(_), players(o)] => ScoreRule::Score(n, o),
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
            [kw_winner(_), kw_is(_), extrema(e), winner_type(w)] => WinnerRule::WinnerWith(e, w),
            [kw_winner(_), kw_is(_), players(p)] => WinnerRule::Winner(p),
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
pub(crate) fn saggregate_player(aggr: AggregatePlayer, span: OwnedSpan) -> SPlayerExpr {
    SPlayerExpr {
        node: PlayerExpr::Aggregate(        
            SAggregatePlayer {
                node: aggr,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn saggregate_filter(aggr: AggregateFilter, span: OwnedSpan) -> SFilterExpr {
    SFilterExpr {
        node: FilterExpr::Aggregate(        
            SAggregateFilter {
                node: aggr,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn saggregate_player_collection(aggr: AggregatePlayerCollection, span: OwnedSpan) -> SPlayerCollection {
    SPlayerCollection {
        node: PlayerCollection::Aggregate(        
            SAggregatePlayerCollection {
                node: aggr,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn sruntime_player_collection(runt: RuntimePlayerCollection, span: OwnedSpan) -> SPlayerCollection {
    SPlayerCollection {
        node: PlayerCollection::Runtime(        
            SRuntimePlayerCollection {
                node: runt,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn sruntime_team_collection(runt: RuntimeTeamCollection, span: OwnedSpan) -> STeamCollection {
    STeamCollection {
        node: TeamCollection::Runtime(        
            SRuntimeTeamCollection {
                node: runt,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn saggregate_team(aggr: AggregateTeam, span: OwnedSpan) -> STeamExpr {
    STeamExpr {
        node: TeamExpr::Aggregate(        
            SAggregateTeam {
                node: aggr,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn sruntime_player(runt: RuntimePlayer, span: OwnedSpan) -> SPlayerExpr {
    SPlayerExpr {
        node: PlayerExpr::Runtime(        
            SRuntimePlayer {
                node: runt,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn squery_card_position(quer: QueryCardPosition, span: OwnedSpan) -> SCardPosition {
    SCardPosition {
        node: CardPosition::Query(
            SQueryCardPosition {
                node: quer,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn saggregate_card_position(aggr: AggregateCardPosition, span: OwnedSpan) -> SCardPosition {
    SCardPosition {
        node: CardPosition::Aggregate(
            SAggregateCardPosition {
                node: aggr,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn squery_int(quer: QueryInt, span: OwnedSpan) -> SIntExpr {
    SIntExpr {
        node: IntExpr::Query(
            SQueryInt {
                node: quer,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn saggregate_int(aggr: AggregateInt, span: OwnedSpan) -> SIntExpr {
    SIntExpr {
        node: IntExpr::Aggregate(
            SAggregateInt {
                node: aggr,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn sruntime_int(runt: RuntimeInt, span: OwnedSpan) -> SIntExpr {
    SIntExpr {
        node: IntExpr::Runtime(        
            SRuntimeInt {
                node: runt,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn squery_string(quer: QueryString, span: OwnedSpan) -> SStringExpr {
    SStringExpr {
        node: StringExpr::Query(
            SQueryString {
                node: quer,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn saggregate_compare_bool(cmp: CompareBool, span: OwnedSpan) -> SBoolExpr {
    SBoolExpr {
        node: BoolExpr::Aggregate(
            SAggregateBool {
                node: AggregateBool::Compare(
                    SCompareBool {
                        node: cmp,
                        span: span.clone()
                    }
                ),
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}

pub(crate) fn saggregate_bool(aggr: AggregateBool, span: OwnedSpan) -> SBoolExpr {
    SBoolExpr {
        node: BoolExpr::Aggregate(
            SAggregateBool {
                node: aggr,
                span: span.clone()
            }
        ),
        span: span.clone()
    }
}
