use crate::game::PlayerId;
use futures::channel::mpsc::UnboundedSender;
use serde;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::filters::ws::Message;
use warp::{Error, Rejection, Reply};

type ConnectionResult<T> = Result<T, Rejection>;
pub type PlayerClients = Arc<Mutex<HashMap<String, PlayerClient>>>;

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
