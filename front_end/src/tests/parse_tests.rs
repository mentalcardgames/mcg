// #[cfg(test)]
// mod tests {
//     use front_end::transform_to_typed::Lower;
//     use front_end::typed_ast::*;
//     use front_end::helper::test_helper::{self as th, ctx_max_cardpos, ctx_min_cardpos};
//     use front_end::helper::test_helper::ctx;
//     use front_end::diagnostic::*;

//     use syn::parse_str;

//     // PlayerExpr ============================================================
    
//     #[test]
//     fn parses_valid_player_current() {
//         let parsed: SPlayerExpr = parse_str(
//           "current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( th::CURRENT));
//     }

//     #[test]
//     fn parses_valid_player_previous() {
//         let parsed: SPlayerExpr = parse_str(
//           "previous"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( th::PREVIOUS));
//     }

//     #[test]
//     fn parses_valid_player_competitor() {
//         let parsed: SPlayerExpr = parse_str(
//           "competitor"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( th::COMPETITOR));
//     }

//     #[test]
//     fn parses_valid_player_owner_highest() {
//         let parsed: SPlayerExpr = parse_str(
//           "owner of highest Mem"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           PlayerExpr::OwnerOfHighest(
//             th::memory("Mem")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_player_owner_lowest() {
//         let parsed: SPlayerExpr = parse_str(
//           "owner of lowest Mem"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           PlayerExpr::OwnerOfLowest(
//             th::memory("Mem")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_player_turnorder() {
//         let parsed: SPlayerExpr = parse_str(
//           "turnorder(3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( PlayerExpr::Turnorder(IntExpr::Int(3))));
//     }
    
//     #[test]
//     fn parses_valid_player_id() {
//         let parsed: SPlayerExpr = parse_str(
//           "P1"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( PlayerExpr::PlayerName(
//           th::playername("P1"))
//         ));
//     }

//     // =======================================================================

//     // Op ====================================================================
    
//     #[test]
//     fn parses_valid_op_plus() {
//         let parsed: SOp = parse_str(
//           "+"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Op::Plus));
//     }

//     #[test]
//     fn parses_valid_op_minus() {
//         let parsed: SOp = parse_str(
//           "-"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Op::Minus));
//     }

//     #[test]
//     fn parses_valid_op_div() {
//         let parsed: SOp = parse_str(
//           "/"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Op::Div));
//     }

//     #[test]
//     fn parses_valid_op_mul() {
//         let parsed: SOp = parse_str(
//           "*"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Op::Mul));
//     }
    
//     #[test]
//     fn parses_valid_op_mod() {
//         let parsed: SOp = parse_str(
//           "%"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Op::Mod));
//     }
//     // =======================================================================

//     // IntCmpOp ==============================================================
    
//     #[test]
//     fn parses_valid_intcmpop_eq() {
//         let parsed: SIntCmpOp = parse_str(
//           "=="
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntCmpOp::Eq));
//     }

//     #[test]
//     fn parses_valid_intcmpop_neq() {
//         let parsed: SIntCmpOp = parse_str(
//           "!="
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntCmpOp::Neq));
//     }

//     #[test]
//     fn parses_valid_intcmpop_le() {
//         let parsed: SIntCmpOp = parse_str(
//           "<="
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntCmpOp::Le));
//     }

//     #[test]
//     fn parses_valid_intcmpop_ge() {
//         let parsed: SIntCmpOp = parse_str(
//           ">="
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntCmpOp::Ge));
//     }

//     #[test]
//     fn parses_valid_intcmpop_lt() {
//         let parsed: SIntCmpOp = parse_str(
//           "<"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntCmpOp::Lt));
//     }

//     #[test]
//     fn parses_valid_intcmpop_gt() {
//         let parsed: SIntCmpOp = parse_str(
//           ">"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntCmpOp::Gt));
//     }
    
//     // =======================================================================

//     // Status ================================================================

//     #[test]
//     fn parses_valid_status_facup() {
//         let parsed: SStatus = parse_str(
//           "face up"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Status::FaceUp));
//     }

//     #[test]
//     fn parses_valid_facedown() {
//         let parsed: SStatus = parse_str(
//           "face down"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Status::FaceDown));
//     }
    
//     #[test]
//     fn parses_valid_private() {
//         let parsed: SStatus = parse_str(
//           "private"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Status::Private));
//     }
    
//     // =======================================================================

//     // Quantifier ============================================================
    
//     #[test]
//     fn parses_valid_quantifier_all() {
//         let parsed: SQuantifier = parse_str(
//           "all"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Quantifier::All));
//     }

//     #[test]
//     fn parses_valid_quantifier_any() {
//         let parsed: SQuantifier = parse_str(
//           "any"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( Quantifier::Any));
//     }

//     // =======================================================================

//     // TeamExpr ==============================================================
    
//     #[test]
//     fn parses_valid_teamexpr_team_of() {
//         let parsed: STeamExpr = parse_str(
//           "team of current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( TeamExpr::TeamOf(th::CURRENT)));
//     }

//     #[test]
//     fn parses_valid_teamexpr_team_id() {
//         let parsed: STeamExpr = parse_str(
//           "T1"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( TeamExpr::TeamName(
//           th::teamname("T1"))
//         ));
//     }

//     // =======================================================================

//     // CardPosition ==========================================================

//     #[test]
//     fn parses_valid_cardposition_top_of() {
//         let parsed: SCardPosition = parse_str(
//           "top(Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( CardPosition::Top(
//           th::location("Hand"))
//         ));
//     }

//     #[test]
//     fn parses_valid_cardposition_bottom_of() {
//         let parsed: SCardPosition = parse_str(
//           "bottom(Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( CardPosition::Bottom(th::location("Hand"))));
//     }

//     #[test]
//     fn parses_valid_cardposition_max() {
//         let parsed: SCardPosition = parse_str(
//           "max(Hand) using Aces"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx_max_cardpos()), Ok( CardPosition::Max(Box::new(CardSet::Group(Group::Location(th::location("Hand")))), th::pointmap("Aces"))));
//     }

//     #[test]
//     fn parses_valid_cardposition_min() {
//         let parsed: SCardPosition = parse_str(
//           "min(Hand) using Aces"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx_min_cardpos()), Ok( CardPosition::Min(Box::new(CardSet::Group(Group::Location(th::location("Hand")))), th::precedence("Aces"))));
//     }

//     #[test]
//     fn parses_valid_cardposition_at() {
//         let parsed: SCardPosition = parse_str(
//           "Hand[3]"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( CardPosition::At(th::location("Hand"), IntExpr::Int(3))));
//     }

//     // =======================================================================

//     // IntExpr ===============================================================

//     #[test]
//     fn parses_valid_intexpr_int() {
//         let parsed: SIntExpr = parse_str(
//           "3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::Int(3)));
//     }

//     #[test]
//     fn parses_valid_intexpr_op() {
//         let parsed: SIntExpr = parse_str(
//           "(3 + 3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::IntOp(Box::new(IntExpr::Int(3)), Op::Plus, Box::new(IntExpr::Int(3)))));
//     }

//     #[test]
//     fn parses_valid_intexpr_size_of() {
//         let parsed: SIntExpr = parse_str(
//           "size of (3, 3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::SizeOf(Collection::IntCollection(
//           IntCollection {
//             ints: vec![IntExpr::Int(3), IntExpr::Int(3)]
//           }
//         ))));
//     }

//     #[test]
//     fn parses_valid_intexpr_sum() {
//         let parsed: SIntExpr = parse_str(
//           "sum(3, 3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::SumOfIntCollection(
//           IntCollection {
//             ints: vec![IntExpr::Int(3), IntExpr::Int(3)]
//           }
//         )));
//     }
    
//     #[test]
//     fn parses_valid_intexpr_sum_of() {
//         let parsed: SIntExpr = parse_str(
//           "sum of Hand using Aces"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::SumOfCardSet(
//           Box::new(CardSet::Group(Group::Location(th::location("Hand")))), th::pointmap("Aces"))
//         ));
//     }

//     #[test]
//     fn parses_valid_intexpr_min_intcollection() {
//         let parsed: SIntExpr = parse_str(
//           "min(3, 3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::MinIntCollection(
//           IntCollection {
//             ints: vec![IntExpr::Int(3), IntExpr::Int(3)]
//           }
//         )));
//     }
    
//     #[test]
//     fn parses_valid_intexpr_max_intcollection() {
//         let parsed: SIntExpr = parse_str(
//           "max(3, 3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::MaxIntCollection(
//           IntCollection {
//             ints: vec![IntExpr::Int(3), IntExpr::Int(3)]
//           }
//         )));
//     }
    
//     #[test]
//     fn parses_valid_intexpr_min_pointmap() {
//         let parsed: SIntExpr = parse_str(
//           "min of Hand using Aces"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::MinOf(
//           Box::new(CardSet::Group(Group::Location(th::location("Hand")))), th::pointmap("Aces"))
//         ));
//     }
    
//     #[test]
//     fn parses_valid_intexpr_max_pointmap() {
//         let parsed: SIntExpr = parse_str(
//           "max of Hand using Aces"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::MaxOf(
//           Box::new(CardSet::Group(Group::Location(th::location("Hand")))), th::pointmap("Aces"))
//         ));
//     }
    
//     #[test]
//     fn parses_valid_intexpr_stageroundcounter() {
//         let parsed: SIntExpr = parse_str(
//           "stageroundcounter"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( IntExpr::StageRoundCounter));
//     }

//     // =======================================================================

//     // BoolExpr ==============================================================

//     #[test]
//     fn parses_valid_boolexpr_eq() {
//         let parsed: SBoolExpr = parse_str(
//           "A == B"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::CardSetEq(
//           CardSet::Group(Group::Location(th::location("A"))),
//           CardSet::Group(Group::Location(th::location("B"))),
//         )));
//     }

//     #[test]
//     fn parses_valid_boolexpr_neq() {
//         let parsed: SBoolExpr = parse_str(
//           "A != B"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::CardSetNeq(
//           CardSet::Group(Group::Location(th::location("A"))),
//           CardSet::Group(Group::Location(th::location("B"))),
//         )));
//     }

//     #[test]
//     fn parses_valid_boolexpr_player_eq() {
//         let parsed: SBoolExpr = parse_str(
//           "current == A"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::PlayerEq(th::CURRENT, PlayerExpr::PlayerName(th::playername("A")))));
//     }

//     #[test]
//     fn parses_valid_boolexpr_player_neq() {
//         let parsed: SBoolExpr = parse_str(
//           "A != current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::PlayerNeq(PlayerExpr::PlayerName(th::playername("A")), th::CURRENT)));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_team_eq() {
//         let parsed: SBoolExpr = parse_str(
//           "team of A == B"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::TeamEq(TeamExpr::TeamOf(PlayerExpr::PlayerName(th::playername("A"))), TeamExpr::TeamName(th::teamname("B")))));
//     }

//     #[test]
//     fn parses_valid_boolexpr_team_neq() {
//         let parsed: SBoolExpr = parse_str(
//           "A != team of B"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::TeamNeq(TeamExpr::TeamName(th::teamname("A")), TeamExpr::TeamOf(PlayerExpr::PlayerName(th::playername("B"))))));
//     }

//     #[test]
//     fn parses_valid_boolexpr_or() {
//         let parsed: SBoolExpr = parse_str(
//           "(A != B or A != B)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::Or(
//           Box::new(
//             BoolExpr::CardSetNeq(
//               CardSet::Group(Group::Location(th::location("A"))),
//               CardSet::Group(Group::Location(th::location("B"))),
//             )
//           ),
//           Box::new(
//             BoolExpr::CardSetNeq(
//               CardSet::Group(Group::Location(th::location("A"))),
//               CardSet::Group(Group::Location(th::location("B"))),
//             )
//           )
//         )));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_and() {
//         let parsed: SBoolExpr = parse_str(
//           "(A != B and A != B)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::And(
//           Box::new(
//             BoolExpr::CardSetNeq(
//               CardSet::Group(Group::Location(th::location("A"))),
//               CardSet::Group(Group::Location(th::location("B"))),
//             )
//           ),
//           Box::new(
//             BoolExpr::CardSetNeq(
//               CardSet::Group(Group::Location(th::location("A"))),
//               CardSet::Group(Group::Location(th::location("B"))),
//             )
//           )
//         )));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_intcmp() {
//         let parsed: SBoolExpr = parse_str(
//           "3 == 2"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::IntCmp(
//           IntExpr::Int(3),
//           IntCmpOp::Eq,
//           IntExpr::Int(2)
//         )));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_cardset_eq() {
//         let parsed: SBoolExpr = parse_str(
//           "Hand of current == Hand"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::CardSetEq(
//           CardSet::GroupOfPlayer(Group::Location(th::location("Hand")), th::CURRENT),
//           CardSet::Group(Group::Location(th::location("Hand"))),
//         )));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_cardset_neq() {
//         let parsed: SBoolExpr = parse_str(
//           "Hand != Hand of current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::CardSetNeq(
//           CardSet::Group(Group::Location(th::location("Hand"))),
//           CardSet::GroupOfPlayer(Group::Location(th::location("Hand")), th::CURRENT),
//         )));
//     }

//     #[test]
//     fn parses_valid_boolexpr_cardset_empty() {
//         let parsed: SBoolExpr = parse_str(
//           "Hand is empty"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::CardSetIsEmpty(
//           CardSet::Group(Group::Location(th::location("Hand")))
//         )));
//     }

//     #[test]
//     fn parses_valid_boolexpr_cardset_not_empty() {
//         let parsed: SBoolExpr = parse_str(
//           "Hand is not empty"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::CardSetIsNotEmpty(
//           CardSet::Group(Group::Location(th::location("Hand")))
//         )));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_not() {
//         let parsed: SBoolExpr = parse_str(
//           "not 3 == 2"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::Not(
//           Box::new(BoolExpr::IntCmp(
//             IntExpr::Int(3),
//             IntCmpOp::Eq,
//             IntExpr::Int(2)
//         )))));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_out_of_stage_player() {
//         let parsed: SBoolExpr = parse_str(
//           "current out of stage"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::OutOfStagePlayer(
//           th::CURRENT
//         )));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_out_of_game_player() {
//         let parsed: SBoolExpr = parse_str(
//           "current out of game"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::OutOfGamePlayer(
//           th::CURRENT
//         )));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_out_of_stage_collection() {
//         let parsed: SBoolExpr = parse_str(
//           "others out of stage"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::OutOfStageCollection(
//           PlayerCollection::Others
//         )));
//     }
    
//     #[test]
//     fn parses_valid_boolexpr_out_of_game_collection() {
//         let parsed: SBoolExpr = parse_str(
//           "others out of game"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( BoolExpr::OutOfGameCollection(
//           PlayerCollection::Others
//         )));
//     }
    
//     // =======================================================================

//     // StringExpr ============================================================
    
//     #[test]
//     fn parses_valid_stringexpr_key_of() {
//         let parsed: SStringExpr = parse_str(
//           "Rank of top(Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( StringExpr::KeyOf(
//           th::key("Rank"),
//           CardPosition::Top(th::location("Hand"))
//         )));
//     }

//     // #[test]
//     // fn parses_valid_stringexpr_collection_at() {
//     //     let parsed: SStringExpr = parse_str(
//     //       "(A, B, C)[3]"
//     //     ).unwrap();
//     //     assert_eq!(parsed.lower(&ctx()), Ok( StringExpr::StringCollectionAt(
//     //       StringCollection {
//     //         strings: vec![
//     //           StringExpr::ID(th::id("A")),
//     //           StringExpr::ID(th::id("B")),
//     //           StringExpr::ID(th::id("C"))
//     //         ]
//     //       },
//     //       IntExpr::Int(3)
//     //     )));
//     // }

//     // =======================================================================

//     // PlayerCollection ======================================================
   
//     #[test]
//     fn parses_valid_player_collection_others() {
//         let parsed: SPlayerCollection = parse_str(
//           "others"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           PlayerCollection::Others
//         ));
//     }

//     #[test]
//     fn parses_valid_player_collection_playersin() {
//         let parsed: SPlayerCollection = parse_str(
//           "playersin"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           PlayerCollection::PlayersIn
//         ));
//     }

//     #[test]
//     fn parses_valid_player_collection_playersout() {
//         let parsed: SPlayerCollection = parse_str(
//           "playersout"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           PlayerCollection::PlayersOut
//         ));
//     }

//     #[test]
//     fn parses_valid_player_collection_collection() {
//         let parsed: SPlayerCollection = parse_str(
//           "(current, current)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           PlayerCollection::Player(
//             vec![
//               th::CURRENT,
//               th::CURRENT,
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_player_collection_quantifier() {
//         let parsed: SPlayerCollection = parse_str(
//           "all"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           PlayerCollection::Quantifier(
//             Quantifier::All
//           )
//         ));
//     }

//     // =======================================================================

//     // FilterExpr ============================================================

//     #[test]
//     fn parses_valid_filter_expr_same_key() {
//         let parsed: SFilterExpr = parse_str(
//           "same Rank"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Same(th::key("Rank"))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_distinct_key() {
//         let parsed: SFilterExpr = parse_str(
//           "distinct Rank"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Distinct(th::key("Rank"))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_adjacent_key() {
//         let parsed: SFilterExpr = parse_str(
//           "adjacent Rank using Aces"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Adjacent(th::key("Rank"), th::precedence("Aces"))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_higher_key() {
//         let parsed: SFilterExpr = parse_str(
//           "higher Rank using Aces"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Higher(th::key("Rank"), th::precedence("Aces"))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_lower_key() {
//         let parsed: SFilterExpr = parse_str(
//           "lower Rank using Aces"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Lower(th::key("Rank"), th::precedence("Aces"))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_size_eq() {
//         let parsed: SFilterExpr = parse_str(
//           "size == 3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Size(IntCmpOp::Eq, Box::new(IntExpr::Int(3)))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_size_neq() {
//         let parsed: SFilterExpr = parse_str(
//           "size != 3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Size(IntCmpOp::Neq, Box::new(IntExpr::Int(3)))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_size_lt() {
//         let parsed: SFilterExpr = parse_str(
//           "size < 3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Size(IntCmpOp::Lt, Box::new(IntExpr::Int(3)))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_size_gt() {
//         let parsed: SFilterExpr = parse_str(
//           "size > 3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Size(IntCmpOp::Gt, Box::new(IntExpr::Int(3)))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_size_le() {
//         let parsed: SFilterExpr = parse_str(
//           "size <= 3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Size(IntCmpOp::Le, Box::new(IntExpr::Int(3)))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_size_ge() {
//         let parsed: SFilterExpr = parse_str(
//           "size >= 3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_rank_eq() {
//         let parsed: SFilterExpr = parse_str(
//           "Rank == Ace"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::KeyEqValue(
//             th::key("Rank"),
//             th::value("Ace"),
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_rank_neq() {
//         let parsed: SFilterExpr = parse_str(
//           "Rank != Ace"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::KeyNeqValue(
//             th::key("Rank"),
//             th::value("Ace"),
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_not_combo() {
//         let parsed: SFilterExpr = parse_str(
//           "not Pair"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::NotCombo(
//             th::combo("Pair")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_combo() {
//         let parsed: SFilterExpr = parse_str(
//           "Pair"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Combo(
//             th::combo("Pair")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_and() {
//         let parsed: SFilterExpr = parse_str(
//           "(Pair and Triple)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::And(
//             Box::new(FilterExpr::Combo(
//               th::combo("Pair")
//             )),
//             Box::new(FilterExpr::Combo(
//               th::combo("Triple")
//             ))
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_filter_expr_or() {
//         let parsed: SFilterExpr = parse_str(
//           "(Pair or Triple)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           FilterExpr::Or(
//             Box::new(FilterExpr::Combo(
//               th::combo("Pair")
//             )),
//             Box::new(FilterExpr::Combo(
//               th::combo("Triple")
//             ))
//           )
//         ));
//     }

//     // =======================================================================

//     // Group =================================================================
//     #[test]
//     fn parses_valid_group_location() {
//         let parsed: SGroup = parse_str(
//           "Hand"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::Location(
//             th::location("Hand")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_group_location_filter() {
//         let parsed: SGroup = parse_str(
//           "Hand where same Rank"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::LocationWhere(
//             th::location("Hand"),
//             FilterExpr::Same(th::key("Rank"))
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_group_location_collection() {
//         let parsed: SGroup = parse_str(
//           "(Hand, Stack)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::LocationCollection(
//             LocationCollection {
//               locations: vec![
//                 th::location("Hand"),
//                 th::location("Stack")
//               ]
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_group_location_collection_filter() {
//         let parsed: SGroup = parse_str(
//           "(Hand, Stack) where same Rank"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::LocationCollectionWhere(
//             LocationCollection {
//               locations: vec![
//                 th::location("Hand"),
//                 th::location("Stack")
//               ]
//             },
//             FilterExpr::Same(th::key("Rank"))
//           )
//         ));
//     }


//     #[test]
//     fn parses_valid_group_combo_in_location() {
//         let parsed: SGroup = parse_str(
//           "Pair in Hand"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::ComboInLocation(
//             th::combo("Pair"),
//             th::location("Hand")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_group_combo_in_location_collection() {
//         let parsed: SGroup = parse_str(
//           "Pair in (Hand, Stack)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::ComboInLocationCollection(
//             th::combo("Pair"),
//             LocationCollection {
//               locations: vec![
//                 th::location("Hand"),
//                 th::location("Stack")
//               ]
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_group_combo_not_in_location() {
//         let parsed: SGroup = parse_str(
//           "Pair not in Hand"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::NotComboInLocation(
//             th::combo("Pair"),
//             th::location("Hand")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_group_combo_not_in_location_collection() {
//         let parsed: SGroup = parse_str(
//           "Pair not in (Hand, Stack)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::NotComboInLocationCollection(
//             th::combo("Pair"),
//             LocationCollection {
//               locations: vec![
//                 th::location("Hand"),
//                 th::location("Stack")
//               ]
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_group_cardposition() {
//         let parsed: SGroup = parse_str(
//           "top(Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           Group::CardPosition(
//             CardPosition::Top(th::location("Hand"))
//           )
//         ));
//     }

//     // =======================================================================

//     // CardSet ===============================================================

//     #[test]
//     fn parses_valid_cardset_group() {
//         let parsed: SCardSet = parse_str(
//           "top(Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           CardSet::Group(
//             Group::CardPosition(
//               CardPosition::Top(th::location("Hand"))
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_cardset_group_of_player() {
//         let parsed: SCardSet = parse_str(
//           "Hand where same Rank of current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           CardSet::GroupOfPlayer(
//             Group::LocationWhere(
//               th::location("Hand"),
//               FilterExpr::Same(th::key("Rank"))
//             ),
//             th::CURRENT
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_cardset_group_of_player_collection() {
//         let parsed: SCardSet = parse_str(
//           "Hand where same Rank of others"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           CardSet::GroupOfPlayerCollection(
//             Group::LocationWhere(
//               th::location("Hand"),
//               FilterExpr::Same(th::key("Rank"))
//             ),
//             PlayerCollection::Others
//           )
//         ));
//     }

//     // =======================================================================

//     // IntCollection =========================================================

//     #[test]
//     fn parses_valid_intcollection() {
//         let parsed: SIntCollection = parse_str(
//           "(1, 2, 3, 4, 5)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           IntCollection {
//             ints: vec![
//               IntExpr::Int(1),
//               IntExpr::Int(2),
//               IntExpr::Int(3),
//               IntExpr::Int(4),
//               IntExpr::Int(5),
//             ]
//           }
//         ));
//     }

//     // =======================================================================

//     // LocationCollection ====================================================

//     #[test]
//     fn parses_valid_locationcollection() {
//         let parsed: SLocationCollection = parse_str(
//           "(Hand, Deck, Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           LocationCollection {
//             locations: vec![
//               th::location("Hand"),
//               th::location("Deck"),
//               th::location("Hand"),
//             ]
//           }
//         ));
//     }

//     // =======================================================================

//     // TeamCollection ========================================================

//     #[test]
//     fn parses_valid_teamcollection_other_teams() {
//         let parsed: STeamCollection = parse_str(
//           "other teams"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok( 
//           TeamCollection::OtherTeams
//         ));
//     }

//     #[test]
//     fn parses_valid_teamcollection_teams() {
//         let parsed: STeamCollection = parse_str(
//           "(T1, T2)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TeamCollection::Team(
//             vec![
//               TeamExpr::TeamName(th::teamname("T1")),
//               TeamExpr::TeamName(th::teamname("T2")),
//             ]
//           )
//         ));
//     }

//     // =======================================================================

//     // StringCollection ======================================================

//     // #[test]
//     // fn parses_valid_stringcollection() {
//     //     let parsed: SStringCollection = parse_str(
//     //       "(A, B)"
//     //     ).unwrap();
//     //     assert_eq!(parsed.lower(&ctx()), Ok(
//     //       StringCollection {
//     //         strings: vec![
//     //           StringExpr::ID(th::id("A")),
//     //           StringExpr::ID(th::id("B")),
//     //         ]
//     //       }
//     //     ));
//     // }

//     // =======================================================================

//     // Collection ============================================================

//     #[test]
//     fn parses_valid_collection_playercollection() {
//         let parsed: SCollection = parse_str(
//           "(current, previous)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Collection::PlayerCollection(
//             PlayerCollection::Player(
//               vec![
//                 th::CURRENT,
//                 th::PREVIOUS,
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_collection_teamcollection() {
//         let parsed: SCollection = parse_str(
//           "(T1, team of current)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Collection::TeamCollection(
//             TeamCollection::Team(
//               vec![
//                 TeamExpr::TeamName(th::teamname("T1")),
//                 TeamExpr::TeamOf(PlayerExpr::Current),
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_collection_intcollection() {
//         let parsed: SCollection = parse_str(
//           "(1, 2, 3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Collection::IntCollection(
//             IntCollection {
//               ints: vec![
//                 IntExpr::Int(1),
//                 IntExpr::Int(2),
//                 IntExpr::Int(3),
//               ]
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_collection_cardset() {
//         let parsed: SCollection = parse_str(
//           "Hand of current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Collection::CardSet(
//             Box::new(CardSet::GroupOfPlayer(Group::Location(th::location("Hand")), th::CURRENT))
//           )
//         ));
//     }

//     // #[test]
//     // fn parses_valid_collection_cardset_ambiguous() {
//     //     let parsed: SCollection = parse_str(
//     //       "(Hand, Deck, Hand)"
//     //     ).unwrap();
//     //     assert_eq!(parsed.lower(&ctx()), Ok(
//     //       Collection::CardSet(
//     //         vec![
//     //           th::id("Hand"),
//     //           th::id("Deck"),
//     //           th::id("Hand"),
//     //         ]
//     //       )
//     //     ));
//     // }

//     #[test]
//     fn parses_valid_collection_stringcollection() {
//         let parsed: SCollection = parse_str(
//           "(Rank of top(Hand), Rank of top(Hand), Rank of top(Hand))"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Collection::StringCollection(
//             StringCollection {
//             strings: vec![
//               StringExpr::KeyOf(th::key("Rank"), CardPosition::Top(th::location("Hand"))),
//               StringExpr::KeyOf(th::key("Rank"), CardPosition::Top(th::location("Hand"))),
//               StringExpr::KeyOf(th::key("Rank"), CardPosition::Top(th::location("Hand"))),
//             ]
//           }
//           )
//         ));
//     }

//     // =======================================================================

//     // Repititions ===========================================================

//     #[test]
//     fn parses_valid_repititions() {
//         let parsed: SRepititions = parse_str(
//           "3 times"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Repititions {
//             times: IntExpr::Int(3)
//           }
//         ));
//     }

//     // =======================================================================

//     // EndCondition ==========================================================

//     #[test]
//     fn parses_valid_endcondition_until_end() {
//         let parsed: SEndCondition = parse_str(
//           "until(end)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           EndCondition::UntilEnd
//         ));
//     }

//     #[test]
//     fn parses_valid_endcondition_until_reps() {
//         let parsed: SEndCondition = parse_str(
//           "until(3 times)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           EndCondition::UntilRep(
//             Repititions {
//               times: IntExpr::Int(3)
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_endcondition_until_bool() {
//         let parsed: SEndCondition = parse_str(
//           "until(3 == 2)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           EndCondition::UntilBool(
//             BoolExpr::IntCmp(IntExpr::Int(3), IntCmpOp::Eq, IntExpr::Int(2))
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_endcondition_until_bool_and_rep() {
//         let parsed: SEndCondition = parse_str(
//           "until(3 == 2 and 3 times)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           EndCondition::UntilBoolAndRep(
//             BoolExpr::IntCmp(IntExpr::Int(3), IntCmpOp::Eq, IntExpr::Int(2)),
//             Repititions {
//               times: IntExpr::Int(3)
//             }
//           )
//         ));
//     }
    
//     #[test]
//     fn parses_valid_endcondition_until_bool_or_rep() {
//         let parsed: SEndCondition = parse_str(
//           "until(3 == 2 or 3 times)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           EndCondition::UntilBoolOrRep(
//             BoolExpr::IntCmp(IntExpr::Int(3), IntCmpOp::Eq, IntExpr::Int(2)),
//             Repititions {
//               times: IntExpr::Int(3)
//             }
//           )
//         ));
//     }
    
//     // =======================================================================

//     // IntRange ==============================================================

//     #[test]
//     fn parses_valid_endcondition_intrange_eq() {
//         let parsed: SIntRange = parse_str(
//           "== 2"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           IntRange {
//             op: IntCmpOp::Eq,
//             int: IntExpr::Int(2)
//           }
//         ));
//     }

//     #[test]
//     fn parses_valid_endcondition_intrange_neq() {
//         let parsed: SIntRange = parse_str(
//           "!= 2"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           IntRange {
//             op: IntCmpOp::Neq,
//             int: IntExpr::Int(2)
//           }
//         ));
//     }
    
//     #[test]
//     fn parses_valid_endcondition_intrange_ge() {
//         let parsed: SIntRange = parse_str(
//           ">= 2"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           IntRange {
//             op: IntCmpOp::Ge,
//             int: IntExpr::Int(2)
//           }
//         ));
//     }
    
//     #[test]
//     fn parses_valid_endcondition_intrange_le() {
//         let parsed: SIntRange = parse_str(
//           "<= 2"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           IntRange {
//             op: IntCmpOp::Le,
//             int: IntExpr::Int(2)
//           }
//         ));
//     }
    
//     #[test]
//     fn parses_valid_endcondition_intrange_gt() {
//         let parsed: SIntRange = parse_str(
//           "> 2"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           IntRange {
//             op: IntCmpOp::Gt,
//             int: IntExpr::Int(2)
//           }
//         ));
//     }
    
//     #[test]
//     fn parses_valid_endcondition_intrange_lt() {
//         let parsed: SIntRange = parse_str(
//           "< 2"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           IntRange {
//             op: IntCmpOp::Lt,
//             int: IntExpr::Int(2)
//           }
//         ));
//     }

//     // =======================================================================

//     // Quantity ==============================================================

//     #[test]
//     fn parses_valid_quantity_int() {
//         let parsed: SQuantity = parse_str(
//           "3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Quantity::Int(
//             IntExpr::Int(3)
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_quantity_intrange() {
//         let parsed: SQuantity = parse_str(
//           "== 3"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Quantity::IntRange(
//             IntRange {
//               op: IntCmpOp::Eq,
//               int: IntExpr::Int(3)
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_quantity_quantifier() {
//         let parsed: SQuantity = parse_str(
//           "all"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Quantity::Quantifier(
//             Quantifier::All
//           )
//         ));
//     }
    
//     // =======================================================================

//     // ClassicMove ===========================================================

//     #[test]
//     fn parses_valid_classicmove_move() {
//         let parsed: SClassicMove = parse_str(
//           "move Hand private to Deck"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           ClassicMove::Move(
//             CardSet::Group(Group::Location(th::location("Hand"))),
//             Status::Private,
//             CardSet::Group(Group::Location(th::location("Deck")))
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_classicmove_move_quantity() {
//         let parsed: SClassicMove = parse_str(
//           "move all from Hand private to Deck"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           ClassicMove::MoveQuantity(
//             Quantity::Quantifier(Quantifier::All),
//             CardSet::Group(Group::Location(th::location("Hand"))),
//             Status::Private,
//             CardSet::Group(Group::Location(th::location("Deck")))
//           )
//         ));
//     }

//     // =======================================================================

//     // DealMove ===========================================================

//     #[test]
//     fn parses_valid_dealmove_deal() {
//         let parsed: SDealMove = parse_str(
//           "deal Hand private to Deck"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           DealMove::Deal(
//             CardSet::Group(Group::Location(th::location("Hand"))),
//             Status::Private,
//             CardSet::Group(Group::Location(th::location("Deck")))
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_dealmove_deal_quantity() {
//         let parsed: SDealMove = parse_str(
//           "deal 12 from Hand private to Deck of all"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           DealMove::DealQuantity(
//             Quantity::Int(IntExpr::Int(12)),
//             CardSet::Group(Group::Location(th::location("Hand"))),
//             Status::Private,
//             CardSet::GroupOfPlayerCollection(Group::Location(th::location("Deck")), PlayerCollection::Quantifier(Quantifier::All))
//           )
//         ));
//     }

//     // =======================================================================

//         // DealMove ===========================================================

//     #[test]
//     fn parses_valid_exchangemove_exchange() {
//         let parsed: SExchangeMove = parse_str(
//           "exchange Hand private with Deck"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           ExchangeMove::Exchange(
//             CardSet::Group(Group::Location(th::location("Hand"))),
//             Status::Private,
//             CardSet::Group(Group::Location(th::location("Deck")))
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_exchangemove_exchange_quantity() {
//         let parsed: SExchangeMove = parse_str(
//           "exchange all from Hand private with Deck"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           ExchangeMove::ExchangeQuantity(
//             Quantity::Quantifier(Quantifier::All),
//             CardSet::Group(Group::Location(th::location("Hand"))),
//             Status::Private,
//             CardSet::Group(Group::Location(th::location("Deck")))
//           )
//         ));
//     }

//     // =======================================================================

//     // TokenLocExpr ==========================================================

//     #[test]
//     fn parses_valid_tokenloc_expr_location() {
//         let parsed: STokenLocExpr = parse_str(
//           "Hand"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TokenLocExpr::Location(
//             th::location("Hand")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_tokenloc_expr_location_player() {
//         let parsed: STokenLocExpr = parse_str(
//           "Hand of current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TokenLocExpr::LocationPlayer(
//             th::location("Hand"),
//             th::CURRENT
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_tokenloc_expr_location_playercollection() {
//         let parsed: STokenLocExpr = parse_str(
//           "Hand of others"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TokenLocExpr::LocationPlayerCollection(
//             th::location("Hand"),
//             PlayerCollection::Others
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_tokenloc_expr_locationcollection() {
//         let parsed: STokenLocExpr = parse_str(
//           "(Hand, Deck)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TokenLocExpr::LocationCollection(
//             LocationCollection {
//               locations: vec![
//                 th::location("Hand"),
//                 th::location("Deck"),
//               ]
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_tokenloc_expr_locationcollection_player() {
//         let parsed: STokenLocExpr = parse_str(
//           "(Hand, Deck) of current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TokenLocExpr::LocationCollectionPlayer(
//             LocationCollection {
//               locations: vec![
//                 th::location("Hand"),
//                 th::location("Deck"),
//               ]
//             },
//             th::CURRENT
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_tokenloc_expr_locationcollection_playercollection() {
//         let parsed: STokenLocExpr = parse_str(
//           "(Hand, Deck) of others"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TokenLocExpr::LocationCollectionPlayerCollection(
//             LocationCollection {
//               locations: vec![
//                 th::location("Hand"),
//                 th::location("Deck"),
//               ]
//             },
//             PlayerCollection::Others
//           )
//         ));
//     }

//     // =======================================================================

//     // TokenMove =============================================================

//     #[test]
//     fn parses_valid_tokenmove_place() {
//         let parsed: STokenMove = parse_str(
//           "place Token Hand to Deck"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TokenMove::Place(
//             th::token("Token"),
//             TokenLocExpr::Location(
//               th::location("Hand")
//             ),
//             TokenLocExpr::Location(
//               th::location("Deck")
//             ),
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_tokenmove_place_quantity() {
//         let parsed: STokenMove = parse_str(
//           "place all Token from Hand to Deck"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           TokenMove::PlaceQuantity(
//             Quantity::Quantifier(Quantifier::All),
//             th::token("Token"),
//             TokenLocExpr::Location(
//               th::location("Hand")
//             ),
//             TokenLocExpr::Location(
//               th::location("Deck")
//             ),
//           )
//         ));
//     }

//     // =======================================================================
    
//     // Rule ==================================================================

//     #[test]
//     fn parses_valid_rule_createplayers() {
//         let parsed: SRule = parse_str(
//           "players: (P1, P2, P3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreatePlayer(
//             vec![
//               th::playername("P1"),
//               th::playername("P2"),
//               th::playername("P3"),
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_createteam() {
//         let parsed: SRule = parse_str(
//           "team T1: (P1, P2, P3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateTeam(
//             th::teamname("T1"),
//             vec![
//               th::playername("P1"),
//               th::playername("P2"),
//               th::playername("P3"),
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_createturnorder() {
//         let parsed: SRule = parse_str(
//           "turnorder: (P1, P2, P3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateTurnorder(
//             vec![
//               th::playername("P1"),
//               th::playername("P2"),
//               th::playername("P3"),
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_createturnorder_random() {
//         let parsed: SRule = parse_str(
//           "random turnorder: (P1, P2, P3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateTurnorderRandom(
//             vec![
//               th::playername("P1"),
//               th::playername("P2"),
//               th::playername("P3"),
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_createlocation_playercollection() {
//         let parsed: SRule = parse_str(
//           "location Hand on (P1, P2, P3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateLocationOnPlayerCollection(
//             th::location("Hand"),
//             PlayerCollection::Player(
//               vec![
//                 PlayerExpr::PlayerName(th::playername("P1"),),
//                 PlayerExpr::PlayerName(th::playername("P2"),),
//                 PlayerExpr::PlayerName(th::playername("P3"),),
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_createlocation_teamcollection() {
//         let parsed: SRule = parse_str(
//           "location Hand on teams(T1, T2, T3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateLocationOnTeamCollection(
//             th::location("Hand"),
//             TeamCollection::Team(
//               vec![
//                 TeamExpr::TeamName(th::teamname("T1")),
//                 TeamExpr::TeamName(th::teamname("T2")),
//                 TeamExpr::TeamName(th::teamname("T3")),
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_createlocation_table() {
//         let parsed: SRule = parse_str(
//           "location Stack on table"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateLocationOnTable(
//             th::location("Stack")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_createcard() {
//         let parsed: SRule = parse_str(
//           "card on Stack: 
//             Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
//               for Suite(Spades, Clubs)
//                 for Color(Black)
//           "
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateCardOnLocation(
//             th::location("Stack"),
//             Types {
//               types: vec![
//                 ( th::key("Rank"), vec![
//                   th::value("Two"),
//                   th::value("Three"),
//                   th::value("Four"),
//                   th::value("Five"),
//                   th::value("Six"),
//                   th::value("Seven"),
//                   th::value("Eight"),
//                   th::value("Nine"),
//                   th::value("Ten"),
//                   th::value("Jack"),
//                   th::value("Queen"),
//                   th::value("King"),
//                   th::value("Ace"),
//                 ]),
//                 (th::key("Suite"), vec![
//                   th::value("Spades"),
//                   th::value("Clubs"),
//                 ]),
//                 (th::key("Color"), vec![
//                   th::value("Black"),
//                 ]),
//               ]
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_createtoken() {
//         let parsed: SRule = parse_str(
//           "token 10 Chip on Stack"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateTokenOnLocation(
//             IntExpr::Int(10),
//             th::token("Chip"),
//             th::location("Stack")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_precedence() {
//         let parsed: SRule = parse_str(
//           "precedence Rank on Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreatePrecedence(
//             th::precedence("Rank"),
//             vec![ 
//               (th::key("Rank"), th::value("Two")),
//               (th::key("Rank"), th::value("Three")),
//               (th::key("Rank"), th::value("Four")),
//               (th::key("Rank"), th::value("Five")),
//               (th::key("Rank"), th::value("Six")),
//               (th::key("Rank"), th::value("Seven")),
//               (th::key("Rank"), th::value("Eight")),
//               (th::key("Rank"), th::value("Nine")),
//               (th::key("Rank"), th::value("Ten")),
//               (th::key("Rank"), th::value("Jack")),
//               (th::key("Rank"), th::value("Queen")),
//               (th::key("Rank"), th::value("King")),
//               (th::key("Rank"), th::value("Ace"))
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_precedence_pair() {
//         let parsed: SRule = parse_str(
//           "precedence Rank (Rank(Two), Suite(Spades), Color(Red))"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreatePrecedence(
//             th::precedence("Rank"),
//             vec![
//               (th::key("Rank"), th::value("Two")),
//               (th::key("Suite"), th::value("Spades")),
//               (th::key("Color"), th::value("Red")),
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_combo() {
//         let parsed: SRule = parse_str(
//           "combo SameSuite where same Suite"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateCombo(
//             th::combo("SameSuite"),
//             FilterExpr::Same(th::key("Suite"))
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_memory_playercollection() {
//         let parsed: SRule = parse_str(
//           "memory Square on (current, P2, P3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateMemoryPlayerCollection(
//             th::memory("Square"),
//             PlayerCollection::Player(
//               vec![
//                 th::CURRENT,
//                 PlayerExpr::PlayerName(th::playername("P2")),
//                 PlayerExpr::PlayerName(th::playername("P3")),
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_memory_table() {
//         let parsed: SRule = parse_str(
//           "memory Square on table"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateMemoryTable(
//             th::memory("Square")
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_memory_int_playercollection() {
//         let parsed: SRule = parse_str(
//           "memory Square 10 on (current, P2, P3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateMemoryIntPlayerCollection(
//             th::memory("Square"),
//             IntExpr::Int(10),
//             PlayerCollection::Player(
//               vec![
//                 th::CURRENT,
//                 PlayerExpr::PlayerName(th::playername("P2")),
//                 PlayerExpr::PlayerName(th::playername("P3")),
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_memory_int_table() {
//         let parsed: SRule = parse_str(
//           "memory Square 10 on table"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateMemoryIntTable(
//             th::memory("Square"),
//             IntExpr::Int(10),
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_memory_string_playercollection() {
//         let parsed: SRule = parse_str(
//           "memory Square Rank of top(Hand) on (current, P2, P3)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateMemoryStringPlayerCollection(
//             th::memory("Square"),
//             StringExpr::KeyOf(th::key("Rank"), CardPosition::Top(th::location("Hand"))),
//             PlayerCollection::Player(
//               vec![
//                 th::CURRENT,
//                 PlayerExpr::PlayerName(th::playername("P2")),
//                 PlayerExpr::PlayerName(th::playername("P3")),
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_memory_string_table() {
//         let parsed: SRule = parse_str(
//           "memory Square Rank of top(Hand) on table"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreateMemoryStringTable(
//             th::memory("Square"),
//             StringExpr::KeyOf(th::key("Rank"), CardPosition::Top(th::location("Hand"))),
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_pointmap() {
//         let parsed: SRule = parse_str(
//           "pointmap Rank on Rank(
//             Two: 1,
//             Three: 1,
//             Four: 1,
//             Five: 1,
//             Six: 1,
//             Seven: 1,
//             Eight: 1,
//             Nine: 1,
//             Ten: 1,
//             Jack: 1,
//             Queen: 1,
//             King: 1,
//             Ace: 1
//           )"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreatePointMap(
//             th::pointmap("Rank"),
//             vec![
//               (th::key("Rank"), th::value("Two"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Three"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Four"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Five"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Six"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Seven"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Eight"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Nine"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Ten"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Jack"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Queen"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("King"), IntExpr::Int(1)),
//               (th::key("Rank"), th::value("Ace"), IntExpr::Int(1)),
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_create_pointmap_pairs() {
//         let parsed: SRule = parse_str(
//           "pointmap Rank (Rank(Two: 1), Suite(Spades: 1), Color(Red: 1))"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CreatePointMap(
//             th::pointmap("Rank"),
//             vec![
//               (th::key("Rank"), th::value("Two"), IntExpr::Int(1)),
//               (th::key("Suite"), th::value("Spades"), IntExpr::Int(1)),
//               (th::key("Color"), th::value("Red"), IntExpr::Int(1)),
//             ]
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_flip_action() {
//         let parsed: SRule = parse_str(
//           "flip Hand to private"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::FlipAction(
//             CardSet::Group(Group::Location(th::location("Hand"))),
//             Status::Private
//           )
//         ));
//     }
    
//     #[test]
//     fn parses_valid_rule_shuffle_action() {
//         let parsed: SRule = parse_str(
//           "shuffle Hand"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::ShuffleAction(
//             CardSet::Group(Group::Location(th::location("Hand"))),
//           )
//         ));
//     }
    
//     #[test]
//     fn parses_valid_rule_player_out_stage() {
//         let parsed: SRule = parse_str(
//           "set current out of stage"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::PlayerOutOfStageAction(
//             th::CURRENT
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_player_out_game_succ() {
//         let parsed: SRule = parse_str(
//           "set current out of game successful"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::PlayerOutOfGameSuccAction(
//             th::CURRENT
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_player_out_game_fail() {
//         let parsed: SRule = parse_str(
//           "set current out of game fail"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::PlayerOutOfGameFailAction(
//             th::CURRENT
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_player_collection_out_stage() {
//         let parsed: SRule = parse_str(
//           "set (current) out of stage"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::PlayerCollectionOutOfStageAction(
//             PlayerCollection::Player(
//               vec![
//                 th::CURRENT
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_player_collection_out_game_succ() {
//         let parsed: SRule = parse_str(
//           "set (current) out of game successful"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::PlayerCollectionOutOfGameSuccAction(
//             PlayerCollection::Player(
//               vec![
//                 th::CURRENT
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_player_collection_out_game_fail() {
//         let parsed: SRule = parse_str(
//           "set (current) out of game fail"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::PlayerCollectionOutOfGameFailAction(
//             PlayerCollection::Player(
//               vec![
//                 th::CURRENT
//               ]
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_set_memory_int() {
//         let parsed: SRule = parse_str(
//           "Square is 10"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::SetMemoryInt(
//             th::memory("Square"),
//             IntExpr::Int(10)
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_set_memory_string() {
//         let parsed: SRule = parse_str(
//           "Square is Rank of top(Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::SetMemoryString(
//             th::memory("Square"),
//             StringExpr::KeyOf(
//               th::key("Rank"),
//               CardPosition::Top(th::location("Hand"))
//             )
//           )
//         ));
//     }

//     // #[test]
//     // fn parses_valid_rule_set_memory_ambiguous() {
//     //     let parsed: SRule = parse_str(
//     //       "Square is A"
//     //     ).unwrap();
//     //     assert_eq!(parsed.lower(&ctx()), Ok(
//     //       Rule::SetMemoryAmbiguous(
//     //         th::memory("Square"),
//     //         th::id("A")
//     //       )
//     //     ));
//     // }

//     #[test]
//     fn parses_valid_rule_set_memory_collection() {
//         let parsed: SRule = parse_str(
//           "Square is (current)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::SetMemoryCollection(
//             th::memory("Square"),
//             Collection::PlayerCollection(
//               PlayerCollection::Player(
//                 vec![
//                   th::CURRENT
//                 ]
//               )
//             )
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_cycle_action() {
//         let parsed: SRule = parse_str(
//           "cycle to next"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::CycleAction(
//             PlayerExpr::Next
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_bid_action() {
//         let parsed: SRule = parse_str(
//           "bid all"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::BidAction(
//             Quantity::Quantifier(Quantifier::All)
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_bid_action_memory() {
//         let parsed: SRule = parse_str(
//           "bid all on Square"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::BidActionMemory(
//             th::memory("Square"),
//             Quantity::Quantifier(Quantifier::All)
//           )
//         ));
//     }
    
//     #[test]
//     fn parses_valid_rule_end_turn() {
//         let parsed: SRule = parse_str(
//           "end turn"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::EndTurn
//         ));
//     }
    
//     #[test]
//     fn parses_valid_rule_end_stage() {
//         let parsed: SRule = parse_str(
//           "end stage"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::EndStage
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_end_game_with_winner() {
//         let parsed: SRule = parse_str(
//           "end game with winner current"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::EndGameWithWinner(
//             th::CURRENT
//           )
//         ));
//     }
    
//     #[test]
//     fn parses_valid_rule_demand_card_position() {
//         let parsed: SRule = parse_str(
//           "demand top(Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::DemandCardPositionAction(
//             CardPosition::Top(
//               th::location("Hand")
//             )
//           )
//         ));
//     }
    
//     #[test]
//     fn parses_valid_rule_demand_string() {
//         let parsed: SRule = parse_str(
//           "demand Rank of top(Hand)"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::DemandStringAction(
//             StringExpr::KeyOf(th::key("Rank"), CardPosition::Top(th::location("Hand"))),
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_rule_demand_int() {
//         let parsed: SRule = parse_str(
//           "demand 10"
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Rule::DemandIntAction(
//             IntExpr::Int(10)
//           )
//         ));
//     }

//     // =======================================================================

//     // SeqStage ==============================================================

//     #[test]
//     fn parses_valid_seq_stage() {
//         let parsed: SSeqStage = parse_str(
//           "
//             stage Play for current until(1 times) {
//               deal 12 from Stock private to Hand of all;
//             }
//           "
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           SeqStage {
//             stage: th::stage("Play"), 
//             player: th::CURRENT, 
//             end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
//             flows: vec![
//               FlowComponent::Rule(
//                 Rule::DealMove(
//                   DealMove::DealQuantity(
//                     Quantity::Int(IntExpr::Int(12)), 
//                     CardSet::Group(Group::Location(th::location("Stock"))), 
//                     Status::Private, 
//                     CardSet::GroupOfPlayerCollection(Group::Location(th::location("Hand")), PlayerCollection::Quantifier(Quantifier::All))
//                   )
//                 )
//               )
//             ] 
//           }
//         ));
//     }

//     // =======================================================================

//     // IfRule ================================================================

//     #[test]
//     fn parses_valid_if_rule() {
//         let parsed: SIfRule = parse_str(
//           "
//             if (current out of stage) {
//               cycle to next;
//             }
//           "
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           IfRule {
//             condition: BoolExpr::OutOfStagePlayer(th::CURRENT),
//             flows: vec![
//               FlowComponent::Rule(
//                 Rule::CycleAction(
//                   PlayerExpr::Next
//                 )
//               )
//             ]
//           }
//         ));
//     }

//     // =======================================================================

//     // OptionalRule ==========================================================

//     #[test]
//     fn parses_valid_optional_rule() {
//         let parsed: SOptionalRule = parse_str(
//           "
//             optional {
//               end turn;
//             }
//           "
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           OptionalRule {
//             flows: vec![
//               FlowComponent::Rule(
//                 Rule::EndTurn
//               )
//             ]
//           }
//         ));
//     }

//     // =======================================================================

//     // ChoiceRule ============================================================

//     #[test]
//     fn parses_valid_choice_rule() {
//         let parsed: SChoiceRule = parse_str(
//           "
//             choose {
//               end turn;
//               or
//               optional {
//                 end stage;
//               } 
//             }
//           "
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           ChoiceRule {
//             options: vec![
//               FlowComponent::Rule(
//                 Rule::EndTurn
//               ),
//               FlowComponent::OptionalRule(
//                 OptionalRule {
//                   flows: vec![
//                       FlowComponent::Rule(
//                         Rule::EndStage
//                       )
//                   ]
//                 }
//               ),
//             ]
//           }
//         ));
//     }

//     // =======================================================================

//     // FlowComponent =========================================================

//     #[test]
//     fn parses_valid_flow_component_choice_rule() {
//         let parsed: SFlowComponent = parse_str(
//           "
//             choose {
//               end turn;
//               or
//               optional {
//                 end stage;
//               } 
//             }
//           "
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           FlowComponent::ChoiceRule(
//             ChoiceRule {
//               options: vec![
//                 FlowComponent::Rule(
//                   Rule::EndTurn
//                 ),
//                 FlowComponent::OptionalRule(
//                   OptionalRule {
//                     flows: vec![
//                         FlowComponent::Rule(
//                           Rule::EndStage
//                         )
//                     ]
//                   }
//                 ),
//               ]
//             }
//           )
//         ));
//     }

//     #[test]
//     fn parses_valid_flow_component_rule() {
//         let parsed: SFlowComponent = parse_str(
//           "
//             end turn;
//           "
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           FlowComponent::Rule(
//             Rule::EndTurn
//           )
//         ));
//     }

//     // =======================================================================

//     // Game ==================================================================

//     #[test]
//     fn parses_valid_game() {
//         let parsed: SGame = parse_str(
//           "
//             players: (P1, P2, P3);
//             turnorder: (P1, P2, P3);
//             location (Hand, LayDown, Trash) on all;
//             location (Stock, Discard) on table;
//             card on Stock:
//               Rank(Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King, Ace)
//                 for Suite(Diamonds, Hearts, Spades, Clubs);
//             precedence RankOrder on Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine , Ten, Jack, Queen, King);
//             pointmap Values on Rank(Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6, Seven: 7, Eight: 8, Nine: 9 , Ten: 10, Jack: 10, Queen: 10, King: 10);
//             combo Sequence where ((size >= 3 and same Suite) and adjacent Rank using RankOrder);
//             combo Set where ((size >= 3 and distinct Suite) and same Rank);
//             combo Deadwood where (not Sequence and not Set);

//             stage Preparation for current until(1 times) {
//               deal 12 from top(Stock) private to Hand of all;
//             }

//             stage Collect for current until(previous out of stage) {
//               choose {
//                 move top(Discard) private to Hand;
//                 or
//                 move top(Stock) private to Hand;
//               }

//               move any from Hand face up to top(Discard);

//               if (sum of Deadwood in Hand using Values <= 10) {
//                 optional {
//                   move all from Set in Hand face up to top(LayDown);
//                   move all from Sequence in Hand face up to top(LayDown);

//                   if (Hand is empty) {
//                     move all from Set in Hand of next face up to top(LayDown) of next;
//                     move all from Sequence in Hand of next face up to top(LayDown) of next;
//                     move Hand of next face up to Trash of next;

//                     move Hand face up to Trash;
//                     set current out of stage;
//                   }
//                 }
//               }

//               cycle to next;
//             }

//             stage FinalLayDown for current until(1 times) {
//               move LayDown of previous face up to Hand of current;
//               move all from Set in Hand face up to top(LayDown);
//               move all from Sequence in Hand face up to top(LayDown);

//               move Hand face up to Trash;
//             }

//             score sum of Trash using Values to LeftOver of all;
//             winner is lowest LeftOver;
//           "
//         ).unwrap();
//         assert_eq!(parsed.lower(&ctx()), Ok(
//           Game {
//             flows: vec![
//               // create players
//               FlowComponent::Rule(
//                 Rule::CreatePlayer(
//                   vec![
//                     th::playername("P1"),
//                     th::playername("P2"),
//                     th::playername("P3"),
//                   ]
//                 )
//               ),
//               // create turnorder
//               FlowComponent::Rule(
//                 Rule::CreateTurnorder(
//                   vec![
//                     th::playername("P1"),
//                     th::playername("P2"),
//                     th::playername("P3"),
//                   ]
//                 )
//               ),
//               // location on all
//               FlowComponent::Rule(
//                 Rule::CreateLocationCollectionOnPlayerCollection(
//                   LocationCollection {
//                     locations: vec![
//                       th::location("Hand"),
//                       th::location("LayDown"),
//                       th::location("Trash"),
//                     ]
//                   },
//                   PlayerCollection::Quantifier(Quantifier::All)
//                 )
//               ),
//               // location on table
//               FlowComponent::Rule(
//                 Rule::CreateLocationCollectionOnTable(
//                   LocationCollection {
//                     locations: vec![
//                       th::location("Stock"),
//                       th::location("Discard"),
//                     ]
//                   }
//                 )
//               ),
//               // card on
//               FlowComponent::Rule(
//                 Rule::CreateCardOnLocation(
//                   th::location("Stock"),
//                   Types {
//                     types: vec![
//                       (th::key("Rank"), vec![
//                         th::value("Two"),
//                         th::value("Three"),
//                         th::value("Four"),
//                         th::value("Five"),
//                         th::value("Six"),
//                         th::value("Seven"),
//                         th::value("Eight"),
//                         th::value("Nine"),
//                         th::value("Ten"),
//                         th::value("Jack"),
//                         th::value("Queen"),
//                         th::value("King"),
//                         th::value("Ace")
//                       ]),
//                       (th::key("Suite"), vec![
//                         th::value("Diamonds"),
//                         th::value("Hearts"),
//                         th::value("Spades"),
//                         th::value("Clubs"),
//                       ]),
//                     ]
//                   }
//                 )
//               ),
//               // RankOrder
//               FlowComponent::Rule(
//                 Rule::CreatePrecedence(
//                   th::precedence("RankOrder"),
//                   vec![
//                     (th::key("Rank"), th::value("Ace")),
//                     (th::key("Rank"), th::value("Two")),
//                     (th::key("Rank"), th::value("Three")),
//                     (th::key("Rank"), th::value("Four")),
//                     (th::key("Rank"), th::value("Five")),
//                     (th::key("Rank"), th::value("Six")),
//                     (th::key("Rank"), th::value("Seven")),
//                     (th::key("Rank"), th::value("Eight")),
//                     (th::key("Rank"), th::value("Nine")),
//                     (th::key("Rank"), th::value("Ten")),
//                     (th::key("Rank"), th::value("Jack")),
//                     (th::key("Rank"), th::value("Queen")),
//                     (th::key("Rank"), th::value("King")),
//                   ]
//                 )
//               ),
//               // Values
//               FlowComponent::Rule(
//                 Rule::CreatePointMap(
//                   th::pointmap("Values"),
//                   vec![
//                     (th::key("Rank"), th::value("Ace"), IntExpr::Int(1)),
//                     (th::key("Rank"), th::value("Two"), IntExpr::Int(2)),
//                     (th::key("Rank"), th::value("Three"), IntExpr::Int(3)),
//                     (th::key("Rank"), th::value("Four"), IntExpr::Int(4)),
//                     (th::key("Rank"), th::value("Five"), IntExpr::Int(5)),
//                     (th::key("Rank"), th::value("Six"), IntExpr::Int(6)),
//                     (th::key("Rank"), th::value("Seven"), IntExpr::Int(7)),
//                     (th::key("Rank"), th::value("Eight"), IntExpr::Int(8)),
//                     (th::key("Rank"), th::value("Nine"), IntExpr::Int(9)),
//                     (th::key("Rank"), th::value("Ten"), IntExpr::Int(10)),
//                     (th::key("Rank"), th::value("Jack"), IntExpr::Int(10)),
//                     (th::key("Rank"), th::value("Queen"), IntExpr::Int(10)),
//                     (th::key("Rank"), th::value("King"), IntExpr::Int(10)),
//                   ]
//                 )
//               ),
//               // Combo Sequence
//               FlowComponent::Rule(
//                 Rule::CreateCombo(
//                   th::combo("Sequence"),
//                   FilterExpr::And(
//                     Box::new(FilterExpr::And(
//                       Box::new(FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))),
//                       Box::new(FilterExpr::Same(th::key("Suite")))
//                     )),
//                     Box::new(FilterExpr::Adjacent(th::key("Rank"), th::precedence("RankOrder")))
//                   )
//                 )
//               ),
//               // Combo Set
//               FlowComponent::Rule(
//                 Rule::CreateCombo(
//                   th::combo("Set"),
//                   FilterExpr::And(
//                     Box::new(FilterExpr::And(
//                       Box::new(FilterExpr::Size(IntCmpOp::Ge, Box::new(IntExpr::Int(3)))),
//                       Box::new(FilterExpr::Distinct(th::key("Suite")))
//                     )),
//                     Box::new(FilterExpr::Same(th::key("Rank")))
//                   )
//                 )
//               ),
//               // Combo Set
//               FlowComponent::Rule(
//                 Rule::CreateCombo(
//                   th::combo("Deadwood"),
//                   FilterExpr::And(
//                     Box::new(
//                       FilterExpr::NotCombo(th::combo("Sequence"))
//                     ),
//                     Box::new(
//                       FilterExpr::NotCombo(th::combo("Set"))
//                     )
//                   )
//                 )
//               ),
//               // Stage Preparation
//               FlowComponent::Stage(
//                 SeqStage {
//                   stage: th::stage("Preparation"), 
//                   player: th::CURRENT, 
//                   end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
//                   flows: vec![
//                     FlowComponent::Rule(
//                       Rule::DealMove(
//                         DealMove::DealQuantity(
//                           Quantity::Int(IntExpr::Int(12)), 
//                           CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Stock")))), 
//                           Status::Private, 
//                           CardSet::GroupOfPlayerCollection(Group::Location(th::location("Hand")), PlayerCollection::Quantifier(Quantifier::All))
//                         )
//                       )
//                     )
//                   ] 
//                 }
//               ),
//               // Stage Collect
//               FlowComponent::Stage(
//                 SeqStage {
//                   stage: th::stage("Collect"), 
//                   player: th::CURRENT, 
//                   end_condition: EndCondition::UntilBool(BoolExpr::OutOfStagePlayer(th::PREVIOUS)), 
//                   flows: vec![
//                     // Choose
//                     FlowComponent::ChoiceRule(
//                       ChoiceRule {
//                         options: vec![
//                           // move top of Discard to Hand
//                           FlowComponent::Rule(
//                             Rule::ClassicMove(
//                               ClassicMove::Move(
//                                 CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Discard")))),
//                                 Status::Private,
//                                 CardSet::Group(Group::Location(th::location("Hand")))
//                               )
//                             )
//                           ),
//                           // move top of Stock to Hand
//                           FlowComponent::Rule(
//                             Rule::ClassicMove(
//                               ClassicMove::Move(
//                                 CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Stock")))),
//                                 Status::Private,
//                                 CardSet::Group(Group::Location(th::location("Hand")))
//                               )
//                             )
//                           ),
//                         ]
//                       }
//                     ),
//                     FlowComponent::Rule(
//                       Rule::ClassicMove(
//                         ClassicMove::MoveQuantity(
//                           Quantity::Quantifier(Quantifier::Any),
//                           CardSet::Group(Group::Location(th::location("Hand"))),
//                           Status::FaceUp,
//                           CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("Discard")))),
//                         )
//                       )
//                     ),
//                     FlowComponent::IfRule(
//                       IfRule { 
//                         condition: BoolExpr::IntCmp(
//                           IntExpr::SumOfCardSet(
//                             Box::new(
//                               CardSet::Group(
//                                 Group::ComboInLocation(
//                                   th::combo("Deadwood"),
//                                   th::location("Hand")
//                                 )
//                               )
//                             ), 
//                             th::pointmap("Values")
//                           ), 
//                           IntCmpOp::Le, 
//                           IntExpr::Int(10)
//                         ),
//                         flows: vec![
//                           FlowComponent::OptionalRule(
//                             OptionalRule { 
//                               flows: vec![
//                                 FlowComponent::Rule(
//                                   Rule::ClassicMove(
//                                     ClassicMove::MoveQuantity(
//                                       Quantity::Quantifier(Quantifier::All),
//                                       CardSet::Group(Group::ComboInLocation(th::combo("Set"), th::location("Hand"))),
//                                       Status::FaceUp,
//                                       CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("LayDown")))),
//                                     )
//                                   )
//                                 ),
//                                 FlowComponent::Rule(
//                                   Rule::ClassicMove(
//                                     ClassicMove::MoveQuantity(
//                                       Quantity::Quantifier(Quantifier::All),
//                                       CardSet::Group(Group::ComboInLocation(th::combo("Sequence"), th::location("Hand"))),
//                                       Status::FaceUp,
//                                       CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("LayDown")))),
//                                     )
//                                   )
//                                 ),
//                                 // If rule
//                                 FlowComponent::IfRule(
//                                   IfRule {
//                                     condition: BoolExpr::CardSetIsEmpty(
//                                       CardSet::Group(
//                                         Group::Location(th::location("Hand"))
//                                       )
//                                     ),
//                                     flows: vec![
//                                       FlowComponent::Rule(
//                                         Rule::ClassicMove(
//                                           ClassicMove::MoveQuantity(
//                                             Quantity::Quantifier(Quantifier::All),
//                                             CardSet::GroupOfPlayer(Group::ComboInLocation(th::combo("Set"), th::location("Hand")), PlayerExpr::Next),
//                                             Status::FaceUp,
//                                             CardSet::GroupOfPlayer(Group::CardPosition(CardPosition::Top(th::location("LayDown"))), PlayerExpr::Next),
//                                           )
//                                         )
//                                       ),
//                                       FlowComponent::Rule(
//                                         Rule::ClassicMove(
//                                           ClassicMove::MoveQuantity(
//                                             Quantity::Quantifier(Quantifier::All),
//                                             CardSet::GroupOfPlayer(Group::ComboInLocation(th::combo("Sequence"), th::location("Hand")), PlayerExpr::Next),
//                                             Status::FaceUp,
//                                             CardSet::GroupOfPlayer(Group::CardPosition(CardPosition::Top(th::location("LayDown"))), PlayerExpr::Next),
//                                           )
//                                         )
//                                       ),
//                                       FlowComponent::Rule(
//                                         Rule::ClassicMove(
//                                           ClassicMove::Move(
//                                             CardSet::GroupOfPlayer(Group::Location(th::location("Hand")), PlayerExpr::Next),
//                                             Status::FaceUp,
//                                             CardSet::GroupOfPlayer(Group::Location(th::location("Trash")), PlayerExpr::Next),
//                                           )
//                                         )
//                                       ),
//                                       FlowComponent::Rule(
//                                         Rule::ClassicMove(
//                                           ClassicMove::Move(
//                                             CardSet::Group(Group::Location(th::location("Hand"))),
//                                             Status::FaceUp,
//                                             CardSet::Group(Group::Location(th::location("Trash"))),
//                                           )
//                                         )
//                                       ),
//                                       FlowComponent::Rule(
//                                         Rule::PlayerOutOfStageAction(
//                                           th::CURRENT
//                                         )
//                                       ),
//                                     ]
//                                   }
//                                 )
//                               ]
//                             }
//                           )
//                         ]
//                       }
//                     ),
//                     FlowComponent::Rule(
//                       Rule::CycleAction(PlayerExpr::Next)
//                     )
//                   ] 
//                 }
//               ),
//               // Stage Preparation
//               FlowComponent::Stage(
//                 SeqStage {
//                   stage: th::stage("FinalLayDown"), 
//                   player: th::CURRENT, 
//                   end_condition: EndCondition::UntilRep(Repititions { times: IntExpr::Int(1) }), 
//                   flows: vec![
//                     FlowComponent::Rule(
//                       Rule::ClassicMove(
//                         ClassicMove::Move(
//                           CardSet::GroupOfPlayer(Group::Location(th::location("LayDown")), th::PREVIOUS),
//                           Status::FaceUp,
//                           CardSet::GroupOfPlayer(Group::Location(th::location("Hand")), th::CURRENT),
//                         )
//                       )
//                     ),
//                     FlowComponent::Rule(
//                       Rule::ClassicMove(
//                         ClassicMove::MoveQuantity(
//                           Quantity::Quantifier(Quantifier::All),
//                           CardSet::Group(Group::ComboInLocation(th::combo("Set"), th::location("Hand"))),
//                           Status::FaceUp,
//                           CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("LayDown")))),
//                         )
//                       )
//                     ),
//                     FlowComponent::Rule(
//                       Rule::ClassicMove(
//                         ClassicMove::MoveQuantity(
//                           Quantity::Quantifier(Quantifier::All),
//                           CardSet::Group(Group::ComboInLocation(th::combo("Sequence"), th::location("Hand"))),
//                           Status::FaceUp,
//                           CardSet::Group(Group::CardPosition(CardPosition::Top(th::location("LayDown")))),
//                         )
//                       )
//                     ),
//                     FlowComponent::Rule(
//                       Rule::ClassicMove(
//                         ClassicMove::Move(
//                           CardSet::Group(Group::Location(th::location("Hand"))),
//                           Status::FaceUp,
//                           CardSet::Group(Group::Location(th::location("Trash"))),
//                         )
//                       )
//                     ),
//                   ] 
//                 }
//               ),
//               FlowComponent::Rule(
//                 Rule::ScoreRule(
//                   ScoreRule::ScorePlayerCollectionMemory(
//                     IntExpr::SumOfCardSet(
//                       Box::new(
//                         CardSet::Group(
//                           Group::Location(
//                             th::location("Trash")
//                           )
//                         )
//                       ),
//                       th::pointmap("Values")
//                     ),
//                     th::memory("LeftOver"),
//                     PlayerCollection::Quantifier(Quantifier::All),
//                   )
//                 )
//               ),
//               FlowComponent::Rule(
//                 Rule::WinnerRule(
//                   WinnerRule::WinnerLowestMemory(
//                     th::memory("LeftOver")
//                   )
//                 )
//               ),
//             ]
//           }
//         ));
//     }

//     // =======================================================================

// }