//! Module containing errors for database account problems

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
/// Errors relating to user lookup.
pub enum AccountError {
    #[error("The username does not exist.")]
    UserDoesNotExist,
    #[error("This email address is already in use.")]
    DuplicateAccount,
    #[error("The given ID is not valid hex for an internal ID.")]
    InvalidID,
    #[error("The given email verification code is not valid or has expired.")]
    InvalidEmailVerification,
    #[error("An unknown error occurred. See logs for more details.")]
    UnknownError,
}
