use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use scrypt::{errors::CheckError, ScryptParams};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
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
    expires_in: u8,
    refresh_token: String,
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

pub async fn create_JWT(user_email: &String) -> JWTResponse {
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
    JWTResponse {
        token_type: "Bearer".to_string(),
        access_token: token,
        expires_in: 60,
        refresh_token: "".to_string(),
    }
}

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
