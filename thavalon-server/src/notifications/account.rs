//! Module containing account-related notification code.

use super::{send_email, NotificationError};
use crate::database::accounts;
use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use std::iter;

const EXPIRATION_DAYS: i64 = 3;
const EMAIL_BASE_PATH: &str = "localhost:8001/api/verify_email";

pub async fn send_email_verification(email: &String) -> Result<(), NotificationError> {
    log::info!("Sending a verification email for a new account.");
    let code: String = iter::repeat(())
        .map(|()| rand::thread_rng().sample(Alphanumeric))
        .take(32)
        .collect();
    let expires_at = Utc::now()
        .checked_add_signed(Duration::days(EXPIRATION_DAYS))
        .expect("Could not create expires time for the verification email link.")
        .timestamp();

    if let Err(e) = accounts::add_unverified_email(&code, email, expires_at).await {
        log::error!("Could not add an unverified email to the database. {}.", e);
        return Err(NotificationError::VerificationEmailError);
    }
    let mut user_link = String::from(EMAIL_BASE_PATH);
    user_link.push_str(&code);
    let subject = "Verify Your Thavalon Account".to_string();
    let body = format!("<p>Please click this <a href={}>link</a> to verify your account. This link expires in {} days.</p>", user_link, EXPIRATION_DAYS);
    send_email(email, &subject, &body).await;
    Ok(())
}
