//! Module for all game-related REST endpoint handlers. This module also handles
//! all websocket related functions.

use crate::database::{accounts, games::DatabaseGame};
use crate::lobby::{Lobby, LobbyCommand, LobbyError, LobbyResponse, LobbyChannel};

use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};
use warp::{
    reject::{self, Reject},
    reply,
    ws::{WebSocket, Ws},
    Rejection, Reply,
};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Type used for a global GameCollection of all active games.
pub type GameCollection =
    Arc<Mutex<HashMap<String, Sender<(LobbyCommand, oneshot::Sender<LobbyResponse>)>>>>;

/// Serializeable response for a new game. Contains the friend code to join the game.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewGameResponse {
    friend_code: String,
}

/// Deserializeable request to join a specified game.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinGameRequest {
    friend_code: String,
    display_name: String,
}

/// Serializable response from the server to a player attempting to join a game
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinGameResponse {
    socket_url: String,
}

/// Rejection for when a player does not have a verified email address.
#[derive(Debug)]
pub struct UnverifiedEmailRejection;
impl Reject for UnverifiedEmailRejection {}

/// Rejection for when a player attempts to join a nonexistant game.
#[derive(Debug)]
pub struct NonexistentGameRejection;
impl Reject for NonexistentGameRejection {}

/// Creates a new game for the given player ID.
/// 
/// # Arguments
///
/// * `player_id` - The Player ID of the game creator.
/// * `game_collection` - The global store of active games.
///
/// # Returns
///
/// * `NewGameResponse` on success.
/// * `UnverifiedEmailRejection` if the player's email isn't verified.
pub async fn create_game(
    player_id: String,
    game_collection: GameCollection,
) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to create new game for player {}.", player_id);

    // Confirm email is verified
    // if !verify_email(&player_id).await {
    //     log::info!(
    //         "Player {} does not have a validated email. Game creation failed.",
    //         player_id
    //     );
    //     return Err(reject::custom(UnverifiedEmailRejection));
    // }

    // Verify that player is not in any games. Need an efficient way to do this somehow.
    // TODO: Implement a database check to confirm the player isn't in a game.

    // Create a new game and add the player.
    let mut lobby_channel = Lobby::new().await;
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    lobby_channel
        .send((LobbyCommand::GetFriendCode, oneshot_tx))
        .await;

    let friend_code = match oneshot_rx.await.unwrap() {
        LobbyResponse::FriendCode(code) => code,
        _ => {
            log::error!("Failed to receive friend code from new lobby.");
            return Err(warp::reject());
        }
    };

    game_collection
        .lock()
        .unwrap()
        .insert(friend_code.clone(), lobby_channel);

    let response = NewGameResponse { friend_code };
    Ok(reply::json(&response))
}

/// Adds a player to an existing game
///
/// # Arguments
///
/// * `info` - The info required to join the game.
/// * `player_id` - The ID of the joining player.
/// * `game_collection` - The global collection of active games.
///
/// # Returns
///
/// * `JoinGameResponse` on success
/// * `NonexistentGameRejection` if the game doesn't exist
pub async fn join_game(
    info: JoinGameRequest,
    player_id: String,
    game_collection: GameCollection,
) -> Result<impl Reply, Rejection> {
    log::info!("Player {} is joining game {}.", player_id, info.friend_code);

    let mut lobby_channel = match game_collection.lock().unwrap().get(&info.friend_code) {
        Some(channel) => channel.clone(),
        None => {
            log::warn!("Game {} does not exist.", info.friend_code);
            return Err(reject::custom(NonexistentGameRejection));
        }
    };

    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    lobby_channel
        .send((
            LobbyCommand::AddPlayer {
                player_id: player_id.clone(),
                display_name: info.display_name.clone(),
            },
            oneshot_tx,
        ))
        .await;

    match oneshot_rx.await.unwrap() {
        LobbyResponse::Standard(result) => {
            if let Err(e) = result {
                log::warn!("Failed to add player {} to game. {}.", player_id, e);
                return Err(warp::reject());
            }
        }
        _ => {
            log::error!("Failed to receive the expected LobbyResponse");
            return Err(warp::reject());
        }
    }

    log::info!("Successfully added player {} to game {}.", player_id, info.friend_code);
    let socket_url = String::from("ws://localhost:8001/api/ws/") + &info.friend_code;
    let response = JoinGameResponse { socket_url };
    Ok(reply::json(&response))
}


/// Handles the initial WS connection. Checks to confirm the player is registered.
/// If they are, will attempt to promote the WS connection and establish a new
/// thread. Otherwise, the connection is rejected. 
///
/// # Arguments
///
/// * `ws` - The unupgraded WS connection.
/// * `friend_code` - The friend code of the game the player is joining.
/// * `player_id` - The player ID connecting to the WS.
/// * `game_collection` - The global collection of active games.
///
/// # Returns
///
/// * Upgraded WS connection for Warp on success
/// * 
pub async fn connect_ws(
    ws: Ws,
    friend_code: String,
    player_id: String,
    game_collection: GameCollection,
) -> Result<impl Reply, Rejection> {
    let mut lobby_channel = match game_collection.lock().unwrap().get(&friend_code) {
        Some(channel) => channel.clone(),
        None => {
            log::error!("Attempted to connect to a non-existent game.");
            return Err(warp::reject());
        }
    };

    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    lobby_channel
        .send((
            LobbyCommand::IsPlayerRegistered {
                player_id: player_id.clone(),
            },
            oneshot_tx,
        ))
        .await;

    match oneshot_rx.await.unwrap() {
        LobbyResponse::IsPlayerRegistered(is_registered) => {
            if !is_registered {
                log::error!("This player is not registered for the game.");
                return Err(warp::reject());
            }
        }
        _ => {
            log::error!("Did not receive the expected lobby response.");
            return Err(warp::reject());
        }
    };

    Ok(ws.on_upgrade(move |socket| client_connection(socket, player_id, lobby_channel)))
}

/// Establishes connections with the player channels to the game and the existing
/// websocket.
///
/// # Arguments
///
/// * `socket` - The upgraded WebSocket connection
/// * `player_id` - The player ID connecting to the game.
/// * `lobby_channel` - The channel to the lobby.
async fn client_connection(
    socket: WebSocket,
    player_id: String,
    mut lobby_channel: LobbyChannel,
) {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    lobby_channel
        .send((
            LobbyCommand::ConnectClientChannels {
                player_id,
                ws: socket,
            },
            oneshot_tx,
        ))
        .await;

    match oneshot_rx.await.unwrap() {
        LobbyResponse::Standard(result) => {
            if let Err(e) = result {
                log::error!("Error while updating player channels. {}", e);
            }
        }
        _ => {
            log::error!("Error while updating player channels.");
        }
    }
}

// /// Helper function to check if a player's email is verified or not.
// ///
// /// # Arguments
// ///
// /// * `player_id` - The player ID to check
// ///
// /// # Returns
// ///
// /// * `true` if the email is verified, false otherwise
// async fn verify_email(player_id: &String) -> bool {
//     match accounts::load_user_by_id(player_id).await {
//         Ok(user) => user.email_verified,
//         Err(_) => false,
//     }
// }
