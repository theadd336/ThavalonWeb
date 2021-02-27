use std::fmt::{self, Write};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Card, GameSpec, Player, Players};

/// A THavalon role
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize)]
pub enum Role {
    // "Good" roles
    Merlin,
    Lancelot,
    Percival,
    Tristan,
    Iseult,
    Nimue,
    Arthur,
    Guinevere,

    // "Misunderstood" roles
    Mordred,
    Morgana,
    Maelegant,
    Maeve,
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
#[serde(rename_all = "camelCase")]
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
    /// Whether or not the player can be assassinated. Only ever true for good players.
    assassinatable: bool,
    /// Whether or not the player is the Assassin. Only ever true for evil players.
    is_assassin: bool,
    /// If the player is the Assassin, this is the Priority Target that they may assassinate.
    priority_target: Option<PriorityTarget>,
}

impl RoleDetails {
    pub fn get_role(&self) -> Role {
        self.role
    }
}

/// A priority assassination target. If the Good team passes 3 missions, then the Assassin must correctly identify
/// the Priority Target in order to win.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PriorityTarget {
    Merlin,
    Lovers,
    Guinevere,
    None,
}

impl Role {
    /// All Good roles
    pub const ALL_GOOD: &'static [Role] = &[
        Role::Merlin,
        Role::Lancelot,
        Role::Percival,
        Role::Tristan,
        Role::Iseult,
        Role::Nimue,
        Role::Arthur,
    ];

    /// All Evil roles
    pub const ALL_EVIL: &'static [Role] = &[
        Role::Mordred,
        Role::Morgana,
        Role::Maelegant,
        Role::Maeve,
        Role::Agravaine,
    ];

    pub fn is_good(self) -> bool {
        use Role::*;
        match self {
            Merlin | Lancelot | Percival | Tristan | Iseult | Nimue | Arthur | Guinevere => true,
            Mordred | Morgana | Maelegant | Maeve | Agravaine => false,
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
        matches!(self, Role::Tristan | Role::Iseult)
    }

    pub fn is_assassinatable(self) -> bool {
        matches!(
            self,
            Role::Merlin | Role::Tristan | Role::Iseult | Role::Guinevere
        )
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
        me: &str,
        spec: &GameSpec,
        players: &Players,
        assassin: &str,
        priority_target: PriorityTarget,
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

                if let Some(morgana) = players.by_role(Role::Morgana) {
                    seen_players.push(morgana.name.clone());
                }

                match priority_target {
                    PriorityTarget::Merlin => {
                        seen_players.push(players.by_role(Role::Merlin).unwrap().name.clone());
                    }
                    PriorityTarget::Lovers => {
                        seen_players.push(players.by_role(Role::Iseult).unwrap().name.clone());
                        seen_players.push(players.by_role(Role::Tristan).unwrap().name.clone());
                    }
                    PriorityTarget::None => (),
                    other => panic!("Unsupported priority target {:?}", other),
                }
            }
            Role::Tristan | Role::Iseult => {
                let _ = writeln!(
                    &mut description,
                    "You may or may not see your Lover at some point I guess? Once you and your Lover go on a mission together, you will be revealed to each other. Until then, you will be told after each mission if it contained your Lover."
                );
            }
            Role::Nimue => {
                let _ = writeln!(
                    &mut description,
                    "You see all roles in the game, but not who has which role."
                );
                seen_players.extend(players.iter().map(|player| player.role.to_string()));
            }
            Role::Arthur => {
                let _ = writeln!(
                    &mut description,
                    "You see all Good roles in the game, but not who has which role. If two missions have failed, but it's not yet mission 5, you may declare. After declaring, your vote counts twice, but you cannot go on missions until mission 5."
                );
                seen_players.extend(
                    players
                        .iter()
                        .filter(|player| player.role.is_good())
                        .map(|player| player.role.to_string()),
                );
            }
            Role::Guinevere => {
                let _ = writeln!(&mut description,  "If you are not on a mission that is sent, you may choose one player on the mission to see that player's card.");
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
            Role::Maeve => {
                let _ = writeln!(&mut abilities, "{} times per game, and only once per round, during a vote on a proposal you may secretly obscure the voting so that only the number of upvotes and downvotes is shown.", spec.max_maeve_obscures);
            }
            Role::Agravaine => {
                let _ = writeln!(&mut abilities, "You may declare to fail a mission you were on that would have otherwise succeeded.");
            }
        }

        // Make sure the order of seen players doesn't leak info
        seen_players.shuffle(rng);

        let team_members = if self.is_evil() {
            players
                .evil_players()
                .iter()
                .filter(|player| **player != me)
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        let is_assassin = me == assassin;

        RoleDetails {
            team: self.team(),
            role: self,
            description,
            abilities,
            seen_players,
            team_members,
            other_info,
            assassinatable: self.is_assassinatable(),
            is_assassin,
            // In 5-player games, knowing the priority target is too powerful. For example, if the Good team is Lancelot and Lovers, and Lancelot
            // reverses, it immediately tells the Assassin who the Lovers are.
            // (We check against 6 players for future-proofing)
            priority_target: if is_assassin && spec.players > 6 {
                Some(priority_target)
            } else {
                None
            },
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Can use debug since it's still the role name
        writeln!(f, "{:?}", self)
    }
}

impl PriorityTarget {
    /// Checks if assassinating `player` as this target is correct.
    pub fn matches(self, player: &Player) -> bool {
        match self {
            PriorityTarget::Merlin => player.role == Role::Merlin,
            PriorityTarget::Guinevere => todo!("Need a Guinevere role"),
            PriorityTarget::Lovers => player.role.is_lover(),
            PriorityTarget::None => false,
        }
    }

    /// The number of expected players in an assassination attempt for this target.
    pub fn expected_targets(self) -> usize {
        match self {
            PriorityTarget::Lovers => 2,
            PriorityTarget::None => 0,
            _ => 1,
        }
    }
}
