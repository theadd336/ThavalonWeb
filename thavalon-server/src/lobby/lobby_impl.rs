use super::client::{OutgoingMessage, PlayerClient};
use super::{LobbyChannel, LobbyCommand, LobbyError, LobbyResponse, ResponseChannel};
use crate::database::games::{DBGameError, DBGameStatus, DatabaseGame};
use crate::game::builder::GameBuilder;
use crate::utils;

use tokio::{
    sync::mpsc::{self, Receiver},
    task,
};
use warp::filters::ws::WebSocket;

use std::collections::HashMap;

/// A lobby for an individual game. The Lobby acts as an interface between the
/// Thavalon game instance, the DatabaseGame which keeps the game state in sync
/// with the database, and all players connected to the game.
pub struct Lobby {
    database_game: DatabaseGame,
    friend_code: String,
    player_ids_to_client_ids: HashMap<String, String>,
    client_ids_to_player_ids: HashMap<String, String>,
    clients: HashMap<String, PlayerClient>,
    status: DBGameStatus,
    builder: Option<GameBuilder>,
    to_lobby: LobbyChannel,
}

impl Lobby {
    /// Creates a new lobby instance on a separate Tokio thread.
    ///
    /// # Returns
    ///
    /// * `LobbyChannel` to communicate with the lobby.
    pub async fn new() -> LobbyChannel {
        let (tx, rx) = mpsc::channel(10);

        let to_lobby = tx.clone();
        task::spawn(async move {
            let database_game = DatabaseGame::new().await.unwrap();
            let friend_code = database_game.get_friend_code().clone();
            let lobby = Lobby {
                database_game,
                friend_code,
                player_ids_to_client_ids: HashMap::with_capacity(10),
                client_ids_to_player_ids: HashMap::with_capacity(10),
                clients: HashMap::with_capacity(10),
                status: DBGameStatus::Lobby,
                builder: Some(GameBuilder::new()),
                to_lobby,
            };
            lobby.listen(rx).await
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

        // First, sanity checks. Are we in the right status, and does the player exist already?
        if self.status != DBGameStatus::Lobby {
            log::warn!(
                "Player {} attempted to join in-progress or finished game {}.",
                player_id,
                self.friend_code
            );
            return LobbyResponse::Standard(Err(LobbyError::InvalidStateError));
        }

        // TODO: In the worst case, a player could lose the entire link to the game
        // and only have the friend code. We should support rejoining games from the
        // add_player path eventually.
        if self.player_ids_to_client_ids.contains_key(&player_id) {
            log::warn!(
                "Player {} is already in game {}.",
                player_id,
                self.friend_code
            );
            return LobbyResponse::Standard(Err(LobbyError::DuplicatePlayerError));
        }

        // The checks passed. Try adding the player into the game.
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

        // Player added to the database game. Now add the player to the game instance.
        let (sender, receiver) = self.builder.as_mut().unwrap().add_player(display_name);

        // Generate a unique client ID for the player and update all our dictionaries.
        let client_id = utils::generate_random_string(32, false);
        let client = PlayerClient::new(client_id.clone(), self.to_lobby.clone(), sender, receiver);
        log::info!(
            "Successfully added player {} to game {} with unique client ID {}.",
            player_id,
            self.friend_code,
            client_id
        );
        self.player_ids_to_client_ids
            .insert(player_id.clone(), client_id.clone());
        self.client_ids_to_player_ids
            .insert(client_id.clone(), player_id);
        self.clients.insert(client_id.clone(), client);
        LobbyResponse::JoinGame(Ok(client_id))
    }

    /// Removes a player from the lobby and game.
    async fn remove_player(&mut self, client_id: String) {
        log::info!(
            "Removing client {} from game {}.",
            client_id,
            self.friend_code
        );
        let player_id = match self.client_ids_to_player_ids.remove(&client_id) {
            Some(player_id) => player_id,
            None => {
                log::warn!("No player ID found matching client ID {}.", client_id);
                return;
            }
        };

        log::info!(
            "Found player {} for client {}. Removing player.",
            player_id,
            client_id
        );

        let display_name = self.database_game.remove_player(&player_id).await.unwrap();
        if display_name == None {
            log::warn!("No player display name found for player {}.", player_id);
            return;
        }
        let display_name = display_name.unwrap();
        self.builder.as_mut().unwrap().remove_player(&display_name);
        self.player_ids_to_client_ids.remove(&player_id);
        self.clients.remove(&client_id);
        self.on_player_list_change().await;
        log::info!("Successfully removed player {} from the game.", player_id);
    }

    /// Updates a player's connections to and from the game and to and from the
    /// client.
    async fn update_player_connections(
        &mut self,
        client_id: String,
        ws: WebSocket,
    ) -> LobbyResponse {
        log::info!("Updating connections for client {}.", client_id);
        let client = match self.clients.get_mut(&client_id) {
            Some(client) => client,
            None => {
                log::warn!(
                    "Client {} tried to connect to lobby {} but is not registered",
                    client_id,
                    self.friend_code
                );
                let _ = ws.close().await;
                return LobbyResponse::Standard(Err(LobbyError::InvalidClientID));
            }
        };

        client.update_websocket(ws).await;
        self.on_player_list_change().await;
        LobbyResponse::Standard(Ok(()))
    }

    /// Sends a pong back to the client that requested it.
    async fn send_pong(&mut self, client_id: String) -> LobbyResponse {
        let client = match self.clients.get_mut(&client_id) {
            Some(client) => client,
            None => {
                log::error!("Client {} does not exist. Cannot send Pong.", client_id);
                return LobbyResponse::Standard(Err(LobbyError::InvalidClientID));
            }
        };
        let message = OutgoingMessage::Pong("Pong".to_string());
        let message = serde_json::to_string(&message).unwrap();
        client.send_message(message).await;
        LobbyResponse::Standard(Ok(()))
    }

    /// Handles a change to the player list, due to a player joining or leaving the game.
    async fn on_player_list_change(&mut self) {
        // Only broadcast to players if the game hasn't started yet.
        // TODO: Maybe a helpful message to players that someone has disconnected.
        // Would need to have a way to lookup name by the disconnected client ID.
        if self.status == DBGameStatus::Lobby {
            let current_players = self.builder.as_ref().unwrap().get_player_list();
            self.broadcast_message(&OutgoingMessage::PlayerList(current_players.to_vec()))
                .await;
        }
    }

    // Handles dealing with a disconnected player.
    // If the lobby isn't in progress or done, a disconnect should remove the player.
    // Otherwise, nothing happens.
    async fn on_player_disconnect(&mut self, client_id: String) -> LobbyResponse {
        log::info!(
            "Client {} has disconnected from game {}.",
            client_id,
            self.friend_code
        );

        // If we're in the lobby phase, a disconnect counts as leaving the game.
        if self.status == DBGameStatus::Lobby {
            self.remove_player(client_id).await;
        }

        LobbyResponse::Standard(Ok(()))
    }

    /// Starts the game and updates statuses
    async fn start_game(&mut self) -> LobbyResponse {
        // The only thing that can fail is updating the database. In this case,
        // the lobby is probably dead, so panic to blow up everything.
        if let Err(e) = self.database_game.start_game().await {
            log::error!("Error while starting game {}. {}", self.friend_code, e);
            panic!();
        }

        // Tell the players the game is about to start to move to the game page.
        self.broadcast_message(&OutgoingMessage::StartGame).await;
        self.status = DBGameStatus::InProgress;
        let builder = self.builder.take().unwrap();
        builder.start();
        return LobbyResponse::Standard(Ok(()));
    }

    /// Broadcasts a message to all clients in the lobby.
    async fn broadcast_message(&mut self, message: &OutgoingMessage) {
        let message = serde_json::to_string(&message).unwrap();
        for client in self.clients.values_mut() {
            client.send_message(message.clone()).await;
        }
    }

    /// Begins a loop for the lobby to listen for incoming commands.
    /// This function should only return when the game ends or when a fatal
    /// error occurs.
    async fn listen(mut self, mut receiver: Receiver<(LobbyCommand, Option<ResponseChannel>)>) {
        while let Some(msg) = receiver.recv().await {
            let (msg_contents, result_channel) = msg;
            let results = match msg_contents {
                LobbyCommand::AddPlayer {
                    player_id,
                    display_name,
                } => self.add_player(player_id, display_name).await,

                LobbyCommand::GetFriendCode => self.get_friend_code(),
                LobbyCommand::IsClientRegistered { client_id } => {
                    LobbyResponse::IsClientRegistered(self.clients.contains_key(&client_id))
                }
                LobbyCommand::ConnectClientChannels { client_id, ws } => {
                    self.update_player_connections(client_id, ws).await
                }
                LobbyCommand::Ping { client_id } => self.send_pong(client_id).await,
                LobbyCommand::StartGame => self.start_game().await,
                LobbyCommand::PlayerDisconnect { client_id } => {
                    self.on_player_disconnect(client_id).await
                }
            };

            if let Some(channel) = result_channel {
                channel
                    .send(results)
                    .expect("Could not send a result message back to caller.");
            }
        }
    }
}
