use crate::connections::account_handlers::{
    DuplicateAccountRejection, InvalidLoginRejection, PasswordInsecureRejection,
    ValidationRejection,
};
use serde::Serialize;
use std::convert::Infallible;
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerError {
    error_code: i32,
    error_message: String,
}

enum ErrorCode {
    Unauthorized = 1,
    DuplicateAccount,
    PasswordInsecure,
    InvalidLogin,
    Unknown = 255,
}

/// Recovers any custom rejections and returns a response to the client.
///
/// # Arguments
///
/// * `err` - The rejection caused by an upstream failure.
pub async fn recover_errors(err: Rejection) -> Result<impl Reply, Infallible> {
    let mut http_response_code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut error_code = ErrorCode::Unknown;
    let mut error_message = "An unknown error occurred.".to_string();

    if let Some(ValidationRejection) = err.find() {
        http_response_code = StatusCode::UNAUTHORIZED;
        error_message = "Bad validation or unauthorized".to_string();
        error_code = ErrorCode::Unauthorized;
    } else if let Some(DuplicateAccountRejection) = err.find() {
        http_response_code = StatusCode::CONFLICT;
        error_message = "An account is already registered with this email address.".to_string();
        error_code = ErrorCode::DuplicateAccount;
    } else if let Some(PasswordInsecureRejection) = err.find() {
        http_response_code = StatusCode::NOT_ACCEPTABLE;
        error_message = "This password does not meet minimum security requirements.".to_string();
        error_code = ErrorCode::PasswordInsecure;
    } else if let Some(InvalidLoginRejection) = err.find() {
        http_response_code = StatusCode::UNAUTHORIZED;
        error_message = "Invalid email or password.".to_string();
        error_code = ErrorCode::InvalidLogin;
    }

    let server_error = ServerError {
        error_code: error_code as i32,
        error_message,
    };

    let error_json = warp::reply::json(&server_error);
    Ok(warp::reply::with_status(error_json, http_response_code))
}
