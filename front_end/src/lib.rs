pub mod parser;
pub mod lower;
pub mod ast;
pub mod ir;
pub mod keywords;
pub mod symbols;
pub mod visitor;
pub mod helper;
pub mod spans;
// pub mod tests;
pub mod diagnostics;
pub mod semantic;
pub mod walker;
pub mod validation;
#[cfg(test)]
pub mod tests;

// The include! macro looks specifically into the OUT_DIR of THIS crate.
// This is why the code must be included here in the lib first.
include!(concat!(env!("OUT_DIR"), "/generated_snippets.rs"));

// Now the function `get_snippet_map()` exists in this module.
// We make it public so the server crate can see it.
pub fn get_all_snippets() -> std::collections::HashMap<&'static str, Vec<&'static str>> {
    get_snippet_map()
}