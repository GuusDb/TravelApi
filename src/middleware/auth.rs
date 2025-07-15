use actix_web::{
    dev::Payload, error::ErrorUnauthorized, http::header, web, Error, FromRequest, HttpRequest,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use log::{error, info};
use utoipa::ToSchema;

use crate::db::connection::{DbPool, DbConnection};
use crate::models::user::User;

const JWT_SECRET: &[u8] = b"secret_key_for_jwt_token_generation";
const TOKEN_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthToken {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl Claims {
    pub fn new(user_id: &str, username: &str) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(TOKEN_EXPIRATION_HOURS);
        
        Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        }
    }
}

pub fn generate_token(user: &User) -> Result<AuthToken, jsonwebtoken::errors::Error> {
    let claims = Claims::new(&user.id, &user.username);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )?;
    
    Ok(AuthToken {
        token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_EXPIRATION_HOURS * 3600,
    })
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )?;
    
    Ok(token_data.claims)
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get(header::AUTHORIZATION);
        let auth_header = match auth_header {
            Some(header) => header,
            None => {
                return ready(Err(ErrorUnauthorized("No authorization header found")));
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return ready(Err(ErrorUnauthorized("Invalid authorization header")));
            }
        };

        if !auth_str.starts_with("Bearer ") {
            return ready(Err(ErrorUnauthorized("Invalid authorization scheme")));
        }

        let token = &auth_str[7..];

        match validate_token(token) {
            Ok(claims) => {
                ready(Ok(AuthenticatedUser {
                    user_id: claims.sub,
                    username: claims.username,
                }))
            }
            Err(e) => {
                error!("Token validation error: {}", e);
                ready(Err(ErrorUnauthorized("Invalid token")))
            }
        }
    }
}

pub struct AuthDbConn(#[allow(dead_code)] pub DbConnection);

impl FromRequest for AuthDbConn {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let pool = match req.app_data::<web::Data<DbPool>>() {
            Some(pool) => pool.get_ref(),
            None => {
                error!("Database pool not found in application data");
                return ready(Err(ErrorUnauthorized("Database error")));
            }
        };

        match pool.get() {
            Ok(conn) => {
                ready(Ok(AuthDbConn(conn)))
            },
            Err(e) => {
                error!("Failed to get database connection from pool: {}", e);
                ready(Err(ErrorUnauthorized("Database error")))
            }
        }
    }
}

#[allow(dead_code)]
pub async fn require_auth(
    _req: HttpRequest,
    auth_user: Option<AuthenticatedUser>,
) -> Result<AuthenticatedUser, Error> {
    match auth_user {
        Some(user) => {
            info!("Authenticated request from user: {}", user.username);
            Ok(user)
        }
        None => {
            error!("Unauthenticated request to protected endpoint");
            Err(ErrorUnauthorized("Authentication required"))
        }
    }
}