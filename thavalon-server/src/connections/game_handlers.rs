use crate::database::accounts;

use serde::Deserialize;
use warp::{
    reject::{self, Reject},
    Rejection, Reply,
};

#[derive(Deserialize)]
pub struct PlayerDisplayName {
    display_name: String,
}

pub struct UnverifiedEmailRejection;
impl Reject for UnverifiedEmailRejection {}

pub async fn create_game(
    player_name: PlayerDisplayName,
    player_id: String,
) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to create new game for player {}.", player_id);

    // Confirm email is verified
    if !verify_email(&player_id).await {
        log::info!(
            "Player {} does not have a validated email. Game creation failed.",
            player_id
        );
        return Err(reject::custom(UnverifiedEmailRejection));
    }

    // Verify that player is not in any games. Need an efficient way to do this somehow.

    // Create a new game and add the player.
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
