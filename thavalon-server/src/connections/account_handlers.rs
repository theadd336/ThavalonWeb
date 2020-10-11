//! Rest handlers for account-based calls
use super::validation::{self, JWTResponse, RefreshTokenInfo, TokenManager, ValidationError};
use crate::database::{self, account_errors::AccountError, DatabaseAccount};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use warp::{
    http::{response::Builder, StatusCode},
    reject::{self, Reject},
    reply, Rejection, Reply,
};

/// Canonical representation of a Thavalon user.
/// This struct is safe to send from the database, as it does not contain a password hash.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    mut token_manager: TokenManager,
) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to add new user: {}.", new_user.email);
    let hash = match validation::hash_password(&new_user.password).await {
        Ok(hash) => hash,
        Err(e) => {
            if e == ValidationError::HashError {
                log::error!(
                    "A hashing error occurred that prevented new user creation. {}",
                    e
                );
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
    let (jwt, refresh_token) = token_manager.create_jwt(&db_user.email).await;
    let response = create_validated_response(jwt, refresh_token, StatusCode::CREATED).await;
    Ok(response)
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
pub async fn handle_user_login(
    user: ThavalonUser,
    mut token_manager: TokenManager,
) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to log user {} in.", user.email);
    let hashed_user = match database::load_user_by_email(&user.email).await {
        Ok(user) => user,
        Err(e) => {
            log::info!("An error occurred while looking up the user. {}", e);
            return Err(reject::custom(InvalidLoginRejection));
        }
    };

    let is_valid = validation::validate_password(&user.password, &hashed_user.hash).await;
    if !is_valid {
        log::info!("Invalid password for {}.", user.email);
        return Err(reject::custom(InvalidLoginRejection));
    }

    log::info!("User {} logged in successfully.", hashed_user.email);
    let (jwt, refresh_token) = token_manager.create_jwt(&hashed_user.email).await;
    let response = create_validated_response(jwt, refresh_token, StatusCode::OK).await;
    Ok(response)
}

/// Loads user account information from the database. The user must already be
/// authenticated with an auth token before calling.
///
/// # Arguments
///
/// * `email` - The email to load information for
///
/// # Returns
///
/// * JSON serialized ThavalonUser on success. Rejection on failure.
pub async fn get_user_account_info(email: String) -> Result<impl Reply, Rejection> {
    log::info!("Loading user account info for the specified account.");
    let user = match database::load_user_by_email(&email).await {
        Ok(user) => user,
        Err(e) => {
            log::warn!(
                "Failed to find user info for an authenticated account. {}",
                e
            );
            return Err(reject::custom(UnknownErrorRejection));
        }
    };

    let user: ThavalonUser = user.into();
    log::info!("Successfully loaded user account information.");
    Ok(reply::json(&user))
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
        log::info!("Error while removing the user from the database. {}", e);

        if e == AccountError::UnknownError {
            return Err(reject::custom(UnknownErrorRejection));
        }
    }
    Ok(warp::reply::with_status(
        warp::reply(),
        StatusCode::NO_CONTENT,
    ))
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
    if &password != "" {
        user.hash = match validation::hash_password(&password).await {
            Ok(hash) => hash,
            Err(e) => {
                log::warn!("Failed to hash password. Update will be skipped. {}", e);
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
            log::warn!("Update to user {} failed. {}", user.email, e);
            Err(reject::custom(NoAccountRejection))
        }
    }
}

/// Validates and renews a given refresh token and returns a new JWT and refresh token if valid.
///
/// # Arguments
///
/// * `refresh_token` - Current refresh token to validate
/// * `token_manager` - A manager of active tokens
///
/// # Returns
///
/// * Reply with cookie and JWT on success. Rejection otherwise.
pub async fn renew_refresh_token(
    refresh_token: String,
    mut token_manager: TokenManager,
) -> Result<impl Reply, Rejection> {
    log::info!("Handling request to refresh a token.");
    let (jwt, refresh_token) = match token_manager.renew_refresh_token(refresh_token).await {
        Ok(sec_tuple) => sec_tuple,
        Err(e) => {
            log::info!("Refresh token is not valid. Rejecting request. {}", e);
            return Err(reject::custom(ValidationRejection));
        }
    };

    log::info!("Refresh token validated. Sending new token to client.");
    let response = create_validated_response(jwt, refresh_token, StatusCode::OK).await;
    Ok(response)
}

/// Creates a warp response with an authorization header for a JWT, a refresh
/// token as a cookie, and a caller-specified status and body.
///
/// # Arguments
///
/// * `jwt` - The javascript web token to send in the authorization header
/// * `refresh_token` - A valid refresh token sent as an HttpOnly cookie
/// * `status_code` - An HTTP status code to send.
/// * `body` - The body of the HTTP response, as a string.
///
/// # Returns
///
/// * Response implementing `warp::Reply`
async fn create_validated_response(
    jwt: JWTResponse,
    refresh_token: RefreshTokenInfo,
    status_code: StatusCode,
) -> impl Reply {
    let exp_datetime = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(refresh_token.expires_at, 0),
        Utc,
    );

    Builder::new()
        .header(
            "Set-Cookie",
            format!(
                "refreshToken={}; Expires={}; path=/; HttpOnly; Secure; SameSite=None",
                refresh_token.token,
                exp_datetime.to_rfc2822()
            ),
        )
        .status(status_code)
        .body(serde_json::to_string(&jwt).expect("Could not serialize JWT."))
}
