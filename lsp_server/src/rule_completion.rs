use front_end::{ast::GameType, parser::Rule, spans::{SGame}, symbols::{SymbolVisitor, Var}, walker::Walker};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse, InsertTextFormat, Position};
use pest_consume::Error;

pub fn try_auto_completion(
    err: Error<Rule>, 
    cursor: Position, 
    text: &str,
    last_ast: Option<&SGame>
) -> Option<CompletionResponse> {
    let ast;
    if let Some(a) = last_ast {
      ast = a;
    } else {
      return None
    }

    let mut symbols = SymbolVisitor::new();
    ast.walk(&mut symbols);
    
    let ctx = symbols.into_typed_vars();

    match err.variant {
        pest::error::ErrorVariant::ParsingError { positives, .. } => {
            if let Some(r) = positives.first() {
                // Convert cursor Position to byte index
                let lines: Vec<&str> = text.lines().collect();
                let mut byte_index = 0;
                for i in 0..cursor.line as usize {
                    byte_index += lines[i].len() + 1; // +1 for newline
                }
                byte_index += cursor.character as usize;

                // Only suggest completion if cursor is at/after error location
                let error_pos = match err.location {
                    pest::error::InputLocation::Pos(p) => p,
                    pest::error::InputLocation::Span((start, _)) => start,
                };

                if byte_index >= error_pos {
                    // Collect all unique completions for the rule the parser expected
                    let mut visited = std::collections::HashSet::new();
                    let items = get_completions_for_rule(r, &ctx, &mut visited, text);
                    
                    if items.is_empty() {
                        return None;
                    }
                    
                    return Some(CompletionResponse::Array(items));
                }
            }
            None
        },
        _ => None,
    }
}

fn get_current_word(text_before_cursor: &str) -> &str {
    text_before_cursor
        .split_whitespace() // Handles tabs, newlines, and multiple spaces
        .last()             // Gets the final element
        .unwrap_or("")      // If string is empty, return empty string
}

fn get_completions_for_rule(
    rule: &Rule, 
    ctx: &Vec<(Var, GameType)>, 
    visited: &mut std::collections::HashSet<Rule>,
    text: &str,
) -> Vec<CompletionItem> {
    let current_word = get_current_word(text);

    // Prevent infinite recursion loops
    if !visited.insert(*rule) {
        return vec![];
    }

    let mut items = Vec::new();

    // 1. Structural Snippets (High Priority)
    items.extend(match_snippet(current_word));

    // 2. Variable Lookups (Type-safe)
    for (label, detail) in complete_var(rule, ctx) {
        let mut item = CompletionItem::new_simple(label.to_string(), detail.to_string());
        item.kind = Some(CompletionItemKind::VARIABLE);
        items.push(item);
    }

    // 3. Keyword-only Rules (operators, comparisons)
    for (label, detail) in complete_kw_only_rule(rule) {
        let mut item = CompletionItem::new_simple(label.to_string(), detail.to_string());
        item.kind = Some(CompletionItemKind::KEYWORD);
        items.push(item);
    }

    // 4. Static Keyword Mappings (Pest atoms)
    for (label, detail) in get_static_key_word_completion(rule) {
        let mut item = CompletionItem::new_simple(label.to_string(), detail.to_string());
        item.kind = Some(CompletionItemKind::KEYWORD);
        items.push(item);
    }

    // 5. One-Level Deep Recursion for Compound Rules
    // This allows Rule::bool_expr to show everything inside int_expr, etc.
    match rule {
        Rule::flow_component => {
            items.extend(get_completions_for_rule(&Rule::game_rule, ctx, visited, text));
        }
        Rule::game_rule => {
            items.extend(get_completions_for_rule(&Rule::move_action, ctx, visited, text));
            // Add other complex sub-rules here
        }
        // Rule::bool_expr => {
        //     items.extend(get_completions_for_rule(&Rule::int_expr, ctx, visited));
        //     items.extend(get_completions_for_rule(&Rule::logical_compare, ctx, visited));
        // }
        _ => {}
    }

    // Deduplicate items by label so the UI stays clean
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.label == b.label);

    items
}

fn match_snippet(current_word: &str) -> Vec<CompletionItem> {
  let snippets: Vec<CompletionItem> = snippet_list();

  snippets.into_iter().filter(|item| item.label.to_lowercase().starts_with(&current_word.to_lowercase())).collect()
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

fn complete_var<'a>(rule: &Rule, ctx: &'a Vec<(Var, GameType)>) -> Vec<(&'a str, &'a str)> {
    match rule {
      Rule::stage => {
        helper_complete_var(ctx, GameType::Stage)
      },
      Rule::teamname => {
        helper_complete_var(ctx, GameType::Team)
      },
      Rule::playername => {
        helper_complete_var(ctx, GameType::Player)
      },
      Rule::location => {
        helper_complete_var(ctx, GameType::Location)
      },
      Rule::token => {
        helper_complete_var(ctx, GameType::Token)
      },
      Rule::precedence => {
        helper_complete_var(ctx, GameType::Precedence)
      },
      Rule::combo => {
        helper_complete_var(ctx, GameType::Combo)
      },
      Rule::memory => {
        helper_complete_var(ctx, GameType::Memory)
      },
      Rule::pointmap => {
        helper_complete_var(ctx, GameType::PointMap)
      },
      Rule::key => {
        helper_complete_var(ctx, GameType::Key)
      },
      Rule::value => {
        helper_complete_var(ctx, GameType::Value)
      },
      _ => vec![]
    }
}

fn helper_complete_var(ctx: &Vec<(Var, GameType)>, game_type: GameType) -> Vec<(&str, &str)> {
  ctx
    .iter()
    .filter(|(_, ty)| *ty == game_type)
    .map(|(v, _)| (v.id.as_str(), "Some detail"))
    .collect::<Vec<(&str, &str)>>()
}

fn complete_kw_only_rule(rule: &Rule) -> Vec<(&str, &str)> {
  match rule {
      Rule::logical_compare => {
        vec![
          ("and", "Some detail"),
          ("or", "Some detail"),
        ]
      },
      Rule::int_op => {
        vec![
          ("+", "Some detail"),
          ("-", "Some detail"),
          ("*", "Some detail"),
          ("/", "Some detail"),
          ("mod", "Some detail"),
        ]
      },
      Rule::int_compare => {
        vec![
          ("==", "Some detail"),
          ("!=", "Some detail"),
          ("<", "Some detail"),
          (">", "Some detail"),
          ("<=", "Some detail"),
          (">=", "Some detail"),
        ]
      },
      Rule::extrema => {
        vec![
          ("min", "Some detail"),
          ("max", "Some detail"),
          ("lowest", "Some detail"),
          ("highest", "Some detail"),
        ]
      },
      Rule::card_set_compare => {
        vec![
          ("==", "Some detail"),
          ("!=", "Some detail"),
        ]
      },
      Rule::player_expr_compare => {
        vec![
          ("==", "Some detail"),
          ("!=", "Some detail"),
        ]
      },
      Rule::team_expr_compare => {
        vec![
          ("==", "Some detail"),
          ("!=", "Some detail"),
        ]
      },
      Rule::string_expr_compare => {
        vec![
          ("==", "Some detail"),
          ("!=", "Some detail"),
        ]
      },
      Rule::bool_op => {
        vec![
          ("and", "Some detail"),
          ("or", "Some detail"),
        ]
      },
      Rule::unary_op => {
        vec![
          ("not", "Some detail"),
        ]
      },
      Rule::face_up => {
        vec![
          ("face up", "Some detail"),
        ]
      },
      Rule::face_down => {
        vec![
          ("face down", "Some detail"),
        ]
      },
      Rule::private => {
        vec![
          ("private", "Some detail"),
        ]
      },
      Rule::status => {
        vec![
          ("face up", "Some detail"),
          ("face down", "Some detail"),
          ("private", "Some detail"),
        ]
      },
      Rule::quantifier => {
        vec![
          ("all", "Some detail"),
          ("any", "Some detail"),
        ]
      }
      Rule::other_teams => {
        vec![
          ("other teams", "Some detail"),
        ]
      },
      Rule::until_end => {
        vec![
          ("until end", "Some detail"),
        ]
      },
      _ => vec![]
  }
}

fn get_static_key_word_completion(rule: &Rule) -> Vec<(&str, &str)> {
  match rule {
    Rule::kw_adjacent => vec![("adjacent", "Some detail")],
    Rule::kw_and => vec![("and", "Some detail")],
    Rule::kw_any => vec![("any", "Some detail")],
    Rule::kw_as => vec![("as", "Some detail")],
    Rule::kw_at => vec![("at", "Some detail")],
    Rule::kw_bid => vec![("bid", "Some detail")],
    Rule::kw_bottom => vec![("bottom", "Some detail")],
    Rule::kw_card => vec![("card", "Some detail")],
    Rule::kw_case => vec![("case", "Some detail")],
    Rule::kw_choose => vec![("choose", "Some detail")],
    Rule::kw_combo => vec![("combo", "Some detail")],
    Rule::kw_competitor => vec![("competitor", "Some detail")],
    Rule::kw_conditional => vec![("conditional", "Some detail")],
    Rule::kw_create => vec![("create", "Some detail")],
    Rule::kw_current => vec![("current", "Some detail")],
    Rule::kw_cycle => vec![("cycle", "Some detail")],
    Rule::kw_deal => vec![("deal", "Some detail")],
    Rule::kw_demand => vec![("demand", "Some detail")],
    Rule::kw_distinct => vec![("distinct", "Some detail")],
    Rule::kw_down => vec![("distinct", "Some detail")],
    // TODO: up + private
    Rule::kw_face => vec![("face", "Some detail")],
    Rule::kw_empty => vec![("empty", "Some detail")],
    Rule::kw_end => vec![("end", "Some detail")],
    Rule::kw_exchange => vec![("exchange", "Some detail")],
    Rule::kw_fail => vec![("fail", "Some detail")],
    Rule::kw_flip => vec![("flip", "Some detail")],
    Rule::kw_for => vec![("for", "Some detail")],
    Rule::kw_from => vec![("from", "Some detail")],
    Rule::kw_game => vec![("game", "Some detail")],
    Rule::kw_higher => vec![("higher", "Some detail")],
    Rule::kw_highest => vec![("highest", "Some detail")],
    Rule::kw_if => vec![("if", "Some detail")],
    Rule::kw_in => vec![("in", "Some detail")],
    Rule::kw_is => vec![("is", "Some detail")],
    Rule::kw_location => vec![("location", "Some detail")],
    Rule::kw_lower => vec![("lower", "Some detail")],
    Rule::kw_lowest => vec![("lowest", "Some detail")],
    Rule::kw_max => vec![("max", "Some detail")],
    Rule::kw_memory => vec![("memory", "Some detail")],
    Rule::kw_min => vec![("min", "Some detail")],
    Rule::kw_move => vec![("move", "Some detail")],
    Rule::kw_next => vec![("next", "Some detail")],
    Rule::kw_not => vec![("not", "Some detail")],
    Rule::kw_of => vec![("of", "Some detail")],
    Rule::kw_on => vec![("on", "Some detail")],
    Rule::kw_optional => vec![("optional", "Some detail")],
    Rule::kw_or => vec![("or", "Some detail")],
    Rule::kw_other => vec![("other", "Some detail")],
    Rule::kw_others => vec![("others", "Some detail")],
    Rule::kw_out => vec![("out", "Some detail")],
    Rule::kw_owner => vec![("owner", "Some detail")],
    Rule::kw_place => vec![("place", "Some detail")],
    Rule::kw_player => vec![("player", "Some detail")],
    Rule::kw_playersin => vec![("playersin", "Some detail")],
    Rule::kw_playersout => vec![("playersout", "Some detail")],
    Rule::kw_playroundcounter => vec![("playroundcounter", "Some detail")],
    Rule::kw_pointmap => vec![("pointmap", "Some detail")],
    Rule::kw_points => vec![("points", "Some detail")],
    Rule::kw_position => vec![("position", "Some detail")],
    Rule::kw_precedence => vec![("precedence", "Some detail")],
    Rule::kw_previous => vec![("previous", "Some detail")],
    Rule::kw_private => vec![("private", "Some detail")],
    Rule::kw_random => vec![("random", "Some detail")],
    Rule::kw_reset => vec![("reset", "Some detail")],
    Rule::kw_same => vec![("same", "Some detail")],
    Rule::kw_score => vec![("score", "Some detail")],
    Rule::kw_set => vec![("set", "Some detail")],
    Rule::kw_shuffle => vec![("shuffle", "Some detail")],
    Rule::kw_size => vec![("size", "Some detail")],
    Rule::kw_stage => vec![("stage", "Some detail")],
    Rule::kw_stageroundcounter => vec![("stageroundcounter", "Some detail")],
    Rule::kw_successful => vec![("successful", "Some detail")],
    Rule::kw_sum => vec![("sum", "Some detail")],
    Rule::kw_table => vec![("table", "Some detail")],
    Rule::kw_team => vec![("team", "Some detail")],
    Rule::kw_teams => vec![("teams", "Some detail")],
    Rule::kw_times => vec![("times", "Some detail")],
    Rule::kw_to => vec![("to", "Some detail")],
    Rule::kw_token => vec![("token", "Some detail")],
    Rule::kw_top => vec![("top", "Some detail")],
    Rule::kw_turn => vec![("turn", "Some detail")],
    Rule::kw_turnorder => vec![("turnorder", "Some detail")],
    Rule::kw_until => vec![("until", "Some detail")],
    Rule::kw_up => vec![("up", "Some detail")],
    Rule::kw_using => vec![("using", "Some detail")],
    Rule::kw_where => vec![("where", "Some detail")],
    Rule::kw_winner => vec![("winner", "Some detail")],
    Rule::kw_with => vec![("with", "Some detail")],
    Rule::eq => vec![("==", "Some detail")],
    Rule::neq => vec![("!=", "Some detail")],
    Rule::lt => vec![("<", "Some detail")],
    Rule::gt => vec![(">", "Some detail")],
    Rule::le => vec![("<=", "Some detail")],
    Rule::ge => vec![(">=", "Some detail")],
    Rule::plus => vec![("+", "Some detail")],
    Rule::minus => vec![("-", "Some detail")],
    Rule::mul => vec![("*", "Some detail")],
    Rule::div => vec![("/", "Some detail")],
    Rule::modulo => vec![("mod", "Some detail")],
    _ => vec![]
  }
}

fn snippet_list() -> Vec<CompletionItem> {
  vec![
    make_snippet(
        "stage", 
        "stage ${1:id} for ${2:player_expr} ${3:end_condition} {\n\t$0\n}", 
        "SeqStage-statement template"
    ),
    make_snippet(
        "if", 
        "if (${1:condition}) {\n\t$0\n}", 
        "If-statement template"
    ),
    make_snippet(
        "optional", 
        "optional {\n\t$0\n}", 
        "Optional-statement template"
    ),
    make_snippet(
        "choose", 
        "choose {\n\t$0\n\t$1\n}", 
        "Choose-statement template"
    ),
    make_snippet(
        "move", 
        "move ${2:location} ${3|face up,face down,private|} to ${3:destination}", 
        "Basic move"
    ),
    make_snippet(
        "move (with quantity)", 
        "move ${1:quantity} from ${2:location} ${3|face up,face down,private|} to ${4:destination}", 
        "Move with Quantity"
    ),
    make_snippet(
        "deal", 
        "deal ${2:location} ${3|face up,face down,private|} to ${3:destination}", 
        "Basic deal"
    ),
    make_snippet(
        "deal (with quantity)", 
        "deal ${1:quantity} from ${2:location} ${3|face up,face down, private|} to ${4:destination}", 
        "Deal with Quantity"
    ),
    make_snippet(
        "exchange", 
        "exchange ${2:location} ${3|face up,face down,private|} to ${3:destination}", 
        "Basic exchange"
    ),
    make_snippet(
        "exchange (with quantity)", 
        "exchange ${1:quantity} from ${2:location} ${3|face up,face down, private|} to ${4:destination}", 
        "Exchange with Quantity"
    ),
    make_snippet(
        "place", 
        "place ${2:token} from ${3:location} to ${4:destination}", 
        "Basic place"
    ),
    make_snippet(
        "place (with quantity)", 
        "place ${1:quantity} ${2:token} from ${3:location} to ${4:destination}", 
        "Place with Quantity"
    ),
  ]
}