use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use scrypt::{errors::CheckError, ScryptParams};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::iter;
use std::sync::{Arc, Mutex};
use thiserror::Error;

const PASSWORD_MIN_LENGTH: usize = 8;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ValidationError {
    #[error("Fatal hashing error")]
    HashError,
    #[error("User does not exist.")]
    InvalidUserError,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("User unathorized for this request.")]
    Unauthorized,
}

#[derive(Serialize)]
pub struct JWTResponse {
    token_type: String,
    access_token: String,
    expires_at: i64,
}

#[derive(Clone)]
pub struct RefreshTokenInfo {
    pub token: String,
    pub expires_at: i64,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    aud: String,
    exp: i64,
    iat: i64,
    iss: String,
    nbf: i64,
    sub: String,
}

pub type TokenStore = Arc<Mutex<HashMap<String, RefreshTokenInfo>>>;

/// Hashes a plaintext password using the currently selected hashing algorithm.
///
/// # Arguments
///
/// * `plaintext` - the plain text password to be hashed
///
/// # Returns
///
/// * `Result<password_hash, error>`
pub async fn hash_password(plaintext: &String) -> Result<String, ValidationError> {
    if plaintext.len() < PASSWORD_MIN_LENGTH {
        log::warn!("Received a password below minimum security specs");
        return Err(ValidationError::InvalidPassword);
    }

    let hash = scrypt::scrypt_simple(plaintext, &ScryptParams::recommended()).map_err(|e| {
        log::error!("An RNG error occurred with the underlying OS.");
        log::error!("{:?}", e);
        ValidationError::HashError
    });

    hash
}

/// Validates a plaintext password against a given hash.
///
/// # Arguments
///
/// * `plaintext` - the plain text password to check
/// * `hash` - Password hash in scrypt format
///
/// # Returns
/// True if passwords match. False otherwise.
pub async fn validate_password(plaintext: &String, hash: &String) -> bool {
    let result = match scrypt::scrypt_check(plaintext, hash) {
        Ok(_) => true,
        Err(e) => {
            if e == CheckError::InvalidFormat {
                log::error!("Database hash is not in a valid scrypt format.");
            }
            false
        }
    };

    result
}

/// Creates a valid JWT using a user's email.
///
/// # Arguments
///
/// * `user_email` - The user's email to create a JWT with.
///
/// # Returns
///
/// A JWTResponse with the JWT
pub async fn create_JWT(
    user_email: &String,
    token_store: TokenStore,
) -> (JWTResponse, RefreshTokenInfo) {
    log::info!("Creating a new JWT for {}.", user_email);
    let jwt_secret = env::var("JWT_SECRET").unwrap_or("JWT_SECRET".to_string());
    let time = Utc::now();
    let expiration_time = time
        .checked_add_signed(Duration::minutes(60))
        .expect("Failed to get expiration time.");
    let claims = JWTClaims {
        aud: user_email.clone(),
        exp: expiration_time.timestamp(),
        iat: time.timestamp(),
        iss: "ThavalonGameServer".to_string(),
        nbf: time.timestamp(),
        sub: "ThavalonAuthenticatedUser".to_string(),
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("Failed to generate a JWT for this claim.");

    log::info!("Successfully created a JWT for {}.", user_email);
    (
        JWTResponse {
            token_type: "Bearer".to_string(),
            access_token: token,
            expires_at: expiration_time.timestamp(),
        },
        create_refresh_token(user_email, token_store).await,
    )
}

/// Validates a JWT given the token.
///
/// # Arguments
///
/// * `token` - A JWT to authenticate
///
/// # Returns
///
/// User ID on success, ValidationError on failure.
pub async fn validate_jwt(token: &str) -> Result<String, ValidationError> {
    log::info!("Validating received JWT");
    let jwt_secret = env::var("JWT_SECRET").unwrap_or("JWT_SECRET".to_string());
    let validation = Validation {
        leeway: 60,
        validate_exp: true,
        validate_nbf: true,
        iss: Some("ThavalonGameServer".to_string()),
        sub: Some("ThavalonAuthenticatedUser".to_string()),
        ..Validation::default()
    };

    let token_claims = match jsonwebtoken::decode::<JWTClaims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data.claims,
        Err(e) => {
            log::info!("Unable to validate claims for the the request.");
            log::info!("{:?}", e);
            return Err(ValidationError::Unauthorized);
        }
    };

    log::info!("Successfully validated {}.", token_claims.aud);
    Ok(token_claims.aud)
}

/// Creates a refresh token with a given expiration time and updates the token store.
///
/// # Arguments
///
/// * `user` - The email address of the user of the refresh token.
/// * `token_store` - A store of all valid refresh tokens. Must be passed unlocked.
///
/// # Returns
///
/// A RefreshTokenInfo struct with all required information.
pub async fn create_refresh_token(user: &String, token_store: TokenStore) -> RefreshTokenInfo {
    log::info!("Creating a refresh token for {}.", user);
    let token: String = iter::repeat(())
        .map(|()| rand::thread_rng().sample(Alphanumeric))
        .take(32)
        .collect();

    let token_info = RefreshTokenInfo {
        token: token.clone(),
        expires_at: Utc::now()
            .checked_add_signed(Duration::weeks(1))
            .expect("Could not create refresh token expires time.")
            .timestamp(),
        email: user.clone(),
    };

    token_store
        .lock()
        .expect("Could not lock refresh token store.")
        .insert(token, token_info.clone());
    token_info
}

/// Validates a refresh token, generating a new JWT and refresh token if valid.
///
/// # Arguments
///
/// * `refresh_token` - String of the refresh token to validate
/// * `token_store` - The store of all current refresh tokens.
///
/// # Returns
///
/// A new JWT and refresh token with an updated store.
pub async fn validate_refresh_token(
    refresh_token: String,
    token_store: TokenStore,
) -> Result<(JWTResponse, RefreshTokenInfo), ValidationError> {
    log::info!("Attempting to validate refresh token {}.", refresh_token);
    let mut token_store_locked = token_store
        .lock()
        .expect("Could not lock token store for validation.");

    let token_info = match token_store_locked.remove(&refresh_token) {
        Some(info) => info,
        None => {
            log::info!("Could not validate this request.");
            return Err(ValidationError::Unauthorized);
        }
    };
    log::info!("Refresh token exists in DB. Validating expiration time.");

    let time = Utc::now().timestamp();
    if time > token_info.expires_at {
        log::info!(
            "Token is not valid. Expired at {}, current time is {}.",
            token_info.expires_at,
            time
        );
        return Err(ValidationError::Unauthorized);
    }

    log::info!("Token is valid. Sending new JWT.");
    Ok(create_JWT(&token_info.email, token_store).await)
}
