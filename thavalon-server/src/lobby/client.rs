//! Module containing the PlayerClient struct, which contains connections
//! to and from the game, lobby, and frontend.

use super::{LobbyChannel, LobbyCommand, LobbyResponse};
use crate::game::{Action, Message};

use std::collections::HashMap;

use futures::{
    future::{AbortHandle, Abortable},
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
pub enum OutgoingMessage {
    Pong(String),
    GameMessage(Message),
}

#[derive(Hash, PartialEq, Eq)]
enum TaskType {
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
    tasks: HashMap<TaskType, AbortHandle>,
    client_id: String,
    to_lobby: LobbyChannel,
    to_game: Sender<Action>,
    to_outbound_task: Sender<OutboundTaskMessageType>,
    oubound_task_receiver: Option<Receiver<OutboundTaskMessageType>>,
}

impl PlayerClient {
    pub fn new(
        client_id: String,
        to_lobby: LobbyChannel,
        to_game: Sender<Action>,
        from_game: Receiver<Message>,
    ) -> Self {
        let (to_outbound_task_tx, to_outbound_task_rx) =
            mpsc::channel::<OutboundTaskMessageType>(10);
        let mut client = PlayerClient {
            client_id,
            tasks: HashMap::new(),
            to_lobby,
            to_game,
            to_outbound_task: to_outbound_task_tx,
            oubound_task_receiver: Some(to_outbound_task_rx),
        };

        client.spawn_from_game_task(from_game);
        client
    }

    pub async fn send_message(&mut self, message: String) {
        log::debug!(
            "Received message {} from lobby to send to client {}.",
            message,
            self.client_id
        );

        let _ = self
            .to_outbound_task
            .send(OutboundTaskMessageType::ToClient(message))
            .await;
    }

    fn spawn_from_game_task(&mut self, mut from_game: Receiver<Message>) {
        log::info!("Creating stable tasks for client {}.", self.client_id);
        // Task to manage messages from the game.
        let mut game_to_outbound_task = self.to_outbound_task.clone();
        let client_id = self.client_id.clone();

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(
            async move {
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
            },
            abort_registration,
        );
        self.tasks.insert(TaskType::FromGame, abort_handle);
        task::spawn(future);

        log::info!(
            "Successfully spawned all stable tasks for client {}.",
            self.client_id
        );
    }

    pub async fn update_websockets(&mut self, ws: WebSocket) {
        log::info!(
            "Creating the client receiver task for client {}.",
            self.client_id
        );

        let (to_client, mut from_client) = ws.split();
        if self.tasks.contains_key(&TaskType::ToClient) {
            self.update_outgoing_ws_task(to_client).await;
        } else {
            self.create_outgoing_ws_task(to_client);
        }

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let mut to_game = self.to_game.clone();
        let mut to_lobby = self.to_lobby.clone();
        let client_id = self.client_id.clone();
        let outgoing_to_client_future = Abortable::new(
            async move {
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
            },
            abort_registration,
        );
        task::spawn(outgoing_to_client_future);

        if let Some(abort_handle) = self.tasks.insert(TaskType::FromClient, abort_handle) {
            abort_handle.abort();
        }
    }

    async fn update_outgoing_ws_task(&mut self, new_ws: SplitSink<WebSocket, ws::Message>) {
        log::debug!(
            "Updating the outgoing WS task for client {}.",
            self.client_id
        );
        let _ = self
            .to_outbound_task
            .send(OutboundTaskMessageType::NewWebSocket(new_ws))
            .await;
    }

    fn create_outgoing_ws_task(&mut self, mut to_client: SplitSink<WebSocket, ws::Message>) {
        // Task to manage all incoming messages and send them to the client.
        log::debug!(
            "Creating a new outgoing WS task for client {}.",
            self.client_id
        );
        let client_id = self.client_id.clone();
        let mut outbound_task_rx = self.oubound_task_receiver.take().unwrap();
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let outbound_to_client_future = Abortable::new(
            async move {
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
            },
            abort_registration,
        );
        self.tasks.insert(TaskType::ToClient, abort_handle);
        task::spawn(outbound_to_client_future);
    }
}
