/// A THavalon role
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
