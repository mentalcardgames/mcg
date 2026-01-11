mod test {

  use std::{path::Path, process::Command};
  use quote::format_ident;

  use ast::ast::*;

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
              stage: format_ident!("Preparation"), 
              player: PlayerExpr::Current, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::DealMove(
                    DealMove::DealQuantity(
                      Quantity::Int(IntExpr::Int(12)), 
                      CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("stock")))), 
                      Status::Private, 
                      CardSet::GroupOfPlayerCollection(Group::Location(format_ident!("hand")), PlayerCollection::Quantifier(Quantifier::All))
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
                format_ident!("P1"),
                format_ident!("P2"),
                format_ident!("P3"),
              ]
            )
          ),
          // create turnorder
          FlowComponent::Rule(
            Rule::CreateTurnorder(
              vec![
                format_ident!("P1"),
                format_ident!("P2"),
                format_ident!("P3"),
              ]
            )
          ),
          // location on all
          FlowComponent::Rule(
            Rule::CreateLocationCollectionOnPlayerCollection(
              LocationCollection {
                locations: vec![
                  format_ident!("hand"),
                  format_ident!("laydown"),
                  format_ident!("trash"),
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
                  format_ident!("stock"),
                  format_ident!("discard"),
                ]
              }
            )
          ),
          // card on
          FlowComponent::Rule(
            Rule::CreateCardOnLocation(
              format_ident!("stock"),
              Types {
                types: vec![
                  (format_ident!("Rank"), vec![
                    format_ident!("Two"),
                    format_ident!("Three"),
                    format_ident!("Four"),
                    format_ident!("Five"),
                    format_ident!("Six"),
                    format_ident!("Seven"),
                    format_ident!("Eight"),
                    format_ident!("Nine"),
                    format_ident!("Ten"),
                    format_ident!("Jack"),
                    format_ident!("Queen"),
                    format_ident!("King"),
                    format_ident!("Ace")
                  ]),
                  (format_ident!("Suite"), vec![
                    format_ident!("Diamonds"),
                    format_ident!("Hearts"),
                    format_ident!("Spades"),
                    format_ident!("Clubs"),
                  ]),
                ]
              }
            )
          ),
          // RankOrder
          FlowComponent::Rule(
            Rule::CreatePrecedence(
              format_ident!("RankOrder"),
              vec![
                (format_ident!("Rank"), format_ident!("Ace")),
                (format_ident!("Rank"), format_ident!("Two")),
                (format_ident!("Rank"), format_ident!("Three")),
                (format_ident!("Rank"), format_ident!("Four")),
                (format_ident!("Rank"), format_ident!("Five")),
                (format_ident!("Rank"), format_ident!("Six")),
                (format_ident!("Rank"), format_ident!("Seven")),
                (format_ident!("Rank"), format_ident!("Eight")),
                (format_ident!("Rank"), format_ident!("Nine")),
                (format_ident!("Rank"), format_ident!("Ten")),
                (format_ident!("Rank"), format_ident!("Jack")),
                (format_ident!("Rank"), format_ident!("Queen")),
                (format_ident!("Rank"), format_ident!("King")),
              ]
            )
          ),
          // Values
          FlowComponent::Rule(
            Rule::CreatePointMap(
              format_ident!("Values"),
              vec![
                (format_ident!("Rank"), format_ident!("Ace"), IntExpr::Int(1)),
                (format_ident!("Rank"), format_ident!("Two"), IntExpr::Int(2)),
                (format_ident!("Rank"), format_ident!("Three"), IntExpr::Int(3)),
                (format_ident!("Rank"), format_ident!("Four"), IntExpr::Int(4)),
                (format_ident!("Rank"), format_ident!("Five"), IntExpr::Int(5)),
                (format_ident!("Rank"), format_ident!("Six"), IntExpr::Int(6)),
                (format_ident!("Rank"), format_ident!("Seven"), IntExpr::Int(7)),
                (format_ident!("Rank"), format_ident!("Eight"), IntExpr::Int(8)),
                (format_ident!("Rank"), format_ident!("Nine"), IntExpr::Int(9)),
                (format_ident!("Rank"), format_ident!("Ten"), IntExpr::Int(10)),
                (format_ident!("Rank"), format_ident!("Jack"), IntExpr::Int(10)),
                (format_ident!("Rank"), format_ident!("Queen"), IntExpr::Int(10)),
                (format_ident!("Rank"), format_ident!("King"), IntExpr::Int(10)),
              ]
            )
          ),
          // Combo Sequence
          FlowComponent::Rule(
            Rule::CreateCombo(
              format_ident!("Sequence"),
              FilterExpr::And(
                Box::new(FilterExpr::And(
                  Box::new(FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))),
                  Box::new(FilterExpr::Same(format_ident!("Suite")))
                )),
                Box::new(FilterExpr::Adjacent(format_ident!("Rank"), format_ident!("RankOrder")))
              )
            )
          ),
          // Combo Set
          FlowComponent::Rule(
            Rule::CreateCombo(
              format_ident!("Set"),
              FilterExpr::And(
                Box::new(FilterExpr::And(
                  Box::new(FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))),
                  Box::new(FilterExpr::Distinct(format_ident!("Suite")))
                )),
                Box::new(FilterExpr::Same(format_ident!("Rank")))
              )
            )
          ),
          // Combo Set
          FlowComponent::Rule(
            Rule::CreateCombo(
              format_ident!("Deadwood"),
              FilterExpr::And(
                Box::new(
                  FilterExpr::NotCombo(format_ident!("Sequence"))
                ),
                Box::new(
                  FilterExpr::NotCombo(format_ident!("Set"))
                )
              )
            )
          ),
          // Stage Preparation
          FlowComponent::Stage(
            SeqStage {
              stage: format_ident!("Preparation"), 
              player: PlayerExpr::Current, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::DealMove(
                    DealMove::DealQuantity(
                      Quantity::Int(IntExpr::Int(12)), 
                      CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("stock")))), 
                      Status::Private, 
                      CardSet::GroupOfPlayerCollection(Group::Location(format_ident!("hand")), PlayerCollection::Quantifier(Quantifier::All))
                    )
                  )
                )
              ] 
            }
          ),
          // Stage Collect
          FlowComponent::Stage(
            SeqStage {
              stage: format_ident!("Collect"), 
              player: PlayerExpr::Current, 
              end_condition: EndCondition::UntilBool(BoolExpr::OutOfStagePlayer(PlayerExpr::Previous)), 
              flows: vec![
                // Choose
                FlowComponent::ChoiceRule(
                  ChoiceRule {
                    options: vec![
                      // move top of discard to hand
                      FlowComponent::Rule(
                        Rule::ClassicMove(
                          ClassicMove::Move(
                            CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("discard")))),
                            Status::Private,
                            CardSet::Group(Group::Location(format_ident!("hand")))
                          )
                        )
                      ),
                      // move top of stock to hand
                      FlowComponent::Rule(
                        Rule::ClassicMove(
                          ClassicMove::Move(
                            CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("stock")))),
                            Status::Private,
                            CardSet::Group(Group::Location(format_ident!("hand")))
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
                      CardSet::Group(Group::Location(format_ident!("hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("discard")))),
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
                              format_ident!("Deadwood"),
                              format_ident!("hand")
                            )
                          )
                        ), 
                        format_ident!("Values")
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
                                  CardSet::Group(Group::ComboInLocation(format_ident!("Set"), format_ident!("hand"))),
                                  Status::FaceUp,
                                  CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("laydown")))),
                                )
                              )
                            ),
                            FlowComponent::Rule(
                              Rule::ClassicMove(
                                ClassicMove::MoveQuantity(
                                  Quantity::Quantifier(Quantifier::All),
                                  CardSet::Group(Group::ComboInLocation(format_ident!("Sequence"), format_ident!("hand"))),
                                  Status::FaceUp,
                                  CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("laydown")))),
                                )
                              )
                            ),
                            // If rule
                            FlowComponent::IfRule(
                              IfRule {
                                condition: BoolExpr::CardSetIsEmpty(
                                  CardSet::Group(
                                    Group::Location(format_ident!("hand"))
                                  )
                                ),
                                flows: vec![
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::MoveQuantity(
                                        Quantity::Quantifier(Quantifier::All),
                                        CardSet::GroupOfPlayer(Group::ComboInLocation(format_ident!("Set"), format_ident!("hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::CardPosition(CardPosition::Top(format_ident!("laydown"))), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::MoveQuantity(
                                        Quantity::Quantifier(Quantifier::All),
                                        CardSet::GroupOfPlayer(Group::ComboInLocation(format_ident!("Sequence"), format_ident!("hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::CardPosition(CardPosition::Top(format_ident!("laydown"))), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::Move(
                                        CardSet::GroupOfPlayer(Group::Location(format_ident!("hand")), PlayerExpr::Next),
                                        Status::FaceUp,
                                        CardSet::GroupOfPlayer(Group::Location(format_ident!("trash")), PlayerExpr::Next),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::ClassicMove(
                                      ClassicMove::Move(
                                        CardSet::Group(Group::Location(format_ident!("hand"))),
                                        Status::FaceUp,
                                        CardSet::Group(Group::Location(format_ident!("trash"))),
                                      )
                                    )
                                  ),
                                  FlowComponent::Rule(
                                    Rule::PlayerOutOfStageAction(
                                      PlayerExpr::Current
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
              stage: format_ident!("FinalLayDown"), 
              player: PlayerExpr::Current, 
              end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
              flows: vec![
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::Move(
                      CardSet::GroupOfPlayer(Group::Location(format_ident!("laydown")), PlayerExpr::Previous),
                      Status::FaceUp,
                      CardSet::GroupOfPlayer(Group::Location(format_ident!("hand")), PlayerExpr::Current),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::MoveQuantity(
                      Quantity::Quantifier(Quantifier::All),
                      CardSet::Group(Group::ComboInLocation(format_ident!("Set"), format_ident!("hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("laydown")))),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::MoveQuantity(
                      Quantity::Quantifier(Quantifier::All),
                      CardSet::Group(Group::ComboInLocation(format_ident!("Sequence"), format_ident!("hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::CardPosition(CardPosition::Top(format_ident!("laydown")))),
                    )
                  )
                ),
                FlowComponent::Rule(
                  Rule::ClassicMove(
                    ClassicMove::Move(
                      CardSet::Group(Group::Location(format_ident!("hand"))),
                      Status::FaceUp,
                      CardSet::Group(Group::Location(format_ident!("trash"))),
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
                        format_ident!("trash")
                      )
                    )
                  ),
                  format_ident!("Values")
                ),
                format_ident!("LeftOver"),
                PlayerCollection::Quantifier(Quantifier::All),
              )
            )
          ),
          FlowComponent::Rule(
            Rule::WinnerRule(
              WinnerRule::WinnerLowestMemory(
                format_ident!("LeftOver")
              )
            )
          ),
        ]
      }
    );

    show_graph(&fsm, "game");
  }
  
}