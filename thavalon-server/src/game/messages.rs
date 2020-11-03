//! Asynchronous engine for running THavalon games

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::role::RoleDetails;
use super::{Card, MissionNumber, PlayerId, ProposalNumber};

// Game-related messages

/// Something the player tries to do
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub enum Action {
    Propose { players: HashSet<PlayerId> },
    Vote { upvote: bool },
    Play { card: Card },
    QuestingBeast,
    Declare,
}

/// A message from the game to a player
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum Message {
    /// Error message, usually when a player does something wrong
    Error(String),

    /// Sends the player their role and information
    RoleInformation { details: RoleDetails },

    /// Announces that a new player is proposing
    NextProposal {
        proposer: PlayerId,
        mission: MissionNumber,
        proposal: ProposalNumber,
    },

    /// Announces that a player made a proposal
    ProposalMade {
        proposer: PlayerId,
        mission: MissionNumber,
        proposal: ProposalNumber,
        players: HashSet<PlayerId>,
    },

    /// Announces that players should submit votes for the latest proposal.
    CommenceVoting,

    /// Announces the results of a vote
    VotingResults { sent: bool, counts: VoteCounts },

    /// Announces the results of a mission going
    MissionResults {
        mission: MissionNumber,
        successes: usize,
        fails: usize,
        reverses: usize,
        questing_beasts: usize,
        passed: bool,
    },

    /// Agravaine declared, so the given mission now failed.
    AgravaineDeclaration { mission: MissionNumber },
}

/// How players voted on a proposal
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum VoteCounts {
    Public {
        upvotes: HashSet<PlayerId>,
        downvotes: HashSet<PlayerId>,
    },
    Obscured {
        upvotes: u32,
        downvotes: u32,
    },
}

#[derive(Error, Debug, Serialize)]
pub enum GameError {
    #[error("Can't reach player {}", id)]
    PlayerUnavailable { id: PlayerId },

    #[error("All players have disconnected")]
    AllDisconnected,
}
