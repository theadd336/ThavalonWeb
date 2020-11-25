//! Asynchronous engine for running THavalon games

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Card, MissionNumber, ProposalNumber};
use super::role::RoleDetails;

// Game-related messages

/// Something the player tries to do
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub enum Action {
    Propose { players: HashSet<String> },
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
        proposer: String,
        mission: MissionNumber,
        proposal: ProposalNumber,
    },

    /// Announces that a player made a proposal
    ProposalMade {
        proposer: String,
        mission: MissionNumber,
        proposal: ProposalNumber,
        players: HashSet<String>,
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
        upvotes: HashSet<String>,
        downvotes: HashSet<String>,
    },
    Obscured {
        upvotes: u32,
        downvotes: u32,
    },
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Could not communicate with player")]
    PlayerDisconnected,

    #[error("Internal interaction error")]
    Internal(#[from] Box<dyn std::error::Error + Send + 'static>),
}
