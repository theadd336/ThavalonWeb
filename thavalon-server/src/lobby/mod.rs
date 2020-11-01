use crate::database::games::{DBGameError, DBGameStatus, DatabaseGame};
use crate::game::{builder::GameBuilder, Action, Message};

use thiserror::Error;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    task,
};

use std::collections::HashMap;

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
    UpdatePlayerConnection {
        player_id: String,
    },
    GetFriendCode,
}

#[derive(Debug)]
pub enum LobbyResponse {
    Standard(Result<(), LobbyError>),
    FriendCode(String),
}

pub struct PlayerClient {
    to_client: Option<mpsc::UnboundedSender<Result<String, warp::Error>>>,
    to_game: Sender<Action>,
    from_game: Receiver<Message>,
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
            to_client: None,
            to_game: sender,
            from_game: receiver,
        };

        self.players.insert(player_id.clone(), client);
        log::info!(
            "Successfully added player {} to game {}.",
            player_id,
            self.friend_code
        );
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
                LobbyCommand::UpdatePlayerConnection { player_id } => self.get_friend_code(),
            };

            result_channel
                .send(results)
                .expect("Could not send a result message back to caller.");
        }
    }
}
