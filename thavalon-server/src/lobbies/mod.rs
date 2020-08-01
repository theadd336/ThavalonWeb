use crate::game::{GameRunner, PlayerId};
use lazy_static::lazy_static;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;
use uuid::Uuid;

mod errors;

lazy_static! {
    static ref LOBBY_MANAGER: RwLock<HashMap<String, Lobby>> = RwLock::new(HashMap::new());
}

enum LobbyStatus {
    Open,
    InProgress,
    Done,
}

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

pub async fn create_new_lobby() -> Result<String, errors::LobbyError> {
    let lobbies = &mut LOBBY_MANAGER.write().unwrap();
    let lobby_id = Uuid::new_v4().to_string()[..6].to_string();
    info!("Created new lobby with ID: {}", &lobby_id);
    lobbies.insert(lobby_id.clone(), Lobby::new());
    return Ok(lobby_id);
}

pub async fn does_lobby_exist(lobby_id: &String) -> bool {
    return LOBBY_MANAGER.read().unwrap().contains_key(lobby_id);
}

pub async fn start_game(lobby_id: &str) {}

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
