//! Module containing functions to send notifications to a user. This can
//! include email, text, or other forms of communication. This module should be
//! used for sending non-game related notifications to a player, such as account
//! notifications.

pub mod account;
mod errors;

pub use errors::NotificationError;
use lazy_static::lazy_static;
use mailgun_rs::{EmailAddress, Mailgun, Message};
use std::env;
use tokio::task;

const SMTP_DOMAIN: &str = "mg.bennavetta.com";
const SMTP_USER: &str = "no-reply@mg.bennavetta.com";

lazy_static! {
    static ref SMTP_API_KEY: String =
        env::var("SMTP_API_KEY").unwrap_or("SMTP_API_KEY".to_string());
}

/// Builds and sends an email to the client and handles any SMTP related errors
///
/// # Arguments
///
/// * `email` - The email address of the recipient
/// * `subject` - The subject to send to the recipient
/// * `body` - The HTML body to send
async fn send_email(
    email: &String,
    subject: String,
    body: String,
) -> Result<(), NotificationError> {
    log::info!("Building email to send to user.");

    log::debug!("Subject: {}.\nBody: {}.", subject, body);
    let message = Message {
        to: vec![EmailAddress::address(email)],
        subject,
        html: body,
        ..Default::default()
    };

    let client = Mailgun {
        api_key: SMTP_API_KEY.to_string(),
        domain: SMTP_DOMAIN.to_string(),
        message,
    };

    let sender = EmailAddress::name_address("ThavalonWeb", SMTP_USER);

    if let Err(e) =
        task::spawn_blocking(move || client.send(mailgun_rs::MailgunRegion::US, &sender)).await
    {
        log::error!("ERROR: Failed to send the message to the recipient. {}.", e);
        return Err(NotificationError::MailServerError);
    }

    log::info!("Successfully sent an email to the recipient.");
    Ok(())
}
