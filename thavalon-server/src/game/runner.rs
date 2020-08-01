//! Asynchronous engine for running THavalon games

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use log::{info, warn, error};
use futures::future::{join_all, TryFutureExt};
use thiserror::Error;

use tokio::sync::mpsc;
use tokio::stream::{StreamMap, StreamExt};

use super::{Game, PlayerId, Card};

pub struct GameRunner {
    game: Game,
    
    /// Receivers for messages from players
    actions: StreamMap<PlayerId, mpsc::Receiver<Action>>,

    /// Senders for each player 
    message_senders: HashMap<PlayerId, mpsc::Sender<Message>>,

    /// Current game state. This is an Option because, while processing an action, we move the state out of GameRunner.
    /// It's illegal to leave the state field invalid, so we replace it with None instead.
    state: Option<GameState>,
}

type PlayerChannels = (mpsc::Sender<Action>, mpsc::Receiver<Message>);

impl GameRunner {
    pub fn launch(players: Vec<(PlayerId, String)>) -> HashMap<PlayerId, PlayerChannels> {
        let game = Game::roll(players);
        let (mut runner, channels) = GameRunner::new(game);
        tokio::spawn(async move {
            runner.run().await
        });
        channels
    }

    pub fn new(game: Game) -> (GameRunner, HashMap<PlayerId, PlayerChannels>) {
        let mut actions = StreamMap::new();
        let mut message_senders = HashMap::new();
        let mut handles = HashMap::new();
        for player in game.players.iter() {
            let (action_tx, action_rx) = mpsc::channel(10);
            actions.insert(player.id, action_rx);

            let (message_tx, message_rx) = mpsc::channel(10);
            message_senders.insert(player.id, message_tx);

            handles.insert(player.id, (action_tx, message_rx));
        }

        (GameRunner { game, actions, message_senders, state: Some(GameState::Pregame) }, handles)
    }

    pub async fn run(&mut self) {
        info!("Starting game");

        for player in self.game.players.iter() {
            info!("{} is {:?}\n{}", player.name, player.role, self.game.info[&player.id]);
        }

        loop {
            match self.actions.next().await {
                Some((player, action)) => {
                    if let Err(e) = self.on_action(player, action).await {
                        error!("Handling action failed: {}", e);
                    }
                },
                None => {
                    warn!("All players disconnected!");
                    break;
                }
            }
        }

        info!("Game ended");
    }

    /// Handles a single action by updating the GameState and handling any side-effects of the state transition.
    async fn on_action(&mut self, from: PlayerId, action: Action) -> Result<(), GameError> {
        let (next_state, effects) = self.state.take().expect("Missing state").on_action(&self.game, from, action);
        self.state = Some(next_state);
        for effect in effects.into_iter() {
            match effect {
                Effect::Send(to, message) => self.send(to, message).await?,
                Effect::Broadcast(message) => self.broadcast(message).await?,
            }
        }
        Ok(())
    }

    /// Send a Message to a single player
    async fn send(&mut self, to: PlayerId, message: Message) -> Result<(), GameError> {
        match self.message_senders.get_mut(&to) {
            Some(tx) => tx.send(message).await.map_err(|_| GameError::PlayerUnavailable { id: to }),
            None => Err(GameError::PlayerUnavailable { id: to })
        }
    }

    /// Send a message to every player
    async fn broadcast(&mut self, message: Message) -> Result<(), GameError> {
        // This uses join_all to send messages in parallel, then collects the Vec<Result> into a single Result
        join_all(self.message_senders.iter_mut().map(|(&id, tx)| {
            tx.send(message.clone()).map_err(move |_| GameError::PlayerUnavailable { id })
        })).await.into_iter().collect()
    }
}

/// Something the player tries to do
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Action {
    Propose { players: Vec<PlayerId> },
    Vote { upvote: bool },
    Play { card: Card }
}

/// A message from the game to a player
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Message {
    Error(String),

    NextProposal { player: PlayerId },
    CommenceVoting { proposal: Vec<PlayerId> },
    VotingResults {
        upvotes: Vec<PlayerId>,
        downvotes: Vec<PlayerId>,
        sent: bool,
    },
    MissionResults {
        successes: usize,
        fails: usize,
        reverses: usize,
        passed: bool
    }
}

// GameState uses this state machine pattern: https://www.reddit.com/r/rust/comments/b32eca/state_machine_implementation_questions_or_to_mut/eix2hxr/
// Each transition consumes the current state and returns the new state and a side-effect. Consumption semantics are nice for state machines in Rust
// (see https://hoverbear.org/blog/rust-state-machine-pattern). Returning an Effect lets us send updates back to the players without having to worry
// about errors putting the state machine in an invalid state. Without this, it's hard to return a Result that doesn't leave us stateless on error.

#[derive(Debug, Eq, PartialEq)]
enum Effect {
    Broadcast(Message),
    Send(PlayerId, Message),
}

impl Effect {
    /// Effect for sending `player` an error message.
    fn error_to<S: Into<String>>(player: PlayerId, error: S) -> Effect {
        Effect::Send(player, Message::Error(error.into()))
    }
}

#[derive(Debug, Eq, PartialEq)]
enum GameState {
    Pregame,
    Proposing(Proposing),
    Voting(Voting),
    Mission,
    Assassination,
    Postgame
}

impl GameState {
    fn on_action(self, game: &Game, from: PlayerId, action: Action) -> (GameState, Vec<Effect>) {
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
struct Proposing {
    /// Which mission this is for
    mission: usize,
    /// Which proposal number this is
    proposal: usize,
    /// Who's proposing the mission
    proposer: PlayerId,
}

#[derive(Debug, Eq, PartialEq)]
struct Voting {
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

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Can't reach player {}", id)]
    PlayerUnavailable { id: PlayerId }
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