//! Module for all game lobby code. The Lobby represents the interface between
//! the actual game, the database, and player connections.

mod client;
mod lobby_impl;

use crate::game::{snapshot::GameSnapshot, Action, Message};
pub use lobby_impl::Lobby;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task;
use tokio::sync::{mpsc::Sender, oneshot};
use warp::filters::ws::WebSocket;

/// Type representing a oneshot sender back to the caller.
pub type ResponseChannel = oneshot::Sender<LobbyResponse>;

/// Type representing a channel to the lobby to issue commands.
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
    #[error("The player tried to reconnect with a new name.")]
    NameChangeOnReconnectError,
    #[error("The display name is already in use.")]
    DuplicateDisplayName,
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
    GetLobbyState {
        client_id: String,
    },
    StartGame,
    EndGame,
    PlayerDisconnect {
        client_id: String,
    },
    GetPlayerList {
        client_id: String,
    },
    GetSnapshots {
        client_id: String,
    },
    PlayerFocusChange {
        client_id: String,
        is_tabbed_out: bool,
    },
}

/// Enum of possible responses from the lobby.
#[derive(Debug)]
pub enum LobbyResponse {
    Standard(Result<(), LobbyError>),
    None,
    JoinGame(Result<String, LobbyError>),
    FriendCode(String),
    IsClientRegistered(bool),
}

/// An incoming message from the client.
#[derive(Deserialize)]
#[serde(tag = "messageType", content = "data")]
enum IncomingMessage {
    Ping,
    StartGame,
    GetLobbyState,
    GameCommand(Action),
    GetPlayerList,
    GetSnapshot,
    PlayerFocusChange(bool),
}

/// An outgoing message to the client.
#[derive(Serialize)]
#[serde(tag = "messageType", content = "data")]
pub enum OutgoingMessage {
    Pong(String),
    PlayerList(Vec<String>),
    LobbyState(LobbyState),
    GameMessage(Message),
    Snapshot(GameSnapshot),
    PlayerFocusChange {
        displayName: String,
        isTabbedOut: bool,
    },
}

#[derive(Serialize, Eq, PartialEq, Clone)]
#[serde(tag = "state")]
pub enum LobbyState {
    Lobby,
    Game,
    Finished,
}
