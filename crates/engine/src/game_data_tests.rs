use super::*;

#[test]
fn test_ensure_stage_entered_is_idempotent_and_sets_flags() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];

    assert_eq!(gd.get_current_stage(), None);
    assert!(gd.stage_stack.is_empty());

    gd.ensure_stage_entered("Play");
    assert_eq!(gd.get_current_stage(), Some("Play".to_string()));
    assert_eq!(gd.stage_stack.len(), 1);
    assert_eq!(gd.players[0].in_stage.get("Play"), Some(&true));
    assert_eq!(gd.players[1].in_stage.get("Play"), Some(&true));

    gd.ensure_stage_entered("Play");
    assert_eq!(
        gd.stage_stack.len(),
        1,
        "ensure_stage_entered must not push twice"
    );

    gd.ensure_stage_entered("Sub");
    assert_eq!(gd.stage_stack.len(), 2);
    assert_eq!(gd.get_current_stage(), Some("Sub".to_string()));
    assert_eq!(gd.players[0].in_stage.get("Sub"), Some(&true));
}
