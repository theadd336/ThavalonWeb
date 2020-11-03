use crate::database::{accounts, games::DatabaseGame};
use crate::lobby::{Lobby, LobbyCommand, LobbyError, LobbyResponse};

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

pub type GameCollection =
    Arc<Mutex<HashMap<String, Sender<(LobbyCommand, oneshot::Sender<LobbyResponse>)>>>>;

#[derive(Deserialize)]
pub struct PlayerDisplayName {
    display_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewGameResponse {
    friend_code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinGameRequest {
    friend_code: String,
    display_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinGameResponse {
    socket_url: String,
}

#[derive(Debug)]
pub struct UnverifiedEmailRejection;
impl Reject for UnverifiedEmailRejection {}

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
            return Err(warp::reject());
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

    let socket_url = String::from("ws://localhost:8001/api/ws/") + &info.friend_code;
    let response = JoinGameResponse { socket_url };
    Ok(reply::json(&response))
}

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

async fn client_connection(
    socket: WebSocket,
    player_id: String,
    mut lobby_channel: Sender<(LobbyCommand, oneshot::Sender<LobbyResponse>)>,
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
                return;
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
