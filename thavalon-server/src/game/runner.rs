//! Asynchronous engine for running THavalon games

use std::collections::HashMap;

use log::{info, warn, error};
use futures::future::{join_all, TryFutureExt};
use thiserror::Error;

use tokio::sync::mpsc;
use tokio::stream::{StreamMap, StreamExt};

use super::{Game, PlayerId, Card};
use super::state::{GameState, Effect};

pub struct GameRunner {
    control_rx: mpsc::Receiver<ControlRequest>,
    control_tx: mpsc::Sender<ControlResponse>,
    
    /// Receivers for messages from players
    actions: StreamMap<PlayerId, mpsc::Receiver<Action>>,

    /// Senders for each player 
    message_senders: HashMap<PlayerId, mpsc::Sender<Message>>,

    // Current game state. This is an Option because, while processing an action, we move the state out of GameRunner.
    // It's illegal to leave the state field invalid, so we replace it with None instead.
    //state: Option<GameState>,
}

type PlayerChannels = (mpsc::Sender<Action>, mpsc::Receiver<Message>);

impl GameRunner {
    /// Create a new GameRunner, returning it and its control channels
    pub fn new() -> (GameRunner, mpsc::Sender<ControlRequest>, mpsc::Receiver<ControlResponse>) {
        let (req_tx, req_rx) = mpsc::channel(1);
        let (resp_tx, resp_rx) = mpsc::channel(1);

        let runner = GameRunner {
            control_rx: req_rx,
            control_tx: resp_tx,

            actions: StreamMap::new(),
            message_senders: HashMap::new()
        };
        (runner, req_tx, resp_rx)
    }

    /// Asynchronously spawn a GameRunner, returning its control channels
    pub fn spawn() -> (mpsc::Sender<ControlRequest>, mpsc::Receiver<ControlResponse>) {
        let (mut game, req, resp) = GameRunner::new();
        tokio::spawn(async move {
            game.run().await;
        });
        (req, resp)
    }

    /// Main game loop. Does not return until the game is over or a fatal error occurs.
    pub async fn run(&mut self) {
        info!("Waiting for players to join...");

        let mut players = vec![];
        loop {
            match self.control_rx.next().await {
                Some(ControlRequest::AddPlayer { id, name }) => {
                    info!("{} joined as player #{}", name, id);
                    players.push((id, name));

                    let (action_tx, action_rx) = mpsc::channel(10);
                    self.actions.insert(id, action_rx);

                    let (message_tx, message_rx) = mpsc::channel(10);
                    self.message_senders.insert(id, message_tx);

                    if let Err(err) = self.control_tx.send(ControlResponse::PlayerAdded { id, actions: action_tx, messages: message_rx }).await {
                        players.pop();
                        self.actions.remove(&id);
                        self.message_senders.remove(&id);
                        error!("Could not send PlayerAdded notification for {}, removing player: {}", id, err);
                    }
                },
                Some(ControlRequest::StartGame) => break,
                None => {
                    error!("Control channel closed");
                    return;
                }
            }
        }

        info!("Starting game!");
        let game = Game::roll(players);
        if let Err(err) = self.control_tx.send(ControlResponse::GameStarted { num_players: game.size() }).await {
            error!("Could not send GameStarted notification: {}", err);
        }

        for player in game.players.iter() {
            info!("{} is {:?}\n{}", player.name, player.role, game.info[&player.id]);
        }

        let mut state = GameState::Pregame;

        loop {
            match self.actions.next().await {
                Some((player, action)) => {
                    let (next_state, effects) = state.on_action(&game, player, action);
                    state = next_state;
                    for effect in effects.into_iter() {
                        let result = match effect {
                            Effect::Send(to, message) => self.send(to, message).await,
                            Effect::Broadcast(message) => self.broadcast(message).await,            
                        };
                        if let Err(err) = result {
                            error!("Handling effect for action failed: {}", err);
                        }
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

/// Control request for the game runner (not gameplay-related, but for game setup/management)
#[derive(Debug)]
pub enum ControlRequest {
    /// Adds a player to the game
    AddPlayer { id: PlayerId, name: String },

    /// Signals for the game to start. Players cannot be added after this point.
    StartGame
}

/// Response to a control request
#[derive(Debug)]
pub enum ControlResponse {
    /// Response to a player being added, containing gameplay-related communication channels for that player.
    PlayerAdded { id: PlayerId, actions: mpsc::Sender<Action>, messages: mpsc::Receiver<Message> },

    /// Confirmation that the game has started.
    GameStarted { num_players: usize }
}

// Game-related messages

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

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Can't reach player {}", id)]
    PlayerUnavailable { id: PlayerId }
}

