#[cfg(test)]
mod tests {
    use crate::symbols::SymbolVisitor;
    use crate::walker::*;
    use crate::{lower::Lower};

    use pest_consume::*;

    use crate::parser::{CGDSLParser, Rule, Node, Result}; // Replace with your actual parser/rule imports

    pub fn test_rule_consume<T, L, F>(
        input: &str, 
        rule: Rule, 
        mapper: F
    ) -> Result<L> // Returns pest_consume::Result
    where 
        F: FnOnce(Node) -> Result<T>,
        T: Lower<L>,
        T: Walker,
    {
        // 1. Parsing: pest_consume::parse already returns Result<Nodes, Error<Rule>>
        let nodes = CGDSLParser::parse(rule, input)?;
        
        // 2. Extract Single Node: .single() returns Result<Node, Error<Rule>>
        let node = nodes.single()?;

        // Capture the span of the root node for error reporting later
        let root_span = node.as_span();

        // 3. Mapping: mapper returns Result<T, Error<Rule>>
        let parsed_ast = mapper(node)?;

        let mut symbol = SymbolVisitor::new();
        parsed_ast.walk(&mut symbol);

        // println!("{:?}", symbol.into_typed_vars());

        // match symbol.check_game_type() {
        //     Ok(_) => println!("Game is fine!"),
        //     Err(e) => println!("{:?}", e),
        // }

        // 4. Lowering: Convert custom logic errors to pest_consume::Error
        // let ctx = LoweringCtx::new(vec![]); 
        let result = parsed_ast.lower();

        Ok(result)
    }

    fn test_rule_no_lower<T, F>(
        input: &str, 
        rule: Rule, 
        mapper: F
    ) -> Result<T> // Returns pest_consume::Result
    where 
        F: FnOnce(Node) -> Result<T>,
        // We assume Lowering provides a way to get a span or we use the node's span
    {
        // 1. Parsing: pest_consume::parse already returns Result<Nodes, Error<Rule>>
        let nodes = CGDSLParser::parse(rule, input)?;
        
        // 2. Extract Single Node: .single() returns Result<Node, Error<Rule>>
        let node = nodes.single()?;

        // Capture the span of the root node for error reporting later
        let root_span = node.as_span();

        // 3. Mapping: mapper returns Result<T, Error<Rule>>
        let result = mapper(node)?;

        Ok(result)
    }

    #[test]
    fn test_game() {
      let input = 
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
            combo Sequence where ((size >= 3 and Suite same) and Rank adjacent using RankOrder)
            combo Set where ((size >= 3 and Suite distinct) and Rank same)
            combo Deadwood where (not Sequence and not Set)

            stage Preparation for current 1 times {
              deal 12 from Stock top private to Hand of all
            }

            stage Collect for current until previous out of stage  {
              choose {
                move Discard top private to Hand
                or
                move Stock top private to Hand
              }

              move any from Hand face up to Discard top

              if (sum of Deadwood in Hand using Values <= 10) {
                optional {
                  move all from Set in Hand face up to LayDown top
                  move all from Sequence in Hand face up to LayDown top

                  if (Hand is empty) {
                    move all from Set in Hand of next face up to LayDown top of next
                    move all from Sequence in Hand of next face up to LayDown top of next
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
              move all from Set in Hand face up to LayDown top
              move all from Sequence in Hand face up to LayDown top

              move Hand face up to Trash
            }

            score sum of Trash using Values to LeftOver of all
            winner is lowest LeftOver
        "
        ;
      
      // We pass CGDSLParser::extrema_of_card_set as the mapper
      let result = test_rule_consume(
          input, 
          Rule::file, 
          CGDSLParser::file
      );

      assert!(result.is_ok(), "Error: {:?}", result.err());

      println!("{:?}", result);
    }

}
