//! THavalon game logic
//!
//! The game implementation is broken into several layers:
//! - [`GameSpec`] describes static game rules based on the number of players, such as the size of each mission and
//!   which roles may be in the game.
//! - [`Game`] holds configuration from when the game is rolled, such as which players have which roles and who the
//!   assassin is.
//! - [`GameState`] and [`Phase`] implement a state machine for while the game is running.
//! - [`Interactions`] abstracts over communication with the players.

use std::collections::HashMap;
use std::fmt;

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod builder;
mod engine;
mod interactions;
pub mod messages;
mod role;
pub mod snapshot;
mod state;

pub use self::messages::{Action, Message};
pub use self::role::*;

/// A mission number (from 1 to 5)
pub type MissionNumber = u8;

/// Game rules determined by the number of players
#[derive(Debug, Clone)]
pub struct GameSpec {
    /// Number of players in the game
    pub players: u8,
    /// The number of players on each mission
    pub mission_sizes: [usize; 5],
    /// Allowed good roles in the game
    pub good_roles: &'static [Role],
    /// Allowed evil roles in the game
    pub evil_roles: &'static [Role],
    /// The number of players on the good team
    pub good_players: u8,
    /// The maximum number of proposals allowed before force activates. Proposals on mission 1 and proposals that are
    /// sent do not count towards this limit.
    pub max_proposals: usize,
    /// The maximum number of times Maeve can obscure voting results in a game
    pub max_maeve_obscures: usize,
    /// True if mission 4 requires at least two failures
    double_fail_mission_four: bool,
}
/// Fixed information about a player, decided at startup
#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub role: Role,
}

/// A collection of players, indexed in various useful ways.
#[derive(Debug, Clone)]
pub struct Players {
    players: HashMap<String, Player>,
    roles: HashMap<Role, String>,
    good_players: Vec<String>,
    evil_players: Vec<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Card {
    Success,
    Fail,
    Reverse,
}

#[derive(Debug, Clone)]
pub struct Game {
    players: Players,
    info: HashMap<String, RoleDetails>,
    proposal_order: Vec<String>,
    assassin: String,
    priority_target: PriorityTarget,
    spec: &'static GameSpec,
}

#[derive(Debug, Clone, Error)]
pub enum CreateGameError {
    #[error("{0}-player games not supported")]
    UnsupportedSize(usize)
}

impl Game {
    pub fn roll(mut names: Vec<String>) -> Result<Game, CreateGameError> {
        let spec = GameSpec::for_players(names.len())?;
        let mut rng = thread_rng();

        let good_roles = spec
            .good_roles
            .choose_multiple(&mut rng, spec.good_players());
        let evil_roles = spec
            .evil_roles
            .choose_multiple(&mut rng, spec.evil_players());

        names.shuffle(&mut rng);
        let mut players = Players::new();
        for (role, name) in good_roles.chain(evil_roles).cloned().zip(names.into_iter()) {
            players.add_player(name, role);
        }

        let assassin = players
            .evil_players()
            .choose(&mut rng)
            .cloned()
            .expect("Could not choose an assassin, game contained no evil players");

        let mut priority_targets = Vec::new();
        if players.has_role(Role::Merlin) {
            priority_targets.push(PriorityTarget::Merlin);
        }
        if players.has_role(Role::Tristan) && players.has_role(Role::Iseult) {
            priority_targets.push(PriorityTarget::Lovers);
        }
        // TODO: Guinevere
        let priority_target = priority_targets
            .choose(&mut rng)
            .copied()
            .unwrap_or(PriorityTarget::None);

        let mut info = HashMap::with_capacity(players.len());
        for player in players.iter() {
            info.insert(
                player.name.clone(),
                player.role.generate_info(
                    &mut rng,
                    &player.name,
                    spec,
                    &players,
                    &assassin,
                    priority_target,
                ),
            );
        }

        let mut proposal_order = info.keys().cloned().collect::<Vec<_>>();
        proposal_order.shuffle(&mut rng);

        Ok(Game {
            players,
            info,
            proposal_order,
            assassin,
            priority_target,
            spec,
        })
    }

    pub fn proposal_order(&self) -> &[String] {
        self.proposal_order.as_slice()
    }

    /// Find the next player in proposal order after the given one.
    pub fn next_proposer(&self, player: &str) -> &str {
        let index = self
            .proposal_order
            .iter()
            .position(|p| *p == player)
            .unwrap();
        if index == self.proposal_order.len() - 1 {
            &self.proposal_order[0]
        } else {
            &self.proposal_order[index + 1]
        }
    }

    /// The number of players in the game
    pub fn size(&self) -> usize {
        self.proposal_order.len()
    }

    /// Look up the display name associated with a given role, if it exists.
    pub fn display_name_from_role(&self, role: Role) -> Option<&String> {
        self.info.iter().find_map(|(player, info)| if info.get_role() == role { Some(player) } else { None })
    }
}

impl Players {
    fn new() -> Players {
        Players {
            players: HashMap::new(),
            roles: HashMap::new(),
            good_players: Vec::new(),
            evil_players: Vec::new(),
        }
    }

    fn add_player(&mut self, name: String, role: Role) {
        self.roles.insert(role, name.clone());
        if role.is_good() {
            self.good_players.push(name.clone());
        } else {
            self.evil_players.push(name.clone());
        }
        self.players.insert(name.clone(), Player { name, role });
    }

    fn by_role(&self, role: Role) -> Option<&Player> {
        self.roles.get(&role).map(|name| &self.players[name])
    }

    fn has_role(&self, role: Role) -> bool {
        self.roles.contains_key(&role)
    }

    fn by_name(&self, name: &str) -> Option<&Player> {
        self.players.get(name)
    }

    fn good_players(&self) -> &[String] {
        self.good_players.as_slice()
    }

    fn evil_players(&self) -> &[String] {
        self.evil_players.as_slice()
    }

    fn iter(&self) -> impl Iterator<Item = &Player> {
        self.players.values()
    }

    fn len(&self) -> usize {
        self.players.len()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Card::Success => "Success",
            Card::Fail => "Fail",
            Card::Reverse => "Reverse",
        })
    }
}

impl GameSpec {
    pub fn for_players(players: usize) -> Result<&'static GameSpec, CreateGameError> {
        match players {
            2 => Ok(&TWO_PLAYER),
            3 => Ok(&THREE_PLAYER),
            4 => Ok(&FOUR_PLAYER),
            5 => Ok(&FIVE_PLAYER),
            7 => Ok(&SEVEN_PLAYER),
            _ => Err(CreateGameError::UnsupportedSize(players)),
        }
    }

    pub fn mission_size(&self, mission: MissionNumber) -> usize {
        self.mission_sizes[mission as usize - 1]
    }

    pub fn good_players(&self) -> usize {
        self.good_players as usize
    }

    pub fn evil_players(&self) -> usize {
        (self.players - self.good_players) as usize
    }

    /// The number of proposals in a round (after the first, which only has two)
    pub fn proposals(&self) -> usize {
        self.evil_players() + 1
    }

    pub fn double_fail_mission_four(&self) -> bool {
        self.double_fail_mission_four
    }

    /// Tests if `role` is allowed in games of this size
    pub fn has_role(&self, role: Role) -> bool {
        if role.is_evil() {
            self.evil_roles.contains(&role)
        } else {
            self.good_roles.contains(&role)
        }
    }
}

static FIVE_PLAYER: GameSpec = GameSpec {
    players: 5,
    mission_sizes: [2, 3, 2, 3, 3],
    good_roles: &[
        Role::Merlin,
        Role::Lancelot,
        Role::Percival,
        Role::Tristan,
        Role::Iseult,
        Role::Nimue,
    ],
    evil_roles: &[
        Role::Mordred,
        Role::Morgana,
        Role::Maelegant,
        Role::Maeve,
        Role::Agravaine,
    ],
    good_players: 3,
    max_proposals: 5,
    max_maeve_obscures: 2,
    double_fail_mission_four: false,
};

static SEVEN_PLAYER: GameSpec = GameSpec {
    players: 7,
    mission_sizes: [2, 3, 3, 4, 4],
    good_roles: &[
        Role::Merlin,
        Role::Lancelot,
        Role::Percival,
        Role::Tristan,
        Role::Iseult,
        Role::Nimue,
    ],
    evil_roles: &[
        Role::Mordred,
        Role::Morgana,
        Role::Maelegant,
        Role::Maeve,
        Role::Agravaine,
    ],
    good_players: 4,
    max_proposals: 7,
    max_maeve_obscures: 3,
    double_fail_mission_four: true,
};

/// Two-player games, for testing
static TWO_PLAYER: GameSpec = GameSpec {
    players: 2,
    mission_sizes: [1, 1, 2, 2, 2],
    good_roles: Role::ALL_GOOD,
    evil_roles: Role::ALL_EVIL,
    good_players: 1,
    max_proposals: 2,
    max_maeve_obscures: 2,
    double_fail_mission_four: false,
};

/// Three-player games, for testing
static THREE_PLAYER: GameSpec = GameSpec {
    players: 3,
    mission_sizes: [1, 2, 2, 2, 3],
    good_roles: Role::ALL_GOOD,
    evil_roles: Role::ALL_EVIL,
    good_players: 2,
    max_proposals: 3,
    max_maeve_obscures: 2,
    double_fail_mission_four: false,
};

static FOUR_PLAYER: GameSpec = GameSpec {
    players: 4,
    mission_sizes: [2, 2, 3, 3, 4],
    good_roles: Role::ALL_GOOD,
    evil_roles: Role::ALL_EVIL,
    good_players: 2,
    max_proposals: 4,
    max_maeve_obscures: 2,
    double_fail_mission_four: true,
};
