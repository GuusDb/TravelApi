use chrono::{DateTime, Utc};
use log::info;
use rand::Rng;
use rusqlite::{Connection, Result, Row, params};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteOption {
    pub id: String,
    pub travel_plan_id: String,
    pub name: String,
    pub description: Option<String>,
    pub distance: Option<f64>,
    pub duration: Option<i64>,
    pub start_coordinates: String,
    pub end_coordinates: String,
    pub waypoints: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewRouteOption {
    pub travel_plan_id: String,
    pub name: String,
    pub description: Option<String>,
    pub distance: Option<f64>,
    pub duration: Option<i64>,
    pub start_coordinates: String,
    pub end_coordinates: String,
    pub waypoints: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRouteOption {
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub distance: Option<f64>,
    #[allow(dead_code)]
    pub duration: Option<i64>,
    #[allow(dead_code)]
    pub waypoints: Option<String>,
}

impl RouteOption {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(RouteOption {
            id: row.get(0)?,
            travel_plan_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            distance: row.get(4)?,
            duration: row.get(5)?,
            start_coordinates: row.get(6)?,
            end_coordinates: row.get(7)?,
            waypoints: row.get(8)?,
            created_at: row.get(9)?,
        })
    }

    pub fn create(conn: &Connection, new_route: &NewRouteOption) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        conn.execute(
            "INSERT INTO route_options (
                id, travel_plan_id, name, description, distance, duration,
                start_coordinates, end_coordinates, waypoints, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                new_route.travel_plan_id,
                new_route.name,
                new_route.description,
                new_route.distance,
                new_route.duration,
                new_route.start_coordinates,
                new_route.end_coordinates,
                new_route.waypoints,
                now
            ],
        )?;

        info!("Created new route option: {}", new_route.name);

        Ok(RouteOption {
            id,
            travel_plan_id: new_route.travel_plan_id.clone(),
            name: new_route.name.clone(),
            description: new_route.description.clone(),
            distance: new_route.distance,
            duration: new_route.duration,
            start_coordinates: new_route.start_coordinates.clone(),
            end_coordinates: new_route.end_coordinates.clone(),
            waypoints: new_route.waypoints.clone(),
            created_at: now,
        })
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, travel_plan_id, name, description, distance, duration,
                    start_coordinates, end_coordinates, waypoints, created_at
             FROM route_options
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_travel_plan_id(conn: &Connection, travel_plan_id: &str) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, travel_plan_id, name, description, distance, duration,
                    start_coordinates, end_coordinates, waypoints, created_at
             FROM route_options
             WHERE travel_plan_id = ?1",
        )?;

        let route_iter = stmt.query_map(params![travel_plan_id], |row| Self::from_row(row))?;

        let mut routes = Vec::new();
        for route_result in route_iter {
            routes.push(route_result?);
        }

        Ok(routes)
    }

    #[allow(dead_code)]
    pub fn update(&self, conn: &Connection, update: &UpdateRouteOption) -> Result<Self> {
        let mut updated_route = self.clone();

        if let Some(name) = &update.name {
            updated_route.name = name.clone();
        }

        if let Some(description) = &update.description {
            updated_route.description = Some(description.clone());
        }

        if let Some(distance) = update.distance {
            updated_route.distance = Some(distance);
        }

        if let Some(duration) = update.duration {
            updated_route.duration = Some(duration);
        }

        if let Some(waypoints) = &update.waypoints {
            updated_route.waypoints = Some(waypoints.clone());
        }

        conn.execute(
            "UPDATE route_options SET
                name = ?1,
                description = ?2,
                distance = ?3,
                duration = ?4,
                waypoints = ?5
             WHERE id = ?6",
            params![
                updated_route.name,
                updated_route.description,
                updated_route.distance,
                updated_route.duration,
                updated_route.waypoints,
                self.id
            ],
        )?;

        info!("Updated route option: {}", updated_route.name);

        Ok(updated_route)
    }

    #[allow(dead_code)]
    pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
        let rows_affected = conn.execute("DELETE FROM route_options WHERE id = ?1", params![id])?;

        if rows_affected > 0 {
            info!("Deleted route option with ID: {}", id);
            Ok(true)
        } else {
            info!("No route option found with ID: {}", id);
            Ok(false)
        }
    }

    // Generate random route options for a travel plan
    pub fn generate_random_options(
        conn: &Connection,
        travel_plan_id: &str,
        count: usize,
    ) -> Result<Vec<Self>> {
        let mut rng = rand::thread_rng();
        let mut routes = Vec::new();

        // Get the travel plan to use its start and end locations
        let mut stmt =
            conn.prepare("SELECT start_location, end_location FROM travel_plans WHERE id = ?1")?;

        let mut rows = stmt.query(params![travel_plan_id])?;

        if let Some(row) = rows.next()? {
            let start_location: String = row.get(0)?;
            let end_location: String = row.get(1)?;

            // Generate random start and end coordinates based on the locations
            let start_coords = format!(
                "{},{}",
                rng.gen_range(-90.0..90.0),
                rng.gen_range(-180.0..180.0)
            );

            let end_coords = format!(
                "{},{}",
                rng.gen_range(-90.0..90.0),
                rng.gen_range(-180.0..180.0)
            );

            for i in 0..count {
                // Generate random route options
                let route_name = format!("Route Option {}", i + 1);
                let description = match i % 3 {
                    0 => Some(format!(
                        "Scenic route from {} to {}",
                        start_location, end_location
                    )),
                    1 => Some(format!(
                        "Fastest route from {} to {}",
                        start_location, end_location
                    )),
                    _ => Some(format!(
                        "Alternative route from {} to {}",
                        start_location, end_location
                    )),
                };

                // Random distance between 10 and 1000 km
                let distance = Some(rng.gen_range(10.0..1000.0));

                // Random duration between 30 minutes and 12 hours (in minutes)
                let duration = Some(rng.gen_range(30..720));

                // Generate random waypoints
                let waypoint_count = rng.gen_range(1..5);
                let mut waypoints = Vec::new();

                for _ in 0..waypoint_count {
                    waypoints.push(format!(
                        "{},{}",
                        rng.gen_range(-90.0..90.0),
                        rng.gen_range(-180.0..180.0)
                    ));
                }

                let waypoints_str = if waypoints.is_empty() {
                    None
                } else {
                    Some(waypoints.join(";"))
                };

                let new_route = NewRouteOption {
                    travel_plan_id: travel_plan_id.to_string(),
                    name: route_name,
                    description,
                    distance,
                    duration,
                    start_coordinates: start_coords.clone(),
                    end_coordinates: end_coords.clone(),
                    waypoints: waypoints_str,
                };

                let route = Self::create(conn, &new_route)?;
                routes.push(route);
            }

            info!(
                "Generated {} random route options for travel plan ID: {}",
                count, travel_plan_id
            );
            Ok(routes)
        } else {
            info!("No travel plan found with ID: {}", travel_plan_id);
            Ok(Vec::new())
        }
    }
}
