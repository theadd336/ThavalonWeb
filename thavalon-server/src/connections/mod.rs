//! Module handling all connections to clients, including REST and Web Sockets.
//! The module will handle requests and interface between the clients and game/lobby as needed.

//#region Modules and Use Statements
mod account_handlers;
mod validation;
use account_handlers::ThavalonUser;
use serde::Serialize;
use std::convert::Infallible;
use warp::{
    body,
    http::StatusCode,
    reject::{self, Reject},
    Filter, Rejection, Reply,
};
//#endregion

const API_BASE_PATH: &str = "api";

/// Main entry point. Serves all warp connections and paths.
/// This function does not return unless warp crashes (bad),
/// or the server is being shut down.
pub async fn serve_connections() {
    let path_test = warp::path("hi").map(|| "Hello, World!");

    let restricted_path_test = warp::path("restricted_hi")
        .and(authorize_request())
        .map(|_| "Hello, restricted world!");

    let add_user_route = warp::path!("add" / "user")
        .and(body::json())
        .and_then(account_handlers::handle_add_user);

    let login_route = warp::path!("authenticate" / "user")
        .and(body::json())
        .and_then(account_handlers::handle_user_login);

    let delete_user_route = warp::path!("remove" / "user")
        .and(body::json())
        .and_then(account_handlers::delete_user);

    let update_user_route = warp::path!("update" / "user")
        .and(body::json())
        .and_then(account_handlers::update_user);

    let get_routes = warp::get().and(path_test.or(restricted_path_test));
    let post_routes = warp::post().and(add_user_route.or(login_route));
    let delete_routes = warp::delete().and(delete_user_route);
    let put_routes = warp::put().and(update_user_route);
    let all_routes = warp::path(API_BASE_PATH)
        .and(get_routes.or(post_routes).or(delete_routes).or(put_routes))
        .recover(recover_errors);
    warp::serve(all_routes).run(([0, 0, 0, 0], 8001)).await;
}

fn authorize_request() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(authorize_user)
}

/// Recovers any custom rejections and returns a response to the client.
///
/// # Arguments
///
/// * `err` - The rejection caused by an upstream failure.
async fn recover_errors(err: Rejection) -> Result<impl Reply, Infallible> {
    let mut http_response_code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut error_code = 255;
    let mut error_message = "An unknown error occurred.".to_string();
    let server_error = ServerError {
        error_code,
        error_message,
    };

    let error_json = warp::reply::json(&server_error);
    Ok(warp::reply::with_status(error_json, http_response_code))
}

async fn authorize_user(header: String) -> Result<String, Rejection> {
    let token_pieces: Vec<&str> = header.split(' ').collect();
    if token_pieces.len() < 2 {
        log::info!(
            "Invalid header format received. Received {}. Expected \"Basic <token>\".",
            header
        );
        return Err(reject::custom(InvalidTokenRejection));
    }
    let token = token_pieces[1];
    let email = match validation::validate_jwt(token).await {
        Ok(email) => email,
        Err(_) => {
            log::info!("JWT is not valid. Rejecting request.");
            return Err(reject::custom(InvalidTokenRejection));
        }
    };

    log::info!("User {} is authorized for the requested service.", email);
    Ok(email)
}
#[derive(Debug, PartialEq)]
struct InvalidTokenRejection;
impl Reject for InvalidTokenRejection {}

#[derive(Serialize)]
struct ServerError {
    error_code: u16,
    error_message: String,
}
