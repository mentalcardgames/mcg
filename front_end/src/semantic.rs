use std::{collections::HashMap};

use crate::{ast::ast::{NodeKind, SID, *}, symbols::Var, walker::AstPass};

pub enum SemanticError {
  KeyNotInPrecedence { key: Var, precedence: Var },
  KeyNoCorrToPrecedence { key: Var, precedence: Var },
  KeyNotInPointMap { key: Var, pointmap: Var },
  KeyNoCorrToPointMap { key: Var, pointmap: Var },
  ValueNotInKey { key: Var, value: Var },
  ValueNoCorrToKey { key: Var, value: Var },
  KeyAndStringDontAllign { key: Var, string_key: Var },
}

#[derive(Debug, PartialEq, Clone)]
pub enum UsedCorrespondence {
  Precedence { name: SID, key: SID },
  PointMap { name: SID, key: SID },
  Value { name: SID, key: SID },
  String { name: SID, key: SID },
}

pub struct SemanticVisitor {
  precedences: HashMap<String, String>,
  pointmaps: HashMap<String, String>,
  values: HashMap<String, String>,

  used_corr: Vec<UsedCorrespondence>
}

impl SemanticVisitor {
  pub fn new() -> Self {
    SemanticVisitor {
      precedences: HashMap::new(),
      pointmaps: HashMap::new(),
      values: HashMap::new(),

      used_corr: Vec::new(),
    }
  }

  pub fn semantic_check(&self) -> Option<Vec<SemanticError>> {
    let mut err = Vec::new();

    for corr in self.used_corr.iter() {
      match corr {
        UsedCorrespondence::Precedence { name, key } => {
          if let Some(corr_key) = self.precedences.get(&name.node) {
            if key.node != *corr_key {
              err.push(SemanticError::KeyNoCorrToPrecedence { key: Var::from(key.clone()), precedence: Var::from(name.clone()) });
            }
          } else {
            err.push(SemanticError::KeyNotInPrecedence { key: Var::from(key.clone()), precedence: Var::from(name.clone()) });
          }
        },
        UsedCorrespondence::PointMap { name, key } => {
          if let Some(corr_key) = self.pointmaps.get(&name.node) {
            if key.node != *corr_key {
              err.push(SemanticError::KeyNoCorrToPointMap { key: Var::from(key.clone()), pointmap: Var::from(name.clone()) });
            }
          } else {
            err.push(SemanticError::KeyNotInPointMap { key: Var::from(key.clone()), pointmap: Var::from(name.clone()) });
          }
        },
        UsedCorrespondence::Value { name, key } => {
          if let Some(corr_key) = self.values.get(&name.node) {
            if key.node != *corr_key {
              err.push(SemanticError::ValueNoCorrToKey { key: Var::from(key.clone()), value: Var::from(name.clone()) });
            }
          } else {
            err.push(SemanticError::ValueNotInKey { key: Var::from(key.clone()), value: Var::from(name.clone()) });
          }
        },
        UsedCorrespondence::String { name, key } => {
          if name.node != key.node {
            err.push(SemanticError::KeyAndStringDontAllign { key: Var::from(key.clone()), string_key: Var::from(name.clone()) });
          }
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
                  self.precedences.insert(precedence.node.clone(), k.node.clone());
                  self.used_corr.push(UsedCorrespondence::Value { name: v.clone(), key: k.clone() });
                }
              },
              SetUpRule::CreatePointMap(pointmap, key_value_int_triples) => {
                for (k, v, _) in key_value_int_triples.iter() {
                  self.pointmaps.insert(pointmap.node.clone(), k.node.clone());
                  self.used_corr.push(UsedCorrespondence::Value { name: v.clone(), key: k.clone() });
                }
              },
              SetUpRule::CreateCardOnLocation(_, types) => {
                for (k, vs) in types.node.types.iter() {
                  for v in vs.iter() {
                    self.values.insert(v.node.clone(), k.node.clone());
                  }
                }
              },
              _ => {},
            }
          },
          NodeKind::AggregateFilter(a) => {
            match a {
                AggregateFilter::Adjacent(key, precedence) => {
                  self.used_corr.push(UsedCorrespondence::Precedence { name: precedence.clone(), key: key.clone() });
                },
                AggregateFilter::Higher(key, precedence) => {
                  self.used_corr.push(UsedCorrespondence::Precedence { name: precedence.clone(), key: key.clone() });
                },
                AggregateFilter::Lower(key, precedence) => {
                  self.used_corr.push(UsedCorrespondence::Precedence { name: precedence.clone(), key: key.clone() });
                },
                AggregateFilter::KeyString(key, _, string) => {
                  match &string.node {
                    StringExpr::Query(q) => {
                      match &q.node {
                        QueryString::KeyOf(key_string, _) => {
                          self.used_corr.push(UsedCorrespondence::String { name: key_string.clone(), key: key.clone() });
                        },
                        _ => {}
                      }
                    },
                    StringExpr::Literal(value) => {
                      self.used_corr.push(UsedCorrespondence::Value { name: value.clone(), key: key.clone() });
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