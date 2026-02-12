use dashmap::DashMap;
use front_end::{ast::ast::{Game, SGame}, get_all_snippets, parser::Rule, symbols::{GameType, SymbolVisitor, Var}, walker::Walker};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse, InsertTextFormat, Position};
use pest_consume::Error;
use pest::error;
// use get_snippet_map;
use std::collections::HashMap;
use std::sync::LazyLock;

static SNIPPET_LOOKUP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    get_all_snippets()
});

// Inside your completion function
pub fn get_completions(err: Error<Rule>, symbol_table: &DashMap<GameType, Vec<String>>) -> Option<CompletionResponse> {
    match err.variant {
        error::ErrorVariant::ParsingError { positives, negatives } => {
          let mut completion_response = vec![];
            for rule in positives.iter() {
              if let Some(v) = variable_completion(rule, symbol_table) {
                completion_response.extend(v);
              }

              let rule_name = format!("{:?}", rule);
              
              let item = SNIPPET_LOOKUP.get(rule_name.as_str()).map(|body| {
                  CompletionItem {
                      label: rule_name.replace("kw_", "").replace("_", " "),
                      insert_text: Some(body.to_string()),
                      insert_text_format: Some(InsertTextFormat::SNIPPET),
                      kind: Some(CompletionItemKind::SNIPPET),
                      ..Default::default()
                  }
              });

              match item {
                Some(completion) => {
                  completion_response.push(completion);
                },
                None => {},
              }
            }

            if completion_response.is_empty() {
              return None
            }

            Some(CompletionResponse::Array(completion_response))
      },
      _ => {None},
  }
}

fn variable_completion(rule: &Rule, symbol_table: &DashMap<GameType, Vec<String>>) -> Option<Vec<CompletionItem>> {
  match rule {
    Rule::playername => {
      make_variable_snippet(GameType::Player, "Player", symbol_table)
    },
    Rule::combo => {
      make_variable_snippet(GameType::Combo, "Combo", symbol_table)
    },
    Rule::teamname => {
      make_variable_snippet(GameType::Team, "Team", symbol_table)
    },
    Rule::precedence => {
      make_variable_snippet(GameType::Precedence, "Precedence", symbol_table)
    },
    Rule::pointmap => {
      make_variable_snippet(GameType::PointMap, "PointMap", symbol_table)
    },
    Rule::key => {
      make_variable_snippet(GameType::Key, "Key", symbol_table)
    },
    Rule::value => {
      make_variable_snippet(GameType::Value, "Value", symbol_table)
    },
    Rule::memory => {
      make_variable_snippet(GameType::Memory, "Memory", symbol_table)
    },
    Rule::token => {
      make_variable_snippet(GameType::Token, "Token", symbol_table)
    },
    Rule::location => {
      make_variable_snippet(GameType::Location, "Location", symbol_table)
    },
    Rule::stage => {
      make_variable_snippet(GameType::Stage, "Stage", symbol_table)
    },
    _ => None,
  }
}

fn make_variable_snippet(ty: GameType, detail: &str, symbol_table: &DashMap<GameType, Vec<String>>) -> Option<Vec<CompletionItem>> {
  if let Some(names) = symbol_table.get(&ty) {
    let mut items = vec![];

    for name in names.iter() {
      items.push(make_snippet(name, name, detail));
    }

    return Some(items)
  }

  None
} 

// Helper to build the Snippet item
fn make_snippet(label: &str, template: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.into(),
        detail: Some(detail.into()),
        insert_text: Some(template.into()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        kind: Some(CompletionItemKind::SNIPPET),
        ..Default::default()
    }
}

// fn complete_var<'a>(rule: &Rule, ctx: &'a Vec<(Var, GameType)>) -> Vec<(&'a str, &'a str)> {
//     match rule {
//       Rule::stage => {
//         helper_complete_var(ctx, GameType::Stage)
//       },
//       Rule::teamname => {
//         helper_complete_var(ctx, GameType::Team)
//       },
//       Rule::playername => {
//         helper_complete_var(ctx, GameType::Player)
//       },
//       Rule::location => {
//         helper_complete_var(ctx, GameType::Location)
//       },
//       Rule::token => {
//         helper_complete_var(ctx, GameType::Token)
//       },
//       Rule::precedence => {
//         helper_complete_var(ctx, GameType::Precedence)
//       },
//       Rule::combo => {
//         helper_complete_var(ctx, GameType::Combo)
//       },
//       Rule::memory => {
//         helper_complete_var(ctx, GameType::Memory)
//       },
//       Rule::pointmap => {
//         helper_complete_var(ctx, GameType::PointMap)
//       },
//       Rule::key => {
//         helper_complete_var(ctx, GameType::Key)
//       },
//       Rule::value => {
//         helper_complete_var(ctx, GameType::Value)
//       },
//       _ => vec![]
//     }
// }

// fn helper_complete_var(ctx: &Vec<(Var, GameType)>, game_type: GameType) -> Vec<(&str, &str)> {
//   ctx
//     .iter()
//     .filter(|(_, ty)| *ty == game_type)
//     .map(|(v, _)| (v.id.as_str(), "Some detail"))
//     .collect::<Vec<(&str, &str)>>()
// }

// fn complete_kw_only_rule(rule: &Rule) -> Vec<(&str, &str)> {
//   match rule {
//       Rule::logical_compare => {
//         vec![
//           ("and", "Some detail"),
//           ("or", "Some detail"),
//         ]
//       },
//       Rule::int_op => {
//         vec![
//           ("+", "Some detail"),
//           ("-", "Some detail"),
//           ("*", "Some detail"),
//           ("/", "Some detail"),
//           ("mod", "Some detail"),
//         ]
//       },
//       Rule::int_compare => {
//         vec![
//           ("==", "Some detail"),
//           ("!=", "Some detail"),
//           ("<", "Some detail"),
//           (">", "Some detail"),
//           ("<=", "Some detail"),
//           (">=", "Some detail"),
//         ]
//       },
//       Rule::extrema => {
//         vec![
//           ("min", "Some detail"),
//           ("max", "Some detail"),
//           ("lowest", "Some detail"),
//           ("highest", "Some detail"),
//         ]
//       },
//       Rule::card_set_compare => {
//         vec![
//           ("==", "Some detail"),
//           ("!=", "Some detail"),
//         ]
//       },
//       Rule::player_expr_compare => {
//         vec![
//           ("==", "Some detail"),
//           ("!=", "Some detail"),
//         ]
//       },
//       Rule::team_expr_compare => {
//         vec![
//           ("==", "Some detail"),
//           ("!=", "Some detail"),
//         ]
//       },
//       Rule::string_expr_compare => {
//         vec![
//           ("==", "Some detail"),
//           ("!=", "Some detail"),
//         ]
//       },
//       Rule::bool_op => {
//         vec![
//           ("and", "Some detail"),
//           ("or", "Some detail"),
//         ]
//       },
//       Rule::unary_op => {
//         vec![
//           ("not", "Some detail"),
//         ]
//       },
//       Rule::face_up => {
//         vec![
//           ("face up", "Some detail"),
//         ]
//       },
//       Rule::face_down => {
//         vec![
//           ("face down", "Some detail"),
//         ]
//       },
//       Rule::private => {
//         vec![
//           ("private", "Some detail"),
//         ]
//       },
//       Rule::status => {
//         vec![
//           ("face up", "Some detail"),
//           ("face down", "Some detail"),
//           ("private", "Some detail"),
//         ]
//       },
//       Rule::quantifier => {
//         vec![
//           ("all", "Some detail"),
//           ("any", "Some detail"),
//         ]
//       }
//       Rule::other_teams => {
//         vec![
//           ("other teams", "Some detail"),
//         ]
//       },
//       Rule::until_end => {
//         vec![
//           ("until end", "Some detail"),
//         ]
//       },
//       _ => vec![]
//   }
// }

// fn get_static_key_word_completion(rule: &Rule) -> Vec<(&str, &str)> {
//   match rule {
//     Rule::kw_adjacent => vec![("adjacent", "Some detail")],
//     Rule::kw_and => vec![("and", "Some detail")],
//     Rule::kw_any => vec![("any", "Some detail")],
//     Rule::kw_as => vec![("as", "Some detail")],
//     Rule::kw_at => vec![("at", "Some detail")],
//     Rule::kw_bid => vec![("bid", "Some detail")],
//     Rule::kw_bottom => vec![("bottom", "Some detail")],
//     Rule::kw_card => vec![("card", "Some detail")],
//     Rule::kw_case => vec![("case", "Some detail")],
//     Rule::kw_choose => vec![("choose", "Some detail")],
//     Rule::kw_combo => vec![("combo", "Some detail")],
//     Rule::kw_competitor => vec![("competitor", "Some detail")],
//     Rule::kw_conditional => vec![("conditional", "Some detail")],
//     Rule::kw_create => vec![("create", "Some detail")],
//     Rule::kw_current => vec![("current", "Some detail")],
//     Rule::kw_cycle => vec![("cycle", "Some detail")],
//     Rule::kw_deal => vec![("deal", "Some detail")],
//     Rule::kw_demand => vec![("demand", "Some detail")],
//     Rule::kw_distinct => vec![("distinct", "Some detail")],
//     Rule::kw_down => vec![("distinct", "Some detail")],
//     // TODO: up + private
//     Rule::kw_face => vec![("face", "Some detail")],
//     Rule::kw_empty => vec![("empty", "Some detail")],
//     Rule::kw_end => vec![("end", "Some detail")],
//     Rule::kw_exchange => vec![("exchange", "Some detail")],
//     Rule::kw_fail => vec![("fail", "Some detail")],
//     Rule::kw_flip => vec![("flip", "Some detail")],
//     Rule::kw_for => vec![("for", "Some detail")],
//     Rule::kw_from => vec![("from", "Some detail")],
//     Rule::kw_game => vec![("game", "Some detail")],
//     Rule::kw_higher => vec![("higher", "Some detail")],
//     Rule::kw_highest => vec![("highest", "Some detail")],
//     Rule::kw_if => vec![("if", "Some detail")],
//     Rule::kw_in => vec![("in", "Some detail")],
//     Rule::kw_is => vec![("is", "Some detail")],
//     Rule::kw_location => vec![("location", "Some detail")],
//     Rule::kw_lower => vec![("lower", "Some detail")],
//     Rule::kw_lowest => vec![("lowest", "Some detail")],
//     Rule::kw_max => vec![("max", "Some detail")],
//     Rule::kw_memory => vec![("memory", "Some detail")],
//     Rule::kw_min => vec![("min", "Some detail")],
//     Rule::kw_move => vec![("move", "Some detail")],
//     Rule::kw_next => vec![("next", "Some detail")],
//     Rule::kw_not => vec![("not", "Some detail")],
//     Rule::kw_of => vec![("of", "Some detail")],
//     Rule::kw_on => vec![("on", "Some detail")],
//     Rule::kw_optional => vec![("optional", "Some detail")],
//     Rule::kw_or => vec![("or", "Some detail")],
//     Rule::kw_other => vec![("other", "Some detail")],
//     Rule::kw_others => vec![("others", "Some detail")],
//     Rule::kw_out => vec![("out", "Some detail")],
//     Rule::kw_owner => vec![("owner", "Some detail")],
//     Rule::kw_place => vec![("place", "Some detail")],
//     Rule::kw_player => vec![("player", "Some detail")],
//     Rule::kw_playersin => vec![("playersin", "Some detail")],
//     Rule::kw_playersout => vec![("playersout", "Some detail")],
//     Rule::kw_playroundcounter => vec![("playroundcounter", "Some detail")],
//     Rule::kw_pointmap => vec![("pointmap", "Some detail")],
//     Rule::kw_points => vec![("points", "Some detail")],
//     Rule::kw_position => vec![("position", "Some detail")],
//     Rule::kw_precedence => vec![("precedence", "Some detail")],
//     Rule::kw_previous => vec![("previous", "Some detail")],
//     Rule::kw_private => vec![("private", "Some detail")],
//     Rule::kw_random => vec![("random", "Some detail")],
//     Rule::kw_reset => vec![("reset", "Some detail")],
//     Rule::kw_same => vec![("same", "Some detail")],
//     Rule::kw_score => vec![("score", "Some detail")],
//     Rule::kw_set => vec![("set", "Some detail")],
//     Rule::kw_shuffle => vec![("shuffle", "Some detail")],
//     Rule::kw_size => vec![("size", "Some detail")],
//     Rule::kw_stage => vec![("stage", "Some detail")],
//     Rule::kw_stageroundcounter => vec![("stageroundcounter", "Some detail")],
//     Rule::kw_successful => vec![("successful", "Some detail")],
//     Rule::kw_sum => vec![("sum", "Some detail")],
//     Rule::kw_table => vec![("table", "Some detail")],
//     Rule::kw_team => vec![("team", "Some detail")],
//     Rule::kw_teams => vec![("teams", "Some detail")],
//     Rule::kw_times => vec![("times", "Some detail")],
//     Rule::kw_to => vec![("to", "Some detail")],
//     Rule::kw_token => vec![("token", "Some detail")],
//     Rule::kw_top => vec![("top", "Some detail")],
//     Rule::kw_turn => vec![("turn", "Some detail")],
//     Rule::kw_turnorder => vec![("turnorder", "Some detail")],
//     Rule::kw_until => vec![("until", "Some detail")],
//     Rule::kw_up => vec![("up", "Some detail")],
//     Rule::kw_using => vec![("using", "Some detail")],
//     Rule::kw_where => vec![("where", "Some detail")],
//     Rule::kw_winner => vec![("winner", "Some detail")],
//     Rule::kw_with => vec![("with", "Some detail")],
//     Rule::eq => vec![("==", "Some detail")],
//     Rule::neq => vec![("!=", "Some detail")],
//     Rule::lt => vec![("<", "Some detail")],
//     Rule::gt => vec![(">", "Some detail")],
//     Rule::le => vec![("<=", "Some detail")],
//     Rule::ge => vec![(">=", "Some detail")],
//     Rule::plus => vec![("+", "Some detail")],
//     Rule::minus => vec![("-", "Some detail")],
//     Rule::mul => vec![("*", "Some detail")],
//     Rule::div => vec![("/", "Some detail")],
//     Rule::modulo => vec![("mod", "Some detail")],
//     _ => vec![]
//   }
// }

// fn snippet_list() -> Vec<CompletionItem> {
//   vec![
//     make_snippet(
//         "stage", 
//         "stage ${1:id} for ${2:player_expr} ${3:end_condition} {\n\t$0\n}", 
//         "SeqStage-statement template"
//     ),
//     make_snippet(
//         "if", 
//         "if (${1:condition}) {\n\t$0\n}", 
//         "If-statement template"
//     ),
//     make_snippet(
//         "optional", 
//         "optional {\n\t$0\n}", 
//         "Optional-statement template"
//     ),
//     make_snippet(
//         "choose", 
//         "choose {\n\t$0\n\t$1\n}", 
//         "Choose-statement template"
//     ),
//     make_snippet(
//         "move", 
//         "move ${2:location} ${3|face up,face down,private|} to ${3:destination}", 
//         "Basic move"
//     ),
//     make_snippet(
//         "move (with quantity)", 
//         "move ${1:quantity} from ${2:location} ${3|face up,face down,private|} to ${4:destination}", 
//         "Move with Quantity"
//     ),
//     make_snippet(
//         "deal", 
//         "deal ${2:location} ${3|face up,face down,private|} to ${3:destination}", 
//         "Basic deal"
//     ),
//     make_snippet(
//         "deal (with quantity)", 
//         "deal ${1:quantity} from ${2:location} ${3|face up,face down, private|} to ${4:destination}", 
//         "Deal with Quantity"
//     ),
//     make_snippet(
//         "exchange", 
//         "exchange ${2:location} ${3|face up,face down,private|} to ${3:destination}", 
//         "Basic exchange"
//     ),
//     make_snippet(
//         "exchange (with quantity)", 
//         "exchange ${1:quantity} from ${2:location} ${3|face up,face down, private|} to ${4:destination}", 
//         "Exchange with Quantity"
//     ),
//     make_snippet(
//         "place", 
//         "place ${2:token} from ${3:location} to ${4:destination}", 
//         "Basic place"
//     ),
//     make_snippet(
//         "place (with quantity)", 
//         "place ${1:quantity} ${2:token} from ${3:location} to ${4:destination}", 
//         "Place with Quantity"
//     ),
//   ]
// }