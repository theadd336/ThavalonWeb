//! Tracks game state related to individual roles, such as how many uses of an ability are left.

use super::prelude::*;

pub struct RoleState {
    pub maeve: MaeveState,
}

pub struct MaeveState {
    obscures_remaining: usize,
    obscured_this_round: bool,
}

impl RoleState {
    pub fn new(game: &Game) -> RoleState {
        RoleState {
            maeve: MaeveState::new(game.spec),
        }
    }

    /// Updates role state at the start of each round.
    pub fn on_round_start(&mut self) {
        self.maeve.on_round_start();
    }
}

impl MaeveState {
    fn new(spec: &GameSpec) -> MaeveState {
        MaeveState {
            obscures_remaining: spec.max_proposals,
            obscured_this_round: false,
        }
    }

    fn on_round_start(&mut self) {
        self.obscured_this_round = false;
    }

    /// Checks if Maeve is allowed to use her ability
    pub fn can_obscure(&self) -> bool {
        !self.obscured_this_round && self.obscures_remaining > 0
    }

    /// Records when Maeve uses her ability.
    pub fn mark_obscure(&mut self) {
        self.obscured_this_round = true;
        self.obscures_remaining -= 1;
    }
}
