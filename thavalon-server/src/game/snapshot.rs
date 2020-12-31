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
    me: String,
    pub role_info: Option<RoleDetails>,
    missions: Vec<Mission>,
    log: Vec<Message>,
}

/// Static information about a player, set at game creation time.
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub role: Role,
    pub info: RoleDetails,
}

/// Details about a mission. If the mission is in process, data may be missing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    /// Submitted proposals. Every proposal that has been voted on will have a corresponding entry in `voting_results`.
    pub proposals: Vec<Proposal>,
    /// Results from voting on proposals.
    pub voting_results: Vec<VotingResults>,
    /// If a proposal has been voted on and sent, this will be the index into `proposals` of the one that was sent. In most
    /// cases, this will be the *last* proposal. However, on mission one it could be either of the two proposals.
    pub sent_proposal: Option<usize>,
    /// If the mission has gone, this has the results, including what cards were played and whether or not it passed. If Agravaine declares,
    /// the mission's `passed` state may change again.
    pub results: Option<MissionResults>,
}

/// The outcome of a mission, including details on which cards were played and whether or not the mission passed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionResults {
    pub successes: usize,
    pub fails: usize,
    pub reverses: usize,
    pub questing_beasts: usize,
    pub passed: bool,
    pub agravaine_declared: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub proposed_by: String,
    pub players: HashSet<String>,
}

/// Results of voting on a mission. Normally, this includes exactly who upvoted or downvoted. If Maeve obscured the mission,
/// only counts are known.
#[derive(Debug, Clone, Serialize)]
pub enum VotingResults {
    Public {
        upvotes: HashSet<String>,
        downvotes: HashSet<String>,
    },
    Obscured {
        upvotes: u32,
        downvotes: u32,
    },
}

impl GameSnapshot {
    pub fn new(player: String) -> GameSnapshot {
        GameSnapshot {
            me: player,
            role_info: None,
            missions: Vec::new(),
            log: Vec::new(),
        }
    }

    /// Looks up mission information by number. If the game has not reached `mission` yet, returns `None`.
    pub fn mission(&self, mission: MissionNumber) -> Option<&Mission> {
        self.missions.get((mission - 1) as usize)
    }

    /// Like [`Self::mission`], but returns a mutable reference
    fn mission_mut(&mut self, mission: MissionNumber) -> &mut Mission {
        &mut self.missions[(mission - 1) as usize]
    }

    /// The number of the mission currently in progress
    pub fn current_mission(&self) -> MissionNumber {
        self.missions.len() as MissionNumber
    }

    /// Get a mutable reference to the current mission
    ///
    /// # Panics
    /// If there is *no* current mission, which would only happen if messages were received in an invalid
    /// order.
    fn current_mut(&mut self) -> &mut Mission {
        self.missions.last_mut().unwrap()
    }

    /// Updates the game snapshot based on a message sent to the player.
    ///
    /// If the message cannot be reconciled with the current snapshot, this returns a [`SnapshotError`]. For example,
    /// if some messages are lost, the snapshot might receive an Agravaine declaration when it has not received the
    /// results of the mission, which is an error.
    pub fn on_message(&mut self, message: Message) -> Result<(), SnapshotError> {
        self.log.push(message.clone());

        match message {
            Message::RoleInformation { details } => {
                self.role_info = Some(details);
                Ok(())
            }

            Message::NextProposal { mission, .. } => {
                // If it's the first proposal of a round, we need to add a new Mission struct
                if self.missions.is_empty() || self.current_mission() != mission {
                    self.missions.push(Mission {
                        proposals: Vec::new(),
                        voting_results: Vec::new(),
                        sent_proposal: None,
                        results: None,
                    });
                }
                Ok(())
            }

            Message::ProposalMade {
                proposer,
                mission,
                players,
                ..
            } => {
                let state = self.mission_mut(mission);
                state.proposals.push(Proposal {
                    proposed_by: proposer,
                    players,
                });
                Ok(())
            }

            Message::VotingResults { sent, counts } => {
                let is_mission_1 = self.current_mission() == 1;
                let mut mission = self.current_mut();
                let expected_proposals = if is_mission_1 { 2 } else { mission.voting_results.len() + 1 };
                if mission.proposals.len() != expected_proposals {
                    return Err(SnapshotError::UnexpectedMessage(Message::VotingResults {
                        sent,
                        counts,
                    }));
                }
                let results = match counts {
                    VoteCounts::Public { upvotes, downvotes } => {
                        VotingResults::Public { upvotes, downvotes }
                    }
                    VoteCounts::Obscured { upvotes, downvotes } => {
                        VotingResults::Obscured { upvotes, downvotes }
                    }
                };
                mission.voting_results.push(results);
                mission.sent_proposal = if is_mission_1 {
                    if sent { Some(0) } else { Some(1) }
                } else {
                    Some(mission.voting_results.len() - 1)
                };
                Ok(())
            }

            Message::MissionResults {
                mission,
                successes,
                fails,
                reverses,
                questing_beasts,
                passed,
            } => {
                let state = self.mission_mut(mission);
                if state.results.is_some() {
                    // This means we already recorded results for this mission
                    return Err(SnapshotError::UnexpectedMessage(Message::MissionResults {
                        mission,
                        successes,
                        fails,
                        reverses,
                        questing_beasts,
                        passed,
                    }));
                }
                state.results = Some(MissionResults {
                    successes,
                    fails,
                    reverses,
                    questing_beasts,
                    passed,
                    agravaine_declared: false,
                });
                Ok(())
            }

            Message::AgravaineDeclaration { mission, player } => {
                let state = self.mission_mut(mission);
                if let Some(mut results) = state.results.as_mut() {
                    results.agravaine_declared = true;
                    results.passed = false;
                    Ok(())
                } else {
                    Err(SnapshotError::UnexpectedMessage(
                        Message::AgravaineDeclaration { mission, player },
                    ))
                }
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
