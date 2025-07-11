use actix_web::{test, web, App, http::header};
use chrono::Utc;
use rusqlite::Connection;
use serde_json::json;

use crate::db::schema;
use crate::middleware::auth::{generate_token, AuthenticatedUser};
use crate::models::travel_plan::{NewTravelPlan, TravelPlan};
use crate::models::route_option::RouteOption;
use crate::models::user::{NewUser, User};
use crate::routes::auth::register;
use crate::routes::travel_plan::create_travel_plan;
use crate::routes::route_option::{
    generate_route_options, get_route_options, get_route_option_by_id
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

// Helper function to create a test travel plan
async fn create_test_travel_plan(
    app: &impl actix_web::dev::Service<actix_web::dev::ServiceRequest, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>,
    user_id: &str,
    token: &str
) -> TravelPlan {
    // Create test travel plan data
    let plan_data = NewTravelPlan {
        user_id: user_id.to_string(),
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
    
    let resp = test::call_service(app, req).await;
    
    // Parse response body
    let body = test::read_body(resp).await;
    serde_json::from_slice(&body).unwrap()
}

#[actix_web::test]
async fn test_generate_route_options() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn.clone());
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/travelplan", web::post().to(create_travel_plan))
            .route("/travelplan/{id}/routes/generate", web::post().to(generate_route_options))
    ).await;
    
    // Create a test user and get a token
    let (user_id, token) = create_test_user_and_token(&app, &conn).await;
    
    // Create a test travel plan
    let travel_plan = create_test_travel_plan(&app, &user_id, &token).await;
    
    // Send generate route options request
    let req = test::TestRequest::post()
        .uri(&format!("/travelplan/{}/routes/generate?count=3", travel_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains expected number of route options
    assert_eq!(response.len(), 3);
    
    // Assert each route option has the expected structure
    for route_option in response {
        assert!(route_option.get("route").is_some());
        assert!(route_option.get("points_of_interest").is_some());
        
        let route = route_option.get("route").unwrap();
        assert!(route.get("id").is_some());
        assert!(route.get("travel_plan_id").is_some());
        assert!(route.get("name").is_some());
        assert!(route.get("start_coordinates").is_some());
        assert!(route.get("end_coordinates").is_some());
        
        // Assert the route belongs to the travel plan
        assert_eq!(route.get("travel_plan_id").unwrap().as_str().unwrap(), travel_plan.id);
        
        // Assert points of interest are present
        let pois = route_option.get("points_of_interest").unwrap().as_array().unwrap();
        assert!(!pois.is_empty());
        
        // Assert each point of interest has the expected structure
        for poi in pois {
            assert!(poi.get("id").is_some());
            assert!(poi.get("route_option_id").is_some());
            assert!(poi.get("name").is_some());
            assert!(poi.get("coordinates").is_some());
            
            // Assert the point of interest belongs to the route
            assert_eq!(
                poi.get("route_option_id").unwrap().as_str().unwrap(),
                route.get("id").unwrap().as_str().unwrap()
            );
        }
    }
}

#[actix_web::test]
async fn test_get_route_options() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn.clone());
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/travelplan", web::post().to(create_travel_plan))
            .route("/travelplan/{id}/routes/generate", web::post().to(generate_route_options))
            .route("/travelplan/{id}/routes", web::get().to(get_route_options))
    ).await;
    
    // Create a test user and get a token
    let (user_id, token) = create_test_user_and_token(&app, &conn).await;
    
    // Create a test travel plan
    let travel_plan = create_test_travel_plan(&app, &user_id, &token).await;
    
    // Generate route options
    let req = test::TestRequest::post()
        .uri(&format!("/travelplan/{}/routes/generate?count=3", travel_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let _ = test::call_service(&app, req).await;
    
    // Send get route options request
    let req = test::TestRequest::get()
        .uri(&format!("/travelplan/{}/routes", travel_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains expected number of route options
    assert_eq!(response.len(), 3);
}

#[actix_web::test]
async fn test_get_route_option_by_id() {
    // Set up in-memory database for testing
    let conn = Connection::open_in_memory().unwrap();
    schema::initialize_database(&conn).unwrap();
    let app_data = web::Data::new(conn.clone());
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_data.clone())
            .route("/travelplan", web::post().to(create_travel_plan))
            .route("/travelplan/{id}/routes/generate", web::post().to(generate_route_options))
            .route("/travelplan/{plan_id}/routes/{route_id}", web::get().to(get_route_option_by_id))
    ).await;
    
    // Create a test user and get a token
    let (user_id, token) = create_test_user_and_token(&app, &conn).await;
    
    // Create a test travel plan
    let travel_plan = create_test_travel_plan(&app, &user_id, &token).await;
    
    // Generate route options
    let req = test::TestRequest::post()
        .uri(&format!("/travelplan/{}/routes/generate?count=1", travel_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Parse response body to get the generated route option
    let body = test::read_body(resp).await;
    let response: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    
    // Get the first route option
    let route_option = response[0].get("route").unwrap();
    let route_id = route_option.get("id").unwrap().as_str().unwrap();
    
    // Send get route option by ID request
    let req = test::TestRequest::get()
        .uri(&format!("/travelplan/{}/routes/{}", travel_plan.id, route_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is successful
    assert!(resp.status().is_success());
    
    // Parse response body
    let body = test::read_body(resp).await;
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Assert response contains expected fields
    assert!(response.get("route").is_some());
    assert!(response.get("points_of_interest").is_some());
    
    let route = response.get("route").unwrap();
    assert_eq!(route.get("id").unwrap().as_str().unwrap(), route_id);
    assert_eq!(route.get("travel_plan_id").unwrap().as_str().unwrap(), travel_plan.id);
    
    // Try to get a non-existent route option
    let req = test::TestRequest::get()
        .uri(&format!("/travelplan/{}/routes/nonexistent-id", travel_plan.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Assert response is not found
    assert_eq!(resp.status().as_u16(), 404);
}