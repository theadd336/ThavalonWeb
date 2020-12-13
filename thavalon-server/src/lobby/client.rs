//! Module containing the PlayerClient struct, which contains connections
//! to and from the game, lobby, and frontend.

use super::{LobbyChannel, LobbyCommand};
use crate::game::{Action, Message};

use std::collections::HashMap;

use futures::{
    future::{AbortHandle, Abortable},
    stream::SplitSink,
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task,
};
use warp::filters::ws::{self, WebSocket};

/// An incoming message from the client.
#[derive(Deserialize)]
#[serde(tag = "messageType", content = "data")]
enum IncomingMessage {
    Ping,
    StartGame,
    GameCommand(Action),
}

/// An outgoing message to the client.
#[derive(Serialize)]
#[serde(tag = "messageType", content = "data")]
pub enum OutgoingMessage {
    Pong(String),
    GameMessage(Message),
    PlayerList(Vec<String>),
    StartGame,
}

/// Task types that the PlayerClient maintains
#[derive(Hash, PartialEq, Eq)]
enum TaskType {
    FromGame,
    FromClient,
    ToClient,
}

/// Message types that can be sent to the outbound messaging task.
/// Most of the time, this will be ToClient, which will forward the message to the client.
/// However, in the event of a new connection, the NewWebSocket can be used to update
/// the outgoing WS connection without needing to recreate the task.
#[derive(Debug)]
enum OutboundTaskMessageType {
    ToClient(String),
    NewWebSocket(SplitSink<WebSocket, ws::Message>),
}

/// Manages the connection to the actual player.
/// `PlayerClient` maintains all the connection tasks, updating and remaking them
/// as needed. The struct also maintains connections to the lobby and to the game.
pub struct PlayerClient {
    tasks: HashMap<TaskType, AbortHandle>,
    client_id: String,
    to_lobby: LobbyChannel,
    to_game: Sender<Action>,
    to_outbound_task: Sender<OutboundTaskMessageType>,
    oubound_task_receiver: Option<Receiver<OutboundTaskMessageType>>,
}

// Implement drop to clean up all outstanding tasks.
impl Drop for PlayerClient {
    fn drop(&mut self) {
        log::debug!(
            "Client {} has been dropped. Stopping all tasks.",
            self.client_id
        );

        for abort_handle in self.tasks.values() {
            abort_handle.abort();
        }
    }
}

impl PlayerClient {
    /// Creates a new PlayerClient using connections from the lobby.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The ID for this client
    /// * `to_lobby` - A `LobbyChannel` back to the owning lobby
    /// * `to_game` - A channel to the game instance
    /// * `from_game` - A channel from the game instance to the player
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

    /// Sends a message directly to the player
    ///
    /// # Arguments
    ///
    /// * `message` - JSON message to send to the player
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

    /// Updates the PlayerClient with a new Websocket connection.
    ///
    /// # Arguments
    ///
    /// `ws` - The new WebSocket connection to use
    pub async fn update_websocket(&mut self, ws: WebSocket) {
        log::info!("Connecting client {}'s websockets.", self.client_id);

        let (to_client, mut from_client) = ws.split();

        // If we already have a ToClient task, which can happen with reconnects,
        // just update the websocket. Otherwise, create a new task.
        if self.tasks.contains_key(&TaskType::ToClient) {
            self.update_outgoing_ws_task(to_client).await;
        } else {
            self.create_outgoing_ws_task(to_client);
        }

        // Always create a new WS receiver task, as the old task will die when
        // the connection closes.
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let mut to_game = self.to_game.clone();
        let mut to_lobby = self.to_lobby.clone();
        let client_id = self.client_id.clone();
        let outgoing_to_client_future = Abortable::new(
            async move {
                while let Some(incoming_msg) = from_client.next().await {
                    if let Err(e) = incoming_msg {
                        log::error!("An error occurred while reading messages from the incoming connection for client {}. {}", client_id, e);
                        break;
                    }

                    let incoming_msg = incoming_msg.unwrap();
                    log::debug!(
                        "Received incoming message {:?} from client {}.",
                        incoming_msg,
                        client_id
                    );

                    let incoming_msg = match incoming_msg.to_str() {
                        Ok(msg) => msg,
                        Err(_) => {
                            break;
                        }
                    };
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
                            // TODO: Implement sending an error code to the client.
                            break;
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

                to_lobby
                    .send((LobbyCommand::PlayerDisconnect { client_id }, None))
                    .await;
            },
            abort_registration,
        );
        task::spawn(outgoing_to_client_future);

        if let Some(abort_handle) = self.tasks.insert(TaskType::FromClient, abort_handle) {
            abort_handle.abort();
        }
    }

    /// Spawns the `from_game` task. This task is a stable task, meaning it should
    /// last the duration of the PlayerClient's life.
    ///
    /// # Arguments
    ///
    /// `from_game` - The channel from the game to the player
    fn spawn_from_game_task(&mut self, mut from_game: Receiver<Message>) {
        log::debug!("Creating from_game task for client {}.", self.client_id);
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

        log::debug!(
            "Successfully spawned the from_game task for client {}.",
            self.client_id
        );
    }

    /// Updates the outgoing task with a new websocket connection
    ///
    /// # Arguments
    ///
    /// * `new_ws` - The new WebSocket to use
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

    /// Creates the outgoing task and adds it to the task dictionary.
    ///
    /// # Arguments
    ///
    /// `to_client` - The outgoing WebSocket connection to the client.
    fn create_outgoing_ws_task(&mut self, mut to_client: SplitSink<WebSocket, ws::Message>) {
        // Task to manage all incoming messages and send them to the client.
        log::info!(
            "Creating a new outgoing websocket task for client {}.",
            self.client_id
        );
        let client_id = self.client_id.clone();

        // Since an mpsc receiver isn't send or sync, we either need to lock it
        // take full ownership. To avoid overhead of a lock or an Arc, we take
        // ownership here. This is a stable task that should never crash, so
        // we don't need to ever remake it.
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
