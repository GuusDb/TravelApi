use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use log::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TravelPlan {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub start_location: String,
    pub end_location: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewTravelPlan {
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub start_location: String,
    pub end_location: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTravelPlan {
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl TravelPlan {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(TravelPlan {
            id: row.get(0)?,
            user_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            start_location: row.get(4)?,
            end_location: row.get(5)?,
            start_date: row.get(6)?,
            end_date: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }

    pub fn create(conn: &Connection, new_plan: &NewTravelPlan) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        conn.execute(
            "INSERT INTO travel_plans (
                id, user_id, name, description, start_location, end_location, 
                start_date, end_date, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id, new_plan.user_id, new_plan.name, new_plan.description, 
                new_plan.start_location, new_plan.end_location, 
                new_plan.start_date, new_plan.end_date, now, now
            ],
        )?;
        
        info!("Created new travel plan: {}", new_plan.name);
        
        Ok(TravelPlan {
            id,
            user_id: new_plan.user_id.clone(),
            name: new_plan.name.clone(),
            description: new_plan.description.clone(),
            start_location: new_plan.start_location.clone(),
            end_location: new_plan.end_location.clone(),
            start_date: new_plan.start_date,
            end_date: new_plan.end_date,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, description, start_location, end_location, 
                    start_date, end_date, created_at, updated_at 
             FROM travel_plans 
             WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_user_id(conn: &Connection, user_id: &str) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, description, start_location, end_location, 
                    start_date, end_date, created_at, updated_at 
             FROM travel_plans 
             WHERE user_id = ?1
             ORDER BY created_at DESC"
        )?;
        
        let plan_iter = stmt.query_map(params![user_id], |row| Self::from_row(row))?;
        
        let mut plans = Vec::new();
        for plan_result in plan_iter {
            plans.push(plan_result?);
        }
        
        Ok(plans)
    }

    #[allow(dead_code)]
    pub fn get_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, description, start_location, end_location, 
                    start_date, end_date, created_at, updated_at 
             FROM travel_plans
             ORDER BY created_at DESC"
        )?;
        
        let plan_iter = stmt.query_map([], |row| Self::from_row(row))?;
        
        let mut plans = Vec::new();
        for plan_result in plan_iter {
            plans.push(plan_result?);
        }
        
        Ok(plans)
    }

    pub fn update(&self, conn: &Connection, update: &UpdateTravelPlan) -> Result<Self> {
        let now = Utc::now();
        
        let mut updated_plan = self.clone();
        
        if let Some(name) = &update.name {
            updated_plan.name = name.clone();
        }
        
        if let Some(description) = &update.description {
            updated_plan.description = Some(description.clone());
        }
        
        if let Some(start_location) = &update.start_location {
            updated_plan.start_location = start_location.clone();
        }
        
        if let Some(end_location) = &update.end_location {
            updated_plan.end_location = end_location.clone();
        }
        
        if let Some(start_date) = update.start_date {
            updated_plan.start_date = Some(start_date);
        }
        
        if let Some(end_date) = update.end_date {
            updated_plan.end_date = Some(end_date);
        }
        
        updated_plan.updated_at = now;
        
        conn.execute(
            "UPDATE travel_plans SET 
                name = ?1, 
                description = ?2, 
                start_location = ?3, 
                end_location = ?4, 
                start_date = ?5, 
                end_date = ?6, 
                updated_at = ?7 
             WHERE id = ?8",
            params![
                updated_plan.name, 
                updated_plan.description, 
                updated_plan.start_location, 
                updated_plan.end_location, 
                updated_plan.start_date, 
                updated_plan.end_date, 
                now, 
                self.id
            ],
        )?;
        
        info!("Updated travel plan: {}", updated_plan.name);
        
        Ok(updated_plan)
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
        let rows_affected = conn.execute("DELETE FROM travel_plans WHERE id = ?1", params![id])?;
        
        if rows_affected > 0 {
            info!("Deleted travel plan with ID: {}", id);
            Ok(true)
        } else {
            info!("No travel plan found with ID: {}", id);
            Ok(false)
        }
    }
}