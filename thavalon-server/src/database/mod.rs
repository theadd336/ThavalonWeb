//! Contains all code related to the Thavalon database.

use lazy_static::lazy_static;
use mongodb::{options::ClientOptions, Client, Database};
use std::sync::RwLock;

pub mod accounts;

const MONGO_HOST: &str = "mongodb://admin:secret@database:27017";
const THAVALON_DB: &str = "thavalon_db";

lazy_static! {
    static ref CLIENT: RwLock<Option<Client>> = RwLock::new(None);
}

/// Gets a MongoDB client connection.
/// Since the database is on the same cluster (?), failing here should cause a crash.
async fn get_database() -> Database {
    get_client_internal().await.database(THAVALON_DB)
}

/// Initializes the MongoDB client. This must be called before the database is accessed.
pub async fn initialize_mongo_client() {
    let client_options = ClientOptions::parse(MONGO_HOST).await.unwrap();
    let client = Client::with_options(client_options).expect("Failed to create a MongoDB client.");
    CLIENT.write().unwrap().replace(client);
}

/// Acquires a read lock and returns a MongoDB Client.
/// Abstracts having to deal with locking and checking for Some().
async fn get_client_internal() -> Client {
    CLIENT
        .read()
        .expect("Could not acquire a read lock on the CLIENT instance.")
        .as_ref()
        .expect("ERROR: client called before initialization.")
        .clone()
}
