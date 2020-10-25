//! THavalon game logic

use std::collections::HashMap;
use std::fmt;
use std::ops::Index;

use rand::prelude::*;

pub mod builder;
mod engine;
mod interactions;
mod messages;
mod role;

pub use self::messages::*;
pub use self::role::*;

/// Key for identifying a player in the game. Cheaper to copy and move around than a String
pub type PlayerId = usize;

/// A mission number (from 1 to 5)
pub type MissionNumber = u8;

/// A proposal number (starts at 0)
pub type ProposalNumber = u8;

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
    /// True if mission 4 requires at least two failures
    double_fail_mission_four: bool,
}
/// Fixed information about a player, decided at startup
#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub role: Role,
}

/// A collection of players, indexed in various useful ways.
#[derive(Debug, Clone)]
pub struct Players {
    players: HashMap<PlayerId, Player>,
    roles: HashMap<Role, PlayerId>,
    good_players: Vec<PlayerId>,
    evil_players: Vec<PlayerId>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Card {
    Success,
    Fail,
    Reverse,
}

#[derive(Debug, Clone)]
pub struct Game {
    players: Players,
    info: HashMap<PlayerId, RoleDetails>,
    proposal_order: Vec<PlayerId>,
    spec: &'static GameSpec,
}

impl Game {
    pub fn roll(mut names: Vec<(PlayerId, String)>) -> Game {
        let spec = GameSpec::for_players(names.len());
        let mut rng = thread_rng();

        let good_roles = spec
            .good_roles
            .choose_multiple(&mut rng, spec.good_players());
        let evil_roles = spec
            .evil_roles
            .choose_multiple(&mut rng, spec.evil_players());

        names.shuffle(&mut rng);
        let mut players = Players::new();
        for (role, (id, name)) in good_roles.chain(evil_roles).cloned().zip(names.into_iter()) {
            players.add_player(Player { id, role, name });
        }

        let mut info = HashMap::with_capacity(players.len());
        for player in players.iter() {
            info.insert(
                player.id,
                player.role.generate_info(&mut rng, player.id, &players),
            );
        }

        let mut proposal_order = info.keys().cloned().collect::<Vec<_>>();
        proposal_order.shuffle(&mut rng);

        Game {
            players,
            info,
            proposal_order,
            spec,
        }
    }

    pub fn proposal_order(&self) -> &[PlayerId] {
        self.proposal_order.as_slice()
    }

    /// Find the next player in proposal order after the given one.
    pub fn next_proposer(&self, player: PlayerId) -> PlayerId {
        let index = self
            .proposal_order
            .iter()
            .position(|p| *p == player)
            .unwrap();
        if index == self.proposal_order.len() - 1 {
            self.proposal_order[0]
        } else {
            self.proposal_order[index + 1]
        }
    }

    pub fn name(&self, player: usize) -> &str {
        self.players[player].name.as_ref()
    }

    /// The number of players in the game
    pub fn size(&self) -> usize {
        self.proposal_order.len()
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

    fn add_player(&mut self, player: Player) {
        self.roles.insert(player.role, player.id);
        if player.role.is_good() {
            self.good_players.push(player.id);
        } else {
            self.evil_players.push(player.id);
        }
        self.players.insert(player.id, player);
    }

    fn by_role(&self, role: Role) -> Option<&Player> {
        self.roles.get(&role).map(|id| &self[*id])
    }

    fn has_role(&self, role: Role) -> bool {
        self.roles.contains_key(&role)
    }

    fn good_players(&self) -> &[PlayerId] {
        self.good_players.as_slice()
    }

    fn evil_players(&self) -> &[PlayerId] {
        self.evil_players.as_slice()
    }

    fn iter(&self) -> impl Iterator<Item = &Player> {
        self.players.values()
    }

    fn len(&self) -> usize {
        self.players.len()
    }
}

impl Index<PlayerId> for Players {
    type Output = Player;

    fn index(&self, player_id: usize) -> &Self::Output {
        &self.players[&player_id]
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
    pub fn for_players(players: usize) -> &'static GameSpec {
        match players {
            5 => &FIVE_PLAYER,
            _ => panic!("{}-player games not supported", players),
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
    ],
    evil_roles: &[
        Role::Mordred,
        Role::Morgana,
        Role::Maelegant,
        Role::Agravaine,
    ],
    good_players: 3,
    double_fail_mission_four: false,
};
