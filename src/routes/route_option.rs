use actix_web::{web, HttpResponse, Responder};
use log::info;
use serde::{Deserialize, Serialize};

use crate::db::connection::DbPool;
use crate::middleware::auth::AuthenticatedUser;
use crate::services::route_option_service::{RouteOptionService, RouteOptionError};
use crate::services::travel_plan_service::TravelPlanError;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateOptionsQuery {
    pub count: Option<usize>,
}

// Get route options for a travel plan
pub async fn get_route_options(
    pool: web::Data<DbPool>,
    auth_user: web::ReqData<AuthenticatedUser>,
    path: web::Path<String>,
) -> impl Responder {
    let plan_id = path.into_inner();
    let auth_user = auth_user.into_inner();
    info!("Fetching route options for travel plan ID: {} for user: {}", plan_id, auth_user.username);
    
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match RouteOptionService::get_route_options(&conn, &plan_id, &auth_user.user_id) {
        Ok(routes_with_pois) => {
            HttpResponse::Ok().json(routes_with_pois)
        }
        Err(RouteOptionError::TravelPlanError(TravelPlanError::NotFound)) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Travel plan not found".to_string(),
            })
        }
        Err(RouteOptionError::TravelPlanError(TravelPlanError::Unauthorized)) => {
            HttpResponse::Forbidden().json(ErrorResponse {
                error: "You don't have permission to access this travel plan".to_string(),
            })
        }
        Err(RouteOptionError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch route options".to_string(),
            })
        }
    }
}

// Generate random route options for a travel plan
pub async fn generate_route_options(
    pool: web::Data<DbPool>,
    auth_user: web::ReqData<AuthenticatedUser>,
    path: web::Path<String>,
    query: web::Query<GenerateOptionsQuery>,
) -> impl Responder {
    let plan_id = path.into_inner();
    let count = query.count.unwrap_or(3); // Default to 3 options if not specified
    let auth_user = auth_user.into_inner();
    
    info!("Generating {} random route options for travel plan ID: {} for user: {}", 
          count, plan_id, auth_user.username);
    
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match RouteOptionService::generate_route_options(&conn, &plan_id, &auth_user.user_id, count) {
        Ok(routes_with_pois) => {
            HttpResponse::Ok().json(routes_with_pois)
        }
        Err(RouteOptionError::TravelPlanError(TravelPlanError::NotFound)) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Travel plan not found".to_string(),
            })
        }
        Err(RouteOptionError::TravelPlanError(TravelPlanError::Unauthorized)) => {
            HttpResponse::Forbidden().json(ErrorResponse {
                error: "You don't have permission to access this travel plan".to_string(),
            })
        }
        Err(RouteOptionError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to generate route options".to_string(),
            })
        }
    }
}

// Get a specific route option by ID
pub async fn get_route_option_by_id(
    pool: web::Data<DbPool>,
    auth_user: web::ReqData<AuthenticatedUser>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (plan_id, route_id) = path.into_inner();
    let auth_user = auth_user.into_inner();
    info!("Fetching route option with ID: {} for travel plan ID: {} for user: {}", 
          route_id, plan_id, auth_user.username);
    
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };
    
    match RouteOptionService::get_route_option_by_id(&conn, &plan_id, &route_id, &auth_user.user_id) {
        Ok(route_with_pois) => {
            HttpResponse::Ok().json(route_with_pois)
        }
        Err(RouteOptionError::TravelPlanError(TravelPlanError::NotFound)) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Travel plan not found".to_string(),
            })
        }
        Err(RouteOptionError::TravelPlanError(TravelPlanError::Unauthorized)) => {
            HttpResponse::Forbidden().json(ErrorResponse {
                error: "You don't have permission to access this travel plan".to_string(),
            })
        }
        Err(RouteOptionError::RouteNotFound) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Route option not found".to_string(),
            })
        }
        Err(RouteOptionError::InvalidRouteOption) => {
            HttpResponse::BadRequest().json(ErrorResponse {
                error: "Route option does not belong to the specified travel plan".to_string(),
            })
        }
        Err(RouteOptionError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch route option".to_string(),
            })
        }
    }
}