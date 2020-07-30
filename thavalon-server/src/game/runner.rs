//! Asynchronous engine for running THavalon games

use std::collections::HashMap;

use log::{info, warn, error};
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
            info!("{} is {:?}", player.name, player.role);
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
                    return;
                }
            }
        }
    }

    /// Handles a single action by updating the GameState and handling any side-effects of the state transition.
    async fn on_action(&mut self, from: PlayerId, action: Action) -> Result<(), GameError> {
        let (next_state, effect) = self.state.take().expect("Missing state").on_action(&self.game, from, action);
        self.state = Some(next_state);
        match effect {
            Effect::Send(to, message) => self.send(to, message).await,
            Effect::Broadcast(_) => unimplemented!(),
        }
    }

    async fn send(&mut self, to: PlayerId, message: Message) -> Result<(), GameError> {
        match self.message_senders.get_mut(&to) {
            Some(tx) => tx.send(message).await.map_err(|_| GameError::PlayerUnavailable { id: to }),
            None => Err(GameError::PlayerUnavailable { id: to })
        }
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

enum Effect {
    Broadcast(Message),
    Send(PlayerId, Message)
}

impl Effect {
    /// Effect for sending `player` an error message.
    fn error_to<S: Into<String>>(player: PlayerId, error: S) -> Effect {
        Effect::Send(player, Message::Error(error.into()))
    }
}

enum GameState {
    Pregame,
    Proposing(Proposing),
    Voting(Voting),
    Mission,
    Assassination,
    Postgame
}

impl GameState {
    fn on_action(self, game: &Game, from: PlayerId, action: Action) -> (GameState, Effect) {
        match action {
            Action::Propose { players } => self.on_proposal(game, from, players),
            _ => unimplemented!()
        }
    }

    fn on_proposal(self, game: &Game, proposer: PlayerId, players: Vec<PlayerId>) -> (GameState, Effect) {
        match self {
            GameState::Proposing(prop) if prop.proposer != proposer => (GameState::Proposing(prop), Effect::error_to(proposer, "It's not your proposal")),
            GameState::Proposing(prop) => {
                let expected_size = game.spec.mission_sizes[prop.mission];
                if players.len() != expected_size {
                    let effect = Effect::error_to(proposer, format!("You proposed {} players, but mission {} needs {}", players.len(), prop.mission, expected_size));
                    (GameState::Proposing(prop), effect)
                } else {
                    // TODO: check no duplicate players
                    let voting = GameState::Voting(Voting {
                        mission: prop.mission,
                        proposal: prop.proposal,
                        proposer: prop.proposer,
                        players: players.clone()
                    });
                    (voting, Effect::Broadcast(Message::CommenceVoting { proposal: players }))
                }
            },
            st => (st, Effect::error_to(proposer, "You can't propose right now"))
        }
    }
}

/// Contents of GameState::Proposing
struct Proposing {
    /// Which mission this is for
    mission: usize,
    /// Which proposal number this is
    proposal: usize,
    /// Who's proposing the mission
    proposer: PlayerId,
}

struct Voting {
    mission: usize,
    proposal: usize,
    proposer: PlayerId,
    players: Vec<PlayerId>
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Can't reach player {}", id)]
    PlayerUnavailable { id: PlayerId }
}