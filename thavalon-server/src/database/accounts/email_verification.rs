//! Module containing email verification related database functions

use super::account_errors::AccountError;
use super::get_database;
use mongodb::{
    bson::{self, doc, Document},
    options::UpdateOptions,
};
use serde::Deserialize;

const EMAIL_VERIFICATION_COLLECTION: &str = "thavalon_unverified_emails";

#[derive(Deserialize)]
pub struct UnverifiedEmailInfo {
    pub verification_code: String,
    pub email: String,
    pub expires_at: i64,
}

/// Adds an unverified email to the collection.
/// This will update the verification code for an email, if it's present already.
///
/// # Arguments
///
/// * `code` - The verification code to send to a user to verify
/// * `email` - The email address to verify
/// * `expires_at` - The timestamp at which the verification code will expire.
///
/// # Returns
///
/// * Empty type on success, AccountError on failure
pub async fn add_unverified_email(
    code: &String,
    email: &String,
    expires_at: i64,
) -> Result<(), AccountError> {
    log::info!(
        "Adding unverified email info with code: {} to the database.",
        code
    );

    let collection = get_database()
        .await
        .collection(EMAIL_VERIFICATION_COLLECTION);

    let filter = doc! {
        "email": email
    };

    // For some reason, Rust won't allow UpdateOptions to be constructed using
    // the standard {upsert: Some(true) ..UpdateOptions::default()}, so this
    // needs to be mut.
    let mut update_options = UpdateOptions::default();
    update_options.upsert = Some(true);

    // Unlike a user, this will blow out an old verification code if one exists.
    // This is because verification codes can be changed for an email on a resend.
    let update_doc = doc! {
        "$set": {
            "verification_code": code,
            "email": email,
            "expires_at": expires_at
         },
    };
    let result = collection
        .update_one(filter, update_doc, update_options)
        .await;

    match result {
        Ok(result) => {
            // If the filter matched, the user already exists, so return an error.
            if result.matched_count > 0 {
                log::info!("The email already exists. Updated verification code successfully.");
                return Ok(());
            }
            log::info!("Successfully added verification code.");
            Ok(())
        }
        Err(e) => {
            log::error!(
                "An unknown error occured while adding unverified email info. {:?}.",
                e
            );
            Err(AccountError::UnknownError)
        }
    }
}

/// Pops unverified email information by verification code from the DB.
///
/// # Arguments
///
/// * `verification_code` - The verification code to use for lookup
///
/// # Returns
///
/// * `UnverifiedEmailInfo` on success, `AccountError` on failure.
pub async fn pop_info_by_code(
    verification_code: &String,
) -> Result<UnverifiedEmailInfo, AccountError> {
    log::info!(
        "Popping unverified email info using code {}.",
        verification_code
    );

    let filter = doc! {
        "verification_code": verification_code
    };

    pop_info_with_filter(filter).await
}

/// Pops unverified email information by email from the DB.
///
/// # Arguments
///
/// * `email` - The email to use for lookup
///
/// # Returns
///
/// * `UnverifiedEmailInfo` on success, `AccountError` on failure.
pub async fn pop_info_by_email(email: &String) -> Result<UnverifiedEmailInfo, AccountError> {
    log::info!("Popping unverified email info using email.");

    let filter = doc! {
        "email": email
    };

    pop_info_with_filter(filter).await
}

/// Internal function to pop email info given a specified filter.
///
/// # Arguments
///
/// * `filter` - The BSON document to use as a filter
///
/// # Returns
///
/// * `UnverifiedEmailInfo` on success, `AccountError` on failure
async fn pop_info_with_filter(filter: Document) -> Result<UnverifiedEmailInfo, AccountError> {
    let collection = get_database()
        .await
        .collection(EMAIL_VERIFICATION_COLLECTION);

    let db_document = match collection.find_one_and_delete(filter, None).await {
        Ok(document) => document,
        Err(e) => {
            log::error!("An error occurred while retrieving information. {:?}", e);
            return Err(AccountError::UnknownError);
        }
    };

    if db_document.is_none() {
        log::info!("No matching unverified email account was found.");
        return Err(AccountError::InvalidEmailVerification);
    }

    let email_info: UnverifiedEmailInfo = bson::from_document(db_document.unwrap())
        .expect("Could not deserialize unverified email info.");
    log::info!("Found a valid unverified email account.");
    Ok(email_info)
}
