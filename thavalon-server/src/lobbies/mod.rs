use lazy_static::lazy_static;
use rand::Rng;
use std::collections::HashMap;
use std::sync::RwLock;

mod errors;

struct Lobby {}

struct LobbyManager {
    lobbies: RwLock<HashMap<u32, Lobby>>,
}

impl LobbyManager {
    pub fn new() -> LobbyManager {
        return LobbyManager {
            lobbies: RwLock::new(HashMap::new()),
        };
    }

    pub async fn create_lobby(&self) -> Result<u32, errors::LobbyError> {
        let mut rng = rand::thread_rng();
        let lobbies = &mut self.lobbies.write().unwrap();
        let mut lobby_id = rng.gen_range(100000u32, 999999u32);
        while lobbies.contains_key(&lobby_id) {
            lobby_id = rng.gen_range(100000u32, 999999u32);
        }
        lobbies.insert(lobby_id, Lobby {});
        return Ok(lobby_id);
    }

    pub async fn does_lobby_exist(&self, lobby_id: u32) -> bool {
        return self.lobbies.read().unwrap().contains_key(&lobby_id);
    }
}

pub async fn create_new_lobby() -> Result<u32, errors::LobbyError> {
    return LOBBY_MANAGER.create_lobby().await;
}

pub async fn does_lobby_exist(lobby_id: u32) -> bool {
    return LOBBY_MANAGER.does_lobby_exist(lobby_id).await;
}

lazy_static! {
    static ref LOBBY_MANAGER: LobbyManager = LobbyManager::new();
}

//#region Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_create_new_lobby() {
        let mut runtime = Runtime::new().unwrap();
        let lobby_id = runtime.block_on(create_new_lobby()).unwrap();
        assert!(lobby_id > 0);
    }
    #[test]
    fn test_does_lobby_exist() {
        let mut runtime = Runtime::new().unwrap();
        let lobby_id = runtime.block_on(create_new_lobby()).unwrap();
        assert!(runtime.block_on(does_lobby_exist(lobby_id)));
    }
}
//#endregion
