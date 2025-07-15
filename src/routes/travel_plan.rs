use actix_web::{HttpResponse, Responder, web};
use log::info;
use serde::Serialize;
use utoipa::ToSchema;

use crate::db::connection::DbPool;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::travel_plan::{NewTravelPlan, UpdateTravelPlan};
use crate::services::travel_plan_service::{TravelPlanError, TravelPlanService};

#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    get,
    path = "/api/travelplan",
    responses(
        (status = 200, description = "List of travel plans retrieved successfully"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "travel_plans"
)]
pub async fn get_travel_plans(
    pool: web::Data<DbPool>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    info!("Fetching travel plans for user: {}", auth_user.username);

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };

    match TravelPlanService::get_travel_plans(&conn, &auth_user.user_id) {
        Ok(plans) => HttpResponse::Ok().json(plans),
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch travel plans".to_string(),
        }),
    }
}

/// Get a specific travel plan by ID
///
/// Retrieves a specific travel plan by its ID.
#[utoipa::path(
    get,
    path = "/api/travelplan/{id}",
    params(
        ("id" = String, Path, description = "Travel plan ID")
    ),
    responses(
        (status = 200, description = "Travel plan retrieved successfully"),
        (status = 403, description = "Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Travel plan not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "travel_plans"
)]
pub async fn get_travel_plan_by_id(
    pool: web::Data<DbPool>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> impl Responder {
    let plan_id = path.into_inner();
    info!(
        "Fetching travel plan with ID: {} for user: {}",
        plan_id, auth_user.username
    );

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };

    match TravelPlanService::get_travel_plan_by_id(&conn, &plan_id, &auth_user.user_id) {
        Ok(plan) => HttpResponse::Ok().json(plan),
        Err(TravelPlanError::NotFound) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Travel plan not found".to_string(),
        }),
        Err(TravelPlanError::Unauthorized) => HttpResponse::Forbidden().json(ErrorResponse {
            error: "You don't have permission to access this travel plan".to_string(),
        }),
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        } // The Err(_) pattern is unreachable since all variants are already covered
    }
}

/// Create a new travel plan
///
/// Creates a new travel plan for the authenticated user.
#[utoipa::path(
    post,
    path = "/api/travelplan",
    request_body = NewTravelPlan,
    responses(
        (status = 201, description = "Travel plan created successfully"),
        (status = 403, description = "Unauthorized access", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "travel_plans"
)]
pub async fn create_travel_plan(
    pool: web::Data<DbPool>,
    auth_user: AuthenticatedUser,
    plan_data: web::Json<NewTravelPlan>,
) -> impl Responder {
    info!("Creating new travel plan for user: {}", auth_user.username);

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };

    let new_plan = plan_data.into_inner();
    
    let user_id = auth_user.user_id.clone();

    match TravelPlanService::create_travel_plan(&conn, &new_plan, &user_id) {
        Ok(plan) => HttpResponse::Created().json(plan),
        Err(TravelPlanError::NotFound) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Resource not found".to_string(),
        }),
        Err(TravelPlanError::Unauthorized) => HttpResponse::Forbidden().json(ErrorResponse {
            error: "You don't have permission to create this travel plan".to_string(),
        }),
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
    }
}

/// Update a travel plan
///
/// Updates an existing travel plan.
#[utoipa::path(
    put,
    path = "/api/travelplan/{id}",
    params(
        ("id" = String, Path, description = "Travel plan ID")
    ),
    request_body = UpdateTravelPlan,
    responses(
        (status = 200, description = "Travel plan updated successfully"),
        (status = 403, description = "Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Travel plan not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "travel_plans"
)]
pub async fn update_travel_plan(
    pool: web::Data<DbPool>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
    update_data: web::Json<UpdateTravelPlan>,
) -> impl Responder {
    let plan_id = path.into_inner();
    info!(
        "Updating travel plan with ID: {} for user: {}",
        plan_id, auth_user.username
    );

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };

    match TravelPlanService::update_travel_plan(&conn, &plan_id, &update_data, &auth_user.user_id) {
        Ok(updated_plan) => HttpResponse::Ok().json(updated_plan),
        Err(TravelPlanError::NotFound) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Travel plan not found".to_string(),
        }),
        Err(TravelPlanError::Unauthorized) => HttpResponse::Forbidden().json(ErrorResponse {
            error: "You don't have permission to update this travel plan".to_string(),
        }),
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
    }
}

/// Deletes an existing travel plan.
#[utoipa::path(
    delete,
    path = "/api/travelplan/{id}",
    params(
        ("id" = String, Path, description = "Travel plan ID")
    ),
    responses(
        (status = 204, description = "Travel plan deleted successfully"),
        (status = 403, description = "Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Travel plan not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "travel_plans"
)]
pub async fn delete_travel_plan(
    pool: web::Data<DbPool>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> impl Responder {
    let plan_id = path.into_inner();
    info!(
        "Deleting travel plan with ID: {} for user: {}",
        plan_id, auth_user.username
    );

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };

    match TravelPlanService::delete_travel_plan(&conn, &plan_id, &auth_user.user_id) {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(TravelPlanError::NotFound) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Travel plan not found".to_string(),
        }),
        Err(TravelPlanError::Unauthorized) => HttpResponse::Forbidden().json(ErrorResponse {
            error: "You don't have permission to delete this travel plan".to_string(),
        }),
        Err(TravelPlanError::DatabaseError(e)) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
    }
}
