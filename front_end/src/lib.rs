pub mod parser;
pub mod lower;
pub mod fmt_ast;
pub mod arbitrary;
include!("ast.rs");
pub mod ir;
pub mod symbols;
pub mod visitor;
pub mod spans;
pub mod fsm_to_dot;
pub mod semantic;
pub mod walker;
pub mod validation;

#[cfg(test)]
pub mod tests;

include!(concat!(env!("OUT_DIR"), "/generated_snippets.rs"));
pub fn get_all_snippets() -> std::collections::HashMap<&'static str, Vec<&'static str>> {
    get_snippet_map()
}