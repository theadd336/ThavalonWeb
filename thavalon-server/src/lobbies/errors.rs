//! Errors module for lobby related errors.

use thiserror::Error;

/// Enum representing an assortment of lobby related errors.
#[derive(Error, Debug)]
pub enum LobbyError {
    /// Error for when a lobby fails to create.
    #[error("Lobby creation failed.")]
    CreationFailed(),
}
