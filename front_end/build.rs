use pest_meta::ast::Expr;
use pest_meta::parser::{self, Rule as MetaRule};

fn main() {
    // Tell Cargo to re-run this script if the grammar changes
    println!("cargo:rerun-if-changed=src/grammar.pest");

    let grammar_content = std::fs::read_to_string("./src/grammar.pest").expect("Check path");
    
    let rules = parser::parse(MetaRule::grammar_rules, &grammar_content)
        .expect("Grammar is invalid");
    let ast = parser::consume_rules(rules).expect("Failed to consume rules");

    let mut code = String::from("use std::collections::HashMap;\n\n");
    code.push_str("pub fn get_snippet_map() -> HashMap<&'static str, &'static str> {\n");
    code.push_str("    let mut m = HashMap::new();\n");

    for rule in &ast {
        // Skip infrastructure and silent rules
        if is_infrastructure(&rule.name) || rule.name.starts_with('_') {
            continue;
        }

        // Generate the snippet body
        let (body, _) = flatten_expression(&rule.expr, 1);
        
        // Escape quotes so the generated Rust code is valid
        let escaped_body = body.replace("\"", "\\\"");

        code.push_str(&format!(
            "    m.insert(\"{}\", \"{}\");\n", 
            rule.name, 
            escaped_body
        ));
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
    )
}

// 4. The "Pass Down" Logic
fn flatten_expression(expr: &Expr, t: usize) -> (String, usize) {
    match expr {
        Expr::Str(s) => (s.clone(), t),
        Expr::Ident(name) => {
            if name.starts_with("kw_") {
                (name.replace("kw_", ""), t)
            } else {
                (format!("${{{}:{}}}", t, name), t + 1)
            }
        }
        Expr::Seq(lhs, rhs) => {
            let (l, next_t) = flatten_expression(lhs, t);
            let (r, final_t) = flatten_expression(rhs, next_t);
            (format!("{} {}", l, r), final_t)
        }
        // For Choices (|), we suggest the first branch as the template
        Expr::Choice(lhs, _) => flatten_expression(lhs, t),
        // For Optionals (?), we wrap it in a tab-stop
        Expr::Opt(inner) => {
            let (s, next_t) = flatten_expression(inner, t);
            (format!("${{{}:[{}]}}", t, s), next_t + 1)
        }
        _ => ("".to_string(), t),
    }
}