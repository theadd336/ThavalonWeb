#![allow(dead_code)]
use std::collections::HashSet;
use std::fmt;

use super::messages::{Action, Message};
use super::{Game, MissionNumber};

use self::proposing::Proposing;
use self::voting::Voting;
use self::on_mission::OnMission;

/// Result of handling a player action. The [`GameStateWrapper`] is the new state of the game and the [`Effect`]
/// [`Vec`] describes side-effects of the state transition.
pub type ActionResult = (GameStateWrapper, Vec<Effect>);

/// Wrapper over specific phases, so callers can hold a game in any phase.
pub enum GameStateWrapper {
    Proposing(GameState<Proposing>),
    Voting(GameState<Voting>),
    OnMission(GameState<OnMission>),
}

/// State of an in-progress game. Game state is divided into two parts. Data needed in all phases of the game,
/// such as player information, is stored in the `GameState` directly. Phase-specific data, such as how players
/// voted on a specific mission proposal, is stored in a particular [`Phase`] implementation.
pub struct GameState<P: Phase> {
    /// State specific to the current game phase.
    phase: P,
    /// Game configuration
    game: Game,
    /// All proposals made in the game
    proposals: Vec<Proposal>,
    /// Results of all completed missions
    mission_results: Vec<MissionResults>,
}

/// A phase of the THavalon state machine
pub trait Phase: Sized {
    /// Lifts a game in this phase back into the [`GameStateWrapper`] enum. This is used by code
    /// that is generic over different game phases and needs a wrapped version of the game.
    fn wrap(game: GameState<Self>) -> GameStateWrapper;
}

// Boilerplate for wrapping game phases into an enum
macro_rules! impl_phase {
    ($phase:ident) => {
        impl $crate::game::state::Phase for $phase {
            fn wrap(game: GameState<Self>) -> GameStateWrapper {
                GameStateWrapper::$phase(game)
            }
        }
    };
}

// For Rust scoping reasons I don't quite understand, these have to come after the macro definition
mod proposing;
mod voting;
mod on_mission;

/// A bundle of imports needed for most game phases
mod prelude {
    pub use super::{GameState, GameStateWrapper, ActionResult, Effect, Proposal, MissionResults};

    pub use super::proposing::Proposing;
    pub use super::voting::Voting;
    pub use super::on_mission::OnMission;

    pub use super::super::{
        Game, GameSpec, Card,
        messages::{self, Action, Message},
        role::Role,
    };
}

/// A side-effect of a state transition. In most cases, this will result in sending a message to some or all players.
pub enum Effect {
    Reply(Message),
    Broadcast(Message),
    StartTimeout,
    ClearTimeout,
}

pub struct Proposal {
    proposer: String,
    players: HashSet<String>,
}

pub struct MissionResults {
    passed: bool,
    players: HashSet<String>,
}


// Convenience methods shared across game phases
impl<P: Phase> GameState<P> {
    /// Generate an [`ActionResult`] that keeps the current state and returns an error reply to the player.
    fn player_error<S: Into<String>>(self, message: S) -> ActionResult {
        (P::wrap(self), vec![player_error(message)])
    }

    /// The current mission, indexed starting at 1
    fn mission(&self) -> MissionNumber {
        self.mission_results.len() as u8 + 1
    }

    /// Calculates the number of "spent" proposals, for the purposes of determining if force is active
    /// - The two proposals on mission 1 do not count
    /// - Proposals that are sent do not count. Equivalently, every time a mission is sent we get a proposal back
    ///
    /// *This will be off by 1 while going on a mission, since the spent proposal has not yet been returned*
    fn spent_proposals(&self) -> usize {
        self.proposals
            .len()
            .saturating_sub(2) // Subtract 2 proposals for mission 1
            .saturating_sub(self.mission_results.len()) // Subtract 1 proposal for each sent mission
    }
}

impl GameStateWrapper {
    /// Advance to the next game state given a player action
    fn handle_action(self, player: &str, action: Action) -> ActionResult {
        log::debug!("Responding to {:?} from {}", action, player);
        match (self, action) {
            (GameStateWrapper::Proposing(inner), Action::Propose { players }) => {
                inner.handle_proposal(player, players)
            }
            (GameStateWrapper::Voting(inner), Action::Vote { upvote }) => {
                inner.handle_vote(player, upvote)
            }
            (GameStateWrapper::OnMission(inner), Action::Play { card }) => {
                inner.handle_card(player, card)
            }
            (GameStateWrapper::OnMission(inner), Action::QuestingBeast) => {
                inner.handle_questing_beast(player)
            }
            (state, _) => (state, vec![player_error("You can't do that right now")]),
        }
    }
}

impl fmt::Display for Proposal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use itertools::Itertools;
        write!(
            f,
            "{} (proposed by {})",
            self.players.iter().format(", "),
            self.proposer
        )
    }
}

/// Generate an [`Effect`] that sends an error reply to the player.
fn player_error<S: Into<String>>(message: S) -> Effect {
    Effect::Reply(Message::Error(message.into()))
}
