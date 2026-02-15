use std::{collections::HashMap, fmt::Debug};
use crate::{ast::ast::*, ast::ast::GameRule};

pub type Stage = String;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct StateID(u32);

impl StateID {
    pub fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct StageExit(u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChoiceExit(u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TransitionID(u32);

pub struct Edge<T> {
  pub to: StateID,
  pub payload: T,
}

pub struct Ir<T> {
  pub states: HashMap<StateID, Vec<Edge<T>>>,
  pub entry: StateID,
  pub goal: StateID,
}

impl<T> Default for Ir<T> {
  fn default() -> Self {
      Ir {
        states: HashMap::new(),
        entry: StateID(0),
        goal: StateID(0)
      }
  }
}

impl<T> Ir<T> {
  /// Both States need to be added before the edge can be added.
  pub fn add_edge(&mut self, from: StateID, to: StateID, payload: T) {
    let edge = Edge { to: to, payload: payload };
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
}


/// There are certain Rules that alter the flow of the game:
/// - End Stage
/// - End Game
/// There might be added more rules that alter the flow of the game.
/// These rules need careful handling for constructing the IR.
#[derive(Debug)]
pub enum GameFlowChange {
  EndCurrentStage(u32),
  EndGame(u32),
  None(u32),
}



/// Each Transition/Edge needs to have some guard/payload.
/// E.g. If we have a condition then the edge's payload is proving the condition.
#[derive(Debug)]
pub enum Payload<T>
  where T: Debug,
{
  Instruction(T),
  Control(Control)
}

impl<C: Debug, E: Debug, S: Debug> Payload<Semantic<C, E, GameRule, S>> {
  pub fn raw(&self) -> &str {
    match self {
        Payload::Instruction(i) => i.raw(),
        Payload::Control(control) => control.raw(),
    }
  }
}


/// A set of Payload-Instructions.
#[derive(Debug)]
pub enum Semantic<C, E, A, S> {
  Condition { expr: C, negated: bool },
  EndCondition { expr: E, negated: bool },
  Action(A),
  StageRoundCounter(S),
  EndStage(S),
  StageExit(S),
  StageEntry(S),
}

impl<C, E, S> Semantic<C, E, GameRule, S> {
  pub fn raw(&self) -> &str {
    match self {
      Semantic::Condition { expr: _, negated } => {
        if *negated {
          return "Not Condition"
        }

        return "Condition"
      },
      Semantic::EndCondition { expr: _, negated } => {
        if *negated {
          return "Not EndCondition"
        }

        return "EndCondition"
      },
      Semantic::Action(rule) => {
        if matches!(rule, GameRule::Action(SActionRule { node: ActionRule::EndAction(SEndType { node: EndType::Stage, span: _ } ), span: _ } )) {
          return "EndStage"
        }
        return "Action"
      },
      Semantic::StageRoundCounter(_) => {
        return "StageRoundCounter"
      },
      Semantic::EndStage(_) => {
        return "EndStage"
      },
      Semantic::StageExit(_) => {
        return "StageExit"
      },
      Semantic::StageEntry(_) => {
        return "StageEntry"
      },
    }
  } 
}

/// Set of control payload. 
#[derive(Debug)]
pub enum Control {
  Choice,
  Optional,
}

impl Control {
  pub fn raw(&self) -> &str {
    match self {
        Control::Choice => "Choice",
        Control::Optional => "Optional",
    }
  }
}

// /// E.g. when constructing the stage-round-counter,
// /// we need to go back to the entry of the current Stage.
// #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
// pub enum Entry {
//   Stage(u32),
//   Choice(u32),
// }

// impl Entry {
//   fn get_id(&self) -> u32 {
//     match self {
//         Entry::Stage(u) => *u,
//         Entry::Choice(u) => *u,
//     }
//   }
// }

// /// E.g. GameRule EndStage needs this information,
// /// we need to go to the exit of the current Stage.
// #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
// pub enum Exit {
//   Stage(u32),
//   Choice(u32),
// }

// impl Exit {
//   fn get_id(&self) -> u32 {
//     match self {
//         Exit::Stage(u) => *u,
//         Exit::Choice(u) => *u,
//     }
//   }
// }

/// fsm: The current IR being constructed.
/// stage_exits: Keeping track of stage_exits
pub struct IrBuilder<T> {
  pub fsm: Ir<T>,
  state_counter: u32,
  stage_exits: Vec<u32>,
}

impl<T> Default for IrBuilder<T> {
  fn default() -> Self {
    IrBuilder {
      fsm: Ir::default(),
      state_counter: 0,
      stage_exits: Vec::new(),
    }
  }
}


pub type PayloadT = Payload<Semantic<SBoolExpr, SEndCondition, GameRule, SID>>;

impl IrBuilder<PayloadT> {
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
  fn new_edge(&mut self, from: u32, to: u32, payload: PayloadT) {
    self.fsm.add_edge(
      StateID(from),
      StateID(to),
      payload
    );
  }

  /// Builds FSM.
  /// Initializes the first state and then continues with the building of the FlowComponent's
  pub fn build_ir(&mut self, game: SGame) {
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

          // Every FlowComponent after this will not be evaluated 
          return GameFlowChange::EndCurrentStage(stage)
        },
        GameFlowChange::EndGame(game) => {
          // The flow_exit was not used, so remove it
          self.remove_state(flow_exit);

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
        FlowComponent::ChoiceRule(choice_rule) => {
            self.build_choice_rule(&choice_rule.node, entry, exit)
        },
        FlowComponent::Stage(seq_stage) => {
            self.build_seq_stage(&seq_stage.node, entry, exit)
        },
        FlowComponent::Rule(rule) => {
            // Can have GameFlowChanges! So return here.
            return self.build_rule(&rule.node, entry, exit)
        },
        FlowComponent::IfRule(if_rule) => {
            self.build_if_rule(&if_rule.node, entry, exit)
        },
        FlowComponent::OptionalRule(optional_rule) => {
            self.build_optional_rule(&optional_rule.node, entry, exit)
        },
        FlowComponent::Conditional(cond_rule) => {
            self.build_cond_rule(&cond_rule.node, entry, exit)
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
        Payload::Control(Control::Choice)
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
    let stage_exit = exit;
    self.stage_exits.push(stage_exit);

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
                  Payload::Instruction(Semantic::StageRoundCounter(stage_id.clone()))
                );
              },
              _ => {}
          }

          // Remove current Stage
          self.stage_exits.pop();

          return stage_exit
        },
        _ => {
          // Do a split with EndCondition and NotEndCondition
          self.new_edge(
            entry,
            stage_exit,
            Payload::Instruction(Semantic::EndCondition { expr: end_condition.clone(), negated: true })
          );

          let else_state = self.new_state();

          self.new_edge(
            entry, 
          else_state,
              Payload::Instruction(Semantic::EndCondition { expr: end_condition.clone(), negated: false })
          );

          let flows_exit = self.new_state();
          match self.build_flows(&stage.flows, else_state, flows_exit) {
              GameFlowChange::None(_) => {
                self.new_edge(
                  flows_exit,
                  entry,
                  Payload::Instruction(Semantic::StageRoundCounter(stage_id.clone()))
                );
              },
              _ => {}
          }

          let flows_exit = self.new_state();
          match self.build_flows(&stage.flows, else_state, flows_exit) {
              GameFlowChange::None(_) => {
                self.new_edge(
                  flows_exit,
                  entry,
                  Payload::Instruction(Semantic::StageRoundCounter(stage_id.clone()))
                );
              },
              _ => {}
          }

          // Remove current Stage
          self.stage_exits.pop();

          return stage_exit
        },
    }
  }
  
  /// Needs to take care of GameFlowChange. Only Source that emits this.
  /// Takes care of EndStage and EndGame
  fn build_rule(&mut self, rule: &GameRule, entry: u32, exit: u32) -> GameFlowChange {
    // Take care of EndStage
    if matches!(rule, GameRule::Action(SActionRule { node: ActionRule::EndAction(SEndType { node: EndType::Stage, span: _ } ), span: _ } )) {
      let last_stage_exit = self.stage_exits.last().expect("No Stage found to end!").clone();

      self.new_edge(
        entry,
        last_stage_exit,
        Payload::Instruction(Semantic::Action(rule.clone()))
      );

      // Nothing after end stage will be evaluated!
      return GameFlowChange::EndCurrentStage(last_stage_exit);
    }

    // Take care of EndGame
    if matches!(rule, GameRule::Action(SActionRule { node: ActionRule::EndAction(SEndType { node: EndType::GameWithWinner(_), span: _ } ), span: _ } )) {
      let goal = self.fsm.goal.0;

      self.new_edge(
        entry,
        goal,
        Payload::Instruction(Semantic::Action(rule.clone()))
      );

      // Nothing after end game will be evaluated!
      return GameFlowChange::EndGame(goal);
    }
    
    // Normal action with no GameFlowChange
    self.new_edge(
      entry,
      exit,
      Payload::Instruction(Semantic::Action(rule.clone()))
    );

    return GameFlowChange::None(exit)
  }  

  /// GameFlowChanges are handled separately. build_if_rule does not need to worry!
  fn build_if_rule(&mut self, if_rule: &IfRule, entry: u32, exit: u32) -> u32 {
    let condition = if_rule.condition.clone();
    let if_body = self.new_state();

    self.new_edge(
      entry,
      if_body,
      Payload::Instruction(Semantic::Condition { expr: condition.clone(), negated: false })
    );

    self.new_edge(
      entry,
      exit,
      Payload::Instruction(Semantic::Condition { expr: condition.clone(), negated: true })
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
        Case::Else(spanneds) => {
          self.build_flows(&spanneds, next_entry, case_exit);
        },
        Case::NoBool(spanneds) => {
          self.build_flows(&spanneds, next_entry, case_exit);
        },
        Case::Bool(spanned, spanneds) => {
          let body = self.new_state();
          let condition = spanned.clone();
          self.new_edge(
            next_entry,
            body,
            Payload::Instruction(Semantic::Condition { expr: condition.clone(), negated: false })
          );

          self.new_edge(
            next_entry,
            case_exit,
            Payload::Instruction(Semantic::Condition { expr: condition.clone(), negated: true })
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
    self.new_edge(entry, optional_body, Payload::Control(Control::Optional));
    self.build_flows(&optional_rule.flows, optional_body, exit);
    self.new_edge(entry, exit, Payload::Control(Control::Optional));

    return exit
  }
}