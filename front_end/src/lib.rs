pub mod arbitrary;
pub mod fmt_ast;
pub mod lower;
pub mod parser;
include!("ast.rs");
pub mod fsm_to_dot;
pub mod ir;
pub mod semantic;
pub mod spans;
pub mod symbols;
pub mod validation;
pub mod walker;

#[cfg(test)]
pub mod tests;

include!(concat!(env!("OUT_DIR"), "/generated_snippets.rs"));
pub fn get_all_snippets() -> std::collections::HashMap<&'static str, Vec<&'static str>> {
    get_snippet_map()
}
