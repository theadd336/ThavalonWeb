use std::fmt::{self, Write};

use rand::prelude::*;

use super::{PlayerId, Players};

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
            Mordred | Morgana | Maelegant => false,
        }
    }

    pub fn is_evil(self) -> bool {
        !self.is_good()
    }

    pub fn is_lover(self) -> bool {
        match self {
            Role::Tristan | Role::Iseult => true,
            _ => false,
        }
    }

    pub fn is_assasinatable(self) -> bool {
        match self {
            Role::Merlin | Role::Tristan | Role::Iseult => true,
            _ => false,
        }
    }

    /// Create role information for a player, `me`, given all `players` in the game.
    pub fn generate_info<R: Rng>(self, _rng: &mut R, me: PlayerId, players: &Players) -> String {
        // TODO: let this be markdown or something?
        let mut info = format!("You are {}", self);

        match self {
            Role::Merlin => {
                let _ = writeln!(&mut info, "You see these players as evil:");
                for player in players.iter() {
                    if player.role == Role::Lancelot
                        || (player.role.is_evil() && player.role != Role::Mordred)
                    {
                        let _ = writeln!(&mut info, "* {}", player.name);
                    }
                }
            }
            Role::Lancelot => {
                let _ = writeln!(&mut info, "You may play Reverse cards on missions.");
            }
            Role::Percival => {
                let _ = writeln!(&mut info, "You see these players as Merlin or Morgana:");
                for player in players.iter() {
                    if player.role == Role::Merlin || player.role == Role::Morgana {
                        let _ = writeln!(&mut info, "* {}", player.name);
                    }
                }
            }
            Role::Tristan | Role::Iseult => {
                let _ = writeln!(
                    &mut info,
                    "You may or may not see your lover at some point I guess?"
                );
            }
            Role::Mordred => {
                let _ = writeln!(&mut info, "You are hidden from all Good information roles.");
            }
            Role::Morgana => {
                let _ = writeln!(&mut info, "You appear like Merlin to Percival.");
            }
            Role::Maelegant => {
                let _ = writeln!(&mut info, "You may play Reverse cards on missions.");
            }
        }

        if self.is_assasinatable() {
            let _ = writeln!(&mut info, "You are assasinatable.");
        }

        if self.is_evil() {
            let _ = writeln!(&mut info, "Your evil team:");
            for player in players.evil_players() {
                if *player != me {
                    let _ = writeln!(&mut info, "* {}", players[*player].name);
                }
            }
        }

        info
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Can use debug since it's still the role name
        writeln!(f, "{:?}", self)
    }
}
