//! Module handling all connections to clients, including REST and Web Sockets.
//! The module will handle requests and interface between the clients and game/lobby as needed.

//#region Modules and Use Statements
mod paths;
mod rest_handlers;
mod websockets;
use paths::*;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use warp::{self, Filter, Rejection, Reply};
use websockets::PlayerClients;
//#endregion

/// Main entry point. Serves all warp connections and paths.
/// This function does not return unless warp crashes (bad),
/// or the server is being shut down.
pub async fn serve_connections() {
    let create_new_lobby = warp::path(CREATE_NEW_LOBBY_PATH)
        .and(warp::post())
        .and_then(rest_handlers::try_create_new_lobby);
    // let connected_players: websockets::PlayerClients = Arc::new(Mutex::new(HashMap::new()));

    // let register = warp::path(WS_REGISTER_PATH);
    // let register_routes = register
    //     .and(warp::post())
    //     .and(warp::body::json())
    //     .and(with_clients(connected_players.clone()))
    //     .and_then(websockets::handler);
    let all_routes = warp::path("api").and(create_new_lobby);
    warp::serve(all_routes).run(([0, 0, 0, 0], 8001)).await;
}

/// Moves a new reference to the connected_players list into the WS register filter.
fn with_clients(
    connected_players: PlayerClients,
) -> impl Filter<Extract = (PlayerClients,), Error = Infallible> + Clone {
    warp::any().map(move || connected_players.clone())
}
