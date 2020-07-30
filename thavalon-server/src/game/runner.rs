//! Asynchronous engine for running THavalon games

use std::collections::HashMap;

use log::info;

use tokio::sync::mpsc;
use tokio::stream::StreamMap;

use super::{Game, PlayerId, Card};

pub struct GameRunner {
    game: Game,
    
    /// Receivers for messages from players
    player_rx: StreamMap<PlayerId, mpsc::Receiver<PlayerMessage>>,

    /// Senders for each player 
    player_tx: HashMap<PlayerId, mpsc::Sender<GameMessage>>,
}

impl GameRunner {
    pub fn new(game: Game) -> (GameRunner, HashMap<PlayerId, (mpsc::Sender<PlayerMessage>, mpsc::Receiver<GameMessage>)>) {
        let mut player_rx = StreamMap::new();
        let mut player_tx = HashMap::new();
        let mut handles = HashMap::new();
        for player in game.players.iter() {
            let (ptx, prx) = mpsc::channel(10);
            player_rx.insert(player.id, prx);

            let (gtx, grx) = mpsc::channel(10);
            player_tx.insert(player.id, gtx);

            handles.insert(player.id, (ptx, grx));
        }

        (GameRunner { game, player_rx, player_tx }, handles)
    }

    pub async fn run(&mut self) {
        info!("Starting game");

        for player in self.game.players.iter() {
            info!("{} is {:?}", player.name, player.role);
        }
    }
}

/// A message from a player to the game engine
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PlayerMessage {
    Propose { players: Vec<PlayerId> },
    Vote { upvote: bool },
    Play { card: Card }
}

/// A message from the game engine to a player
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GameMessage {
    NewProposal { player: PlayerId },
    VotingResults {
        upvotes: Vec<PlayerId>,
        downvotes: Vec<PlayerId>,
        sent: bool,
    },
    MissionResults {
        successes: usize,
        fails: usize,
        reverses: usize,
        passed: bool
    }
}