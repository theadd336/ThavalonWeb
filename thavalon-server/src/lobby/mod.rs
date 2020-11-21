//! Module for all game lobby code. The Lobby represents the interface between
//! the actual game, the database, and player connections.

use crate::database::games::{DBGameError, DBGameStatus, DatabaseGame};
use crate::game::{builder::GameBuilder, Action, Message};

use futures::{FutureExt, SinkExt, StreamExt};
use thiserror::Error;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot, Mutex,
    },
    task,
};
use warp::filters::ws::{self, WebSocket};

use std::collections::HashMap;
use std::sync::Arc;

/// Type repreesenting a channel to the lobby to issue commands.
pub type LobbyChannel = Sender<(LobbyCommand, ResponseChannel)>;

/// Type representing a oneshot sender back to the caller.
pub type ResponseChannel = oneshot::Sender<LobbyResponse>;

/// Enum of possible lobby-related errors.
#[derive(Debug, Error)]
pub enum LobbyError {
    #[error("An error occurred while updating the database.")]
    DatabaseError,
    #[error("The player is already in the game.")]
    DuplicatePlayerError,
    #[error("A lobby update was attemped in a state that doesn't permit this update.")]
    InvalidStateError,
}

/// Enum of available commands to send to the lobby.
pub enum LobbyCommand {
    AddPlayer {
        player_id: String,
        display_name: String,
    },
    GetFriendCode,
    IsPlayerRegistered {
        player_id: String,
    },
    ConnectClientChannels {
        player_id: String,
        ws: WebSocket,
    },
}

/// Enum of possible responses from the lobby.
#[derive(Debug)]
pub enum LobbyResponse {
    Standard(Result<(), LobbyError>),
    FriendCode(String),
    IsPlayerRegistered(bool),
}

/// Represents connections to and from the game for an individual player.
#[derive(Debug)]
pub struct PlayerClient {
    to_game: Sender<Action>,
    from_game: Arc<Mutex<Receiver<Message>>>,
}

/// A lobby for an individual game. The Lobby acts as an interface between the 
/// Thavalon game instance, the DatabaseGame which keeps the game state in sync
/// with the database, and all players connected to the game.
pub struct Lobby {
    database_game: DatabaseGame,
    friend_code: String,
    players: HashMap<String, PlayerClient>,
    status: DBGameStatus,
    builder: Option<GameBuilder>,
    receiver: Receiver<(LobbyCommand, ResponseChannel)>,
}

impl Lobby {
    /// Creates a new lobby instance on a separate Tokio thread.
    /// 
    /// # Returns
    ///
    /// * `LobbyChannel` to communicate with the lobby.
    pub async fn new() -> LobbyChannel {
        let (mut tx, rx) = mpsc::channel(10);

        task::spawn(async move {
            let database_game = DatabaseGame::new().await.unwrap();
            let friend_code = database_game.get_friend_code().clone();
            let lobby = Lobby {
                database_game,
                friend_code,
                players: HashMap::with_capacity(10),
                status: DBGameStatus::Lobby,
                builder: Some(GameBuilder::new()),
                receiver: rx,
            };
            lobby.listen().await
        });

        tx
    }

    /// Gets the friend code for the lobby in question.
    fn get_friend_code(&self) -> LobbyResponse {
        LobbyResponse::FriendCode(self.friend_code.clone())
    }

    /// Adds a player to the lobby and all associated games.
    async fn add_player(&mut self, player_id: String, display_name: String) -> LobbyResponse {
        if self.status != DBGameStatus::Lobby {
            log::warn!(
                "Player {} attempted to join in-progress of finished game {}.",
                player_id,
                self.friend_code
            );
            return LobbyResponse::Standard(Err(LobbyError::InvalidStateError));
        }

        if let Err(e) = self
            .database_game
            .add_player(player_id.clone(), display_name.clone())
            .await
        {
            log::error!("Error adding player to the DB game. {}.", e);
            return LobbyResponse::Standard(Err(LobbyError::InvalidStateError));
        }
        let (sender, receiver) = self.builder.as_mut().unwrap().add_player(display_name);
        let client = PlayerClient {
            to_game: sender,
            from_game: Arc::new(Mutex::new(receiver)),
        };

        self.players.insert(player_id.clone(), client);
        log::info!(
            "Successfully added player {} to game {}.",
            player_id,
            self.friend_code
        );
        LobbyResponse::Standard(Ok(()))
    }

    /// Updates a player's connections to and from the game and to and from the
    /// client.
    async fn update_player_connections(
        &mut self,
        player_id: String,
        ws: WebSocket,
    ) -> LobbyResponse {
        let client = self.players.get(&player_id).unwrap();
        let (mut to_player_client, mut from_player_client) = ws.split();
        let mut to_game = client.to_game.clone();
        let from_game = client.from_game.clone();
        tokio::task::spawn(async move {
            while let Some(msg) = from_player_client.next().await {
                let msg = msg.unwrap();
                let action: Action = serde_json::from_str(msg.to_str().unwrap()).unwrap();
                to_game.send(action).await;
            }
        });

        tokio::task::spawn(async move {
            while let Some(msg) = from_game.lock().await.recv().await {
                let msg = serde_json::to_string(&msg).expect("Could not serialize game message.");
                let msg = ws::Message::text(msg);
                if let Err(e) = to_player_client.send(msg).await {
                    log::warn!("Connection lost. {}", e);
                    break;
                }
            }
        });

        LobbyResponse::Standard(Ok(()))
    }

    /// Begins a loop for the lobby to listen for incoming commands.
    /// This function should only return when the game ends or when a fatal
    /// error occurs.
    async fn listen(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            let (msg_contents, mut result_channel) = msg;
            let results = match msg_contents {
                LobbyCommand::AddPlayer {
                    player_id,
                    display_name,
                } => self.add_player(player_id, display_name).await,

                LobbyCommand::GetFriendCode => self.get_friend_code(),
                LobbyCommand::IsPlayerRegistered { player_id } => {
                    LobbyResponse::IsPlayerRegistered(self.players.contains_key(&player_id))
                }
                LobbyCommand::ConnectClientChannels { player_id, ws } => {
                    self.update_player_connections(player_id, ws).await
                }
            };

            result_channel
                .send(results)
                .expect("Could not send a result message back to caller.");
        }
    }
}
