//! THavalon game logic

use std::collections::HashMap;

use rand::prelude::*;

pub mod runner;
mod role;

pub use self::role::*;

/// Key for identifying a player in the game. Cheaper to copy and move around than a String
pub type PlayerId = usize;

pub struct Game {
    players: Players,
    info: HashMap<PlayerId, String>,
    proposal_order: Vec<PlayerId>,
    spec: &'static GameSpec
}

impl Game {
    pub fn roll(mut names: Vec<String>) -> Game {
        let spec = GameSpec::for_players(names.len());
        let mut rng = thread_rng();

        let good_roles = spec.good_roles.choose_multiple(&mut rng, spec.good_players);
        let evil_roles = spec.evil_roles.choose_multiple(&mut rng, names.len() - spec.good_players);

        names.shuffle(&mut rng);
        let mut players = Players::new();
        for (id, (role, name)) in good_roles.chain(evil_roles).cloned().zip(names.into_iter()).enumerate() {
            players.add_player(Player {
                id,
                role,
                name
            });
        }

        let mut info = HashMap::with_capacity(players.len());
        for player in players.iter() {
            info.insert(player.id, player.role.generate_info(&mut rng, player.id, &players));
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
}

/// Fixed information about a player, decided at startup
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub role: Role,
}

/// A collection of players, indexed in various useful ways.
pub struct Players {
    players: HashMap<PlayerId, Player>,
    roles: HashMap<Role, PlayerId>,
    good_players: Vec<PlayerId>,
    evil_players: Vec<PlayerId>
}

impl Players {
    fn new() -> Players {
        Players {
            players: HashMap::new(),
            roles: HashMap::new(),
            good_players: Vec::new(),
            evil_players: Vec::new()
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

    fn get(&self, id: PlayerId) -> &Player {
        &self.players[&id]
    }

    fn by_role(&self, role: Role) -> Option<&Player> {
        self.roles.get(&role).map(|id| self.get(*id))
    }

    fn good_players(&self) -> &[PlayerId] {
        self.good_players.as_slice()
    }

    fn evil_players(&self) -> &[PlayerId] {
        self.evil_players.as_slice()
    }

    fn iter(&self) -> impl Iterator<Item=&Player> {
        self.players.values()
    }

    fn len(&self) -> usize {
        self.players.len()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Card {
    Success,
    Fail,
    Reverse,
}

/// Game rules determined by the number of players
#[derive(Debug, Clone)]
pub struct GameSpec {
    /// The number of players on each mission
    pub mission_sizes: [usize; 5],
    /// Allowed good roles in the game
    pub good_roles: &'static [Role],
    /// Allowed evil roles in the game
    pub evil_roles: &'static [Role],
    /// The number of players on the good team
    pub good_players: usize,
}

impl GameSpec {
    pub fn for_players(players: usize) -> &'static GameSpec {
        match players {
            5 => &FIVE_PLAYER,
            _ => panic!("{}-player games not supported", players)
        }
    }
}

static FIVE_PLAYER: GameSpec = GameSpec {
    mission_sizes: [2, 3, 2, 3, 3],
    good_roles: &[
        Role::Merlin, Role::Lancelot, Role::Percival, Role::Tristan, Role::Iseult,
    ],
    evil_roles: &[
        Role::Mordred, Role::Morgana, Role::Maelegant
    ],
    good_players: 3,
};
