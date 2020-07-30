use rand::prelude::*;

use super::{PlayerId, Player, Players};

/// A THavalon role
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Role {
    Merlin,
    Lancelot,
    Percival,
    Tristan,
    Iseult,

    Mordred,
    Morgana,
    Maelegant,
}

impl Role {
    pub fn is_good(self) -> bool {
        use Role::*;
        match self {
            Merlin | Lancelot | Percival | Tristan | Iseult => true,
            Mordred | Morgana | Maelegant => false
        }
    }

    pub fn is_evil(self) -> bool {
        !self.is_good()
    }

    pub fn is_lover(self) -> bool {
        match self {
            Role::Tristan | Role::Iseult => true,
            _ => false
        }
    }

    /// Create role information for a player, `me`, given all `players` in the game.
    pub fn generate_info<R: Rng>(self, rng: &mut R, me: PlayerId, players: &Players) -> String {
        "".to_string()
    }
}
