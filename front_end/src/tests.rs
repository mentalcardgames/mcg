use std::path::Path;
use std::process::Command;

use crate::fsm_to_dot::fsm_to_dot;
use crate::ir::{Ir, IrBuilder, SpannedPayload};
use crate::lower::Lower;
use crate::walker::*;
// use crate::{lower::Lower};
use pest_consume::*;
use crate::parser::{CGDSLParser, Rule, Node, Result};

pub fn test_rule_consume<T, F>(
    input: &str, 
    rule: Rule, 
    mapper: F
) -> Result<T> // Returns pest_consume::Result
where 
    F: FnOnce(Node) -> Result<T>,
    // T: Lower<L>,
    T: Walker,
{
    // 1. Parsing: pest_consume::parse already returns Result<Nodes, Error<Rule>>
    let nodes = CGDSLParser::parse(rule, input)?;
    
    // 2. Extract Single Node: .single() returns Result<Node, Error<Rule>>
    let node = nodes.single()?;

    // 3. Mapping: mapper returns Result<T, Error<Rule>>
    let parsed_ast = mapper(node)?;

    // // 4. Lowering: Convert custom logic errors to pest_consume::Error
    // let result = parsed_ast.lower();

    Ok(parsed_ast)
}

fn parse_game(input: &str) {
    test_rule_consume(
        input,
        Rule::file,
        CGDSLParser::file,
    ).expect("parse failed");
}

// ===========================================================================
// Run one test game (one benchmark)
// ===========================================================================
// #[test]
// fn test_prase_rule() {
//   parse_game(
//     "
//       set current out of stage
//     "
//   );
// }

// #[test]
// fn test_parse_optional() {
//   parse_game(
//     "
//       optional {
//         set current out of stage
//       }
//     "
//   );
// }

// #[test]
// fn test_parse_if() {
//   parse_game(
//     "
//       if (Hand is empty) {
//         set current out of stage
//       }
//     "
//   );
// }

// #[test]
// fn test_parse_stage() {
//   parse_game(
// "
//     stage Preparation for current 1 times {
//       deal 12 from Stock top private to Hand of all
//     }      
//     "
//   );
// }

// #[test]
// fn test_parse_choose() {
//   parse_game(
// "
//     choose {
//       move Discard top private to Hand
//       or
//       move Stock top private to Hand
//     }
//     "
//   );
// }

// #[test]
// fn test_parse_conditional() {
//   parse_game(
// "
//     conditional {
//       case:
//         move Discard top private to Hand
//       case Hand is empty:
//         move Stock top private to Hand
//       case else:
//         move Stock top private to Hand 
//     }
//     "
//   );
// }

// #[test]
// fn test_parse_game() {
//   parse_game( 
//     "
//         player P1, P2, P3
//         turnorder (P1, P2, P3)
//         location Hand, LayDown, Trash on all
//         location Stock, Discard on table
//         card on Stock:
//           Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
//             for Suite(Diamonds, Hearts, Spades, Clubs)
//         precedence RankOrder on Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King)
//         points Values on Rank(Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6, Seven: 7, Eight: 8, Nine: 9 , Ten: 10, Jack: 10, Queen: 10, King: 10)
//         combo Sequence where ((size >= 3 and Suite same) and Rank adjacent using RankOrder)
//         combo Set where ((size >= 3 and Suite distinct) and Rank same)
//         combo Deadwood where (not Sequence and not Set)

//         stage Preparation for current 1 times {
//           deal 12 from Stock top private to Hand of all
//         }

//         stage Collect for current until previous out of stage  {
//           choose {
//             move Discard top private to Hand
//             or
//             move Stock top private to Hand
//           }

//           move any from Hand face up to Discard top

//           if (sum of Deadwood in Hand using Values <= 10) {
//             optional {
//               move all from Set in Hand face up to LayDown top
//               move all from Sequence in Hand face up to LayDown top

//               if (Hand is empty) {
//                 move all from Set in Hand of next face up to LayDown top of next
//                 move all from Sequence in Hand of next face up to LayDown top of next
//                 move Hand of next face up to Trash of next

//                 move Hand face up to Trash
//                 set current out of stage
//               }
//             }
//           }

//           cycle to next
//         }

//         stage FinalLayDown for current 1 times {
//           move LayDown of previous face up to Hand of current
//           move all from Set in Hand face up to LayDown top
//           move all from Sequence in Hand face up to LayDown top

//           move Hand face up to Trash
//         }

//         score sum of Trash using Values to LeftOver of all
//         winner is lowest LeftOver
//     "
//   );
// }

// ===========================================================================
// Test IR builder
// ===========================================================================
fn build_ir_from(input: &str) -> Ir<SpannedPayload> {
    let mut builder: IrBuilder<SpannedPayload> = IrBuilder::default();

    let game = test_rule_consume(
        input,
        Rule::file,
        CGDSLParser::file,
    ).expect("parse failed");

    println!("{}", game.lower());

    builder.build_ir(&game);
    builder.fsm
}

fn parse_ast_parse(input: &str) {
    let game = match test_rule_consume(
        input,
        Rule::file,
        CGDSLParser::file,
    ) {
      Ok(a) => a,
      Err(e) => {
        println!("{}", input);
        println!("{:?}", e);
        panic!("parse failed")
      }
    };

    // let fmt_game = &format!("{}", game.lower());
    
    // let parsed_fmt_game = test_rule_consume(
    //     fmt_game,
    //     Rule::file,
    //     CGDSLParser::file,
    // ).expect("parse failed");


    // assert_eq!(game.lower(), parsed_fmt_game.lower()); 
}

fn show_graph(fsm: &Ir<SpannedPayload>, name: &str) {
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
// fn test_rule_ir() {
//   let fsm = build_ir_from(
//     "
//       set current out of stage
//     "
//   );
//   show_graph(&fsm, "rule");
// }

// #[test]
// fn test_optional_ir() {
//   let fsm = build_ir_from(
//     "
//       optional {
//         set current out of stage
//       }
//     "
//   );
//   show_graph(&fsm, "optional");
// }

// #[test]
// fn test_if_ir() {
//   let fsm = build_ir_from(
//     "
//       if (Hand is empty) {
//         set current out of stage
//       }
//     "
//   );
//   show_graph(&fsm, "if");
// }

// #[test]
// fn test_stage_ir() {
//   let fsm = build_ir_from(
// "
//     stage Preparation for current 1 times {
//       deal 12 from Stock top private to Hand of all
//     }      
//     "
//   );
//   show_graph(&fsm, "stage");
// }

// #[test]
// fn test_choose_ir() {
//   let fsm = build_ir_from(
// "
//     choose {
//       move Discard top private to Hand
//       or
//       move Stock top private to Hand
//     }
//     "
//   );
//   show_graph(&fsm, "choose");
// }

// #[test]
// fn test_conditional_ir() {
//   let fsm = build_ir_from(
// "
//     conditional {
//       case:
//         move Discard top private to Hand
//       case Hand is empty:
//         move Stock top private to Hand
//       case else:
//         move Stock top private to Hand 
//     }
//     "
//   );
//   show_graph(&fsm, "conditional");
// }

#[test]
fn test_game_ir() {
  let fsm = build_ir_from(
"
          player P1, P2, P3
          turnorder (P1, P2, P3)
          location Hand, LayDown, Trash on all
          location Stock, Discard on table
          card on Stock:
            Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
              for Suite(Diamonds, Hearts, Spades, Clubs)
          precedence RankOrder on Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King)
          points Values on Rank(Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6, Seven: 7, Eight: 8, Nine: 9 , Ten: 10, Jack: 10, Queen: 10, King: 10)
          combo Sequence where ((size >= 3 and same Suite) and adjacent Rank using RankOrder)
          combo Set where ((size >= 3 and distinct Suite) and Rank s== \"Ace\")
          combo Deadwood where (not Sequence and not Set)

          stage Preparation for current 1 times {
            deal 12 from top(Stock) private to Hand of all
          }

          stage Collect for current until previous out of stage  {
            choose {
              move top(Discard) private to Hand
              or
              move top(Stock) private to Hand
            }

            move any from Hand face up to top(Discard)

            if (sum of Deadwood in Hand using Values <= 10) {
              optional {
                move all from Set in Hand face up to top(LayDown)
                move all from Sequence in Hand face up to top(LayDown)

                if (Hand is empty) {
                  move all from Set in Hand of next face up to top(LayDown) of next
                  move all from Sequence in Hand of next face up to top(LayDown) of next
                  move Hand of next face up to Trash of next

                  move Hand face up to Trash
                  set current out of stage
                }
              }
            }

            cycle to next
          }

          stage FinalLayDown for current 1 times {
            move LayDown of previous face up to Hand of current
            move all from Set in Hand face up to top(LayDown)
            move all from Sequence in Hand face up to top(LayDown)

            move Hand face up to Trash
          }

          memory LeftOver on all

          score sum of Trash using Values to LeftOver of all
          winner is lowest LeftOver
      "
  );
  show_graph(&fsm, "game");
}

// ===========================================================================
// Parser tests
// ===========================================================================
use proptest::proptest;
use crate::ast::*;
use proptest_arbitrary_interop::arb;
use proptest::test_runner::Config;

proptest! {
    #[test]
    fn parser_never_panics(s in ".*") {
        let _ = CGDSLParser::parse(Rule::file, &s);
    }
}

proptest! {
    #![proptest_config(Config {
        cases: 100000,
        .. Config::default()
    })]

    #[test]
    fn test_game(expr in arb::<Game>()) {
      parse_ast_parse(&format!("{}", expr));
    }

    #[test]
    fn test_seq_stage(expr in arb::<SeqStage>()) {
      parse_ast_parse(&format!("{}", expr));
    }

    #[test]
    fn test_if_rule(expr in arb::<IfRule>()) {
      parse_ast_parse(&format!("{}", expr));
    }

    #[test]
    fn test_optional_rule(expr in arb::<OptionalRule>()) {
      parse_ast_parse(&format!("{}", expr));
    }

    #[test]
    fn test_choice_rule(expr in arb::<ChoiceRule>()) {
      parse_ast_parse(&format!("{}", expr));
    }

    #[test]
    fn test_conditional(expr in arb::<Conditional>()) {
      parse_ast_parse(&format!("{}", expr));
    }

    #[test]
    fn test_game_rule(expr in arb::<GameRule>()) {
      parse_ast_parse(&format!("{}", expr));
    }
}

#[test]
fn parse_int_collection() {
  let input = "&Id832 c!= &Id581 of other teams";
  match test_rule_consume(
      input,
      Rule::bool_expr,
      CGDSLParser::bool_expr,
  ) {
    Ok(a) => {
      println!("{:?}", a.lower());
      a
    },
    Err(e) => {
      println!("{}", input);
      println!("{:?}", e);
      panic!("parse failed")
    }
  };
}

#[test]
fn test_arbitrary_builds() {
    use arbitrary::{Arbitrary, Unstructured};

    // Some random bytes
    let data: Vec<u8> = (0..4096).map(|i| i as u8).collect();

    let mut u = Unstructured::new(&data);

    let result = GameRule::arbitrary(&mut u);

    println!("{:?}", result);
}

#[test]
fn test_reparse_game() {
  parse_ast_parse(
    "
          player P1, P2, P3
          turnorder (P1, P2, P3)
          location Hand, LayDown, Trash on all
          location Stock, Discard on table
          card on Stock:
            Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
              for Suite(Diamonds, Hearts, Spades, Clubs)
          precedence RankOrder on Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King)
          points Values on Rank(Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6, Seven: 7, Eight: 8, Nine: 9 , Ten: 10, Jack: 10, Queen: 10, King: 10)
          combo Sequence where ((size >= 3 and same Suite) and adjacent Rank using RankOrder)
          combo Set where ((size >= 3 and distinct Suite) and Rank s== \"Ace\")
          combo Deadwood where (not Sequence and not Set)

          stage Preparation for current 1 times {
            deal 12 from top(Stock) private to Hand of all
          }

          stage Collect for current until previous out of stage  {
            choose {
              move top(Discard) private to Hand
              or
              move top(Stock) private to Hand
            }

            move any from Hand face up to top(Discard)

            if (sum of Deadwood in Hand using Values <= 10) {
              optional {
                move all from Set in Hand face up to top(LayDown)
                move all from Sequence in Hand face up to top(LayDown)

                if (Hand is empty) {
                  move all from Set in Hand of next face up to top(LayDown) of next
                  move all from Sequence in Hand of next face up to top(LayDown) of next
                  move Hand of next face up to Trash of next

                  move Hand face up to Trash
                  set current out of stage
                }
              }
            }

            cycle to next
          }

          stage FinalLayDown for current 1 times {
            move LayDown of previous face up to Hand of current
            move all from Set in Hand face up to top(LayDown)
            move all from Sequence in Hand face up to top(LayDown)

            move Hand face up to Trash
          }

          memory LeftOver on all

          score sum of Trash using Values to LeftOver of all
          winner is lowest LeftOver
      "
  );
}
