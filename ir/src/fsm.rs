use std::{collections::HashMap};
use ast::asts::typed_ast::*;

pub type Stage = String;

pub type StateID = i32;
pub type StageExit = i32;
pub type ChoiceExit = i32;
pub type TransitionID = i32;

#[derive(Clone, Debug)]
pub enum Transition {
  Action(Rule),
  Condition(BoolExpr),
  NotCondition(BoolExpr),
  EndCondition(EndCondition),
  NotEndCondition(EndCondition),
  StageCounter(Stage),
  StageEntry(Stage),
  StageExit(Stage),
  Optional,
  Choice,
}

#[derive(Clone, Debug)]
pub struct FSM {
  pub states: HashMap<StateID, Vec<(TransitionID, StateID)>>,
  pub transitions: HashMap<TransitionID, Transition>,
  pub entry: StateID,
  pub goals: Vec<StateID>,
}

impl Default for FSM {
  fn default() -> Self {
      FSM {
        states: HashMap::new(),
        transitions: HashMap::new(),
        entry: 0,
        goals: Vec::new()
      }
  }
}

impl FSM {
  /// Adds a transition to the current FSM.
  fn add_transition(
    &mut self,
    from_state: StateID,
    to_state: StateID,
    transition_id: TransitionID,
    transition: Transition
  ) {
    let transitions = self.states.get_mut(&from_state).expect("State could not been found!");
    transitions.push((transition_id, to_state));

    self.transitions.insert(transition_id, transition);
  }

  /// Adds a state to the current FSM.
  fn add_state(&mut self, state_id: StateID) {
    self.states.insert(state_id, Vec::new());
  }
}

pub struct FSMBuilder {
  fsm: FSM,
  current_state_id: i32,
  state_counter: i32,
  current_transition_id: i32,
  stage_exits: Vec<StageExit>,
  choice_exits: Vec<i32>,
}

impl Default for FSMBuilder {
  fn default() -> Self {
    FSMBuilder {
      fsm: FSM::default(),
      current_state_id: 0,
      state_counter: 0,
      current_transition_id: 0,
      stage_exits: Vec::new(),
      choice_exits: Vec::new(),
    }
  }
}

impl FSMBuilder {
  /// Builds FSM.
  /// Initializes the first state and then continues with the building of the FlowComponent's
  pub fn build_fsm(&mut self, game: Game) -> FSM {
    // initialize first state
    self.fsm.add_state(self.current_state_id);

    self.build_flows(&game.flows);

    return self.fsm.clone()
  }

  /// Takes a Vector of FlowComponent's and extends the FSM with them.
  fn build_flows(&mut self, flows: &Vec<FlowComponent>) {
    for flow in flows.iter() {
      self.build_flow(flow);
    }
  }

  /// Increments the state_counter.
  /// Adds the the new state (id of state == state_counter) to the FSM.
  /// Sets current_state_id to state_counter.
  /// Returns updated state_counter.
  fn new_state(&mut self) -> i32 {
    self.state_counter += 1;
    self.fsm.add_state(self.state_counter);
    self.current_state_id = self.state_counter;

    return self.state_counter
  }

  /// Updates the current_transition_id.
  /// Adds the transition to the FSM.
  fn new_transition(&mut self, from_state: StateID, to_state: StateID, transition: Transition) {
    self.current_transition_id += 1;

    self.fsm.add_transition(
      from_state,
      to_state,
      self.current_transition_id,
      transition
    );
  }

  /// Checks if we are in a ChoiceRule.
  /// If we are in a ChoiceRule:
  ///   > Set the exit of the next FlowComponent to the exit of the ChoiceRule and returns it
  /// Else:
  ///   > Return a new exit (create a new state and return it)
  fn new_exit(&mut self) -> StateID {
    if !self.choice_exits.is_empty() {
      return self.choice_exits.last().unwrap().clone()
    }

    return self.new_state()
  }

  /// Builds a singular FlowComponent.
  fn build_flow(&mut self, flow: &FlowComponent) {
    match flow {
        FlowComponent::ChoiceRule(choice_rule) => {
          self.build_choice_rule(choice_rule)
        },
        FlowComponent::Stage(seq_stage) => {
            self.build_seq_stage(seq_stage)
        },
        FlowComponent::Rule(rule) => {
            self.build_rule(rule)
        },
        FlowComponent::IfRule(if_rule) => {
            self.build_if_rule(if_rule)
        },
        FlowComponent::OptionalRule(optional_rule) => {
            self.build_optional_rule(optional_rule)
        },
    }
  }

  fn build_choice_rule(&mut self, choice_rule: &ChoiceRule) {
    let entry = self.current_state_id;
    let exit = self.new_exit();

    // =======================================================================
    // start new "choice-block"
    self.choice_exits.push(exit);

    for option in choice_rule.options.iter() {
      let choice = self.new_state();

      self.new_transition(
        entry,
        choice,
        Transition::Choice
      );

      self.build_flow(option);
    }

    // end choice block
    self.choice_exits.pop();
    // =======================================================================

    // continue building from the exit
    self.current_state_id = exit;
  }

  fn build_seq_stage(&mut self, stage: &SeqStage) {
    // Getting the StateID to keep track on when it starts, ends and
    // the stage-counter increments for the specific Stage
    let stage_id = stage.stage.id.clone();
    // Adding more to when a Stage starts
    let stage_entry = self.current_state_id;
    let entry = self.new_state();

    self.new_transition(
      stage_entry,
      entry,
      Transition::StageEntry(stage_id.clone())
    );

    let exit = self.new_exit();
    self.stage_exits.push(exit);
    let end_condition = stage.end_condition.clone();

    self.new_transition(
      entry,
      exit,
      Transition::NotEndCondition(end_condition.clone())
    );

    let else_state = self.new_state();

    self.new_transition(
      entry, 
      else_state,
      Transition::EndCondition(end_condition.clone())
    );

    self.build_flows(&stage.flows);

    self.new_transition(
      self.current_state_id,
      entry,
      Transition::StageCounter(stage_id.clone())
    );

    self.stage_exits.pop();

    // continue building from the exit
    self.current_state_id = exit;

    // Adding a specific end transition to mark the end of
    // the specific stage
    let stage_exit = self.new_state();

    self.new_transition(
      exit,
      stage_exit,
      Transition::StageExit(stage_id.clone())
    );
  }
  
  fn build_rule(&mut self, rule: &Rule) {

    match rule {
      Rule::EndStage => {
        let entry = self.current_state_id;
        let exit = self.stage_exits.last().expect("No Stage found to end!").clone();

        self.new_transition(
          entry,
          exit,
          Transition::Action(rule.clone())
        );

        // continue building from the exit
        self.current_state_id = exit;
      },

      _ => {
        let entry = self.current_state_id;
        let exit = self.new_exit();

        self.new_transition(
          entry,
          exit,
          Transition::Action(rule.clone())
        );
        
        // continue building from the exit
        self.current_state_id = exit;
      }
    }
    
  }

  fn build_if_rule(&mut self, if_rule: &IfRule) {
    let entry = self.current_state_id;
    let condition = if_rule.condition.clone();
    let if_body = self.new_state();

    self.new_transition(
      entry,
      if_body,
      Transition::Condition(condition.clone())
    );

    self.build_flows(&if_rule.flows);

    self.new_transition(
      entry,
      self.current_state_id,
      Transition::NotCondition(condition.clone())
    );
  }

  fn build_optional_rule(&mut self, optional_rule: &OptionalRule) {
    let entry = self.current_state_id;
    let optional_body = self.new_state();

    self.new_transition(entry, optional_body, Transition::Optional);
    self.build_flows(&optional_rule.flows);
    self.new_transition(entry, self.current_state_id, Transition::Optional);
  }
}
