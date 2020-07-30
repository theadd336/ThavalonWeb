//! Asynchronous engine for running THavalon games

use std::collections::HashMap;

use log::info;

use tokio::sync::mpsc;
use tokio::stream::StreamMap;

use super::{Game, PlayerId};

struct GameRunner {
    game: Game,
    
    /// Receivers for messages from players
    player_rx: StreamMap<PlayerId, mpsc::Receiver<PlayerMessage>>,

    /// Senders for each player 
    player_tx: HashMap<PlayerId, mpsc::Sender<GameMessage>>,
}

impl GameRunner {
    async fn run(&mut self) {
        info!("Starting game");

        for info in self.game.player_info.values() {
            info!("{} is {:?}", info.name, info.role);
        }
    }
}

/// A message from a player to the game engine
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PlayerMessage {

}

/// A message from the game engine to a player
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GameMessage {

}