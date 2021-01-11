use super::client::PlayerClient;
use super::{IncomingMessage, LobbyState, OutgoingMessage};
use super::{LobbyChannel, LobbyCommand, LobbyError, LobbyResponse, ResponseChannel};
use crate::database::games::{DBGameError, DBGameStatus, DatabaseGame};
use crate::game::{
    builder::GameBuilder,
    snapshot::{GameSnapshot, Snapshots},
};
use crate::utils;

use futures::future::AbortHandle;
use tokio::{
    sync::mpsc::{self, Receiver},
    sync::oneshot,
    task,
};
use warp::filters::ws::WebSocket;

use std::collections::HashMap;

const MAX_NUM_PLAYERS: usize = 10;

/// A lobby for an individual game. The Lobby acts as an interface between the
/// Thavalon game instance, the DatabaseGame which keeps the game state in sync
/// with the database, and all players connected to the game.
pub struct Lobby {
    game_over: bool,
    game_over_channel: Option<oneshot::Sender<bool>>,
    database_game: DatabaseGame,
    friend_code: String,
    player_ids_to_client_ids: HashMap<String, String>,
    // Map of client IDs to player ID and display name.
    client_ids_to_player_info: HashMap<String, (String, String)>,
    clients: HashMap<String, PlayerClient>,
    status: LobbyState,
    builder: Option<GameBuilder>,
    snapshots: Option<Snapshots>,
    game_abort_handle: Option<AbortHandle>,
    to_lobby: LobbyChannel,
}

impl Lobby {
    /// Creates a new lobby instance on a separate Tokio thread.
    ///
    /// # Arguments
    ///
    /// * `end_game_channel` A channel this lobby should publish to when it's finished running.
    ///
    /// # Returns
    ///
    /// * `LobbyChannel` A channel for sending messages to this lobby.
    pub async fn new(game_over_channel: oneshot::Sender<bool>) -> LobbyChannel {
        let (tx, rx) = mpsc::channel(10);

        let to_lobby = tx.clone();
        task::spawn(async move {
            let database_game = DatabaseGame::new().await.unwrap();
            let friend_code = database_game.get_friend_code().clone();
            let lobby = Lobby {
                game_over: false,
                game_over_channel: Some(game_over_channel),
                database_game,
                friend_code,
                player_ids_to_client_ids: HashMap::with_capacity(MAX_NUM_PLAYERS),
                client_ids_to_player_info: HashMap::with_capacity(MAX_NUM_PLAYERS),
                clients: HashMap::with_capacity(MAX_NUM_PLAYERS),
                status: LobbyState::Lobby,
                builder: Some(GameBuilder::new()),
                snapshots: None,
                game_abort_handle: None,
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

        // First, check if this player is already in game. If so, this is a reconnect. Otherwise,
        // this is a new player.
        if self.player_ids_to_client_ids.contains_key(&player_id) {
            return self.reconnect_player(&player_id, &display_name);
        }

        // Unlike reconnecting, new players may only join when the game is in Lobby.
        if self.status != LobbyState::Lobby {
            log::warn!(
                "Player {} attempted to join in-progress or finished game {}.",
                player_id,
                self.friend_code
            );
            return LobbyResponse::Standard(Err(LobbyError::InvalidStateError));
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
                DBGameError::DuplicateDisplayName => Err(LobbyError::DuplicateDisplayName),
                _ => {
                    log::error!("An unknown error occurred in game {}.", self.friend_code);
                    Err(LobbyError::UnknownError)
                }
            };

            return LobbyResponse::Standard(return_err);
        }

        // Player added to the database game. Now add the player to the game instance.
        let (sender, receiver) = self
            .builder
            .as_mut()
            .unwrap()
            .add_player(display_name.clone());

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
        self.client_ids_to_player_info
            .insert(client_id.clone(), (player_id, display_name));
        self.clients.insert(client_id.clone(), client);
        LobbyResponse::JoinGame(Ok(client_id))
    }

    /// Reconnect a player to an existing game in progress. Helper for add_player.
    fn reconnect_player(&self, player_id: &str, display_name: &str) -> LobbyResponse {
        log::info!(
            "Player {} is already in game {}, reconnecting.",
            player_id,
            self.friend_code
        );
        // Reconnecting can only happen for games in progress, as lobbies just kick disconnected players
        // and finished games are finished.
        if self.status != LobbyState::Game {
            log::warn!(
                "Player {} attempted to join game not in progress {}.",
                player_id,
                self.friend_code
            );
            return LobbyResponse::Standard(Err(LobbyError::InvalidStateError));
        }
        let client_id = self.player_ids_to_client_ids.get(player_id).unwrap().clone();
        let existing_display_name = &self.client_ids_to_player_info.get(&client_id).unwrap().1;
        if existing_display_name != &display_name {
            log::warn!(
                "Player {} attempted to reconnect with display name {}, but previously had display name {}.",
                player_id,
                display_name,
                existing_display_name);
            return LobbyResponse::Standard(Err(LobbyError::NameChangeOnReconnectError));
        }
        return LobbyResponse::JoinGame(Ok(client_id));
    }

    /// Removes a player from the lobby and game.
    async fn remove_player(&mut self, client_id: String) {
        log::info!(
            "Removing client {} from game {}.",
            client_id,
            self.friend_code
        );
        let player_id = match self.client_ids_to_player_info.remove(&client_id) {
            Some((player_id, _)) => player_id,
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
        if self.status == LobbyState::Lobby {
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
        if self.status == LobbyState::Lobby {
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

        let builder = self.builder.take().unwrap();
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        self.game_abort_handle = Some(abort_handle);
        match builder.start(self.to_lobby.clone(), abort_registration) {
            Ok((snapshots, _)) => {
                self.snapshots = Some(snapshots);
                // Tell the players the game is about to start to move to the game page.
                self.broadcast_message(&OutgoingMessage::LobbyState(LobbyState::Game))
                    .await;
                self.status = LobbyState::Game;
                LobbyResponse::None
            }
            Err(err) => {
                // Starting the game can fail, for example if there are too many players in the lobby
                // Since that isn't necessarily a fatal error, don't close the lobby
                log::error!("Error creating game {}: {}", self.friend_code, err);
                LobbyResponse::Standard(Err(LobbyError::InvalidStateError))
            }
        }
    }

    // End the lobby, including ending the database game and aborting the game thread.
    async fn end_game(&mut self) -> LobbyResponse {
        self.game_over = true;
        self.database_game.end_game().await.expect("Failed to end database game!");
        // game_abort_handle is None if the game has not been started. In that case, do nothing to end it.
        if let Some(handle) = self.game_abort_handle.take() { handle.abort() }
        self.game_over_channel.take().unwrap().send(true).expect("Failed to notify lobby manager!");
        LobbyResponse::None
    }

    /// Sends the current player list to the client.
    async fn send_player_list(&mut self, client_id: String) -> LobbyResponse {
        let mut client = self.clients.get_mut(&client_id).unwrap();
        let player_list = self.builder.as_ref().unwrap().get_player_list().to_vec();
        let player_list = OutgoingMessage::PlayerList(player_list);
        let player_list = serde_json::to_string(&player_list).unwrap();
        client.send_message(player_list).await;
        LobbyResponse::None
    }

    /// Sends the current state of the lobby to the client.
    async fn send_current_state(&mut self, client_id: String) -> LobbyResponse {
        let mut client = self.clients.get_mut(&client_id).unwrap();
        let state = OutgoingMessage::LobbyState(self.status.clone());
        let message = serde_json::to_string(&state).unwrap();
        client.send_message(message).await;
        LobbyResponse::None
    }

    /// Gets all snapshots that have occurred for a given client ID.
    async fn get_snapshots(&mut self, client_id: String) -> LobbyResponse {
        let (_, display_name) = &self.client_ids_to_player_info[&client_id];
        let snapshot = self
            .snapshots
            .as_ref()
            .unwrap()
            .get(display_name)
            .unwrap()
            .lock()
            .unwrap()
            .clone();
        let mut client = self.clients.get_mut(&client_id).unwrap();
        let message = OutgoingMessage::Snapshot(snapshot);
        let message = serde_json::to_string(&message).unwrap();
        client.send_message(message).await;
        LobbyResponse::None
    }

    /// Handles a player focus change event by telling all clients that a player's
    /// visibility has changed.
    async fn player_focus_changed(
        &mut self,
        client_id: String,
        is_tabbed_out: bool,
    ) -> LobbyResponse {
        let (_, display_name) = &self.client_ids_to_player_info[&client_id];
        let display_name = display_name.clone();
        let message = OutgoingMessage::PlayerFocusChange {
            displayName: display_name,
            isTabbedOut: is_tabbed_out,
        };
        self.broadcast_message(&message).await;
        LobbyResponse::None
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
            if self.game_over {
                break;
            }
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
                LobbyCommand::GetLobbyState { client_id } => {
                    self.send_current_state(client_id).await
                }
                LobbyCommand::StartGame => self.start_game().await,
                LobbyCommand::EndGame => self.end_game().await,
                LobbyCommand::PlayerDisconnect { client_id } => {
                    self.on_player_disconnect(client_id).await
                }
                LobbyCommand::GetPlayerList { client_id } => self.send_player_list(client_id).await,
                LobbyCommand::GetSnapshots { client_id } => self.get_snapshots(client_id).await,
                LobbyCommand::PlayerFocusChange {
                    client_id,
                    is_tabbed_out,
                } => self.player_focus_changed(client_id, is_tabbed_out).await,
                LobbyCommand::PollLobby => LobbyResponse::None,
            };

            if let Some(channel) = result_channel {
                channel
                    .send(results)
                    .expect("Could not send a result message back to caller.");
            }
        }
    }
}
