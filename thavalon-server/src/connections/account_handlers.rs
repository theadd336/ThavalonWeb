//! Rest handlers for account-based calls
use super::validation::{self, TokenStore, ValidationError};
use crate::database::{self, account_errors::AccountError, DatabaseAccount};
use serde::{Deserialize, Serialize};
use serde_json;
use warp::{
    http::{
        response::{Builder, Response},
        StatusCode,
    },
    hyper::Body,
    reject::{self, Reject},
    Rejection, Reply,
};

/// Canonical representation of a Thavalon user.
/// This struct is safe to send from the database, as it does not contain a password hash.
#[derive(Debug, Serialize, Deserialize)]
pub struct ThavalonUser {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub display_name: String,
    pub profile_picture: Option<Vec<u8>>,
    #[serde(default)]
    pub email_verified: bool,
}

impl From<DatabaseAccount> for ThavalonUser {
    fn from(db_account: DatabaseAccount) -> Self {
        ThavalonUser {
            email: db_account.email,
            password: String::from(""),
            display_name: db_account.display_name,
            profile_picture: db_account.profile_picture,
            email_verified: db_account.email_verified,
        }
    }
}

impl Into<DatabaseAccount> for ThavalonUser {
    fn into(self) -> DatabaseAccount {
        DatabaseAccount {
            email: self.email,
            hash: String::from(""),
            display_name: self.display_name,
            profile_picture: self.profile_picture,
            email_verified: self.email_verified,
        }
    }
}
#[derive(Debug)]
pub struct ValidationRejection;
impl Reject for ValidationRejection {}

#[derive(Debug)]
pub struct FatalHashingError;
impl Reject for FatalHashingError {}

#[derive(Debug)]
pub struct PasswordInsecureRejection;
impl Reject for PasswordInsecureRejection {}

#[derive(Debug)]
pub struct DuplicateAccountRejection;
impl Reject for DuplicateAccountRejection {}

#[derive(Debug)]
pub struct InvalidLoginRejection;
impl Reject for InvalidLoginRejection {}

#[derive(Debug)]
pub struct NoAccountRejection;
impl Reject for NoAccountRejection {}

#[derive(Debug)]
pub struct UnknownErrorRejection;
impl Reject for UnknownErrorRejection {}

/// Handles a request to add a user to the database.
///
/// # Arguments
///
/// `new_user` - The ThavalonUser to add
///
/// # Returns
///
/// * Success reply on success, a variety of rejections otherwise.
pub async fn handle_add_user(
    new_user: ThavalonUser,
    token_store: TokenStore,
) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to add new user: {}.", new_user.email);
    let hash = match validation::hash_password(&new_user.password).await {
        Ok(hash) => hash,
        Err(e) => {
            if e == ValidationError::HashError {
                log::error!("A hashing error occurred that prevented new user creation.");
                log::error!("{:?}", e);
                return Err(reject::custom(FatalHashingError));
            }
            log::info!("Password below minimum security requirements");
            return Err(reject::custom(PasswordInsecureRejection));
        }
    };

    let mut db_user: DatabaseAccount = new_user.into();
    db_user.hash = hash;
    if let Err(e) = database::create_new_user(&db_user).await {
        log::info!("{:?}", e);
        return Err(reject::custom(DuplicateAccountRejection));
    }
    log::info!("Successfully added {} to the database.", db_user.email);
    let (jwt, refresh_token) = validation::create_JWT(&db_user.email, token_store).await;
    let builder = Builder::new();
    Ok(builder
        .header(
            "Authentication",
            serde_json::to_string(&jwt).expect("Could not serialize JWT."),
        )
        .header(
            "Set-Cookie",
            format!(
                "refreshToken={}; Expires={}; Secure; HttpOnly; SameSite=Strict",
                refresh_token.token, refresh_token.expires_at
            ),
        )
        .status(StatusCode::CREATED)
        .body(""))
}

/// Authenticates a user by email and sends back the full user data to the game server.
///
/// # Arguments
///
/// * `user` - The thavalon user to authenticate. At this point, only email and password are populated.
///
/// # Returns
///
/// * Reply containing full user info on success. Password rejection otherwise.
pub async fn handle_user_login(user: ThavalonUser) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to log user {} in.", user.email);
    let hashed_user = match database::load_user_by_email(&user.email).await {
        Ok(user) => user,
        Err(e) => {
            log::info!("An error occurred while looking up the user.");
            log::info!("{:?}", e);
            return Err(reject::custom(InvalidLoginRejection));
        }
    };

    let is_valid = validation::validate_password(&user.password, &hashed_user.hash).await;
    if !is_valid {
        log::info!("Invalid password for {}.", user.email);
        return Err(reject::custom(InvalidLoginRejection));
    }

    let authed_user: ThavalonUser = hashed_user.into();
    log::info!("User {} logged in successfully.", authed_user.email);
    Ok(serde_json::to_string(&authed_user).expect("Could not serialize authenticated user."))
}

/// Deletes a user and all associated information from the database.
///
/// # Arguments
///
/// * `user` - The Thavalon user to remove
///
/// # Returns
///
/// * Empty reply on success, descriptive rejection otherwise.
pub async fn delete_user(user: ThavalonUser) -> Result<impl Reply, Rejection> {
    log::info!(
        "Attempting to delete user {} from the database.",
        user.email
    );
    let db_account: DatabaseAccount = user.into();
    if let Err(e) = database::remove_user(&db_account).await {
        log::info!("Error while removing the user from the database.");
        log::info!("{:?}", e);

        if e == AccountError::UserDoesNotExist {
            return Err(reject::custom(NoAccountRejection));
        } else {
            return Err(reject::custom(UnknownErrorRejection));
        }
    }
    Ok(warp::reply())
}

/// Updates a user with new information from the client.
/// This will blow out the old user info with the new user.
///
/// # Arguments
///
/// * `user` - The user to update
///
/// # Returns
///
/// Status 200 reply on success, Rejection on failure.
pub async fn update_user(user: ThavalonUser) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to update user {} in the database.", user.email);
    let password = user.password.clone();
    let mut user: DatabaseAccount = user.into();
    if password != String::from("") {
        user.hash = match validation::hash_password(&password).await {
            Ok(hash) => hash,
            Err(e) => {
                log::warn!("Failed to hash password. Update will be skipped.");
                log::warn!("{:?}", e);
                if e == ValidationError::HashError {
                    return Err(reject::custom(FatalHashingError));
                }
                return Err(reject::custom(PasswordInsecureRejection));
            }
        };
    }

    match database::update_user(&user).await {
        Ok(_) => Ok(warp::reply()),
        Err(e) => {
            log::warn!("Update to user {} failed.", user.email);
            log::warn!("{:?}", e);
            Err(reject::custom(NoAccountRejection))
        }
    }
}

/// Validates a given refresh token and returns a new JWT and refresh token if valid.
///
/// # Arguments
///
/// * `refresh_token` - Current refresh token to validate
/// * `token_store` - A store of active tokens
///
/// # Returns
///
/// * Reply with cookie and JWT on success. Rejection otherwise.
pub async fn validate_refresh_token(
    refresh_token: String,
    token_store: TokenStore,
) -> Result<impl Reply, Rejection> {
    log::info!("Handling request to refresh a token.");
    let (jwt, refresh_token) =
        match validation::validate_refresh_token(refresh_token, token_store).await {
            Ok(sec_tuple) => sec_tuple,
            Err(e) => {
                log::info!("Refresh token is not valid. Rejecting request.");
                return Err(reject::custom(ValidationRejection));
            }
        };

    log::info!("Refresh token validated. Sending new token to client.");
    let builder = Builder::new();
    let response = builder
        .header(
            "Authentication",
            serde_json::to_string(&jwt).expect("Could not serialize JWT."),
        )
        .header(
            "Set-Cookie",
            format!(
                "refreshToken={}; Expires={}; Secure; HttpOnly; SameSite=Strict",
                refresh_token.token, refresh_token.expires_at
            ),
        )
        .status(StatusCode::OK)
        .body("");
    Ok(response)
}
