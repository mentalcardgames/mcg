/// The testing consists of two types of testing:
/// - Unit-Tests -> for special cases
/// - Generated-Tests -> Arbitrary-style tests (Proptests)
/// 
/// We need a lot of generated tests because pest is a recursive descent Parser.
/// 
/// In addition to this:
/// - Each AST-struct/-enum must have a fmt::Display trait that is the corresponding
/// Rule in the grammar.pest (e.g. PlayerExpr::Current => "current").
/// 
/// This way we can check for mistakes in the AST-declaration and further Parsing errors
/// by doing: Generate AST -> String-Represenation -> Parse -> assert_eq Generated-AST and Parse-Output

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::fsm_to_dot::fsm_to_dot;
use crate::ir::{Ir, IrBuilder, SpannedPayload};
use crate::lower::Lower;
use crate::parser::{CGDSLParser, Node, Result, Rule};
use crate::walker::*;
use pest_consume::*;

pub fn test_rule_consume<T, F>(input: &str, rule: Rule, mapper: F) -> Result<T>
// Returns pest_consume::Result
where
    F: FnOnce(Node) -> Result<T>,
    T: Walker,
{
    // 1. Use parse_with_userdata instead of parse
    // This ensures the Nodes contain RefCell<SymbolTable> instead of ()
    let nodes = CGDSLParser::parse(rule, input)?;

    // 2. Extract Single Node
    let node = nodes.single()?;

    // 3. Mapping
    let parsed_ast = mapper(node)?;

    Ok(parsed_ast)
}

pub fn test_rule_consume_with_table<T, F>(
    input: &str,
    rule: Rule,
    mapper: F,
) -> Result<T>
// Returns pest_consume::Result
where
    F: FnOnce(Node) -> Result<T>,
    // T: Lower<L>,
    T: Walker,
{
    // 2. Use parse_with_userdata instead of parse
    // This ensures the Nodes contain RefCell<SymbolTable> instead of ()
    let nodes = CGDSLParser::parse(rule, input)?;

    // 3. Extract Single Node
    let node = nodes.single()?;

    // 4. Mapping
    let parsed_ast = mapper(node)?;

    Ok(parsed_ast)
}

// ===========================================================================
// Test IR builder
// ===========================================================================
fn build_ir_from(input: &str) -> Ir<SpannedPayload> {
    let mut builder: IrBuilder<SpannedPayload> = IrBuilder::default();

    let game = test_rule_consume(input, Rule::file, CGDSLParser::file).expect("parse failed");

    println!("{}", game.lower());

    builder.build_ir(&game);
    builder.fsm
}

fn parse_ast_parse(input: &str) {
    let game = match test_rule_consume(input, Rule::file, CGDSLParser::file) {
        Ok(a) => a,
        Err(e) => {
            println!("{}", input);
            println!("{:?}", e);
            panic!("parse failed")
        }
    };

    let fmt_game = &format!("{}", game.lower());

    let parsed_fmt_game =
        test_rule_consume(fmt_game, Rule::file, CGDSLParser::file).expect("parse failed");

    assert_eq!(game.lower(), parsed_fmt_game.lower());
}

fn show_graph(fsm: &Ir<SpannedPayload>, name: &str) {
    // Make sure the output folder exists
    let out_dir = Path::new("../tests_out");
    if !out_dir.exists() {
        fs::create_dir_all(out_dir).expect("Failed to create output folder");
    }

    let dot_path = out_dir.join(format!("{}.dot", name));
    let png_path = out_dir.join(format!("{}.png", name));

    // Generate .dot file
    fsm_to_dot(&fsm, &dot_path).unwrap();

    // Call Graphviz to generate PNG
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
fn test_rule_ir() {
    let fsm = build_ir_from(
        "
      set current out of stage
    ",
    );

    assert_eq!(true, fsm.is_connected());

    show_graph(&fsm, "rule");
}

#[test]
fn test_optional_ir() {
    let fsm = build_ir_from(
        "
      optional {
        set current out of stage
      }
    ",
    );

    assert_eq!(true, fsm.is_connected());

    show_graph(&fsm, "optional");
}

#[test]
fn test_if_ir() {
    let fsm = build_ir_from(
        "
      if (Hand empty) {
        set current out of stage
      }
    ",
    );

    assert_eq!(true, fsm.is_connected());

    show_graph(&fsm, "if");
}

#[test]
fn test_stage_ir() {
    let fsm = build_ir_from(
        "
      stage Outside for current until end {
        deal 12 from top(Stock) private to Hand of all
        stage Preparation for current 1 times {
          deal 12 from top(Stock) private to Hand of all
          if (Hand empty) {
            end Outside
          }
        } 
      }
    ",
    );

    assert_eq!(true, fsm.is_connected());

    show_graph(&fsm, "stage");
}

#[test]
fn test_choose_ir() {
    let fsm = build_ir_from(
        "
    choose {
      move top(Discard) private to Hand
      or
      move top(Stock) private to Hand
    }
    ",
    );

    assert_eq!(true, fsm.is_connected());

    show_graph(&fsm, "choose");
}

#[test]
fn test_conditional_ir() {
    let fsm = build_ir_from(
        "
    conditional {
      case Hand empty:
        move top(Discard) private to Hand
      case Hand empty:
        move top(Stock) private to Hand
      case else:
        move top(Stock) private to Hand 
    }
    ",
    );

    assert_eq!(true, fsm.is_connected());

    show_graph(&fsm, "conditional");
}

#[test]
fn test_game_ir() {
    let fsm = build_ir_from(
"
          player P1, P2, P3
          turnorder (P:P1, P:P2, P:P3)
          location Hand, LayDown, Trash on all
          location Stock, Discard on table
          card on Stock:
            Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
              for Suite(Diamonds, Hearts, Spades, Clubs)
          precedence RankOrder on Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King)
          points Values on Rank(Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6, Seven: 7, Eight: 8, Nine: 9 , Ten: 10, Jack: 10, Queen: 10, King: 10)
          combo Sequence where ((size >= 3 and same Suite) and adjacent Rank using RankOrder)
          combo Set where ((size >= 3 and distinct Suite) and Rank is \"Ace\")
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

                if (Hand empty) {
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

    assert_eq!(true, fsm.is_connected());

    show_graph(&fsm, "game");
}

// ===========================================================================
// Proptests
// ===========================================================================
use crate::ast::*;
use proptest::proptest;
use proptest::test_runner::Config;
use proptest_arbitrary_interop::arb;

proptest! {
    #[test]
    fn parser_never_panics(s in ".*") {
        let _ = CGDSLParser::parse(Rule::file, &s);
    }
}

proptest! {
    #![proptest_config(Config {
        cases: 10000,
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
    fn test_sim_stage(expr in arb::<SimStage>()) {
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
fn test_reparse_game() {
    parse_ast_parse(
    "
          player P1, P2, P3
          turnorder (P:P1, P:P2, P:P3)
          location Hand, LayDown, Trash on all
          location Stock, Discard on table
          card on Stock:
            Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
              for Suite(Diamonds, Hearts, Spades, Clubs)
          precedence RankOrder on Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King)
          points Values on Rank(Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6, Seven: 7, Eight: 8, Nine: 9 , Ten: 10, Jack: 10, Queen: 10, King: 10)
          combo Sequence where ((size >= 3 and same Suite) and adjacent Rank using RankOrder)
          combo Set where ((size >= 3 and distinct Suite) and Rank is \"Ace\")
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

                if (Hand empty) {
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

// ===========================================================================
// If parsing for a rule fails (for experimenting and trying out)
// ===========================================================================
//  out of stage"
#[test]
fn test_specific_rule() {
    let input = "(&IC:Id608 of (&T:Id100 of playersin))";
    let _ = match test_rule_consume(input, Rule::int_collection, CGDSLParser::int_collection) {
        Ok(a) => {
            println!("{:?}", a);
        }
        Err(e) => {
            println!("{}", input);
            println!("{:?}", e);
            panic!("parse failed")
        }
    };
}
