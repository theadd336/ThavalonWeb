//! Rest handlers for account-based calls
use super::validation::{self, JWTResponse, RefreshTokenInfo, TokenManager, ValidationError};
use super::REFRESH_TOKEN_COOKIE;
use crate::database::accounts::{self, AccountError, DatabaseAccount};
use crate::notifications::account;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::Infallible;
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
    pub player_id: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub profile_picture: Option<Vec<u8>>,
    pub email_verified: bool,
}

impl From<DatabaseAccount> for ThavalonUser {
    fn from(db_account: DatabaseAccount) -> Self {
        ThavalonUser {
            player_id: db_account.id,
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
            id: self.player_id,
            email: self.email,
            hash: String::from(""),
            display_name: self.display_name,
            profile_picture: self.profile_picture,
            email_verified: self.email_verified,
        }
    }
}

/// Represents information required to log a user in.
#[derive(Deserialize)]
pub struct LoginRequestInfo {
    email: String,
    password: String,
}

/// Represents information required to create a new user account.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUserInfo {
    email: String,
    password: String,
    display_name: String,
}

/// Represents information required to verify a user account.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyAccountInfo {
    verification_code: String,
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

#[derive(Debug)]
pub struct EmailVerificationRejection;
impl Reject for EmailVerificationRejection {}

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
    new_user: NewUserInfo,
    mut token_manager: TokenManager,
) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to add new user.");
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

    let player_id =
        match accounts::create_new_user(&new_user.email, &hash, &new_user.display_name).await {
            Ok(id) => id,
            Err(e) => {
                log::info!("{:?}", e);
                return Err(reject::custom(DuplicateAccountRejection));
            }
        };
    log::info!("Successfully added user to the database.");
    let (jwt, refresh_token) = token_manager.create_jwt(&player_id).await;
    let response = create_validated_response(jwt, refresh_token, StatusCode::CREATED).await;
    if let Err(e) = account::send_email_verification(&new_user.email).await {
        log::error!(
            "ERROR: failed to send account verification email to {}. {}.",
            player_id,
            e
        );
        // TODO: Support resending email verification. We don't want to return
        // an error, since the account was created succesfully, but we need to
        // resend this email somehow.
    }
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
    login_info: LoginRequestInfo,
    mut token_manager: TokenManager,
) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to log a user in.");
    let hashed_user = match accounts::load_user_by_email(&login_info.email).await {
        Ok(user) => user,
        Err(e) => {
            log::info!("An error occurred while looking up the user. {}", e);
            return Err(reject::custom(InvalidLoginRejection));
        }
    };

    let is_valid = validation::validate_password(&login_info.password, &hashed_user.hash).await;
    if !is_valid {
        log::info!("Invalid password for {}.", hashed_user.id);
        return Err(reject::custom(InvalidLoginRejection));
    }

    log::info!("User {} logged in successfully.", hashed_user.id);
    let (jwt, refresh_token) = token_manager.create_jwt(&hashed_user.id).await;
    let response = create_validated_response(jwt, refresh_token, StatusCode::OK).await;
    Ok(response)
}

/// Handles a user logging out. This revokes the user's refresh token.
///
/// # Arguments
///
/// * `refresh_token` - The user's refresh token to revoke.
/// * `token_manager` - The token store with refresh tokens.
pub async fn handle_logout(
    refresh_token: String,
    mut token_manager: TokenManager,
) -> Result<impl Reply, Infallible> {
    log::info!("Logging out user with refresh token {}.", refresh_token);
    token_manager.revoke_refresh_token(&refresh_token).await;
    Ok(StatusCode::RESET_CONTENT)
}

/// Loads user account information from the database. The user must already be
/// authenticated with an auth token before calling.
///
/// # Arguments
///
/// * `player_id` - The player ID to load information for
///
/// # Returns
///
/// * JSON serialized ThavalonUser on success. Rejection on failure.
pub async fn get_user_account_info(player_id: String) -> Result<impl Reply, Rejection> {
    log::info!("Loading user account info for the specified account.");
    let user = match accounts::load_user_by_id(&player_id).await {
        Ok(user) => user,
        Err(e) => {
            log::error!(
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
/// * `player_id` - The ID of the user to remove
///
/// # Returns
///
/// * Empty reply on success, descriptive rejection otherwise.
pub async fn delete_user(player_id: String) -> Result<impl Reply, Rejection> {
    log::info!("Attempting to delete user {} from the database.", player_id);
    let user = match accounts::remove_user(&player_id).await {
        Ok(user) => user,
        Err(e) => {
            log::info!("Error while removing the user from the database. {}", e);

            if e == AccountError::UnknownError {
                return Err(reject::custom(UnknownErrorRejection));
            }

            // Return success even if the account doesn't exist. Account
            // deletion should always succeed from the client perspective.
            return Ok(StatusCode::NO_CONTENT);
        }
    };

    // Use _ here to avoid compiler warning about unusued result.
    let _ = accounts::pop_info_by_email(&user.email).await;
    Ok(StatusCode::NO_CONTENT)
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
pub async fn update_user(user: ThavalonUser, _: String) -> Result<impl Reply, Rejection> {
    log::info!(
        "Attempting to update user {} in the database.",
        user.player_id
    );
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

    match accounts::update_user(user).await {
        Ok(_) => Ok(warp::reply()),
        Err(e) => {
            log::warn!("Failed to update user. {}", e);
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

/// Verifies an account's email using a verification code.
///
/// # Arguments
///
/// * `verification_request` - The request to verify a user's account
///
/// # Returns
/// * `200 OK` on success, `EmailVerificationRejection` on failure
pub async fn verify_account(
    verification_request: VerifyAccountInfo,
) -> Result<impl Reply, Rejection> {
    let verification_code = &verification_request.verification_code;
    log::info!("Verifying account using code {}.", verification_code);

    // First, load the verification info from the database.
    let info = match accounts::pop_info_by_code(verification_code).await {
        Ok(info) => info,
        Err(e) => {
            if e == AccountError::UnknownError {
                log::error!(
                    "An unknown error occurred while loading verification info. {}",
                    e
                );
                return Err(reject::custom(UnknownErrorRejection));
            }
            log::warn!("An error occurred while loading verification info. {}", e);
            return Err(reject::custom(EmailVerificationRejection));
        }
    };

    // Verify that it's not expired.
    let now = chrono::Utc::now().timestamp();
    if info.expires_at > now {
        log::info!(
            "The validation code {} has expired. Current time {}. Expiration time: {}.",
            verification_code,
            now,
            info.expires_at
        );

        return Err(reject::custom(EmailVerificationRejection));
    }

    log::info!(
        "Verification code {} is valid. Updating the user account.",
        verification_code
    );

    // Update the user account. This technically is subject to race conditions
    // since we have to load the account first. However, it's highly unlikely
    // a user could do this fast enough to actually cause an issue.
    // Notably, deleted accounts won't be recreated by this process.
    let mut user = match accounts::load_user_by_email(&info.email).await {
        Ok(user) => user,
        Err(e) => {
            if e == AccountError::UnknownError {
                log::error!("An unknown error occurred while loading the user. {}", e);
                return Err(reject::custom(UnknownErrorRejection));
            }
            log::warn!("Error occurred while loading the user. {}.", e);
            return Err(reject::custom(EmailVerificationRejection));
        }
    };
    user.email_verified = true;
    if let Err(e) = accounts::update_user(user).await {
        if e == AccountError::UnknownError {
            log::error!(
                "An unknown error occurred while marking the user account as verified. {}",
                e
            );
            return Err(reject::custom(UnknownErrorRejection));
        }
        log::warn!("An error occurred while verifying the user account. {}.", e);
        return Err(reject::custom(EmailVerificationRejection));
    }

    log::info!("Successfully validated the user's account.");
    Ok(StatusCode::OK)
}

/// Creates a warp response with an authorization header for a JWT, a refresh
/// token as a cookie, and a caller-specified status and body.
///
/// # Arguments
///
/// * `jwt` - The javascript web token to send in the authorization header
/// * `refresh_token` - A valid refresh token sent as an HttpOnly cookie
/// * `status_code` - An HTTP status code to send.
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
                "{}={}; Expires={}; path=/; HttpOnly; SameSite=Strict",
                REFRESH_TOKEN_COOKIE,
                refresh_token.token,
                exp_datetime.to_rfc2822()
            ),
        )
        .status(status_code)
        .body(serde_json::to_string(&jwt).expect("Could not serialize JWT."))
}
