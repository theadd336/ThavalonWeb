use std::collections::HashSet;

use super::prelude::*;

/// Phase for waiting for a player to make a mission proposal.
pub struct Proposing {
    /// The player whose turn it is to propose
    proposer: String,
    selected_players: HashSet<String>,
}

const NOT_PROPOSER_ERROR: &str = "It's not your proposal";

impl GameState<Proposing> {
    /// Respond to the proposer adding a player to their proposal. If the player performing the action
    /// is not the proposer, this sends them an error message. It validates that the added player is
    /// in the game, but does not check if they are already on the proposal or if the proposal is the
    /// correct size, since final validation is done when the proposal is submitted.
    pub fn handle_player_selected(mut self, player: &str, added_player: String) -> ActionResult {
        if player != self.phase.proposer {
            return self.player_error(NOT_PROPOSER_ERROR);
        }

        if let Some(error) = self.validate_player(player) {
            return self.player_error(error);
        }

        self.phase.selected_players.insert(added_player);
        let effects = vec![Effect::Broadcast(Message::ProposalUpdated {
            players: self.phase.selected_players.clone(),
        })];

        (GameStateWrapper::Proposing(self), effects)
    }

    /// Respond to the proposer removing a player from their proposal. If the player performing the action is not
    /// the proposer, this sends them an error message. It also validates that the removed player is in the game
    /// and was on the proposal.
    pub fn handle_player_unselected(
        mut self,
        player: &str,
        removed_player: String,
    ) -> ActionResult {
        if player != self.phase.proposer {
            return self.player_error(NOT_PROPOSER_ERROR);
        }

        if self.game.players.by_name(&removed_player).is_none() {
            return self.player_error(format!("{} is not in the game!", removed_player));
        }

        if self.phase.selected_players.remove(&removed_player) {
            let effects = vec![Effect::Broadcast(Message::ProposalUpdated {
                players: self.phase.selected_players.clone(),
            })];

            (GameStateWrapper::Proposing(self), effects)
        } else {
            self.player_error(format!("{} wasn't on the proposal!", removed_player))
        }
    }

    /// Respond to a proposal. If the proposal is invalid, or it's not the player's turn to propose, this returns
    /// an error reply. Otherwise, the proposal is sent to all players and the game moves into the [`Voting`] phase.
    /// On mission 1, we instead ask for two proposals before voting. Once force is active, we skip the voting phase
    /// and go straight to [`OnMission`].
    pub fn handle_proposal(mut self, player: &str, players: HashSet<String>) -> ActionResult {
        if player != self.phase.proposer {
            return self.player_error(NOT_PROPOSER_ERROR);
        }

        let mission = self.mission();
        let expected_size = self.game.spec.mission_size(mission);
        if players.len() != expected_size {
            return self.player_error(format!("Proposal must contain {} players", expected_size));
        }

        for player in players.iter() {
            if let Some(error) = self.validate_player(player.as_str()) {
                return self.player_error(error);
            }
        }

        let proposal = Proposal {
            proposer: player.to_string(),
            players: players.clone(),
        };
        log::debug!("Got {} for mission {}", proposal, mission);
        self.proposals.push(proposal);

        let mut effects = vec![Effect::Broadcast(Message::ProposalMade {
            proposer: player.to_string(),
            mission,
            players,
        })];

        if mission == 1 && self.proposals.len() == 1 {
            // On mission 1, we need the first 2 proposals before voting
            log::debug!("Getting a second proposal for mission 1");
            let next_proposer = self.game.next_proposer(player).to_string();
            self.into_proposing(next_proposer, effects)
        } else if self.spent_proposals() > self.game.spec.max_proposals {
            let proposal = self.proposals.last().unwrap();
            log::debug!("Sending {} with force for mission {}", proposal, mission);
            effects.push(Effect::Broadcast(Message::MissionGoing {
                mission,
                players: proposal.players.clone(),
            }));
            let next_phase = OnMission::new(self.proposals.len() - 1);
            (
                GameStateWrapper::OnMission(self.with_phase(next_phase)),
                effects,
            )
        } else {
            if mission == 1 {
                let first_proposal = &self.proposals[self.proposals.len() - 2];
                let second_proposal = self.proposals.last().unwrap();
                log::debug!(
                    "Voting on {} and {} for mission {}",
                    first_proposal,
                    second_proposal,
                    mission
                );
            } else {
                let proposal = self.proposals.last().unwrap();
                log::debug!("Voting on {} for mission {}", proposal, mission);
            }

            effects.push(Effect::Broadcast(Message::CommenceVoting));
            let next_state = self.with_phase(Voting::new());
            (GameStateWrapper::Voting(next_state), effects)
        }
    }

    /// Checks if `player` is allowed on this proposal, returning an error message if not.
    fn validate_player(&self, player_name: &str) -> Option<String> {
        match self.game.players.by_name(player_name) {
            Some(player) => {
                if player.role == Role::Arthur
                    && self.role_state.arthur.has_declared()
                    && self.mission() != 5
                {
                    Some(format!("Arthur cannot go until mission 5"))
                } else {
                    None
                }
            }
            None => Some(format!("{} is not in the game", player_name)),
        }
    }
}

impl Proposing {
    /// Create a new `Proposing` phase given the player proposing.
    pub fn new(proposer: String) -> Proposing {
        Proposing {
            proposer,
            selected_players: HashSet::new(),
        }
    }
}

impl_phase!(Proposing);
