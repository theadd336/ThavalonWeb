//! Errors module for lobby related errors.

use thiserror::Error;

/// Enum representing an assortment of lobby related errors.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum LobbyError {
    /// Error for when a lobby fails to create.
    #[error("Lobby creation failed.")]
    CreationFailed(),

    /// Error for when a lobby does not exist.
    #[error("Lobby ID {} does not exist.", lobby_id)]
    LobbyDoesNotExist { lobby_id: String },

    /// Error for when a duplicate player name exists in the lobby.
    #[error("Player name {} already exists in the lobby.", player_name)]
    PlayerNameTaken { player_name: String },

    ///Error for when the lobby is at an invalid status.
    #[error("Invalid lobby status for action.")]
    InvalidLobbyStatus(),

    ///Error for when the name provided is too long.
    #[error("The player name exceeds the limit of 30 characters.")]
    PlayerNameTooLong(),
}
