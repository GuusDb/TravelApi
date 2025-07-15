use rusqlite::Connection;
use log::{error, info};

use crate::models::travel_plan::{NewTravelPlan, TravelPlan, UpdateTravelPlan};

pub struct TravelPlanService;

#[derive(Debug)]
pub enum TravelPlanError {
    NotFound,
    Unauthorized,
    DatabaseError(String),
}

impl TravelPlanService {
    pub fn get_travel_plans(conn: &Connection, user_id: &str) -> Result<Vec<TravelPlan>, TravelPlanError> {
        info!("Fetching travel plans for user: {}", user_id);
        
        match TravelPlan::find_by_user_id(conn, user_id) {
            Ok(plans) => {
                info!("Found {} travel plans for user {}", plans.len(), user_id);
                Ok(plans)
            }
            Err(e) => {
                error!("Error fetching travel plans: {}", e);
                Err(TravelPlanError::DatabaseError(e.to_string()))
            }
        }
    }
    
    pub fn get_travel_plan_by_id(conn: &Connection, plan_id: &str, user_id: &str) -> Result<TravelPlan, TravelPlanError> {
        info!("Fetching travel plan with ID: {} for user: {}", plan_id, user_id);
        
        match TravelPlan::find_by_id(conn, plan_id) {
            Ok(Some(plan)) => {
                if plan.user_id != user_id {
                    info!("User {} attempted to access travel plan {} belonging to user {}", 
                          user_id, plan_id, plan.user_id);
                    return Err(TravelPlanError::Unauthorized);
                }
                
                info!("Found travel plan: {}", plan.name);
                Ok(plan)
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
    
    pub fn create_travel_plan(conn: &Connection, plan_data: &NewTravelPlan, user_id: &str) -> Result<TravelPlan, TravelPlanError> {
        info!("Creating new travel plan for user: {}", user_id);
        
        match TravelPlan::create(conn, plan_data, user_id) {
            Ok(plan) => {
                info!("Created new travel plan: {}", plan.name);
                Ok(plan)
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
        user_id: &str
    ) -> Result<TravelPlan, TravelPlanError> {
        info!("Updating travel plan with ID: {} for user: {}", plan_id, user_id);
        
        // Find the travel plan
        let plan = Self::get_travel_plan_by_id(conn, plan_id, user_id)?;
        
        // Update the plan
        match plan.update(conn, update_data) {
            Ok(updated_plan) => {
                info!("Updated travel plan: {}", updated_plan.name);
                Ok(updated_plan)
            }
            Err(e) => {
                error!("Error updating travel plan: {}", e);
                Err(TravelPlanError::DatabaseError(e.to_string()))
            }
        }
    }
    
    pub fn delete_travel_plan(conn: &Connection, plan_id: &str, user_id: &str) -> Result<(), TravelPlanError> {
        info!("Deleting travel plan with ID: {} for user: {}", plan_id, user_id);
        
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