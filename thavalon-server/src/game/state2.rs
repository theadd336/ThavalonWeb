use std::collections::{HashSet, HashMap};

use super::{Game, Card};
use super::messages::{Action, Message};
use super::role::PriorityTarget;

type ActionResult = (PhaseWrapper, Vec<Effect>);

trait Phase: Sized + Into<PhaseWrapper> {
    fn handle_action(self, game: &Game, player: &str, action: Action) -> ActionResult {
        match action {
            Action::Propose { players } => self.handle_proposal(game, player, players),
            Action::Vote { upvote } => self.handle_vote(game, player, upvote),
            Action::Play { card } => self.handle_play(game, player, card),
            Action::QuestingBeast => self.handle_questing_beast(game, player),
            Action::Declare => self.handle_declaration(game, player),
            Action::Assassinate { players, target } => self.handle_assassination(game, players, target),
            Action::MoveToAssassination => self.handle_move_to_assassination(game, player)
        }
    }

    fn handle_proposal(self, game: &Game, player: &str, proposal: HashSet<String>) -> ActionResult {
        (self.into(), vec![player_error("You can't do that right now")])
    }

    fn handle_vote(self, game: &Game, player: &str, upvote: bool) -> ActionResult {
        (self.into(), vec![player_error("You can't do that right now")])
    }

    fn handle_play(self, game: &Game, player: &str, card: Card) -> ActionResult {
        (self.into(), vec![player_error("You can't do that right now")])
    }

    fn handle_questing_beast(self, game: &Game, player: &str) -> ActionResult {
        (self.into(), vec![player_error("You can't do that right now")])
    }

    fn handle_declaration(self, game: &Game, player: &str) -> ActionResult {
        (self.into(), vec![player_error("You can't do that right now")])
    }

    fn handle_assassination(self, game: &Game, players: HashSet<String>, target: PriorityTarget) -> ActionResult {
        (self.into(), vec![player_error("You can't do that right now")])
    }

    fn handle_move_to_assassination(self, game: &Game, player: &str) -> ActionResult {
        (self.into(), vec![player_error("You can't do that right now")])
    }
}

enum PhaseWrapper {

}

enum Effect {
    Reply(Message),
    Broadcast(Message),
    StartTimeout,
    ClearTimeout
}

struct Proposing {
    
}

fn player_error<S: Into<String>>(message: S) -> Effect {
    Effect::Reply(Message::Error(message.into()))
}
