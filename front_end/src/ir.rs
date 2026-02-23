use std::{collections::{HashMap, HashSet, VecDeque}, fmt::Debug};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::ast as L;
use crate::ast::ast_spanned::*;
use crate::{lower::Lower, spans::*};

// ===========================================================================
// Implement transform to Ir from AST
// ===========================================================================
impl SGame {
  pub fn to_graph(&self) -> Ir<SpannedPayload> {
    let mut builder: IrBuilder<SpannedPayload> = IrBuilder::default();
    builder.build_ir(self);

    return builder.fsm
  }

  pub fn to_lowered_graph(&self) -> Ir<LoweredPayLoad> {
    Ir::from(self.to_graph())
  }
}

// ===========================================================================
// Ir-Definition
// ===========================================================================
pub type Stage = String;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateID(u32);

impl StateID {
    pub fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")] // Tell Serde how to handle the generic
pub struct Edge<T>
  where T: serde::Serialize
{
  pub to: StateID,
  pub payload: T,
  pub meta: Option<Vec<Meta>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Meta {
  SimStageEndCondition { stage: Stage, end_condition: EndCondition, players: PlayerCollection }
  // Add new Meta-Information here
  // -------------------------------------
  
  // -------------------------------------
}

// ===========================================================================
// Ir-Logic
// ===========================================================================
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")] // Tell Serde how to handle the generic
pub struct Ir<T: serde::Serialize>
{
  pub states: HashMap<StateID, Vec<Edge<T>>>,
  pub entry: StateID,
  pub goal: StateID,
}

impl<T: Serialize + DeserializeOwned> Default for Ir<T> {
  fn default() -> Self {
      Ir {
        states: HashMap::new(),
        entry: StateID(0),
        goal: StateID(0)
      }
  }
}

impl<T: Serialize + DeserializeOwned> Ir<T> {
  /// Both States need to be added before the edge can be added.
  pub fn add_edge(&mut self, from: StateID, to: StateID, payload: T, meta: Option<Vec<Meta>>) {
    let edge = Edge { to: to, payload: payload, meta };
    let vec = self.states
      .get_mut(&from)
      .expect("StateID was not added before!");

    vec.push(edge);
  }

  /// Add state.
  pub fn add_state(&mut self, state: StateID) {
    self.states.insert(state, Vec::new());
  }

  /// Remove state.
  pub fn remove_state(&mut self, state: StateID) {
    self.states.remove(&state);
  }

  pub fn reachable_from_entry(&self) -> HashSet<StateID>
    where
        StateID: Eq + std::hash::Hash + Copy,
    {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(self.entry);
        visited.insert(self.entry);

        while let Some(current) = queue.pop_front() {
            if let Some(edges) = self.states.get(&current) {
                for edge in edges {
                    let target = edge.to;
                    if visited.insert(target) {
                        queue.push_back(target);
                    }
                }
            }
        }

        visited
    }

    pub fn is_connected(&self) -> bool
    where
        StateID: Eq + std::hash::Hash + Copy,
    {
        let reachable = self.reachable_from_entry();
        reachable.len() == self.states.len()
    }

    pub fn edges_to_highest_state_sub_graph(&self) -> Vec<&Edge<T>>
    where
        StateID: Ord + Copy,
    {
        let Some(max_state) = self.reachable_from_entry().into_iter().max() else {
            return Vec::new();
        };

        self.states
            .values()
            .flat_map(|edges| edges.iter())
            .filter(|edge| edge.to == max_state)
            .collect()
    }
}

// ===========================================================================
// Diagnositics
// ===========================================================================
impl Ir<SpannedPayload> {
  pub fn diagnostics(&self) -> Option<Vec<GameFlowError>> {
    if !self.is_connected() {
      let mut errs: Vec<GameFlowError> = Vec::new();
      for edge in self.edges_to_highest_state_sub_graph().iter() {
        match &edge.payload {
            Payload::Condition { expr, negated: _ } => {
              errs.push(GameFlowError::FlowNotConnected { span: expr.span.clone() });
            },
            Payload::EndCondition { expr, negated: _ } => {
              errs.push(GameFlowError::FlowNotConnected { span: expr.span.clone() });
            },
            Payload::Action(a) => {
              errs.push(GameFlowError::FlowNotConnected { span: a.span.clone() });
            },
            Payload::StageRoundCounter(stage) => {
              errs.push(GameFlowError::FlowNotConnected { span: stage.span.clone() });
            },
            Payload::EndStage(stage) => {
              errs.push(GameFlowError::FlowNotConnected { span: stage.span.clone() });
            },
            _ => {},
        }
      }

      if errs.is_empty() {
        return Some(vec![GameFlowError::FlowNotConnectedWithControl])
      }

      return Some(errs)
    }

    return None
  }
}

// ===========================================================================
// Return-Type
// ===========================================================================
/// There are certain Rules that alter the flow of the game:
/// - End Stage
/// - End Game
/// There might be added more rules that alter the flow of the game.
/// These rules need careful handling for constructing the IR.
#[derive(Debug, Serialize, Deserialize)]
pub enum GameFlowChange {
  EndCurrentStage(u32),
  EndStage(u32),
  EndGame(u32),
  None(u32),
}

// ===========================================================================
// Payload
// ===========================================================================
pub trait AstContext {
    type Condition: Serialize + DeserializeOwned + Debug + Clone;
    type EndCondition: Serialize + DeserializeOwned + Debug + Clone;
    type GameRule: Serialize + DeserializeOwned + Debug + Clone;
    type Id: Serialize + DeserializeOwned + Debug + Clone;
}

/// Each Transition/Edge needs to have some guard/payload.
/// E.g. If we have a condition then the edge's payload is proving the condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "Ctx: Serialize + DeserializeOwned")] // Tell Serde how to handle the generic
pub enum Payload<Ctx: AstContext>
{
  Condition { expr: Ctx::Condition, negated: bool },
  EndCondition { expr: Ctx::EndCondition, negated: bool },
  Action(Ctx::GameRule),
  StageRoundCounter(Ctx::Id),
  EndStage(Ctx::Id),
  Choice,
  Optional,
  Trigger,
}

impl<Ctx: AstContext> Payload<Ctx> {
    pub fn to_string(&self) -> String {
        let result = match &self {
            Payload::Condition { expr: _, negated } => {
              if *negated {"Not Condition"} else {"Condition"}
            },
            Payload::EndCondition { expr: _, negated } => {
              if *negated {"Not EndCondition"} else {"EndCondition"}
            },
            Payload::Action(_) => "Action",
            Payload::StageRoundCounter(_) => {
              &format!("Stage Round Counter")
            },
            Payload::EndStage(_) => {
              &format!("End Counter")
            },
            Payload::Choice => {
              "Choice"
            },
            Payload::Optional => {
              "Optional"
            },
            Payload::Trigger => {
              "Trigger"
            },
        };

        result.to_string()
    }
}


pub type SpannedPayload = Payload<SpannedCtx>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SpannedCtx;
impl AstContext for SpannedCtx {
    type Condition = SBoolExpr;
    type EndCondition = SEndCondition;
    type GameRule = SGameRule;
    type Id = SID;
}


pub type LoweredPayLoad = Payload<LoweredCtx>;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoweredCtx;
impl AstContext for LoweredCtx {
    type Condition = L::BoolExpr;
    type EndCondition = L::EndCondition;
    type GameRule = L::GameRule;
    type Id = String;
}

// ===========================================================================
// Error
// ===========================================================================
#[derive(Debug, Serialize, Deserialize)]
pub enum GameFlowError {
  Unreachable { span: OwnedSpan },
  NoStageToEnd { span: OwnedSpan },
  FlowNotConnected { span: OwnedSpan },
  FlowNotConnectedWithControl
}

// ===========================================================================
// Lower
// ===========================================================================
impl From<Ir<SpannedPayload>> for Ir<LoweredPayLoad> {
    fn from(value: Ir<SpannedPayload>) -> Self {
        let mut lowered_ir: Ir<LoweredPayLoad> = Ir::default();
        lowered_ir.entry = value.entry;
        lowered_ir.goal = value.goal;
        lowered_ir.states = value.states.into_iter().map(|(s, es)| 
        {
          let mut edges = Vec::new();
          for e in es.iter() {
            let lowered_payload: LoweredPayLoad = match &e.payload {
              Payload::Condition { expr, negated } => {
                Payload::Condition { expr: expr.lower(), negated: *negated }
              },
              Payload::EndCondition { expr, negated } => {
                Payload::EndCondition { expr: expr.lower(), negated: *negated }
              },
              Payload::Action(a) => {
                Payload::Action(a.lower())
              },
              Payload::StageRoundCounter(s) => {
                Payload::StageRoundCounter(s.lower())
              },
              Payload::EndStage(s) => {
                Payload::EndStage(s.lower())
              },
              Payload::Choice => Payload::Choice,
              Payload::Optional => Payload::Optional,
              Payload::Trigger => Payload::Trigger,
            };
          
            edges.push(Edge { to: e.to,  payload: lowered_payload, meta: None});
          }

          (s, edges)
        }
      ).collect();

      return lowered_ir;
    }
}


// ===========================================================================
// Builder
// ===========================================================================
/// fsm: The current IR being constructed.
/// stage_exits: Keeping track of stage_exits
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct IrBuilder<T: serde::Serialize> {
  pub fsm: Ir<T>,
  state_counter: u32,
  stage_exits: Vec<u32>,
  stage_to_exit: HashMap<String, u32>,
  pub diagnostics: Vec<GameFlowError>, 
}

impl<T: Serialize + DeserializeOwned> Default for IrBuilder<T> {
  fn default() -> Self {
    IrBuilder {
      fsm: Ir::default(),
      state_counter: 0,
      stage_exits: Vec::new(),
      stage_to_exit: HashMap::new(),
      diagnostics: Vec::new()
    }
  }
}

impl IrBuilder<SpannedPayload> {
  /// Increments the state_counter.
  /// Adds the the new state (id of state == state_counter) to the FSM.
  fn new_state(&mut self) -> u32 {
    self.state_counter += 1;
    self.fsm.add_state(StateID(self.state_counter));

    return self.state_counter
  }

  /// Decrements the state_counter.
  /// Removes the new state (id of state == state_counter) to the FSM.
  fn remove_state(&mut self, state: u32) -> u32 {
    self.state_counter -= 1;
    self.fsm.remove_state(StateID(state));

    return state
  }

  /// Adds edge to the FSM.
  fn new_edge(&mut self, from: u32, to: u32, payload: SpannedPayload, meta: Option<Vec<Meta>>) {
    self.fsm.add_edge(
      StateID(from),
      StateID(to),
      payload,
      meta
    );
  }

  fn unreachable(&mut self, flows: &[SFlowComponent]) {
    if !flows.is_empty() {
      for f in flows.iter() {
        self.diagnostics.push(
          GameFlowError::Unreachable { span: f.span.clone() }
        );
      }
    }
  }

  /// Builds FSM.
  /// Initializes the first state and then continues with the building of the FlowComponent's
  pub fn build_ir(&mut self, game: &SGame) {
    // Initialize entry
    let entry = 0;
    self.fsm.add_state(StateID(entry));

    // Initialize goal
    let goal = self.new_state();
    self.fsm.goal = StateID(goal);

    // Build IR
    self.build_flows(&game.node.flows, entry, goal);
  }

  /// Takes a Vector of FlowComponent's and extends the FSM with them.
  /// Returns a GameFlowChangeType because it needs to be handled by certain components.
  fn build_flows(&mut self, flows: &Vec<SFlowComponent>, entry: u32, exit: u32) -> GameFlowChange {
    let mut next_entry = entry;
    let mut flow_exit;
    for i in 0..flows.len() {
      if i == flows.len() - 1 {
        // Return the last node with its GameFlowChange-type
        return self.build_flow(&flows[i], next_entry, exit);
      }

      flow_exit = self.new_state();
      match self.build_flow(&flows[i], next_entry, flow_exit) {
        GameFlowChange::EndCurrentStage(stage) => {
          // The flow_exit was not used, so remove it
          self.remove_state(flow_exit);

          // Check for unreachable code
          self.unreachable(&flows[(i+1)..]);

          // Every FlowComponent after this will not be evaluated 
          return GameFlowChange::EndCurrentStage(stage)
        },
        GameFlowChange::EndStage(stage) => {
          // The flow_exit was not used, so remove it
          self.remove_state(flow_exit);

          // Check for unreachable code
          self.unreachable(&flows[(i+1)..]);

          // Every FlowComponent after this will not be evaluated 
          return GameFlowChange::EndStage(stage)
        },
        GameFlowChange::EndGame(game) => {
          // The flow_exit was not used, so remove it
          self.remove_state(flow_exit);

          // Check for unreachable code
          self.unreachable(&flows[(i+1)..]);

          // Every FlowComponent after this will not be evaluated 
          return GameFlowChange::EndGame(game)
        },
        GameFlowChange::None(none) => {
          // No GameFlowChange -> continue building
          next_entry = none;
        },
      }
    }

    return GameFlowChange::None(exit)
  }

  /// Builds a singular FlowComponent.
  fn build_flow(&mut self, flow: &SFlowComponent, entry: u32, exit: u32) -> GameFlowChange {
    let exit = match &flow.node {
        FlowComponent::ChoiceRule { choice_rule } => {
            self.build_choice_rule(&choice_rule.node, entry, exit)
        },
        FlowComponent::SeqStage { stage } => {
            self.build_seq_stage(&stage.node, entry, exit)
        },
        FlowComponent::SimStage { stage } => {
            // TODO:
            // Place holde
            // Function is just seq stage
            self.build_sim_stage(&stage.node, entry, exit)
            
        },
        FlowComponent::Rule {game_rule} => {
            // Can have GameFlowChanges! So return here.
            return self.build_rule(game_rule, entry, exit)
        },
        FlowComponent::IfRule {if_rule} => {
            self.build_if_rule(&if_rule.node, entry, exit)
        },
        FlowComponent::OptionalRule { optional_rule} => {
            self.build_optional_rule(&optional_rule.node, entry, exit)
        },
        FlowComponent::TriggerRule { trigger_rule} => {
            self.build_trigger_rule(&trigger_rule.node, entry, exit)
        },
        FlowComponent::Conditional {conditional} => {
            self.build_cond_rule(&conditional.node, entry, exit)
        },
    };

    return GameFlowChange::None(exit)
  }

  fn build_choice_rule(&mut self, choice_rule: &ChoiceRule, entry: u32, exit: u32) -> u32 {
    let entry = entry;
    let choice_exit = exit;

    for option in choice_rule.options.iter() {
      let choice = self.new_state();

      self.new_edge(
        entry,
        choice,
        Payload::Choice,
        None
      );

      self.build_flow(option, choice, choice_exit);
    }

    return choice_exit
  }

  /// Build SeqStage has to worry about the most GameFlowChanges.
  /// UntilEnd has to be handled separately.
  /// EndStage and EndGame are already handled in the build_flows method (can be ignored here).
  fn build_seq_stage(&mut self, stage: &SeqStage, entry: u32, exit: u32) -> u32 {
    // Stage information
    let stage_id = stage.stage.clone();
    let end_condition = stage.end_condition.clone();

    // Creating a new stage_exit
    self.stage_exits.push(exit);
    self.stage_to_exit.insert(stage_id.node.clone(), exit);

    // Check End-Condition Type
    match end_condition.node {
        EndCondition::UntilEnd => {
          let flows_exit = self.new_state();
          // Dont do a split with EndCondition and NotEndCondition
          match self.build_flows(&stage.flows, entry, flows_exit) {
              GameFlowChange::None(_) => {
                self.new_edge(
                  flows_exit,
                  entry,
                  Payload::StageRoundCounter(stage_id.clone()),
                  None
                );
              },
              _ => {}
          }

          // Remove current Stage
          self.stage_exits.pop();

          return exit
        },
        _ => {
          // Do a split with EndCondition and NotEndCondition
          self.new_edge(
            entry,
            exit,
            Payload::EndCondition { expr: end_condition.clone(), negated: true },
            None
          );

          let else_state = self.new_state();

          self.new_edge(
            entry, 
          else_state,
              Payload::EndCondition { expr: end_condition.clone(), negated: false },
              None
          );

          let flows_exit = self.new_state();
          match self.build_flows(&stage.flows, else_state, flows_exit) {
              GameFlowChange::None(_) => {
                self.new_edge(
                  flows_exit,
                  entry,
                  Payload::StageRoundCounter(stage_id.clone()),
                  None
                );
              },
              _ => {}
          }

          // Remove current Stage
          self.stage_exits.pop();

          return exit
        },
    }
  }

  // =========================================================================
  // =========================================================================
  // =========================================================================
  // =========================================================================
  // =========================================================================
  // TODO: SimStage is not implemented!
  /// Build SimStage has to worry about the most GameFlowChanges.
  /// UntilEnd has to be handled separately.
  /// EndStage and EndGame are already handled in the build_flows method (can be ignored here).
  /// Additionally we need to spawn a sub graph for each player and each edge needs to check
  /// if no player has finished the stage already (or any other condition).
  fn build_sim_stage(&mut self, stage: &SimStage, entry: u32, exit: u32) -> u32 {
    // Stage information
    let stage_id = stage.stage.clone();
    let end_condition = stage.end_condition.clone();

    // Creating a new stage_exit
    self.stage_exits.push(exit);
    self.stage_to_exit.insert(stage_id.node.clone(), exit);

    // Check End-Condition Type
    match end_condition.node {
        EndCondition::UntilEnd => {
          let flows_exit = self.new_state();
          // Dont do a split with EndCondition and NotEndCondition
          match self.build_flows(&stage.flows, entry, flows_exit) {
              GameFlowChange::None(_) => {
                self.new_edge(
                  flows_exit,
                  entry,
                  Payload::StageRoundCounter(stage_id.clone()),
                  None
                );
              },
              _ => {}
          }

          // Remove current Stage
          self.stage_exits.pop();

          return exit
        },
        _ => {
          // Do a split with EndCondition and NotEndCondition
          self.new_edge(
            entry,
            exit,
            Payload::EndCondition { expr: end_condition.clone(), negated: true },
            None
          );

          let else_state = self.new_state();

          self.new_edge(
            entry, 
          else_state,
              Payload::EndCondition { expr: end_condition.clone(), negated: false },
              None
          );

          let flows_exit = self.new_state();
          match self.build_flows(&stage.flows, else_state, flows_exit) {
              GameFlowChange::None(_) => {
                self.new_edge(
                  flows_exit,
                  entry,
                  Payload::StageRoundCounter(stage_id.clone()),
                  None
                );
              },
              _ => {}
          }

          // Remove current Stage
          self.stage_exits.pop();

          return exit
        },
    }
  }
  // =========================================================================
  // =========================================================================
  // =========================================================================
  // =========================================================================
  // =========================================================================
  
  /// Needs to take care of GameFlowChange. Only Source that emits this.
  /// Takes care of EndStage and EndGame
  fn build_rule(&mut self, rule: &SGameRule, entry: u32, exit: u32) -> GameFlowChange {
    // Take care of EndStage
    match &rule.node {
        GameRule::Action { action: spanned } => {
          match &spanned.node {
            ActionRule::EndAction { end_type } => {
              match &end_type.node {
                EndType::CurrentStage => {
                  if let Some(last_stage_exit) = self.stage_exits.last().cloned() {
                    self.new_edge(
                      entry,
                      last_stage_exit,
                      Payload::Action(rule.clone()),
                      None
                    );

                    // Nothing after end stage will be evaluated!
                    return GameFlowChange::EndCurrentStage(last_stage_exit);
                  } else {
                    // No stage found to end
                    self.diagnostics.push(
                      GameFlowError::NoStageToEnd { span: spanned.span.clone() }
                    );

                    return GameFlowChange::None(exit)
                  }
                },
                EndType::Stage { stage } => {
                  if let Some(specific_exit) = self.stage_to_exit.get(&stage.node).cloned() {
                    self.new_edge(
                      entry,
                      specific_exit,
                      Payload::Action(rule.clone()),
                      None
                    );

                    // Nothing after end stage will be evaluated!
                    return GameFlowChange::EndStage(specific_exit);
                  } else {
                    // No stage found to end
                    self.diagnostics.push(
                      GameFlowError::NoStageToEnd { span: spanned.span.clone() }
                    );

                    return GameFlowChange::None(exit)
                  }
                },
                EndType::GameWithWinner { players: _ } => {
                  let goal = self.fsm.goal.0;

                  self.new_edge(
                    entry,
                    goal,
                    Payload::Action(rule.clone()),
                    None
                  );

                  // Nothing after end game will be evaluated!
                  return GameFlowChange::EndGame(goal)
                },
                EndType::Turn => {
                  // Normal action with no GameFlowChange
                  self.new_edge(
                    entry,
                    exit,
                    Payload::Action(rule.clone()),
                    None
                  );

                  return GameFlowChange::None(exit)
                },
              }
            },
            _ => {
              // Normal action with no GameFlowChange
              self.new_edge(
                entry,
                exit,
                Payload::Action(rule.clone()),
                None
              );

              return GameFlowChange::None(exit)
            }
          }
        },
        GameRule::SetUp {setup: _} => {
          // Normal action with no GameFlowChange
          self.new_edge(
            entry,
            exit,
            Payload::Action(rule.clone()),
            None
          );

          return GameFlowChange::None(exit)
        },
        GameRule::Scoring { scoring: _ } => {
          // Normal action with no GameFlowChange
          self.new_edge(
            entry,
            exit,
            Payload::Action(rule.clone()),
            None
          );

          return GameFlowChange::None(exit)
        },
    }
  }  

  /// GameFlowChanges are handled separately. build_if_rule does not need to worry!
  fn build_if_rule(&mut self, if_rule: &IfRule, entry: u32, exit: u32) -> u32 {
    let condition = if_rule.condition.clone();
    let if_body = self.new_state();

    self.new_edge(
      entry,
      if_body,
      Payload::Condition { expr: condition.clone(), negated: false },
      None
    );

    self.new_edge(
      entry,
      exit,
      Payload::Condition { expr: condition.clone(), negated: true },
      None
    );

    self.build_flows(&if_rule.flows, if_body, exit);

    return exit
  }

  /// GameFlowChanges are handled separately. build_cond_rule does not need to worry!
  /// Needs to check if case has a Condition or does not have a Condition.
  fn build_cond_rule(&mut self, cond_rule: &Conditional, entry: u32, exit: u32) -> u32 {
    let _len = cond_rule.cases.len() - 1;
    let mut next_entry = entry;
    let mut case_exit = exit;
    for i in 0..cond_rule.cases.len() {
      case_exit = if i == _len { exit } else { self.new_state() };
      match &cond_rule.cases[i].node {
        // Case::Else { flows: spanneds } => {
        //   self.build_flows(&spanneds, next_entry, case_exit);
        // },
        Case::NoBool { flows: spanneds } => {
          self.build_flows(&spanneds, next_entry, case_exit);
        },
        Case::Bool { bool_expr: spanned, flows: spanneds } => {
          let body = self.new_state();
          let condition = spanned.clone();
          self.new_edge(
            next_entry,
            body,
            Payload::Condition { expr: condition.clone(), negated: false },
            None
          );

          self.new_edge(
            next_entry,
            case_exit,
            Payload::Condition { expr: condition.clone(), negated: true },
            None
          );

          self.build_flows(&spanneds, body, case_exit);          
        },
      }

      next_entry = case_exit;
    }

    return case_exit
  }

  /// GameFlowChanges are handled separately. build_optional_rule does not need to worry!
  fn build_optional_rule(&mut self, optional_rule: &OptionalRule, entry: u32, exit: u32) -> u32 {
    let optional_body = self.new_state();
    self.new_edge(entry, optional_body, Payload::Optional, None);
    self.build_flows(&optional_rule.flows, optional_body, exit);
    self.new_edge(entry, exit, Payload::Optional, None);

    return exit
  }

  /// GameFlowChanges are handled separately. build_optional_rule does not need to worry!
  fn build_trigger_rule(&mut self, trigger_rule: &TriggerRule, entry: u32, exit: u32) -> u32 {
    let trigger_body = self.new_state();
    self.new_edge(entry, trigger_body, Payload::Trigger, None);
    self.build_flows(&trigger_rule.flows, trigger_body, exit);

    return exit
  }
}