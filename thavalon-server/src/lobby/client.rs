//! Module containing the PlayerClient struct, which contains connections
//! to and from the game, lobby, and frontend.

use super::{LobbyChannel, LobbyCommand, LobbyResponse};
use crate::game::{Action, Message};

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot, Mutex,
    },
    task,
};
use warp::filters::ws::{self, WebSocket};

#[derive(Deserialize)]
#[serde(tag = "message_type", content = "data")]
enum IncomingMessage {
    Ping,
    StartGame,
    GameCommand(Action),
}

#[derive(Serialize)]
#[serde(tag = "message_type", content = "data")]
enum OutgoingMessage {
    Pong(String),
    GameMessage(Message),
}

#[derive(Hash, PartialEq)]
enum TaskType {
    FromLobby,
    FromGame,
    FromClient,
    ToClient,
}

#[derive(Debug)]
enum OutboundTaskMessageType {
    ToClient(String),
    NewWebSocket(SplitSink<WebSocket, ws::Message>),
}
pub struct PlayerClient {
    tasks: Vec<String>,
    client_id: String,
    to_lobby: LobbyChannel,
    to_game: Sender<Action>,
    to_outbound_task: Sender<OutboundTaskMessageType>,
}

impl PlayerClient {
    pub fn new(
        client_id: String,
        to_lobby: LobbyChannel,
        to_game: Sender<Action>,
        from_game: Receiver<Message>,
        to_client: SplitSink<WebSocket, ws::Message>,
        mut from_client: SplitStream<WebSocket>,
    ) -> Self {
        let (lobby_tx, lobby_rx) = mpsc::channel(10);
        let (to_outbound_task_tx, to_outbound_task_rx) =
            mpsc::channel::<OutboundTaskMessageType>(10);
        let client = PlayerClient {
            client_id,
            tasks: Vec::new(),
            to_lobby,
            to_game,
            to_outbound_task: to_outbound_task_tx,
        };

        client.spawn_stable_tasks(from_game, lobby_rx, to_client, to_outbound_task_rx);
        client.spawn_from_client_task(from_client);
        client
    }

    fn spawn_stable_tasks(
        &self,
        mut from_game: Receiver<Message>,
        mut from_lobby: Receiver<String>,
        mut to_client: SplitSink<WebSocket, ws::Message>,
        mut outbound_task_rx: Receiver<OutboundTaskMessageType>,
    ) {
        log::info!("Creating stable tasks for client {}.", self.client_id);
        // Task to manage messages from the game.
        let mut game_to_outbound_task = self.to_outbound_task.clone();
        let client_id = self.client_id.clone();
        task::spawn(async move {
            while let Some(game_msg) = from_game.recv().await {
                log::debug!(
                    "Received game message {:?} from client {}.",
                    game_msg,
                    client_id
                );
                let game_msg = serde_json::to_string(&game_msg).unwrap();

                // Can't unwrap, but this should never fail, since the task is
                // stable.
                let _ = game_to_outbound_task
                    .send(OutboundTaskMessageType::ToClient(game_msg))
                    .await;
            }
        });

        // Task to manage messages from the lobby
        let mut lobby_to_outbound_task = self.to_outbound_task.clone();
        let client_id = self.client_id.clone();
        task::spawn(async move {
            while let Some(lobby_msg) = from_lobby.recv().await {
                log::debug!(
                    "Received lobby message {} for client {}.",
                    lobby_msg,
                    client_id
                );
                let outbound_task_msg = OutboundTaskMessageType::ToClient(lobby_msg);

                // Can't unwrap, but this should never fail, since the task is
                // stable.
                let _ = lobby_to_outbound_task.send(outbound_task_msg).await;
            }
        });

        // Task to manage all incoming messages and send them to the client.
        let client_id = self.client_id.clone();
        task::spawn(async move {
            while let Some(outbound_msg) = outbound_task_rx.recv().await {
                log::debug!(
                    "Received outbound message {:?} for client {}.",
                    outbound_msg,
                    client_id
                );
                match outbound_msg {
                    OutboundTaskMessageType::ToClient(msg) => {
                        log::debug!("Sending message {} to client {}.", msg, client_id);
                        let msg = ws::Message::text(&msg);
                        if let Err(e) = to_client.send(msg).await {
                            log::error!(
                                "Error while sending message to client {}. {}.",
                                client_id,
                                e
                            );
                            continue;
                        }
                        log::debug!("Successfully sent message to client {}.", client_id);
                    }

                    OutboundTaskMessageType::NewWebSocket(ws) => {
                        log::info!("Received new connection for client {}.", client_id);
                        to_client = ws;
                    }
                }
            }
        });

        log::info!(
            "Successfully spawned all stable tasks for client {}.",
            self.client_id
        );
    }

    fn spawn_from_client_task(&self, mut from_client: SplitStream<WebSocket>) {
        log::info!(
            "Creating the client receiver task for client {}.",
            self.client_id
        );

        let mut to_game = self.to_game.clone();
        let mut to_lobby = self.to_lobby.clone();
        let client_id = self.client_id.clone();
        task::spawn(async move {
            while let Some(incoming_msg) = from_client.next().await {
                if let Err(e) = incoming_msg {
                    log::error!("An error occurred while reading messages from the incoming connection for client {}. {}", client_id, e);
                    return;
                }

                let incoming_msg = incoming_msg.unwrap();
                log::debug!(
                    "Received incoming message {:?} from client {}.",
                    incoming_msg,
                    client_id
                );

                let incoming_msg = incoming_msg.to_str().unwrap();
                log::debug!(
                    "Attempting to deserialize message {} from client {}.",
                    incoming_msg,
                    client_id
                );

                let incoming_msg: IncomingMessage = match serde_json::from_str(incoming_msg) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::error!(
                            "Failed to deserialize incoming message for client {}. {}",
                            client_id,
                            e
                        );
                        panic!();
                        // // TODO: Implement sending an error code to the client.
                        // todo!()
                    }
                };

                match incoming_msg {
                    IncomingMessage::Ping => {
                        let _ = to_lobby
                            .send((
                                LobbyCommand::Ping {
                                    client_id: client_id.clone(),
                                },
                                None,
                            ))
                            .await;
                    }
                    IncomingMessage::StartGame => {
                        let _ = to_lobby.send((LobbyCommand::StartGame, None)).await;
                    }
                    IncomingMessage::GameCommand(cmd) => {
                        let _ = to_game.send(cmd).await;
                    }
                }
            }
        });
    }
}
