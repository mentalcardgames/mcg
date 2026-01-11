#[cfg(test)]
mod tests {
    use ast::ast::*;
    use syn::parse_str;
    use quote::format_ident;

    // PlayerExpr ============================================================
    
    #[test]
    fn parses_valid_player_current() {
        let parsed: PlayerExpr = parse_str(
          "current"
        ).unwrap();
        assert_eq!(parsed, PlayerExpr::Current);
    }

    #[test]
    fn parses_valid_player_previous() {
        let parsed: PlayerExpr = parse_str(
          "previous"
        ).unwrap();
        assert_eq!(parsed, PlayerExpr::Previous);
    }

    #[test]
    fn parses_valid_player_competitor() {
        let parsed: PlayerExpr = parse_str(
          "competitor"
        ).unwrap();
        assert_eq!(parsed, PlayerExpr::Competitor);
    }

    #[test]
    fn parses_valid_player_owner_highest() {
        let parsed: PlayerExpr = parse_str(
          "owner of highest P1"
        ).unwrap();
        assert_eq!(parsed, PlayerExpr::OwnerOfHighest(format_ident!("P1")));
    }

    #[test]
    fn parses_valid_player_owner_lowest() {
        let parsed: PlayerExpr = parse_str(
          "owner of lowest P1"
        ).unwrap();
        assert_eq!(parsed, PlayerExpr::OwnerOfLowest(format_ident!("P1")));
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
        assert_eq!(parsed, PlayerExpr::PlayerName(format_ident!("P1")));
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
        assert_eq!(parsed, TeamExpr::TeamOf(PlayerExpr::Current));
    }

    #[test]
    fn parses_valid_teamexpr_team_id() {
        let parsed: TeamExpr = parse_str(
          "T1"
        ).unwrap();
        assert_eq!(parsed, TeamExpr::TeamName(format_ident!("T1")));
    }

    // =======================================================================

    // CardPosition ==========================================================

    #[test]
    fn parses_valid_cardposition_top_of() {
        let parsed: CardPosition = parse_str(
          "top(hand)"
        ).unwrap();
        assert_eq!(parsed, CardPosition::Top(format_ident!("hand")));
    }

    #[test]
    fn parses_valid_cardposition_bottom_of() {
        let parsed: CardPosition = parse_str(
          "bottom(hand)"
        ).unwrap();
        assert_eq!(parsed, CardPosition::Bottom(format_ident!("hand")));
    }

    #[test]
    fn parses_valid_cardposition_max_of_using_prec() {
        let parsed: CardPosition = parse_str(
          "max(hand) using prec(aces)"
        ).unwrap();
        assert_eq!(parsed, CardPosition::MaxPrec(Box::new(CardSet::Group(Group::Location(format_ident!("hand")))), format_ident!("aces")));
    }

    #[test]
    fn parses_valid_cardposition_min_of_using_prec() {
        let parsed: CardPosition = parse_str(
          "min(hand) using prec(aces)"
        ).unwrap();
        assert_eq!(parsed, CardPosition::MinPrec(Box::new(CardSet::Group(Group::Location(format_ident!("hand")))), format_ident!("aces")));
    }

    #[test]
    fn parses_valid_cardposition_max_of_using_point() {
        let parsed: CardPosition = parse_str(
          "max(hand) using point(aces)"
        ).unwrap();
        assert_eq!(parsed, CardPosition::MaxPoint(Box::new(CardSet::Group(Group::Location(format_ident!("hand")))), format_ident!("aces")));
    }

    #[test]
    fn parses_valid_cardposition_min_of_using_point() {
        let parsed: CardPosition = parse_str(
          "min(hand) using point(aces)"
        ).unwrap();
        assert_eq!(parsed, CardPosition::MinPoint(Box::new(CardSet::Group(Group::Location(format_ident!("hand")))), format_ident!("aces")));
    }

    #[test]
    fn parses_valid_cardposition_at() {
        let parsed: CardPosition = parse_str(
          "hand[3]"
        ).unwrap();
        assert_eq!(parsed, CardPosition::At(format_ident!("hand"), IntExpr::Int(3)));
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
          "size of ints(3, 3)"
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
          "sum of hand using aces"
        ).unwrap();
        assert_eq!(parsed, IntExpr::SumOfCardSet(
          Box::new(CardSet::Group(Group::Location(format_ident!("hand")))), format_ident!("aces"))
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
          "min of hand using aces"
        ).unwrap();
        assert_eq!(parsed, IntExpr::MinOf(
          Box::new(CardSet::Group(Group::Location(format_ident!("hand")))), format_ident!("aces"))
        );
    }
    
    #[test]
    fn parses_valid_intexpr_max_pointmap() {
        let parsed: IntExpr = parse_str(
          "max of hand using aces"
        ).unwrap();
        assert_eq!(parsed, IntExpr::MaxOf(
          Box::new(CardSet::Group(Group::Location(format_ident!("hand")))), format_ident!("aces"))
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
    fn parses_valid_boolexpr_stringeq() {
        let parsed: BoolExpr = parse_str(
          "A == B"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::StringEq(StringExpr::ID(format_ident!("A")), StringExpr::ID(format_ident!("B"))));
    }

    #[test]
    fn parses_valid_boolexpr_stringneq() {
        let parsed: BoolExpr = parse_str(
          "A != B"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::StringNeq(StringExpr::ID(format_ident!("A")), StringExpr::ID(format_ident!("B"))));
    }

    #[test]
    fn parses_valid_boolexpr_playereq() {
        let parsed: BoolExpr = parse_str(
          "player(A == B)"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::PlayerEq(PlayerExpr::PlayerName(format_ident!("A")), PlayerExpr::PlayerName(format_ident!("B"))));
    }

    #[test]
    fn parses_valid_boolexpr_playerneq() {
        let parsed: BoolExpr = parse_str(
          "player(A != B)"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::PlayerNeq(PlayerExpr::PlayerName(format_ident!("A")), PlayerExpr::PlayerName(format_ident!("B"))));
    }
    
    #[test]
    fn parses_valid_boolexpr_team_eq() {
        let parsed: BoolExpr = parse_str(
          "team(A == B)"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::TeamEq(TeamExpr::TeamName(format_ident!("A")), TeamExpr::TeamName(format_ident!("B"))));
    }

    #[test]
    fn parses_valid_boolexpr_team_neq() {
        let parsed: BoolExpr = parse_str(
          "team(A != B)"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::TeamNeq(TeamExpr::TeamName(format_ident!("A")), TeamExpr::TeamName(format_ident!("B"))));
    }

    #[test]
    fn parses_valid_boolexpr_or() {
        let parsed: BoolExpr = parse_str(
          "(player(A != B) or player(A != B))"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::Or(
          Box::new(BoolExpr::PlayerNeq(PlayerExpr::PlayerName(format_ident!("A")), PlayerExpr::PlayerName(format_ident!("B")))),
          Box::new(BoolExpr::PlayerNeq(PlayerExpr::PlayerName(format_ident!("A")), PlayerExpr::PlayerName(format_ident!("B"))))
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_and() {
        let parsed: BoolExpr = parse_str(
          "(player(A != B) and player(A != B))"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::And(
          Box::new(BoolExpr::PlayerNeq(PlayerExpr::PlayerName(format_ident!("A")), PlayerExpr::PlayerName(format_ident!("B")))),
          Box::new(BoolExpr::PlayerNeq(PlayerExpr::PlayerName(format_ident!("A")), PlayerExpr::PlayerName(format_ident!("B"))))
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
          "cards(hand == hand)"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::CardSetEq(
          CardSet::Group(Group::Location(format_ident!("hand"))),
          CardSet::Group(Group::Location(format_ident!("hand"))),
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_cardset_neq() {
        let parsed: BoolExpr = parse_str(
          "cards(hand != hand)"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::CardSetNeq(
          CardSet::Group(Group::Location(format_ident!("hand"))),
          CardSet::Group(Group::Location(format_ident!("hand"))),
        ));
    }

    #[test]
    fn parses_valid_boolexpr_cardset_empty() {
        let parsed: BoolExpr = parse_str(
          "hand is empty"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::CardSetIsEmpty(
          CardSet::Group(Group::Location(format_ident!("hand")))
        ));
    }

    #[test]
    fn parses_valid_boolexpr_cardset_not_empty() {
        let parsed: BoolExpr = parse_str(
          "hand is not empty"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::CardSetIsNotEmpty(
          CardSet::Group(Group::Location(format_ident!("hand")))
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
          PlayerExpr::Current
        ));
    }
    
    #[test]
    fn parses_valid_boolexpr_out_of_game_player() {
        let parsed: BoolExpr = parse_str(
          "current out of game"
        ).unwrap();
        assert_eq!(parsed, BoolExpr::OutOfGamePlayer(
          PlayerExpr::Current
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
          format_ident!("Monkey")
        ));
    }

    #[test]
    fn parses_valid_stringexpr_key_of() {
        let parsed: StringExpr = parse_str(
          "rank of top(hand)"
        ).unwrap();
        assert_eq!(parsed, StringExpr::KeyOf(
          format_ident!("rank"),
          CardPosition::Top(format_ident!("hand"))
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
              StringExpr::ID(format_ident!("A")),
              StringExpr::ID(format_ident!("B")),
              StringExpr::ID(format_ident!("C"))
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
              PlayerExpr::Current,
              PlayerExpr::Current,
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
          "same rank"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Same(format_ident!("rank"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_distinct_key() {
        let parsed: FilterExpr = parse_str(
          "distinct rank"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Distinct(format_ident!("rank"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_adjacent_key() {
        let parsed: FilterExpr = parse_str(
          "adjacent rank using aces"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Adjacent(format_ident!("rank"), format_ident!("aces"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_higher_key() {
        let parsed: FilterExpr = parse_str(
          "higher rank using aces"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Higher(format_ident!("rank"), format_ident!("aces"))
        );
    }

    #[test]
    fn parses_valid_filter_expr_lower_key() {
        let parsed: FilterExpr = parse_str(
          "lower rank using aces"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::Lower(format_ident!("rank"), format_ident!("aces"))
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
          "key(rank == ace)"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::KeyEq(
            format_ident!("rank"),
            Box::new(StringExpr::ID(format_ident!("ace"))))
        );
    }

    #[test]
    fn parses_valid_filter_expr_rank_neq() {
        let parsed: FilterExpr = parse_str(
          "key(rank != ace)"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::KeyNeq(
            format_ident!("rank"),
            Box::new(StringExpr::ID(format_ident!("ace"))))
        );
    }

    #[test]
    fn parses_valid_filter_expr_not_combo() {
        let parsed: FilterExpr = parse_str(
          "not Pair"
        ).unwrap();
        assert_eq!(parsed, 
          FilterExpr::NotCombo(
            format_ident!("Pair")
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
            format_ident!("Pair")
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
              format_ident!("Pair")
            )),
            Box::new(FilterExpr::Combo(
              format_ident!("Triple")
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
              format_ident!("Pair")
            )),
            Box::new(FilterExpr::Combo(
              format_ident!("Triple")
            ))
          )
        );
    }

    // =======================================================================

    // Group =================================================================

    #[test]
    fn parses_valid_group_location() {
        let parsed: Group = parse_str(
          "hand"
        ).unwrap();
        assert_eq!(parsed, 
          Group::Location(
            format_ident!("hand")
          )
        );
    }

    #[test]
    fn parses_valid_group_location_filter() {
        let parsed: Group = parse_str(
          "hand where same rank"
        ).unwrap();
        assert_eq!(parsed, 
          Group::LocationWhere(
            format_ident!("hand"),
            FilterExpr::Same(format_ident!("rank"))
          )
        );
    }

    #[test]
    fn parses_valid_group_location_collection() {
        let parsed: Group = parse_str(
          "(hand, stack)"
        ).unwrap();
        assert_eq!(parsed, 
          Group::LocationCollection(
            LocationCollection {
              locations: vec![
                format_ident!("hand"),
                format_ident!("stack")
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_group_location_collection_filter() {
        let parsed: Group = parse_str(
          "(hand, stack) where same rank"
        ).unwrap();
        assert_eq!(parsed, 
          Group::LocationCollectionWhere(
            LocationCollection {
              locations: vec![
                format_ident!("hand"),
                format_ident!("stack")
              ]
            },
            FilterExpr::Same(format_ident!("rank"))
          )
        );
    }


    #[test]
    fn parses_valid_group_combo_in_location() {
        let parsed: Group = parse_str(
          "Pair in hand"
        ).unwrap();
        assert_eq!(parsed, 
          Group::ComboInLocation(
            format_ident!("Pair"),
            format_ident!("hand")
          )
        );
    }

    #[test]
    fn parses_valid_group_combo_in_location_collection() {
        let parsed: Group = parse_str(
          "Pair in (hand, stack)"
        ).unwrap();
        assert_eq!(parsed, 
          Group::ComboInLocationCollection(
            format_ident!("Pair"),
            LocationCollection {
              locations: vec![
                format_ident!("hand"),
                format_ident!("stack")
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_group_combo_not_in_location() {
        let parsed: Group = parse_str(
          "Pair not in hand"
        ).unwrap();
        assert_eq!(parsed, 
          Group::NotComboInLocation(
            format_ident!("Pair"),
            format_ident!("hand")
          )
        );
    }

    #[test]
    fn parses_valid_group_combo_not_in_location_collection() {
        let parsed: Group = parse_str(
          "Pair not in (hand, stack)"
        ).unwrap();
        assert_eq!(parsed, 
          Group::NotComboInLocationCollection(
            format_ident!("Pair"),
            LocationCollection {
              locations: vec![
                format_ident!("hand"),
                format_ident!("stack")
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_group_cardposition() {
        let parsed: Group = parse_str(
          "top(hand)"
        ).unwrap();
        assert_eq!(parsed, 
          Group::CardPosition(
            CardPosition::Top(format_ident!("hand"))
          )
        );
    }

    // =======================================================================

    // CardSet ===============================================================

    #[test]
    fn parses_valid_cardset_group() {
        let parsed: CardSet = parse_str(
          "top(hand)"
        ).unwrap();
        assert_eq!(parsed, 
          CardSet::Group(
            Group::CardPosition(
              CardPosition::Top(format_ident!("hand"))
            )
          )
        );
    }

    #[test]
    fn parses_valid_cardset_group_of_player() {
        let parsed: CardSet = parse_str(
          "hand where same rank of current"
        ).unwrap();
        assert_eq!(parsed, 
          CardSet::GroupOfPlayer(
            Group::LocationWhere(
              format_ident!("hand"),
              FilterExpr::Same(format_ident!("rank"))
            ),
            PlayerExpr::Current
          )
        );
    }

    #[test]
    fn parses_valid_cardset_group_of_player_collection() {
        let parsed: CardSet = parse_str(
          "hand where same rank of others"
        ).unwrap();
        assert_eq!(parsed, 
          CardSet::GroupOfPlayerCollection(
            Group::LocationWhere(
              format_ident!("hand"),
              FilterExpr::Same(format_ident!("rank"))
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
          "(hand, deck, hand)"
        ).unwrap();
        assert_eq!(parsed, 
          LocationCollection {
            locations: vec![
              format_ident!("hand"),
              format_ident!("deck"),
              format_ident!("hand"),
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
              TeamExpr::TeamName(format_ident!("T1")),
              TeamExpr::TeamName(format_ident!("T2")),
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
              StringExpr::ID(format_ident!("A")),
              StringExpr::ID(format_ident!("B")),
            ]
          }
        );
    }

    // =======================================================================

    // Collection ============================================================

    #[test]
    fn parses_valid_collection_playercollection() {
        let parsed: Collection = parse_str(
          "players(current, previous)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::PlayerCollection(
            PlayerCollection::Player(
              vec![
                PlayerExpr::Current,
                PlayerExpr::Previous,
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_collection_teamcollection() {
        let parsed: Collection = parse_str(
          "teams(T1, T2)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::TeamCollection(
            TeamCollection::Team(
              vec![
                TeamExpr::TeamName(format_ident!("T1")),
                TeamExpr::TeamName(format_ident!("T2")),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_collection_intcollection() {
        let parsed: Collection = parse_str(
          "ints(1, 2, 3)"
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
    fn parses_valid_collection_locationcollection() {
        let parsed: Collection = parse_str(
          "locations(hand, deck, hand)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::LocationCollection(
            LocationCollection {
              locations: vec![
                format_ident!("hand"),
                format_ident!("deck"),
                format_ident!("hand")
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_collection_cardset() {
        let parsed: Collection = parse_str(
          "cards(hand, deck, hand)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::CardSet(
            Box::new(
              CardSet::Group(
                Group::LocationCollection(
                  LocationCollection {
                    locations: vec![
                      format_ident!("hand"),
                      format_ident!("deck"),
                      format_ident!("hand")
                    ]
                  }
                )
              )
            )
          )
        );
    }

    #[test]
    fn parses_valid_collection_stringcollection() {
        let parsed: Collection = parse_str(
          "(A, B, C)"
        ).unwrap();
        assert_eq!(parsed,
          Collection::StringCollection(
            StringCollection {
            strings: vec![
              StringExpr::ID(format_ident!("A")),
              StringExpr::ID(format_ident!("B")),
              StringExpr::ID(format_ident!("C")),
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
          "range(== 2)"
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
          "range(!= 2)"
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
          "range(>= 2)"
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
          "range(<= 2)"
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
          "range(> 2)"
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
          "range(< 2)"
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
          "range(== 3)"
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
          "move hand private to deck"
        ).unwrap();
        assert_eq!(parsed,
          ClassicMove::Move(
            CardSet::Group(Group::Location(format_ident!("hand"))),
            Status::Private,
            CardSet::Group(Group::Location(format_ident!("deck")))
          )
        );
    }

    #[test]
    fn parses_valid_classicmove_move_quantity() {
        let parsed: ClassicMove = parse_str(
          "move all from hand private to deck"
        ).unwrap();
        assert_eq!(parsed,
          ClassicMove::MoveQuantity(
            Quantity::Quantifier(Quantifier::All),
            CardSet::Group(Group::Location(format_ident!("hand"))),
            Status::Private,
            CardSet::Group(Group::Location(format_ident!("deck")))
          )
        );
    }

    // =======================================================================

    // DealMove ===========================================================

    #[test]
    fn parses_valid_dealmove_deal() {
        let parsed: DealMove = parse_str(
          "deal hand private to deck"
        ).unwrap();
        assert_eq!(parsed,
          DealMove::Deal(
            CardSet::Group(Group::Location(format_ident!("hand"))),
            Status::Private,
            CardSet::Group(Group::Location(format_ident!("deck")))
          )
        );
    }

    #[test]
    fn parses_valid_dealmove_deal_quantity() {
        let parsed: DealMove = parse_str(
          "deal 12 from hand private to deck of all"
        ).unwrap();
        assert_eq!(parsed,
          DealMove::DealQuantity(
            Quantity::Int(IntExpr::Int(12)),
            CardSet::Group(Group::Location(format_ident!("hand"))),
            Status::Private,
            CardSet::GroupOfPlayerCollection(Group::Location(format_ident!("deck")), PlayerCollection::Quantifier(Quantifier::All))
          )
        );
    }

    // =======================================================================

        // DealMove ===========================================================

    #[test]
    fn parses_valid_exchangemove_exchange() {
        let parsed: ExchangeMove = parse_str(
          "exchange hand private with deck"
        ).unwrap();
        assert_eq!(parsed,
          ExchangeMove::Exchange(
            CardSet::Group(Group::Location(format_ident!("hand"))),
            Status::Private,
            CardSet::Group(Group::Location(format_ident!("deck")))
          )
        );
    }

    #[test]
    fn parses_valid_exchangemove_exchange_quantity() {
        let parsed: ExchangeMove = parse_str(
          "exchange all from hand private with deck"
        ).unwrap();
        assert_eq!(parsed,
          ExchangeMove::ExchangeQuantity(
            Quantity::Quantifier(Quantifier::All),
            CardSet::Group(Group::Location(format_ident!("hand"))),
            Status::Private,
            CardSet::Group(Group::Location(format_ident!("deck")))
          )
        );
    }

    // =======================================================================

    // TokenLocExpr ==========================================================

    #[test]
    fn parses_valid_tokenloc_expr_location() {
        let parsed: TokenLocExpr = parse_str(
          "hand"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::Location(
            format_ident!("hand")
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_location_player() {
        let parsed: TokenLocExpr = parse_str(
          "hand of current"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationPlayer(
            format_ident!("hand"),
            PlayerExpr::Current
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_location_playercollection() {
        let parsed: TokenLocExpr = parse_str(
          "hand of others"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationPlayerCollection(
            format_ident!("hand"),
            PlayerCollection::Others
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_locationcollection() {
        let parsed: TokenLocExpr = parse_str(
          "(hand, deck)"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationCollection(
            LocationCollection {
              locations: vec![
                format_ident!("hand"),
                format_ident!("deck"),
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_locationcollection_player() {
        let parsed: TokenLocExpr = parse_str(
          "(hand, deck) of current"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationCollectionPlayer(
            LocationCollection {
              locations: vec![
                format_ident!("hand"),
                format_ident!("deck"),
              ]
            },
            PlayerExpr::Current
          )
        );
    }

    #[test]
    fn parses_valid_tokenloc_expr_locationcollection_playercollection() {
        let parsed: TokenLocExpr = parse_str(
          "(hand, deck) of others"
        ).unwrap();
        assert_eq!(parsed,
          TokenLocExpr::LocationCollectionPlayerCollection(
            LocationCollection {
              locations: vec![
                format_ident!("hand"),
                format_ident!("deck"),
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
          "place hand to deck"
        ).unwrap();
        assert_eq!(parsed,
          TokenMove::Place(
            TokenLocExpr::Location(
              format_ident!("hand")
            ),
            TokenLocExpr::Location(
              format_ident!("deck")
            ),
          )
        );
    }

    #[test]
    fn parses_valid_tokenmove_place_quantity() {
        let parsed: TokenMove = parse_str(
          "place all from hand to deck"
        ).unwrap();
        assert_eq!(parsed,
          TokenMove::PlaceQuantity(
            Quantity::Quantifier(Quantifier::All),
            TokenLocExpr::Location(
              format_ident!("hand")
            ),
            TokenLocExpr::Location(
              format_ident!("deck")
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
              format_ident!("P1"),
              format_ident!("P2"),
              format_ident!("P3"),
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
            format_ident!("T1"),
            vec![
              format_ident!("P1"),
              format_ident!("P2"),
              format_ident!("P3"),
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
              format_ident!("P1"),
              format_ident!("P2"),
              format_ident!("P3"),
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
              format_ident!("P1"),
              format_ident!("P2"),
              format_ident!("P3"),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_createlocation_playercollection() {
        let parsed: Rule = parse_str(
          "location hand on players(P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateLocationOnPlayerCollection(
            format_ident!("hand"),
            PlayerCollection::Player(
              vec![
                PlayerExpr::PlayerName(format_ident!("P1")),
                PlayerExpr::PlayerName(format_ident!("P2")),
                PlayerExpr::PlayerName(format_ident!("P3")),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_createlocation_teamcollection() {
        let parsed: Rule = parse_str(
          "location hand on teams(T1, T2, T3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateLocationOnTeamCollection(
            format_ident!("hand"),
            TeamCollection::Team(
              vec![
                TeamExpr::TeamName(format_ident!("T1")),
                TeamExpr::TeamName(format_ident!("T2")),
                TeamExpr::TeamName(format_ident!("T3")),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_createlocation_table() {
        let parsed: Rule = parse_str(
          "location stack on table"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateLocationOnTable(
            format_ident!("stack")
          )
        );
    }

    #[test]
    fn parses_valid_rule_createcard() {
        let parsed: Rule = parse_str(
          "card on stack: 
            Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
              for Suite(Spades, Clubs)
                for Color(Black)
          "
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateCardOnLocation(
            format_ident!("stack"),
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
                  format_ident!("Spades"),
                  format_ident!("Clubs"),
                ]),
                (format_ident!("Color"), vec![
                  format_ident!("Black"),
                ]),
              ]
            }
          )
        );
    }

    #[test]
    fn parses_valid_rule_createtoken() {
        let parsed: Rule = parse_str(
          "token 10 Chip on stack"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateTokenOnLocation(
            IntExpr::Int(10),
            format_ident!("Chip"),
            format_ident!("stack")
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
            format_ident!("Rank"),
            vec![ 
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
              (format_ident!("Rank"), format_ident!("Ace"))
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
            format_ident!("Rank"),
            vec![
              (format_ident!("Rank"), format_ident!("Two")),
              (format_ident!("Suite"), format_ident!("Spades")),
              (format_ident!("Color"), format_ident!("Red")),
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
            format_ident!("SameSuite"),
            FilterExpr::Same(format_ident!("Suite"))
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_playercollection() {
        let parsed: Rule = parse_str(
          "memory Square on (P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryPlayerCollection(
            format_ident!("Square"),
            PlayerCollection::Player(
              vec![
                PlayerExpr::PlayerName(format_ident!("P1")),
                PlayerExpr::PlayerName(format_ident!("P2")),
                PlayerExpr::PlayerName(format_ident!("P3")),
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
            format_ident!("Square")
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_int_playercollection() {
        let parsed: Rule = parse_str(
          "memory Square 10 on (P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryIntPlayerCollection(
            format_ident!("Square"),
            IntExpr::Int(10),
            PlayerCollection::Player(
              vec![
                PlayerExpr::PlayerName(format_ident!("P1")),
                PlayerExpr::PlayerName(format_ident!("P2")),
                PlayerExpr::PlayerName(format_ident!("P3")),
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
            format_ident!("Square"),
            IntExpr::Int(10),
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_string_playercollection() {
        let parsed: Rule = parse_str(
          "memory Square monkey on (P1, P2, P3)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryStringPlayerCollection(
            format_ident!("Square"),
            StringExpr::ID(format_ident!("monkey")),
            PlayerCollection::Player(
              vec![
                PlayerExpr::PlayerName(format_ident!("P1")),
                PlayerExpr::PlayerName(format_ident!("P2")),
                PlayerExpr::PlayerName(format_ident!("P3")),
              ]
            )
          )
        );
    }

    #[test]
    fn parses_valid_rule_create_memory_string_table() {
        let parsed: Rule = parse_str(
          "memory Square monkey on table"
        ).unwrap();
        assert_eq!(parsed,
          Rule::CreateMemoryStringTable(
            format_ident!("Square"),
            StringExpr::ID(format_ident!("monkey")),
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
            format_ident!("Rank"),
            vec![
              (format_ident!("Rank"), format_ident!("Two"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Three"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Four"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Five"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Six"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Seven"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Eight"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Nine"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Ten"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Jack"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Queen"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("King"), IntExpr::Int(1)),
              (format_ident!("Rank"), format_ident!("Ace"), IntExpr::Int(1)),
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
            format_ident!("Rank"),
            vec![
              (format_ident!("Rank"), format_ident!("Two"), IntExpr::Int(1)),
              (format_ident!("Suite"), format_ident!("Spades"), IntExpr::Int(1)),
              (format_ident!("Color"), format_ident!("Red"), IntExpr::Int(1)),
            ]
          )
        );
    }

    #[test]
    fn parses_valid_rule_flip_action() {
        let parsed: Rule = parse_str(
          "flip hand to private"
        ).unwrap();
        assert_eq!(parsed,
          Rule::FlipAction(
            CardSet::Group(Group::Location(format_ident!("hand"))),
            Status::Private
          )
        );
    }
    
    #[test]
    fn parses_valid_rule_shuffle_action() {
        let parsed: Rule = parse_str(
          "shuffle hand"
        ).unwrap();
        assert_eq!(parsed,
          Rule::ShuffleAction(
            CardSet::Group(Group::Location(format_ident!("hand"))),
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
            PlayerExpr::Current
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
            PlayerExpr::Current
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
            PlayerExpr::Current
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
                PlayerExpr::Current
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
                PlayerExpr::Current
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
                PlayerExpr::Current
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
            format_ident!("Square"),
            IntExpr::Int(10)
          )
        );
    }

    #[test]
    fn parses_valid_rule_set_memory_string() {
        let parsed: Rule = parse_str(
          "Square is A"
        ).unwrap();
        assert_eq!(parsed,
          Rule::SetMemoryString(
            format_ident!("Square"),
            StringExpr::ID(format_ident!("A"))
          )
        );
    }

    #[test]
    fn parses_valid_rule_set_memory_collection() {
        let parsed: Rule = parse_str(
          "Square is players(current)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::SetMemoryCollection(
            format_ident!("Square"),
            Collection::PlayerCollection(
              PlayerCollection::Player(
                vec![
                  PlayerExpr::Current
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
            format_ident!("Square"),
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
            PlayerExpr::Current
          )
        );
    }
    
    #[test]
    fn parses_valid_rule_demand_card_position() {
        let parsed: Rule = parse_str(
          "demand top(hand)"
        ).unwrap();
        assert_eq!(parsed,
          Rule::DemandCardPositionAction(
            CardPosition::Top(
              format_ident!("hand")
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
              format_ident!("A")
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
              deal 12 from stock private to hand of all;
            }
          "
        ).unwrap();
        assert_eq!(parsed,
          SeqStage {
            stage: format_ident!("Play"), 
            player: PlayerExpr::Current, 
            end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
            flows: vec![
              FlowComponent::Rule(
                Rule::DealMove(
                  DealMove::DealQuantity(
                    Quantity::Int(IntExpr::Int(12)), 
                    CardSet::Group(Group::Location(format_ident!("stock"))), 
                    Status::Private, 
                    CardSet::GroupOfPlayerCollection(Group::Location(format_ident!("hand")), PlayerCollection::Quantifier(Quantifier::All))
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
            condition: BoolExpr::OutOfStagePlayer(PlayerExpr::Current),
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
            location (hand, laydown, trash) on players all;
            location (stock, discard) on table;
            card on stock:
              Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
                for Suite(Diamonds, Hearts, Spades, Clubs);
            precedence RankOrder on Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King);
            pointmap Values on Rank(Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6, Seven: 7, Eight: 8, Nine: 9 , Ten: 10, Jack: 10, Queen: 10, King: 10);
            combo Sequence where ((size >= 3 and same Suite) and adjacent Rank using RankOrder);
            combo Set where ((size >= 3 and distinct Suite) and same Rank);
            combo Deadwood where (not Sequence and not Set);

            stage Preparation for current until(1 times) {
              deal 12 from top(stock) private to hand of all;
            }

            stage Collect for current until(previous out of stage) {
              choose {
                move top(discard) private to hand;
                or
                move top(stock) private to hand;
              }

              move any from hand face up to top(discard);

              if (sum of Deadwood in hand using Values <= 10) {
                optional {
                  move all from Set in hand face up to top(laydown);
                  move all from Sequence in hand face up to top(laydown);

                  if (hand is empty) {
                    move all from Set in hand of next face up to top(laydown) of next;
                    move all from Sequence in hand of next face up to top(laydown) of next;
                    move hand of next face up to trash of next;

                    move hand face up to trash;
                    set current out of stage;
                  }
                }
              }

              cycle to next;
            }

            stage FinalLayDown for current until(1 times) {
              move laydown of previous face up to hand of current;
              move all from Set in hand face up to top(laydown);
              move all from Sequence in hand face up to top(laydown);

              move hand face up to trash;
            }

            score sum of trash using Values to LeftOver of all;
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
    }

    // =======================================================================

}