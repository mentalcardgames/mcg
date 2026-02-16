use std::{collections::HashMap};

use crate::{ast::ast_spanned::{NodeKind, *}, spans::OwnedSpan, symbols::Var, walker::AstPass};

pub enum SemanticError {
  KeyNotFoundForType { ty: String, key: Var },
  NoCorrToType { ty: Var, key: Var },
}

#[derive(Debug, PartialEq, Clone)]
pub struct UsedCorrespondence {
  pub ty: CorrespondanceType,
  pub key: String,
  pub span: OwnedSpan,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum CorrespondanceType {
  Precedence { node: String },
  PointMap   { node: String },
  Value      { node: String },
  Key        { node: String },
}

impl CorrespondanceType {
  pub fn get_node(&self) -> String {
    match self {
        CorrespondanceType::Precedence { node } => node.clone(),
        CorrespondanceType::PointMap { node } => node.clone(),
        CorrespondanceType::Value { node } => node.clone(),
        CorrespondanceType::Key { node } => node.clone(),
    }
  }
}

pub struct SemanticVisitor {
  init_corr: HashMap<CorrespondanceType, (String, OwnedSpan)>,
  used_corr: Vec<UsedCorrespondence>
}

impl SemanticVisitor {
  pub fn new() -> Self {
    SemanticVisitor {
      init_corr: HashMap::new(),
      used_corr: Vec::new(),
    }
  }

  pub fn semantic_check(&self) -> Option<Vec<SemanticError>> {
    let mut err = Vec::new();

    for corr in self.used_corr.iter() {
      match self.init_corr.get(&corr.ty) {
        Some((key, span)) => {
          if corr.key != *key {
            err.push(SemanticError::NoCorrToType { 
              ty: Var { id: corr.ty.get_node(), span: span.clone() },
              key: Var { id: corr.key.clone(), span: corr.span.clone() } 
            });
          }
        },
        None => {
          err.push(SemanticError::KeyNotFoundForType { 
            ty: corr.ty.get_node(), 
            key: Var { id: corr.key.clone(), span: corr.span.clone() } 
          });
        },
      }
    }

    if err.is_empty() {
      return None
    }

    return Some(err)
  }
}

impl AstPass for SemanticVisitor {
  fn enter_node<T: crate::walker::Walker>(&mut self, node: &T)
  where
    Self: Sized {
      if let Some(unwrapped_node) = node.kind() {      
        match unwrapped_node {
          NodeKind::SetUpRule(s) => {
            match s {
              SetUpRule::CreatePrecedence(precedence, key_value_pairs) => {
                for (k, v) in key_value_pairs.iter() {
                  self.init_corr.insert( 
                    CorrespondanceType::Precedence { 
                      node: precedence.node.clone() 
                    }, 
                    (k.node.clone(), precedence.span.clone())
                  );
                  self.used_corr.push(
                    UsedCorrespondence { 
                      ty: CorrespondanceType::Value { node: v.node.clone() }, 
                      key: k.node.clone(), 
                      span: v.span.clone() 
                    }
                  );
                }
              },
              SetUpRule::CreatePointMap(pointmap, key_value_int_triples) => {
                for (k, v, _) in key_value_int_triples.iter() {
                  self.init_corr.insert( 
                    CorrespondanceType::PointMap { 
                      node: pointmap.node.clone() 
                    }, 
                    (k.node.clone(), pointmap.span.clone())
                  );
                  self.used_corr.push(
                    UsedCorrespondence { 
                      ty: CorrespondanceType::Value { node: v.node.clone() }, 
                      key: k.node.clone(), 
                      span: v.span.clone() 
                    }
                  );
                }
              },
              SetUpRule::CreateCardOnLocation(_, types) => {
                for (k, vs) in types.node.types.iter() {
                  for v in vs.iter() {
                    self.init_corr.insert( 
                        CorrespondanceType::Value { 
                          node: v.node.clone() 
                        }, 
                      (k.node.clone(), v.span.clone())
                    );
                  }
                }
              },
              _ => {},
            }
          },
          NodeKind::AggregateFilter(a) => {
            match a {
                AggregateFilter::Adjacent(key, precedence) => {
                  self.used_corr.push(
                    UsedCorrespondence { 
                      ty: CorrespondanceType::Precedence { node: precedence.node.clone() }, 
                      key: key.node.clone(), 
                      span: precedence.span.clone() 
                    }
                  );
                },
                AggregateFilter::Higher(key, precedence) => {
                  self.used_corr.push(
                    UsedCorrespondence { 
                      ty: CorrespondanceType::Precedence { node: precedence.node.clone() }, 
                      key: key.node.clone(), 
                      span: precedence.span.clone() 
                    }
                  );
                },
                AggregateFilter::Lower(key, precedence) => {
                  self.used_corr.push(
                    UsedCorrespondence { 
                      ty: CorrespondanceType::Precedence { node: precedence.node.clone() }, 
                      key: key.node.clone(), 
                      span: precedence.span.clone() 
                    }
                  );
                },
                AggregateFilter::KeyString(key, _, string) => {
                  match &string.node {
                    StringExpr::Query(q) => {
                      match &q.node {
                        QueryString::KeyOf(k, _) => {
                          self.init_corr.insert( 
                              CorrespondanceType::Key { 
                                node: k.node.clone() 
                              }, 
                            (k.node.clone(), k.span.clone())
                          );
                          self.used_corr.push(
                            UsedCorrespondence { 
                              ty: CorrespondanceType::Key { node: k.node.clone() }, 
                              key: key.node.clone(), 
                              span: k.span.clone() 
                            }
                          );
                        },
                        _ => {}
                      }
                    },
                    StringExpr::Literal(value) => {
                      self.used_corr.push(
                        UsedCorrespondence { 
                          ty: CorrespondanceType::Value { node: value.node.clone() }, 
                          key: key.node.clone(), 
                          span: value.span.clone() 
                        }
                      );
                    },
                  }
                },
                _ => {}
            }
          },
          _ => {},
        }
      }
    }

    fn exit_node<T: crate::walker::Walker>(&mut self, _: &T)
    where
        Self: Sized {
    }
}