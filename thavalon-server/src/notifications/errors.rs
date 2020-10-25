//! Error enum for any notification errors.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
/// Errors relating to user lookup.
pub enum NotificationError {
    #[error("Error while sending a verification email.")]
    VerificationEmailError,
    #[error("Error connecting to the SMTP server.")]
    SMTPError,
}
