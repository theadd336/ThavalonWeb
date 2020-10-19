//! Module containing functions to send notifications to a user. This can
//! include email, text, or other forms of communication. This module should be
//! used for sending non-game related notifications to a player, such as account
//! notifications.

pub mod account;
mod errors;

pub use errors::NotificationError;
use lazy_static::lazy_static;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

const SMTP_SERVER: &str = "smtp.gmail.com";
const SMTP_USER: &str = "thavalonmanager@gmail.com";

lazy_static! {
    static ref SMTP_PASSWORD: String =
        env::var("SMTP_PASSWORD").unwrap_or("SMTP_SECRET".to_string());
}

async fn send_email(email: &String, subject: &String, body: &String) {
    log::info!("Building email to send to user.");

    let email = Message::builder()
        .from("NoReply <noreply@thavalonweb.com>".parse().unwrap())
        .to(email.parse().unwrap())
        .subject(subject)
        .body(body)
        .expect("ERROR: Could not parse email information.");

    let creds = Credentials::new(SMTP_USER.to_string(), SMTP_PASSWORD.to_string());

    log::info!(
        "Email and credentials built. Opening secure connection to {}.",
        SMTP_SERVER
    );

    let mailer = SmtpTransport::relay(SMTP_SERVER)
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => log::info!("Email sent successfully"),
        Err(e) => panic!(e),
    }
}
