use dashmap::DashMap;
use front_end::{ast::ast::{Game, SGame}, get_all_snippets, parser::Rule, symbols::{GameType, SymbolVisitor, Var}, walker::Walker};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse, InsertTextFormat, Position};
use pest_consume::Error;
use pest::error;
// use get_snippet_map;
use std::collections::HashMap;
use std::sync::LazyLock;

static SNIPPET_LOOKUP: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
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
              
              let items = SNIPPET_LOOKUP.get(rule_name.as_str()).map(|body| {
                  let mut vec = Vec::new();

                  for s in body.iter() {
                    vec.push(
                      CompletionItem {
                          label: clean_label(s),
                          insert_text: Some(s.to_string()),
                          insert_text_format: Some(InsertTextFormat::SNIPPET),
                          kind: Some(CompletionItemKind::SNIPPET),
                          ..Default::default()
                      }
                    )
                  }

                  vec
              });

              match items {
                Some(completion) => {
                  completion_response.extend(completion);
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

fn clean_label(input: &str) -> String {
    let clean = input.replace("kw_", "").replace("_", " ");
    clean.chars()
        .filter(|c| {
            // Keep the character ONLY if it is NOT one of these:
            !c.is_numeric() && !matches!(c, '$' | ':' | '{' | '}')
        })
        .collect::<String>()
        .trim() // Optional: clean up leading/trailing spaces
        .to_string()
}
