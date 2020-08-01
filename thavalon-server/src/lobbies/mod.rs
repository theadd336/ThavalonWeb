//! Module containing structs and functions for lobby-related activities.
//! Handles lobby creation and destruction, game creation, lobby statuses, and logging.
//! This module may change drastically.

use crate::game::{GameRunner, PlayerId};
use lazy_static::lazy_static;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;
use uuid::Uuid;

mod errors;

lazy_static! {
    /// Instance of a thread-safe HashMap used for managing lobbies.
    static ref LOBBY_MANAGER: RwLock<HashMap<String, Lobby>> = RwLock::new(HashMap::new());
}

//#region Structs and Enums
/// Enum representing various lobby statuses
enum LobbyStatus {
    Open,
    InProgress,
    Done,
}

/// Represents a single lobby. A lobby will have a unique game, status, and players.
/// Creation time is used to track how long the lobby has been around and delete it if it is stale.
struct Lobby {
    game: Option<GameRunner>,
    status: LobbyStatus,
    creation_time: Instant,
    player_ids_to_name: HashMap<PlayerId, String>,
    player_names_to_id: HashMap<String, PlayerId>,
}

impl Lobby {
    pub fn new() -> Lobby {
        return Lobby {
            game: Option::None,
            status: LobbyStatus::Open,
            creation_time: Instant::now(),
            player_ids_to_name: HashMap::new(),
            player_names_to_id: HashMap::new(),
        };
    }
}

//#endregion

//#region Public Functions
/// Creates a new lobby for the game.
pub async fn create_new_lobby() -> Result<String, errors::LobbyError> {
    let lobbies = &mut LOBBY_MANAGER.write().unwrap();
    let lobby_id = Uuid::new_v4().to_string()[..6].to_string();
    info!("Created new lobby with ID: {}", &lobby_id);
    lobbies.insert(lobby_id.clone(), Lobby::new());
    return Ok(lobby_id);
}

/// Checks if a given lobby exists.
pub async fn does_lobby_exist(lobby_id: &String) -> bool {
    return LOBBY_MANAGER.read().unwrap().contains_key(lobby_id);
}

/// Instructs the lobby to start the game and changes statuses accordingly.
pub async fn start_game(lobby_id: &str) {}

//#endregion

//#region Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_create_new_lobby() {
        let mut runtime = Runtime::new().unwrap();
        let lobby_id = runtime.block_on(create_new_lobby()).unwrap();
        assert! {LOBBY_MANAGER.read().unwrap().contains_key(&lobby_id)};
    }
    #[test]
    fn test_does_lobby_exist() {
        let mut runtime = Runtime::new().unwrap();
        let lobby_id = runtime.block_on(create_new_lobby()).unwrap();
        // Positive test. Does the lobby exist.
        assert! {runtime.block_on(does_lobby_exist(&lobby_id))};

        // Negative test. Does the lobby not exist.
        assert! {!runtime.block_on(does_lobby_exist(&String::from(
            "Testing with a string that will fail."
        )))};
    }
}
//#endregion
