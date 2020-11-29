//! Asynchronous engine for running THavalon games

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::role::{RoleDetails, PriorityTarget};
use super::{Card, MissionNumber};

// Game-related messages

/// Something the player tries to do
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub enum Action {
    Propose { players: HashSet<String> },
    Vote { upvote: bool },
    Play { card: Card },
    QuestingBeast,
    Declare,
    Assassinate {
        players: HashSet<String>,
        target: PriorityTarget,
    }
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
        /// The player who will be proposing
        proposer: String,
        /// The mission this proposal is for
        mission: MissionNumber,
        /// The number of proposals made so far, excluding mission 1 and sent proposals
        proposals_made: usize,
        /// The maximum number of unsent proposals before force activates
        max_proposals: usize,
    },

    /// Announces that a player made a proposal
    ProposalMade {
        /// The player who made the proposal
        proposer: String,
        /// The mission they're proposing for
        mission: MissionNumber,
        /// The players on the mission
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
    /// Public mission votes, where it is known who up- or downvoted.
    Public {
        upvotes: HashSet<String>,
        downvotes: HashSet<String>,
    },
    /// Obscured mission votes, where it is not known who up- or downvoted.
    Obscured { upvotes: u32, downvotes: u32 },
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Could not communicate with player")]
    PlayerDisconnected,

    #[error("Internal interaction error")]
    Internal(#[from] Box<dyn std::error::Error + Send + 'static>),
}
