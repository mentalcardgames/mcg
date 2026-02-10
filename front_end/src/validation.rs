use pest_consume::Parser;
use crate::parser::Result;
use crate::semantic::{SemanticError, SemanticVisitor};
use crate::{ast::ast::SGame, parser::{CGDSLParser, Rule}, symbols::{SymbolError, SymbolVisitor}, walker::Walker};

pub fn parse_document(text: &str) -> Result<SGame> {
  // 1. Parsing: pest_consume::parse already returns Result<Nodes, Error<Rule>>
  let nodes = CGDSLParser::parse(Rule::file, text)?;

  // 2. Extract Single Node: .single() returns Result<Node, Error<Rule>>
  let node = nodes.single()?;

  // 3. Mapping: mapper returns Result<T, Error<Rule>>
  let parsed_ast = CGDSLParser::file(node)?;

  Ok(parsed_ast)
}

pub fn symbol_validation(game: &SGame) -> Option<Vec<SymbolError>> {
  let mut symbols = SymbolVisitor::new();
  game.walk(&mut symbols);

  return symbols.check_game_type()
}

pub fn semantic_validation(game: &SGame) -> Option<Vec<SemanticError>> {
  let mut semantic = SemanticVisitor::new();
  game.walk(&mut semantic);

  return semantic.semantic_check()
}