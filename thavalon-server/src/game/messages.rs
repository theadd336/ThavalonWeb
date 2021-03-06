//! Asynchronous engine for running THavalon games

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::role::{PriorityTarget, RoleDetails, Team};
use super::{Card, MissionNumber};

// Game-related messages

/// Something the player tries to do
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub enum Action {
    Propose {
        players: HashSet<String>,
    },
    SelectPlayer {
        player: String,
    },
    UnselectPlayer {
        player: String,
    },
    Vote {
        upvote: bool,
    },
    Play {
        card: Card,
    },
    Obscure,
    QuestingBeast,
    Declare,
    Assassinate {
        players: HashSet<String>,
        target: PriorityTarget,
    },
    MoveToAssassination,
}

/// A message from the game to a player
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(tag = "messageType", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum Message {
    /// Error message, usually when a player does something wrong
    Error(String),

    /// The proposal order of the players in the game. Sent at the start of the game.
    ProposalOrder(Vec<String>),

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
        /// The number of players to include on the proposal
        mission_size: usize,
    },

    /// The current proposal was updated
    ProposalUpdated { players: HashSet<String> },

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

    /// Announces that a player has submitted a valid vote.
    VoteReceived,

    /// Announces the results of a vote
    VotingResults { sent: bool, counts: VoteCounts },

    /// Announces that a mission is going
    MissionGoing {
        mission: MissionNumber,
        players: HashSet<String>,
    },

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
    AgravaineDeclaration {
        mission: MissionNumber,
        player: String,
    },

    /// Assassination has begun. This can either be because 3 missions passed or because the assassin moved to assassinate.
    BeginAssassination { assassin: String },

    /// The results of an assassination attempt.
    AssassinationResult {
        /// The players that were assassinated (usually just 1)
        players: HashSet<String>,
        /// What the players were assassinated as
        target: PriorityTarget,
        /// Whether or not the assassination was correct
        correct: bool,
    },

    /// Sent to Arthur to indicate that they can declare
    ArthurCanDeclare,

    /// Sent to Arthur to indicate that they cannot declare
    ArthurCannotDeclare,

    /// Announces that Arthur has declared
    // TODO: generic Declaration message with the player and role instead?
    ArthurDeclaration { player: String },

    /// Sent when the game is over to announce who won.
    GameOver {
        winning_team: Team,
        roles: HashMap<String, RoleDetails>,
    },

    /// Message that a client should surface to the end user.
    Toast {
        severity: ToastSeverity,
        message: String,
    },
}

/// Severity of a toast notification
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ToastSeverity {
    INFO,
    WARN,
    URGENT,
}

/// How players voted on a proposal
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(tag = "voteType")]
pub enum VoteCounts {
    /// Public mission votes, where it is known who up- or downvoted.
    Public {
        upvotes: HashSet<String>,
        downvotes: HashSet<String>,
    },
    /// Obscured mission votes, where it is not known who up- or downvoted.
    Obscured { upvotes: u32, downvotes: u32 },
}

#[derive(Error, Debug, Serialize)]
pub enum GameError {
    #[error("Could not communicate with player")]
    PlayerDisconnected,

    #[error("Internal interaction error")]
    #[serde(serialize_with = "serialize_internal_error")]
    Internal(#[from] Box<dyn std::error::Error + Send + 'static>),
}

#[allow(clippy::borrowed_box)] // We need &Box<T> instead of &T here to match what serde expects and to add the Send + 'static constraints
fn serialize_internal_error<S: serde::Serializer>(
    error: &Box<dyn std::error::Error + Send + 'static>,
    ser: S,
) -> Result<S::Ok, S::Error> {
    let error_message = error.to_string();
    ser.serialize_str(&error_message)
}
