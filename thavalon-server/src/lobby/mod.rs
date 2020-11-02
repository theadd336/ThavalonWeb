use crate::database::games::{DBGameError, DBGameStatus, DatabaseGame};
use crate::game::{builder::GameBuilder, Action, Message};

use futures::{FutureExt, SinkExt, StreamExt};
use thiserror::Error;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    task,
};
use warp::filters::ws::WebSocket;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ResponseChannel = oneshot::Sender<LobbyResponse>;

#[derive(Debug, Error)]
pub enum LobbyError {
    #[error("An error occurred while updating the database.")]
    DatabaseError,
    #[error("The player is already in the game.")]
    DuplicatePlayerError,
    #[error("A lobby update was attemped in a state that doesn't permit this update.")]
    InvalidStateError,
}

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

#[derive(Debug)]
pub enum LobbyResponse {
    Standard(Result<(), LobbyError>),
    FriendCode(String),
    IsPlayerRegistered(bool),
}

#[derive(Debug)]
pub struct PlayerClient {
    to_game: Sender<Action>,
    from_game: Arc<Mutex<Receiver<Message>>>,
}

pub struct Lobby {
    database_game: DatabaseGame,
    friend_code: String,
    players: HashMap<String, PlayerClient>,
    status: DBGameStatus,
    builder: Option<GameBuilder>,
    receiver: Receiver<(LobbyCommand, ResponseChannel)>,
}

impl Lobby {
    pub async fn new() -> Sender<(LobbyCommand, ResponseChannel)> {
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

    pub fn get_friend_code(&self) -> LobbyResponse {
        LobbyResponse::FriendCode(self.friend_code.clone())
    }

    pub async fn add_player(&mut self, player_id: String, display_name: String) -> LobbyResponse {
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

    pub async fn update_player_connections(
        &mut self,
        player_id: String,
        ws: WebSocket,
    ) -> LobbyResponse {
        let client = self.players.get(&player_id).unwrap();
        let (to_player_client, from_player_client) = ws.split();
        let to_game = client.to_game.clone();
        let from_game = client.from_game.clone();
        tokio::task::spawn(async move {
            from_player_client.forward(to_game);
        });

        tokio::task::spawn(async move {
            while let Some(msg) = from_game.lock().unwrap().recv().await {
                if let Err(e) = to_player_client.send(msg).await {
                    log::warn!("Connection lost. {}", e);
                    break;
                }
            }
        });

        LobbyResponse::Standard(Ok(()))
    }

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
