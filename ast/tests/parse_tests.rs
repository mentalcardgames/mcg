#[cfg(test)]
mod tests {
    use ast::{ast::*};
    use syn::parse_str;

    const CURRENT: PlayerExpr = PlayerExpr::Current;
    const PREVIOUS: PlayerExpr = PlayerExpr::Previous;
    const COMPETITOR: PlayerExpr = PlayerExpr::Competitor;

    fn id(id: &str) -> ID {
      ID::new(id)
    }

    fn stage(id: &str) -> Stage {
      Stage::new(ID::new(id))
    }

    fn playername(id: &str) -> PlayerName {
      PlayerName::new(ID::new(id))
    }

    fn teamname(id: &str) -> TeamName {
      TeamName::new(ID::new(id))
    }

    fn location(id: &str) -> Location {
      Location::new(ID::new(id))
    }

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

    // PlayerExpr ============================================================
    
    #[test]
    fn parses_valid_player_current() {
        let parsed: PlayerExpr = parse_str(
          "current"
        ).unwrap();
        assert_eq!(parsed, CURRENT);
    }

    #[test]
    fn parses_valid_player_previous() {
        let parsed: PlayerExpr = parse_str(
          "previous"
        ).unwrap();
        assert_eq!(parsed, PREVIOUS);
    }

    #[test]
    fn parses_valid_player_competitor() {
        let parsed: PlayerExpr = parse_str(
          "competitor"
        ).unwrap();
        assert_eq!(parsed, COMPETITOR);
    }

    #[test]
    fn parses_valid_player_owner_highest() {
        let parsed: PlayerExpr = parse_str(
          "owner of highest Mem"
        ).unwrap();
        assert_eq!(parsed,
          PlayerExpr::OwnerOfHighest(
            memory("Mem")
          )
        );
    }

    #[test]
    fn parses_valid_player_owner_lowest() {
        let parsed: PlayerExpr = parse_str(
          "owner of lowest Mem"
        ).unwrap();
        assert_eq!(parsed,
          PlayerExpr::OwnerOfLowest(
            memory("Mem")
          )
        );
    }

    #[test]
    fn parses_valid_player_turnorder() {
        let parsed: PlayerExpr = parse_str(
          "turnorder(3)"
        ).unwrap();
        assert_eq!(parsed, PlayerExpr::Turnorder(IntExpr::Int(3)));
    }
    
    #[test]
    fn parses_valid_player_id() {
        let parsed: PlayerExpr = parse_str(
          "P1"
        ).unwrap();
        assert_eq!(parsed, PlayerExpr::PlayerName(
          playername("P1"))
        );
    }

    // =======================================================================

    // Op ====================================================================
    
    #[test]
    fn parses_valid_op_plus() {
        let parsed: Op = parse_str(
          "+"
        ).unwrap();
        assert_eq!(parsed, Op::Plus);
    }

    #[test]
    fn parses_valid_op_minus() {
        let parsed: Op = parse_str(
          "-"
        ).unwrap();
        assert_eq!(parsed, Op::Minus);
    }

    #[test]
    fn parses_valid_op_div() {
        let parsed: Op = parse_str(
          "/"
        ).unwrap();
        assert_eq!(parsed, Op::Div);
    }

    #[test]
    fn parses_valid_op_mul() {
        let parsed: Op = parse_str(
          "*"
        ).unwrap();
        assert_eq!(parsed, Op::Mul);
    }
    
    #[test]
    fn parses_valid_op_mod() {
        let parsed: Op = parse_str(
          "%"
        ).unwrap();
        assert_eq!(parsed, Op::Mod);
    }
    // =======================================================================

    // IntCmpOp ==============================================================
    
    #[test]
    fn parses_valid_intcmpop_eq() {
        let parsed: IntCmpOp = parse_str(
          "=="
        ).unwrap();
        assert_eq!(parsed, IntCmpOp::Eq);
    }

    #[test]
    fn parses_valid_intcmpop_neq() {
        let parsed: IntCmpOp = parse_str(
          "!="
        ).unwrap();
        assert_eq!(parsed, IntCmpOp::Neq);
    }

    #[test]
    fn parses_valid_intcmpop_le() {
        let parsed: IntCmpOp = parse_str(
          "<="
        ).unwrap();
        assert_eq!(parsed, IntCmpOp::Le);
    }

    #[test]
    fn parses_valid_intcmpop_ge() {
        let parsed: IntCmpOp = parse_str(
          ">="
        ).unwrap();
        assert_eq!(parsed, IntCmpOp::Ge);
    }

    #[test]
    fn parses_valid_intcmpop_lt() {
        let parsed: IntCmpOp = parse_str(
          "<"
        ).unwrap();
        assert_eq!(parsed, IntCmpOp::Lt);
    }

    #[test]
    fn parses_valid_intcmpop_gt() {
        let parsed: IntCmpOp = parse_str(
          ">"
        ).unwrap();
        assert_eq!(parsed, IntCmpOp::Gt);
    }
    
    // =======================================================================

    // Status ================================================================

    #[test]
    fn parses_valid_status_facup() {
        let parsed: Status = parse_str(
          "face up"
        ).unwrap();
        assert_eq!(parsed, Status::FaceUp);
    }

    #[test]
    fn parses_valid_facedown() {
        let parsed: Status = parse_str(
          "face down"
        ).unwrap();
        assert_eq!(parsed, Status::FaceDown);
    }
    
    #[test]
    fn parses_valid_private() {
        let parsed: Status = parse_str(
          "private"
        ).unwrap();
        assert_eq!(parsed, Status::Private);
    }
    
    // =======================================================================

    // Quantifier ============================================================
    
    #[test]
    fn parses_valid_quantifier_all() {
        let parsed: Quantifier = parse_str(
          "all"
        ).unwrap();
        assert_eq!(parsed, Quantifier::All);
    }

    #[test]
    fn parses_valid_quantifier_any() {
        let parsed: Quantifier = parse_str(
          "any"
        ).unwrap();
        assert_eq!(parsed, Quantifier::Any);
    }

    // =======================================================================

    // TeamExpr ==============================================================
    
    #[test]
    fn parses_valid_teamexpr_team_of() {
        let parsed: TeamExpr = parse_str(
          "team of current"
        ).unwrap();
        assert_eq!(parsed, TeamExpr::TeamOf(CURRENT));
    }

    #[test]
    fn parses_valid_teamexpr_team_id() {
        let parsed: TeamExpr = parse_str(
          "T1"
        ).unwrap();
        assert_eq!(parsed, TeamExpr::TeamName(
          teamname("T1"))
        );
    }

    // =======================================================================

    // CardPosition ==========================================================

    #[test]
    fn parses_valid_cardposition_top_of() {
        let parsed: CardPosition = parse_str(
          "top(Hand)"
        ).unwrap();
        assert_eq!(parsed, CardPosition::Top(
          location("Hand"))
        );
    }

    #[test]
    fn parses_valid_cardposition_bottom_of() {
        let parsed: CardPosition = parse_str(
          "bottom(Hand)"
        ).unwrap();
        assert_eq!(parsed, CardPosition::Bottom(location("Hand")));
    }

    #[test]
    fn parses_valid_cardposition_max() {
        let parsed: CardPosition = parse_str(
          "max(Hand) using Aces"
        ).unwrap();
        assert_eq!(parsed, CardPosition::Max(Box::new(CardSet::Group(Group::Location(location("Hand")))), id("Aces")));
    }

    #[test]
    fn parses_valid_cardposition_min() {
        let parsed: CardPosition = parse_str(
          "min(Hand) using Aces"
        ).unwrap();
        assert_eq!(parsed, CardPosition::Min(Box::new(CardSet::Group(Group::Location(location("Hand")))), id("Aces")));
    }

    #[test]
    fn parses_valid_cardposition_at() {
        let parsed: CardPosition = parse_str(
          "Hand[3]"
        ).unwrap();
        assert_eq!(parsed, CardPosition::At(location("Hand"), IntExpr::Int(3)));
    }

    // =======================================================================

    // IntExpr ===============================================================

    #[test]
    fn parses_valid_intexpr_int() {
        let parsed: IntExpr = parse_str(
          "3"
        ).unwrap();
        assert_eq!(parsed, IntExpr::Int(3));
    }

    #[test]
    fn parses_valid_intexpr_op() {
        let parsed: IntExpr = parse_str(
          "(3 + 3)"
        ).unwrap();
        assert_eq!(parsed, IntExpr::IntOp(Box::new(IntExpr::Int(3)), Op::Plus, Box::new(IntExpr::Int(3))));
    }

    #[test]
    fn parses_valid_intexpr_size_of() {
        let parsed: IntExpr = parse_str(
          "size of (3, 3)"
        ).unwrap();
        assert_eq!(parsed, IntExpr::SizeOf(Collection::IntCollection(
          IntCollection {
            ints: vec![IntExpr::Int(3), IntExpr::Int(3)]
          }
        )));
    }

    #[test]
    fn parses_valid_intexpr_sum() {
        let parsed: IntExpr = parse_str(
          "sum(3, 3)"
        ).unwrap();
        assert_eq!(parsed, IntExpr::SumOfIntCollection(
          IntCollection {
            ints: vec![IntExpr::Int(3), IntExpr::Int(3)]
          }
        ));
    }
    
    #[test]
    fn parses_valid_intexpr_sum_of() {
        let parsed: IntExpr = parse_str(
          "sum of Hand using Aces"
        ).unwrap();
        assert_eq!(parsed, IntExpr::SumOfCardSet(
          Box::new(CardSet::Group(Group::Location(location("Hand")))), pointmap("Aces"))
        );
    }

    #[test]
    fn parses_valid_intexpr_min_intcollection() {
        let parsed: IntExpr = parse_str(
          "min(3, 3)"
        ).unwrap();
        assert_eq!(parsed, IntExpr::MinIntCollection(
          IntCollection {
            ints: vec![IntExpr::Int(3), IntExpr::Int(3)]
          }
        ));
    }
    
    #[test]
    fn parses_valid_intexpr_max_intcollection() {
        let parsed: IntExpr = parse_str(
          "max(3, 3)"
        ).unwrap();
        assert_eq!(parsed, IntExpr::MaxIntCollection(
          IntCollection {
            ints: vec![IntExpr::Int(3), IntExpr::Int(3)]
          }
        ));
    }
    
    #[test]
    fn parses_valid_intexpr_min_pointmap() {
        let parsed: IntExpr = parse_str(
          "min of Hand using Aces"
        ).unwrap();
        assert_eq!(parsed, IntExpr::MinOf(
          Box::new(CardSet::Group(Group::Location(location("Hand")))), pointmap("Aces"))
        );
    }
    
    #[test]
    fn parses_valid_intexpr_max_pointmap() {
        let parsed: IntExpr = parse_str(
          "max of Hand using Aces"
        ).unwrap();
        assert_eq!(parsed, IntExpr::MaxOf(
          Box::new(CardSet::Group(Group::Location(location("Hand")))), pointmap("Aces"))
        );
    }
    
    #[test]
    fn parses_valid_intexpr_stageroundcounter() {
        let parsed: IntExpr = parse_str(
          "stageroundcounter"
        ).unwrap();
        assert_eq!(parsed, IntExpr::StageRoundCounter);
    }

    // =======================================================================

    // BoolExpr ==============================================================

    #[test]
    fn parses_valid_boolexpr_eq() {
        let parsed: BoolExpr = parse_str(
          "A == B"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::Eq(id("A"), id("B")));
    }

    #[test]
    fn parses_valid_boolexpr_neq() {
        let parsed: BoolExpr = parse_str(
          "A != B"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::Neq(id("A"), id("B")));
    }

    #[test]
    fn parses_valid_boolexpr_player_eq() {
        let parsed: BoolExpr = parse_str(
          "current == A"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::PlayerEq(CURRENT, PlayerExpr::PlayerName(playername("A"))));
    }

    #[test]
    fn parses_valid_boolexpr_player_neq() {
        let parsed: BoolExpr = parse_str(
          "A != current"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::PlayerNeq(PlayerExpr::PlayerName(playername("A")), CURRENT));
    }
    
    #[test]
    fn parses_valid_boolexpr_team_eq() {
        let parsed: BoolExpr = parse_str(
          "team of A == B"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::TeamEq(TeamExpr::TeamOf(PlayerExpr::PlayerName(playername("A"))), TeamExpr::TeamName(teamname("B"))));
    }

    #[test]
    fn parses_valid_boolexpr_team_neq() {
        let parsed: BoolExpr = parse_str(
          "A != team of B"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::TeamNeq(TeamExpr::TeamName(teamname("A")), TeamExpr::TeamOf(PlayerExpr::PlayerName(playername("B")))));
    }

    #[test]
    fn parses_valid_boolexpr_or() {
        let parsed: BoolExpr = parse_str(
          "(A != B or A != B)"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::Or(
          Box::new(BoolExpr::Neq(id("A"), id("B"))),
          Box::new(BoolExpr::Neq(id("A"), id("B")))
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_and() {
        let parsed: BoolExpr = parse_str(
          "(A != B and A != B)"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::And(
          Box::new(BoolExpr::Neq(id("A"), id("B"))),
          Box::new(BoolExpr::Neq(id("A"), id("B")))
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_intcmp() {
        let parsed: BoolExpr = parse_str(
          "3 == 2"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::IntCmp(
          IntExpr::Int(3),
          IntCmpOp::Eq,
          IntExpr::Int(2)
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_cardset_eq() {
        let parsed: BoolExpr = parse_str(
          "Hand of current == Hand"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::CardSetEq(
          CardSet::GroupOfPlayer(Group::Location(location("Hand")), CURRENT),
          CardSet::Group(Group::Location(location("Hand"))),
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_cardset_neq() {
        let parsed: BoolExpr = parse_str(
          "Hand != Hand of current"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::CardSetNeq(
          CardSet::Group(Group::Location(location("Hand"))),
          CardSet::GroupOfPlayer(Group::Location(location("Hand")), CURRENT),
        ));
    }

    #[test]
    fn parses_valid_boolexpr_cardset_empty() {
        let parsed: BoolExpr = parse_str(
          "Hand is empty"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::CardSetIsEmpty(
          CardSet::Group(Group::Location(location("Hand")))
        ));
    }

    #[test]
    fn parses_valid_boolexpr_cardset_not_empty() {
        let parsed: BoolExpr = parse_str(
          "Hand is not empty"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::CardSetIsNotEmpty(
          CardSet::Group(Group::Location(location("Hand")))
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_not() {
        let parsed: BoolExpr = parse_str(
          "not 3 == 2"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::Not(
          Box::new(BoolExpr::IntCmp(
            IntExpr::Int(3),
            IntCmpOp::Eq,
            IntExpr::Int(2)
        ))));
    }
    
    #[test]
    fn parses_valid_boolexpr_out_of_stage_player() {
        let parsed: BoolExpr = parse_str(
          "current out of stage"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::OutOfStagePlayer(
          CURRENT
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_out_of_game_player() {
        let parsed: BoolExpr = parse_str(
          "current out of game"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::OutOfGamePlayer(
          CURRENT
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_out_of_stage_collection() {
        let parsed: BoolExpr = parse_str(
          "others out of stage"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::OutOfStageCollection(
          PlayerCollection::Others
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_out_of_game_collection() {
        let parsed: BoolExpr = parse_str(
          "others out of game"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::OutOfGameCollection(
          PlayerCollection::Others
        ));
    }
    
    // =======================================================================

    // StringExpr ============================================================
    
    #[test]
    fn parses_valid_stringexpr_id() {
        let parsed: StringExpr = parse_str(
          "Monkey"
        ).unwrap();
        assert_eq!(parsed, StringExpr::ID(
          id("Monkey")
        ));
    }

    #[test]
    fn parses_valid_stringexpr_key_of() {
        let parsed: StringExpr = parse_str(
          "Rank of top(Hand)"
        ).unwrap();
        assert_eq!(parsed, StringExpr::KeyOf(
          key("Rank"),
          CardPosition::Top(location("Hand"))
        ));
    }

    #[test]
    fn parses_valid_stringexpr_collection_at() {
        let parsed: StringExpr = parse_str(
          "(A, B, C)[3]"
        ).unwrap();
        assert_eq!(parsed, StringExpr::StringCollectionAt(
          StringCollection {
            strings: vec![
              StringExpr::ID(id("A")),
              StringExpr::ID(id("B")),
              StringExpr::ID(id("C"))
            ]
          },
          IntExpr::Int(3)
        ));
    }

    // =======================================================================

    // PlayerCollection ======================================================
   
    #[test]
    fn parses_valid_player_collection_others() {
        let parsed: PlayerCollection = parse_str(
          "others"
        ).unwrap();
        assert_eq!(parsed, 
          PlayerCollection::Others
        );
    }

    #[test]
    fn parses_valid_player_collection_playersin() {
        let parsed: PlayerCollection = parse_str(
          "playersin"
        ).unwrap();
        assert_eq!(parsed, 
          PlayerCollection::PlayersIn
        );
    }

    #[test]
    fn parses_valid_player_collection_playersout() {
        let parsed: PlayerCollection = parse_str(
          "playersout"
        ).unwrap();
        assert_eq!(parsed, 
          PlayerCollection::PlayersOut
        );
    }

    #[test]
    fn parses_valid_player_collection_collection() {
        let parsed: PlayerCollection = parse_str(
          "(current, current)"
        ).unwrap();
        assert_eq!(parsed, 
          PlayerCollection::Player(
            vec![
              CURRENT,
              CURRENT,
            ]
          )
        );
    }

    #[test]
    fn parses_valid_player_collection_quantifier() {
        let parsed: PlayerCollection = parse_str(
          "all"
        ).unwrap();
        assert_eq!(parsed, 
          PlayerCollection::Quantifier(
            Quantifier::All
          )
        );
    }

    // =======================================================================

    // FilterExpr ============================================================

    #[test]
    fn parses_valid_filter_expr_same_key() {
        let parsed: FilterExpr = parse_str(
          "same Rank"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Same(key("Rank"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_distinct_key() {
        let parsed: FilterExpr = parse_str(
          "distinct Rank"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Distinct(key("Rank"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_adjacent_key() {
        let parsed: FilterExpr = parse_str(
          "adjacent Rank using Aces"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Adjacent(key("Rank"), precedence("Aces"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_higher_key() {
        let parsed: FilterExpr = parse_str(
          "higher Rank using Aces"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Higher(key("Rank"), precedence("Aces"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_lower_key() {
        let parsed: FilterExpr = parse_str(
          "lower Rank using Aces"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Lower(key("Rank"), precedence("Aces"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_size_eq() {
        let parsed: FilterExpr = parse_str(
          "size == 3"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Size(IntCmpOp::Eq, Box::new(IntExpr::Int(3)))
        );
    }

    #[test]
    fn parses_valid_filter_expr_size_neq() {
        let parsed: FilterExpr = parse_str(
          "size != 3"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Size(IntCmpOp::Neq, Box::new(IntExpr::Int(3)))
        );
    }

    #[test]
    fn parses_valid_filter_expr_size_lt() {
        let parsed: FilterExpr = parse_str(
          "size < 3"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Size(IntCmpOp::Lt, Box::new(IntExpr::Int(3)))
        );
    }

    #[test]
    fn parses_valid_filter_expr_size_gt() {
        let parsed: FilterExpr = parse_str(
          "size > 3"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Size(IntCmpOp::Gt, Box::new(IntExpr::Int(3)))
        );
    }

    #[test]
    fn parses_valid_filter_expr_size_le() {
        let parsed: FilterExpr = parse_str(
          "size <= 3"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Size(IntCmpOp::Le, Box::new(IntExpr::Int(3)))
        );
    }

    #[test]
    fn parses_valid_filter_expr_size_ge() {
        let parsed: FilterExpr = parse_str(
          "size >= 3"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))
        );
    }

    #[test]
    fn parses_valid_filter_expr_rank_eq() {
        let parsed: FilterExpr = parse_str(
          "Rank == Ace"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::KeyEq(
            key("Rank"),
            // TODO: Should be Value not StringExpr
            Box::new(StringExpr::ID(id("Ace"))))
        );
    }

    #[test]
    fn parses_valid_filter_expr_rank_neq() {
        let parsed: FilterExpr = parse_str(
          "Rank != Ace"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::KeyNeq(
            key("Rank"),
            // TODO: Should be Value not StringExpr
            Box::new(StringExpr::ID(id("Ace"))))
        );
    }

    #[test]
    fn parses_valid_filter_expr_not_combo() {
        let parsed: FilterExpr = parse_str(
          "not Pair"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::NotCombo(
            combo("Pair")
          )
        );
    }

    #[test]
    fn parses_valid_filter_expr_combo() {
        let parsed: FilterExpr = parse_str(
          "Pair"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Combo(
            combo("Pair")
          )
        );
    }

    #[test]
    fn parses_valid_filter_expr_and() {
        let parsed: FilterExpr = parse_str(
          "(Pair and Triple)"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::And(
            Box::new(FilterExpr::Combo(
              combo("Pair")
            )),
            Box::new(FilterExpr::Combo(
              combo("Triple")
            ))
          )
        );
    }

    #[test]
    fn parses_valid_filter_expr_or() {
        let parsed: FilterExpr = parse_str(
          "(Pair or Triple)"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Or(
            Box::new(FilterExpr::Combo(
              combo("Pair")
            )),
            Box::new(FilterExpr::Combo(
              combo("Triple")
            ))
          )
        );
    }

    // =======================================================================

    // Group =================================================================
    #[test]
    fn parses_valid_group_location() {
        let parsed: Group = parse_str(
          "Hand"
        ).unwrap();
        assert_eq!(parsed, 
          Group::Location(
            location("Hand")
          )
        );
    }

    #[test]
    fn parses_valid_group_location_filter() {
        let parsed: Group = parse_str(
          "Hand where same Rank"
        ).unwrap();
        assert_eq!(parsed, 
          Group::LocationWhere(
            location("Hand"),
            FilterExpr::Same(key("Rank"))
          )
        );
    }

    #[test]
    fn parses_valid_group_location_collection() {
        let parsed: Group = parse_str(
          "(Hand, Stack)"
        ).unwrap();
        assert_eq!(parsed, 
          Group::LocationCollection(
            LocationCollection {
              locations: vec![
                location("Hand"),
                location("Stack")
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_group_location_collection_filter() {
        let parsed: Group = parse_str(
          "(Hand, Stack) where same Rank"
        ).unwrap();
        assert_eq!(parsed, 
          Group::LocationCollectionWhere(
            LocationCollection {
              locations: vec![
                location("Hand"),
                location("Stack")
              ]
            },
            FilterExpr::Same(key("Rank"))
          )
        );
    }


    #[test]
    fn parses_valid_group_combo_in_location() {
        let parsed: Group = parse_str(
          "Pair in Hand"
        ).unwrap();
        assert_eq!(parsed, 
          Group::ComboInLocation(
            combo("Pair"),
            location("Hand")
          )
        );
    }

    #[test]
    fn parses_valid_group_combo_in_location_collection() {
        let parsed: Group = parse_str(
          "Pair in (Hand, Stack)"
        ).unwrap();
        assert_eq!(parsed, 
          Group::ComboInLocationCollection(
            combo("Pair"),
            LocationCollection {
              locations: vec![
                location("Hand"),
                location("Stack")
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_group_combo_not_in_location() {
        let parsed: Group = parse_str(
          "Pair not in Hand"
        ).unwrap();
        assert_eq!(parsed, 
          Group::NotComboInLocation(
            combo("Pair"),
            location("Hand")
          )
        );
    }

    #[test]
    fn parses_valid_group_combo_not_in_location_collection() {
        let parsed: Group = parse_str(
          "Pair not in (Hand, Stack)"
        ).unwrap();
        assert_eq!(parsed, 
          Group::NotComboInLocationCollection(
            combo("Pair"),
            LocationCollection {
              locations: vec![
                location("Hand"),
                location("Stack")
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_group_cardposition() {
        let parsed: Group = parse_str(
          "top(Hand)"
        ).unwrap();
        assert_eq!(parsed, 
          Group::CardPosition(
            CardPosition::Top(location("Hand"))
          )
        );
    }

    // =======================================================================

    // CardSet ===============================================================

    #[test]
    fn parses_valid_cardset_group() {
        let parsed: CardSet = parse_str(
          "top(Hand)"
        ).unwrap();
        assert_eq!(parsed, 
          CardSet::Group(
            Group::CardPosition(
              CardPosition::Top(location("Hand"))
            )
          )
        );
    }

    #[test]
    fn parses_valid_cardset_group_of_player() {
        let parsed: CardSet = parse_str(
          "Hand where same Rank of current"
        ).unwrap();
        assert_eq!(parsed, 
          CardSet::GroupOfPlayer(
            Group::LocationWhere(
              location("Hand"),
              FilterExpr::Same(key("Rank"))
            ),
            CURRENT
          )
        );
    }

    #[test]
    fn parses_valid_cardset_group_of_player_collection() {
        let parsed: CardSet = parse_str(
          "Hand where same Rank of others"
        ).unwrap();
        assert_eq!(parsed, 
          CardSet::GroupOfPlayerCollection(
            Group::LocationWhere(
              location("Hand"),
              FilterExpr::Same(key("Rank"))
            ),
            PlayerCollection::Others
          )
        );
    }

    // =======================================================================

    // IntCollection =========================================================

    #[test]
    fn parses_valid_intcollection() {
        let parsed: IntCollection = parse_str(
          "(1, 2, 3, 4, 5)"
        ).unwrap();
        assert_eq!(parsed, 
          IntCollection {
            ints: vec![
              IntExpr::Int(1),
              IntExpr::Int(2),
              IntExpr::Int(3),
              IntExpr::Int(4),
              IntExpr::Int(5),
            ]
          }
        );
    }

    // =======================================================================

    // LocationCollection ====================================================

    #[test]
    fn parses_valid_locationcollection() {
        let parsed: LocationCollection = parse_str(
          "(Hand, Deck, Hand)"
        ).unwrap();
        assert_eq!(parsed, 
          LocationCollection {
            locations: vec![
              location("Hand"),
              location("Deck"),
              location("Hand"),
            ]
          }
        );
    }

    // =======================================================================

    // TeamCollection ========================================================

    #[test]
    fn parses_valid_teamcollection_other_teams() {
        let parsed: TeamCollection = parse_str(
          "other teams"
        ).unwrap();
        assert_eq!(parsed, 
          TeamCollection::OtherTeams
        );
    }

    #[test]
    fn parses_valid_teamcollection_teams() {
        let parsed: TeamCollection = parse_str(
          "(T1, T2)"
        ).unwrap();
        assert_eq!(parsed,
          TeamCollection::Team(
            vec![
              TeamExpr::TeamName(teamname("T1")),
              TeamExpr::TeamName(teamname("T2")),
            ]
          )
        );
    }

    // =======================================================================

    // StringCollection ======================================================

    #[test]
    fn parses_valid_stringcollection() {
        let parsed: StringCollection = parse_str(
          "(A, B)"
        ).unwrap();
        assert_eq!(parsed,
          StringCollection {
            strings: vec![
              StringExpr::ID(id("A")),
              StringExpr::ID(id("B")),
            ]
          }
        );
    }

    // =======================================================================

    // Collection ============================================================

    #[test]
    fn parses_valid_collection_playercollection() {
        let parsed: Collection = parse_str(
          "(current, previous)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::PlayerCollection(
            PlayerCollection::Player(
              vec![
                CURRENT,
                PREVIOUS,
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_collection_teamcollection() {
        let parsed: Collection = parse_str(
          "(T1, team of current)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::TeamCollection(
            TeamCollection::Team(
              vec![
                TeamExpr::TeamName(teamname("T1")),
                TeamExpr::TeamOf(PlayerExpr::Current),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_collection_intcollection() {
        let parsed: Collection = parse_str(
          "(1, 2, 3)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::IntCollection(
            IntCollection {
              ints: vec![
                IntExpr::Int(1),
                IntExpr::Int(2),
                IntExpr::Int(3),
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_collection_cardset() {
        let parsed: Collection = parse_str(
          "Hand of current"
        ).unwrap();
        assert_eq!(parsed,
          Collection::CardSet(
            Box::new(CardSet::GroupOfPlayer(Group::Location(location("Hand")), CURRENT))
          )
        );
    }

    #[test]
    fn parses_valid_collection_cardset_ambiguous() {
        let parsed: Collection = parse_str(
          "(Hand, Deck, Hand)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::Ambiguous(
            vec![
              ID::new("Hand"),
              ID::new("Deck"),
              ID::new("Hand"),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_collection_stringcollection() {
        let parsed: Collection = parse_str(
          "(A, Rank of top(Hand), C)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::StringCollection(
            StringCollection {
            strings: vec![
              StringExpr::ID(id("A")),
              StringExpr::KeyOf(key("Rank"), CardPosition::Top(location("Hand"))),
              StringExpr::ID(id("C")),
            ]
          }
          )
        );
    }

    // =======================================================================

    // Repititions ===========================================================

    #[test]
    fn parses_valid_repititions() {
        let parsed: Repititions = parse_str(
          "3 times"
        ).unwrap();
        assert_eq!(parsed,
          Repititions {
            times: IntExpr::Int(3)
          }
        );
    }

    // =======================================================================

    // EndCondition ==========================================================

    #[test]
    fn parses_valid_endcondition_until_end() {
        let parsed: EndCondition = parse_str(
          "until(end)"
        ).unwrap();
        assert_eq!(parsed,
          EndCondition::UntilEnd
        );
    }

    #[test]
    fn parses_valid_endcondition_until_reps() {
        let parsed: EndCondition = parse_str(
          "until(3 times)"
        ).unwrap();
        assert_eq!(parsed,
          EndCondition::UntilRep(
            Repititions {
              times: IntExpr::Int(3)
            }
          )
        );
    }

    #[test]
    fn parses_valid_endcondition_until_bool() {
        let parsed: EndCondition = parse_str(
          "until(3 == 2)"
        ).unwrap();
        assert_eq!(parsed,
          EndCondition::UntilBool(
            BoolExpr::IntCmp(IntExpr::Int(3), IntCmpOp::Eq, IntExpr::Int(2))
          )
        );
    }

    #[test]
    fn parses_valid_endcondition_until_bool_and_rep() {
        let parsed: EndCondition = parse_str(
          "until(3 == 2 and 3 times)"
        ).unwrap();
        assert_eq!(parsed,
          EndCondition::UntilBoolAndRep(
            BoolExpr::IntCmp(IntExpr::Int(3), IntCmpOp::Eq, IntExpr::Int(2)),
            Repititions {
              times: IntExpr::Int(3)
            }
          )
        );
    }
    
    #[test]
    fn parses_valid_endcondition_until_bool_or_rep() {
        let parsed: EndCondition = parse_str(
          "until(3 == 2 or 3 times)"
        ).unwrap();
        assert_eq!(parsed,
          EndCondition::UntilBoolOrRep(
            BoolExpr::IntCmp(IntExpr::Int(3), IntCmpOp::Eq, IntExpr::Int(2)),
            Repititions {
              times: IntExpr::Int(3)
            }
          )
        );
    }
    
    // =======================================================================

    // IntRange ==============================================================

    #[test]
    fn parses_valid_endcondition_intrange_eq() {
        let parsed: IntRange = parse_str(
          "== 2"
        ).unwrap();
        assert_eq!(parsed,
          IntRange {
            op: IntCmpOp::Eq,
            int: IntExpr::Int(2)
          }
        );
    }

    #[test]
    fn parses_valid_endcondition_intrange_neq() {
        let parsed: IntRange = parse_str(
          "!= 2"
        ).unwrap();
        assert_eq!(parsed,
          IntRange {
            op: IntCmpOp::Neq,
            int: IntExpr::Int(2)
          }
        );
    }
    
    #[test]
    fn parses_valid_endcondition_intrange_ge() {
        let parsed: IntRange = parse_str(
          ">= 2"
        ).unwrap();
        assert_eq!(parsed,
          IntRange {
            op: IntCmpOp::Ge,
            int: IntExpr::Int(2)
          }
        );
    }
    
    #[test]
    fn parses_valid_endcondition_intrange_le() {
        let parsed: IntRange = parse_str(
          "<= 2"
        ).unwrap();
        assert_eq!(parsed,
          IntRange {
            op: IntCmpOp::Le,
            int: IntExpr::Int(2)
          }
        );
    }
    
    #[test]
    fn parses_valid_endcondition_intrange_gt() {
        let parsed: IntRange = parse_str(
          "> 2"
        ).unwrap();
        assert_eq!(parsed,
          IntRange {
            op: IntCmpOp::Gt,
            int: IntExpr::Int(2)
          }
        );
    }
    
    #[test]
    fn parses_valid_endcondition_intrange_lt() {
        let parsed: IntRange = parse_str(
          "< 2"
        ).unwrap();
        assert_eq!(parsed,
          IntRange {
            op: IntCmpOp::Lt,
            int: IntExpr::Int(2)
          }
        );
    }

    // =======================================================================

    // Quantity ==============================================================

    #[test]
    fn parses_valid_quantity_int() {
        let parsed: Quantity = parse_str(
          "3"
        ).unwrap();
        assert_eq!(parsed,
          Quantity::Int(
            IntExpr::Int(3)
          )
        );
    }

    #[test]
    fn parses_valid_quantity_intrange() {
        let parsed: Quantity = parse_str(
          "== 3"
        ).unwrap();
        assert_eq!(parsed,
          Quantity::IntRange(
            IntRange {
              op: IntCmpOp::Eq,
              int: IntExpr::Int(3)
            }
          )
        );
    }

    #[test]
    fn parses_valid_quantity_quantifier() {
        let parsed: Quantity = parse_str(
          "all"
        ).unwrap();
        assert_eq!(parsed,
          Quantity::Quantifier(
            Quantifier::All
          )
        );
    }
    
    // =======================================================================

    // ClassicMove ===========================================================

    #[test]
    fn parses_valid_classicmove_move() {
        let parsed: ClassicMove = parse_str(
          "move Hand private to Deck"
        ).unwrap();
        assert_eq!(parsed,
          ClassicMove::Move(
            CardSet::Group(Group::Location(location("Hand"))),
            Status::Private,
            CardSet::Group(Group::Location(location("Deck")))
          )
        );
    }

    #[test]
    fn parses_valid_classicmove_move_quantity() {
        let parsed: ClassicMove = parse_str(
          "move all from Hand private to Deck"
        ).unwrap();
        assert_eq!(parsed,
          ClassicMove::MoveQuantity(
            Quantity::Quantifier(Quantifier::All),
            CardSet::Group(Group::Location(location("Hand"))),
            Status::Private,
            CardSet::Group(Group::Location(location("Deck")))
          )
        );
    }

    // =======================================================================

    // DealMove ===========================================================

    #[test]
    fn parses_valid_dealmove_deal() {
        let parsed: DealMove = parse_str(
          "deal Hand private to Deck"
        ).unwrap();
        assert_eq!(parsed,
          DealMove::Deal(
            CardSet::Group(Group::Location(location("Hand"))),
            Status::Private,
            CardSet::Group(Group::Location(location("Deck")))
          )
        );
    }

    #[test]
    fn parses_valid_dealmove_deal_quantity() {
        let parsed: DealMove = parse_str(
          "deal 12 from Hand private to Deck of all"
        ).unwrap();
        assert_eq!(parsed,
          DealMove::DealQuantity(
            Quantity::Int(IntExpr::Int(12)),
            CardSet::Group(Group::Location(location("Hand"))),
            Status::Private,
            CardSet::GroupOfPlayerCollection(Group::Location(location("Deck")), PlayerCollection::Quantifier(Quantifier::All))
          )
        );
    }

    // =======================================================================

        // DealMove ===========================================================

    #[test]
    fn parses_valid_exchangemove_exchange() {
        let parsed: ExchangeMove = parse_str(
          "exchange Hand private with Deck"
        ).unwrap();
        assert_eq!(parsed,
          ExchangeMove::Exchange(
            CardSet::Group(Group::Location(location("Hand"))),
            Status::Private,
            CardSet::Group(Group::Location(location("Deck")))
          )
        );
    }

    #[test]
    fn parses_valid_exchangemove_exchange_quantity() {
        let parsed: ExchangeMove = parse_str(
          "exchange all from Hand private with Deck"
        ).unwrap();
        assert_eq!(parsed,
          ExchangeMove::ExchangeQuantity(
            Quantity::Quantifier(Quantifier::All),
            CardSet::Group(Group::Location(location("Hand"))),
            Status::Private,
            CardSet::Group(Group::Location(location("Deck")))
          )
        );
    }

    // =======================================================================

    // TokenLocExpr ==========================================================

    #[test]
    fn parses_valid_tokenloc_expr_location() {
        let parsed: TokenLocExpr = parse_str(
          "Hand"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::Location(
            location("Hand")
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_location_player() {
        let parsed: TokenLocExpr = parse_str(
          "Hand of current"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationPlayer(
            location("Hand"),
            CURRENT
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_location_playercollection() {
        let parsed: TokenLocExpr = parse_str(
          "Hand of others"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationPlayerCollection(
            location("Hand"),
            PlayerCollection::Others
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_locationcollection() {
        let parsed: TokenLocExpr = parse_str(
          "(Hand, Deck)"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationCollection(
            LocationCollection {
              locations: vec![
                location("Hand"),
                location("Deck"),
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_locationcollection_player() {
        let parsed: TokenLocExpr = parse_str(
          "(Hand, Deck) of current"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationCollectionPlayer(
            LocationCollection {
              locations: vec![
                location("Hand"),
                location("Deck"),
              ]
            },
            CURRENT
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_locationcollection_playercollection() {
        let parsed: TokenLocExpr = parse_str(
          "(Hand, Deck) of others"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationCollectionPlayerCollection(
            LocationCollection {
              locations: vec![
                location("Hand"),
                location("Deck"),
              ]
            },
            PlayerCollection::Others
          )
        );
    }

    // =======================================================================

    // TokenMove =============================================================

    #[test]
    fn parses_valid_tokenmove_place() {
        let parsed: TokenMove = parse_str(
          "place Hand to Deck"
        ).unwrap();
        assert_eq!(parsed,
          TokenMove::Place(
            TokenLocExpr::Location(
              location("Hand")
            ),
            TokenLocExpr::Location(
              location("Deck")
            ),
          )
        );
    }

    #[test]
    fn parses_valid_tokenmove_place_quantity() {
        let parsed: TokenMove = parse_str(
          "place all from Hand to Deck"
        ).unwrap();
        assert_eq!(parsed,
          TokenMove::PlaceQuantity(
            Quantity::Quantifier(Quantifier::All),
            TokenLocExpr::Location(
              location("Hand")
            ),
            TokenLocExpr::Location(
              location("Deck")
            ),
          )
        );
    }

    // =======================================================================
    
    // Rule ==================================================================

    #[test]
    fn parses_valid_rule_createplayers() {
        let parsed: Rule = parse_str(
          "players: (P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreatePlayer(
            vec![
              playername("P1"),
              playername("P2"),
              playername("P3"),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_createteam() {
        let parsed: Rule = parse_str(
          "team T1: (P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateTeam(
            teamname("T1"),
            vec![
              playername("P1"),
              playername("P2"),
              playername("P3"),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_createturnorder() {
        let parsed: Rule = parse_str(
          "turnorder: (P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateTurnorder(
            vec![
              playername("P1"),
              playername("P2"),
              playername("P3"),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_createturnorder_random() {
        let parsed: Rule = parse_str(
          "random turnorder: (P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateTurnorderRandom(
            vec![
              playername("P1"),
              playername("P2"),
              playername("P3"),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_createlocation_playercollection() {
        let parsed: Rule = parse_str(
          "location Hand on players(P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateLocationOnPlayerCollection(
            location("Hand"),
            PlayerCollection::Player(
              vec![
                PlayerExpr::PlayerName(playername("P1"),),
                PlayerExpr::PlayerName(playername("P2"),),
                PlayerExpr::PlayerName(playername("P3"),),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_createlocation_teamcollection() {
        let parsed: Rule = parse_str(
          "location Hand on teams(T1, T2, T3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateLocationOnTeamCollection(
            location("Hand"),
            TeamCollection::Team(
              vec![
                TeamExpr::TeamName(teamname("T1")),
                TeamExpr::TeamName(teamname("T2")),
                TeamExpr::TeamName(teamname("T3")),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_createlocation_table() {
        let parsed: Rule = parse_str(
          "location Stack on table"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateLocationOnTable(
            location("Stack")
          )
        );
    }

    #[test]
    fn parses_valid_rule_createcard() {
        let parsed: Rule = parse_str(
          "card on Stack: 
            Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
              for Suite(Spades, Clubs)
                for Color(Black)
          "
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateCardOnLocation(
            location("Stack"),
            Types {
              types: vec![
                ( key("Rank"), vec![
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
                  value("Ace"),
                ]),
                (key("Suite"), vec![
                  value("Spades"),
                  value("Clubs"),
                ]),
                (key("Color"), vec![
                  value("Black"),
                ]),
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_rule_createtoken() {
        let parsed: Rule = parse_str(
          "token 10 Chip on Stack"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateTokenOnLocation(
            IntExpr::Int(10),
            token("Chip"),
            location("Stack")
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_precedence() {
        let parsed: Rule = parse_str(
          "precedence Rank on Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreatePrecedence(
            precedence("Rank"),
            vec![ 
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
              (key("Rank"), value("Ace"))
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_precedence_pair() {
        let parsed: Rule = parse_str(
          "precedence Rank (Rank(Two), Suite(Spades), Color(Red))"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreatePrecedence(
            precedence("Rank"),
            vec![
              (key("Rank"), value("Two")),
              (key("Suite"), value("Spades")),
              (key("Color"), value("Red")),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_combo() {
        let parsed: Rule = parse_str(
          "combo SameSuite where same Suite"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateCombo(
            combo("SameSuite"),
            FilterExpr::Same(key("Suite"))
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_playercollection() {
        let parsed: Rule = parse_str(
          "memory Square on (current, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryPlayerCollection(
            memory("Square"),
            PlayerCollection::Player(
              vec![
                CURRENT,
                PlayerExpr::PlayerName(playername("P2")),
                PlayerExpr::PlayerName(playername("P3")),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_table() {
        let parsed: Rule = parse_str(
          "memory Square on table"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryTable(
            memory("Square")
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_int_playercollection() {
        let parsed: Rule = parse_str(
          "memory Square 10 on (current, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryIntPlayerCollection(
            memory("Square"),
            IntExpr::Int(10),
            PlayerCollection::Player(
              vec![
                CURRENT,
                PlayerExpr::PlayerName(playername("P2")),
                PlayerExpr::PlayerName(playername("P3")),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_int_table() {
        let parsed: Rule = parse_str(
          "memory Square 10 on table"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryIntTable(
            memory("Square"),
            IntExpr::Int(10),
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_string_playercollection() {
        let parsed: Rule = parse_str(
          "memory Square Monkey on (current, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryStringPlayerCollection(
            memory("Square"),
            StringExpr::ID(id("Monkey")),
            PlayerCollection::Player(
              vec![
                CURRENT,
                PlayerExpr::PlayerName(playername("P2")),
                PlayerExpr::PlayerName(playername("P3")),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_string_table() {
        let parsed: Rule = parse_str(
          "memory Square Monkey on table"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryStringTable(
            memory("Square"),
            StringExpr::ID(id("Monkey")),
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_pointmap() {
        let parsed: Rule = parse_str(
          "pointmap Rank on Rank(
            Two: 1,
            Three: 1,
            Four: 1,
            Five: 1,
            Six: 1,
            Seven: 1,
            Eight: 1,
            Nine: 1,
            Ten: 1,
            Jack: 1,
            Queen: 1,
            King: 1,
            Ace: 1
          )"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreatePointMap(
            pointmap("Rank"),
            vec![
              (key("Rank"), value("Two"), IntExpr::Int(1)),
              (key("Rank"), value("Three"), IntExpr::Int(1)),
              (key("Rank"), value("Four"), IntExpr::Int(1)),
              (key("Rank"), value("Five"), IntExpr::Int(1)),
              (key("Rank"), value("Six"), IntExpr::Int(1)),
              (key("Rank"), value("Seven"), IntExpr::Int(1)),
              (key("Rank"), value("Eight"), IntExpr::Int(1)),
              (key("Rank"), value("Nine"), IntExpr::Int(1)),
              (key("Rank"), value("Ten"), IntExpr::Int(1)),
              (key("Rank"), value("Jack"), IntExpr::Int(1)),
              (key("Rank"), value("Queen"), IntExpr::Int(1)),
              (key("Rank"), value("King"), IntExpr::Int(1)),
              (key("Rank"), value("Ace"), IntExpr::Int(1)),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_pointmap_pairs() {
        let parsed: Rule = parse_str(
          "pointmap Rank (Rank(Two: 1), Suite(Spades: 1), Color(Red: 1))"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreatePointMap(
            pointmap("Rank"),
            vec![
              (key("Rank"), value("Two"), IntExpr::Int(1)),
              (key("Suite"), value("Spades"), IntExpr::Int(1)),
              (key("Color"), value("Red"), IntExpr::Int(1)),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_flip_action() {
        let parsed: Rule = parse_str(
          "flip Hand to private"
        ).unwrap();
        assert_eq!(parsed,
          Rule::FlipAction(
            CardSet::Group(Group::Location(location("Hand"))),
            Status::Private
          )
        );
    }
    
    #[test]
    fn parses_valid_rule_shuffle_action() {
        let parsed: Rule = parse_str(
          "shuffle Hand"
        ).unwrap();
        assert_eq!(parsed,
          Rule::ShuffleAction(
            CardSet::Group(Group::Location(location("Hand"))),
          )
        );
    }
    
    #[test]
    fn parses_valid_rule_player_out_stage() {
        let parsed: Rule = parse_str(
          "set current out of stage"
        ).unwrap();
        assert_eq!(parsed,
          Rule::PlayerOutOfStageAction(
            CURRENT
          )
        );
    }

    #[test]
    fn parses_valid_rule_player_out_game_succ() {
        let parsed: Rule = parse_str(
          "set current out of game successful"
        ).unwrap();
        assert_eq!(parsed,
          Rule::PlayerOutOfGameSuccAction(
            CURRENT
          )
        );
    }

    #[test]
    fn parses_valid_rule_player_out_game_fail() {
        let parsed: Rule = parse_str(
          "set current out of game fail"
        ).unwrap();
        assert_eq!(parsed,
          Rule::PlayerOutOfGameFailAction(
            CURRENT
          )
        );
    }

    #[test]
    fn parses_valid_rule_player_collection_out_stage() {
        let parsed: Rule = parse_str(
          "set (current) out of stage"
        ).unwrap();
        assert_eq!(parsed,
          Rule::PlayerCollectionOutOfStageAction(
            PlayerCollection::Player(
              vec![
                CURRENT
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_player_collection_out_game_succ() {
        let parsed: Rule = parse_str(
          "set (current) out of game successful"
        ).unwrap();
        assert_eq!(parsed,
          Rule::PlayerCollectionOutOfGameSuccAction(
            PlayerCollection::Player(
              vec![
                CURRENT
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_player_collection_out_game_fail() {
        let parsed: Rule = parse_str(
          "set (current) out of game fail"
        ).unwrap();
        assert_eq!(parsed,
          Rule::PlayerCollectionOutOfGameFailAction(
            PlayerCollection::Player(
              vec![
                CURRENT
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_set_memory_int() {
        let parsed: Rule = parse_str(
          "Square is 10"
        ).unwrap();
        assert_eq!(parsed,
          Rule::SetMemoryInt(
            memory("Square"),
            IntExpr::Int(10)
          )
        );
    }

    #[test]
    fn parses_valid_rule_set_memory_string() {
        let parsed: Rule = parse_str(
          "Square is Rank of top(Hand)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::SetMemoryString(
            memory("Square"),
            StringExpr::KeyOf(
              key("Rank"),
              CardPosition::Top(location("Hand"))
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_set_memory_ambiguous() {
        let parsed: Rule = parse_str(
          "Square is A"
        ).unwrap();
        assert_eq!(parsed,
          Rule::SetMemoryAmbiguous(
            memory("Square"),
            id("A")
          )
        );
    }

    #[test]
    fn parses_valid_rule_set_memory_collection() {
        let parsed: Rule = parse_str(
          "Square is (current)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::SetMemoryCollection(
            memory("Square"),
            Collection::PlayerCollection(
              PlayerCollection::Player(
                vec![
                  CURRENT
                ]
              )
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_cycle_action() {
        let parsed: Rule = parse_str(
          "cycle to next"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CycleAction(
            PlayerExpr::Next
          )
        );
    }

    #[test]
    fn parses_valid_rule_bid_action() {
        let parsed: Rule = parse_str(
          "bid all"
        ).unwrap();
        assert_eq!(parsed,
          Rule::BidAction(
            Quantity::Quantifier(Quantifier::All)
          )
        );
    }

    #[test]
    fn parses_valid_rule_bid_action_memory() {
        let parsed: Rule = parse_str(
          "bid all on Square"
        ).unwrap();
        assert_eq!(parsed,
          Rule::BidActionMemory(
            memory("Square"),
            Quantity::Quantifier(Quantifier::All)
          )
        );
    }
    
    #[test]
    fn parses_valid_rule_end_turn() {
        let parsed: Rule = parse_str(
          "end turn"
        ).unwrap();
        assert_eq!(parsed,
          Rule::EndTurn
        );
    }
    
    #[test]
    fn parses_valid_rule_end_stage() {
        let parsed: Rule = parse_str(
          "end stage"
        ).unwrap();
        assert_eq!(parsed,
          Rule::EndStage
        );
    }

    #[test]
    fn parses_valid_rule_end_game_with_winner() {
        let parsed: Rule = parse_str(
          "end game with winner current"
        ).unwrap();
        assert_eq!(parsed,
          Rule::EndGameWithWinner(
            CURRENT
          )
        );
    }
    
    #[test]
    fn parses_valid_rule_demand_card_position() {
        let parsed: Rule = parse_str(
          "demand top(Hand)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::DemandCardPositionAction(
            CardPosition::Top(
              location("Hand")
            )
          )
        );
    }
    
    #[test]
    fn parses_valid_rule_demand_string() {
        let parsed: Rule = parse_str(
          "demand A"
        ).unwrap();
        assert_eq!(parsed,
          Rule::DemandStringAction(
            StringExpr::ID(
              id("A")
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_demand_int() {
        let parsed: Rule = parse_str(
          "demand 10"
        ).unwrap();
        assert_eq!(parsed,
          Rule::DemandIntAction(
            IntExpr::Int(10)
          )
        );
    }

    // =======================================================================

    // SeqStage ==============================================================

    #[test]
    fn parses_valid_seq_stage() {
        let parsed: SeqStage = parse_str(
          "
            stage Play for current until(1 times) {
              deal 12 from Stock private to Hand of all;
            }
          "
        ).unwrap();
        assert_eq!(parsed,
          SeqStage {
            stage: stage("Play"), 
            player: CURRENT, 
            end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
            flows: vec![
              FlowComponent::Rule(
                Rule::DealMove(
                  DealMove::DealQuantity(
                    Quantity::Int(IntExpr::Int(12)), 
                    CardSet::Group(Group::Location(location("Stock"))), 
                    Status::Private, 
                    CardSet::GroupOfPlayerCollection(Group::Location(location("Hand")), PlayerCollection::Quantifier(Quantifier::All))
                  )
                )
              )
            ] 
          }
        );
    }

    // =======================================================================

    // IfRule ================================================================

    #[test]
    fn parses_valid_if_rule() {
        let parsed: IfRule = parse_str(
          "
            if (current out of stage) {
              cycle to next;
            }
          "
        ).unwrap();
        assert_eq!(parsed,
          IfRule {
            condition: BoolExpr::OutOfStagePlayer(CURRENT),
            flows: vec![
              FlowComponent::Rule(
                Rule::CycleAction(
                  PlayerExpr::Next
                )
              )
            ]
          }
        );
    }

    // =======================================================================

    // OptionalRule ==========================================================

    #[test]
    fn parses_valid_optional_rule() {
        let parsed: OptionalRule = parse_str(
          "
            optional {
              end turn;
            }
          "
        ).unwrap();
        assert_eq!(parsed,
          OptionalRule {
            flows: vec![
              FlowComponent::Rule(
                Rule::EndTurn
              )
            ]
          }
        );
    }

    // =======================================================================

    // ChoiceRule ============================================================

    #[test]
    fn parses_valid_choice_rule() {
        let parsed: ChoiceRule = parse_str(
          "
            choose {
              end turn;
              or
              optional {
                end stage;
              } 
            }
          "
        ).unwrap();
        assert_eq!(parsed,
          ChoiceRule {
            options: vec![
              FlowComponent::Rule(
                Rule::EndTurn
              ),
              FlowComponent::OptionalRule(
                OptionalRule {
                  flows: vec![
                      FlowComponent::Rule(
                        Rule::EndStage
                      )
                  ]
                }
              ),
            ]
          }
        );
    }

    // =======================================================================

    // FlowComponent =========================================================

    #[test]
    fn parses_valid_flow_component_choice_rule() {
        let parsed: FlowComponent = parse_str(
          "
            choose {
              end turn;
              or
              optional {
                end stage;
              } 
            }
          "
        ).unwrap();
        assert_eq!(parsed,
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
                          Rule::EndStage
                        )
                    ]
                  }
                ),
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_flow_component_rule() {
        let parsed: FlowComponent = parse_str(
          "
            end turn;
          "
        ).unwrap();
        assert_eq!(parsed,
          FlowComponent::Rule(
            Rule::EndTurn
          )
        );
    }

    // =======================================================================

    // Game ==================================================================

    #[test]
    fn parses_valid_game() {
        let parsed: Game = parse_str(
          "
            players: (P1, P2, P3);
            turnorder: (P1, P2, P3);
            location (Hand, LayDown, Trash) on players all;
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

            score sum of Trash using Values to LeftOver of all;
            winner is lowest LeftOver;
          "
        ).unwrap();
        assert_eq!(parsed,
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
    }

    // =======================================================================

}