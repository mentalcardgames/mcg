mod test {
  use ast::{analyzer::analyzer::analyze_ast, asts::ast::*};
  use syn::parse_str;


  #[test]
  fn analyzer_valid_create_player() {
    let parsed: Game = parse_str(
      "players: (P1, P2, P3);"
    ).unwrap();

    let res = analyze_ast(&parsed);

    println!("{:?}", res)
  }
  
  #[test]
  fn analyzer_fail_create_player() {
    let parsed: Game = parse_str(
      "players: (P1, P1, P3);"
    ).unwrap();

    let res = analyze_ast(&parsed);

    println!("{:?}", res)
  }

  #[test]
  fn analyzer_fail_create_team() {
    let parsed: Game = parse_str(
      "team T1: (P1, P2, P3);"
    ).unwrap();

    let res = analyze_ast(&parsed);

    println!("{:?}", res)
  }

  #[test]
  fn analyzer_fail_create_team_game() {
    let parsed: Game = parse_str(
      "
        players: (P1, P2, P3);
        team T1: (P1, P2, P3);
      "
    ).unwrap();

    let res = analyze_ast(&parsed);

    println!("{:?}", res)
  }

  #[test]
  fn analyzer_valid_game() {
    let parsed: Game = parse_str(
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

    let res = analyze_ast(&parsed);

    println!("{:?}", res)
  }
}