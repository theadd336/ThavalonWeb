//! Module handling all connections to clients, including REST and Web Sockets.
//! The module will handle requests and interface between the clients and game/lobby as needed.

//#region Modules and Use Statements
mod paths;
mod rest_handlers;
mod websockets;
use crate::game::PlayerId;
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
    // Set up connected_players for websockets.
    let connected_players: PlayerClients = Arc::new(Mutex::new(HashMap::new()));
    let create_new_lobby = warp::path(CREATE_NEW_LOBBY_PATH)
        .and(warp::post())
        .and_then(rest_handlers::try_create_new_lobby);

    let join_lobby = warp::path(JOIN_LOBBY_PATH)
        .and(warp::post())
        .and(with_clients(connected_players.clone()))
        .and(warp::path::param())
        .and(warp::path::param())
        .and_then(rest_handlers::try_join_lobby);

    let websocket_route = warp::path("ws").map(move || 1u32);
    // let register = warp::path(WS_REGISTER_PATH);
    // let register_routes = register
    //     .and(warp::post())
    //     .and(warp::body::json())
    //     .and(with_clients(connected_players.clone()))
    //     .and_then(websockets::handler);
    let rest_routes = warp::path("api").and(create_new_lobby.or(join_lobby));
    let all_routes = websocket_route.or(rest_routes);
    warp::serve(all_routes).run(([0, 0, 0, 0], 8001)).await;
}

/// Moves a new reference to the connected_players list into the WS register filter.
fn with_clients(
    connected_players: PlayerClients,
) -> impl Filter<Extract = (PlayerClients,), Error = Infallible> + Clone {
    warp::any().map(move || connected_players.clone())
}
