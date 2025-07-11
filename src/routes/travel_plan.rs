use actix_web::{web, HttpResponse, Responder};
use log::info;
use serde::Serialize;

use crate::db::connection::DbPool;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::travel_plan::{NewTravelPlan, UpdateTravelPlan};
use crate::services::travel_plan_service::{TravelPlanService, TravelPlanError};

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// Get all travel plans for the authenticated user
pub async fn get_travel_plans(
    pool: web::Data<DbPool>,
    auth_user: web::ReqData<AuthenticatedUser>,
) -> impl Responder {
    let auth_user = auth_user.into_inner();
    info!("Fetching travel plans for user: {}", auth_user.username);
    
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match TravelPlanService::get_travel_plans(&conn, &auth_user.user_id) {
        Ok(plans) => {
            HttpResponse::Ok().json(plans)
        }
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch travel plans".to_string(),
            })
        }
    }
}

// Get a specific travel plan by ID
pub async fn get_travel_plan_by_id(
    pool: web::Data<DbPool>,
    auth_user: web::ReqData<AuthenticatedUser>,
    path: web::Path<String>,
) -> impl Responder {
    let plan_id = path.into_inner();
    let auth_user = auth_user.into_inner();
    info!("Fetching travel plan with ID: {} for user: {}", plan_id, auth_user.username);
    
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match TravelPlanService::get_travel_plan_by_id(&conn, &plan_id, &auth_user.user_id) {
        Ok(plan) => {
            HttpResponse::Ok().json(plan)
        }
        Err(TravelPlanError::NotFound) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Travel plan not found".to_string(),
            })
        }
        Err(TravelPlanError::Unauthorized) => {
            HttpResponse::Forbidden().json(ErrorResponse {
                error: "You don't have permission to access this travel plan".to_string(),
            })
        }
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
        // The Err(_) pattern is unreachable since all variants are already covered
    }
}

// Create a new travel plan
pub async fn create_travel_plan(
    pool: web::Data<DbPool>,
    auth_user: web::ReqData<AuthenticatedUser>,
    plan_data: web::Json<NewTravelPlan>,
) -> impl Responder {
    let auth_user = auth_user.into_inner();
    info!("Creating new travel plan for user: {}", auth_user.username);
    
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    // Ensure the user_id in the plan matches the authenticated user
    let mut new_plan = plan_data.into_inner();
    new_plan.user_id = auth_user.user_id.clone();
    
    match TravelPlanService::create_travel_plan(&conn, &new_plan) {
        Ok(plan) => {
            HttpResponse::Created().json(plan)
        }
        Err(TravelPlanError::NotFound) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Resource not found".to_string(),
            })
        }
        Err(TravelPlanError::Unauthorized) => {
            HttpResponse::Forbidden().json(ErrorResponse {
                error: "You don't have permission to create this travel plan".to_string(),
            })
        }
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
    }
}

// Update a travel plan
pub async fn update_travel_plan(
    pool: web::Data<DbPool>,
    auth_user: web::ReqData<AuthenticatedUser>,
    path: web::Path<String>,
    update_data: web::Json<UpdateTravelPlan>,
) -> impl Responder {
    let plan_id = path.into_inner();
    let auth_user = auth_user.into_inner();
    info!("Updating travel plan with ID: {} for user: {}", plan_id, auth_user.username);
    
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match TravelPlanService::update_travel_plan(&conn, &plan_id, &update_data, &auth_user.user_id) {
        Ok(updated_plan) => {
            HttpResponse::Ok().json(updated_plan)
        }
        Err(TravelPlanError::NotFound) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Travel plan not found".to_string(),
            })
        }
        Err(TravelPlanError::Unauthorized) => {
            HttpResponse::Forbidden().json(ErrorResponse {
                error: "You don't have permission to update this travel plan".to_string(),
            })
        }
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
    }
}

// Delete a travel plan
pub async fn delete_travel_plan(
    pool: web::Data<DbPool>,
    auth_user: web::ReqData<AuthenticatedUser>,
    path: web::Path<String>,
) -> impl Responder {
    let plan_id = path.into_inner();
    let auth_user = auth_user.into_inner();
    info!("Deleting travel plan with ID: {} for user: {}", plan_id, auth_user.username);
    
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match TravelPlanService::delete_travel_plan(&conn, &plan_id, &auth_user.user_id) {
        Ok(()) => {
            HttpResponse::NoContent().finish()
        }
        Err(TravelPlanError::NotFound) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Travel plan not found".to_string(),
            })
        }
        Err(TravelPlanError::Unauthorized) => {
            HttpResponse::Forbidden().json(ErrorResponse {
                error: "You don't have permission to delete this travel plan".to_string(),
            })
        }
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
    }
}