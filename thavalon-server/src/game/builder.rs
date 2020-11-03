//! Builder for configuring and launching a new game.

use tokio::sync::mpsc;
use tokio::task;

use super::Game;
use super::engine;
use super::interactions::ChannelInteractions;
use super::messages::{Action, Message};

/// Builder for starting a new THavalon game
pub struct GameBuilder {
    interactions: ChannelInteractions,
    players: Vec<String>,
}

impl GameBuilder {
    /// Add a new player to the game. Any actions performed by the player should be sent to the returned `mpsc::Sender`. All messages
    /// on the returned `mpsc::Receiver` should be shown to the player.
    pub fn add_player(&mut self, name: String) -> (mpsc::Sender<Action>, mpsc::Receiver<Message>) {
        // Allow a 10-message backlog for each channel, in case tasks get backed up.
        let (action_tx, action_rx) = mpsc::channel(10);
        let (message_tx, message_rx) = mpsc::channel(10);

        self.interactions.add_player(name.clone(), action_rx, message_tx);
        self.players.push(name);

        (action_tx, message_rx)
    }

    /// Start the game. This consumes `self` because no new players can be added once the game starts.
    /// The returned `task::JoinHandle` will complete once the game has ended.
    pub fn start(self) -> task::JoinHandle<()> {
        let game = Game::roll(self.players);
        let mut interactions = self.interactions;
        task::spawn(async move {
            if let Err(e) = engine::run_game(&game, &mut interactions).await {
                log::error!("Fatal game error: {}", e);
            }
        })
    }
}
