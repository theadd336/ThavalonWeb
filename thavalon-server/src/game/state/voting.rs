use std::collections::{HashMap, HashSet};

use super::prelude::*;

/// Phase for voting on a mission proposal
pub struct Voting {
    /// Map from players to whether they up- or down-voted the proposal.
    votes: HashMap<String, bool>,
}

impl GameState<Voting> {
    pub fn handle_vote(mut self, player: &str, is_upvote: bool) -> ActionResult {
        if self.phase.votes.contains_key(player) {
            return self.player_error("You already voted");
        }

        log::debug!(
            "{} {}",
            player,
            if is_upvote { "upvoted" } else { "downvoted" }
        );
        self.phase.votes.insert(player.to_string(), is_upvote);

        if self.phase.votes.len() == self.game.size() {
            let mission = self.mission();
            let spent_proposals = self.spent_proposals();
            // TODO: avoid destructuring so we can use GameState helpers
            let GameState {
                game,
                proposals,
                mission_results,
                phase: Voting { votes },
            } = self;

            let mut upvotes = HashSet::new();
            let mut downvotes = HashSet::new();

            for (player, vote) in votes.into_iter() {
                if vote {
                    upvotes.insert(player);
                } else {
                    downvotes.insert(player);
                }
            }

            let sent = upvotes.len() > downvotes.len();
            let mut effects = vec![Effect::Broadcast(Message::VotingResults {
                sent,
                counts: messages::VoteCounts::Public { upvotes, downvotes },
            })];

            if mission == 1 {
                let proposal_index = if sent { 0 } else { 1 };
                let proposal = &proposals[proposal_index];
                log::debug!("Voted to send {} on mission 1", proposal);
                effects.push(Effect::Broadcast(Message::MissionGoing {
                    mission,
                    players: proposal.players.clone(),
                }));
                let next_game = GameStateWrapper::OnMission(GameState {
                    phase: OnMission::new(proposal_index),
                    game,
                    proposals,
                    mission_results,
                });
                (next_game, effects)
            } else {
                let proposal = proposals.last().expect("Voted with no proposals!");
                if sent {
                    log::debug!("Voted to send {} on mission {}", proposal, mission);
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
                    log::debug!("Voted not to send {}", proposal);
                    let next_proposer = game.next_proposer(&proposal.proposer);
                    effects.push(Effect::Broadcast(Message::NextProposal {
                        proposer: next_proposer.to_string(),
                        mission,
                        proposals_made: spent_proposals,
                        max_proposals: game.spec.max_proposals,
                    }));
                    let next_game = GameStateWrapper::Proposing(GameState {
                        phase: Proposing::new(next_proposer.to_string()),
                        game,
                        proposals,
                        mission_results,
                    });
                    (next_game, effects)
                }
            }
        } else {
            // If we don't have all the votes yet, there's no state change
            (GameStateWrapper::Voting(self), vec![])
        }
    }
}

impl Voting {
    /// Create a new `Voting` phase with no votes yet cast.
    pub fn new() -> Voting {
        Voting {
            votes: HashMap::new(),
        }
    }
}

impl_phase!(Voting);
