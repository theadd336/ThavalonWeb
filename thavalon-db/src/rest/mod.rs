use std::convert::Infallible;
use warp::{body, http::StatusCode, Filter, Rejection, Reply};
mod account_handlers;

pub const API_BASE_PATH: &str = "api";

/// Main entry point for Warp. REST API that accepts and routes requests.
pub async fn accept_requests() {
    let path_test = warp::path("hi").map(|| "Hello, World!");

    let add_user_route = warp::path!("add" / "user")
        .and(validate_admin())
        .and(body::json())
        .and_then(account_handlers::handle_add_user);

    let auth_user_route = warp::path!("authenticate" / "user")
        .and(validate_admin())
        .and(body::json())
        .and_then(account_handlers::handle_user_login);

    let delete_user_route = warp::path!("remove" / "user")
        .and(validate_admin())
        .and(body::json())
        .and_then(account_handlers::delete_user);

    let update_user_route = warp::path!("update" / "user")
        .and(validate_admin())
        .and(body::json())
        .and_then(account_handlers::update_user);

    let get_routes = warp::get().and(path_test);
    let post_routes = warp::post().and(add_user_route.or(auth_user_route));
    let delete_routes = warp::delete().and(delete_user_route);
    let put_routes = warp::put().and(update_user_route);
    let all_routes = warp::path(API_BASE_PATH)
        .and(get_routes.or(post_routes).or(delete_routes).or(put_routes))
        .recover(reject_by_type);
    warp::serve(all_routes).run(([0, 0, 0, 0], 6543)).await;
}

/// Recover function to return any unauthorized requests.
async fn reject_by_type(rejection: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(account_handlers::ValidationRejection) = rejection.find() {
        return Ok(warp::reply::with_status(
            "Unathorized",
            StatusCode::UNAUTHORIZED,
        ));
    }
    Ok(warp::reply::with_status(
        "Unathorized",
        StatusCode::UNAUTHORIZED,
    ))
}

fn validate_admin() -> impl warp::Filter<Extract = (), Error = warp::Rejection> + Copy {
    log::info!("Restricted API called. Attempting to validate calling user");
    warp::header::<String>("authorization")
        .and_then(account_handlers::handle_admin_auth_request)
        .untuple_one()
}
