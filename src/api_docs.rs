use utoipa::{OpenApi};
use crate::models::{
    user::{User, NewUser, LoginCredentials},
    travel_plan::{TravelPlan, NewTravelPlan, UpdateTravelPlan},
    route_option::{RouteOption, NewRouteOption, UpdateRouteOption},
    point_of_interest::{PointOfInterest, NewPointOfInterest, UpdatePointOfInterest}
};
use crate::middleware::auth::{AuthToken, Claims};
use crate::routes::route_option::ErrorResponse;
use crate::routes::route_option::GenerateOptionsQuery;

/// API documentation for the Travel API
#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth routes
        crate::routes::auth::register,
        crate::routes::auth::login,
        
        // Travel plan routes
        crate::routes::travel_plan::get_travel_plans,
        crate::routes::travel_plan::create_travel_plan,
        crate::routes::travel_plan::get_travel_plan_by_id,
        crate::routes::travel_plan::update_travel_plan,
        crate::routes::travel_plan::delete_travel_plan,
        
        // Route options routes
        crate::routes::route_option::get_route_options,
        crate::routes::route_option::generate_route_options,
        crate::routes::route_option::get_route_option_by_id
    ),
    components(
        schemas(
            // Auth models
            User, NewUser, LoginCredentials, AuthToken, Claims,
            
            // Travel plan models
            TravelPlan, NewTravelPlan, UpdateTravelPlan,
            
            // Route option models
            RouteOption, NewRouteOption, UpdateRouteOption, GenerateOptionsQuery,
            
            // Point of interest models
            PointOfInterest, NewPointOfInterest, UpdatePointOfInterest,
            
            // Error response
            ErrorResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "travel_plans", description = "Travel plan management endpoints"),
        (name = "route_options", description = "Route options management endpoints")
    ),
    info(
        title = "Travel API",
        version = "1.0.0",
        description = "API for managing travel plans, route options, and points of interest",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        ),
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    )
)]
pub struct ApiDoc;