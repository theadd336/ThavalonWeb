/// Rest handlers for account-based calls
use crate::database::{self, account_errors::AccountError, DatabaseAccount};
use crate::validation::{self, DBAdminPreAuth, ValidationError};
use serde::{Deserialize, Serialize};
use serde_json;
use warp::{
    http::StatusCode,
    reject::{self, Reject},
    reply::{self, Response},
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

/// Handles a request to authenticate an admin.
///
/// # Arguments
///
/// * `db_admin` - canonical representation of a database admin to authenticate.
pub async fn handle_admin_auth_request(header: String) -> Result<(), Rejection> {
    log::info!("Attempting to deserialize auth string.");
    log::debug!("{}", header);
    let db_admin: DBAdminPreAuth = match serde_json::from_str(&header) {
        Ok(admin) => admin,
        Err(_) => {
            log::warn!("Failed to deserialize auth string to a preauth admin account.");
            return Err(warp::reject::custom(ValidationRejection));
        }
    };
    if let Err(e) = validation::validate_admin(&db_admin).await {
        if e == ValidationError::HashError {
            log::error!(
                "Received fatal hashing error while trying to authorize {}.",
                db_admin.username
            );
        } else {
            log::info!("Failed to validate {}. {:?}", db_admin.username, e);
        }
        return Err(warp::reject::custom(ValidationRejection));
    }
    log::info!("Successfully validated {}.", db_admin.username);
    Ok(())
}

/// Handles a request to add a user to the database.
///
/// # Arguments
///
/// `new_user` - The ThavalonUser to add
///
/// # Returns
///
/// * Success reply on success, a variety of rejections otherwise.
pub async fn handle_add_user(new_user: ThavalonUser) -> Result<impl Reply, Rejection> {
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
    Ok(warp::reply())
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
