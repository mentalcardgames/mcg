mod test {
  use std::{path::Path, process::Command};

  use ast::parse::ast_to_typed_ast::Lower;
  use ast::parse::ast_to_typed_ast::LoweringCtx;
  use ast::analyzer::type_analyzer::ctx;

  use ir::fsm::*;
  use ir::fsm_to_dot::*;
  use syn::parse_str;

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

  // #[test]
  // fn test_rule() {
  //   let mut builder = FSMBuilder::default();

  //   let fsm = builder.build_fsm(
  //     Game { 
  //       flows: vec![
  //         FlowComponent::Rule(Rule::EndTurn)
  //       ] 
  //     }
  //   );

  //   show_graph(&fsm, "rule");    
  // }

  // #[test]
  // fn test_if_rule() {
  //   let mut builder = FSMBuilder::default();

  //   let fsm = builder.build_fsm(
  //     Game { 
  //       flows: vec![
  //         FlowComponent::IfRule(
  //           IfRule {
  //             condition: BoolExpr::OutOfStagePlayer(PlayerExpr::Current),
  //             flows: vec![
  //               FlowComponent::Rule(
  //                 Rule::CycleAction(
  //                   PlayerExpr::Next
  //                 )
  //               )
  //             ]
  //           }
  //         )
  //       ] 
  //     }
  //   );

  //   show_graph(&fsm, "if_rule");
  // }

  // #[test]
  // fn test_optional_rule() {
  //   let mut builder = FSMBuilder::default();

  //   let fsm = builder.build_fsm(
  //     Game { 
  //       flows: vec![
  //         FlowComponent::OptionalRule(
  //           OptionalRule {
  //             flows: vec![
  //               FlowComponent::Rule(
  //                 Rule::EndTurn
  //               )
  //             ]
  //           }
  //         )
  //       ] 
  //     }
  //   );

  //   show_graph(&fsm, "optional_rule");
  // }

  // #[test]
  // fn test_choice_rule() {
  //   let mut builder = FSMBuilder::default();

  //   let fsm = builder.build_fsm(
  //     Game { 
  //       flows: vec![
  //         FlowComponent::ChoiceRule(
  //           ChoiceRule {
  //             options: vec![
  //               FlowComponent::Rule(
  //                 Rule::EndTurn
  //               ),
  //               FlowComponent::OptionalRule(
  //                 OptionalRule {
  //                   flows: vec![
  //                       FlowComponent::Rule(
  //                         Rule::EndTurn
  //                       )
  //                   ]
  //                 }
  //               ),
  //             ]
  //           }
  //         )
  //       ] 
  //     }
  //   );

  //   show_graph(&fsm, "choice_rule");
  // }

  // #[test]
  // fn test_stage() {
  //   let mut builder = FSMBuilder::default();

  //   let fsm = builder.build_fsm(
  //     Game { 
  //       flows: vec![
  //         FlowComponent::Stage(
  //           SeqStage {
  //             stage: th::stage("Preparation"),
  //             player: PlayerExpr::Current, 
  //             end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
  //             flows: vec![
  //               FlowComponent::Rule(
  //                 Rule::DealMove(
  //                   DealMove::DealQuantity(
  //                     Quantity::Int(IntExpr::Int(12)), 
  //                     CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Stock")))), 
  //                     Status::Private, 
  //                     CardSet::GroupOfPlayerCollection(Group::Location(th::location("Hand")), PlayerCollection::Quantifier(Quantifier::All))
  //                   )
  //                 )
  //               )
  //             ] 
  //           }
  //         )
  //       ] 
  //     }
  //   );

  //   show_graph(&fsm, "stage");
  // }
  
  #[test]
  fn test_game() {
    let mut builder = FSMBuilder::default();

    let game: ast::asts::ast::Game = parse_str(
      "
        players: (P1, P2, P3);
        turnorder: (P1, P2, P3);
        location (Hand, LayDown, Trash) on all;
        location (Stock, Discard) on table;
        card on Stock:
          Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
            for Suite(Diamonds, Hearts, Spades, Clubs);
        precedence RankOrder on Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King);
        pointmap Values on Rank(Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6, Seven: 7, Eight: 8, Nine: 9 , Ten: 10, Jack: 10, Queen: 10, King: 10);
        combo Sequence where ((size >= 3 and same Suite) and adjacent Rank using RankOrder);
        combo Set where ((size >= 3 and distinct Suite) and same Rank);
        combo Deadwood where (not Sequence and not Set);

        stage Preparation for current until(1 times) {
          deal 12 from top(Stock) private to Hand of all;
        }

        stage Collect for current until(previous out of stage) {
          choose {
            move top(Discard) private to Hand;
            or
            move top(Stock) private to Hand;
          }

          move any from Hand face up to top(Discard);

          if (sum of Deadwood in Hand using Values <= 10) {
            optional {
              move all from Set in Hand face up to top(LayDown);
              move all from Sequence in Hand face up to top(LayDown);

              if (Hand is empty) {
                move all from Set in Hand of next face up to top(LayDown) of next;
                move all from Sequence in Hand of next face up to top(LayDown) of next;
                move Hand of next face up to Trash of next;

                move Hand face up to Trash;
                set current out of stage;
              }
            }
          }

          cycle to next;
        }

        stage FinalLayDown for current until(1 times) {
          move LayDown of previous face up to Hand of current;
          move all from Set in Hand face up to top(LayDown);
          move all from Sequence in Hand face up to top(LayDown);

          move Hand face up to Trash;
        }

        memory LeftOver on all;
        score sum of Trash using Values to LeftOver of all;
        winner is lowest LeftOver;
      "
    ).unwrap();


    let ctx = ctx(&game);
    let lowering_ctx = LoweringCtx::new(ctx);

    let typed_game = game.lower(&lowering_ctx).unwrap();

    let fsm = builder.build_fsm(
      typed_game
    );

    show_graph(&fsm, "game");
  }
  
}