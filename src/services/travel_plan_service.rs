use log::{error, info};
use rusqlite::Connection;
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::route_option::RouteOption;
use crate::models::travel_plan::{NewTravelPlan, TravelPlan, UpdateTravelPlan};

pub struct TravelPlanService;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TravelPlanDto {
    #[serde(flatten)]
    pub travel_plan: TravelPlan,
    pub has_routes_generated: bool,
}

#[derive(Debug)]
pub enum TravelPlanError {
    NotFound,
    Unauthorized,
    DatabaseError(String),
}

impl TravelPlanService {
    pub fn get_travel_plans(
        conn: &Connection,
        user_id: &str,
    ) -> Result<Vec<TravelPlanDto>, TravelPlanError> {
        info!("Fetching travel plans for user: {}", user_id);

        match TravelPlan::find_by_user_id(conn, user_id) {
            Ok(plans) => {
                info!("Found {} travel plans for user {}", plans.len(), user_id);

                let mut plan_dtos = Vec::new();

                for plan in plans {
                    let has_routes = match RouteOption::find_by_travel_plan_id(conn, &plan.id) {
                        Ok(routes) => !routes.is_empty(),
                        Err(e) => {
                            error!("Error checking for route options: {}", e);
                            false
                        }
                    };

                    plan_dtos.push(TravelPlanDto {
                        travel_plan: plan,
                        has_routes_generated: has_routes,
                    });
                }

                Ok(plan_dtos)
            }
            Err(e) => {
                error!("Error fetching travel plans: {}", e);
                Err(TravelPlanError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn get_travel_plan_by_id(
        conn: &Connection,
        plan_id: &str,
        user_id: &str,
    ) -> Result<TravelPlanDto, TravelPlanError> {
        info!(
            "Fetching travel plan with ID: {} for user: {}",
            plan_id, user_id
        );

        match TravelPlan::find_by_id(conn, plan_id) {
            Ok(Some(plan)) => {
                if plan.user_id != user_id {
                    info!(
                        "User {} attempted to access travel plan {} belonging to user {}",
                        user_id, plan_id, plan.user_id
                    );
                    return Err(TravelPlanError::Unauthorized);
                }

                let has_routes = match RouteOption::find_by_travel_plan_id(conn, plan_id) {
                    Ok(routes) => !routes.is_empty(),
                    Err(e) => {
                        error!("Error checking for route options: {}", e);
                        false
                    }
                };

                info!(
                    "Found travel plan: {} (has routes: {})",
                    plan.name, has_routes
                );

                Ok(TravelPlanDto {
                    travel_plan: plan,
                    has_routes_generated: has_routes,
                })
            }
            Ok(None) => {
                info!("Travel plan not found with ID: {}", plan_id);
                Err(TravelPlanError::NotFound)
            }
            Err(e) => {
                error!("Error fetching travel plan: {}", e);
                Err(TravelPlanError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn create_travel_plan(
        conn: &Connection,
        plan_data: &NewTravelPlan,
        user_id: &str,
    ) -> Result<TravelPlanDto, TravelPlanError> {
        info!("Creating new travel plan for user: {}", user_id);

        match TravelPlan::create(conn, plan_data, user_id) {
            Ok(plan) => {
                info!("Created new travel plan: {}", plan.name);
                Ok(TravelPlanDto {
                    travel_plan: plan,
                    has_routes_generated: false,
                })
            }
            Err(e) => {
                error!("Error creating travel plan: {}", e);
                Err(TravelPlanError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn update_travel_plan(
        conn: &Connection,
        plan_id: &str,
        update_data: &UpdateTravelPlan,
        user_id: &str,
    ) -> Result<TravelPlanDto, TravelPlanError> {
        info!(
            "Updating travel plan with ID: {} for user: {}",
            plan_id, user_id
        );

        // Find the travel plan
        let plan_dto = Self::get_travel_plan_by_id(conn, plan_id, user_id)?;

        // Update the plan
        match plan_dto.travel_plan.update(conn, update_data) {
            Ok(updated_plan) => {
                info!("Updated travel plan: {}", updated_plan.name);

                // Return the updated plan with the has_routes_generated flag
                Ok(TravelPlanDto {
                    travel_plan: updated_plan,
                    has_routes_generated: plan_dto.has_routes_generated,
                })
            }
            Err(e) => {
                error!("Error updating travel plan: {}", e);
                Err(TravelPlanError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn delete_travel_plan(
        conn: &Connection,
        plan_id: &str,
        user_id: &str,
    ) -> Result<(), TravelPlanError> {
        info!(
            "Deleting travel plan with ID: {} for user: {}",
            plan_id, user_id
        );

        // Find the travel plan to ensure it exists and belongs to the user
        let _plan = Self::get_travel_plan_by_id(conn, plan_id, user_id)?;

        // Delete the plan
        match TravelPlan::delete(conn, plan_id) {
            Ok(true) => {
                info!("Deleted travel plan with ID: {}", plan_id);
                Ok(())
            }
            Ok(false) => {
                info!("Travel plan not found with ID: {}", plan_id);
                Err(TravelPlanError::NotFound)
            }
            Err(e) => {
                error!("Error deleting travel plan: {}", e);
                Err(TravelPlanError::DatabaseError(e.to_string()))
            }
        }
    }
}
