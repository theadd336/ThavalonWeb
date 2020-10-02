use mongodb::{options::ClientOptions, Client, Database};

mod accounts;
mod db_admin;

pub use self::accounts::*;
pub use self::db_admin::*;

const MONGO_HOST: &str = "mongodb://admin:secret@database:27017";
const THAVALON_DB: &str = "thavalon_db";

/// Gets a MongoDB client connection.
/// Since the database is on the same cluster (?), failing here should cause a crash.
async fn get_db_client() -> Database {
    let client_options = ClientOptions::parse(MONGO_HOST).await.unwrap();
    let client = Client::with_options(client_options).expect("Failed to create a MongoDB client.");

    client.database(THAVALON_DB)
}
