//! Module for all game lobby code. The Lobby represents the interface between
//! the actual game, the database, and player connections.

use crate::database::games::{DBGameError, DBGameStatus, DatabaseGame};
use crate::game::{builder::GameBuilder, Action, Message};
use crate::utils;

mod client;

use futures::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
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
    #[error("An unknown error occurred. See logs for details.")]
    UnknownError,
}

/// Enum of available commands to send to the lobby.
pub enum LobbyCommand {
    AddPlayer {
        player_id: String,
        display_name: String,
    },
    GetFriendCode,
    IsClientRegistered {
        client_id: String,
    },
    ConnectClientChannels {
        client_id: String,
        ws: WebSocket,
    },
}

/// Enum of possible responses from the lobby.
#[derive(Debug)]
pub enum LobbyResponse {
    Standard(Result<(), LobbyError>),
    JoinGame(Result<String, LobbyError>),
    FriendCode(String),
    IsClientRegistered(bool),
}

/// Represents connections to and from the game for an individual player.
#[derive(Debug)]
struct PlayerClient {
    to_game: Sender<Action>,
    from_game: Arc<Mutex<Receiver<Message>>>,
    to_client: Option<SplitSink<WebSocket, ws::Message>>,
}

#[derive(Deserialize)]
#[serde(tag = "message_type", content = "data")]
enum IncomingMessage {
    Ping,
    StartGame,
    GameCommand(Action),
}

#[derive(Serialize)]
#[serde(tag = "message_type", content = "data")]
enum OutgoingMessage {
    Pong(String),
    GameMessage(Message),
}

/// A lobby for an individual game. The Lobby acts as an interface between the
/// Thavalon game instance, the DatabaseGame which keeps the game state in sync
/// with the database, and all players connected to the game.
pub struct Lobby {
    database_game: DatabaseGame,
    friend_code: String,
    players: HashMap<String, PlayerClient>,
    client_ids: HashMap<String, String>,
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
                client_ids: HashMap::with_capacity(10),
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
        log::info!(
            "Attempting to add player {} to lobby {}.",
            player_id,
            self.friend_code
        );
        if self.status != DBGameStatus::Lobby {
            log::warn!(
                "Player {} attempted to join in-progress or finished game {}.",
                player_id,
                self.friend_code
            );
            return LobbyResponse::Standard(Err(LobbyError::InvalidStateError));
        }

        if self.players.contains_key(&player_id) {
            log::warn!(
                "Player {} is already in game {}.",
                player_id,
                self.friend_code
            );
            return LobbyResponse::Standard(Err(LobbyError::DuplicatePlayerError));
        }

        if let Err(e) = self
            .database_game
            .add_player(player_id.clone(), display_name.clone())
            .await
        {
            log::error!(
                "Error while adding player {} to game {}. {}",
                player_id,
                self.friend_code,
                e
            );
            let return_err = match e {
                DBGameError::UpdateError => Err(LobbyError::DatabaseError),
                DBGameError::InvalidStateError => Err(LobbyError::InvalidStateError),
                _ => {
                    log::error!("An unknown error occurred in game {}.", self.friend_code);
                    Err(LobbyError::UnknownError)
                }
            };

            return LobbyResponse::Standard(return_err);
        }
        let (sender, receiver) = self.builder.as_mut().unwrap().add_player(display_name);

        let client = PlayerClient {
            to_game: sender,
            from_game: Arc::new(Mutex::new(receiver)),
            to_client: None,
        };

        self.players.insert(player_id.clone(), client);
        let client_id = utils::generate_random_string(32, false);
        log::info!(
            "Successfully added player {} to game {} with unique client ID {}.",
            player_id,
            self.friend_code,
            client_id
        );

        // Insert strings after logging since this step can never fail.
        // Inserting here saves a string allocation for player_id.
        self.client_ids.insert(client_id.clone(), player_id);
        LobbyResponse::JoinGame(Ok(client_id))
    }

    /// Updates a player's connections to and from the game and to and from the
    /// client.
    async fn update_player_connections(
        &mut self,
        client_id: String,
        ws: WebSocket,
    ) -> LobbyResponse {
        log::info!("Updating connections for client {}.", client_id);

        // Get the client and create the four connection endpoints.
        let player_id = self.client_ids.get(&client_id).unwrap();
        log::info!("Found player ID {} for client ID {}.", player_id, client_id);

        let mut client = self.players.get_mut(player_id).unwrap();
        let (mut to_player_client, mut from_player_client) = ws.split();
        let to_client = Arc::new(Mutex::new(to_player_client));
        let mut to_game = client.to_game.clone();
        let from_game = client.from_game.clone();

        // Spawn a new task to listen for incoming commands from the player.
        let from_player_id = player_id.clone();
        let to_player_id = player_id.clone();
        let lobby_to_client = to_client.clone();
        let game_to_client = to_client.clone();
        tokio::task::spawn(async move {
            while let Some(msg) = from_player_client.next().await {
                let msg = match msg {
                    Ok(message) => message.to_str().unwrap().to_owned(),
                    Err(e) => {
                        // It's not entirely clear what an error here means, so
                        // probably best to just close the entire connection.
                        log::error!(
                            "An error occurred while waiting for messages from player {}. {}",
                            from_player_id,
                            e
                        );
                        break;
                    }
                };

                log::debug!("Received message {} from player {}.", msg, client_id);
                let msg: IncomingMessage = match serde_json::from_str(&msg) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::warn!(
                            "Could not deserialize message from player {}. Message: {}",
                            to_player_id,
                            msg
                        );
                        // No way to recover if we get bad JSON, so just kill
                        // the connection.
                        break;
                    }
                };

                match msg {
                    IncomingMessage::Ping => lobby_to_client
                        .lock()
                        .await
                        .send(ws::Message::text(
                            serde_json::to_string(&OutgoingMessage::Pong(String::from("Pong")))
                                .unwrap(),
                        ))
                        .await
                        .unwrap(),
                    IncomingMessage::GameCommand(action) => to_game.send(action).await.unwrap(),
                }
            }
        });

        // Spawn a new task to forward messages from the game to the player.
        // The lock may appear weird, but it is needed to get send + sync on
        // the channel. The channel will only be held by one person ever.
        tokio::task::spawn(async move {
            while let Some(msg) = from_game.lock().await.recv().await {
                let msg = serde_json::to_string(&msg).expect("Could not serialize game message.");
                let msg = ws::Message::text(msg);
                if let Err(e) = game_to_client.lock().await.send(msg).await {
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
            let (msg_contents, result_channel) = msg;
            let results = match msg_contents {
                LobbyCommand::AddPlayer {
                    player_id,
                    display_name,
                } => self.add_player(player_id, display_name).await,

                LobbyCommand::GetFriendCode => self.get_friend_code(),
                LobbyCommand::IsClientRegistered { client_id } => {
                    LobbyResponse::IsClientRegistered(self.client_ids.contains_key(&client_id))
                }
                LobbyCommand::ConnectClientChannels { client_id, ws } => {
                    self.update_player_connections(client_id, ws).await
                }
            };

            result_channel
                .send(results)
                .expect("Could not send a result message back to caller.");
        }
    }
}
