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
  pub goals: Vec<StateID>,
}

impl<T> Default for Ir<T> {
  fn default() -> Self {
      Ir {
        states: HashMap::new(),
        entry: StateID(0),
        goals: Vec::new()
      }
  }
}

impl<T> Ir<T> {
  pub fn add_edge(&mut self, from: StateID, to: StateID, payload: T) {
    let edge = Edge { to: to, payload: payload };
    let vec = self.states
      .get_mut(&from)
      .expect("StateID was not added before!");

    vec.push(edge);
  }

  pub fn add_state(&mut self, state: StateID) {
    self.states.insert(state, Vec::new());
  }

  pub fn remove_state(&mut self, state: StateID) {
    self.states.remove(&state);
  }
}


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
        Payload::Instruction(i) => i.raw().clone(),
        Payload::Control(control) => control.raw(),
    }
  }
}


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
      Semantic::Condition { expr, negated } => {
        if *negated {
          return "Not Condition"
        }

        return "Condition"
      },
      Semantic::EndCondition { expr, negated } => {
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Entry {
  Stage(u32),
  Choice(u32),
}

impl Entry {
  fn get_id(&self) -> u32 {
    match self {
        Entry::Stage(u) => *u,
        Entry::Choice(u) => *u,
    }
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Exit {
  Stage(u32),
  Choice(u32),
}

impl Exit {
  fn get_id(&self) -> u32 {
    match self {
        Exit::Stage(u) => *u,
        Exit::Choice(u) => *u,
    }
  }
}

pub struct Block {
  pub entry: u32,
  pub exit: u32,
}

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
  /// Sets current_state_id to state_counter.
  /// Returns updated state_counter.
  fn new_state(&mut self) -> u32 {
    self.state_counter += 1;
    self.fsm.add_state(StateID(self.state_counter));

    return self.state_counter
  }

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
    let entry = 0;
    self.fsm.add_state(StateID(entry));

    let exit = self.new_state();
    self.build_flows(&game.node.flows, entry, exit);
  }

  /// Takes a Vector of FlowComponent's and extends the FSM with them.
  fn build_flows(&mut self, flows: &Vec<SFlowComponent>, entry: u32, exit: u32) -> u32 {
    let mut flow_entry = entry;
    let mut flow_exit= exit;
    for i in 0..flows.len() {
      if i == flows.len() - 1 {
        return self.build_flow(&flows[i], flow_entry, exit);
      }

      flow_exit = self.new_state();
      flow_entry = self.build_flow(&flows[i], flow_entry, flow_exit);

      // End Stage! Or other end Rules!
      if let Some(stage_exit) = self.stage_exits.last() && *stage_exit == flow_exit {
        return self.remove_state(flow_exit);
      }
    }

    return flow_exit    
  }

  /// Builds a singular FlowComponent.
  fn build_flow(&mut self, flow: &SFlowComponent, entry: u32, exit: u32) -> u32 {
    match &flow.node {
        FlowComponent::ChoiceRule(choice_rule) => {
            self.build_choice_rule(&choice_rule.node, entry, exit)
        },
        FlowComponent::Stage(seq_stage) => {
            self.build_seq_stage(&seq_stage.node, entry, exit)
        },
        FlowComponent::Rule(rule) => {
          self.build_rule(&rule.node, entry, exit)
        },
        FlowComponent::IfRule(if_rule) => {
            self.build_if_rule(&if_rule.node, entry, exit)
        },
        FlowComponent::OptionalRule(optional_rule) => {
            self.build_optional_rule(&optional_rule.node, entry, exit)
        },
    }
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

  fn build_seq_stage(&mut self, stage: &SeqStage, entry: u32, exit: u32) -> u32 {
    // Getting the StateID to keep track on when it starts, ends and
    // the stage-counter increments for the specific Stage
    let stage_id = stage.stage.clone();
    let end_condition = stage.end_condition.clone();
    let stage_exit = exit;

    self.stage_exits.push(stage_exit);

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
    let flows_exit = self.build_flows(&stage.flows, else_state, flows_exit);

    if flows_exit != exit {
      self.new_edge(
        flows_exit,
        entry,
        Payload::Instruction(Semantic::StageRoundCounter(stage_id.clone()))
      );
    }

    self.stage_exits.pop();

    return stage_exit
  }
  
  fn build_rule(&mut self, rule: &GameRule, entry: u32, exit: u32) -> u32 {
    let rule_exit = exit;
    if matches!(rule, GameRule::Action(SActionRule { node: ActionRule::EndAction(SEndType { node: EndType::Stage, span: _ } ), span: _ } )) {
      let last_stage_exit = self.stage_exits.last().expect("No Stage found to end!").clone();

      self.new_edge(
        entry,
        last_stage_exit,
        Payload::Instruction(Semantic::Action(rule.clone()))
      );

      // Nothing after end stage will be evaluated!
      return last_stage_exit;
    } else {
      self.new_edge(
        entry,
        rule_exit,
        Payload::Instruction(Semantic::Action(rule.clone()))
      );
    }

    return rule_exit
  }  

  fn build_if_rule(&mut self, if_rule: &IfRule, entry: u32, exit: u32) -> u32 {
    let condition = if_rule.condition.clone();
    let if_body = self.new_state();
    let if_exit = exit;

    self.new_edge(
      entry,
      if_body,
      Payload::Instruction(Semantic::Condition { expr: condition.clone(), negated: false })
    );

    self.build_flows(&if_rule.flows, if_body, if_exit);

    self.new_edge(
      entry,
      if_exit,
      Payload::Instruction(Semantic::Condition { expr: condition.clone(), negated: true })
    );

    return if_exit
  }

  fn build_optional_rule(&mut self, optional_rule: &OptionalRule, entry: u32, exit: u32) -> u32 {
    let optional_body = self.new_state();
    let optional_exit = exit;
    self.new_edge(entry, optional_body, Payload::Control(Control::Optional));
    self.build_flows(&optional_rule.flows, optional_body, optional_exit);
    self.new_edge(entry, optional_exit, Payload::Control(Control::Optional));

    return optional_exit
  }
}