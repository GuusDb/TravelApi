use actix_web::{web, HttpResponse, Responder};
use log::info;
use serde::Serialize;
use utoipa::ToSchema;

use crate::db::connection::DbPool;
use crate::models::user::{LoginCredentials, NewUser};
use crate::services::auth_service::{AuthService, AuthError};

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    message: String,
    user_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    token: String,
    token_type: String,
    expires_in: i64,
    user_id: String,
    username: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    post,
    path = "/api/register",
    request_body = NewUser,
    responses(
        (status = 201, description = "User created successfully", body = RegisterResponse),
        (status = 409, description = "Username already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn register(
    pool: web::Data<DbPool>,
    user_data: web::Json<NewUser>,
) -> impl Responder {
    info!("Received registration request for user: {}", user_data.username);
    
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match AuthService::register(&conn, &user_data) {
        Ok(user) => {
            HttpResponse::Created().json(RegisterResponse {
                message: "User registered successfully".to_string(),
                user_id: user.id,
            })
        }
        Err(AuthError::UsernameTaken) => {
            HttpResponse::Conflict().json(ErrorResponse {
                error: "Username already exists".to_string(),
            })
        }
        Err(AuthError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to register user".to_string(),
            })
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginCredentials,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    pool: web::Data<DbPool>,
    credentials: web::Json<LoginCredentials>,
) -> impl Responder {
    info!("Received login request for user: {}", credentials.username);
    
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match AuthService::login(&conn, &credentials) {
        Ok((user, token, expires_in)) => {
            HttpResponse::Ok().json(LoginResponse {
                token,
                token_type: "Bearer".to_string(),
                expires_in,
                user_id: user.id,
                username: user.username,
            })
        }
        Err(AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            })
        }
        Err(AuthError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
        Err(AuthError::TokenGenerationError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Token generation error: {}", e),
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to authenticate user".to_string(),
            })
        }
    }
}