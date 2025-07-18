use actix_web::{HttpResponse, Responder, web};
use log::info;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::connection::DbPool;
use crate::middleware::auth::AuthenticatedUser;
use crate::services::route_option_service::{RouteOptionError, RouteOptionService};
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
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> impl Responder {
    let plan_id = path.into_inner();
    info!(
        "Fetching route options for travel plan ID: {} for user: {}",
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

    match RouteOptionService::get_route_options(&conn, &plan_id, &auth_user.user_id) {
        Ok(routes_with_pois) => HttpResponse::Ok().json(routes_with_pois),
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
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch route options".to_string(),
        }),
    }
}

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
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<GenerateOptionsQuery>,
) -> impl Responder {
    let plan_id = path.into_inner();
    let count = query.count.unwrap_or(3);

    info!(
        "Generating {} random route options for travel plan ID: {} for user: {}",
        count, plan_id, auth_user.username
    );

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };

    match RouteOptionService::generate_route_options(&conn, &plan_id, &auth_user.user_id, count) {
        Ok(routes_with_pois) => HttpResponse::Ok().json(routes_with_pois),
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
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to generate route options".to_string(),
        }),
    }
}

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
    auth_user: AuthenticatedUser,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (plan_id, route_id) = path.into_inner();
    info!(
        "Fetching route option with ID: {} for travel plan ID: {} for user: {}",
        route_id, plan_id, auth_user.username
    );

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };

    match RouteOptionService::get_route_option_by_id(&conn, &plan_id, &route_id, &auth_user.user_id)
    {
        Ok(route_with_pois) => HttpResponse::Ok().json(route_with_pois),
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
        Err(RouteOptionError::RouteNotFound) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Route option not found".to_string(),
        }),
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
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch route option".to_string(),
        }),
    }
}

#[utoipa::path(
    delete,
    path = "/api/travelplan/{plan_id}/routes/{route_id}",
    params(
        ("plan_id" = String, Path, description = "Travel plan ID"),
        ("route_id" = String, Path, description = "Route option ID")
    ),
    responses(
        (status = 200, description = "Route option deleted successfully"),
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
pub async fn delete_route_option(
    pool: web::Data<DbPool>,
    auth_user: AuthenticatedUser,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (plan_id, route_id) = path.into_inner();
    info!(
        "Deleting route option with ID: {} for travel plan ID: {} for user: {}",
        route_id, plan_id, auth_user.username
    );

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database connection error: {}", e),
            });
        }
    };

    match RouteOptionService::delete_route_option(&conn, &plan_id, &route_id, &auth_user.user_id) {
        Ok(deleted) => {
            if deleted {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("Route option with ID: {} deleted successfully", route_id)
                }))
            } else {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "Route option not found".to_string(),
                })
            }
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
        Err(RouteOptionError::RouteNotFound) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Route option not found".to_string(),
        }),
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
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to delete route option".to_string(),
        }),
    }
}

#[utoipa::path(
    delete,
    path = "/api/travelplan/{id}/routes",
    params(
        ("id" = String, Path, description = "Travel plan ID")
    ),
    responses(
        (status = 200, description = "All route options deleted successfully"),
        (status = 403, description = "Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Travel plan not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    ),
    tag = "route_options"
)]
pub async fn delete_all_route_options(
    pool: web::Data<DbPool>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> impl Responder {
    let plan_id = path.into_inner();
    info!(
        "Deleting all route options for travel plan ID: {} for user: {}",
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

    match RouteOptionService::delete_all_route_options(&conn, &plan_id, &auth_user.user_id) {
        Ok(count) => {
            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Deleted {} route options for travel plan ID: {}", count, plan_id)
            }))
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
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to delete route options".to_string(),
        }),
    }
}
