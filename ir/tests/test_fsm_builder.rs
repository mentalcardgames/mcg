mod test {

  use std::{path::Path, process::Command};

  use ast::ast::*;

  use ir::fsm::*;
  use ir::fsm_to_dot::*;

  const CURRENT: PlayerExpr = PlayerExpr::Current;
  const PREVIOUS: PlayerExpr = PlayerExpr::Previous;

  #[warn(unused)]
  fn id(id: &str) -> ID {
    ID::new(id)
  }

  fn stage(id: &str) -> Stage {
    Stage::new(ID::new(id))
  }

  fn playername(id: &str) -> PlayerName {
    PlayerName::new(ID::new(id))
  }

  #[warn(unused)]
  fn teamname(id: &str) -> TeamName {
    TeamName::new(ID::new(id))
  }

  fn location(id: &str) -> Location {
    Location::new(ID::new(id))
  }

  #[warn(unused)]
  fn token(id: &str) -> Token {
    Token::new(ID::new(id))
  }

  fn precedence(id: &str) -> Precedence {
    Precedence::new(ID::new(id))
  }

  fn pointmap(id: &str) -> PointMap {
    PointMap::new(ID::new(id))
  }

  fn combo(id: &str) -> Combo {
    Combo::new(ID::new(id))
  }
  
  fn memory(id: &str) -> Memory {
    Memory::new(ID::new(id))
  }

  fn key(id: &str) -> Key {
    Key::new(ID::new(id))
  }
  
  fn value(id: &str) -> Value {
    Value::new(ID::new(id))
  }

  fn show_graph(fsm: &FSM, name: &str) {
    let dot_path_name: &str = &format!("tests_out/{}.dot", name);
    let png_path_name: &str = &format!("tests_out/{}.png", name);

    let dot_path = Path::new(dot_path_name);
    let png_path = Path::new(png_path_name);

    fsm_to_dot(&fsm, dot_path).unwrap();

    // Call Graphviz
    let status = Command::new("dot")
      .args([
          "-Tpng",
          dot_path.to_str().unwrap(),
          "-o",
          png_path.to_str().unwrap(),
      ])
      .status()
      .expect("failed to run dot");

    assert!(status.success());
  }

  #[test]
  fn test_rule() {
    let mut builder = FSMBuilder::default();

    let fsm = builder.build_fsm(
      Game { 
        flows: vec![
          FlowComponent::Rule(Rule::EndTurn)
        ] 
      }
    );

    show_graph(&fsm, "rule");    
  }

  #[test]
  fn test_if_rule() {
    let mut builder = FSMBuilder::default();

    let fsm = builder.build_fsm(
      Game { 
        flows: vec![
          FlowComponent::IfRule(
            IfRule {
              condition: BoolExpr::OutOfStagePlayer(PlayerExpr::Current),
              flows: vec![
                FlowComponent::Rule(
                  Rule::CycleAction(
                    PlayerExpr::Next
                  )
                )
              ]
            }
          )
        ] 
      }
    );

    show_graph(&fsm, "if_rule");
  }

  #[test]
  fn test_optional_rule() {
    let mut builder = FSMBuilder::default();

    let fsm = builder.build_fsm(
      Game { 
        flows: vec![
          FlowComponent::OptionalRule(
            OptionalRule {
              flows: vec![
                FlowComponent::Rule(
                  Rule::EndTurn
                )
              ]
            }
          )
        ] 
      }
    );

    show_graph(&fsm, "optional_rule");
  }

  #[test]
  fn test_choice_rule() {
    let mut builder = FSMBuilder::default();

    let fsm = builder.build_fsm(
      Game { 
        flows: vec![
          FlowComponent::ChoiceRule(
            ChoiceRule {
              options: vec![
                FlowComponent::Rule(
                  Rule::EndTurn
                ),
                FlowComponent::OptionalRule(
                  OptionalRule {
                    flows: vec![
                        FlowComponent::Rule(
                          Rule::EndTurn
                        )
                    ]
                  }
                ),
              ]
            }
          )
        ] 
      }
    );

    show_graph(&fsm, "choice_rule");
  }

  #[test]
  fn test_stage() {
    let mut builder = FSMBuilder::default();

    let fsm = builder.build_fsm(
      Game { 
        flows: vec![
          FlowComponent::Stage(
            SeqStage {
              stage: stage("Preparation"),
              player: PlayerExpr::Current, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::DealMove(
                    DealMove::DealQuantity(
                      Quantity::Int(IntExpr::Int(12)), 
                      CardSet::Group(Group::CardPosition(CardPosition::Top(location("Stock")))), 
                      Status::Private, 
                      CardSet::GroupOfPlayerCollection(Group::Location(location("Hand")), PlayerCollection::Quantifier(Quantifier::All))
                    )
                  )
                )
              ] 
            }
          )
        ] 
      }
    );

    show_graph(&fsm, "stage");
  }
  
  #[test]
  fn test_game() {
    let mut builder = FSMBuilder::default();

    let fsm = builder.build_fsm(
      Game {
        flows: vec![
          // create players
          FlowComponent::Rule(
            Rule::CreatePlayer(
              vec![
                playername("P1"),
                playername("P2"),
                playername("P3"),
              ]
            )
          ),
          // create turnorder
          FlowComponent::Rule(
            Rule::CreateTurnorder(
              vec![
                playername("P1"),
                playername("P2"),
                playername("P3"),
              ]
            )
          ),
          // location on all
          FlowComponent::Rule(
            Rule::CreateLocationCollectionOnPlayerCollection(
              LocationCollection {
                locations: vec![
                  location("Hand"),
                  location("LayDown"),
                  location("Trash"),
                ]
              },
              PlayerCollection::Quantifier(Quantifier::All)
            )
          ),
          // location on table
          FlowComponent::Rule(
            Rule::CreateLocationCollectionOnTable(
              LocationCollection {
                locations: vec![
                  location("Stock"),
                  location("Discard"),
                ]
              }
            )
          ),
          // card on
          FlowComponent::Rule(
            Rule::CreateCardOnLocation(
              location("Stock"),
              Types {
                types: vec![
                  (key("Rank"), vec![
                    value("Two"),
                    value("Three"),
                    value("Four"),
                    value("Five"),
                    value("Six"),
                    value("Seven"),
                    value("Eight"),
                    value("Nine"),
                    value("Ten"),
                    value("Jack"),
                    value("Queen"),
                    value("King"),
                    value("Ace")
                  ]),
                  (key("Suite"), vec![
                    value("Diamonds"),
                    value("Hearts"),
                    value("Spades"),
                    value("Clubs"),
                  ]),
                ]
              }
            )
          ),
          // RankOrder
          FlowComponent::Rule(
            Rule::CreatePrecedence(
              precedence("RankOrder"),
              vec![
                (key("Rank"), value("Ace")),
                (key("Rank"), value("Two")),
                (key("Rank"), value("Three")),
                (key("Rank"), value("Four")),
                (key("Rank"), value("Five")),
                (key("Rank"), value("Six")),
                (key("Rank"), value("Seven")),
                (key("Rank"), value("Eight")),
                (key("Rank"), value("Nine")),
                (key("Rank"), value("Ten")),
                (key("Rank"), value("Jack")),
                (key("Rank"), value("Queen")),
                (key("Rank"), value("King")),
              ]
            )
          ),
          // Values
          FlowComponent::Rule(
            Rule::CreatePointMap(
              pointmap("Values"),
              vec![
                (key("Rank"), value("Ace"), IntExpr::Int(1)),
                (key("Rank"), value("Two"), IntExpr::Int(2)),
                (key("Rank"), value("Three"), IntExpr::Int(3)),
                (key("Rank"), value("Four"), IntExpr::Int(4)),
                (key("Rank"), value("Five"), IntExpr::Int(5)),
                (key("Rank"), value("Six"), IntExpr::Int(6)),
                (key("Rank"), value("Seven"), IntExpr::Int(7)),
                (key("Rank"), value("Eight"), IntExpr::Int(8)),
                (key("Rank"), value("Nine"), IntExpr::Int(9)),
                (key("Rank"), value("Ten"), IntExpr::Int(10)),
                (key("Rank"), value("Jack"), IntExpr::Int(10)),
                (key("Rank"), value("Queen"), IntExpr::Int(10)),
                (key("Rank"), value("King"), IntExpr::Int(10)),
              ]
            )
          ),
          // Combo Sequence
          FlowComponent::Rule(
            Rule::CreateCombo(
              combo("Sequence"),
              FilterExpr::And(
                Box::new(FilterExpr::And(
                  Box::new(FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))),
                  Box::new(FilterExpr::Same(key("Suite")))
                )),
                Box::new(FilterExpr::Adjacent(key("Rank"), precedence("RankOrder")))
              )
            )
          ),
          // Combo Set
          FlowComponent::Rule(
            Rule::CreateCombo(
              combo("Set"),
              FilterExpr::And(
                Box::new(FilterExpr::And(
                  Box::new(FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))),
                  Box::new(FilterExpr::Distinct(key("Suite")))
                )),
                Box::new(FilterExpr::Same(key("Rank")))
              )
            )
          ),
          // Combo Set
          FlowComponent::Rule(
            Rule::CreateCombo(
              combo("Deadwood"),
              FilterExpr::And(
                Box::new(
                  FilterExpr::NotCombo(combo("Sequence"))
                ),
                Box::new(
                  FilterExpr::NotCombo(combo("Set"))
                )
              )
            )
          ),
          // Stage Preparation
          FlowComponent::Stage(
            SeqStage {
              stage: stage("Preparation"), 
              player: CURRENT, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::DealMove(
                    DealMove::DealQuantity(
                      Quantity::Int(IntExpr::Int(12)), 
                      CardSet::Group(Group::CardPosition(CardPosition::Top(location("Stock")))), 
                      Status::Private, 
                      CardSet::GroupOfPlayerCollection(Group::Location(location("Hand")), PlayerCollection::Quantifier(Quantifier::All))
                    )
                  )
                )
              ] 
            }
          ),
          // Stage Collect
          FlowComponent::Stage(
            SeqStage {
              stage: stage("Collect"), 
              player: CURRENT, 
              end_condition: EndCondition::UntilBool(BoolExpr::OutOfStagePlayer(PREVIOUS)), 
              flows: vec![
                // Choose
                FlowComponent::ChoiceRule(
                  ChoiceRule {
                    options: vec![
                      // move top of Discard to Hand
                      FlowComponent::Rule(
                        Rule::ClassicMove(
                          ClassicMove::Move(
                            CardSet::Group(Group::CardPosition(CardPosition::Top(location("Discard")))),
                            Status::Private,
                            CardSet::Group(Group::Location(location("Hand")))
                          )
                        )
                      ),
                      // move top of Stock to Hand
                      FlowComponent::Rule(
                        Rule::ClassicMove(
                          ClassicMove::Move(
                            CardSet::Group(Group::CardPosition(CardPosition::Top(location("Stock")))),
                            Status::Private,
                            CardSet::Group(Group::Location(location("Hand")))
                          )
                        )
                      ),
                    ]
                  }
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::MoveQuantity(
                      Quantity::Quantifier(Quantifier::Any),
                      CardSet::Group(Group::Location(location("Hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(location("Discard")))),
                    )
                  )
                ),
                FlowComponent::IfRule(
                  IfRule { 
                    condition: BoolExpr::IntCmp(
                      IntExpr::SumOfCardSet(
                        Box::new(
                          CardSet::Group(
                            Group::ComboInLocation(
                              combo("Deadwood"),
                              location("Hand")
                            )
                          )
                        ), 
                        pointmap("Values")
                      ), 
                      IntCmpOp::Le, 
                      IntExpr::Int(10)
                    ),
                    flows: vec![
                      FlowComponent::OptionalRule(
                        OptionalRule { 
                          flows: vec![
                            FlowComponent::Rule(
                              Rule::ClassicMove(
                                ClassicMove::MoveQuantity(
                                  Quantity::Quantifier(Quantifier::All),
                                  CardSet::Group(Group::ComboInLocation(combo("Set"), location("Hand"))),
                                  Status::FaceUp,
                                  CardSet::Group(Group::CardPosition(CardPosition::Top(location("LayDown")))),
                                )
                              )
                            ),
                            FlowComponent::Rule(
                              Rule::ClassicMove(
                                ClassicMove::MoveQuantity(
                                  Quantity::Quantifier(Quantifier::All),
                                  CardSet::Group(Group::ComboInLocation(combo("Sequence"), location("Hand"))),
                                  Status::FaceUp,
                                  CardSet::Group(Group::CardPosition(CardPosition::Top(location("LayDown")))),
                                )
                              )
                            ),
                            // If rule
                            FlowComponent::IfRule(
                              IfRule {
                                condition: BoolExpr::CardSetIsEmpty(
                                  CardSet::Group(
                                    Group::Location(location("Hand"))
                                  )
                                ),
                                flows: vec![
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::MoveQuantity(
                                        Quantity::Quantifier(Quantifier::All),
                                        CardSet::GroupOfPlayer(Group::ComboInLocation(combo("Set"), location("Hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::CardPosition(CardPosition::Top(location("LayDown"))), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::MoveQuantity(
                                        Quantity::Quantifier(Quantifier::All),
                                        CardSet::GroupOfPlayer(Group::ComboInLocation(combo("Sequence"), location("Hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::CardPosition(CardPosition::Top(location("LayDown"))), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::Move(
                                        CardSet::GroupOfPlayer(Group::Location(location("Hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::Location(location("Trash")), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::Move(
                                        CardSet::Group(Group::Location(location("Hand"))),
                                        Status::FaceUp,
                                        CardSet::Group(Group::Location(location("Trash"))),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::PlayerOutOfStageAction(
                                      CURRENT
                                    )
                                  ),
                                ]
                              }
                            )
                          ]
                        }
                      )
                    ]
                  }
                ),
                FlowComponent::Rule(
                  Rule::CycleAction(PlayerExpr::Next)
                )
              ] 
            }
          ),
          // Stage Preparation
          FlowComponent::Stage(
            SeqStage {
              stage: stage("FinalLayDown"), 
              player: CURRENT, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::Move(
                      CardSet::GroupOfPlayer(Group::Location(location("LayDown")), PREVIOUS),
                      Status::FaceUp,
                      CardSet::GroupOfPlayer(Group::Location(location("Hand")), CURRENT),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::MoveQuantity(
                      Quantity::Quantifier(Quantifier::All),
                      CardSet::Group(Group::ComboInLocation(combo("Set"), location("Hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(location("LayDown")))),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::MoveQuantity(
                      Quantity::Quantifier(Quantifier::All),
                      CardSet::Group(Group::ComboInLocation(combo("Sequence"), location("Hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(location("LayDown")))),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::Move(
                      CardSet::Group(Group::Location(location("Hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::Location(location("Trash"))),
                    )
                  )
                ),
              ] 
            }
          ),
          FlowComponent::Rule(
            Rule::ScoreRule(
              ScoreRule::ScorePlayerCollectionMemory(
                IntExpr::SumOfCardSet(
                  Box::new(
                    CardSet::Group(
                      Group::Location(
                        location("Trash")
                      )
                    )
                  ),
                  pointmap("Values")
                ),
                memory("LeftOver"),
                PlayerCollection::Quantifier(Quantifier::All),
              )
            )
          ),
          FlowComponent::Rule(
            Rule::WinnerRule(
              WinnerRule::WinnerLowestMemory(
                memory("LeftOver")
              )
            )
          ),
        ]
      }
    );

    show_graph(&fsm, "game");
  }
  
}