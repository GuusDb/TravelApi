use actix_web::{test, web, App, http::header};
use chrono::Utc;
use rusqlite::Connection;
use serde_json::json;

use crate::db::schema;
use crate::middleware::auth::{generate_token, AuthenticatedUser};
use crate::models::travel_plan::{NewTravelPlan, TravelPlan, UpdateTravelPlan};
use crate::models::user::{NewUser, User};
use crate::routes::auth::register;
use crate::routes::travel_plan::{
    create_travel_plan, delete_travel_plan, get_travel_plan_by_id, get_travel_plans, update_travel_plan
};

// Helper function to create a test user and get a token
async fn create_test_user_and_token(app: &impl actix_web::dev::Service<actix_web::dev::ServiceRequest, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>, conn: &Connection) -> (String, String) {
    // Create a test user
    let user_data = NewUser {
        username: "testuser".to_string(),
        password: "password123".to_string(),
        email: "test@example.com".to_string(),
    };
    
    let user = User::create(conn, &user_data).unwrap();
    let token = generate_token(&user).unwrap();
    
    (user.id, token.token)
}

#[actix_web::test]
async fn test_create_travel_plan() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn.clone());
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/travelplan", web::post().to(create_travel_plan))
    ).await;
    
    // Create a test user and get a token
    let (user_id, token) = create_test_user_and_token(&app, &conn).await;
    
    // Create test travel plan data
    let plan_data = NewTravelPlan {
        user_id: user_id.clone(),
        name: "Test Travel Plan".to_string(),
        description: Some("A test travel plan".to_string()),
        start_location: "New York".to_string(),
        end_location: "Los Angeles".to_string(),
        start_date: Some(Utc::now()),
        end_date: Some(Utc::now()),
    };
    
    // Send create travel plan request
    let req = test::TestRequest::post()
        .uri("/travelplan")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&plan_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: TravelPlan = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains expected fields
    assert_eq!(response.name, plan_data.name);
    assert_eq!(response.description, plan_data.description);
    assert_eq!(response.start_location, plan_data.start_location);
    assert_eq!(response.end_location, plan_data.end_location);
    assert_eq!(response.user_id, user_id);
}

#[actix_web::test]
async fn test_get_travel_plans() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn.clone());
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/travelplan", web::post().to(create_travel_plan))
            .route("/travelplan", web::get().to(get_travel_plans))
    ).await;
    
    // Create a test user and get a token
    let (user_id, token) = create_test_user_and_token(&app, &conn).await;
    
    // Create multiple test travel plans
    for i in 1..=3 {
        let plan_data = NewTravelPlan {
            user_id: user_id.clone(),
            name: format!("Test Travel Plan {}", i),
            description: Some(format!("A test travel plan {}", i)),
            start_location: "New York".to_string(),
            end_location: "Los Angeles".to_string(),
            start_date: Some(Utc::now()),
            end_date: Some(Utc::now()),
        };
        
        let req = test::TestRequest::post()
            .uri("/travelplan")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(&plan_data)
            .to_request();
        
        let _ = test::call_service(&app, req).await;
    }
    
    // Send get travel plans request
    let req = test::TestRequest::get()
        .uri("/travelplan")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: Vec<TravelPlan> = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains expected number of travel plans
    assert_eq!(response.len(), 3);
    
    // Assert all travel plans belong to the test user
    for plan in response {
        assert_eq!(plan.user_id, user_id);
    }
}

#[actix_web::test]
async fn test_get_travel_plan_by_id() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn.clone());
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/travelplan", web::post().to(create_travel_plan))
            .route("/travelplan/{id}", web::get().to(get_travel_plan_by_id))
    ).await;
    
    // Create a test user and get a token
    let (user_id, token) = create_test_user_and_token(&app, &conn).await;
    
    // Create a test travel plan
    let plan_data = NewTravelPlan {
        user_id: user_id.clone(),
        name: "Test Travel Plan".to_string(),
        description: Some("A test travel plan".to_string()),
        start_location: "New York".to_string(),
        end_location: "Los Angeles".to_string(),
        start_date: Some(Utc::now()),
        end_date: Some(Utc::now()),
    };
    
    let req = test::TestRequest::post()
        .uri("/travelplan")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&plan_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Parse response body to get the created travel plan
    let body = test::read_body(resp).await;
    let created_plan: TravelPlan = serde_json::from_slice(&body).unwrap();
    
    // Send get travel plan by ID request
    let req = test::TestRequest::get()
        .uri(&format!("/travelplan/{}", created_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: TravelPlan = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains expected fields
    assert_eq!(response.id, created_plan.id);
    assert_eq!(response.name, plan_data.name);
    assert_eq!(response.description, plan_data.description);
    assert_eq!(response.start_location, plan_data.start_location);
    assert_eq!(response.end_location, plan_data.end_location);
    assert_eq!(response.user_id, user_id);
    
    // Try to get a non-existent travel plan
    let req = test::TestRequest::get()
        .uri("/travelplan/nonexistent-id")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is not found
    assert_eq!(resp.status().as_u16(), 404);
}

#[actix_web::test]
async fn test_update_travel_plan() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn.clone());
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/travelplan", web::post().to(create_travel_plan))
            .route("/travelplan/{id}", web::put().to(update_travel_plan))
    ).await;
    
    // Create a test user and get a token
    let (user_id, token) = create_test_user_and_token(&app, &conn).await;
    
    // Create a test travel plan
    let plan_data = NewTravelPlan {
        user_id: user_id.clone(),
        name: "Test Travel Plan".to_string(),
        description: Some("A test travel plan".to_string()),
        start_location: "New York".to_string(),
        end_location: "Los Angeles".to_string(),
        start_date: Some(Utc::now()),
        end_date: Some(Utc::now()),
    };
    
    let req = test::TestRequest::post()
        .uri("/travelplan")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&plan_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Parse response body to get the created travel plan
    let body = test::read_body(resp).await;
    let created_plan: TravelPlan = serde_json::from_slice(&body).unwrap();
    
    // Create update data
    let update_data = UpdateTravelPlan {
        name: Some("Updated Travel Plan".to_string()),
        description: Some("An updated test travel plan".to_string()),
        start_location: Some("Boston".to_string()),
        end_location: Some("San Francisco".to_string()),
        start_date: None,
        end_date: None,
    };
    
    // Send update travel plan request
    let req = test::TestRequest::put()
        .uri(&format!("/travelplan/{}", created_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: TravelPlan = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains updated fields
    assert_eq!(response.id, created_plan.id);
    assert_eq!(response.name, update_data.name.unwrap());
    assert_eq!(response.description, update_data.description);
    assert_eq!(response.start_location, update_data.start_location.unwrap());
    assert_eq!(response.end_location, update_data.end_location.unwrap());
    assert_eq!(response.user_id, user_id);
}

#[actix_web::test]
async fn test_delete_travel_plan() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn.clone());
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/travelplan", web::post().to(create_travel_plan))
            .route("/travelplan/{id}", web::get().to(get_travel_plan_by_id))
            .route("/travelplan/{id}", web::delete().to(delete_travel_plan))
    ).await;
    
    // Create a test user and get a token
    let (user_id, token) = create_test_user_and_token(&app, &conn).await;
    
    // Create a test travel plan
    let plan_data = NewTravelPlan {
        user_id: user_id.clone(),
        name: "Test Travel Plan".to_string(),
        description: Some("A test travel plan".to_string()),
        start_location: "New York".to_string(),
        end_location: "Los Angeles".to_string(),
        start_date: Some(Utc::now()),
        end_date: Some(Utc::now()),
    };
    
    let req = test::TestRequest::post()
        .uri("/travelplan")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&plan_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Parse response body to get the created travel plan
    let body = test::read_body(resp).await;
    let created_plan: TravelPlan = serde_json::from_slice(&body).unwrap();
    
    // Send delete travel plan request
    let req = test::TestRequest::delete()
        .uri(&format!("/travelplan/{}", created_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful (no content)
    assert_eq!(resp.status().as_u16(), 204);
    
    // Try to get the deleted travel plan
    let req = test::TestRequest::get()
        .uri(&format!("/travelplan/{}", created_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is not found
    assert_eq!(resp.status().as_u16(), 404);
}