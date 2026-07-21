/*
    We want to have some auto-completion. This is just a prototype sketch for very basic auto-completion.

    Feel free to optimize and making it better as it is right now.

    However due to the deeply recursive nature of the grammar this will be a big task to do it nicely.
*/

use std::collections::{HashMap, HashSet};

use pest_meta::ast::Expr;
use pest_meta::parser::{self, Rule as MetaRule};

fn main() {
    // Tell Cargo to re-run this script if the grammar changes
    println!("cargo:rerun-if-changed=src/grammar.pest");

    let grammar_content = std::fs::read_to_string("./src/grammar.pest").expect("Check path");

    let rules =
        parser::parse(MetaRule::grammar_rules, &grammar_content).expect("Grammar is invalid");
    let ast = parser::consume_rules(rules).expect("Failed to consume rules");

    // Create a map for rule lookups
    let rule_map: HashMap<String, &Expr> = ast.iter().map(|r| (r.name.clone(), &r.expr)).collect();

    let mut code = String::from("use std::collections::HashMap;\n\n");
    code.push_str("pub fn get_snippet_map() -> HashMap<&'static str, Vec<&'static str>> {\n");
    code.push_str("    let mut m = HashMap::new();\n");

    for rule in &ast {
        if is_infrastructure(&rule.name) || rule.name.starts_with('_') {
            continue;
        }

        let mut visited = Vec::new();
        let variants = get_first_terminals(&rule.expr, &rule_map, &mut visited);

        // Use a HashSet to filter out duplicates like "(" appearing multiple times
        let unique_variants: HashSet<String> = variants.into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect();

        for body in unique_variants {
            let escaped_body = body.replace("\"", "\\\"");

            code.push_str(&format!(
                "    m.entry(\"{}\").or_insert_with(Vec::new).push(\"{}\");\n",
                rule.name, escaped_body
            ));
        }
    }

    code.push_str("    m\n}");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("generated_snippets.rs");
    std::fs::write(&dest_path, code).unwrap();
}

fn is_infrastructure(name: &str) -> bool {
    matches!(
        name,
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
            | "game"
            | "flow_component"
            | "file"
            | "game_flow"
    )
}

fn is_ident(name: &str) -> bool {
    matches!(name, |"playername"| "teamname"
        | "location"
        | "precedence"
        | "pointmap"
        | "combo"
        | "key"
        | "value"
        | "memory"
        | "token"
        | "stage")
}

// The "Pass Down" Logic
#[allow(dead_code)]
fn flatten_expression(
    expr: &Expr,
    t: usize,
    rule_map: &HashMap<String, &Expr>,
    visited: &mut Vec<String>,
    depth: usize, // <--- Add this
) -> Vec<(String, usize)> {
    match expr {
        Expr::Ident(name) => {
            if name.starts_with("kw_") {
                vec![(name.replace("kw_", ""), t)]
            } else if depth > 0 && !visited.contains(name) && !is_ident(name) {
                if let Some(sub_expr) = rule_map.get(name) {
                    visited.push(name.clone());
                    // Recurse with depth - 1
                    let res = flatten_expression(sub_expr, t, rule_map, visited, depth - 1);
                    visited.pop();
                    return res;
                }
                // Fallback if rule not found
                vec![(format!("${{{}:{}}}", t, name), t + 1)]
            } else {
                // We reached max depth or a cycle: stop and show the rule name
                vec![(format!("${{{}:{}}}", t, name), t + 1)]
            }
        }
        // For Seq, Choice, and Opt, just pass the SAME depth down.
        // Only Ident "consumes" a level of depth.
        Expr::Seq(lhs, rhs) => {
            let lefts = flatten_expression(lhs, t, rule_map, visited, depth);
            let mut results = Vec::new();
            for (l, next_t) in lefts {
                let rights = flatten_expression(rhs, next_t, rule_map, visited, depth);
                for (r, final_t) in rights {
                    results.push((format!("{} {}", l, r), final_t));
                }
            }
            results
        }

        // CHOICE: Flatten both sides and collect them all
        Expr::Choice(lhs, rhs) => {
            let mut variants = flatten_expression(lhs, t, rule_map, visited, depth);
            variants.extend(flatten_expression(rhs, t, rule_map, visited, depth));
            variants
        }

        // OPTIONAL: Return a version with the content AND a version with nothing
        Expr::Opt(inner) => {
            let mut variants = flatten_expression(inner, t, rule_map, visited, depth); // Version with
            variants.push(("".to_string(), t)); // Version without
            variants
        }

        _ => vec![("".to_string(), t)],
    }
}

fn get_first_terminals(
    expr: &Expr,
    rule_map: &HashMap<String, &Expr>,
    visited: &mut Vec<String>,
) -> Vec<String> {
    match expr {
        Expr::Ident(name) => {
            // 1. If it's a keyword terminal, we found it!
            if name.starts_with("kw_") {
                return vec![name.replace("kw_", "")];
            }

            // 2. If it's a known placeholder (int, ident, etc.), 
            // you might want to return the name or nothing.
            if is_ident(name) || is_infrastructure(name) {
                // Return name as a hint, or return empty vec![] if you only want strings
                return vec![name.clone()]; 
            }

            // 3. Recurse into the rule definition
            if !visited.contains(name) {
                if let Some(sub_expr) = rule_map.get(name) {
                    visited.push(name.clone());
                    let res = get_first_terminals(sub_expr, rule_map, visited);
                    visited.pop();
                    return res;
                }
            }
            vec![]
        }

        // We only care about the first part of a sequence
        Expr::Seq(lhs, rhs) => {
            let mut results = get_first_terminals(lhs, rule_map, visited);
            // If the LHS could be empty (like an Optional rule), 
            // the "first" terminal could actually be the start of the RHS.
            if can_be_empty(lhs, rule_map, &mut Vec::new()) {
                results.extend(get_first_terminals(rhs, rule_map, visited));
            }
            results
        }

        // This is exactly what you want: get the firsts for ALL branches
        Expr::Choice(lhs, rhs) => {
            let mut variants = get_first_terminals(lhs, rule_map, visited);
            variants.extend(get_first_terminals(rhs, rule_map, visited));
            variants
        }

        // Drills into the content
        Expr::Opt(inner) | Expr::Rep(inner) | Expr::RepOnce(inner) => {
            get_first_terminals(inner, rule_map, visited)
        }

        // Direct string literals in the grammar: e.g., "(" or "match"
        Expr::Str(s) => vec![s.clone()],

        _ => vec![],
    }
}

fn can_be_empty(expr: &Expr, rule_map: &HashMap<String, &Expr>, visited: &mut Vec<String>) -> bool {
    match expr {
        Expr::Opt(_) | Expr::Rep(_) => true,
        Expr::Ident(name) => {
            if !visited.contains(name) {
                if let Some(sub_expr) = rule_map.get(name) {
                    visited.push(name.clone());
                    return can_be_empty(sub_expr, rule_map, visited);
                }
            }
            false
        },
        Expr::Seq(a, b) => can_be_empty(a, rule_map, visited) && can_be_empty(b, rule_map, visited),
        Expr::Choice(a, b) => can_be_empty(a, rule_map, visited) || can_be_empty(b, rule_map, visited),
        _ => false,
    }
}
