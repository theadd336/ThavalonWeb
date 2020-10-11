pub mod account_errors;
use super::get_database;
use account_errors::AccountError;
use mongodb::{
    bson::{self, doc},
    options::{FindOneOptions, UpdateOptions},
};
use serde::{Deserialize, Serialize};
const USER_COLLECTION: &str = "thavalon_users";
/// Canonical representation of a database account.
/// This should never leave the database, as it contains a password hash!
#[derive(Serialize, Deserialize)]
pub struct DatabaseAccount {
    pub email: String,
    pub hash: String,
    pub display_name: String,
    pub profile_picture: Option<Vec<u8>>,
    pub email_verified: bool,
}

/// Creates a new user and adds to the database
///
/// # Arguments
///
/// * `user` - A DatabaseAccount holding information for the new user
///
/// # Returns
///
/// * Null on success, AccountCreationError on failure
pub async fn create_new_user(user: &DatabaseAccount) -> Result<(), AccountError> {
    log::info!("Attempting to add thavalon user: {}.", user.email);
    let collection = get_database().await.collection(USER_COLLECTION);
    let filter = doc! {
        "email": user.email.clone()
    };

    // For some reason, Rust won't allow UpdateOptions to be constructed using
    // the standard {upsert: Some(true) ..UpdateOptions::default()}, so this
    // needs to be mut.
    let mut update_options = UpdateOptions::default();
    update_options.upsert = Some(true);

    let user_doc = bson::to_document(user).expect("Could not serialize user information.");
    // Use setOnInsert to ensure we don't blow out a user if they already exist.
    let update_doc = doc! {
        "$setOnInsert": user_doc
    };
    let result = collection
        .update_one(filter, update_doc, update_options)
        .await;
    match result {
        Ok(result) => {
            // If the filter matched, the user already exists, so return an error.
            if result.matched_count > 0 {
                log::info!("The username already exists. Email addresses must be unique.");
                return Err(AccountError::DuplicateAccount);
            }
            log::info!("Successfully added user.");
            Ok(())
        }
        Err(e) => {
            log::error!(
                "Could not add unique user to thavalon users collection. {:?}.",
                e
            );
            Err(AccountError::UnknownError)
        }
    }
}

/// Removes a thavalon user from the database, deleting all information for the user.
///
/// # Arguments
///
/// * `user` - The user to remove
///
/// # Returns
///
/// * None on success, account error on failure.
pub async fn remove_user(user: &DatabaseAccount) -> Result<(), AccountError> {
    log::info!(
        "Attempting to remove user: {} from the database.",
        user.email
    );
    let user_hashed = match load_user_by_email(&user.email).await {
        Ok(user) => user,
        Err(_) => {
            log::warn!("User {} does not exist in the database.", user.email);
            return Err(AccountError::UserDoesNotExist);
        }
    };

    let collection = get_database().await.collection(USER_COLLECTION);
    let document =
        bson::to_document(&user_hashed).expect("Could not serialize user to database document.");

    let result = collection.delete_one(document, None).await;
    match result {
        Ok(_) => {
            log::info!("Successfully removed {} from database.", user.email);
            Ok(())
        }
        Err(e) => {
            log::warn!("Failed to remove {} from database. {:?}", user.email, e);
            Err(AccountError::UnknownError)
        }
    }
}

/// Loads an existing user to a DatabaseAccount
///
/// # Arguments
///
/// * `email` - An email to find an account for
///
/// # Returns
///
/// * `DatabaseAccount` on success, `AccountError` on failure.
pub async fn load_user_by_email(email: &String) -> Result<DatabaseAccount, AccountError> {
    // Get the collection and set up options and filters.
    log::info!("Loading user account for email: {}", email);
    let collection = get_database().await.collection(USER_COLLECTION);
    let find_options = FindOneOptions::builder().show_record_id(false).build();
    let filter = doc! {"email": email.clone()};

    // Look up the document from the collection. An error signifies something went very wrong.
    // If no matches are found, None will be returned instead.
    let document = collection
        .find_one(filter, find_options)
        .await
        .expect("An unexpected error occurred while loading the user.");

    // No error, so see if we found a user or not.
    // Log and return appropriately
    if let Some(user) = document {
        log::info!("Found a valid player for the given username.");
        let user_account: DatabaseAccount =
            bson::from_document(user).expect("Could not decode database BSON.");
        Ok(user_account)
    } else {
        log::warn!("Did not find username: {} in database.", email);
        Err(AccountError::UserDoesNotExist)
    }
}

/// Updates a user to match the given DatabaseAccount.
/// Will blow out the old user and match to the new one.
///
/// # Arguments
///
/// * `user` - The user to update.
///
/// # Returns
///
/// * None on success. Error on failure.
pub async fn update_user(user: &DatabaseAccount) -> Result<(), AccountError> {
    log::info!("Attempting to update user {}.", user.email);

    let collection = get_database().await.collection(USER_COLLECTION);
    let filter = doc! {"email": user.email.clone()};
    let user_doc = bson::to_document(user).expect("Failed to serialize user to BSON.");
    match collection
        .find_one_and_replace(filter, user_doc, None)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Failed to find and replace user {}. {:?}", user.email, e);
            Err(AccountError::UnknownError)
        }
    }
}
