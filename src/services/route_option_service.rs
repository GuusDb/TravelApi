use crate::models::point_of_interest::PointOfInterest;
use crate::models::route_option::RouteOption;
use crate::services::travel_plan_service::{TravelPlanError, TravelPlanService};
use log::{error, info};
use rusqlite::Connection;
use serde::Serialize;

pub struct RouteOptionService;

#[derive(Debug)]
pub enum RouteOptionError {
    TravelPlanError(TravelPlanError),
    RouteNotFound,
    InvalidRouteOption,
    DatabaseError(String),
}

impl From<TravelPlanError> for RouteOptionError {
    fn from(error: TravelPlanError) -> Self {
        RouteOptionError::TravelPlanError(error)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteOptionWithPois {
    pub route: RouteOption,
    pub points_of_interest: Vec<PointOfInterest>,
}

impl RouteOptionService {
    pub fn get_route_options(
        conn: &Connection,
        plan_id: &str,
        user_id: &str,
    ) -> Result<Vec<RouteOptionWithPois>, RouteOptionError> {
        info!(
            "Fetching route options for travel plan ID: {} for user: {}",
            plan_id, user_id
        );

        // First, check if the travel plan exists and belongs to the user
        let _ = TravelPlanService::get_travel_plan_by_id(conn, plan_id, user_id)?;

        // Get route options for the travel plan
        match RouteOption::find_by_travel_plan_id(conn, plan_id) {
            Ok(routes) => {
                // For each route option, get its points of interest
                let mut routes_with_pois = Vec::new();

                for route in routes {
                    match PointOfInterest::find_by_route_option_id(conn, &route.id) {
                        Ok(pois) => {
                            routes_with_pois.push(RouteOptionWithPois {
                                route,
                                points_of_interest: pois,
                            });
                        }
                        Err(e) => {
                            error!("Error fetching points of interest: {}", e);
                            return Err(RouteOptionError::DatabaseError(e.to_string()));
                        }
                    }
                }

                info!(
                    "Found {} route options for travel plan ID: {}",
                    routes_with_pois.len(),
                    plan_id
                );
                Ok(routes_with_pois)
            }
            Err(e) => {
                error!("Error fetching route options: {}", e);
                Err(RouteOptionError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn generate_route_options(
        conn: &Connection,
        plan_id: &str,
        user_id: &str,
        count: usize,
    ) -> Result<Vec<RouteOptionWithPois>, RouteOptionError> {
        info!(
            "Generating {} random route options for travel plan ID: {} for user: {}",
            count, plan_id, user_id
        );

        // First, check if the travel plan exists and belongs to the user
        let _ = TravelPlanService::get_travel_plan_by_id(conn, plan_id, user_id)?;

        // Generate random route options
        match RouteOption::generate_random_options(conn, plan_id, count) {
            Ok(routes) => {
                let mut routes_with_pois = Vec::new();

                // For each route option, generate random points of interest
                for route in routes {
                    // Generate 2-5 random points of interest for each route
                    let poi_count = 2 + (count % 4); // Between 2 and 5

                    match PointOfInterest::generate_random_pois(conn, &route.id, poi_count) {
                        Ok(pois) => {
                            routes_with_pois.push(RouteOptionWithPois {
                                route,
                                points_of_interest: pois,
                            });
                        }
                        Err(e) => {
                            error!("Error generating points of interest: {}", e);
                            return Err(RouteOptionError::DatabaseError(e.to_string()));
                        }
                    }
                }

                info!(
                    "Generated {} route options with points of interest for travel plan ID: {}",
                    routes_with_pois.len(),
                    plan_id
                );
                Ok(routes_with_pois)
            }
            Err(e) => {
                error!("Error generating route options: {}", e);
                Err(RouteOptionError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn get_route_option_by_id(
        conn: &Connection,
        plan_id: &str,
        route_id: &str,
        user_id: &str,
    ) -> Result<RouteOptionWithPois, RouteOptionError> {
        info!(
            "Fetching route option with ID: {} for travel plan ID: {} for user: {}",
            route_id, plan_id, user_id
        );

        // First, check if the travel plan exists and belongs to the user
        let _ = TravelPlanService::get_travel_plan_by_id(conn, plan_id, user_id)?;

        // Get the route option
        match RouteOption::find_by_id(conn, route_id) {
            Ok(Some(route)) => {
                // Check if the route option belongs to the travel plan
                if route.travel_plan_id != plan_id {
                    return Err(RouteOptionError::InvalidRouteOption);
                }

                // Get points of interest for the route option
                match PointOfInterest::find_by_route_option_id(conn, &route.id) {
                    Ok(pois) => {
                        info!(
                            "Found route option with ID: {} with {} points of interest",
                            route.id,
                            pois.len()
                        );

                        Ok(RouteOptionWithPois {
                            route,
                            points_of_interest: pois,
                        })
                    }
                    Err(e) => {
                        error!("Error fetching points of interest: {}", e);
                        Err(RouteOptionError::DatabaseError(e.to_string()))
                    }
                }
            }
            Ok(None) => {
                info!("Route option not found with ID: {}", route_id);
                Err(RouteOptionError::RouteNotFound)
            }
            Err(e) => {
                error!("Error fetching route option: {}", e);
                Err(RouteOptionError::DatabaseError(e.to_string()))
            }
        }
    }
}
