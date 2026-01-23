use crate::{ast_to_typed_ast::TypeError, visit_typed_vars::TypedVars};

#[derive(Debug)]
pub enum AnalyzerError {
    NoDslType,
    IdUsed,
    IdNotCapitalOrEmpty,
    InvalidInteger,
    ReservedKeyword,
    UnknownID(String),
    UnknownPlayerNameUsed(String),
    DuplicateIDs(Vec<String>),
    TypeError(TypeError),
    IDWithMultipleTypes { id: String },
    IDWithNoType { id: String },
    IDNotInitialized {id: String },
    NonDeterministicInitialization { created: TypedVars },
    // TODO: More precise AnalyzerErrors
    Default,
}
