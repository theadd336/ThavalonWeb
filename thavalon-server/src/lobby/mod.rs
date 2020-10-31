use crate::database::games::{DBGameError, DBGameStatus, DatabaseGame};
use crate::game::builder::GameBuilder;

use std::collections::HashSet;

pub struct Lobby {
    database_game: DatabaseGame,
    players: HashSet<String>,
    status: DBGameStatus,
    builder: GameBuilder,
}

impl Lobby {
    pub async fn new() -> Lobby {
        Lobby {
            database_game: DatabaseGame::new().await.unwrap(),
            players: HashSet::new(),
            status: DBGameStatus::Lobby,
            builder: GameBuilder::
        }
    }

    pub fn get_friend_code(&self) -> &String {
        self.database_game.get_friend_code()
    }

    pub async fn add_player(
        &mut self,
        player_id: String,
        display_name: String,
    ) -> Result<String, DBGameError> {
        if self.status != DBGameStatus::Lobby {
            log::warn!(
                "Player {} attempted to join in-progress of finished game {}.",
                player_id,
                self.database_game.get_friend_code()
            );

            return Err(DBGameError::InvalidStateError);
        }

        self.database_game
            .add_player(player_id, display_name)
            .await?;

        Ok(self.get_friend_code().clone())
    }
}
