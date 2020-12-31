#![allow(dead_code)]
use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use super::messages::{Action, Message};
use super::role::Team;
use super::{Game, MissionNumber};

use self::assassination::Assassination;
use self::on_mission::{OnMission, WaitingForAgravaine};
use self::proposing::Proposing;
use self::role_state::RoleState;
use self::voting::Voting;

mod role_state;

/// Result of handling a player action. The [`GameStateWrapper`] is the new state of the game and the [`Effect`]
/// [`Vec`] describes side-effects of the state transition.
pub type ActionResult = (GameStateWrapper, Vec<Effect>);

/// Wrapper over specific phases, so callers can hold a game in any phase.
pub enum GameStateWrapper {
    Proposing(GameState<Proposing>),
    Voting(GameState<Voting>),
    OnMission(GameState<OnMission>),
    WaitingForAgravaine(GameState<WaitingForAgravaine>),
    Assassination(GameState<Assassination>),
    Done(GameState<Done>),
}

/// State of an in-progress game. Game state is divided into two parts. Data needed in all phases of the game,
/// such as player information, is stored in the `GameState` directly. Phase-specific data, such as how players
/// voted on a specific mission proposal, is stored in a particular [`Phase`] implementation.
pub struct GameState<P: Phase> {
    /// State specific to the current game phase.
    phase: P,
    /// Game configuration
    game: Game,
    role_state: RoleState,
    /// All proposals made in the game
    proposals: Vec<Proposal>,
    /// Results of all completed missions
    mission_results: Vec<MissionResults>,
}

/// Phase used when the game is over.
pub struct Done {
    winning_team: Team,
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

impl_phase!(Done);

// For Rust scoping reasons I don't quite understand, these have to come after the macro definition
mod assassination;
mod on_mission;
mod proposing;
mod voting;

/// A bundle of imports needed for most game phases
mod prelude {
    pub use super::{
        ActionResult, Done, Effect, GameState, GameStateWrapper, MissionResults, Phase, Proposal,
    };

    pub use super::assassination::Assassination;
    pub use super::on_mission::OnMission;
    pub use super::proposing::Proposing;
    pub use super::voting::Voting;

    pub use super::super::{
        messages::{self, Action, Message},
        role::{PriorityTarget, Role, Team},
        Card, Game, GameSpec,
    };
}

/// A side-effect of a state transition. In most cases, this will result in sending a message to some or all players.
#[derive(Debug)]
pub enum Effect {
    Reply(Message),
    Broadcast(Message),
    Send(String, Message),
    StartTimeout(Duration),
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

    /// Transition this game state into a new phase. All non-phase-specific state is copied over.
    fn with_phase<Q: Phase>(self, next_phase: Q) -> GameState<Q> {
        GameState {
            phase: next_phase,
            game: self.game,
            role_state: self.role_state,
            proposals: self.proposals,
            mission_results: self.mission_results,
        }
    }

    /// Switch into the `Proposing` state with `proposer` as the next player to propose. In addition to effects
    /// related to the next proposal, the returned [`ActionResult`] will include `effects`.
    fn into_proposing(self, proposer: String, mut effects: Vec<Effect>) -> ActionResult {
        effects.push(Effect::Broadcast(Message::NextProposal {
            proposer: proposer.clone(),
            mission: self.mission(),
            proposals_made: self.spent_proposals(),
            max_proposals: self.game.spec.max_proposals,
        }));
        let next_state = self.with_phase(Proposing::new(proposer));
        (GameStateWrapper::Proposing(next_state), effects)
    }

    /// Switch into the `Done` state with `winning_team` as the winners. The returned [`ActionResult`]
    /// will include `effects`.
    fn into_done(self, winning_team: Team, mut effects: Vec<Effect>) -> ActionResult {
        effects.push(Effect::Broadcast(Message::GameOver {
            winning_team,
            roles: self.game.info.clone(),
        }));
        let next_state = self.with_phase(Done::new(winning_team));
        (GameStateWrapper::Done(next_state), effects)
    }
}

/// Macro for repeating identical code across phases, with a fallback for any other phases.
///
/// # Examples
/// ```
/// in_phases!(
///   state,
///   Phase1 | Phase2 => |inner_state| do_stuff(inner_state),
///   |other_state| error_message(other_state)
/// )
/// ```
macro_rules! in_phases {
    // Arguments are divided into 3 parts:
    // - the expression to match on, of type GameStateWrapper
    // - included phases, separated by `|`, with a closure-like action
    // - a closure-like action for excluded phases
    ($wrapper:expr, $($phase:ident)|+ => |$inner:pat| $action:expr , |$other:pat| => $fallback:expr) => {
        match $wrapper {
            $(
                GameStateWrapper::$phase($inner) => $action,
            )+
            $other => $fallback
        }
    }
}

impl GameStateWrapper {
    /// Creates the [`GameState`] wrapper for a new game.
    pub fn new(game: Game) -> ActionResult {
        let first_proposer = &game.proposal_order()[0];
        let phase = Proposing::new(first_proposer.clone());

        let mut effects = vec![
            Effect::Broadcast(Message::ProposalOrder(game.proposal_order.clone())),
            Effect::Broadcast(Message::NextProposal {
                proposer: first_proposer.clone(),
                mission: 1,
                proposals_made: 0,
                max_proposals: game.spec.max_proposals,
            }),
        ];

        for player in game.players.iter() {
            effects.push(Effect::Send(
                player.name.clone(),
                Message::RoleInformation {
                    details: game.info[&player.name].clone(),
                },
            ));
        }

        // Send NextProposal last to move client to the proposal phase after
        // receiving role information.
        effects.push(Effect::Broadcast(Message::NextProposal {
            proposer: first_proposer.clone(),
            mission: 1,
            proposals_made: 0,
            max_proposals: game.spec.max_proposals,
        }));

        let mut role_state = RoleState::new(&game);
        role_state.on_round_start();

        let state = GameStateWrapper::Proposing(GameState {
            phase,
            role_state,
            game,
            proposals: vec![],
            mission_results: vec![],
        });
        (state, effects)
    }

    /// Advance to the next game state given a player action
    pub fn handle_action(self, player: &str, action: Action) -> ActionResult {
        log::debug!("Responding to {:?} from {}", action, player);
        match (self, action) {
            (GameStateWrapper::Proposing(inner), Action::Propose { players }) => {
                inner.handle_proposal(player, players)
            }
            (GameStateWrapper::Proposing(inner), Action::SelectPlayer { player: selected }) => {
                inner.handle_player_selected(player, selected)
            }
            (GameStateWrapper::Proposing(inner), Action::UnselectPlayer { player: unselected }) => {
                inner.handle_player_unselected(player, unselected)
            }
            (GameStateWrapper::Voting(inner), Action::Vote { upvote }) => {
                inner.handle_vote(player, upvote)
            }
            (GameStateWrapper::Voting(inner), Action::Obscure) => inner.handle_obscure(player),
            (GameStateWrapper::OnMission(inner), Action::Play { card }) => {
                inner.handle_card(player, card)
            }
            (GameStateWrapper::OnMission(inner), Action::QuestingBeast) => {
                inner.handle_questing_beast(player)
            }
            (GameStateWrapper::WaitingForAgravaine(inner), Action::Declare) => {
                inner.handle_declaration(player)
            }
            (GameStateWrapper::Assassination(inner), Action::Assassinate { target, players }) => {
                inner.handle_assassination(player, target, players)
            }

            (state, Action::MoveToAssassination) => {
                // For now, the in_phases! macro is somewhat overcomplicated, but it'll be useful for other cross-phase
                // actions like declarations
                in_phases!(state,
                    Proposing | Voting | OnMission | WaitingForAgravaine => |inner| inner.move_to_assassinate(player),
                    |state| => (state, vec![player_error("You can't move to assassination right now")])
                )
            }

            (state, _) => (state, vec![player_error("You can't do that right now")]),
        }
    }

    /// Handles a timeout set by [`Effect::SetTimeout`] expiring. This is used for player actions which must happen in a
    /// certain time window, like Agravaine declarations.
    pub fn handle_timeout(self) -> ActionResult {
        log::debug!("Action timeout expired");
        match self {
            GameStateWrapper::WaitingForAgravaine(inner) => inner.handle_timeout(),
            _ => {
                // This might happen if we transition to a new phase (like assassination) while a timeout is active.
                log::warn!("Timeout expired when no timeout should have been set");
                (self, vec![])
            }
        }
    }

    /// Returns whether or not the game is over
    pub fn is_done(&self) -> bool {
        matches!(self, GameStateWrapper::Done(_))
    }
}

impl Done {
    pub fn new(winning_team: Team) -> Done {
        Done { winning_team }
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
