//! Tracks game state related to individual roles, such as how many uses of an ability are left.

use super::prelude::*;

pub struct RoleState {
    pub maeve: MaeveState,
    pub arthur: ArthurState,
}

pub struct MaeveState {
    obscures_remaining: usize,
    obscured_this_round: bool,
}

pub struct ArthurState {
    has_declared: bool,
}

impl RoleState {
    pub fn new(game: &Game) -> RoleState {
        RoleState {
            maeve: MaeveState::new(game.spec),
            arthur: ArthurState::new(),
        }
    }

    /// Updates role state at the start of each round. This has an unusual signature (taking `&mut GameState` instead of `&mut self`) because
    /// some roles require the entire game state to update.
    pub fn on_round_start<P: Phase>(state: &mut GameState<P>, effects: &mut Vec<Effect>) {
        state.role_state.maeve.on_round_start();
        state.role_state.arthur.on_round_start(state, effects);
    }
}

impl MaeveState {
    fn new(spec: &GameSpec) -> MaeveState {
        MaeveState {
            obscures_remaining: spec.max_maeve_obscures,
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

impl ArthurState {
    fn new() -> ArthurState {
        ArthurState {
            has_declared: false,
        }
    }

    pub fn declare(&mut self) {
        self.has_declared = true;
    }

    /// Checks if Arthur has already declared
    pub fn has_declared(&self) -> bool {
        self.has_declared
    }

    /// Checks if Arthur is allowed to declare
    pub fn can_declare<P: Phase>(&self, state: &GameState<P>) -> bool {
        if self.has_declared || state.mission() == 5 {
            // Arthur cannot declare on mission 5
            false
        } else {
            // Arthur can declare if 2 missions have failed
            state.mission_results.iter().filter(|m| !m.passed).count() == 2
        }
    }

    fn on_round_start<P: Phase>(&self, state: &GameState<P>, effects: &mut Vec<Effect>) {
        if let Some(arthur) = state.game.players.by_role(Role::Arthur) {
            let message = if self.can_declare(state) {
                Message::ArthurCanDeclare
            } else {
                Message::ArthurCannotDeclare
            };
            effects.push(Effect::Send(arthur.name.clone(), message));
        }
    }
}
