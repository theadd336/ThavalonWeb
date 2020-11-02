//! Module handling all connections to clients, including REST and Web Sockets.
//! The module will handle requests and interface between the clients and game/lobby as needed.

//#region Modules and Use Statements
mod account_handlers;
mod errors;
mod game_handlers;
mod validation;
use crate::lobby::Lobby;
use game_handlers::GameCollection;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use validation::TokenManager;
use warp::{
    body,
    filters::cookie,
    reject::{self, Reject},
    Filter, Rejection,
};
//#endregion

const API_BASE_PATH: &str = "api";
const REFRESH_TOKEN_COOKIE: &str = "refreshToken";

#[derive(Debug, PartialEq)]
struct InvalidTokenRejection;
impl Reject for InvalidTokenRejection {}

/// Main entry point. Serves all warp connections and paths.
/// This function does not return unless warp crashes (bad),
/// or the server is being shut down.
pub async fn serve_connections() {
    let token_manager = TokenManager::new();

    let game_collection: GameCollection = Arc::new(Mutex::new(HashMap::new()));

    // TEST ROUTES
    let path_test = warp::path("hi").map(|| "Hello, World!");

    let restricted_path_test = warp::path("restricted_hi")
        .and(authorize_request(&token_manager))
        .map(|_| "Hello, restricted world!");

    // Account and Security
    let add_user_route = warp::path!("add" / "user")
        .and(body::json())
        .and(with_token_manager(token_manager.clone()))
        .and_then(account_handlers::handle_add_user);

    let login_route = warp::path!("auth" / "login")
        .and(body::json())
        .and(with_token_manager(token_manager.clone()))
        .and_then(account_handlers::handle_user_login);

    let logout_route = warp::path!("auth" / "logout")
        .and(cookie::cookie(REFRESH_TOKEN_COOKIE))
        .and(with_token_manager(token_manager.clone()))
        .and_then(account_handlers::handle_logout);

    let get_user_info_route = warp::path!("get" / "user")
        .and(authorize_request(&token_manager))
        .and_then(account_handlers::get_user_account_info);

    let refresh_jwt_route = warp::path!("auth" / "refresh")
        .and(cookie::cookie(REFRESH_TOKEN_COOKIE))
        .and(with_token_manager(token_manager.clone()))
        .and_then(account_handlers::renew_refresh_token);

    let delete_user_route = warp::path!("remove" / "user")
        .and(authorize_request(&token_manager))
        .and_then(account_handlers::delete_user);

    let update_user_route = warp::path!("update" / "user")
        .and(body::json())
        .and(authorize_request(&token_manager))
        .and_then(account_handlers::update_user);

    let verify_account_route = warp::path!("update" / "verifed_email")
        .and(body::json())
        .and_then(account_handlers::verify_account);

    // Game routes
    let create_game_route = warp::path!("add" / "game")
        .and(authorize_request(&token_manager))
        .and(with_game_collection(game_collection.clone()))
        .and_then(game_handlers::create_game);

    let join_game_route = warp::path!("join" / "game")
        .and(body::json())
        .and(authorize_request(&token_manager))
        .and(with_game_collection(game_collection.clone()))
        .and_then(game_handlers::join_game);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(authorize_request(&token_manager))
        .and(with_game_collection(game_collection.clone()))
        .and_then(game_handlers::connect_ws);

    // Putting everything together
    let get_routes = warp::get().and(path_test.or(restricted_path_test).or(get_user_info_route));
    let post_routes = warp::post().and(
        add_user_route
            .or(login_route)
            .or(refresh_jwt_route)
            .or(logout_route)
            .or(create_game_route)
            .or(join_game_route),
    );
    let delete_routes = warp::delete().and(delete_user_route);
    let put_routes = warp::put().and(update_user_route.or(verify_account_route));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Content-Type",
            "Authorization",
        ])
        .allow_methods(vec!["POST", "GET", "PUT", "DELETE"])
        .allow_credentials(true);

    let all_routes = warp::path(API_BASE_PATH)
        .and(get_routes.or(post_routes).or(delete_routes).or(put_routes))
        .recover(errors::recover_errors)
        .with(cors);
    warp::serve(all_routes).run(([0, 0, 0, 0], 8001)).await;
}

/// Authorizes a request for downstream endpoints.
/// This function returns a filter that passes along the user ID or a rejection.
fn authorize_request(
    token_manager: &TokenManager,
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    log::info!("Restricted API called. Validating auth header.");
    warp::header::<String>("Authorization")
        .and(with_token_manager(token_manager.clone()))
        .and_then(authorize_user)
}

/// Authorizes a user via JWT.
/// Returns either the user ID or a rejection if the user isn't authorized.
async fn authorize_user(header: String, token_manager: TokenManager) -> Result<String, Rejection> {
    log::info!("Authorizing user for restricted API by JWT.");
    let token_pieces: Vec<&str> = header.split(' ').collect();
    if token_pieces.len() < 2 {
        log::info!(
            "Invalid header format received. Received {}. Expected \"Basic <token>\".",
            header
        );
        return Err(reject::custom(InvalidTokenRejection));
    }
    let token = token_pieces[1];
    let player_id = match token_manager.validate_jwt(token).await {
        Ok(player_id) => player_id,
        Err(_) => {
            log::info!("JWT is not valid. Rejecting request.");
            return Err(reject::custom(InvalidTokenRejection));
        }
    };

    log::info!(
        "User {} is authorized for the requested service.",
        player_id
    );
    Ok(player_id)
}

/// Moves a token_store reference into downstream filters.
/// Used to add and validate refresh tokens and JWTs.
///
/// # Arguments
///
/// `token_store` - A token store with all active refresh tokens
fn with_token_manager(
    token_manager: TokenManager,
) -> impl Filter<Extract = (TokenManager,), Error = Infallible> + Clone {
    warp::any().map(move || token_manager.clone())
}

fn with_game_collection(
    game_collection: GameCollection,
) -> impl Filter<Extract = (GameCollection,), Error = Infallible> + Clone {
    warp::any().map(move || game_collection.clone())
}
