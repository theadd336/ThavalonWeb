//! Game collection related functions and structs
use crate::database::get_database;
use crate::utils;

use std::collections::HashSet;

use async_trait::async_trait;
use chrono::Utc;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    error::Error,
    options::{InsertOneOptions, ReplaceOptions, UpdateModifications, UpdateOptions},
    results::{InsertOneResult, UpdateResult},
    Collection,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const GAME_COLLECTION: &str = "thavalon_games";
const FRIEND_CODE_LENGTH: usize = 4;

/// Contains errors related to database games.
#[derive(PartialEq, Error, Debug)]
pub enum DBGameError {
    #[error("The game could not be created.")]
    CreationError,
    #[error("An error occurred while updating the game in the database.")]
    UpdateError,
    #[error("Invalid state for the requested update.")]
    InvalidStateError,
}

/// Enum representing the three possible states of a game in the DB.
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum DBGameStatus {
    Lobby,
    InProgress,
    Finished,
}

/// Struct representing a single database game.
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseGame {
    _id: ObjectId,
    friend_code: String,
    players: HashSet<String>,
    display_names: HashSet<String>,
    status: DBGameStatus,
    created_time: i64,
    start_time: Option<i64>,
    end_time: Option<i64>,
    snapshot_id: Option<String>,
}

impl DatabaseGame {
    /// Creates a new DB game entry and returns a DatabaseGame
    ///
    /// # Returns
    ///
    /// * `DatabaseGame` on success. `GameError::CreationError` on failure.
    pub async fn new() -> Result<Self, DBGameError> {
        log::info!("Creating a new database game.");
        let collection = DatabaseGame::get_collection().await;
        let _id: ObjectId = match collection.insert_one(doc! {}, None).await {
            Ok(result) => {
                bson::from_bson(result.inserted_id).expect("Could not deserialize new game _id.")
            }
            Err(e) => {
                log::error!("ERROR: failed to create new game. {}.", e);
                return Err(DBGameError::CreationError);
            }
        };

        let friend_code = utils::generate_random_string(4, true);
        let game = DatabaseGame {
            friend_code,
            _id,
            players: HashSet::with_capacity(10),
            display_names: HashSet::with_capacity(10),
            status: DBGameStatus::Lobby,
            created_time: Utc::now().timestamp(),
            start_time: None,
            end_time: None,
            snapshot_id: None,
        };

        collection
            .replace_one(
                doc! {"_id": &game._id},
                bson::to_document(&game).unwrap(),
                None,
            )
            .await
            .unwrap();

        log::info!("Successfully created DB entry for game {}.", game._id);
        Ok(game)
    }

    /// Starts the database game, updating the DB as needed. Once started,
    /// no players may be added or removed.
    ///
    /// # Returns
    ///
    /// Empty type on success, `DBGameError` on failure.
    pub async fn start_game(&mut self) -> Result<(), DBGameError> {
        log::info!("Starting DB game {}.", self._id);
        self.start_time = Some(Utc::now().timestamp());
        self.status = DBGameStatus::InProgress;

        // TODO: update snapshot ID as well once we have snapshot code.
        let update_doc = doc! {
            "$set": {
                "start_time": bson::to_bson(&self.start_time).unwrap(),
                "status": bson::to_bson(&self.status).unwrap(),
                "snapshot_id": bson::to_bson(&self.snapshot_id).unwrap(),
            }
        };

        self.update_db(update_doc).await
    }

    /// Ends the database game, updating the DB as needed.
    ///
    /// # Returns
    ///
    /// Empty type on success, `DBGameError` on failure.
    pub async fn end_game(&mut self) -> Result<(), DBGameError> {
        // Remove friend code, since games can only be looked up by friend
        // code while active.
        self.friend_code.clear();
        self.end_time = Some(Utc::now().timestamp());
        self.status = DBGameStatus::Finished;
        let update_doc = doc! {
            "$set": {
                "friend_code": bson::to_bson(&self.friend_code).unwrap(),
                "end_time": bson::to_bson(&self.end_time).unwrap(),
                "status": bson::to_bson(&self.status).unwrap()
            }
        };

        self.update_db(update_doc).await
    }

    /// Adds a player to the DB game instance, updating the DB accordingly.
    /// Players can only be added if the game status is `Lobby`.
    ///
    /// # Arguments
    ///
    /// * `player_id` - The player ID to add to the game
    /// * `display_name` - The display name of the joining player
    ///
    /// # Returns
    ///
    /// * Empty type on success
    /// * `DBGameError::InvalidStateError` if the game state isn't `Lobby`
    /// * `DBGameError::UpdateError` if a DB update fails
    pub async fn add_player(
        &mut self,
        player_id: String,
        display_name: String,
    ) -> Result<(), DBGameError> {
        log::info!("Adding player {} to game {}.", player_id, self._id);

        if self.status != DBGameStatus::Lobby {
            log::error!(
                "ERROR: attempted to add player {} to game {} while in state {:?}. 
            Players may only be added during the Lobby phase.",
                player_id,
                self._id,
                self.status
            );
            return Err(DBGameError::InvalidStateError);
        }

        self.players.insert(player_id);
        self.display_names.insert(display_name);
        let update_doc = doc! {
            "$set": {
                "players": bson::to_bson(&self.players).unwrap(),
                "display_names": bson::to_bson(&self.display_names).unwrap()
            }
        };

        self.update_db(update_doc).await
    }

    /// Removes a player to the DB game instance, updating the DB accordingly.
    /// Players can only be removed if the game status is `Lobby`.
    ///
    /// # Arguments
    ///
    /// * `player_id` - The player ID to add to the game
    /// * `display_name` - The display name of the joining player
    ///
    /// # Returns
    ///
    /// * Empty type on success
    /// * `DBGameError::InvalidStateError` if the game state isn't `Lobby`
    /// * `DBGameError::UpdateError` if a DB update fails
    pub async fn remove_player(
        &mut self,
        player_id: &String,
        display_name: &String,
    ) -> Result<(), DBGameError> {
        log::info!("Removing player {} to game {}.", player_id, self._id);
        if self.status != DBGameStatus::Lobby {
            log::error!(
                "ERROR: attempted to remove player {} to game {} while in state {:?}. 
            Players may only be removed during the Lobby phase.",
                player_id,
                self._id,
                self.status
            );
            return Err(DBGameError::InvalidStateError);
        }

        self.players.remove(player_id);
        self.display_names.remove(display_name);
        let update_doc = doc! {
            "$set": {
                "players": bson::to_bson(&self.players).unwrap(),
                "display_names": bson::to_bson(&self.display_names).unwrap()
            }
        };

        self.update_db(update_doc).await
    }

    /// Helper function to get a handle to the game collection.
    ///
    /// # Returns
    ///
    /// `Collection` of Thavalon DB games
    async fn get_collection() -> Collection {
        get_database().await.collection(GAME_COLLECTION)
    }

    /// Updates the game database using the provided update document.
    ///
    /// # Arguments
    ///
    /// * `update_doc` - The document with fields to update
    ///
    /// # Returns
    ///
    /// * Empty type on success, `DBGameError` on failure
    async fn update_db(&self, update_doc: Document) -> Result<(), DBGameError> {
        let collection = DatabaseGame::get_collection().await;
        if let Err(e) = collection
            .update_one(doc! {"_id": &self._id}, update_doc, None)
            .await
        {
            log::error!("ERROR: failed to update database game. {}.", e);
            return Err(DBGameError::UpdateError);
        }

        log::info!("DB game {} updated successfully.", self._id);
        Ok(())
    }

    /// Getter for the friend_code field.
    ///
    /// # Returns
    ///
    /// A `string` representing the friend code
    pub fn get_friend_code(&self) -> &String {
        &self.friend_code
    }
}
