use actix_web::{test, web, App};
use rusqlite::Connection;
use serde_json::json;

use crate::db::connection;
use crate::db::schema;
use crate::models::user::{LoginCredentials, NewUser};
use crate::routes::auth::{login, register};

#[actix_web::test]
async fn test_user_registration() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn);
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/register", web::post().to(register))
    ).await;
    
    // Create test user data
    let user_data = NewUser {
        username: "testuser".to_string(),
        password: "password123".to_string(),
        email: "test@example.com".to_string(),
    };
    
    // Send registration request
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&user_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains expected fields
    assert!(response.get("message").is_some());
    assert!(response.get("user_id").is_some());
    
    // Try to register the same user again (should fail)
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&user_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is conflict
    assert_eq!(resp.status().as_u16(), 409);
}

#[actix_web::test]
async fn test_user_login() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn);
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    ).await;
    
    // Create and register a test user
    let user_data = NewUser {
        username: "testuser".to_string(),
        password: "password123".to_string(),
        email: "test@example.com".to_string(),
    };
    
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&user_data)
        .to_request();
    
    let _ = test::call_service(&app, req).await;
    
    // Try to login with correct credentials
    let login_data = LoginCredentials {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };
    
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains expected fields
    assert!(response.get("token").is_some());
    assert!(response.get("token_type").is_some());
    assert!(response.get("expires_in").is_some());
    assert!(response.get("user_id").is_some());
    assert!(response.get("username").is_some());
    
    // Try to login with incorrect password
    let login_data = LoginCredentials {
        username: "testuser".to_string(),
        password: "wrongpassword".to_string(),
    };
    
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is unauthorized
    assert_eq!(resp.status().as_u16(), 401);
    
    // Try to login with non-existent user
    let login_data = LoginCredentials {
        username: "nonexistentuser".to_string(),
        password: "password123".to_string(),
    };
    
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is unauthorized
    assert_eq!(resp.status().as_u16(), 401);
}