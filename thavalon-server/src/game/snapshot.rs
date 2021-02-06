//! Snapshots of the current state of a game. These can be persisted or sent to clients.
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use super::interactions::Interactions;
use super::messages::{Action, GameError, Message, VoteCounts};
use super::role::{Role, RoleDetails};
use super::MissionNumber;

/// Snapshot of game state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSnapshot {
    pub me: String,
    pub role_info: Option<RoleDetails>,
    pub log: Vec<Message>,
}

impl GameSnapshot {
    pub fn new(player: String) -> GameSnapshot {
        GameSnapshot {
            me: player,
            role_info: None,
            log: Vec::new(),
        }
    }

    /// Updates the game snapshot based on a message sent to the player.
    ///
    /// If the message cannot be reconciled with the current snapshot, this returns a [`SnapshotError`]. This should
    /// never happen.
    pub fn on_message(&mut self, message: Message) -> Result<(), SnapshotError> {
        self.log.push(message.clone());

        match message {
            Message::RoleInformation { details } => {
                self.role_info = Some(details);
                Ok(())
            }

            _ => Ok(()), // Some messages don't require a state update
        }
    }
}

#[derive(Error, Debug)]
pub enum SnapshotError {
    #[error("Unexpected message: {0:?}")]
    UnexpectedMessage(Message),

    #[error("No such player: {0}")]
    NoSuchPlayer(String),
}

impl From<SnapshotError> for GameError {
    fn from(error: SnapshotError) -> GameError {
        GameError::Internal(Box::new(error))
    }
}

/// An [`Interactions`] wrapper which snapshots all messages in addition to forwarding them to another [`Interactions`]
pub struct SnapshotInteractions<I: Interactions> {
    inner: I,
    snapshots: Arc<Mutex<HashMap<String, Arc<Mutex<GameSnapshot>>>>>,
}

/// Handle to the per-player snapshots maintained by [`SnapshotInteractions`].
#[derive(Debug, Clone)]
pub struct Snapshots {
    inner: Arc<Mutex<HashMap<String, Arc<Mutex<GameSnapshot>>>>>,
}

impl<I: Interactions> SnapshotInteractions<I> {
    /// Create a new `SnapshotInteractions` that delegates to `inner`.
    pub fn new<P: IntoIterator<Item = String>>(inner: I, players: P) -> SnapshotInteractions<I> {
        let snapshots = players
            .into_iter()
            .map(|player| {
                let snapshot = Arc::new(Mutex::new(GameSnapshot::new(player.clone())));
                (player, snapshot)
            })
            .collect();

        SnapshotInteractions {
            inner,
            snapshots: Arc::new(Mutex::new(snapshots)),
        }
    }

    /// Create a new [`Snapshots`] handle, which will have access to all game snapshots this creates.
    pub fn snapshots(&self) -> Snapshots {
        Snapshots {
            inner: self.snapshots.clone(),
        }
    }

    fn snapshot(&mut self, player: &str) -> Option<Arc<Mutex<GameSnapshot>>> {
        let snapshots = self.snapshots.lock().unwrap();
        snapshots.get(player).cloned()
    }
}

#[async_trait]
impl<I: Interactions + Send> Interactions for SnapshotInteractions<I> {
    async fn send_to(&mut self, player: &str, message: Message) -> Result<(), GameError> {
        {
            let snapshot = self
                .snapshot(player)
                .ok_or_else(|| SnapshotError::NoSuchPlayer(player.to_string()))?;
            let mut snapshot = snapshot.lock().unwrap();
            snapshot.on_message(message.clone())?;
        }
        self.inner.send_to(player, message).await
    }

    async fn send(&mut self, message: Message) -> Result<(), GameError> {
        {
            let snapshots = self.snapshots.lock().unwrap();
            for snapshot in snapshots.values() {
                let mut snapshot = snapshot.lock().unwrap();
                snapshot.on_message(message.clone())?;
            }
        }
        self.inner.send(message).await
    }

    async fn receive(&mut self) -> Result<(String, Action), GameError> {
        self.inner.receive().await
    }
}

impl Snapshots {
    /// Gets a handle to the snapshot for a given player. This will return `None` if the player does not
    /// exist. As the game progresses, the [`GameSnapshot`] inside the [`Mutex`] will update.
    pub fn get(&self, player: &str) -> Option<Arc<Mutex<GameSnapshot>>> {
        let snapshots = self.inner.lock().unwrap();
        snapshots.get(player).cloned()
    }
}
