//! Module containing top-level server error handling. Errors here are
//! formatted and sent back to a client.

use crate::connections::account_handlers::{
    DuplicateAccountRejection, EmailVerificationRejection, InvalidLoginRejection,
    PasswordInsecureRejection, ValidationRejection,
};
use serde::Serialize;
use std::convert::Infallible;
use warp::{http::StatusCode, reject::InvalidHeader, Rejection, Reply};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerError {
    error_code: i32,
    error_message: String,
}

#[derive(PartialEq)]
enum ErrorCode {
    Unauthorized = 1,
    DuplicateAccount,
    PasswordInsecure,
    InvalidLogin,
    MissingHeader,
    InvalidAccountVerification,
    Unknown = 255,
}

/// Recovers any custom rejections and returns a response to the client.
///
/// # Arguments
///
/// * `err` - The rejection caused by an upstream failure.
pub async fn recover_errors(err: Rejection) -> Result<impl Reply, Infallible> {
    log::info!("Handling rejections: {:?}", err);
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
    } else if let Some(super::InvalidTokenRejection) = err.find() {
        http_response_code = StatusCode::UNAUTHORIZED;
        error_message = "Missing or invalid JSON web token.".to_string();
        error_code = ErrorCode::Unauthorized;
    } else if let Some(e) = err.find::<InvalidHeader>() {
        // Since MissingHeader has fields, need to use the generic fn notation here.
        http_response_code = StatusCode::UNAUTHORIZED;
        error_message = format!("Missing or invalid header: {}.", e.name());
        error_code = ErrorCode::MissingHeader;
    } else if let Some(EmailVerificationRejection) = err.find() {
        http_response_code = StatusCode::FORBIDDEN;
        error_message = "Verification code expired or the account has been deleted.".to_string();
        error_code = ErrorCode::InvalidAccountVerification;
    }

    if error_code == ErrorCode::Unknown {
        log::warn!(
            "WARNING: an unhandled server exception occurred. 
            Please see logs for more info. Rejection: {:?}.",
            err
        );
    }

    let server_error = ServerError {
        error_code: error_code as i32,
        error_message,
    };

    let error_json = warp::reply::json(&server_error);
    Ok(warp::reply::with_status(error_json, http_response_code))
}
