use std::collections::HashMap;

use pest_meta::ast::Expr;
use pest_meta::parser::{self, Rule as MetaRule};

fn main() {
    // Tell Cargo to re-run this script if the grammar changes
    println!("cargo:rerun-if-changed=src/grammar.pest");

    let grammar_content = std::fs::read_to_string("./src/grammar.pest").expect("Check path");
    
    let rules = parser::parse(MetaRule::grammar_rules, &grammar_content)
        .expect("Grammar is invalid");
    let ast = parser::consume_rules(rules).expect("Failed to consume rules");

    // Create a map for rule lookups
    let rule_map: HashMap<String, &Expr> = ast
        .iter()
        .map(|r| (r.name.clone(), &r.expr))
        .collect();

    let mut code = String::from("use std::collections::HashMap;\n\n");
    code.push_str("pub fn get_snippet_map() -> HashMap<&'static str, Vec<&'static str>> {\n");
    code.push_str("    let mut m = HashMap::new();\n");

    // Inside your main loop in build.rs
    for rule in &ast {
        if is_infrastructure(&rule.name) || rule.name.starts_with('_') {
            continue;
        }

        let variants = flatten_expression(&rule.expr, 1, &rule_map);
        
        for (body, _) in variants {
            let trimmed = body.trim();
            if trimmed.is_empty() { continue; }

            let escaped_body = trimmed.replace("\"", "\\\"");
            
            // We use rule.name as the key. 
            // Note: If one rule has multiple snippets, you might want to 
            // store Vec<&'static str> in your Map instead of a single string.
            code.push_str(&format!(
                "    m.entry(\"{}\").or_insert_with(Vec::new).push(\"{}\");\n", 
                rule.name, 
                escaped_body
            ));
        }
    }

    code.push_str("    m\n}");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("generated_snippets.rs");
    std::fs::write(&dest_path, code).unwrap();
}

fn is_infrastructure(name: &str) -> bool {
    matches!(name,
        "WHITESPACE" 
      | "alpha" 
      | "digit" 
      | "int" 
      | "ident" 
      | "SOI" 
      | "EOI" 
      | "kw" 
      | "block_comment" 
      | "line_comment" 
      | "playername" 
      | "teamname"
      | "location"
      | "precedence"
      | "pointmap"
      | "combo"
      | "key"
      | "value"
      | "memory"
      | "token"
      | "stage"
      | "game"
      | "flow_component"
      | "file"
      | "game_flow"
    )
}

// 4. The "Pass Down" Logic
fn flatten_expression(expr: &Expr, t: usize, rule_map: &HashMap<String, &Expr>) -> Vec<(String, usize)> {
    match expr {
        Expr::Str(s) => vec![(s.clone(), t)],
        
        Expr::Ident(name) => {
            if name.starts_with("kw_") {
                // It's a keyword, stop and return the text
                vec![(name.replace("kw_", ""), t)]
            } else if let Some(sub_expr) = rule_map.get(name)
                   && (
                       name == "int_expr"
                    || name == "string_expr"
                    || name == "player_expr"
                    || name == "team_expr"
                    || name == "bool_expr"
                    || name == "filter_expr"
                    || name == "card_set"
                    || name == "card_position"
                   ) {
                // It's a sub-rule (e.g., 'player_expr'), flatten its definition
                flatten_expression(sub_expr, t, rule_map)
            } else {
                // Fallback for built-ins or rules not in map: return as placeholder
                vec![(format!("${{{}:{}}}", t, name), t + 1)]
            }
        }

        // SEQUENCE: Combine every left variation with every right variation
        Expr::Seq(lhs, rhs) => {
            let left_variations = flatten_expression(lhs, t, rule_map);
            let mut results = Vec::new();
            for (l_str, next_t) in left_variations {
                let right_variations = flatten_expression(rhs, next_t, rule_map);
                for (r_str, final_t) in right_variations {
                    results.push((format!("{} {}", l_str, r_str), final_t));
                }
            }
            results
        }

        // CHOICE: Flatten both sides and collect them all
        Expr::Choice(lhs, rhs) => {
            let mut variants = flatten_expression(lhs, t, rule_map);
            variants.extend(flatten_expression(rhs, t, rule_map));
            variants
        }

        // OPTIONAL: Return a version with the content AND a version with nothing
        Expr::Opt(inner) => {
            let mut variants = flatten_expression(inner, t, rule_map); // Version with
            variants.push(("".to_string(), t));            // Version without
            variants
        }

        _ => vec![("".to_string(), t)],
    }
}