//! Snapshots of the current state of a game. These can be persisted or sent to clients.
#![allow(dead_code)]

use std::collections::HashSet;

use thiserror::Error;

use super::messages::{Message, VoteCounts};
use super::role::{Role, RoleDetails};
use super::MissionNumber;

/// Snapshot of game state.
#[derive(Debug, Clone)]
pub struct GameSnapshot {
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct MissionResults {
    pub successes: usize,
    pub fails: usize,
    pub reverses: usize,
    pub questing_beasts: usize,
    pub passed: bool,
    pub agravaine_declared: bool,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub proposed_by: String,
    pub players: HashSet<String>,
}

/// Results of voting on a mission. Normally, this includes exactly who upvoted or downvoted. If Maeve obscured the mission,
/// only counts are known.
#[derive(Debug, Clone)]
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
    pub fn new() -> GameSnapshot {
        GameSnapshot {
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

    /// Get a mutable reference to the current mission.alloc
    /// 
    /// # Panics
    /// If there is *no* current mission, which would only happen if messages were received in an invalid
    /// order.
    fn current_mut(&mut self) -> &mut Mission {
        self.missions.last_mut().unwrap()
    }

    pub fn on_message(&mut self, message: Message) -> Result<(), SnapshotError> {
        self.log.push(message.clone());

        match message {
            Message::RoleInformation { details } => {
                self.role_info = Some(details);
                Ok(())
            }

            Message::NextProposal {
                proposal,
                mission,
                proposer,
            } => {
                // If it's the first proposal of a round, we need to add a new Mission struct
                if proposal == 0 {
                    if self.current_mission() != mission - 1 {
                        return Err(SnapshotError::UnexpectedMessage(Message::NextProposal {
                            proposer,
                            proposal,
                            mission,
                        }));
                    }
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
                let mut mission = self.current_mut();
                if mission.proposals.len() != mission.voting_results.len() + 1 {
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
                if sent {
                    mission.sent_proposal = Some(mission.voting_results.len() - 1);
                }
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
                if state.results.is_none() {
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

            Message::AgravaineDeclaration { mission } => {
                let state = self.mission_mut(mission);
                if let Some(mut results) = state.results.as_mut() {
                    results.agravaine_declared = true;
                    results.passed = false;
                    Ok(())
                } else {
                    Err(SnapshotError::UnexpectedMessage(
                        Message::AgravaineDeclaration { mission },
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
}
