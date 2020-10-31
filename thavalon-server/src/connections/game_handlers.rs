use crate::database::{accounts, games::DatabaseGame};
use crate::lobby::Lobby;

use serde::{Deserialize, Serialize};
use warp::{
    reject::{self, Reject},
    reply, Rejection, Reply,
};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type GameCollection = Arc<Mutex<HashMap<String, Lobby>>>;

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
    let lobby = Lobby::new().await;
    let friend_code = lobby.get_friend_code().clone();
    game_collection
        .lock()
        .unwrap()
        .insert(friend_code.clone(), lobby);

    let response = NewGameResponse { friend_code };
    Ok(reply::json(&response))
}

pub async fn join_game(
    info: JoinGameRequest,
    player_id: String,
    game_collection: GameCollection,
) -> Result<impl Reply, Rejection> {
    log::info!("Player {} is joining game {}.", player_id, info.friend_code);
    let mut game = match game_collection.lock().unwrap().get(&info.friend_code) {
        Some(game) => game,
        None => {
            log::warn!("Game {} does not exist.", info.friend_code);
            return Err(warp::reject());
        }
    };

    game

    Ok(warp::reply())
}

/// Helper function to check if a player's email is verified or not.
///
/// # Arguments
///
/// * `player_id` - The player ID to check
///
/// # Returns
///
/// * `true` if the email is verified, false otherwise
async fn verify_email(player_id: &String) -> bool {
    match accounts::load_user_by_id(player_id).await {
        Ok(user) => user.email_verified,
        Err(_) => false,
    }
}
