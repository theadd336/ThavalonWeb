use std::collections::HashSet;

use itertools::Itertools;

use super::prelude::*;

/// Phase for end-game assassination.
pub struct Assassination {}

impl GameState<Assassination> {
    pub fn handle_assassination(
        mut self,
        player: &str,
        target: PriorityTarget,
        players: HashSet<String>,
    ) -> ActionResult {
        if player == self.game.assassin {
            log::debug!("{} assassinated {} as {:?}", player, players.iter().format(" and "), target);

            todo!()
        } else {
            self.player_error("You are not the assassin")
        }
    }
}

impl<P: Phase> GameState<P> {
    pub fn move_to_assassinate(self, player: &str) -> ActionResult {
        if player == self.game.assassin {
            log::debug!("{} moved to assassinate", player);
            let effects = vec![Effect::Broadcast(Message::BeginAssassination {
                assassin: player.to_string(),
            })];
            let next_state = GameStateWrapper::Assassination(self.with_phase(Assassination {}));
            (next_state, effects)
        } else {
            self.player_error("You are not the assassin")
        }
    }
}

impl_phase!(Assassination);
