//! Module containing structs and functions for lobby-related activities.
//! Handles lobby creation and destruction, game creation, lobby statuses, and logging.
//! This module may change drastically.

use crate::game::{GameRunner, PlayerId};
use lazy_static::lazy_static;
use log::{error, info, warn};
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use std::time::Instant;
use uuid::Uuid;

mod errors;
use errors::LobbyError;

lazy_static! {
    /// Instance of a thread-safe HashMap used for managing lobbies.
    static ref LOBBY_MANAGER: RwLock<HashMap<String, Mutex<Lobby>>> = RwLock::new(HashMap::new());
}

//#region Structs and Enums
/// Enum representing various lobby statuses
#[derive(Eq, PartialEq, Debug)]
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
    player_ids_to_names: HashMap<PlayerId, String>,
    player_names_to_ids: HashMap<String, PlayerId>,
}

impl Lobby {
    pub fn new() -> Lobby {
        return Lobby {
            game: Option::None,
            status: LobbyStatus::Open,
            creation_time: Instant::now(),
            player_ids_to_names: HashMap::new(),
            player_names_to_ids: HashMap::new(),
        };
    }
}

//#endregion

//#region Public Functions
/// Creates a new lobby for the game.
pub async fn create_new_lobby() -> Result<String, LobbyError> {
    let lobbies = &mut LOBBY_MANAGER.write().unwrap();
    let lobby_id = Uuid::new_v4().to_string()[..6].to_string();
    info!("Created new lobby with ID: {}", &lobby_id);
    lobbies.insert(lobby_id.clone(), Mutex::new(Lobby::new()));
    return Ok(lobby_id);
}

/// Checks if a given lobby exists.
pub async fn does_lobby_exist(lobby_id: &String) -> bool {
    return LOBBY_MANAGER.read().unwrap().contains_key(lobby_id);
}

/// Instructs the lobby to start the game and changes statuses accordingly.
pub async fn start_game(lobby_id: &str) {}

///Adds a player to the lobby and assigns a player ID.
pub async fn add_player(lobby_id: &String, player_name: &String) -> Result<PlayerId, LobbyError> {
    // Attempt to look up the lobby and return an error if there is no lobby.
    if !does_lobby_exist(lobby_id).await {
        warn!("Lobby {} does not exist.", lobby_id);
        return Err(LobbyError::LobbyDoesNotExist {
            lobby_id: lobby_id.clone(),
        });
    }
    // Check if the player name is too long.
    if player_name.chars().count() > 30 {
        warn!("Player name exceeds 30 characters.");
        return Err(LobbyError::PlayerNameTooLong());
    }

    // Aquire a read lock on the outer lobby manager, as we aren't editing data.
    // Then get a mutex lock on the lobby, as we are adding data.
    let lock = LOBBY_MANAGER.read().unwrap();
    let mut lobby = lock[lobby_id].lock().unwrap();

    // Make sure the lobby is at a status that accepts players.
    // If not, return an error.
    if lobby.status != LobbyStatus::Open {
        warn!(
            "Lobby {} is at a status of {:?}. Lobby must be Open to add players.",
            lobby_id, lobby.status
        );
        return Err(LobbyError::InvalidLobbyStatus());
    }

    // Make sure that a player name isn't already in the lobby.
    // If it is, return an error.
    if lobby.player_names_to_ids.contains_key(player_name) {
        let conflicting_player_id = lobby.player_names_to_ids[player_name];
        warn!(
            "Player name: {} already belongs to player {}",
            player_name, conflicting_player_id
        );
        return Err(LobbyError::PlayerNameTaken {
            player_name: player_name.clone(),
        });
    }

    // Generate a new player ID.
    let mut rng = rand::thread_rng();
    let mut player_id = rng.gen();
    while lobby.player_ids_to_names.contains_key(&player_id) {
        player_id = rng.gen();
    }

    // Add the player ID and names to the lobby and log it.
    lobby
        .player_ids_to_names
        .insert(player_id, player_name.clone());
    lobby
        .player_names_to_ids
        .insert(player_name.clone(), player_id);
    info!(
        "Added player {} [{}] to lobby {}.",
        player_name, player_id, lobby_id
    );
    return Ok(player_id);
}
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

    /// Helper function that sets up a lobby and runtime for testing.
    fn setup_lobby_and_runtime() -> (String, Runtime) {
        let mut runtime = Runtime::new().unwrap();
        let lobby_id = runtime.block_on(create_new_lobby()).unwrap();
        return (lobby_id, runtime);
    }
    #[test]
    fn test_add_player_valid() {
        let (lobby_id, mut runtime) = setup_lobby_and_runtime();
        let player_id = runtime
            .block_on(add_player(&lobby_id, &String::from("Paul")))
            .unwrap();
        assert! {player_id > 0};
    }

    #[test]
    fn test_add_duplicate_player() {
        let (lobby_id, mut runtime) = setup_lobby_and_runtime();
        runtime
            .block_on(add_player(&lobby_id, &String::from("Paul")))
            .unwrap();
        let result = runtime.block_on(add_player(&lobby_id, &String::from("Paul")));
        assert_eq!(
            result,
            Err(LobbyError::PlayerNameTaken {
                player_name: String::from("Paul")
            })
        );
    }

    #[test]
    fn test_add_player_to_invalid_lobby() {
        let (_, mut runtime) = setup_lobby_and_runtime();
        let invalid_id = String::from("INVALID");
        let result = runtime.block_on(add_player(&invalid_id, &String::from("Paul")));
        assert_eq!(
            result,
            Err(LobbyError::LobbyDoesNotExist {
                lobby_id: invalid_id
            })
        );
    }

    #[test]
    fn test_add_player_too_long() {
        let (lobby_id, mut runtime) = setup_lobby_and_runtime();
        let invalid_name =
            String::from("thisisareallyreallyreallyreallyreallyreallyreallylongstring.");
        let result = runtime.block_on(add_player(&lobby_id, &invalid_name));
        assert_eq!(result, Err(LobbyError::PlayerNameTooLong()))
    }
}
//#endregion
