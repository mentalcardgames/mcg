use super::*;

#[test]
fn idx_returns_choice_idx() {
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 3 }
        }
        .idx(),
        3
    );
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 }
        }
        .idx(),
        0
    );
}

#[test]
fn idx_returns_0_for_optional_accept() {
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::OptionalAccept
        }
        .idx(),
        0
    );
}

#[test]
fn idx_returns_1_for_optional_decline() {
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::OptionalDecline
        }
        .idx(),
        1
    );
}

#[test]
fn idx_returns_0_for_choose_player_and_cards() {
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::ChoosePlayer { idx: 2 }
        }
        .idx(),
        0
    );
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::ChooseCards {
                selected: vec![1, 2]
            }
        }
        .idx(),
        0
    );
}

#[test]
fn player_idx_returns_some_for_choose_player() {
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::ChoosePlayer { idx: 1 }
        }
        .player_idx(),
        Some(1)
    );
}

#[test]
fn player_idx_returns_none_for_other_variants() {
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 }
        }
        .player_idx(),
        None
    );
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::OptionalAccept
        }
        .player_idx(),
        None
    );
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::OptionalDecline
        }
        .player_idx(),
        None
    );
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::ChooseCards { selected: vec![] }
        }
        .player_idx(),
        None
    );
}

#[test]
fn card_selection_returns_some_for_choose_cards() {
    let input = Input {
        player_id: "P1".into(),
        kind: InputKind::ChooseCards {
            selected: vec![0, 2, 4],
        },
    };
    assert_eq!(input.card_selection(), Some(&[0, 2, 4][..]));
}

#[test]
fn card_selection_returns_none_for_other_variants() {
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 }
        }
        .card_selection(),
        None
    );
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::OptionalAccept
        }
        .card_selection(),
        None
    );
    assert_eq!(
        Input {
            player_id: "P1".into(),
            kind: InputKind::ChoosePlayer { idx: 0 }
        }
        .card_selection(),
        None
    );
}

#[test]
fn card_selection_returns_empty_slice_for_empty_selected() {
    let input = Input {
        player_id: "P1".into(),
        kind: InputKind::ChooseCards { selected: vec![] },
    };
    assert_eq!(input.card_selection(), Some(&[][..]));
}
