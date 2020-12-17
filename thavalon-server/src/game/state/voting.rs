use std::collections::{HashMap, HashSet};

use super::prelude::*;

/// Phase for voting on a mission proposal
pub struct Voting {
    /// Map from players to whether they up- or down-voted the proposal.
    votes: HashMap<String, bool>,
    /// Whether or not the voting results are obscured
    obscured: bool,
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

            let mut upvotes = HashSet::new();
            let mut downvotes = HashSet::new();

            for (player, vote) in self.phase.votes.drain() {
                if vote {
                    upvotes.insert(player);
                } else {
                    downvotes.insert(player);
                }
            }

            let sent = upvotes.len() > downvotes.len();
            let vote_counts = if self.phase.obscured {
                messages::VoteCounts::Obscured {
                    upvotes: upvotes.len() as u32,
                    downvotes: downvotes.len() as u32,
                }
            } else {
                messages::VoteCounts::Public { upvotes, downvotes }
            };

            let mut effects = vec![Effect::Broadcast(Message::VotingResults {
                sent,
                counts: vote_counts,
            })];

            if mission == 1 {
                let proposal_index = if sent { 0 } else { 1 };
                let proposal = &self.proposals[proposal_index];
                log::debug!("Voted to send {} on mission 1", proposal);
                effects.push(Effect::Broadcast(Message::MissionGoing {
                    mission,
                    players: proposal.players.clone(),
                }));
                let next_state = self.with_phase(OnMission::new(proposal_index));
                (GameStateWrapper::OnMission(next_state), effects)
            } else {
                let proposal = self.proposals.last().expect("Voted with no proposals!");
                if sent {
                    log::debug!("Voted to send {} on mission {}", proposal, mission);
                    effects.push(Effect::Broadcast(Message::MissionGoing {
                        mission,
                        players: proposal.players.clone(),
                    }));
                    let proposal_index = self.proposals.len() - 1;
                    let next_state = self.with_phase(OnMission::new(proposal_index));
                    (GameStateWrapper::OnMission(next_state), effects)
                } else {
                    log::debug!("Voted not to send {}", proposal);

                    let next_proposer = self.game.next_proposer(&proposal.proposer).to_string();
                    self.into_proposing(next_proposer, effects)
                }
            }
        } else {
            // If we don't have all the votes yet, there's no state change
            (GameStateWrapper::Voting(self), vec![])
        }
    }

    pub fn handle_obscure(self, player: &str) -> ActionResult {
        if self.game.players.by_name(player).unwrap().role == Role::Maeve {
            if self.phase.obscured {
                self.player_error("You already obscured the votes for this proposal")
            } else if self.role_state.maeve.can_obscure() {
                log::debug!("Maeve obscured the votes!");
                (GameStateWrapper::Voting(self), vec![])
            } else {
                self.player_error("You can't obscure this round")
            }
        } else {
            self.player_error("You can't obscure votes")
        }
    }
}

impl Voting {
    /// Create a new `Voting` phase with no votes yet cast.
    pub fn new() -> Voting {
        Voting {
            votes: HashMap::new(),
            obscured: false,
        }
    }
}

impl_phase!(Voting);
