// GameState uses this state machine pattern: https://www.reddit.com/r/rust/comments/b32eca/state_machine_implementation_questions_or_to_mut/eix2hxr/
// Each transition consumes the current state and returns the new state and side effects. Consumption semantics are nice for state machines in Rust
// (see https://hoverbear.org/blog/rust-state-machine-pattern). Returning an Effect lets us send updates back to the players without having to worry
// about errors putting the state machine in an invalid state. Without this, it's hard to return a Result that doesn't leave us stateless on error.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt;

use itertools::Itertools;
use log::debug;

use super::runner::{Action, Message};
use super::{Card, Game, MissionNumber, PlayerId, ProposalNumber};

/// State machine wrapper for THavalon games. Holds information needed throughout the game.ProposalNumber
#[derive(Debug, Clone)]
pub struct GameState {
    game: Game,
    phase: GamePhase,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum GamePhase {
    Pregame,
    OnMission(Mission),
    Assassination,
    Postgame,
}

impl GameState {
    pub fn new(game: Game) -> GameState {
        GameState {
            game,
            phase: GamePhase::Pregame,
        }
    }

    /// Start the game. Can only be called from the pre-game state
    pub fn on_start_game(self) -> (GameState, Vec<Effect>) {
        if self.phase != GamePhase::Pregame {
            // TODO: indicate the error
            return (self, vec![]);
        }

        let mut effects: Vec<_> = self
            .game
            .players
            .iter()
            .map(|player| {
                let info = &self.game.info[&player.id];
                Effect::Send(
                    player.id,
                    Message::RoleInformation {
                        role: player.role,
                        information: info.clone(),
                    },
                )
            })
            .collect();

        // Last 2 players in proposal order propose the first mission
        let first = self.game.proposal_order[self.game.size() - 2];
        let second = self.game.proposal_order[self.game.size() - 1];
        let next = GameState {
            game: self.game,
            phase: GamePhase::OnMission(Mission::new(1, first)),
        };

        effects.push(Effect::Broadcast(Message::NextProposal {
            proposer: first,
            mission: 1,
            proposal: 0,
        }));

        debug!(
            "Starting game. {} and {} are proposing first",
            first, second
        );
        (next, effects)
    }

    /// Update the game state based on a player action
    pub fn on_action(self, from: PlayerId, action: Action) -> (GameState, Vec<Effect>) {
        match action {
            Action::Propose { players } => self.on_proposal(from, players),
            Action::Vote { upvote } => self.on_vote(from, upvote),
            _ => unimplemented!(),
        }
    }

    fn on_proposal(
        self,
        proposer: PlayerId,
        players: HashSet<PlayerId>,
    ) -> (GameState, Vec<Effect>) {
        let GameState { game, phase } = self;
        match phase {
            GamePhase::OnMission(mission) => {
                let (next, effects) = mission.on_proposal(&game, proposer, players);
                (
                    GameState {
                        game,
                        phase: GamePhase::OnMission(next),
                    },
                    effects,
                )
            }
            _ => (
                GameState { game, phase },
                vec![Effect::error_to(
                    proposer,
                    "You can't propose a mission right now",
                )],
            ),
        }
    }

    fn on_vote(self, voter: PlayerId, upvote: bool) -> (GameState, Vec<Effect>) {
        let GameState { game, phase } = self;
        match phase {
            GamePhase::OnMission(mission) => {
                let (next, effects) = mission.on_vote(&game, voter, upvote);
                (
                    GameState {
                        game,
                        phase: GamePhase::OnMission(next),
                    },
                    effects,
                )
            }
            _ => (
                GameState { game, phase },
                vec![Effect::error_to(
                    voter,
                    "You can't vote a mission right now",
                )],
            ),
        }
    }
}

/// State machine wrapper for individual missions in a game. This holds shared information needed for every mission state.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mission {
    /// What mission number this is
    number: MissionNumber,
    /// Accumulator for submitted proposals
    proposals: Vec<Proposal>,
    /// Current mission state
    state: MissionState,
}

/// State machine for individual missions. Missions start out in the `Proposing` state, and alternate between `Proposing` and `Voting` until a mission
/// proposal passes or is forced to go. The mission then enters the `Going` state while players on the mission play their cards. Finally, it enters the
/// `Complete` state.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MissionState {
    Proposing {
        proposer: PlayerId,
    },
    Voting {
        votes: HashMap<PlayerId, bool>,
    },
    Going {
        sent_proposal: usize,
        cards: HashMap<PlayerId, Card>,
    },
    // Will probably need an intermediate state when waiting for Agravaine declarations.
    Complete {
        success: bool,
    },
}

impl Mission {
    /// Creates a new mission at its first proposal.
    pub fn new(number: MissionNumber, first_proposer: PlayerId) -> Mission {
        Mission {
            number,
            proposals: vec![],
            state: MissionState::Proposing {
                proposer: first_proposer,
            },
        }
    }

    /// Callback for proposals made on this mission
    pub fn on_proposal(
        mut self,
        game: &Game,
        proposer: PlayerId,
        players: HashSet<PlayerId>,
    ) -> (Mission, Vec<Effect>) {
        match self.state {
            MissionState::Proposing {
                proposer: expected_proposer,
            } if expected_proposer != proposer => {
                debug!("{} tried to propose out of turn", proposer);
                (
                    self.with_state(MissionState::Proposing {
                        proposer: expected_proposer,
                    }),
                    vec![Effect::error_to(proposer, "It's not your proposal")],
                )
            }
            MissionState::Proposing { .. } => {
                if players.len() != game.spec.mission_size(self.number) {
                    debug!("{} made an invalid proposal", proposer);
                    (
                        self,
                        vec![Effect::error_to(proposer, "Not a valid proposa;")],
                    )
                } else {
                    debug!("{} proposed {}", proposer, players.iter().format(", "));

                    let mut effects = vec![Effect::Broadcast(Message::ProposalMade {
                        proposer,
                        mission: self.number,
                        proposal: self.proposals.len() as ProposalNumber,
                        players: players.clone(),
                    })];

                    self.proposals.push(Proposal { proposer, players });

                    // Mission 1 is a special case where players see both proposals before voting
                    let next_state = if self.number == 1 && self.proposals.len() == 1 {
                        let next_proposer = game.next_proposer(proposer);
                        effects.push(Effect::Broadcast(Message::NextProposal {
                            mission: self.number,
                            proposer: next_proposer,
                            proposal: self.proposals.len() as ProposalNumber,
                        }));
                        MissionState::Proposing {
                            proposer: next_proposer,
                        }
                    } else {
                        effects.push(Effect::Broadcast(Message::CommenceVoting));
                        MissionState::Voting {
                            votes: HashMap::new(),
                        }
                    };

                    (self.with_state(next_state), effects)
                }
            }
            _ => (
                self,
                vec![Effect::error_to(
                    proposer,
                    "You can't propose a mission right now",
                )],
            ),
        }
    }

    pub fn on_vote(self, game: &Game, player: PlayerId, upvote: bool) -> (Mission, Vec<Effect>) {
        match self.state {
            MissionState::Voting { mut votes } => {
                debug_assert!(
                    !self.proposals.is_empty(),
                    "Should not be voting with no proposals"
                );
                match votes.entry(player) {
                    Entry::Occupied(_) => (
                        Mission {
                            number: self.number,
                            proposals: self.proposals,
                            state: MissionState::Voting { votes },
                        },
                        vec![Effect::error_to(
                            player,
                            "You already voted on this mission",
                        )],
                    ),
                    Entry::Vacant(entry) => {
                        entry.insert(upvote);
                        debug!(
                            "{} voted {} the proposal",
                            player,
                            if upvote { "for" } else { "against" }
                        );

                        if votes.len() == game.size() {
                            let mut upvotes = Vec::new();
                            let mut downvotes = Vec::new();
                            for (&player, &vote) in votes.iter() {
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
                                sent,
                            };

                            // Voting on mission 1 always sends a mission
                            if self.number == 1 {
                                debug_assert!(
                                    self.proposals.len() == 2,
                                    "Should not be voting on mission 1 without both proposals"
                                );
                                let sent_proposal = if sent { 0 } else { 1 };
                                debug!("Sending {}", &self.proposals[sent_proposal]);
                                let next = Mission {
                                    number: self.number,
                                    proposals: self.proposals,
                                    state: MissionState::Going {
                                        sent_proposal,
                                        cards: HashMap::new(),
                                    },
                                };
                                (next, vec![Effect::Broadcast(voting_results)])
                            } else if sent {
                                let sent_proposal = self.proposals.len() - 1;
                                debug!("Sending {}", &self.proposals[sent_proposal]);
                                let next = Mission {
                                    number: self.number,
                                    proposals: self.proposals,
                                    state: MissionState::Going {
                                        sent_proposal,
                                        cards: HashMap::new(),
                                    },
                                };
                                (next, vec![Effect::Broadcast(voting_results)])
                            } else {
                                debug!("Did not send {}", self.proposals.last().unwrap());
                                let next_proposer =
                                    game.next_proposer(self.proposals.last().unwrap().proposer);
                                let effects = vec![
                                    Effect::Broadcast(voting_results),
                                    Effect::Broadcast(Message::NextProposal {
                                        proposer: next_proposer,
                                        mission: self.number,
                                        proposal: self.proposals.len() as ProposalNumber,
                                    }),
                                ];
                                let next = Mission {
                                    number: self.number,
                                    proposals: self.proposals,
                                    state: MissionState::Proposing {
                                        proposer: next_proposer,
                                    },
                                };
                                (next, effects)
                            }
                        } else {
                            (
                                Mission {
                                    number: self.number,
                                    proposals: self.proposals,
                                    state: MissionState::Voting { votes },
                                },
                                vec![],
                            )
                        }
                    }
                }
            }
            _ => (
                self,
                vec![Effect::error_to(
                    player,
                    "You can't vote on a mission right now",
                )],
            ),
        }
    }

    fn with_state(self, next_state: MissionState) -> Mission {
        Mission {
            number: self.number,
            proposals: self.proposals,
            state: next_state,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Proposal {
    proposer: PlayerId,
    players: HashSet<PlayerId>,
}

impl fmt::Display for Proposal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}'s proposal of {}",
            self.proposer,
            self.players.iter().format(", ")
        )
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

/*

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

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
            role: Role::Maelegant,
        });
        players.add_player(Player {
            id: 5,
            name: "Player 5".to_string(),
            role: Role::Morgana,
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
            prev_proposals: vec![],
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
            prev_proposals: vec![],
        };
        let state = GameState::Proposing(proposing.clone());
        let (next_state, effects) = state.on_proposal(&game, 2, hashset![3, 4]);
        assert_eq!(next_state, GameState::Proposing(proposing));
        assert_eq!(effects, vec![Effect::error_to(2, "It's not your proposal")]);
    }

    #[test]
    fn test_good_proposal() {
        let game = fresh_game();
        let first_proposal = Proposal {
            mission: 2,
            proposal: 1,
            proposer: 4,
            players: hashset![4, 3],
        };
        let state = GameState::Proposing(Proposing {
            mission: 2,
            proposal: 2,
            proposer: 5,
            prev_proposals: vec![first_proposal.clone()],
        });
        let (next_state, effects) = state.on_proposal(&game, 5, hashset![5, 1, 2]);
        assert_eq!(
            next_state,
            GameState::Voting(Voting {
                votes: HashMap::new(),
                proposals: vec![
                    first_proposal,
                    Proposal {
                        mission: 2,
                        proposal: 2,
                        proposer: 5,
                        players: hashset![5, 1, 2]
                    }
                ]
            })
        );
        assert_eq!(
            effects,
            vec![
                Effect::Broadcast(Message::ProposalMade {
                    mission: 2,
                    proposal: 2,
                    proposer: 5,
                    players: hashset![5, 1, 2]
                }),
                Effect::Broadcast(Message::CommenceVoting)
            ]
        );
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
        assert_eq!(
            effects,
            vec![Effect::error_to(3, "You already voted on this mission")]
        );
    }

    #[test]
    fn test_middle_vote() {
        let game = fresh_game();
        let proposal = Proposal {
            mission: 1,
            proposal: 2,
            proposer: 3,
            players: hashset![3, 2],
        };
        let state = GameState::Voting(Voting {
            proposals: vec![proposal.clone()],
            votes: hashmap![
                3 => true
            ],
        });
        let (next_state, effects) = state.on_vote(&game, 4, true);
        assert_eq!(
            next_state,
            GameState::Voting(Voting {
                proposals: vec![proposal],
                votes: hashmap![3 => true, 4 => true],
            })
        );
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
                players: hashset![3, 2],
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
        assert_eq!(
            effects,
            vec![Effect::Broadcast(Message::VotingResults {
                upvotes: vec![2, 3, 5],
                downvotes: vec![1, 4],
                sent: true
            })]
        );
    }

    #[test]
    fn test_failing_vote() {
        let game = fresh_game();
        let failed_proposal = Proposal {
            mission: 2,
            proposal: 2,
            proposer: 3,
            players: hashset![3, 2],
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
        assert_eq!(
            next_state,
            GameState::Proposing(Proposing {
                mission: 2,
                proposal: 3,
                proposer: 4,
                prev_proposals: vec![failed_proposal]
            })
        );
        assert_eq!(
            effects,
            vec![
                Effect::Broadcast(Message::VotingResults {
                    upvotes: vec![2, 3],
                    downvotes: vec![1, 4, 5],
                    sent: false
                }),
                Effect::Broadcast(Message::NextProposal {
                    proposer: 4,
                    proposal: 3,
                    mission: 2
                })
            ]
        );
    }
}

*/
