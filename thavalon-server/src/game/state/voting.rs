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

        let mut effects = vec![Effect::Broadcast(Message::VoteReceived)];

        if self.phase.votes.len() == self.game.size() {
            let mission = self.mission();

            let mut upvotes = HashSet::new();
            let mut downvotes = HashSet::new();

            for (player, vote) in self.phase.votes.drain() {
                let is_arthur = self.game.players.is(&player, Role::Arthur)
                    && self.role_state.arthur.has_declared();

                let collection = if vote { &mut upvotes } else { &mut downvotes };

                if is_arthur {
                    collection.insert(format!("{} (Arthur)", player));
                }
                collection.insert(player);
            }

            let sent = upvotes.len() > downvotes.len();

            // TODO: This probably could be cleaner, but hacking this for pre-alpha.
            if self.phase.obscured {
                effects.push(Effect::Broadcast(Message::Toast {
                    severity: ToastSeverity::WARN,
                    message: format!(
                        "Mission {}: Maeve has obscured the votes!\nUpvotes: {}\nDownvotes: {}",
                        mission,
                        upvotes.len(),
                        downvotes.len()
                    ),
                }));
            }

            let vote_counts = if self.phase.obscured {
                messages::VoteCounts::Obscured {
                    upvotes: upvotes.len() as u32,
                    downvotes: downvotes.len() as u32,
                }
            } else {
                messages::VoteCounts::Public { upvotes, downvotes }
            };

            effects.push(Effect::Broadcast(Message::VotingResults {
                sent,
                counts: vote_counts,
            }));

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
            (GameStateWrapper::Voting(self), effects)
        }
    }

    pub fn handle_obscure(mut self, player: &str) -> ActionResult {
        if self.game.players.by_name(player).unwrap().role == Role::Maeve {
            if self.phase.obscured {
                self.player_error("You already obscured the votes for this proposal")
            } else if self.role_state.maeve.can_obscure() {
                log::debug!("Maeve obscured the votes!");
                self.role_state.maeve.mark_obscure();
                self.phase.obscured = true;
                (GameStateWrapper::Voting(self), vec![])
            } else {
                self.player_error("You can't obscure this round")
            }
        } else {
            self.player_error("You can't obscure votes")
        }
    }

    /// Cancels voting, returning to the player who had been proposing. This is used for Arthur declarations while voting, since
    /// if Arthur were on the proposal it is no longer valid.
    pub fn cancel_vote(mut self, effects: Vec<Effect>) -> ActionResult {
        // Remove the last proposal, since it's getting re-proposed
        let proposal = self
            .proposals
            .pop()
            .expect("In Voting phase with no proposals");
        log::debug!("Cancelling vote on {}", proposal);
        self.into_proposing(proposal.proposer, effects)
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
