// GameState uses this state machine pattern: https://www.reddit.com/r/rust/comments/b32eca/state_machine_implementation_questions_or_to_mut/eix2hxr/
// Each transition consumes the current state and returns the new state and side effects. Consumption semantics are nice for state machines in Rust
// (see https://hoverbear.org/blog/rust-state-machine-pattern). Returning an Effect lets us send updates back to the players without having to worry
// about errors putting the state machine in an invalid state. Without this, it's hard to return a Result that doesn't leave us stateless on error.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use super::{Game, PlayerId};
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
        });
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

    fn on_proposal(self, game: &Game, proposer: PlayerId, players: Vec<PlayerId>) -> (GameState, Vec<Effect>) {
        match self {
            GameState::Proposing(prop) if prop.proposer != proposer => (GameState::Proposing(prop), vec![Effect::error_to(proposer, "It's not your proposal")]),
            GameState::Proposing(prop) => {
                let expected_size = game.spec.mission_size(prop.mission);
                if players.len() != expected_size {
                    let effect = Effect::error_to(proposer, format!("You proposed {} players, but mission {} needs {}", players.len(), prop.mission, expected_size));
                    (GameState::Proposing(prop), vec![effect])
                } else {
                    // TODO: check no duplicate players
                    let voting = GameState::Voting(Voting::from_proposal(prop, players.clone()));
                    (voting, vec![Effect::Broadcast(Message::CommenceVoting { proposal: players })])
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
                        if vote.votes.len() == game.size() {
                            let mut upvotes = Vec::new();
                            let mut downvotes = Vec::new();
                            for (player, vote) in vote.votes.iter() {
                                if *vote {
                                    upvotes.push(*player);
                                } else {
                                    downvotes.push(*player);
                                }
                            }
                            let sent = upvotes.len() > game.size() / 2;
                            // Sort for stable output order (mostly for tests)
                            upvotes.sort();
                            downvotes.sort();
                            let voting_results = Message::VotingResults {
                                upvotes,
                                downvotes,
                                sent
                            };

                            if sent {
                                (GameState::Mission, vec![Effect::Broadcast(voting_results)])
                            } else {
                                let next_proposer = game.next_proposer(vote.proposer);
                                let next_state = GameState::Proposing(Proposing {
                                    mission: vote.mission,
                                    proposal: vote.proposal + 1, // TODO: max # of proposals
                                    proposer: next_proposer,
                                });
                                let effects = vec![Effect::Broadcast(voting_results), Effect::Broadcast(Message::NextProposal { player: next_proposer })];
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

/// Contents of GameState::Proposing
#[derive(Debug, Eq, PartialEq)]
pub struct Proposing {
    /// Which mission this is for
    mission: usize,
    /// Which proposal number this is
    proposal: usize,
    /// Who's proposing the mission
    proposer: PlayerId,
}

/// Contents of GameState::Voting
#[derive(Debug, Eq, PartialEq)]
pub struct Voting {
    mission: usize,
    proposal: usize,
    proposer: PlayerId,
    players: Vec<PlayerId>,

    /// Tracks how each player voted. true = upvote, false = downvote
    // TODO: questing beast
    votes: HashMap<PlayerId, bool>,
}

impl Voting {
    fn from_proposal(prop: Proposing, players: Vec<PlayerId>) -> Voting {
        Voting {
            mission: prop.mission,
            proposal: prop.proposal,
            proposer: prop.proposer,
            players,
            votes: HashMap::new()
        }
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

    use maplit::hashmap;

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
        let state = GameState::Proposing(Proposing {
            mission: 1,
            proposal: 2,
            proposer: 5,
        });

        let (next_state, effects) = state.on_proposal(&game, 5, vec![1]);
        assert_eq!(next_state, GameState::Proposing(Proposing {
            mission: 1,
            proposal: 2,
            proposer: 5,
        }));
        assert_eq!(effects, vec![Effect::error_to(5, "You proposed 1 players, but mission 1 needs 2")]);
    }

    #[test]
    fn test_invalid_proposer() {
        let game = fresh_game();
        let state = GameState::Proposing(Proposing {
            mission: 1,
            proposal: 2,
            proposer: 5
        });
        let (next_state, effects) = state.on_proposal(&game, 2, vec![3, 4]);
        assert_eq!(next_state, GameState::Proposing(Proposing {
            mission: 1,
            proposal: 2,
            proposer: 5
        }));
        assert_eq!(effects, vec![Effect::error_to(2, "It's not your proposal")]);
    }

    #[test]
    fn test_good_proposal() {
        let game = fresh_game();
        let state = GameState::Proposing(Proposing {
            mission: 1,
            proposal: 2,
            proposer: 5
        });
        let (next_state, effects) = state.on_proposal(&game, 5, vec![5, 1]);
        assert_eq!(next_state, GameState::Voting(Voting {
            mission: 1,
            proposal: 2,
            proposer: 5,
            players: vec![5, 1],
            votes: HashMap::new()
        }));
        assert_eq!(effects, vec![Effect::Broadcast(Message::CommenceVoting {
            proposal: vec![5, 1]
        })]);
    }

    #[test]
    fn test_double_vote() {
        let game = fresh_game();
        let state = GameState::Voting(Voting {
            mission: 1,
            proposal: 2,
            proposer: 3,
            players: vec![3, 2],
            votes: hashmap![
                3 => true
            ]
        });
        let (_, effects) = state.on_vote(&game, 3, false);
        assert_eq!(effects, vec![Effect::error_to(3, "You already voted on this mission")]);
    }

    #[test]
    fn test_middle_vote() {
        let game = fresh_game();
        let state = GameState::Voting(Voting {
            mission: 1,
            proposal: 2,
            proposer: 3,
            players: vec![3, 2],
            votes: hashmap![
                3 => true
            ]
        });
        let (next_state, effects) = state.on_vote(&game, 4, true);
        let mut votes = HashMap::new();
        votes.insert(3, true);
        votes.insert(4, true);
        assert_eq!(next_state, GameState::Voting(Voting {
            mission: 1,
            proposal: 2,
            proposer: 3,
            players: vec![3, 2],
            votes
        }));
        assert_eq!(effects, vec![]);
    }

    #[test]
    fn test_passing_vote() {
        let game = fresh_game();
        let state = GameState::Voting(Voting {
            mission: 1,
            proposal: 2,
            proposer: 3,
            players: vec![3, 2],
            votes: hashmap![
                1 => false,
                2 => true,
                3 => true,
                4 => false
            ]
        });
        let (next_state, effects) = state.on_vote(&game, 5, true);
        let mut votes = HashMap::new();
        votes.insert(3, true);
        votes.insert(4, true);
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
        let state = GameState::Voting(Voting {
            mission: 1,
            proposal: 2,
            proposer: 3,
            players: vec![3, 2],
            votes: hashmap![
                1 => false,
                2 => true,
                3 => true,
                4 => false
            ]
        });
        let (next_state, effects) = state.on_vote(&game, 5, false);
        let mut votes = HashMap::new();
        votes.insert(3, true);
        votes.insert(4, true);
        assert_eq!(next_state, GameState::Proposing(Proposing {
            mission: 1,
            proposal: 3,
            proposer: 4,
        }));
        assert_eq!(effects, vec![
            Effect::Broadcast(Message::VotingResults {
                upvotes: vec![2, 3],
                downvotes: vec![1, 4, 5],
                sent: false
            }),
            Effect::Broadcast(Message::NextProposal { player: 4 })
        ]);
    }
}