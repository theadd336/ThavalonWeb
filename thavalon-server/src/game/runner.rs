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

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Can't reach player {}", id)]
    PlayerUnavailable { id: PlayerId }
}

