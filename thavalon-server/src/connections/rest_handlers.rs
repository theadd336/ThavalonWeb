//! Module containing REST handlers for any REST endpoint.

use crate::lobbies;
use log::{error, info};
use serde::Serialize;
use serde_json;
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

//#endregion

/// Attempts to create a new lobby and logs the action.
pub async fn try_create_new_lobby() -> Result<impl Reply, Rejection> {
    match lobbies::create_new_lobby().await {
        Ok(lobby_id) => {
            log::info!("Sending lobby ID {} to client.", lobby_id);
            let response = CreateLobbyResponse::new(lobby_id);
            return Ok(serde_json::to_string(&response).unwrap());
        }
        Err(_) => {
            log::error!("Failed to create a lobby. Sending response code 500 to client.");
            Err(reject::custom(LobbyCreationFailed))
        }
    }
}
