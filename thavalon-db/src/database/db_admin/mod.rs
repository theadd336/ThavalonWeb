use super::get_db_client;
use mongodb::{
    bson::{self, doc},
    options::FindOneOptions,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DB_ADMIN_COLLECTION: &str = "db_admin";

/// Canonical representation of a DB Admin.
/// These users have the power to call other database REST endpoints.
#[derive(Serialize, Deserialize)]
pub struct DBAdmin {
    pub username: String,
    pub hash: String,
}

/// Errors for database admin related functions
#[derive(Debug, Error, Eq, PartialEq)]
pub enum DBAdminError {
    #[error("Username: {0} is not a DB Admin.")]
    InvalidUsername(String),
    // #[error("Username: {0} is already an admin.")]
    // DuplicateUsername(String),
}

/// Loads a database admin user from the database by username.
///
/// # Arguments
///
/// * `username`: Admin user username
pub async fn load_db_admin(username: &String) -> Result<DBAdmin, DBAdminError> {
    // Get the collection and set up options and filters.
    log::info!("Loading db admin for username: {}", username);
    let collection = get_db_client().await.collection(DB_ADMIN_COLLECTION);
    let find_options = FindOneOptions::builder().show_record_id(false).build();
    let filter = doc! {"username": username.clone()};

    // Look up the document from the collection. An error signifies something went very wrong.
    // If no matches are found, None will be returned instead.
    let document = collection
        .find_one(filter, find_options)
        .await
        .expect("An unexpected error occurred while loading the user.");

    // No error, so see if we found a user or not.
    // Log and return appropriately
    if let Some(user) = document {
        log::info!("Found a valid DB admin for the given username.");
        let db_admin: DBAdmin = bson::from_document(user).expect("Could not decode database BSON.");
        Ok(db_admin)
    } else {
        log::warn!("Did not find username: {} in database.", username);
        Err(DBAdminError::InvalidUsername(username.to_string()))
    }
}

// /// Adds a database admin user to the database
// pub async fn add_db_admin(db_admin: &DBAdmin) -> Result<(), DBAdminError> {
//     log::info!("Attempting to add admin user: {}.", db_admin.username);
//     if does_user_exist(db_admin).await {
//         log::warn!("User {} already exists.", db_admin.username);
//         return Err(DBAdminError::DuplicateUsername(db_admin.username.clone()));
//     }

//     let collection = get_db_client().await.collection(DB_ADMIN_COLLECTION);
//     let user_doc = bson::to_document(db_admin).expect("Could not serialize user information.");
//     let result = collection.insert_one(user_doc, None).await;
//     match result {
//         Ok(_) => {
//             log::info!("Successfully added user.");
//             Ok(())
//         }
//         Err(e) => {
//             log::error!("Could not add unique user to db admin collection.");
//             log::error!("{:?}", e);
//             panic![];
//         }
//     }
// }

// /// Removes a database admin user from the database.
// pub async fn remove_db_admin(db_admin: &DBAdmin) -> Result<(), DBAdminError> {
//     log::info!(
//         "Attempting to remove user: {} from the database.",
//         db_admin.username
//     );
//     let db_admin_hashed = match load_db_admin(&db_admin.username).await {
//         Ok(user) => user,
//         Err(_) => {
//             log::warn!("User {} does not exist in the database.", db_admin.username);
//             return Err(DBAdminError::InvalidUsername(db_admin.username.clone()));
//         }
//     };

//     let collection = get_db_client().await.collection(DB_ADMIN_COLLECTION);
//     let document = bson::to_document(&db_admin_hashed)
//         .expect("Could not serialize user to database document.");

//     let result = collection.delete_one(document, None).await;
//     match result {
//         Ok(_) => {
//             log::info!("Successfully removed {} from database.", db_admin.username);
//             Ok(())
//         }
//         Err(e) => {
//             log::warn!("Failed to remove {} from database.", db_admin.username);
//             log::warn!("{:?}", e);
//             Err(DBAdminError::InvalidUsername(db_admin.username.clone()))
//         }
//     }
// }

// /// Checks if a given username exists in the database already
// async fn does_user_exist(db_admin: &DBAdmin) -> bool {
//     match load_db_admin(&db_admin.username).await {
//         Ok(_) => true,
//         Err(_) => false,
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     /// Tests add_db_admin for successful user adding and duplicate handling.
//     #[tokio::test]
//     async fn test_add_db_admin() {
//         cleanup_tests().await;
//         // Test 1, can we add a unique user?
//         let db_admin = DBAdmin {
//             username: "Paul".to_owned(),
//             hash: "weak password".to_owned(),
//         };

//         add_db_admin(&db_admin)
//             .await
//             .expect("Failed to add test user.");

//         // Test 2, do we fail to add a duplicate user?
//         let result = add_db_admin(&db_admin).await;
//         if let Err(err) = result {
//             assert_eq!(
//                 err,
//                 DBAdminError::DuplicateUsername(db_admin.username.clone())
//             );
//         } else {
//             panic!();
//         }
//         println!("End of add testing. Count: {:}.", collection_stats().await);
//     }

//     #[tokio::test]
//     /// Tests remove_db_admin for successful user removing and duplicate handling.
//     async fn test_remove_db_admin() {
//         cleanup_tests().await;
//         // Test 1, can we remove a user that exists.
//         let mut db_admin = DBAdmin {
//             username: "Paul".to_owned(),
//             hash: "weak password".to_owned(),
//         };

//         if !does_user_exist(&db_admin).await {
//             println!(
//                 "Adding a non-existent account for testing. Count: {:}.",
//                 collection_stats().await
//             );
//             add_db_admin(&db_admin).await.unwrap();
//             println!(
//                 "Force-added a non-existent account for testing. Count: {:}.",
//                 collection_stats().await
//             );
//         }

//         db_admin.hash = String::from("");
//         remove_db_admin(&db_admin)
//             .await
//             .expect("Failed to remove an existing user.");

//         println!(
//             "After first removal attempt. Count: {:}.",
//             collection_stats().await
//         );
//         // Test 2, do we handle removing an empty user correctly?
//         let result = remove_db_admin(&db_admin).await;
//         if let Err(err) = result {
//             assert_eq!(
//                 err,
//                 DBAdminError::InvalidUsername(db_admin.username.clone())
//             );
//         } else {
//             println!("{:}", collection_stats().await);
//             panic!("Failed to handle removing a non-existent user correctly.");
//         }
//     }

//     async fn cleanup_tests() {
//         let collection = get_db_client().await.collection(DB_ADMIN_COLLECTION);
//         collection.drop(None).await.unwrap();
//     }

//     async fn collection_stats() -> i64 {
//         let collection = get_db_client().await.collection(DB_ADMIN_COLLECTION);
//         let count = collection.estimated_document_count(None).await.unwrap();
//         count
//     }

//     #[tokio::test]
//     async fn test_does_user_exist() {
//         cleanup_tests().await;
//         let db_admin = DBAdmin {
//             username: "Paul".to_owned(),
//             hash: "weak password".to_owned(),
//         };

//         // Test 1, does an empty database contain our user?
//         assert!(!does_user_exist(&db_admin).await);

//         // Test 2, does the user that exists get identified as existing?
//         add_db_admin(&db_admin).await.unwrap();
//         assert!(does_user_exist(&db_admin).await);

//         // Test 3, does removing the user and retesting show the user has been removed.
//         remove_db_admin(&db_admin).await.unwrap();
//         assert!(!does_user_exist(&db_admin).await);
//     }
// }
