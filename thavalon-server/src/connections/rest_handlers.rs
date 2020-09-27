//! Module containing REST handlers for any REST endpoint.

use crate::game::PlayerId;
use crate::lobbies;
use log::{error, info, warn};
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use warp::{reject, reject::Reject, Rejection, Reply};

//#region Structs and Implementations

/// Serializable response for when a lobby is successfully created.
/// Contains a lobby ID.
#[derive(Serialize)]
struct CreateLobbyResponse {
    lobby_id: String,
}

impl CreateLobbyResponse {
    /// Returns a new CreateLobbyResponse given a lobby ID.
    pub fn new(lobby_id: String) -> CreateLobbyResponse {
        CreateLobbyResponse { lobby_id }
    }
}

/// Serializable response for when lobby creation fails.test
/// This should probably never happen.
#[derive(Debug)]
struct LobbyCreationFailed;

impl Reject for LobbyCreationFailed {}

/// Serializable response for when a player joins a lobby.
/// Contains the player's ID and the websocket path.
#[derive(Serialize)]
struct AddPlayerResponse {
    player_id: PlayerId,
    websocket_path: String,
}

impl AddPlayerResponse {
    pub fn new(player_id: PlayerId, websocket_path: String) -> AddPlayerResponse {
        AddPlayerResponse {
            player_id,
            websocket_path,
        }
    }
}

/// Serializable response for an error while adding a player to the lobby.
/// This could happen for a variety of reasons, so an error message is included.
#[derive(Debug)]
struct AddPlayerFailed {
    error_message: String,
}

impl AddPlayerFailed {
    pub fn new(error_message: String) -> AddPlayerFailed {
        AddPlayerFailed { error_message }
    }
}

impl Reject for AddPlayerFailed {}
//#endregion

/// Attempts to create a new lobby and logs the action.
pub async fn try_create_new_lobby() -> Result<impl Reply, Rejection> {
    match lobbies::create_new_lobby().await {
        Ok(lobby_id) => {
            info!("Sending lobby ID {} to client.", lobby_id);
            let response = CreateLobbyResponse::new(lobby_id);
            return Ok(serde_json::to_string(&response).unwrap());
        }
        Err(_) => {
            error!("Failed to create a lobby. Sending response code 500 to client.");
            Err(reject::custom(LobbyCreationFailed))
        }
    }
}

/// Tries to add a player to a lobby. Will send back the player ID and the websocket path if success.
pub async fn try_join_lobby(
    connected_players: Arc<Mutex<HashMap<String, (String, PlayerId)>>>,
    lobby_id: String,
    player_name: String,
) -> Result<impl Reply, Rejection> {
    info!("Attempting to join lobby {}.", lobby_id);
    match lobbies::add_player(&lobby_id, &player_name).await {
        Ok(player_id) => {
            let websocket_path = Uuid::new_v4().to_string();
            connected_players
                .lock()
                .unwrap()
                .insert(websocket_path.clone(), (lobby_id, player_id));
            info!(
                "Sending player ID {} and websocket path {} to the client.",
                player_id, websocket_path
            );
            let response = AddPlayerResponse::new(player_id, websocket_path);
            return Ok(serde_json::to_string(&response).unwrap());
        }
        Err(error) => {
            let error_string = format!("{}", error);
            warn!("{}", error_string);
            let response = AddPlayerFailed::new(error_string);
            return Err(reject::custom(response));
        }
    }
}
