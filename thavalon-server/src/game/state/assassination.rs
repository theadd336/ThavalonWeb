use std::collections::HashSet;

use itertools::Itertools;

use super::prelude::*;

/// Phase for end-game assassination.
pub struct Assassination {}

/*
How assassination works now:
- Can assassinate any role but Lancelot and declared Arthur (or "no assassination target")
- They have to get the priority target and one other target (but order doesn't matter)
- If you move to assassinate, you can either:
    - Assassinate any 1 person, no discussion
    - Do normal endgame assassination
*/

impl GameState<Assassination> {
    pub fn handle_assassination(
        self,
        player: &str,
        target: PriorityTarget,
        players: HashSet<String>,
    ) -> ActionResult {
        if player == self.game.assassin {
            log::debug!(
                "{} assassinated {} as {:?}",
                player,
                players.iter().format(" and "),
                target
            );

            // All priority assassinations (so far) take the form of "X players are one of Y roles", so we model that as
            // `expected_targets` and `matches` methods on `PriorityTarget` to cut down on duplication.
            // The `None` priority target is special, because it matches no players.

            if players.len() != target.expected_targets() {
                return self.player_error(format!(
                    "You must assassinate {} players as {:?}",
                    target.expected_targets(),
                    target
                ));
            }

            let mut is_correct = true;
            if target == PriorityTarget::None {
                // If there are no assassination targets in the game, we'll have checked for that at the beginning
                is_correct = self.game.priority_target == PriorityTarget::None
            } else {
                for name in players.iter() {
                    if let Some(player) = self.game.players.by_name(name) {
                        if !target.matches(player) {
                            is_correct = false;
                        }
                    } else {
                        return self.player_error(format!("{} is not in the game", name));
                    }
                }    
            }

            let effects = vec![Effect::Broadcast(Message::AssassinationResult {
                players,
                target,
                correct: is_correct,
            })];

            if is_correct {
                log::debug!("Assassination was correct!");
                self.into_done(Team::Evil, effects)
            } else {
                log::debug!("Assassination was incorrect!");
                self.into_done(Team::Good, effects)
            }
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
