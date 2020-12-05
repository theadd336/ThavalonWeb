use std::collections::{HashMap, HashSet};

use super::interactions::Interactions;
use super::messages::{Action, Message};
use super::Game;

// Game logic implemented as an impure state machine. Instead of having states consume themselves and return new
// states, GameState mutates itself in-place to reduce the amount of cross-phase bookkeeping.

struct GameState {
    game: Game,
    phase: Phase,
    proposals: Vec<Proposal>,
    passed_missions: usize,
    failed_missions: usize,
}

enum Phase {
    Starting,
    Proposing {
        proposer: String,
    },
    Voting,
    OnMission,
    TimedDeclaration,
    Assassination,
    Done
}

enum Effect {
    Reply(Message),
    Broadcast(Message),
    StartTimeout,
    ClearTimeout
}

struct Proposal {
    proposer: String,
    players: HashSet<String>,
}

impl GameState {
    pub fn handle_action(&mut self, player: &str, action: Action) -> Vec<Effect> {
        let mission_number = passed_missions + failed_missions + 1;

        let (next_phase, effects) = match (phase, action) {
            (Phase::Proposing { proposer }, Action::Propose { players }) => {
                if player != proposer {
                    (Phase::Proposing { proposer }, vec![player_error("It's not your proposal")])
                } else if valid_proposal(game, &players, mission_number) {
                    proposals.push(Proposal {
                        proposer: proposer.clone(),
                        players: players.clone(),
                    });

                    let mut effects = vec![
                        Effect::Broadcast(Message::ProposalMade {
                            proposer,
                            players: players.clone(),
                            mission: mission_number as u8,
                        })
                    ];

                    let phase = if mission_number == 1 && proposals.len() == 1 {
                        Phase::Proposing { proposer: game.next_proposer(player).to_string() }
                    } else if proposals.len() > game.spec.max_proposals {
                        effects.push(Effect::Broadcast(Message::MissionGoing {
                            mission: mission_number as u8,
                            players,
                        }));
                        Phase::OnMission
                    } else {
                        effects.push(Effect::Broadcast(Message::CommenceVoting));
                        Phase::Voting
                    };

                    (phase, effects)
                } else {
                    (Phase::Proposing { proposer }, vec![player_error("Not a valid proposal")])
                }
            },

            (Phase::Assassination, Action::Assassinate { target, players }) => todo!("assassination"),
            (phase, Action::MoveToAssassination) => {
                if player == game.assassin {
                    (Phase::Assassination, todo!("assassination message"))
                } else {
                    (phase, vec![player_error("You are not the assassin")])
                }
            },


            (phase, _) => (phase, vec![player_error("You can't do that right now")])
        };

        let next_game = GameState { phase: next_phase, proposals, passed_missions, failed_missions };
        (next_game, effects)
    }

    pub fn handle_timeout(self) -> (GameState, Vec<Effect>) {
        (self, vec![])
    }

}

fn player_error<S: Into<String>>(message: S) -> Effect {
    Effect::Reply(Message::Error(message.into()))
}

fn valid_proposal(game: &Game, players: &HashSet<String>, mission_number: usize) -> bool {
    let mission_size = game.spec.mission_size(mission_number as u8);
    if players.len() != mission_size {
        return false
    }

    for player in players.iter() {
        if game.players.by_name(player).is_none() {
            return false
        }
    }

    true
}


// For nicer error handling / validation, have an error enum representing
// - validation errors to send back to the player
// - communication errors (non-fatal?)
// - (possibly) fatal game logic errors
// this means I can have a bunch of states that share helper logic (ex. for special mission 1 proposals)
// have game (e.g.) keep a log of all proposals so for voting (esp mission 1) we can just look back

/*

Hierarchical State Machines:

GameState
|- Mission
|  |- Proposing
|  |- Voting
|  |- Going
|  |- WaitingForAgravaine
|- Assassination
|- Done

But list of proposals should be global :/

*/

trait StateMachine: Sized {
    type Event;
    type Command;

    fn handle(self, event: Self::Event) -> (Self, Vec<Self::Command>);
}