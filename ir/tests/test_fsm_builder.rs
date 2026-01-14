mod test {

  use std::{path::Path, process::Command};

  use ast::ast::*;
  use ast::test_helper::test_helper as th;

  use ir::fsm::*;
  use ir::fsm_to_dot::*;

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
              stage: th::stage("Preparation"),
              player: PlayerExpr::Current, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::DealMove(
                    DealMove::DealQuantity(
                      Quantity::Int(IntExpr::Int(12)), 
                      CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Stock")))), 
                      Status::Private, 
                      CardSet::GroupOfPlayerCollection(Group::Location(th::location("Hand")), PlayerCollection::Quantifier(Quantifier::All))
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
                th::playername("P1"),
                th::playername("P2"),
                th::playername("P3"),
              ]
            )
          ),
          // create turnorder
          FlowComponent::Rule(
            Rule::CreateTurnorder(
              vec![
                th::playername("P1"),
                th::playername("P2"),
                th::playername("P3"),
              ]
            )
          ),
          // location on all
          FlowComponent::Rule(
            Rule::CreateLocationCollectionOnPlayerCollection(
              LocationCollection {
                locations: vec![
                  th::location("Hand"),
                  th::location("LayDown"),
                  th::location("Trash"),
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
                  th::location("Stock"),
                  th::location("Discard"),
                ]
              }
            )
          ),
          // card on
          FlowComponent::Rule(
            Rule::CreateCardOnLocation(
              th::location("Stock"),
              Types {
                types: vec![
                  (th::key("Rank"), vec![
                    th::value("Two"),
                    th::value("Three"),
                    th::value("Four"),
                    th::value("Five"),
                    th::value("Six"),
                    th::value("Seven"),
                    th::value("Eight"),
                    th::value("Nine"),
                    th::value("Ten"),
                    th::value("Jack"),
                    th::value("Queen"),
                    th::value("King"),
                    th::value("Ace")
                  ]),
                  (th::key("Suite"), vec![
                    th::value("Diamonds"),
                    th::value("Hearts"),
                    th::value("Spades"),
                    th::value("Clubs"),
                  ]),
                ]
              }
            )
          ),
          // RankOrder
          FlowComponent::Rule(
            Rule::CreatePrecedence(
              th::precedence("RankOrder"),
              vec![
                (th::key("Rank"), th::value("Ace")),
                (th::key("Rank"), th::value("Two")),
                (th::key("Rank"), th::value("Three")),
                (th::key("Rank"), th::value("Four")),
                (th::key("Rank"), th::value("Five")),
                (th::key("Rank"), th::value("Six")),
                (th::key("Rank"), th::value("Seven")),
                (th::key("Rank"), th::value("Eight")),
                (th::key("Rank"), th::value("Nine")),
                (th::key("Rank"), th::value("Ten")),
                (th::key("Rank"), th::value("Jack")),
                (th::key("Rank"), th::value("Queen")),
                (th::key("Rank"), th::value("King")),
              ]
            )
          ),
          // Values
          FlowComponent::Rule(
            Rule::CreatePointMap(
              th::pointmap("Values"),
              vec![
                (th::key("Rank"), th::value("Ace"), IntExpr::Int(1)),
                (th::key("Rank"), th::value("Two"), IntExpr::Int(2)),
                (th::key("Rank"), th::value("Three"), IntExpr::Int(3)),
                (th::key("Rank"), th::value("Four"), IntExpr::Int(4)),
                (th::key("Rank"), th::value("Five"), IntExpr::Int(5)),
                (th::key("Rank"), th::value("Six"), IntExpr::Int(6)),
                (th::key("Rank"), th::value("Seven"), IntExpr::Int(7)),
                (th::key("Rank"), th::value("Eight"), IntExpr::Int(8)),
                (th::key("Rank"), th::value("Nine"), IntExpr::Int(9)),
                (th::key("Rank"), th::value("Ten"), IntExpr::Int(10)),
                (th::key("Rank"), th::value("Jack"), IntExpr::Int(10)),
                (th::key("Rank"), th::value("Queen"), IntExpr::Int(10)),
                (th::key("Rank"), th::value("King"), IntExpr::Int(10)),
              ]
            )
          ),
          // Combo Sequence
          FlowComponent::Rule(
            Rule::CreateCombo(
              th::combo("Sequence"),
              FilterExpr::And(
                Box::new(FilterExpr::And(
                  Box::new(FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))),
                  Box::new(FilterExpr::Same(th::key("Suite")))
                )),
                Box::new(FilterExpr::Adjacent(th::key("Rank"), th::precedence("RankOrder")))
              )
            )
          ),
          // Combo Set
          FlowComponent::Rule(
            Rule::CreateCombo(
              th::combo("Set"),
              FilterExpr::And(
                Box::new(FilterExpr::And(
                  Box::new(FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))),
                  Box::new(FilterExpr::Distinct(th::key("Suite")))
                )),
                Box::new(FilterExpr::Same(th::key("Rank")))
              )
            )
          ),
          // Combo Set
          FlowComponent::Rule(
            Rule::CreateCombo(
              th::combo("Deadwood"),
              FilterExpr::And(
                Box::new(
                  FilterExpr::NotCombo(th::combo("Sequence"))
                ),
                Box::new(
                  FilterExpr::NotCombo(th::combo("Set"))
                )
              )
            )
          ),
          // Stage Preparation
          FlowComponent::Stage(
            SeqStage {
              stage: th::stage("Preparation"), 
              player: th::CURRENT, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::DealMove(
                    DealMove::DealQuantity(
                      Quantity::Int(IntExpr::Int(12)), 
                      CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Stock")))), 
                      Status::Private, 
                      CardSet::GroupOfPlayerCollection(Group::Location(th::location("Hand")), PlayerCollection::Quantifier(Quantifier::All))
                    )
                  )
                )
              ] 
            }
          ),
          // Stage Collect
          FlowComponent::Stage(
            SeqStage {
              stage: th::stage("Collect"), 
              player: th::CURRENT, 
              end_condition: EndCondition::UntilBool(BoolExpr::OutOfStagePlayer(th::PREVIOUS)), 
              flows: vec![
                // Choose
                FlowComponent::ChoiceRule(
                  ChoiceRule {
                    options: vec![
                      // move top of Discard to Hand
                      FlowComponent::Rule(
                        Rule::ClassicMove(
                          ClassicMove::Move(
                            CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Discard")))),
                            Status::Private,
                            CardSet::Group(Group::Location(th::location("Hand")))
                          )
                        )
                      ),
                      // move top of Stock to Hand
                      FlowComponent::Rule(
                        Rule::ClassicMove(
                          ClassicMove::Move(
                            CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Stock")))),
                            Status::Private,
                            CardSet::Group(Group::Location(th::location("Hand")))
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
                      CardSet::Group(Group::Location(th::location("Hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Discard")))),
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
                              th::combo("Deadwood"),
                              th::location("Hand")
                            )
                          )
                        ), 
                        th::pointmap("Values")
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
                                  CardSet::Group(Group::ComboInLocation(th::combo("Set"), th::location("Hand"))),
                                  Status::FaceUp,
                                  CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("LayDown")))),
                                )
                              )
                            ),
                            FlowComponent::Rule(
                              Rule::ClassicMove(
                                ClassicMove::MoveQuantity(
                                  Quantity::Quantifier(Quantifier::All),
                                  CardSet::Group(Group::ComboInLocation(th::combo("Sequence"), th::location("Hand"))),
                                  Status::FaceUp,
                                  CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("LayDown")))),
                                )
                              )
                            ),
                            // If rule
                            FlowComponent::IfRule(
                              IfRule {
                                condition: BoolExpr::CardSetIsEmpty(
                                  CardSet::Group(
                                    Group::Location(th::location("Hand"))
                                  )
                                ),
                                flows: vec![
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::MoveQuantity(
                                        Quantity::Quantifier(Quantifier::All),
                                        CardSet::GroupOfPlayer(Group::ComboInLocation(th::combo("Set"), th::location("Hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::CardPosition(CardPosition::Top(th::location("LayDown"))), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::MoveQuantity(
                                        Quantity::Quantifier(Quantifier::All),
                                        CardSet::GroupOfPlayer(Group::ComboInLocation(th::combo("Sequence"), th::location("Hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::CardPosition(CardPosition::Top(th::location("LayDown"))), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::Move(
                                        CardSet::GroupOfPlayer(Group::Location(th::location("Hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::Location(th::location("Trash")), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::Move(
                                        CardSet::Group(Group::Location(th::location("Hand"))),
                                        Status::FaceUp,
                                        CardSet::Group(Group::Location(th::location("Trash"))),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::PlayerOutOfStageAction(
                                      th::CURRENT
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
              stage: th::stage("FinalLayDown"), 
              player: th::CURRENT, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::Move(
                      CardSet::GroupOfPlayer(Group::Location(th::location("LayDown")), th::PREVIOUS),
                      Status::FaceUp,
                      CardSet::GroupOfPlayer(Group::Location(th::location("Hand")), th::CURRENT),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::MoveQuantity(
                      Quantity::Quantifier(Quantifier::All),
                      CardSet::Group(Group::ComboInLocation(th::combo("Set"), th::location("Hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("LayDown")))),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::MoveQuantity(
                      Quantity::Quantifier(Quantifier::All),
                      CardSet::Group(Group::ComboInLocation(th::combo("Sequence"), th::location("Hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("LayDown")))),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::Move(
                      CardSet::Group(Group::Location(th::location("Hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::Location(th::location("Trash"))),
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
                        th::location("Trash")
                      )
                    )
                  ),
                  th::pointmap("Values")
                ),
                th::memory("LeftOver"),
                PlayerCollection::Quantifier(Quantifier::All),
              )
            )
          ),
          FlowComponent::Rule(
            Rule::WinnerRule(
              WinnerRule::WinnerLowestMemory(
                th::memory("LeftOver")
              )
            )
          ),
        ]
      }
    );

    show_graph(&fsm, "game");
  }
  
}