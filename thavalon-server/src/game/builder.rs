//! Builder for configuring and launching a new game.

use crate::lobby::{LobbyChannel, LobbyCommand};

use tokio::sync::mpsc;
use tokio::task;

use super::engine;
use super::interactions::ChannelInteractions;
use super::messages::{Action, Message};
use super::snapshot::{SnapshotInteractions, Snapshots};
use super::{CreateGameError, Game};

use futures::future::{AbortRegistration, Abortable};

/// Builder for starting a new THavalon game
pub struct GameBuilder {
    interactions: ChannelInteractions,
    players: Vec<String>,
}

impl GameBuilder {
    /// Add a new player to the game. Any actions performed by the player should be sent to the returned `mpsc::Sender`. All messages
    /// on the returned [`mpsc::Receiver`] should be shown to the player.
    pub fn add_player(&mut self, name: String) -> (mpsc::Sender<Action>, mpsc::Receiver<Message>) {
        // Allow a 10-message backlog for each channel, in case tasks get backed up.
        let (action_tx, action_rx) = mpsc::channel(10);
        let (message_tx, message_rx) = mpsc::channel(10);

        self.interactions
            .add_player(name.clone(), action_rx, message_tx);
        self.players.push(name);

        (action_tx, message_rx)
    }

    pub fn remove_player(&mut self, name: &str) {
        self.interactions.remove_player(name);
        self.players.retain(|player| player != name);
    }

    pub fn get_player_list(&self) -> &Vec<String> {
        &self.players
    }

    /// Start the game. This consumes `self` because no new players can be added once the game starts.
    /// The returned [`task::JoinHandle`] will complete once the game has ended. The [`Snapshots`] may be
    /// used to track per-player snapshots of the game state.
    pub fn start(
        self,
        mut lobby_channel: LobbyChannel,
        abort_registration: AbortRegistration,
    ) -> Result<
        (
            Snapshots,
            task::JoinHandle<std::result::Result<(), futures::future::Aborted>>,
        ),
        CreateGameError,
    > {
        let mut interactions =
            SnapshotInteractions::new(self.interactions, self.players.iter().cloned());
        let game = Game::roll(self.players)?;
        let snapshots = interactions.snapshots();
        let task_handle = task::spawn(Abortable::new(
            async move {
                if let Err(e) = engine::run_game(game, &mut interactions).await {
                    log::error!("Fatal game error: {}", e);
                }
                lobby_channel.send((LobbyCommand::EndGame, None)).await;
            },
            abort_registration,
        ));
        Ok((snapshots, task_handle))
    }

    pub fn new() -> Self {
        GameBuilder {
            interactions: ChannelInteractions::new(),
            players: vec![],
        }
    }
}
