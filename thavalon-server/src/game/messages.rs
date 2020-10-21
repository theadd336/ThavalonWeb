//! Asynchronous engine for running THavalon games

use std::collections::HashSet;

use thiserror::Error;

use super::role::Role;
use super::{Card, MissionNumber, PlayerId, ProposalNumber};

// Game-related messages

/// Something the player tries to do
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Action {
    Propose { players: HashSet<PlayerId> },
    Vote { upvote: bool },
    Play { card: Card },
    Declare,
}

/// A message from the game to a player
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Message {
    /// Error message, usually when a player does something wrong
    Error(String),

    /// Sends the player their role and information
    RoleInformation { role: Role, information: String },

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
    VotingResults {
        upvotes: Vec<PlayerId>,
        downvotes: Vec<PlayerId>,
        sent: bool,
    },

    /// Announces the results of a mission going
    MissionResults {
        successes: usize,
        fails: usize,
        reverses: usize,
        passed: bool,
    },
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Can't reach player {}", id)]
    PlayerUnavailable { id: PlayerId },

    #[error("All players have disconnected")]
    AllDisconnected,
}
