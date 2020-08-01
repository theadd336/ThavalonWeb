use thiserror::Error;

#[derive(Error, Debug)]
pub enum LobbyError {
    #[error("Lobby creation failed.")]
    CreationFailed(),
}
