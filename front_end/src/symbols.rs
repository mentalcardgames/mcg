use std::collections::HashMap;

use crate::spanned_ast::*;
use crate::spans::*;
use crate::walker::AstPass;
use crate::walker::NodeKind;
use crate::walker::Walker;
use crate::{ast::GameType};

#[derive(Debug, Clone)]
pub enum SymbolError {
    NotInitialized {
        var: Var,
    },
    DefinedMultipleTimes {
        var: Var,
    },
}

pub struct SymbolVisitor {
    symbols: HashMap<SID, GameType>,
}

impl SymbolVisitor {
    pub fn new() -> Self {
        SymbolVisitor { symbols: HashMap::new() }
    }
    fn use_id(&mut self, id: &SID) {
        self.symbols.insert(id.clone(), GameType::NoType);
    }

    fn init_id(&mut self, id: &SID, game_type: GameType) {
        self.symbols.insert(id.clone(), game_type);
    }

    pub fn check_game_type(&self) -> Option<Vec<SymbolError>> {
        let mut errs = Vec::new();

        // 1. Group by the variable name (String)
        // Map: "var_name" -> Vec<(SID, GameType)>
        let mut groups: HashMap<String, Vec<(&SID, &GameType)>> = HashMap::new();

        for (sid, gtype) in &self.symbols {
            groups
                .entry(sid.node.clone())
                .or_default()
                .push((sid, gtype));
        }

        // Process each group
        for (_, occurrences) in groups {
            // 2. Check if a Group only has NoType
            let all_notype = occurrences.iter().all(|(_, t)| **t == GameType::NoType);
            if all_notype {
                // If the group only has NoType, it was used but never initialized
                for (sid, _) in occurrences.clone() {
                    errs.push(SymbolError::NotInitialized {
                        var: Var::from(sid.clone()),
                    });
                }
            }

            // 3. Check if multiple concrete types are assigned to the same name
            // Filter out NoType to see what concrete types were assigned
            let concrete_assignments: Vec<_> = occurrences
                .iter()
                .filter(|(_, t)| **t != GameType::NoType)
                .collect();

            if concrete_assignments.len() > 1 {
                for (sid, _) in concrete_assignments {
                    // 4. Return the SID with multiple definitions
                    errs.push(SymbolError::DefinedMultipleTimes { var: Var::from((*sid).clone()) });
                }
            }
        }

        if errs.is_empty() {
            return None
        }

        // 5. Else return Ok
        Some(errs)
    }

    pub fn into_typed_vars(&mut self) -> TypedVars {
        self.symbols
            .iter()
            .filter(|(_, v)| **v != GameType::NoType)
            .map(|(k, v)| (Var::from(k.clone()), v.clone()))
            .collect::<TypedVars>()
    }
}

impl AstPass for SymbolVisitor {
    fn enter_node<T: Walker>(&mut self, node: &T)
    where
        Self: Sized,
    {
        match node.kind() {
            NodeKind::OutOf(o) => match o {
                OutOf::Stage(spanned) => {
                    self.use_id(spanned);
                },
                _ => {},
            },
            NodeKind::Groupable(g) => match g {
                Groupable::Location(spanned) => {
                    self.use_id(spanned);
                },
                _ => {}
            },
            NodeKind::Types(t) => {
                for (k, vs) in t.types.iter() {
                    self.init_id(k, GameType::Key);
                    for v in vs.iter() {
                        self.init_id(v, GameType::Value);
                    }
                }
            },
            NodeKind::AggregatePlayer(a) => {
                match a {
                    AggregatePlayer::OwnerOfMemory(_, spanned1) => {
                        self.use_id(spanned1);
                    },
                    _ => {},
                }
            },
            NodeKind::PlayerExpr(p) => {
                match p {
                    PlayerExpr::Literal(spanned) => {
                        self.use_id(spanned);
    
                    },
                    _ => {},
                }
            },
            NodeKind::AggregateInt(a) => {
                match a {
                    AggregateInt::SumOfCardSet(_, spanned1) => {
                        self.use_id(spanned1);
                    },
                    AggregateInt::ExtremaCardset(_, _, spanned2) => {
                        self.use_id(spanned2);
                    },
                    _ => {},
                }
            },
            NodeKind::QueryString(q) => {
                match q {
                    QueryString::KeyOf(spanned, _) => {
                        self.use_id(spanned);
                    },
                    _ => {}
                }
            },
            NodeKind::StringExpr(s) => {
                match s {
                    StringExpr::Literal(spanned) => {
                        self.use_id(spanned);
                    },
                    _ => {},
                }
            },
            NodeKind::TeamExpr(t) => {
                match t {
                    TeamExpr::Literal(spanned) => {
                        self.use_id(spanned);
                    },
                    _ => {},
                }
            },
            NodeKind::QueryCardPosition(q) => {
                match q {
                    QueryCardPosition::At(spanned, _) => {
                        self.use_id(spanned);
                    },
                    QueryCardPosition::Top(spanned) => {
                        self.use_id(spanned);
                    },
                    QueryCardPosition::Bottom(spanned) => {
                        self.use_id(spanned);
                    },
                }
            },
            NodeKind::AggregateCardPosition(a) => {
                match a {
                    AggregateCardPosition::ExtremaPointMap(_, _, spanned2) => {
                        self.use_id(spanned2);
                    },
                    AggregateCardPosition::ExtremaPrecedence(_, _, spanned2) => {
                        self.use_id(spanned2);
                    },
                }
            },
            NodeKind::LocationCollection(l) => {
                for location in l.locations.iter() {
                    self.use_id(location);
                }
            },
            NodeKind::Group(g) => {
                match g {
                    Group::NotCombo(spanned, _) => {
                        self.use_id(spanned);
                    },
                    Group::Combo(spanned, _) => {
                        self.use_id(spanned);
                    },
                    _ => {},
                }
            },
            NodeKind::AggregateFilter(a) => {
                match a {
                    AggregateFilter::Same(spanned) => {
                        self.use_id(spanned);
                    },
                    AggregateFilter::Distinct(spanned) => {
                        self.use_id(spanned);
                    },
                    AggregateFilter::Adjacent(spanned, spanned1) => {
                        self.use_id(spanned);
                        self.use_id(spanned1);
                    },
                    AggregateFilter::Higher(spanned, spanned1) => {
                        self.use_id(spanned);
                        self.use_id(spanned1);
                    },
                    AggregateFilter::Lower(spanned, spanned1) => {
                        self.use_id(spanned);
                        self.use_id(spanned1);
                    },
                    AggregateFilter::KeyString(spanned, _, _)=> {
                        self.use_id(spanned);
                    },
                    AggregateFilter::Combo(spanned) => {
                        self.use_id(spanned);
                    },
                    AggregateFilter::NotCombo(spanned) => {
                        self.use_id(spanned);
                    },
                    _ => {}
                }
            },
            NodeKind::SetUpRule(s) => {
                match s {
                    SetUpRule::CreatePlayer(spanneds) => {
                        for s in spanneds.iter() {
                            self.init_id(s, GameType::Player);
                        } 
                    },
                    SetUpRule::CreateTeams(items) => {
                        for (t, _) in items.iter() {
                            self.init_id(t, GameType::Team);
                        }
                    },
                    SetUpRule::CreateLocation(spanneds, _) => {
                        for s in spanneds.iter() {
                            self.init_id(s, GameType::Location);
                        }
                    },
                    SetUpRule::CreateCardOnLocation(spanned, _) => {
                        self.use_id(spanned);
                    },
                    SetUpRule::CreateTokenOnLocation(_, spanned1, spanned2) => {
                        self.init_id(spanned1, GameType::Token);
                        self.use_id(spanned2);
                    },
                    SetUpRule::CreateCombo(spanned, _) => {
                        self.init_id(spanned, GameType::Combo);
                    },
                    SetUpRule::CreateMemory(spanned, _) => {
                        self.init_id(spanned, GameType::Memory);
                    },
                    SetUpRule::CreateMemoryWithMemoryType(spanned, _, _) => {
                        self.init_id(spanned, GameType::Memory);
                    },
                    SetUpRule::CreatePrecedence(spanned, items) => {
                        self.init_id(spanned, GameType::Precedence);
                        for (k, v) in items.iter() {
                            self.use_id(k);
                            self.use_id(v);
                        }
                    },
                    SetUpRule::CreatePointMap(spanned, items) => {
                        self.init_id(spanned, GameType::PointMap);
                        for (k, v, _) in items.iter() {
                            self.use_id(k);
                            self.use_id(v);
                        }
                    },
                    _ => {}
                }
            },
            NodeKind::ActionRule(a) => {
                match a {
                    ActionRule::SetMemory(spanned, _) => {
                        self.use_id(spanned);
                    },
                    ActionRule::ResetMemory(spanned) => {
                        self.use_id(spanned);
                    },
                    ActionRule::BidMemoryAction(spanned, _) => {
                        self.use_id(spanned);
                    },
                    _ => {}
                }
            },
            NodeKind::SeqStage(s) => {
                self.init_id(&s.stage, GameType::Stage);
            },
            NodeKind::TokenMove(t) => {
                match t {
                    TokenMove::Place(spanned, _, _) => {
                        self.use_id(spanned);
                    },
                    TokenMove::PlaceQuantity(_, spanned1, _, _) => {
                        self.use_id(spanned1);
                    },
                }
            },
            NodeKind::ScoreRule(s) => {
                match s {
                    ScoreRule::ScoreMemory(_, spanned1, _) => {
                        self.use_id(spanned1);
                    },
                    _ => {}
                }
            },
            NodeKind::WinnerType(w) => {
                match w {
                    WinnerType::Memory(spanned) => {
                        self.use_id(spanned);
                    },
                    _ => {},
                }
            }
            _ => {}
        }
    }

    fn exit_node<T: Walker>(&mut self, _: &T)
    where
        Self: Sized,
    {
        
    }
}

#[derive(Debug, Clone)]
pub struct Var {
  pub id: String,
  pub span: OwnedSpan,
}

impl From<SID> for Var {
    fn from(value: SID) -> Self {
      Var {
        id: value.node,
        span: value.span
      }
    }
}

impl PartialEq for Var {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub type TypedVars = Vec<(Var, GameType)>;