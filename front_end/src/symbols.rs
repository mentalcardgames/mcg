use std::collections::HashMap;

use crate::ast::ast_spanned::*;
use crate::spans::*;
use crate::walker::AstPass;
use crate::ast::ast_spanned::NodeKind as NodeKind;
use crate::walker::Walker;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GameType {
  Player,
  Team,
  Location,
  Precedence,
  PointMap,
  Combo,
  Key,
  Value,
  Memory,
  Token,
  Stage,
  NoType,
}

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

    pub fn name_resolution(&self) -> TypedVars {
        // Build lookup: String -> GameType
        let string_to_type: HashMap<String, GameType> = self.symbols
            .iter()
            .filter(|(_, v)| **v != GameType::NoType)
            .map(|(s, t)| (s.node.clone(), t.clone()))
            .collect();

        self.symbols
            .iter()
            .map(|(k, _)| {
                let ty = string_to_type
                    .get(&k.node)
                    .cloned()
                    .unwrap_or(GameType::NoType);

                (Var::from(k.clone()), ty)
            })
            .collect()
    }

    pub fn type_to_variable(&mut self) -> HashMap<GameType, Vec<String>> {
        let typed_vars: Vec<(Var, GameType)> = self.into_typed_vars();
        let mut map: HashMap<GameType, Vec<String>> = HashMap::new();

        for (var, game_type) in typed_vars {
            // .entry() handles the case where the GameType isn't in the map yet
            map.entry(game_type)
                .or_insert_with(Vec::new)
                .push(var.id); // Assuming var is the String name
        }

        map
    }
}

impl AstPass for SymbolVisitor {
    fn enter_node<T: Walker>(&mut self, node: &T)
    where
        Self: Sized,
    {
      if let Some(unwrapped) = node.kind() {
        match unwrapped {
            NodeKind::OutOf(o) => match o {
                OutOf::Stage { name: spanned } => {
                    self.use_id(&spanned);
                },
                _ => {},
            },
            NodeKind::Groupable(g) => match g {
                Groupable::Location { name: spanned } => {
                    self.use_id(&spanned);
                },
                _ => {}
            },
            NodeKind::Types(t) => {
                for (k, vs) in t.types.iter() {
                    self.init_id(&k, GameType::Key);
                    for v in vs.iter() {
                        self.init_id(&v, GameType::Value);
                    }
                }
            },
            NodeKind::AggregatePlayer(a) => {
                match a {
                    AggregatePlayer::OwnerOfMemory { extrema: _, memory: spanned1} => {
                        self.use_id(&spanned1);
                    },
                    _ => {},
                }
            },
            NodeKind::PlayerExpr(p) => {
                match p {
                    PlayerExpr::Literal { name: spanned } => {
                        self.use_id(&spanned);
    
                    },
                    _ => {},
                }
            },
            NodeKind::AggregateInt(a) => {
                match a {
                    AggregateInt::SumOfCardSet{ card_set: _, pointmap: spanned1} => {
                        self.use_id(&spanned1);
                    },
                    AggregateInt::ExtremaCardset { extrema: _, card_set: _, pointmap: spanned2} => {
                        self.use_id(&spanned2);
                    },
                    _ => {},
                }
            },
            NodeKind::QueryString(q) => {
                match q {
                    QueryString::KeyOf { key: spanned, card_position: _} => {
                        self.use_id(&spanned);
                    },
                    _ => {}
                }
            },
            NodeKind::StringExpr(s) => {
                match s {
                    StringExpr::Literal { value: spanned} => {
                        self.use_id(&spanned);
                    },
                    _ => {},
                }
            },
            NodeKind::TeamExpr(t) => {
                match t {
                    TeamExpr::Literal { name: spanned } => {
                        self.use_id(&spanned);
                    },
                    _ => {},
                }
            },
            NodeKind::QueryCardPosition(q) => {
                match q {
                    QueryCardPosition::At { location: spanned, int_expr: _} => {
                        self.use_id(&spanned);
                    },
                    QueryCardPosition::Top { location: spanned} => {
                        self.use_id(&spanned);
                    },
                    QueryCardPosition::Bottom { location: spanned} => {
                        self.use_id(&spanned);
                    },
                }
            },
            NodeKind::AggregateCardPosition(a) => {
                match a {
                    AggregateCardPosition::ExtremaPointMap{extrema: _, card_set: _, pointmap: spanned2} => {
                        self.use_id(&spanned2);
                    },
                    AggregateCardPosition::ExtremaPrecedence{extrema: _, card_set: _, precedence: spanned2} => {
                        self.use_id(&spanned2);
                    },
                }
            },
            NodeKind::LocationCollection(l) => {
                match l {
                    LocationCollection::Literal { locations } => {
                        for location in locations.iter() {
                            self.use_id(&location);
                        }
                    },
                    LocationCollection::Memory { memory } => {
                        /* TODO */
                    },
                }
            },
            NodeKind::Group(g) => {
                match g {
                    Group::NotCombo{combo: spanned, groupable: _} => {
                        self.use_id(&spanned);
                    },
                    Group::Combo{combo: spanned, groupable: _} => {
                        self.use_id(&spanned);
                    },
                    _ => {},
                }
            },
            NodeKind::AggregateFilter(a) => {
                match a {
                    AggregateFilter::Same{key:spanned} => {
                        self.use_id(&spanned);
                    },
                    AggregateFilter::Distinct{key:spanned} => {
                        self.use_id(&spanned);
                    },
                    AggregateFilter::Adjacent{key: spanned, precedence: spanned1} => {
                        self.use_id(&spanned);
                        self.use_id(&spanned1);
                    },
                    AggregateFilter::Higher{key: spanned, value: _, precedence: spanned1} => {
                        self.use_id(&spanned);
                        self.use_id(&spanned1);
                    },
                    AggregateFilter::Lower{key: spanned, value: _, precedence: spanned1} => {
                        self.use_id(&spanned);
                        self.use_id(&spanned1);
                    },
                    AggregateFilter::KeyString{key: spanned, cmp: _, string: _} => {
                        self.use_id(&spanned);
                    },
                    AggregateFilter::Combo{combo: spanned} => {
                        self.use_id(&spanned);
                    },
                    AggregateFilter::NotCombo{combo: spanned} => {
                        self.use_id(&spanned);
                    },
                    _ => {}
                }
            },
            NodeKind::SetUpRule(s) => {
                match s {
                    SetUpRule::CreatePlayer{players: spanneds} => {
                        for s in spanneds.iter() {
                            self.init_id(&s, GameType::Player);
                        } 
                    },
                    SetUpRule::CreateTeams{teams: items} => {
                        for (t, _) in items.iter() {
                            self.init_id(&t, GameType::Team);
                        }
                    },
                    SetUpRule::CreateLocation{locations: spanneds, owner: _} => {
                        for s in spanneds.iter() {
                            self.init_id(&s, GameType::Location);
                        }
                    },
                    SetUpRule::CreateCardOnLocation{location: spanned, types: _} => {
                        self.use_id(&spanned);
                    },
                    SetUpRule::CreateTokenOnLocation{int: _, token: spanned1, location: spanned2} => {
                        self.init_id(&spanned1, GameType::Token);
                        self.use_id(&spanned2);
                    },
                    SetUpRule::CreateCombo{combo: spanned, filter: _} => {
                        self.init_id(&spanned, GameType::Combo);
                    },
                    SetUpRule::CreateMemory{memory: spanned, owner: _} => {
                        self.init_id(&spanned, GameType::Memory);
                    },
                    SetUpRule::CreateMemoryWithMemoryType{memory: spanned, memory_type: _, owner: _} => {
                        self.init_id(&spanned, GameType::Memory);
                    },
                    SetUpRule::CreatePrecedence{precedence: spanned, kvs: items} => {
                        self.init_id(&spanned, GameType::Precedence);
                        for (k, v) in items.iter() {
                            self.use_id(&k);
                            self.use_id(&v);
                        }
                    },
                    SetUpRule::CreatePointMap{pointmap: spanned, kvis: items} => {
                        self.init_id(&spanned, GameType::PointMap);
                        for (k, v, _) in items.iter() {
                            self.use_id(&k);
                            self.use_id(&v);
                        }
                    },
                    _ => {}
                }
            },
            NodeKind::ActionRule(a) => {
                match a {
                    ActionRule::SetMemory{memory: spanned, memory_type: _} => {
                        self.use_id(&spanned);
                    },
                    ActionRule::ResetMemory{memory: spanned} => {
                        self.use_id(&spanned);
                    },
                    ActionRule::BidMemoryAction{ memory: spanned, quantity: _, owner: _} => {
                        self.use_id(&spanned);
                    },
                    _ => {}
                }
            },
            NodeKind::SeqStage(s) => {
                self.init_id(&s.stage, GameType::Stage);
            },
            NodeKind::TokenMove(t) => {
                match t {
                    TokenMove::Place { token: spanned, from_loc:  _, to_loc:  _} => {
                        self.use_id(&spanned);
                    },
                    TokenMove::PlaceQuantity{quantity: _, token: spanned1, from_loc: _, to_loc: _} => {
                        self.use_id(&spanned1);
                    },
                }
            },
            NodeKind::ScoreRule(s) => {
                match s {
                    ScoreRule::ScoreMemory{int: _, memory: spanned1, players: _} => {
                        self.use_id(&spanned1);
                    },
                    _ => {}
                }
            },
            NodeKind::WinnerType(w) => {
                match w {
                    WinnerType::Memory { memory: spanned} => {
                        self.use_id(&spanned);
                    },
                    _ => {},
                }
            }
            _ => {}
        }
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