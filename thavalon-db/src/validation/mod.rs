use crate::database;
use scrypt::{errors::CheckError, ScryptParams};
use serde::Deserialize;
use thiserror::Error;
const PASSWORD_MIN_LENGTH: usize = 8;

/// Representation of a database admin before authorization.
/// Password is in plain text at this stage.
#[derive(Deserialize)]
pub struct DBAdminPreAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ValidationError {
    #[error("Fatal hashing error")]
    HashError,
    #[error("User does not exist.")]
    InvalidUserError,
    #[error("Invalid password")]
    InvalidPassword,
}

/// Validates a given database admin user against the database's hash.
///
/// # Arguments
///
/// * `db_admin` - admin user attempting to authenticate.
///
/// # Returns
///
/// * () on success, ValidationError on failure
pub async fn validate_admin(db_admin: &DBAdminPreAuth) -> Result<(), ValidationError> {
    let user = match database::load_db_admin(&db_admin.username).await {
        Ok(user) => user,
        Err(e) => {
            log::warn!("{:?}", e);
            return Err(ValidationError::InvalidUserError);
        }
    };

    let is_valid = validate_password(&db_admin.password, &user.hash).await;
    if !is_valid {
        log::info!("Invalid password for {}.", db_admin.username.clone());
        return Err(ValidationError::InvalidPassword);
    }
    Ok(())
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
