use core::fmt;
use std::fmt::format;

use crate::ast::*;

impl fmt::Display for BinCompare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinCompare::Eq  => "==",
            BinCompare::Neq => "!=",
        };
        f.write_str(s)
    }
}

impl fmt::Display for LogicBinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogicBinOp::And => "and",
            LogicBinOp::Or  => "or",
        };
        f.write_str(s)
    }
}

impl fmt::Display for IntOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IntOp::Plus  => "+",
            IntOp::Minus => "-",
            IntOp::Mul   => "*",
            IntOp::Div   => "/",
            IntOp::Mod   => "mod",
        };
        f.write_str(s)
    }
}

impl fmt::Display for IntCompare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IntCompare::Eq => "==",
            IntCompare::Neq => "!=",
            IntCompare::Gt => ">",
            IntCompare::Lt => "<",
            IntCompare::Ge => ">=",
            IntCompare::Le => "<=",
        };
        f.write_str(s)
    }
}

impl fmt::Display for Extrema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Extrema::Min => "min",
            Extrema::Max => "max",
        };
        f.write_str(s)
    }
}

impl fmt::Display for OutOf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OutOf::CurrentStage => "stage",
            OutOf::Stage { name: stage } => stage,
            OutOf::Game => "game",
        };
        f.write_str(s)
    }
}

impl fmt::Display for Groupable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Groupable::Location { name: location } => location,
            Groupable::LocationCollection { location_collection: location_collection } => &format!("{}", location_collection),
        };
        f.write_str(s)
    }
}

impl fmt::Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Owner::Player { player: player } => &format!("{}", player),
            Owner::PlayerCollection { player_collection: player_collection } => &format!("{}", player_collection),
            Owner::Team { team: team } => &format!("{}", team),
            Owner::TeamCollection { team_collection: team_collection} => &format!("{}", team_collection),
            Owner::Table => "table",
            // Owner::Memory { memory } => &format!("&{}", memory),
          };
        f.write_str(s)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Quantity::Int { int: int} => &format!("{}", int),
            Quantity::Quantifier { qunatifier: quant }  => &format!("{}", quant),
            Quantity::IntRange { int_range: int_range } => &format!("{}", int_range),
        };
        f.write_str(s)
    }
}
impl fmt::Display for IntRangeOperator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IntRangeOperator::And => "and",
            IntRangeOperator::Or => "or",
        };
        f.write_str(s)
    }
}


impl fmt::Display for IntRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start as "start.0 start.1"
        let start = format!("{} {}", self.start.0, self.start.1);

        // Map op_int into strings like "op cmp int" and join with spaces
        let rest = self.op_int
            .iter()
            .map(|(op, cmp, int)| format!("{} {} {}", op, cmp, int))
            .collect::<Vec<_>>()
            .join(" ");

        // Combine start + rest
        if rest.is_empty() {
            write!(f, "{}", start)
        } else {
            write!(f, "{} {}", start, rest)
        }
    }
}

impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Quantifier::All => "all",
            Quantifier::Any => "any",
        };
        f.write_str(s)
    }
}

impl fmt::Display for EndCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EndCondition::UntilBool { bool_expr: bool_expr } => &format!("until {}", bool_expr),
            EndCondition::UntilBoolRep { bool_expr: bool_expr, logic: logic_bin_op, reps: repititions} => &format!("until {} {} {}", bool_expr, logic_bin_op, repititions),
            EndCondition::UntilRep { reps: repititions } => &format!("{}", repititions),
            EndCondition::UntilEnd => "until end",
        };
        f.write_str(s)
    }
}

impl fmt::Display for Repititions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = &format!("{} times", self.times);
        f.write_str(s)
    }
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MemoryType::Int { int: int_expr } => &format!("{}", int_expr),
            MemoryType::String { string: string_expr } => &format!("{}", string_expr),
            MemoryType::StringCollection { strings } => &format!("{}", strings),
            MemoryType::IntCollection { ints } => &format!("{}", ints),
            MemoryType::PlayerCollection { players } => &format!("{}", players),
            MemoryType::TeamCollection { teams } => &format!("{}", teams),
            MemoryType::LocationCollection { locations } => &format!("{}", locations),
            MemoryType::CardSet { card_set } => &format!("cards {}", card_set),
        };
        f.write_str(s)
    }
}

impl fmt::Display for Players {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Players::Player { player } => &format!("{}", player),
            Players::PlayerCollection { player_collection} => &format!("{}", player_collection),
        };
        f.write_str(s)
    }
}

impl fmt::Display for EndType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EndType::Turn => "turn",
            EndType::Stage => "stage",
            EndType::GameWithWinner { players } => 
              &format!("game with winner {}", players),
        };
        f.write_str(s)
    }
}

impl fmt::Display for DemandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DemandType::CardPosition { card_position } => &format!("{}", card_position),
            DemandType::String { string } => &format!("{}", string),
            DemandType::Int { int } => &format!("{}", int),
        };
        f.write_str(s)
    }
}

impl fmt::Display for Types {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first_entry = true;

        for (key, values) in &self.types {
            if !first_entry {
                write!(f, " ")?; // separate entries
            }

            // Join values with ", "
            let vs = values.join(", ");

            if first_entry {
                write!(f, "{} ({})", key, vs)?;
                first_entry = false;
            } else {
                write!(f, "for {} ({})", key, vs)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for RuntimePlayer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        RuntimePlayer::Current => "current",
        RuntimePlayer::Next => "next",
        RuntimePlayer::Previous => "previous",
        RuntimePlayer::Competitor => "competitor",
      };
      f.write_str(s)
  }
}

impl fmt::Display for QueryPlayer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        QueryPlayer::Turnorder { int: int_expr } => &format!("turnorder[{}]", int_expr),
      };
      f.write_str(s)
  }
}

impl fmt::Display for AggregatePlayer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        AggregatePlayer::OwnerOfCardPostion { card_position: card_position } => &format!("owner of {}", card_position),
        AggregatePlayer::OwnerOfMemory { extrema, memory } => &format!("owner of {} {}", extrema, memory),
      };
      f.write_str(s)
  }
}

impl fmt::Display for PlayerExpr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        PlayerExpr::Literal{name} => name,
        PlayerExpr::Runtime { runtime } => &format!("{}", runtime),
        PlayerExpr::Aggregate { aggregate } => &format!("{}", aggregate),
        PlayerExpr::Query {  query } => &format!("{}", query),
      };
      f.write_str(s)
  }
}

impl fmt::Display for QueryInt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        QueryInt::IntCollectionAt { int_collection, int_expr} => &format!("{}[{}]", int_collection, int_expr),
      };
      f.write_str(s)
  }
}

impl fmt::Display for AggregateInt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        AggregateInt::SizeOf { collection } => &format!("size({})", collection),
        AggregateInt::SumOfIntCollection { int_collection} => &format!("sum({})", int_collection),
        AggregateInt::SumOfCardSet { card_set, pointmap} => &format!("sum of {} using {}", card_set, pointmap),
        AggregateInt::ExtremaCardset { extrema, card_set, pointmap} => &format!("{} of {} using {}", extrema, card_set, pointmap),
        AggregateInt::ExtremaIntCollection { extrema, int_collection } => &format!("{}({})", extrema, int_collection),
      };
      f.write_str(s)
  }
}

impl fmt::Display for RuntimeInt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        RuntimeInt::StageRoundCounter => "stageroundcounter",
        RuntimeInt::PlayRoundCounter => "playroundcounter",
          };
      f.write_str(s)
  }
}

impl fmt::Display for IntExpr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        IntExpr::Literal { int } => &format!("{}", int),
        IntExpr::Binary { int: int_expr, op: int_op, int1: int_expr1} => 
          &format!("({} {} {})", int_expr, int_op, int_expr1),
        IntExpr::Query {query: query_int} => &format!("{}", query_int),
        IntExpr::Aggregate{aggregate: aggregate_int} => &format!("{}", aggregate_int),
        IntExpr::Runtime{runtime: runtime_int} => &format!("{}", runtime_int),
        IntExpr::Memory { memory } => {
          &format!("&{}", memory)
        },
      };
      f.write_str(s)
  }
}

impl fmt::Display for QueryString {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        QueryString::KeyOf{key, card_position} => &format!("{} of {}", key, card_position),
        QueryString::StringCollectionAt{string_collection, int_expr} => &format!("{}[{}]", string_collection, int_expr),
      };
      f.write_str(s)
  }
}

impl fmt::Display for StringExpr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        StringExpr::Literal{value} => &format!("\"{}\"", value),
        StringExpr::Query { query: query_string} => &format!("{}", query_string),
        StringExpr::Memory { memory } => &format!("&{}", memory),
      };
      f.write_str(s)
  }
}

impl fmt::Display for CardSetCompare {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        Self::Eq => "c==",
        Self::Neq => "c!=",
      };
      f.write_str(s)
  }
}

impl fmt::Display for StringCompare {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        Self::Eq => "s==",
        Self::Neq => "s!=",
      };
      f.write_str(s)
  }
}

impl fmt::Display for PlayerCompare {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        Self::Eq => "p==",
        Self::Neq => "p!=",
      };
      f.write_str(s)
  }
}

impl fmt::Display for TeamCompare {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        Self::Eq => "t==",
        Self::Neq => "t!=",
      };
      f.write_str(s)
  }
}

impl fmt::Display for UnaryOp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        Self::Not => "not",
      };
      f.write_str(s)
  }
}

impl fmt::Display for CompareBool {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        CompareBool::Int { int: int_expr, cmp: int_compare, int1: int_expr1 } => 
          &format!("{} {} {}", int_expr, int_compare, int_expr1),
        CompareBool::CardSet { card_set, cmp: card_set_compare, card_set1 } => 
          &format!("{} {} {}", card_set, card_set_compare, card_set1),
        CompareBool::String { string: string_expr, cmp: string_compare, string1: string_expr1} =>
          &format!("{} {} {}", string_expr, string_compare, string_expr1),
        CompareBool::Player { player: player_expr, cmp: player_compare, player1: player_expr1} => 
          &format!("{} {} {}", player_expr, player_compare, player_expr1),
        CompareBool::Team { team: team_expr, cmp: team_compare, team1: team_expr1} => 
          &format!("{} {} {}", team_expr, team_compare, team_expr1),
      };
      f.write_str(s)
  }
}

impl fmt::Display for AggregateBool {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        AggregateBool::Compare { cmp_bool: compare_bool} => 
          &format!("{}", compare_bool),
        AggregateBool::CardSetEmpty{card_set} => 
          &format!("{} is empty", card_set),
        AggregateBool::CardSetNotEmpty{card_set} => 
          &format!("{} is not empty", card_set),
        AggregateBool::OutOfPlayer{players, out_of} => 
          &format!("{} out of {}", players, out_of),        
      };
      f.write_str(s)
  }
}

impl fmt::Display for BoolOp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        BoolOp::And => "and",
        BoolOp::Or => "or",
      };
      f.write_str(s)
  }
}

impl fmt::Display for BoolExpr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        BoolExpr::Binary{bool_expr, op: bool_op, bool_expr1} =>
          &format!("( {} {} {} )", bool_expr, bool_op, bool_expr1),
        BoolExpr::Unary { op: unary_op, bool_expr} =>
          &format!("{} {}", unary_op, bool_expr),
        BoolExpr::Aggregate{ aggregate: aggregate_bool} =>
          &format!("{}", aggregate_bool),
      };
      f.write_str(s)
  }
}

impl fmt::Display for AggregateTeam {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        AggregateTeam::TeamOf { player: player_expr } => 
          &format!("team of {}", player_expr),        
      };
      f.write_str(s)
  }
}

impl fmt::Display for TeamExpr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        TeamExpr::Literal { name } => name,
        TeamExpr::Aggregate {aggregate: aggregate_team} => 
        &format!("{}", aggregate_team),
      };
      f.write_str(s)
  }
}

impl fmt::Display for QueryCardPosition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        QueryCardPosition::At { location, int_expr } => 
          &format!("{}[{}]", location, int_expr),
        QueryCardPosition::Top { location } => 
          &format!("top({})", location),
        QueryCardPosition::Bottom { location } => 
          &format!("bottom({})", location),
        };
      f.write_str(s)
  }
}

impl fmt::Display for AggregateCardPosition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        AggregateCardPosition::ExtremaPointMap { extrema, card_set, pointmap} => 
          &format!("{} of {} using points {}", extrema, card_set, pointmap),
        AggregateCardPosition::ExtremaPrecedence { extrema, card_set, precedence} => 
          &format!("{} of {} using {}", extrema, card_set, precedence),
      };
      f.write_str(s)
  }
}

impl fmt::Display for CardPosition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        CardPosition::Query { query: query_card_position } => 
          &format!("{}", query_card_position),
        CardPosition::Aggregate { aggregate: aggregate_card_position } => 
          &format!("{}", aggregate_card_position),
      };
      f.write_str(s)
  }
}

impl fmt::Display for CardSet {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        CardSet::Group { group } => &format!("{}", group),
        CardSet::GroupOwner { group, owner } => &format!("{} of {}", group, owner),
        CardSet::Memory { memory } => &format!("&{}",memory),
      };
      f.write_str(s)
  }
}

impl fmt::Display for Group {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        Group::Groupable { groupable } => &format!("{}", groupable),
        Group::Where{groupable, filter: filter_expr} => 
          &format!("{} where {}", groupable, filter_expr),
        Group::NotCombo { combo, groupable} => 
          &format!("not {} in {}", combo, groupable),
        Group::Combo { combo, groupable } => 
          &format!("{} in {}", combo, groupable),
        Group::CardPosition { card_position} => 
          &format!("{}", card_position),
      };
      f.write_str(s)
  }
}

impl fmt::Display for AggregateFilter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        AggregateFilter::Size { cmp: int_compare, int_expr } => 
          &format!("size {} {}", int_compare, int_expr),
        AggregateFilter::Same{key} =>
          &format!("same {}", key),
        AggregateFilter::Distinct{key} => 
          &format!("distinct {}", key),
        AggregateFilter::Adjacent{key, precedence} =>
          &format!("adjacent {} using {}", key, precedence),
        AggregateFilter::Higher { key, value, precedence} => 
          &format!("{} higher than {} using {}", key, value, precedence),
        AggregateFilter::Lower { key, value, precedence} => 
          &format!("{} lower than {} using {}", key, value, precedence),
        AggregateFilter::KeyString { key, cmp: string_compare, string: string_expr} => 
          &format!("{} {} {}", key, string_compare, string_expr),
        AggregateFilter::Combo { combo } => 
          &format!("{}", combo),
        AggregateFilter::NotCombo { combo } =>
          &format!("not {}", combo),
      };
      f.write_str(s)
  }
}

impl fmt::Display for FilterOp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        FilterOp::And => "and",
        FilterOp::Or => "or",
      };
      f.write_str(s)
  }
}

impl fmt::Display for FilterExpr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        FilterExpr::Aggregate { aggregate: aggregate_filter } => 
          &format!("{}", aggregate_filter),
        FilterExpr::Binary { filter: filter_expr, op: filter_op, filter1: filter_expr1 } =>
          &format!("({} {} {})", filter_expr, filter_op, filter_expr1),
      };
      f.write_str(s)
  }
}

impl fmt::Display for Collection {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        Collection::IntCollection { int: int_collection} => 
          &format!("{}", int_collection),
        Collection::StringCollection { string: string_collection} => 
          &format!("{}", string_collection),
        Collection::LocationCollection { location: location_collection} =>
          &format!("{}", location_collection),
        Collection::PlayerCollection {player: player_collection} =>
          &format!("{}", player_collection),
        Collection::TeamCollection{ team: team_collection} =>
          &format!("{}", team_collection),
        Collection::CardSet{card_set} =>
          &format!("cards {}", card_set),
        // Collection::Memory { memory } => 
        //   &format!("&{}",memory),
      };
      f.write_str(s)
  }
}

impl fmt::Display for IntCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
        IntCollection::Literal { ints} => {
          let s = ints
              .iter()
              .map(|i| i.to_string()) // convert each int to string
              .collect::<Vec<_>>()    // collect into Vec<String>
              .join(", ");             // join with commas

          write!(f, "( {} )", s)
        },
        IntCollection::Memory { memory } => write!(f, "&{}", memory),
      }
    }
}

impl fmt::Display for LocationCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
        LocationCollection::Literal { locations} => {
          let s = locations
              .iter()
              .map(|i| i.to_string()) // convert each int to string
              .collect::<Vec<_>>()    // collect into Vec<String>
              .join(", ");             // join with commas

          write!(f, "( {} )", s)
        },
        LocationCollection::Memory { memory } => write!(f, "&{}", memory),
      }
    }
}
impl fmt::Display for StringCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
          StringCollection::Literal { strings} => {
            let s = strings
                .iter()
                .map(|i| i.to_string()) // convert each int to string
                .collect::<Vec<_>>()    // collect into Vec<String>
                .join(", ");             // join with commas

            write!(f, "( {} )", s)
          },
          StringCollection::Memory { memory } => write!(f, "&{}", memory),
      }
    }
}

impl fmt::Display for RuntimePlayerCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RuntimePlayerCollection::PlayersOut => "playersout",
            RuntimePlayerCollection::PlayersIn => "playersin",
            RuntimePlayerCollection::Others => "others",
        };
        f.write_str(s)
    }
}

impl fmt::Display for AggregatePlayerCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AggregatePlayerCollection::Quantifier { quantifier: q} => 
              &format!("{}", q),
        };
        f.write_str(s)
    }
}

impl fmt::Display for PlayerCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PlayerCollection::Literal { players: names} => {
              let s_inner = names
                  .iter()
                  .map(|i| i.to_string()) // convert each int to string
                  .collect::<Vec<_>>()    // collect into Vec<String>
                  .join(", ");             // join with commas

              &format!("( {} )", s_inner)
            },
            PlayerCollection::Aggregate { aggregate: aggregate_player_collection } => 
              &format!("{}", aggregate_player_collection),
            PlayerCollection::Runtime { runtime: runtime_player_collection } =>
              &format!("{}", runtime_player_collection),
            PlayerCollection::Memory { memory } => &format!("&{}", memory),
        };
        f.write_str(s)
    }
}

impl fmt::Display for RuntimeTeamCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RuntimeTeamCollection::OtherTeams => "other teams",
        };
        f.write_str(s)
    }
}

impl fmt::Display for TeamCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TeamCollection::Literal { teams: names} => {
              let s_inner = names
                  .iter()
                  .map(|i| i.to_string()) // convert each int to string
                  .collect::<Vec<_>>()    // collect into Vec<String>
                  .join(", ");             // join with commas

              &format!("( {} )", s_inner)
            },
            TeamCollection::Runtime { runtime: runtime_team_collection} =>
              &format!("{}", runtime_team_collection),
            TeamCollection::Memory { memory } => &format!("&{}", memory),
        };
        f.write_str(s)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
          Status::FaceDown => "face down",
          Status::FaceUp => "face up",
          Status::Private => "private",
        };
        f.write_str(s)
    }
}

impl fmt::Display for SetUpRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SetUpRule::CreatePlayer { players: items} => {
              let s_inner = items
                .iter()
                .map(|i| i.to_string()) // convert each int to string
                .collect::<Vec<_>>()    // collect into Vec<String>
                .join(", ");
              &format!("player {}", s_inner)
            },
            SetUpRule::CreateTeams { teams: items} => {
              let s_inner = items
                .iter()
                .map(|(i, ps)| format!("{} with {}", i, ps)) // convert each int to string
                .collect::<Vec<_>>()    // collect into Vec<String>
                .join(", ");
              &format!("team {}", s_inner)
            },
            SetUpRule::CreateTurnorder { player_collection} => {
              &format!("turnorder {}", player_collection)
            },
            SetUpRule::CreateTurnorderRandom { player_collection} => {
              &format!("turnorder {} random", player_collection)
            },
            SetUpRule::CreateLocation { locations: items, owner} => {
              let s_inner = items
                  .iter()
                  .map(|i| i.to_string()) // convert each int to string
                  .collect::<Vec<_>>()    // collect into Vec<String>
                  .join(", ");
              &format!("location {} on {}", s_inner, owner)
            },
            SetUpRule::CreateCardOnLocation { location, types } => {
              &format!("card on {} : {}", location, types)
            },
            SetUpRule::CreateTokenOnLocation { int: int_expr, token, location} => {
              &format!("token {} {} on {}", int_expr, token, location)
            },
            SetUpRule::CreateCombo { combo, filter: filter_expr} => {              
              &format!("combo {} where {}", combo, filter_expr)
            },
            SetUpRule::CreateMemoryWithMemoryType { memory, memory_type, owner} => {
              &format!("memory {} {} on {}", memory, memory_type, owner)
            },
            SetUpRule::CreateMemory { memory, owner} => {
              &format!("memory {} on {}", memory, owner)
            },
            SetUpRule::CreatePrecedence {precedence: prec, kvs: items} => {
              let s_inner = items
                  .iter()
                  .map(|(k, v)| format!("{} {}", k, v)) // convert each int to string
                  .collect::<Vec<_>>()    // collect into Vec<String>
                  .join(", ");
              &format!("precedence {} ( {} )", prec, s_inner)
            },
            SetUpRule::CreatePointMap { pointmap: point, kvis: items} => {
              let s_inner = items
                  .iter()
                  .map(|(k, v, i)| format!("{} {}: {}", k, v, i)) // convert each int to string
                  .collect::<Vec<_>>()    // collect into Vec<String>
                  .join(", ");
              &format!("points {} ( {} )", point, s_inner)
            },
        };
        f.write_str(s)
    }
}


impl fmt::Display for ActionRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ActionRule::FlipAction { card_set, status} => {
              &format!("flip {} to {}", card_set, status)
            },
            ActionRule::ShuffleAction{card_set} => {
              &format!("shuffle {}", card_set)
            },
            ActionRule::PlayerOutOfStageAction{players} => {
              &format!("set {} out of stage", players)
            },
            ActionRule::PlayerOutOfGameSuccAction{players} => {
              &format!("set {} out of game successful", players)
            },
            ActionRule::PlayerOutOfGameFailAction{players} => {
              &format!("set {} out of game fail", players)
            },
            ActionRule::SetMemory{memory, memory_type} => {
              &format!("{} is {}", memory, memory_type)
            },
            ActionRule::ResetMemory{memory} => {
              &format!("reset {}", memory)
            },
            ActionRule::CycleAction{player: player_expr} => {
              &format!("cycle to {}", player_expr)
            },
            ActionRule::BidAction{quantitiy} => {
              &format!("bid {}", quantitiy)
            },
            ActionRule::BidMemoryAction {memory, quantity, owner} => {
              &format!("bid {} on {} of {}", quantity, memory, owner)
            },
            ActionRule::EndAction{end_type} => {
              &format!("end {}", end_type)
            },
            ActionRule::DemandAction{demand_type} => {
              &format!("demand {}", demand_type)
            },
            ActionRule::DemandMemoryAction{demand_type, memory} => {
              &format!("demand {} as {}", demand_type, memory)
            },
            ActionRule::Move{move_type} => {
              &format!("{}", move_type)
            },
        };
        f.write_str(s)
    }
}

impl fmt::Display for UseMemory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        UseMemory::Memory { memory } => 
          &format!("{}", memory),
        UseMemory::WithOwner { memory, owner } => 
          &format!("{} of {}", memory, owner),
    };
    f.write_str(s)
  }
}

impl fmt::Display for MoveType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        MoveType::Deal { deal: deal_move} => 
          &format!("{}", deal_move),
        MoveType::Exchange { exchange: exchange_move} =>
          &format!("{}", exchange_move),
        MoveType::Classic{classic: classic_move} => 
          &format!("{}", classic_move),
        MoveType::Place{ token: token_move} => 
          &format!("{}", token_move),
    };
    f.write_str(s)
  }
}

impl fmt::Display for MoveCardSet {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        MoveCardSet::Move { from: card_set, status, to: card_set1} => 
          &format!("{} {} to {}", card_set, status, card_set1),
        MoveCardSet::MoveQuantity { quantity, from: card_set, status, to: card_set1} => 
          &format!("{} from {} {} to {}", quantity, card_set, status, card_set1),
    };
    f.write_str(s)
  }
}

impl fmt::Display for ClassicMove {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        ClassicMove::MoveCardSet { move_cs: move_card_set} => 
          &format!("move {}", move_card_set),
    };
    f.write_str(s)
  }
}

impl fmt::Display for DealMove {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        DealMove::MoveCardSet { deal_cs: move_card_set } => 
          &format!("deal {}", move_card_set),
    };
    f.write_str(s)
  }
}

impl fmt::Display for ExchangeMove {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        ExchangeMove::MoveCardSet { exchange_cs: move_card_set} => 
          &format!("exchange {}", move_card_set),
    };
    f.write_str(s)
  }
}

impl fmt::Display for TokenMove {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        TokenMove::Place { token, from_loc: token_loc_expr, to_loc: token_loc_expr1 } => {
          &format!("place {} from {} to {}", token, token_loc_expr, token_loc_expr1)
        },
        TokenMove::PlaceQuantity { quantity, token, from_loc: token_loc_expr, to_loc: token_loc_expr1} => {
          &format!("place {} {} from {} to {}", quantity, token, token_loc_expr, token_loc_expr1)
        },
    };
    f.write_str(s)
  }
}

impl fmt::Display for TokenLocExpr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        TokenLocExpr::Groupable{groupable} => 
          &format!("{}", groupable),
        TokenLocExpr::GroupablePlayers{groupable, players} => 
          &format!("{} of {}", groupable, players),
    };
    f.write_str(s)
  }
}

impl fmt::Display for ScoringRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        ScoringRule::ScoreRule{score_rule} => 
          &format!("{}", score_rule),
        ScoringRule::WinnerRule{winner_rule} => 
          &format!("{}", winner_rule),
    };
    f.write_str(s)
  }
}

impl fmt::Display for ScoreRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        ScoreRule::Score{int: int_expr, players} => 
          &format!("score {} to {}", int_expr, players),
        ScoreRule::ScoreMemory{int: int_expr, memory, players} => 
          &format!("score {} to {} of {}", int_expr, memory, players),
    };
    f.write_str(s)
  }
}

impl fmt::Display for WinnerRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        WinnerRule::Winner{players} => 
            &format!("winner is {}", players),
        WinnerRule::WinnerWith{extrema, winner_type} =>
            &format!("winner is {} {}", extrema, winner_type),
    };
    f.write_str(s)
  }
}

impl fmt::Display for WinnerType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        WinnerType::Score => "score",
        WinnerType::Position => "position",
        WinnerType::Memory {memory} => memory,
    };
    f.write_str(s)
  }
}

impl fmt::Display for GameRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        GameRule::SetUp{setup} =>
          &format!("{}", setup),
        GameRule::Action{action} =>
          &format!("{}", action),
        GameRule::Scoring{scoring} =>
          &format!("{}", scoring),
        
    };
    f.write_str(s)
  }
}

impl fmt::Display for SeqStage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let flows = self.flows
      .iter()
      .map(|f| format!("{}", f)) // convert each int to string
      .collect::<Vec<_>>()    // collect into Vec<String>
      .join("\n");

    let s = &format!("stage {} for {} {} {{\n {} }}\n", self.stage, self.player, self.end_condition, flows);
    f.write_str(s)
  }
}

impl fmt::Display for IfRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let flows = self.flows
      .iter()
      .map(|f| format!("{}", f)) // convert each int to string
      .collect::<Vec<_>>()    // collect into Vec<String>
      .join("\n");

    let s = &format!("if ({}) {{\n {} }}\n", self.condition, flows);
    f.write_str(s)
  }
}

impl fmt::Display for ChoiceRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let flows = self.options
      .iter()
      .map(|f| format!("{}", f)) // convert each int to string
      .collect::<Vec<_>>()    // collect into Vec<String>
      .join("\nor\n");

    let s = &format!("choose {{\n {} }}\n", flows);
    f.write_str(s)
  }
}

impl fmt::Display for OptionalRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let flows = self.flows
      .iter()
      .map(|f| format!("{}", f)) // convert each int to string
      .collect::<Vec<_>>()    // collect into Vec<String>
      .join("\n");

    let s = &format!("optional {{\n {} }}\n", flows);
    f.write_str(s)
  }
}

impl fmt::Display for Case {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        Case::Bool{bool_expr, flows} => {
          let case_flows = flows
                .iter()
                .map(|f| format!("{}", f)) // convert each int to string
                .collect::<Vec<_>>()    // collect into Vec<String>
                .join("\n");

          &format!("case {}:\n {}", bool_expr, case_flows)
        },
        Case::NoBool{flows} => {
          let case_flows = flows
                .iter()
                .map(|f| format!("{}", f)) // convert each int to string
                .collect::<Vec<_>>()    // collect into Vec<String>
                .join("\n");

          &format!("case:\n {}", case_flows)
        },
        // Case::Else{flows} => {
        //   let case_flows = flows
        //         .iter()
        //         .map(|f| format!("{}", f)) // convert each int to string
        //         .collect::<Vec<_>>()    // collect into Vec<String>
        //         .join("\n");

        //   &format!("case else:\n {}", case_flows)
        // },
        
    };
    f.write_str(s)
  }
}

impl fmt::Display for Conditional {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let cases = self.cases
      .iter()
      .map(|c| format!("{}", c)) // convert each int to string
      .collect::<Vec<_>>()    // collect into Vec<String>
      .join("\n");

    let s = &format!("conditional {{\n {} }}\n", cases);
    f.write_str(s)
  }
}

impl fmt::Display for FlowComponent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
        FlowComponent::Stage{stage} =>
          &format!("{}", stage),
        FlowComponent::Rule{game_rule} =>
          &format!("{}", game_rule),
        FlowComponent::IfRule{if_rule} =>
          &format!("{}", if_rule),
        FlowComponent::ChoiceRule{choice_rule} =>
          &format!("{}", choice_rule),
        FlowComponent::OptionalRule{optional_rule} =>
          &format!("{}", optional_rule),
        FlowComponent::Conditional{conditional} =>
          &format!("{}", conditional),
    };
    f.write_str(s)
  }
}

impl fmt::Display for Game {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let fs = self.flows
      .iter()
      .map(|f| format!("{}", f)) // convert each int to string
      .collect::<Vec<_>>()    // collect into Vec<String>
      .join("\n");

    let s = &format!("{}", fs);
    f.write_str(s)
  }
}
