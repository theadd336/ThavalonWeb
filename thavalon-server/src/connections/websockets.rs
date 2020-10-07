use crate::game::PlayerId;
use futures::channel::mpsc::UnboundedSender;
use serde;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::filters::ws::Message;
use warp::{Error, Rejection, Reply};
use warp::ws::{Ws, WebSocket};
use std::sync::mpsc;

type ConnectionResult<T> = Result<T, Rejection>;
pub type PlayerClients = Arc<Mutex<HashMap<String, (String, PlayerId)>>>;

pub struct PlayerClient {
    pub player_id: PlayerId,
    pub sender: Option<UnboundedSender<Result<Message, Error>>>,
}

pub async fn handler(
    body: RegisterRequest,
    clients: PlayerClients,
) -> ConnectionResult<impl Reply> {
    return Ok(serde_json::to_string(&Response {
        url: format!("ws://127.0.00.1:8000/ws/{}", 5),
    })
    .unwrap());
}

#[derive(serde::Serialize)]
pub struct Response {
    pub url: String,
}

#[derive(serde::Deserialize)]
pub struct RegisterRequest {
    pub user_id: usize,
}

pub async fn ws_handler(ws: Ws, id: String, clients: PlayerClients) -> Result<impl Reply> {
    let client = clients.lock().unwrap().get(&id).cloned();
    match client {
      Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c))),
      None => Err(warp::reject::not_found()),
    }
  }

  pub async fn client_connection(ws: WebSocket, id: String, clients: PlayerClients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
  
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
      if let Err(e) = result {
        eprintln!("error sending websocket msg: {}", e);
      }
    }));