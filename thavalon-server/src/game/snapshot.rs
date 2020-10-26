//! Snapshots of the current state of a game. These can be persisted or sent to clients.

use std::collections::HashSet;

use super::{PlayerId, MissionNumber};
use super::role::Role;

struct GameState {
    current_mission: MissionNumber,
    past_missions: Vec<Mission>,
}

struct Mission {
    proposals: Vec<Proposal>,
    sent_proposal: usize,
}

struct Proposal {
    proposed_by: PlayerId,
    players: HashSet<PlayerId>,
    upvotes: HashSet<PlayerId>,
    downvotes: HashSet<PlayerId>,
    sent: bool,
}