use crate::game::engine::MAX_RECENT_ACTIONS;
use crate::game::Game;

pub(crate) fn cap_logs(game: &mut Game) {
    if game.recent_actions.len() > MAX_RECENT_ACTIONS {
        let to_remove = game.recent_actions.len() - MAX_RECENT_ACTIONS;
        game.recent_actions.drain(0..to_remove);
    }
}
