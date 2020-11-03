use std::fmt::{self, Write};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Card, PlayerId, Players};

/// A THavalon role
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize)]
pub enum Role {
    Merlin,
    Lancelot,
    Percival,
    Tristan,
    Iseult,

    Mordred,
    Morgana,
    Maelegant,
    Agravaine,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum Team {
    Good,
    /// "Misunderstood"
    Evil,
}

/// Information a player receives based on their role.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct RoleDetails {
    /// The team the player is on.
    team: Team,
    /// The player's role.
    role: Role,
    /// A high-level description of the role.
    description: String,
    /// Other players that this player sees.
    seen_players: Vec<String>,
    /// Other members of this player's team, empty if the player is not evil.
    team_members: Vec<String>,
    /// Miscellaneous other information the player posesses.
    other_info: String,
    /// Special abilities the player has.
    abilities: String,
    /// Whether or not the player can be assassinated.
    assassinatable: bool,
}

impl Role {
    pub fn is_good(self) -> bool {
        use Role::*;
        match self {
            Merlin | Lancelot | Percival | Tristan | Iseult => true,
            Mordred | Morgana | Maelegant | Agravaine => false,
        }
    }

    pub fn is_evil(self) -> bool {
        !self.is_good()
    }

    pub fn team(self) -> Team {
        if self.is_good() {
            Team::Good
        } else {
            Team::Evil
        }
    }

    pub fn is_lover(self) -> bool {
        match self {
            Role::Tristan | Role::Iseult => true,
            _ => false,
        }
    }

    pub fn is_assassinatable(self) -> bool {
        match self {
            Role::Merlin | Role::Tristan | Role::Iseult => true,
            _ => false,
        }
    }

    pub fn can_play(self, card: Card) -> bool {
        // The life of an Agravaine is a simple one
        if self == Role::Agravaine {
            card == Card::Fail
        } else {
            match card {
                Card::Success => true,
                Card::Reverse => matches!(self, Role::Lancelot | Role::Maelegant),
                Card::Fail => self.is_evil(),
            }
        }
    }

    /// Create role information for a player, `me`, given all `players` in the game.
    pub fn generate_info<R: Rng>(
        self,
        rng: &mut R,
        me: PlayerId,
        players: &Players,
    ) -> RoleDetails {
        let mut seen_players = Vec::new();
        let mut description = String::new();
        let mut abilities = String::new();
        let mut other_info = String::new();

        match self {
            Role::Merlin => {
                seen_players.extend(
                    players
                        .iter()
                        .filter(|player| {
                            (player.role.is_evil() && player.role != Role::Mordred)
                                || player.role == Role::Lancelot
                        })
                        .map(|player| player.name.clone()),
                );
                let _ = writeln!(&mut description, "You know who is evil, but not their roles. You do not see Mordred, but do see Lancelot as evil.");
            }
            Role::Lancelot => {
                let _ = writeln!(&mut abilities, "You may play Reverse cards on missions.");
            }
            Role::Percival => {
                let _ = writeln!(
                    &mut description,
                    "You see Morgana and the priority assassination targets."
                );

                // TODO: priority targets
                for player in players.iter() {
                    if player.role == Role::Morgana {
                        seen_players.push(player.name.clone());
                    }
                }
            }
            Role::Tristan | Role::Iseult => {
                let _ = writeln!(
                    &mut description,
                    "You may or may not see your Lover at some point I guess? Once you and your Lover go on a mission together, you will be revealed to each other. Until then, you will be told after each mission if it contained your Lover."
                );
            }
            Role::Mordred => {
                let _ = writeln!(&mut description, "You are hidden from Merlin.");
            }
            Role::Morgana => {
                let _ = writeln!(&mut description, "You appear like Merlin to Percival.");
            }
            Role::Maelegant => {
                let _ = writeln!(&mut abilities, "You may play Reverse cards on missions.");
                if players.has_role(Role::Lancelot) {
                    let _ = writeln!(&mut other_info, "There is a Lancelot in the game.");
                } else {
                    let _ = writeln!(&mut other_info, "There is not a Lancelot in the game.");
                }
            }
            Role::Agravaine => {
                let _ = writeln!(&mut abilities, "You may declare to fail a mission you were on that would have otherwise succeeded.");
            }
        }

        seen_players.shuffle(rng);

        let team_members = if self.is_evil() {
            players
                .evil_players()
                .iter()
                .filter(|player| **player != me)
                .map(|player| players[*player].name.clone())
                .collect()
        } else {
            Vec::new()
        };

        RoleDetails {
            team: self.team(),
            role: self,
            description,
            abilities,
            seen_players,
            team_members,
            other_info,
            assassinatable: self.is_assassinatable(),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Can use debug since it's still the role name
        writeln!(f, "{:?}", self)
    }
}
