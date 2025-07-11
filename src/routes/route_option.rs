use actix_web::{web, HttpResponse, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::connection::DbPool;
use crate::middleware::auth::AuthenticatedUser;
use crate::services::route_option_service::{RouteOptionService, RouteOptionError};
use crate::services::travel_plan_service::TravelPlanError;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GenerateOptionsQuery {
    #[schema(example = 3)]
    pub count: Option<usize>,
}

/// Get route options for a travel plan
///
/// Retrieves all route options for a specific travel plan.
#[utoipa::path(
    get,
    path = "/api/travelplan/{id}/routes",
    params(
        ("id" = String, Path, description = "Travel plan ID")
    ),
    responses(
        (status = 200, description = "List of route options retrieved successfully"),
        (status = 403, description = "Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Travel plan not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "route_options"
)]
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

/// Generate random route options for a travel plan
///
/// Creates random route options for a specific travel plan.
#[utoipa::path(
    post,
    path = "/api/travelplan/{id}/routes/generate",
    params(
        ("id" = String, Path, description = "Travel plan ID")
    ),
    request_body(content = GenerateOptionsQuery, description = "Number of route options to generate"),
    responses(
        (status = 200, description = "Route options generated successfully"),
        (status = 403, description = "Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Travel plan not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "route_options"
)]
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

/// Get a specific route option by ID
///
/// Retrieves a specific route option for a travel plan.
#[utoipa::path(
    get,
    path = "/api/travelplan/{plan_id}/routes/{route_id}",
    params(
        ("plan_id" = String, Path, description = "Travel plan ID"),
        ("route_id" = String, Path, description = "Route option ID")
    ),
    responses(
        (status = 200, description = "Route option retrieved successfully"),
        (status = 400, description = "Invalid route option", body = ErrorResponse),
        (status = 403, description = "Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Travel plan or route option not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "route_options"
)]
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