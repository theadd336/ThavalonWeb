use std::collections::HashSet;

use super::prelude::*;

/// Phase for waiting for a player to make a mission proposal.
pub struct Proposing {
    /// The player whose turn it is to propose
    proposer: String,
}

impl GameState<Proposing> {
    /// Respond to a proposal. If the proposal is invalid, or it's not the player's turn to propose, this returns
    /// an error reply. Otherwise, the proposal is sent to all players and the game moves into the [`Voting`] phase.
    /// On mission 1, we instead ask for two proposals before voting. Once force is active, we skip the voting phase
    /// and go straight to [`OnMission`].
    pub fn handle_proposal(mut self, player: &str, players: HashSet<String>) -> ActionResult {
        if player != self.phase.proposer {
            return self.player_error("It's not your proposal");
        }

        let mission = self.mission();
        let expected_size = self.game.spec.mission_size(mission);
        if players.len() != expected_size {
            return self.player_error(format!("Proposal must contain {} players", expected_size));
        }

        for player in players.iter() {
            if self.game.players.by_name(player).is_none() {
                return self.player_error(format!("{} is not in the game", player));
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
}

impl Proposing {
    /// Create a new `Proposing` phase given the player proposing.
    pub fn new(proposer: String) -> Proposing {
        Proposing { proposer }
    }
}

impl_phase!(Proposing);
