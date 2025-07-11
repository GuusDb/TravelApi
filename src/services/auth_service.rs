use rusqlite::Connection;
use log::{error, info};

use crate::middleware::auth::generate_token;
use crate::models::user::{LoginCredentials, NewUser, User};

pub struct AuthService;

#[derive(Debug)]
pub enum AuthError {
    UsernameTaken,
    InvalidCredentials,
    DatabaseError(String),
    TokenGenerationError(String),
}

impl AuthService {
    pub fn register(conn: &Connection, user_data: &NewUser) -> Result<User, AuthError> {
        info!("Registering new user: {}", user_data.username);
        
        // Check if username already exists
        match User::find_by_username(conn, &user_data.username) {
            Ok(Some(_)) => {
                info!("Username already exists: {}", user_data.username);
                return Err(AuthError::UsernameTaken);
            }
            Ok(None) => {}
            Err(e) => {
                error!("Database error during user lookup: {}", e);
                return Err(AuthError::DatabaseError(e.to_string()));
            }
        }
        
        // Create new user
        match User::create(conn, user_data) {
            Ok(user) => {
                info!("User registered successfully: {}", user.username);
                Ok(user)
            }
            Err(e) => {
                error!("Error creating user: {}", e);
                Err(AuthError::DatabaseError(e.to_string()))
            }
        }
    }
    
    pub fn login(conn: &Connection, credentials: &LoginCredentials) -> Result<(User, String, i64), AuthError> {
        info!("Authenticating user: {}", credentials.username);
        
        // Authenticate user
        match User::authenticate(conn, credentials) {
            Ok(Some(user)) => {
                // Generate JWT token
                match generate_token(&user) {
                    Ok(token) => {
                        info!("User logged in successfully: {}", user.username);
                        Ok((user, token.token, token.expires_in))
                    }
                    Err(e) => {
                        error!("Error generating token: {}", e);
                        Err(AuthError::TokenGenerationError(e.to_string()))
                    }
                }
            }
            Ok(None) => {
                info!("Login failed for user: {}", credentials.username);
                Err(AuthError::InvalidCredentials)
            }
            Err(e) => {
                error!("Database error during authentication: {}", e);
                Err(AuthError::DatabaseError(e.to_string()))
            }
        }
    }
}