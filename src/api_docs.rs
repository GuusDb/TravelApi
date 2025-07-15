use utoipa::{OpenApi, Modify};
use crate::models::{
    user::{User, NewUser, LoginCredentials},
    travel_plan::{TravelPlan, NewTravelPlan, UpdateTravelPlan},
    route_option::{RouteOption, NewRouteOption, UpdateRouteOption},
    point_of_interest::{PointOfInterest, NewPointOfInterest, UpdatePointOfInterest}
};
use crate::middleware::auth::{AuthToken, Claims};
use crate::routes::route_option::ErrorResponse;
use crate::routes::route_option::GenerateOptionsQuery;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = &mut openapi.components {
            components.add_security_scheme(
                "Bearer",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    )
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        crate::routes::auth::register,
        crate::routes::auth::login,
        
        crate::routes::travel_plan::get_travel_plans,
        crate::routes::travel_plan::create_travel_plan,
        crate::routes::travel_plan::get_travel_plan_by_id,
        crate::routes::travel_plan::update_travel_plan,
        crate::routes::travel_plan::delete_travel_plan,
        
        crate::routes::route_option::get_route_options,
        crate::routes::route_option::generate_route_options,
        crate::routes::route_option::get_route_option_by_id
    ),
    components(
        schemas(
            User, NewUser, LoginCredentials, AuthToken, Claims,
            
            TravelPlan, NewTravelPlan, UpdateTravelPlan,
            
            RouteOption, NewRouteOption, UpdateRouteOption, GenerateOptionsQuery,
            
            PointOfInterest, NewPointOfInterest, UpdatePointOfInterest,
            
            ErrorResponse
        )
    ),
    security(
        ("Bearer" = [])
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