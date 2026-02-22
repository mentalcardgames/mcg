use std::{collections::HashMap, fmt};

use crate::{ast::ast_spanned::{NodeKind, *}, parser::MemType, spans::OwnedSpan, symbols::Var, walker::AstPass};

pub enum SemanticError {
  KeyNotFoundForType { ty: String, key: Var },
  NoCorrToType { ty: Var, key: Var },
  MemoryMismatch { memory: Var },
}

#[derive(Debug, PartialEq, Clone)]
pub struct UsedCorrespondence {
  pub ty: CorrespondanceType,
  pub key: String,
  pub span: OwnedSpan,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum CorrespondanceType {
  // Key Correspondance
  Precedence { node: String },
  PointMap   { node: String },
  Value      { node: String },
  Key        { node: String },
}

impl fmt::Display for MemType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let s = match self {
        MemType::Int => "Int",
        MemType::String => "String",
        MemType::PlayerCollection => "PlayerCollection",
        MemType::StringCollection => "StringCollection",
        MemType::IntCollection => "IntCollection",
        MemType::TeamCollection => "TeamCollection",
        MemType::LocationCollection => "LocationCollection",
        MemType::CardSet => "CardSet",
      };
      f.write_str(s)
  }
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
  used_corr: Vec<UsedCorrespondence>,
  memories: Vec<(String, (MemType, OwnedSpan))>
}

impl SemanticVisitor {
  pub fn new() -> Self {
    SemanticVisitor {
      init_corr: HashMap::new(),
      used_corr: Vec::new(),
      memories: Vec::new(),
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

    let memory_errs = self.find_memory_mismatches();

    err.extend(memory_errs);

    if err.is_empty() {
      return None
    }

    return Some(err)
  }


  fn find_memory_mismatches(&self) -> Vec<SemanticError> {
      let mut grouped: HashMap<String, Vec<(MemType, OwnedSpan)>> = HashMap::new();

      // Group by variable name
      for (name, (mem_type, span)) in self.memories.iter() {
          grouped
              .entry(name.clone())
              .or_default()
              .push((mem_type.clone(), span.clone()));
      }

      let mut mismatches: HashMap<String, Vec<OwnedSpan>> = HashMap::new();

      // Check each group
      for (name, occurrences) in grouped {
          if occurrences.is_empty() {
              continue;
          }

          // First occurrence defines the initialization type
          let (initial_type, _) = &occurrences[0];

          let mut wrong_spans = Vec::new();

          for (mem_type, span) in occurrences.iter().skip(1) {
              if mem_type != initial_type {
                  wrong_spans.push(span.clone());
              }
          }

          if !wrong_spans.is_empty() {
              mismatches.insert(name, wrong_spans);
          }
      }

      let mut errs = Vec::new();

      for (name, spans) in mismatches.iter() {
        for span in spans.iter() {
          errs.push(SemanticError::MemoryMismatch { memory: Var { id: name.clone(), span: span.clone() } });
        }
      }

      return errs
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
              SetUpRule::CreateMemoryWithMemoryType { memory, memory_type, owner } => {
                  self.memories.push((memory.node.clone(), (memory_type_to_mem_type(&memory_type.node), memory.span.clone())));
                  // TODO: Owner
              },
              SetUpRule::CreatePrecedence { precedence: precedence, kvs: key_value_pairs } => {
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
              SetUpRule::CreatePointMap { pointmap: pointmap, kvis: key_value_int_triples} => {
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
              SetUpRule::CreateCardOnLocation { location: _, cards } => {
                for types in cards.iter() {
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
                }
              },
              _ => {},
            }
          },
          NodeKind::StringExpr(s) => {
            match s {
              StringExpr::Memory { memory } => {
                match &memory.node {
                  UseMemory::Memory { memory: mem } => {
                    self.memories.push((mem.node.clone(), (MemType::String, mem.span.clone())));
                  },
                  UseMemory::WithOwner { memory: mem, owner } => {
                      self.memories.push((mem.node.clone(), (MemType::String, mem.span.clone())));

                    // match &owner.node {
                    //     Owner::PlayerCollection { player_collection } => {
                    //       self.memories.push((mem.node.clone(), (MemType::StringCollection, mem.span.clone())));
                    //     }
                    //     Owner::TeamCollection { team_collection } => {
                    //       self.memories.push((mem.node.clone(), (MemType::StringCollection, mem.span.clone())));
                    //     },
                    //     _ => {
                    //       self.memories.push((mem.node.clone(), (MemType::String, mem.span.clone())));
                    //     },
                    // }
                    // TODO check if owner is correct
                  },
                }
              },
              _ => {}
            }
          },
          NodeKind::IntExpr(s) => {
            match s {
              IntExpr::Memory { memory } => {
                match &memory.node {
                  UseMemory::Memory { memory: mem } => {
                    self.memories.push((mem.node.clone(), (MemType::Int, mem.span.clone())));
                  },
                  UseMemory::WithOwner { memory: mem, owner } => {
                    self.memories.push((mem.node.clone(), (MemType::Int, mem.span.clone())));

                    // match &owner.node {
                    //     Owner::PlayerCollection { player_collection } => {
                    //       self.memories.push((mem.node.clone(), (MemType::IntCollection, mem.span.clone())));
                    //     }
                    //     Owner::TeamCollection { team_collection } => {
                    //       self.memories.push((mem.node.clone(), (MemType::IntCollection, mem.span.clone())));
                    //     },
                    //     _ => {
                    //       self.memories.push((mem.node.clone(), (MemType::Int, mem.span.clone())));
                    //     },
                    // }
                    // TODO check if owner is correct
                  },
                }
              },
              _ => {}
            }
          },
          NodeKind::IntCollection(s) => {
            match s {
              IntCollection::Memory { memory } => {
                match &memory.node {
                  UseMemory::Memory { memory: mem } => {
                    self.memories.push((mem.node.clone(), (MemType::IntCollection, mem.span.clone())));
                  },
                  UseMemory::WithOwner { memory: mem, owner } => {
                    // self.memories.push((mem.node.clone(), (MemType::IntCollection, mem.span.clone())));

                    match &owner.node {
                        Owner::PlayerCollection { player_collection } => {
                          self.memories.push((mem.node.clone(), (MemType::Int, mem.span.clone())));
                        }
                        Owner::TeamCollection { team_collection } => {
                          self.memories.push((mem.node.clone(), (MemType::Int, mem.span.clone())));
                        },
                        _ => {
                          self.memories.push((mem.node.clone(), (MemType::IntCollection, mem.span.clone())));
                        },
                    }
                    // TODO check if owner is correct
                  },
                }
              },
              _ => {}
            }
          },
          NodeKind::StringCollection(s) => {
            match s {
              StringCollection::Memory { memory } => {
                match &memory.node {
                  UseMemory::Memory { memory: mem } => {
                    self.memories.push((mem.node.clone(), (MemType::StringCollection, mem.span.clone())));
                  },
                  UseMemory::WithOwner { memory: mem, owner } => {
                    // self.memories.push((mem.node.clone(), (MemType::StringCollection, mem.span.clone())));

                    match &owner.node {
                        Owner::PlayerCollection { player_collection } => {
                          self.memories.push((mem.node.clone(), (MemType::String, mem.span.clone())));
                        }
                        Owner::TeamCollection { team_collection } => {
                          self.memories.push((mem.node.clone(), (MemType::String, mem.span.clone())));
                        },
                        _ => {
                          self.memories.push((mem.node.clone(), (MemType::StringCollection, mem.span.clone())));
                        },
                    }
                    // TODO check if owner is correct
                  },
                }
              },
              _ => {}
            }
          },
          NodeKind::LocationCollection(s) => {
            match s {
              LocationCollection::Memory { memory } => {
                match &memory.node {
                  UseMemory::Memory { memory: mem } => {
                    self.memories.push((mem.node.clone(), (MemType::LocationCollection, mem.span.clone())));
                  },
                  UseMemory::WithOwner { memory: mem, owner } => {
                    self.memories.push((mem.node.clone(), (MemType::LocationCollection, mem.span.clone())));
                    // TODO check if owner is correct
                  },
                }
              },
              _ => {}
            }
          },
          NodeKind::PlayerCollection(s) => {
            match s {
              PlayerCollection::Memory { memory } => {
                match &memory.node {
                  UseMemory::Memory { memory: mem } => {
                    self.memories.push((mem.node.clone(), (MemType::PlayerCollection, mem.span.clone())));
                  },
                  UseMemory::WithOwner { memory: mem, owner } => {
                    self.memories.push((mem.node.clone(), (MemType::PlayerCollection, mem.span.clone())));
                    // TODO check if owner is correct
                  },
                }
              },
              _ => {}
            }
          },
          NodeKind::TeamCollection(s) => {
            match s {
              TeamCollection::Memory { memory } => {
                match &memory.node {
                  UseMemory::Memory { memory: mem } => {
                    self.memories.push((mem.node.clone(), (MemType::TeamCollection, mem.span.clone())));
                  },
                  UseMemory::WithOwner { memory: mem, owner } => {
                    self.memories.push((mem.node.clone(), (MemType::TeamCollection, mem.span.clone())));
                    // TODO check if owner is correct
                  },
                }
              },
              _ => {}
            }
          },
          NodeKind::CardSet(s) => {
            match s {
              CardSet::Memory { memory } => {
                match &memory.node {
                  UseMemory::Memory { memory: mem } => {
                    self.memories.push((mem.node.clone(), (MemType::CardSet, mem.span.clone())));
                  },
                  UseMemory::WithOwner { memory: mem, owner } => {
                    self.memories.push((mem.node.clone(), (MemType::CardSet, mem.span.clone())));
                    // TODO check if owner is correct
                  },
                }
              },
              _ => {}
            }
          },
          NodeKind::AggregateFilter(a) => {
            match a {
                AggregateFilter::Adjacent { key: key, precedence: precedence} => {
                  self.used_corr.push(
                    UsedCorrespondence { 
                      ty: CorrespondanceType::Precedence { node: precedence.node.clone() }, 
                      key: key.node.clone(), 
                      span: precedence.span.clone() 
                    }
                  );
                },
                AggregateFilter::Higher { key: key, value, precedence: precedence} => {
                  self.used_corr.push(
                    UsedCorrespondence { 
                      ty: CorrespondanceType::Precedence { node: precedence.node.clone() }, 
                      key: key.node.clone(), 
                      span: precedence.span.clone() 
                    }
                  );
                  match &value.node {
                    StringExpr::Query { query: q} => {
                      match &q.node {
                        QueryString::KeyOf { key: k, card_position: _} => {
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
                    StringExpr::Literal { value: value} => {
                      self.used_corr.push(
                        UsedCorrespondence { 
                          ty: CorrespondanceType::Value { node: value.node.clone() }, 
                          key: key.node.clone(), 
                          span: value.span.clone() 
                        }
                      );
                    },
                    _ => {}
                  }
                },
                AggregateFilter::Lower { key: key, value, precedence: precedence} => {
                  self.used_corr.push(
                    UsedCorrespondence { 
                      ty: CorrespondanceType::Precedence { node: precedence.node.clone() }, 
                      key: key.node.clone(), 
                      span: precedence.span.clone() 
                    }
                  );
                  match &value.node {
                    StringExpr::Query { query: q} => {
                      match &q.node {
                        QueryString::KeyOf { key: k, card_position: _} => {
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
                    StringExpr::Literal { value: value} => {
                      self.used_corr.push(
                        UsedCorrespondence { 
                          ty: CorrespondanceType::Value { node: value.node.clone() }, 
                          key: key.node.clone(), 
                          span: value.span.clone() 
                        }
                      );
                    },
                    _ => {}
                  }
                },
                AggregateFilter::KeyIsString { key: key, string: string} => {
                  match &string.node {
                    StringExpr::Query { query: q} => {
                      match &q.node {
                        QueryString::KeyOf { key: k, card_position: _} => {
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
                    StringExpr::Literal { value: value} => {
                      self.used_corr.push(
                        UsedCorrespondence { 
                          ty: CorrespondanceType::Value { node: value.node.clone() }, 
                          key: key.node.clone(), 
                          span: value.span.clone() 
                        }
                      );
                    },
                    StringExpr::Memory { memory } => {
                      /* TODO */
                    },
                  }
                },
                AggregateFilter::KeyIsNotString { key: key, string: string} => {
                  match &string.node {
                    StringExpr::Query { query: q} => {
                      match &q.node {
                        QueryString::KeyOf { key: k, card_position: _} => {
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
                    StringExpr::Literal { value: value} => {
                      self.used_corr.push(
                        UsedCorrespondence { 
                          ty: CorrespondanceType::Value { node: value.node.clone() }, 
                          key: key.node.clone(), 
                          span: value.span.clone() 
                        }
                      );
                    },
                    _ => {}
                  }
                },
                _ => {}
            }
          },
          NodeKind::ActionRule(a) => {
            match a {
              ActionRule::SetMemory { memory, memory_type } => {
                self.memories.push((memory.node.clone(), (memory_type_to_mem_type(&memory_type.node), memory.span.clone())));
              },
              ActionRule::BidMemoryAction { memory, quantity, owner } => {
                self.memories.push((memory.node.clone(), (MemType::Int, memory.span.clone())));
              },
              _ => {}
            }
          },
          NodeKind::ScoreRule(s) => {
            match s {
                ScoreRule::ScoreMemory { int, memory, players } => {
                  self.memories.push((memory.node.clone(), (MemType::Int, memory.span.clone())));
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


fn memory_type_to_mem_type(mt: &MemoryType) -> MemType {
  match mt {
    MemoryType::Int { int } => MemType::Int,
    MemoryType::String { string } => MemType::String,
    MemoryType::PlayerCollection { players } => MemType::PlayerCollection,
    MemoryType::StringCollection { strings } => MemType::StringCollection,
    MemoryType::TeamCollection { teams } => MemType::TeamCollection,
    MemoryType::IntCollection { ints } => MemType::IntCollection,
    MemoryType::LocationCollection { locations } => MemType::LocationCollection,
    MemoryType::CardSet { card_set } => MemType::CardSet,
  }
}