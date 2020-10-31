use crate::database::games::DatabaseGame;

use std::collections::HashSet;

pub struct Lobby {
    database_game: DatabaseGame,
    players: HashSet<String>,
}

impl Lobby {
    pub async fn new() -> Lobby {
        Lobby {
            database_game: DatabaseGame::new().await.unwrap(),
            players: HashSet::new(),
        }
    }

    pub fn get_friend_code(&self) -> &String {
        self.database_game.get_friend_code()
    }
}
