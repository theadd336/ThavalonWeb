use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};
use scrypt::{errors::CheckError, ScryptParams};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, iter,
    sync::{Arc, Mutex},
};
use thiserror::Error;

const PASSWORD_MIN_LENGTH: usize = 8;

lazy_static! {
    /// Secret key used to create JWTs. In production, this should be set to an
    /// actually secure value.
    static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap_or("JWT_SECRET".to_string());
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ValidationError {
    #[error("Fatal hashing error")]
    HashError,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("User unathorized for this request.")]
    Unauthorized,
}

#[derive(Serialize, Debug)]
pub struct JWTResponse {
    token_type: String,
    access_token: String,
    expires_at: i64,
}

#[derive(Clone, Debug)]
pub struct RefreshTokenInfo {
    pub token: String,
    pub expires_at: i64,
    pub player_id: String,
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

#[derive(Clone)]
pub struct TokenManager {
    sub: &'static str,
    iss: &'static str,
    refresh_tokens: Arc<Mutex<HashMap<String, RefreshTokenInfo>>>,
}

impl TokenManager {
    /// Creates a new TokenManager with default values for JWT subject and issuer.
    pub fn new() -> TokenManager {
        TokenManager {
            sub: "ThavalonAuthenticatedUser",
            iss: "ThavalonGameServer",
            refresh_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a valid JWT using a user's ID.
    ///
    /// # Arguments
    ///
    /// * `player_id` - The user's ID to create a JWT with.
    ///
    /// # Returns
    ///
    /// A JWTResponse with the JWT
    pub async fn create_jwt(&mut self, player_id: &String) -> (JWTResponse, RefreshTokenInfo) {
        log::info!("Creating a new JWT for {}.", player_id);
        let time = Utc::now();
        let expiration_time = time
            .checked_add_signed(Duration::minutes(15))
            .expect("Failed to get expiration time.");
        let claims = JWTClaims {
            aud: player_id.clone(),
            exp: expiration_time.timestamp(),
            iat: time.timestamp(),
            iss: self.iss.to_string(),
            nbf: time.timestamp(),
            sub: self.sub.to_string(),
        };

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )
        .expect("Failed to generate a JWT for this claim.");

        log::info!("Successfully created a JWT for {}.", player_id);
        (
            JWTResponse {
                token_type: "Bearer".to_string(),
                access_token: token,
                expires_at: expiration_time.timestamp(),
            },
            self.create_refresh_token(player_id).await,
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
    pub async fn validate_jwt(&self, token: &str) -> Result<String, ValidationError> {
        log::info!("Validating received JWT");
        let validation = Validation {
            leeway: 60,
            validate_exp: true,
            validate_nbf: true,
            iss: Some(self.iss.to_string()),
            sub: Some(self.sub.to_string()),
            ..Validation::default()
        };

        let token_claims = match jsonwebtoken::decode::<JWTClaims>(
            &token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &validation,
        ) {
            Ok(data) => data.claims,
            Err(e) => {
                log::info!("Unable to validate claims for the the request. {}", e);
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
    /// * `user` - The ID of the user of the refresh token.
    ///
    /// # Returns
    ///
    /// A RefreshTokenInfo struct with all required information.
    pub async fn create_refresh_token(&mut self, user: &String) -> RefreshTokenInfo {
        log::info!("Creating a refresh token for {}.", user);
        let token: String;
        {
            let mut rng = rand::thread_rng();
            token = iter::repeat(())
                .map(|()| rng.sample(Alphanumeric))
                .take(32)
                .collect();
        }
        let token_info = RefreshTokenInfo {
            token: token.clone(),
            expires_at: Utc::now()
                .checked_add_signed(Duration::weeks(1))
                .expect("Could not create refresh token expires time.")
                .timestamp(),
            player_id: user.clone(),
        };

        self.refresh_tokens
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
    ///
    /// # Returns
    ///
    /// A new JWT and refresh token with an updated store.
    pub async fn renew_refresh_token(
        &mut self,
        refresh_token: String,
    ) -> Result<(JWTResponse, RefreshTokenInfo), ValidationError> {
        log::info!("Attempting to validate refresh token {}.", refresh_token);

        let token_info;
        {
            let mut token_store_locked = self
                .refresh_tokens
                .lock()
                .expect("Could not lock token store for validation.");

            token_info = match token_store_locked.remove(&refresh_token) {
                Some(info) => info,
                None => {
                    log::info!("Could not validate this request.");
                    return Err(ValidationError::Unauthorized);
                }
            };
            log::info!("Refresh token exists in DB. Validating expiration time.");
        }
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
        Ok(self.create_jwt(&token_info.player_id).await)
    }

    /// Revokes a refresh token, making the token invalid.
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - The refresh token to remove.
    pub async fn revoke_refresh_token(&mut self, refresh_token: &String) {
        log::info!("Revoking refresh token {}.", refresh_token);
        match self
            .refresh_tokens
            .lock()
            .expect("Failed to acquire lock on refresh token store.")
            .remove(refresh_token)
        {
            Some(_) => log::info!("Successfully revoked the refresh token."),
            None => log::info!("Refresh token does not exist to revoke."),
        };
    }
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
        log::error!("An RNG error occurred with the underlying OS. {}", e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use scrypt::ScryptParams;

    /// Tests hashing passwords against an insecure password.
    #[tokio::test]
    async fn test_hash_password_insecure() {
        let result = hash_password(&String::from("not_sec")).await;
        assert!(result.unwrap_err() == ValidationError::InvalidPassword);
    }

    /// Tests hasing passwords against a secure password.
    #[tokio::test]
    async fn test_hash_password_normal_pw() {
        let password = String::from("32fdsfjidsfj293423");
        let result = hash_password(&password)
            .await
            .expect("Failed to hash password correctly.");
        scrypt::scrypt_check(&password, &result).expect("Failed to match password hashes.");
    }

    /// Tests validating a password with a matching hash.
    #[tokio::test]
    async fn test_validate_password_match() {
        let password = String::from("asdfwe322ef2342");
        let hash = scrypt::scrypt_simple(&password, &ScryptParams::recommended()).unwrap();
        assert!(validate_password(&password, &hash).await);
    }

    /// Tests validating a password with an invalid hash. This shouldn't match.
    #[tokio::test]
    async fn test_validate_password_bad_hash() {
        let password = String::from("23qsadf2323f");
        assert!(!validate_password(&password, &password).await);
    }

    /// Tests validating a password with a mismatched hash.
    #[tokio::test]
    async fn test_validate_password_mismatch() {
        let password = String::from("32f23f2ef23");
        let other_password = String::from("342f98j98j34gf");
        let hash = scrypt::scrypt_simple(&other_password, &ScryptParams::recommended()).unwrap();
        assert!(!validate_password(&password, &hash).await);
    }

    /// Tests creating a JWT for a given player ID. Expected results generated from jwt.io.
    #[tokio::test]
    async fn test_create_jwt_valid() {
        let mut mananger = TokenManager::new();
        let (jwt, _) = mananger.create_jwt(&String::from("TESTING_THIS")).await;
        let expires_at = Utc::now()
            .checked_add_signed(Duration::minutes(15))
            .unwrap()
            .timestamp();
        assert!(expires_at - 60 <= jwt.expires_at);
        assert!(jwt.expires_at <= expires_at + 60);
        assert_eq!(jwt.token_type.as_str(), "Bearer");
    }

    /// Tests validate_jwt with a valid JWT.
    #[tokio::test]
    async fn test_validate_jwt_valid() {
        let mut manager = TokenManager::new();
        let input_player = String::from("TESTING");
        let (jwt, _) = manager.create_jwt(&input_player).await;
        let player_id = manager
            .validate_jwt(&jwt.access_token)
            .await
            .expect("Token was marked as invalid, but should be valid.");

        assert_eq!(player_id, input_player);
    }

    /// Tests validate_jwt with a tampered JWT.
    #[tokio::test]
    async fn test_validate_jwt_invalid() {
        let mut manager = TokenManager::new();
        let input_player = String::from("TESTING");
        let (mut jwt, _) = manager.create_jwt(&input_player).await;
        jwt.access_token.insert(5, 'A');
        let result = manager
            .validate_jwt(&jwt.access_token)
            .await
            .expect_err("WARNING: invalid JWT showing as valid.");
        assert_eq!(result, ValidationError::Unauthorized);
    }

    /// Tests create_refresh_token for a valid refresh token.
    #[tokio::test]
    async fn test_create_refresh_token() {
        let mut manager = TokenManager::new();
        let player = String::from("TESTING");
        let token = manager.create_refresh_token(&player).await;

        let expires_at = Utc::now()
            .checked_add_signed(Duration::weeks(1))
            .unwrap()
            .timestamp();
        assert_eq!(token.player_id, player);
        assert!(expires_at - 60 <= token.expires_at);
        assert!(token.expires_at <= expires_at + 60);
        assert_eq!(
            manager
                .refresh_tokens
                .lock()
                .unwrap()
                .get(&token.token)
                .unwrap()
                .player_id,
            player
        );
    }

    /// Tests renewing a refresh token with a valid refresh token
    #[tokio::test]
    async fn test_renew_refresh_token_valid() {
        let mut manager = TokenManager::new();
        let player_id = String::from("TESTING");
        let token = manager.create_refresh_token(&player_id).await;

        let (_, new_token) = manager
            .renew_refresh_token(token.token.clone())
            .await
            .expect("Failed to generate a new refresh token with a valid refresh token.");
        assert!(new_token.player_id == player_id);
        assert!(manager
            .refresh_tokens
            .lock()
            .unwrap()
            .contains_key(&new_token.token));
        assert!(!manager
            .refresh_tokens
            .lock()
            .unwrap()
            .contains_key(&token.token));
    }

    /// Tests renewing a refresh token with an invalid refresh token.
    #[tokio::test]
    async fn test_renew_refresh_token_invalid() {
        let mut manager = TokenManager::new();
        manager.create_refresh_token(&String::from("TESTING")).await;

        if let Err(e) = manager
            .renew_refresh_token(String::from("WER@#R@F@#"))
            .await
        {
            assert_eq!(e, ValidationError::Unauthorized);
        } else {
            panic!("ERROR: Successfully validated an invalid refresh token.");
        }
    }

    /// Tests revoking a refresh token with both a valid and invalid token. The results should be the same.
    #[tokio::test]
    async fn test_revoke_refresh_token() {
        let mut manager = TokenManager::new();
        let info = manager.create_refresh_token(&String::from("TESTING")).await;
        manager.revoke_refresh_token(&info.token).await;
        assert!(!manager
            .refresh_tokens
            .lock()
            .unwrap()
            .contains_key(&info.token));

        let result = manager
            .renew_refresh_token(info.token.clone())
            .await
            .expect_err("ERROR: revoked refresh token is still valid.");
        assert_eq!(result, ValidationError::Unauthorized);

        // This shouldn't crash here.
        manager
            .revoke_refresh_token(&"This shouldn't crash".to_string())
            .await;
    }
}
