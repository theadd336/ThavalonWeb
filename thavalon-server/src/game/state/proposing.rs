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
    pub fn handle_proposal(self, player: &str, proposal: HashSet<String>) -> ActionResult {
        if player != self.phase.proposer {
            return self.player_error("It's not your proposal");
        }

        let mission = self.mission();
        let expected_size = self.game.spec.mission_size(mission);
        if proposal.len() != expected_size {
            return self.player_error(format!("Proposal must contain {} players", expected_size));
        }

        for player in proposal.iter() {
            if self.game.players.by_name(player).is_none() {
                return self.player_error(format!("{} is not in the game", player));
            }
        }

        use itertools::Itertools;
        log::debug!(
            "{} proposed {} for mission {}",
            player,
            proposal.iter().format(", "),
            mission
        );

        let mut effects = vec![Effect::Broadcast(Message::ProposalMade {
            proposer: player.to_string(),
            mission,
            players: proposal.clone(),
        })];

        let spent_proposals = self.spent_proposals();

        let GameState {
            game,
            mut proposals,
            mission_results,
            ..
        } = self;
        proposals.push(Proposal {
            proposer: player.to_string(),
            players: proposal,
        });

        if mission == 1 && proposals.len() == 1 {
            // On mission 1, we need the first 2 proposals before voting
            log::debug!("Getting a second proposal for mission 1");
            let next_proposer = game.next_proposer(player).to_string();
            effects.push(Effect::Broadcast(Message::NextProposal {
                proposer: next_proposer.clone(),
                mission,
                proposals_made: spent_proposals,
                max_proposals: game.spec.max_proposals,
            }));

            let next_game = GameStateWrapper::Proposing(GameState {
                game,
                proposals,
                mission_results,
                phase: Proposing::new(next_proposer),
            });
            (next_game, effects)
        } else if spent_proposals > game.spec.max_proposals {
            let proposal = proposals.last().unwrap();
            log::debug!("Sending {} with force for mission {}", proposal, mission);
            effects.push(Effect::Broadcast(Message::MissionGoing {
                mission,
                players: proposal.players.clone(),
            }));
            let next_game = GameStateWrapper::OnMission(GameState {
                phase: OnMission::new(proposals.len() - 1),
                game,
                proposals,
                mission_results,
            });
            (next_game, effects)
        } else {
            if mission == 1 {
                let first_proposal = &proposals[proposals.len() - 2];
                let second_proposal = proposals.last().unwrap();
                log::debug!(
                    "Voting on {} and {} for mission {}",
                    first_proposal,
                    second_proposal,
                    mission
                );
            } else {
                let proposal = proposals.last().unwrap();
                log::debug!("Voting on {} for mission {}", proposal, mission);
            }

            effects.push(Effect::Broadcast(Message::CommenceVoting));
            let next_game = GameStateWrapper::Voting(GameState {
                game,
                proposals,
                mission_results,
                phase: Voting::new(),
            });
            (next_game, effects)
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