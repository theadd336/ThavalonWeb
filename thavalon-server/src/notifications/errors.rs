use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
/// Errors relating to user lookup.
pub enum NotificationError {
    #[error("Error connecting to the SMTP server.")]
    SMTPError,
}
