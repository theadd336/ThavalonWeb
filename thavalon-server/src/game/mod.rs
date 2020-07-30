//! THavalon game logic

use std::collections::HashMap;

use rand::prelude::*;

pub mod runner;
mod model;

pub use self::model::*;

/// Key for identifying a player in the game. Cheaper to copy and move around than a String
pub type PlayerId = usize;

pub struct Game {
    /// Static information about each player
    player_info: HashMap<PlayerId, PlayerInfo>,

    /// Proposal / seating order
    proposal_order: Vec<PlayerId>,

    /// State machine for game progression
    state: GameState,

    /// Convenience reference to the GameSpec
    spec: &'static GameSpec
}

impl Game {
    pub fn roll(mut players: Vec<String>) -> Game {
        let spec = GameSpec::for_players(players.len());
        let mut rng = thread_rng();

        let good_roles = spec.good_roles.choose_multiple(&mut rng, spec.good_players);
        let evil_roles = spec.evil_roles.choose_multiple(&mut rng, players.len() - spec.good_players);

        players.shuffle(&mut rng);
        let mut infos = HashMap::with_capacity(players.len());
        let mut iter = players.into_iter().enumerate();
        
        for good_role in good_roles.into_iter() {
            let (player_id, name) = iter.next().unwrap();
            infos.insert(player_id, PlayerInfo {
                name,
                role: *good_role,
                information: String::new()
            });
        }

        for evil_role in evil_roles.into_iter() {
            let (player_id, name) = iter.next().unwrap();
            infos.insert(player_id, PlayerInfo {
                name,
                role: *evil_role,
                information: String::new()
            });
        }

        let mut proposal_order = infos.keys().cloned().collect::<Vec<_>>();
        proposal_order.shuffle(&mut rng);

        Game {
            player_info: infos,
            proposal_order,
            state: GameState::Pregame,
            spec,
        }
    }

    pub fn player_info(&self, id: PlayerId) -> &PlayerInfo {
        &self.player_info[&id]
    }

    pub fn proposal_order(&self) -> &[PlayerId] {
        self.proposal_order.as_slice()
    }
}

/// Fixed information about a player, decided at startup
pub struct PlayerInfo {
    pub name: String,
    pub role: Role,
    /// Role-based information
    pub information: String,
}

enum GameState {
    Pregame,
    Proposing,
    Voting,
    Mission,
    Assassination,
    Postgame
}
