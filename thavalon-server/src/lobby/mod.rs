//! Module for all game lobby code. The Lobby represents the interface between
//! the actual game, the database, and player connections.

mod client;
mod lobby_impl;

pub use lobby_impl::Lobby;
use thiserror::Error;
use tokio::sync::{mpsc::Sender, oneshot};
use warp::filters::ws::WebSocket;

/// Type representing a oneshot sender back to the caller.
pub type ResponseChannel = oneshot::Sender<LobbyResponse>;

/// Type repreesenting a channel to the lobby to issue commands.
pub type LobbyChannel = Sender<(LobbyCommand, Option<ResponseChannel>)>;

/// Enum of possible lobby-related errors.
#[derive(Debug, Error)]
pub enum LobbyError {
    #[error("An error occurred while updating the database.")]
    DatabaseError,
    #[error("The player is already in the game.")]
    DuplicatePlayerError,
    #[error("A lobby update was attemped in a state that doesn't permit this update.")]
    InvalidStateError,
    #[error("An unknown error occurred. See logs for details.")]
    UnknownError,
    #[error("Client ID is not registered for the game.")]
    InvalidClientID,
}

/// Enum of available commands to send to the lobby.
pub enum LobbyCommand {
    AddPlayer {
        player_id: String,
        display_name: String,
    },
    GetFriendCode,
    IsClientRegistered {
        client_id: String,
    },
    ConnectClientChannels {
        client_id: String,
        ws: WebSocket,
    },
    Ping {
        client_id: String,
    },
    StartGame,
}

/// Enum of possible responses from the lobby.
#[derive(Debug)]
pub enum LobbyResponse {
    Standard(Result<(), LobbyError>),
    JoinGame(Result<String, LobbyError>),
    FriendCode(String),
    IsClientRegistered(bool),
}
