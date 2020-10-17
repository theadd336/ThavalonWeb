pub mod account_errors;
use super::get_database;
use account_errors::AccountError;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::{FindOneOptions, UpdateOptions},
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

const USER_COLLECTION: &str = "thavalon_users";

/// Canonical representation of a database account.
/// This should never leave the database, as it contains a password hash!
pub struct DatabaseAccount {
    pub id: String,
    pub email: String,
    pub hash: String,
    pub display_name: String,
    pub profile_picture: Option<Vec<u8>>,
    pub email_verified: bool,
}

/// Internal representation of a user. Notably, this uses _id as an ObjectId
/// instead of a hex string. This also supports serialization and deserialization.
#[derive(Serialize, Deserialize)]
struct InternalDBAccount {
    _id: ObjectId,
    email: String,
    hash: String,
    display_name: String,
    profile_picture: Option<Vec<u8>>,
    email_verified: bool,
}

impl TryFrom<DatabaseAccount> for InternalDBAccount {
    type Error = String;

    fn try_from(public_account: DatabaseAccount) -> Result<Self, Self::Error> {
        let _id = match ObjectId::with_string(&public_account.id) {
            Ok(id) => id,
            Err(e) => {
                return Err(format!(
                    "Given ID {} is not valid hex. {}",
                    &public_account.id, e
                ));
            }
        };

        Ok(InternalDBAccount {
            _id,
            email: public_account.email,
            hash: public_account.hash,
            display_name: public_account.display_name,
            profile_picture: public_account.profile_picture,
            email_verified: public_account.email_verified,
        })
    }
}

impl From<InternalDBAccount> for DatabaseAccount {
    fn from(int_account: InternalDBAccount) -> Self {
        DatabaseAccount {
            id: int_account._id.to_hex(),
            email: int_account.email,
            hash: int_account.hash,
            display_name: int_account.display_name,
            profile_picture: int_account.profile_picture,
            email_verified: int_account.email_verified,
        }
    }
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
pub async fn create_new_user(
    email: &String,
    hash: &String,
    display_name: &String,
) -> Result<String, AccountError> {
    log::info!("Attempting to add thavalon user.");
    let collection = get_database().await.collection(USER_COLLECTION);
    let filter = doc! {
        "email": &email
    };

    // For some reason, Rust won't allow UpdateOptions to be constructed using
    // the standard {upsert: Some(true) ..UpdateOptions::default()}, so this
    // needs to be mut.
    let mut update_options = UpdateOptions::default();
    update_options.upsert = Some(true);

    // Use setOnInsert to ensure we don't blow out a user if they already exist.
    let update_doc = doc! {
        "$setOnInsert": {
            "email": email,
            "hash": hash,
            "display_name": display_name,
        },
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
            let id = bson::from_bson::<ObjectId>(result.upserted_id.unwrap()).unwrap();
            log::info!("Successfully added user {}.", id);
            Ok(id.to_hex())
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
/// * `user` - The user ID to remove
///
/// # Returns
///
/// * None on success, account error on failure.
pub async fn remove_user(user_id: &String) -> Result<(), AccountError> {
    log::info!("Attempting to remove user {} from the database.", user_id);

    let collection = get_database().await.collection(USER_COLLECTION);
    let filter = doc! {
        "_id": ObjectId::with_string(user_id).unwrap()
    };

    let result = collection.find_one_and_delete(filter, None).await;
    match result {
        Ok(_) => {
            log::info!("Successfully removed {} from database.", user_id);
            Ok(())
        }
        Err(e) => {
            log::warn!("Failed to remove {} from database. {:?}", user_id, e);
            Err(AccountError::UnknownError)
        }
    }
}

/// Loads an existing user to a DatabaseAccount by email
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
    log::info!("Loading user account by email.");
    let filter = doc! {"email": email};
    load_user_with_filter(filter).await
}

/// Loads an existing user to a DatabaseAccount by ID
///
/// # Arguments
///
/// * `id` - An ID to find an account for
///
/// # Returns
///
/// * `DatabaseAccount` on success, `AccountError` on failure.
pub async fn load_user_by_id(id: &String) -> Result<DatabaseAccount, AccountError> {
    log::info!("Loading user account for ID {}.", id);
    let filter = doc! {
        "_id": ObjectId::with_string(id).unwrap()
    };
    load_user_with_filter(filter).await
}

/// Internal function to load a user account given a filter.
///
/// # Arguments
///
/// * `filter` - The filter to search the DB for matches
///
/// # Returns
///
/// * A DatabaseAccount on success. Error on failure.
async fn load_user_with_filter(filter: Document) -> Result<DatabaseAccount, AccountError> {
    let collection = get_database().await.collection(USER_COLLECTION);
    let find_options = FindOneOptions::builder().show_record_id(true).build();

    // Look up the document from the collection. An error signifies something went very wrong.
    // If no matches are found, None will be returned instead.
    let document = collection
        .find_one(filter, find_options)
        .await
        .expect("An unexpected error occurred while loading the user.");

    // No error, so see if we found a user or not.
    // Log and return appropriately
    if let Some(user) = document {
        log::info!("Found a valid player for the given filter.");
        let user_account: InternalDBAccount =
            bson::from_document(user).expect("Could not decode database BSON.");
        Ok(user_account.into())
    } else {
        log::warn!("Did not find any filter matches in database.");
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
pub async fn update_user(user: DatabaseAccount) -> Result<(), AccountError> {
    log::info!("Attempting to update user {}.", user.id);
    let user = match InternalDBAccount::try_from(user) {
        Ok(user) => user,
        Err(e) => {
            log::warn!("Could not generate an ObjectId from the received ID. {}", e);
            return Err(AccountError::InvalidID);
        }
    };

    let collection = get_database().await.collection(USER_COLLECTION);
    let filter = doc! {"_id": &user._id};
    let user_doc = bson::to_document(&user).expect("Failed to serialize user to BSON.");
    match collection
        .find_one_and_replace(filter, user_doc, None)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Failed to find and replace user. {:?}", e);
            Err(AccountError::UnknownError)
        }
    }
}
