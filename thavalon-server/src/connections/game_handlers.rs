//! Module for all game-related REST endpoint handlers. This module also handles
//! all websocket related functions.

use crate::database::accounts;
use crate::lobby::{Lobby, LobbyChannel, LobbyCommand, LobbyError, LobbyResponse};

use serde::{Deserialize, Serialize};
use tokio::{
    sync::oneshot,
    time::{Duration, Instant},
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
pub type GameCollection = Arc<Mutex<HashMap<String, LobbyChannel>>>;

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
    let (end_game_tx, end_game_rx) = oneshot::channel();
    let mut lobby_channel = Lobby::new(end_game_tx).await;
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    // TODO: Error handling here.
    let _ = lobby_channel
        .send((LobbyCommand::GetFriendCode, Some(oneshot_tx)))
        .await;

    let friend_code = match oneshot_rx.await.unwrap() {
        LobbyResponse::FriendCode(code) => code,
        _ => {
            panic!("Failed to receive friend code from new lobby.");
        }
    };

    let monitor_lobby_channel = lobby_channel.clone();
    let monitor_friend_code = friend_code.clone();
    let monitor_game_collection = game_collection.clone();

    game_collection
        .lock()
        .unwrap()
        .insert(friend_code.clone(), lobby_channel);

    // Spawn a thread to monitor this lobby and remove it from game_collection when it's over or timed out.
    tokio::spawn(monitor_lobby_task(
        monitor_lobby_channel,
        end_game_rx,
        monitor_friend_code,
        monitor_game_collection,
    ));

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

    // TODO: Figure out if this needs error handling.
    // Can't use .unwrap() here since SendError doesn't implement Debug
    let _ = lobby_channel
        .send((
            LobbyCommand::AddPlayer {
                player_id: player_id.clone(),
                display_name: info.display_name.clone(),
            },
            Some(oneshot_tx),
        ))
        .await;

    let client_id = match oneshot_rx.await.unwrap() {
        LobbyResponse::JoinGame(result) => match result {
            Ok(client_id) => client_id,
            Err(e) => {
                log::warn!("Failed to add player {} to game. {}.", player_id, e);
                return Err(warp::reject());
            }
        },
        _ => {
            panic!("Failed to receive the expected LobbyResponse");
        }
    };

    log::info!(
        "Successfully added player {} to game {}.",
        player_id,
        info.friend_code
    );
    let socket_url = format!(
        "ws://localhost:8001/api/ws/{}/{}",
        info.friend_code, client_id
    );
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
/// * `client_id` - The client ID connecting to the WS.
/// * `game_collection` - The global collection of active games.
///
/// # Returns
///
/// * Upgraded WS connection for Warp on success
/// *
pub async fn connect_ws(
    ws: Ws,
    friend_code: String,
    client_id: String,
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

    // TODO: Error handling here.
    let _ = lobby_channel
        .send((
            LobbyCommand::IsClientRegistered {
                client_id: client_id.clone(),
            },
            Some(oneshot_tx),
        ))
        .await;

    match oneshot_rx.await.unwrap() {
        LobbyResponse::IsClientRegistered(is_registered) => {
            if !is_registered {
                log::error!("This player is not registered for the game.");
                return Err(warp::reject());
            }
        }
        _ => {
            panic!("Did not receive the expected lobby response.");
        }
    };

    Ok(ws.on_upgrade(move |socket| client_connection(socket, client_id, lobby_channel)))
}

/// Establishes connections with the player channels to the game and the existing
/// websocket.
///
/// # Arguments
///
/// * `socket` - The upgraded WebSocket connection
/// * `client_id` - The client ID connecting to the game.
/// * `lobby_channel` - The channel to the lobby.
async fn client_connection(socket: WebSocket, client_id: String, mut lobby_channel: LobbyChannel) {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    // TODO: Error handling may be needed here.
    let _ = lobby_channel
        .send((
            LobbyCommand::ConnectClientChannels {
                client_id,
                ws: socket,
            },
            Some(oneshot_tx),
        ))
        .await;

    match oneshot_rx.await.unwrap() {
        LobbyResponse::Standard(result) => {
            if let Err(e) = result {
                log::error!("Error while updating player channels. {}", e);
            }
        }
        _ => {
            panic!("Error while updating player channels.");
        }
    }
}

/// Helper function for monitoring a lobby, intended to run as a tokio task. This will remove the lobby from
/// GameCollection once the lobby ends or exceeds the maximum lobby lifetime.
async fn monitor_lobby_task(
    mut lobby_channel: LobbyChannel,
    mut end_game_rx: oneshot::Receiver<bool>,
    friend_code: String,
    game_collection: GameCollection,
) {
    // Lobby timeout is 6 hours from creation across all phases.
    let timeout = tokio::time::delay_until(Instant::now() + Duration::from_secs(60 * 60 * 6));
    tokio::select! {
        _ = timeout => {
            log::error!("Lobby {} has exceeded timeout, killing this lobby now.", &friend_code);
            lobby_channel.send((LobbyCommand::EndGame, None)).await;
        }
        _ = end_game_rx => {
            log::info!("Lobby {} completed, removing it from game collection.", &friend_code);
        }
    }
    game_collection.lock().unwrap().remove(&friend_code);
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
