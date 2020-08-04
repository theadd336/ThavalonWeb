// GameState uses this state machine pattern: https://www.reddit.com/r/rust/comments/b32eca/state_machine_implementation_questions_or_to_mut/eix2hxr/
// Each transition consumes the current state and returns the new state and side effects. Consumption semantics are nice for state machines in Rust
// (see https://hoverbear.org/blog/rust-state-machine-pattern). Returning an Effect lets us send updates back to the players without having to worry
// about errors putting the state machine in an invalid state. Without this, it's hard to return a Result that doesn't leave us stateless on error.

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt;

use itertools::Itertools;
use log::debug;

use super::{Game, PlayerId, MissionNumber, ProposalNumber};
use super::runner::{Action, Message};

/// State machine for THavalon games
#[derive(Debug, Eq, PartialEq)]
pub enum GameState {
    Pregame,
    Proposing(Proposing),
    Voting(Voting),
    Mission,
    Assassination,
    Postgame
}

// Idea: nested state machines, one for the overall game and one for each mission
// cuts down on mission # tracking a bit?
// Also an idea: make GameState own Game, instead of GameRunner

impl GameState {
    /// Start the game. Can only be called from the pre-game state
    pub fn on_start_game(self, game: &Game) -> (GameState, Vec<Effect>) {
        if self != GameState::Pregame {
            // TODO: indicate the error
            return (self, vec![]);
        }

        let effects = game.players.iter().map(|player| {
            let info = &game.info[&player.id];
            Effect::Send(player.id, Message::RoleInformation { role: player.role, information: info.clone() })
        }).collect();

        // Last 2 players in proposal order propose the first mission
        let first_proposer = game.proposal_order[game.size() - 2];
        let state = GameState::Proposing(Proposing {
            mission: 1,
            proposal: 0,
            proposer: first_proposer,
            prev_proposals: vec![]
        });
        debug!("Starting game. {} and {} are proposing first", first_proposer, game.proposal_order[game.size() - 1]);
        (state, effects)
    }

    /// Update the game state based on a player action
    pub fn on_action(self, game: &Game, from: PlayerId, action: Action) -> (GameState, Vec<Effect>) {
        match action {
            Action::Propose { players } => self.on_proposal(game, from, players),
            Action::Vote { upvote } => self.on_vote(game, from, upvote),
            _ => unimplemented!()
        }
    }

    /// Handles a player making a mission proposal.
    fn on_proposal(self, game: &Game, proposer: PlayerId, players: HashSet<PlayerId>) -> (GameState, Vec<Effect>) {
        match self {
            GameState::Proposing(prop) if prop.proposer != proposer => (GameState::Proposing(prop), vec![Effect::error_to(proposer, "It's not your proposal")]),
            GameState::Proposing(prop) => {
                if !prop.is_valid(game, &players) {
                    debug!("{} made an invalid proposal of {}", proposer, players.iter().join(", "));
                    (GameState::Proposing(prop), vec![Effect::error_to(proposer, "Not a valid proposal")])
                } else {
                    debug!("{} proposed {} for mission {}", proposer, players.iter().join(", "), prop.mission);

                    // Mission 1 is a special case where players see both proposals before voting
                    if prop.mission == 1 && prop.proposal == 0 {
                        let this_proposal = prop.make_proposal(players.clone());
                        let mut proposals = prop.prev_proposals;
                        proposals.push(this_proposal);
                        let next_proposer = game.next_proposer(proposer);

                        let next_state = GameState::Proposing(Proposing {
                            mission: 1,
                            proposer: next_proposer,
                            proposal: 1,
                            prev_proposals: proposals,
                        });

                        let effects = vec![
                            Effect::Broadcast(Message::ProposalMade {
                                proposer,
                                mission: 1,
                                proposal: 0,
                                players,
                            }),
                            Effect::Broadcast(Message::NextProposal {
                                proposer: next_proposer,
                                mission: 1,
                                proposal: 1
                            })
                        ];
                        (next_state, effects)
                    } else {
                        let effects = vec![
                            Effect::Broadcast(Message::ProposalMade {
                                proposer: prop.proposer,
                                mission: prop.mission,
                                proposal: prop.proposal,
                                players: players.clone()
                            }),
                            Effect::Broadcast(Message::CommenceVoting),
                        ];
                        let voting = Voting::from_proposing(prop, players);
                        debug!("Voting on {}", voting.voting_proposal());
                        let next_state = GameState::Voting(voting);
                        (next_state, effects)
                    }
                }
            },
            st => (st, vec![Effect::error_to(proposer, "You can't propose right now")])
        }
    }

    fn on_vote(self, game: &Game, player: PlayerId, upvote: bool) -> (GameState, Vec<Effect>) {
        match self {
            GameState::Voting(mut vote) => {
                match vote.votes.entry(player) {
                    Entry::Occupied(_) => (GameState::Voting(vote), vec![Effect::error_to(player, "You already voted on this mission")]),
                    Entry::Vacant(entry) => {
                        entry.insert(upvote);
                        debug!("{} {} the proposal", player, if upvote { "upvoted" } else { "downvoted" });
                        if vote.votes.len() == game.size() {
                            let mut upvotes = Vec::new();
                            let mut downvotes = Vec::new();
                            for (&player, &vote) in vote.votes.iter() {
                                if vote {
                                    upvotes.push(player);
                                } else {
                                    downvotes.push(player);
                                }
                            }
                            // Sort for stable output order (mostly for tests)
                            upvotes.sort();
                            downvotes.sort();
                            let sent = upvotes.len() > game.size() / 2;
                            let voting_results = Message::VotingResults {
                                upvotes,
                                downvotes,
                                sent
                            };

                            if sent {
                                debug!("Sending {}", vote.voting_proposal());
                                (GameState::Mission, vec![Effect::Broadcast(voting_results)])
                            } else if vote.mission() == 1 {
                                let sent_proposal = vote.proposals.get(1).expect("Invalid state: voted on mission 1 with only one proposal");
                                debug!("Sending {}", sent_proposal);
                                (GameState::Mission, vec![Effect::Broadcast(voting_results)])
                            } else {
                                let failed_proposal = vote.voting_proposal();
                                debug!("Did not send {}", failed_proposal);
                                let next_proposer = game.next_proposer(failed_proposal.proposer);
                                let mission = failed_proposal.mission;
                                let proposal = failed_proposal.proposal + 1; // TODO: force (needs to be handled in proposals state)

                                let next_state = GameState::Proposing(Proposing {
                                    mission,
                                    proposal,
                                    proposer: next_proposer,
                                    prev_proposals: vote.proposals,
                                });
                                let effects = vec![
                                    Effect::Broadcast(voting_results),
                                    Effect::Broadcast(Message::NextProposal {
                                        proposer: next_proposer,
                                        mission,
                                        proposal,
                                    })
                                ];
                                (next_state, effects)
                            }
                        } else {
                            // notify them that we got their vote?
                            (GameState::Voting(vote), vec![])
                        }
                    }
                }
            },
            st => (st, vec![Effect::error_to(player, "There's nothing to vote on")])
        }
    }
}

/// The Proposing state.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Proposing {
    /// Which mission this is for
    mission: MissionNumber,
    /// Which proposal number this is
    proposal: ProposalNumber,
    /// Who's proposing the mission
    proposer: PlayerId,

    /// Previous proposals this round
    prev_proposals: Vec<Proposal>,
}

impl Proposing {
    /// Checks if `players` is a valid proposal
    fn is_valid(&self, game: &Game, players: &HashSet<PlayerId>) -> bool {
        players.len() == game.spec.mission_size(self.mission)
    }

    /// Creates a Proposal given a valid set of proposed players
    fn make_proposal(&self, players: HashSet<PlayerId>) -> Proposal {
        Proposal {
            mission: self.mission,
            proposer: self.proposer,
            proposal: self.proposal,
            players,
        }
    }
}

/// Contents of GameState::Voting
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Voting {
    /// All proposals so far this round. In almost all cases, the proposal being voted on is the last one. The exception is mission 1, where
    /// an upvote is a vote for the _first_ proposal of the round.
    proposals: Vec<Proposal>,

    /// Tracks how each player voted. true = upvote, false = downvote
    // TODO: questing beast
    votes: HashMap<PlayerId, bool>,
}

impl Voting {
    fn from_proposing(prop: Proposing, players: HashSet<PlayerId>) -> Voting {
        let proposal = prop.make_proposal(players);
        let mut proposals = prop.prev_proposals;
        proposals.push(proposal);
        Voting {
            proposals,
            votes: HashMap::new()
        }
    }

    /// The mission we're voting on a proposal for
    fn mission(&self) -> MissionNumber {
        self.proposals.first().expect("Cannot be in Voting state with no proposals").mission
    }

    /// The proposal being voted on (the one sent if there are enough upvotes)
    fn voting_proposal(&self) -> &Proposal {
        if self.mission() == 1 {
            &self.proposals[0]
        } else {
            self.proposals.last().expect("Cannot be in Voting state with no proposals")
        }
    }
}

/// A mission proposal
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Proposal {
    mission: MissionNumber,
    proposal: ProposalNumber,
    proposer: PlayerId,
    players: HashSet<PlayerId>
}

impl fmt::Display for Proposal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Player {}'s proposal on mission {} (proposal #{}): ", self.proposer, self.mission, self.proposal)?;
        write!(f, "{}", self.players.iter().format(", "))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Effect {
    Broadcast(Message),
    Send(PlayerId, Message),
}

impl Effect {
    /// Effect for sending `player` an error message.
    pub fn error_to<S: Into<String>>(player: PlayerId, error: S) -> Effect {
        Effect::Send(player, Message::Error(error.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

    use maplit::{hashmap, hashset};

    fn fresh_game() -> Game {
        let mut players = Players::new();
        players.add_player(Player {
            id: 1,
            name: "Player 1".to_string(),
            role: Role::Merlin,
        });
        players.add_player(Player {
            id: 2,
            name: "Player 2".to_string(),
            role: Role::Percival,
        });
        players.add_player(Player {
            id: 3,
            name: "Player 3".to_string(),
            role: Role::Iseult,
        });
        players.add_player(Player {
            id: 4,
            name: "Player 4".to_string(),
            role: Role::Maelegant
        });
        players.add_player(Player {
            id: 5,
            name: "Player 5".to_string(),
            role: Role::Morgana
        });

        Game {
            players,
            info: HashMap::new(),
            proposal_order: vec![1, 2, 3, 4, 5],
            spec: &FIVE_PLAYER,
        }
    }

    #[test]
    fn test_invalid_proposal_size() {
        let game = fresh_game();
        let proposing = Proposing {
            mission: 1,
            proposal: 2,
            proposer: 5,
            prev_proposals: vec![]
        };
        let state = GameState::Proposing(proposing.clone());

        let (next_state, effects) = state.on_proposal(&game, 5, hashset![1]);
        assert_eq!(next_state, GameState::Proposing(proposing));
        assert_eq!(effects, vec![Effect::error_to(5, "Not a valid proposal")]);
    }

    #[test]
    fn test_invalid_proposer() {
        let game = fresh_game();
        let proposing = Proposing {
            mission: 1,
            proposal: 2,
            proposer: 5,
            prev_proposals: vec![]
        };
        let state = GameState::Proposing(proposing.clone());
        let (next_state, effects) = state.on_proposal(&game, 2, hashset![3, 4]);
        assert_eq!(next_state, GameState::Proposing(proposing));
        assert_eq!(effects, vec![Effect::error_to(2, "It's not your proposal")]);
    }

    #[test]
    fn test_good_proposal() {
        let game = fresh_game();
        let first_proposal = Proposal { mission: 2, proposal: 1, proposer: 4, players: hashset![4, 3] };
        let state = GameState::Proposing(Proposing {
            mission: 2,
            proposal: 2,
            proposer: 5,
            prev_proposals: vec![first_proposal.clone()],
        });
        let (next_state, effects) = state.on_proposal(&game, 5, hashset![5, 1, 2]);
        assert_eq!(next_state, GameState::Voting(Voting {
            votes: HashMap::new(),
            proposals: vec![first_proposal, Proposal {
                mission: 2,
                proposal: 2,
                proposer: 5,
                players: hashset![5, 1, 2]

            }]
        }));
        assert_eq!(effects, vec![
            Effect::Broadcast(Message::ProposalMade { mission: 2, proposal: 2, proposer: 5, players: hashset![5, 1, 2] }),
            Effect::Broadcast(Message::CommenceVoting)
        ]);
    }

    #[test]
    fn test_double_vote() {
        let game = fresh_game();
        let voting = Voting {
            proposals: vec![Proposal {
                mission: 1,
                proposal: 2,
                proposer: 3,
                players: hashset![3, 2],
            }],
            votes: hashmap![3 => true],
        };
        let state = GameState::Voting(voting.clone());
        let (next_state, effects) = state.on_vote(&game, 3, false);
        assert_eq!(next_state, GameState::Voting(voting));
        assert_eq!(effects, vec![Effect::error_to(3, "You already voted on this mission")]);
    }

    #[test]
    fn test_middle_vote() {
        let game = fresh_game();
        let proposal = Proposal {
            mission: 1,
            proposal: 2,
            proposer: 3,
            players: hashset![3, 2]
        };
        let state = GameState::Voting(Voting {
            proposals: vec![proposal.clone()],
            votes: hashmap![
                3 => true
            ],
        });
        let (next_state, effects) = state.on_vote(&game, 4, true);
        assert_eq!(next_state, GameState::Voting(Voting {
            proposals: vec![proposal],
            votes: hashmap![3 => true, 4 => true],
        }));
        assert_eq!(effects, vec![]);
    }

    #[test]
    fn test_passing_vote() {
        let game = fresh_game();
        let state = GameState::Voting(Voting {
            proposals: vec![Proposal {
                mission: 1,
                proposal: 2,
                proposer: 3,
                players: hashset![3, 2]
            }],
            votes: hashmap![
                1 => false,
                2 => true,
                3 => true,
                4 => false
            ],
        });
        let (next_state, effects) = state.on_vote(&game, 5, true);
        assert_eq!(next_state, GameState::Mission);
        assert_eq!(effects, vec![Effect::Broadcast(Message::VotingResults {
            upvotes: vec![2, 3, 5],
            downvotes: vec![1, 4],
            sent: true
        })]);
    }

    #[test]
    fn test_failing_vote() {
        let game = fresh_game();
        let failed_proposal = Proposal {
            mission: 2,
            proposal: 2,
            proposer: 3,
            players: hashset![3, 2]
        };
        let state = GameState::Voting(Voting {
            proposals: vec![failed_proposal.clone()],
            votes: hashmap![
                1 => false,
                2 => true,
                3 => true,
                4 => false
            ],
        });
        let (next_state, effects) = state.on_vote(&game, 5, false);
        assert_eq!(next_state, GameState::Proposing(Proposing {
            mission: 2,
            proposal: 3,
            proposer: 4,
            prev_proposals: vec![failed_proposal]
        }));
        assert_eq!(effects, vec![
            Effect::Broadcast(Message::VotingResults {
                upvotes: vec![2, 3],
                downvotes: vec![1, 4, 5],
                sent: false
            }),
            Effect::Broadcast(Message::NextProposal { proposer: 4, proposal: 3, mission: 2 })
        ]);
    }
}