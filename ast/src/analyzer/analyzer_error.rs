use crate::parse::ast_to_typed_ast::TypeError;

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
    // TODO: More precise AnalyzerErrors
    Default,
}
