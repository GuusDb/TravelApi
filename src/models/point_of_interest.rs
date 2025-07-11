use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use log::info;
use rand::Rng;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PointOfInterest {
    pub id: String,
    pub route_option_id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub coordinates: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewPointOfInterest {
    pub route_option_id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub coordinates: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePointOfInterest {
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub category: Option<String>,
}

impl PointOfInterest {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(PointOfInterest {
            id: row.get(0)?,
            route_option_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            category: row.get(4)?,
            coordinates: row.get(5)?,
            created_at: row.get(6)?,
        })
    }

    pub fn create(conn: &Connection, new_poi: &NewPointOfInterest) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        conn.execute(
            "INSERT INTO points_of_interest (
                id, route_option_id, name, description, category, coordinates, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id, new_poi.route_option_id, new_poi.name, new_poi.description,
                new_poi.category, new_poi.coordinates, now
            ],
        )?;
        
        info!("Created new point of interest: {}", new_poi.name);
        
        Ok(PointOfInterest {
            id,
            route_option_id: new_poi.route_option_id.clone(),
            name: new_poi.name.clone(),
            description: new_poi.description.clone(),
            category: new_poi.category.clone(),
            coordinates: new_poi.coordinates.clone(),
            created_at: now,
        })
    }

    #[allow(dead_code)]
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, route_option_id, name, description, category, coordinates, created_at
             FROM points_of_interest
             WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_route_option_id(conn: &Connection, route_option_id: &str) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, route_option_id, name, description, category, coordinates, created_at
             FROM points_of_interest
             WHERE route_option_id = ?1"
        )?;
        
        let poi_iter = stmt.query_map(params![route_option_id], |row| Self::from_row(row))?;
        
        let mut pois = Vec::new();
        for poi_result in poi_iter {
            pois.push(poi_result?);
        }
        
        Ok(pois)
    }

    #[allow(dead_code)]
    pub fn update(&self, conn: &Connection, update: &UpdatePointOfInterest) -> Result<Self> {
        let mut updated_poi = self.clone();
        
        if let Some(name) = &update.name {
            updated_poi.name = name.clone();
        }
        
        if let Some(description) = &update.description {
            updated_poi.description = Some(description.clone());
        }
        
        if let Some(category) = &update.category {
            updated_poi.category = Some(category.clone());
        }
        
        conn.execute(
            "UPDATE points_of_interest SET
                name = ?1,
                description = ?2,
                category = ?3
             WHERE id = ?4",
            params![
                updated_poi.name,
                updated_poi.description,
                updated_poi.category,
                self.id
            ],
        )?;
        
        info!("Updated point of interest: {}", updated_poi.name);
        
        Ok(updated_poi)
    }

    #[allow(dead_code)]
    pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
        let rows_affected = conn.execute("DELETE FROM points_of_interest WHERE id = ?1", params![id])?;
        
        if rows_affected > 0 {
            info!("Deleted point of interest with ID: {}", id);
            Ok(true)
        } else {
            info!("No point of interest found with ID: {}", id);
            Ok(false)
        }
    }

    // Generate random points of interest for a route option
    pub fn generate_random_pois(conn: &Connection, route_option_id: &str, count: usize) -> Result<Vec<Self>> {
        let mut rng = rand::thread_rng();
        let mut pois = Vec::new();
        
        // Get the route option to use its waypoints
        let mut stmt = conn.prepare(
            "SELECT start_coordinates, end_coordinates, waypoints FROM route_options WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query(params![route_option_id])?;
        
        if let Some(row) = rows.next()? {
            let start_coords: String = row.get(0)?;
            let end_coords: String = row.get(1)?;
            let waypoints: Option<String> = row.get(2)?;
            
            // Categories for points of interest
            let categories = vec![
                "Restaurant", "Museum", "Park", "Hotel", "Landmark", 
                "Beach", "Mountain", "Lake", "Forest", "Historical Site"
            ];
            
            for i in 0..count {
                // Generate a random coordinate near the route
                let coords = if i == 0 {
                    // Near start
                    start_coords.clone()
                } else if i == count - 1 {
                    // Near end
                    end_coords.clone()
                } else if let Some(waypoints_str) = &waypoints {
                    // Near a waypoint if available
                    let waypoint_list: Vec<&str> = waypoints_str.split(';').collect();
                    if !waypoint_list.is_empty() {
                        let idx = rng.gen_range(0..waypoint_list.len());
                        waypoint_list[idx].to_string()
                    } else {
                        // Random coordinates
                        format!("{},{}", 
                            rng.gen_range(-90.0..90.0), 
                            rng.gen_range(-180.0..180.0)
                        )
                    }
                } else {
                    // Random coordinates
                    format!("{},{}", 
                        rng.gen_range(-90.0..90.0), 
                        rng.gen_range(-180.0..180.0)
                    )
                };
                
                // Generate a random name and category
                let category_idx = rng.gen_range(0..categories.len());
                let category = categories[category_idx];
                
                let poi_name = match category {
                    "Restaurant" => format!("The {} Restaurant", ["Delicious", "Tasty", "Gourmet", "Cozy", "Fancy"][rng.gen_range(0..5)]),
                    "Museum" => format!("{} Museum", ["Art", "History", "Science", "Natural", "Modern"][rng.gen_range(0..5)]),
                    "Park" => format!("{} Park", ["Central", "City", "National", "Memorial", "State"][rng.gen_range(0..5)]),
                    "Hotel" => format!("Hotel {}", ["Grand", "Royal", "Luxury", "Comfort", "Plaza"][rng.gen_range(0..5)]),
                    "Landmark" => format!("The {} Monument", ["Historic", "Ancient", "Famous", "Iconic", "Majestic"][rng.gen_range(0..5)]),
                    "Beach" => format!("{} Beach", ["Sandy", "Golden", "Paradise", "Sunset", "Crystal"][rng.gen_range(0..5)]),
                    "Mountain" => format!("Mount {}", ["Everest", "Fuji", "Kilimanjaro", "McKinley", "Blanc"][rng.gen_range(0..5)]),
                    "Lake" => format!("Lake {}", ["Superior", "Victoria", "Michigan", "Geneva", "Como"][rng.gen_range(0..5)]),
                    "Forest" => format!("{} Forest", ["Enchanted", "Dark", "Ancient", "Mystic", "Green"][rng.gen_range(0..5)]),
                    _ => format!("{} Site", ["Historical", "Cultural", "Heritage", "Ancient", "Traditional"][rng.gen_range(0..5)]),
                };
                
                let description = match category {
                    "Restaurant" => Some(format!("A {} restaurant with excellent food and service.", ["cozy", "fancy", "family-friendly", "romantic", "traditional"][rng.gen_range(0..5)])),
                    "Museum" => Some(format!("A museum showcasing {} exhibits.", ["historical", "artistic", "scientific", "cultural", "interactive"][rng.gen_range(0..5)])),
                    "Park" => Some(format!("A beautiful park with {} views.", ["scenic", "panoramic", "breathtaking", "relaxing", "peaceful"][rng.gen_range(0..5)])),
                    "Hotel" => Some(format!("A {} hotel with excellent amenities.", ["luxury", "boutique", "historic", "modern", "charming"][rng.gen_range(0..5)])),
                    "Landmark" => Some(format!("A famous landmark known for its {} architecture.", ["impressive", "unique", "historic", "stunning", "iconic"][rng.gen_range(0..5)])),
                    "Beach" => Some(format!("A {} beach with crystal clear waters.", ["sandy", "secluded", "popular", "pristine", "tropical"][rng.gen_range(0..5)])),
                    "Mountain" => Some(format!("A majestic mountain offering {} hiking trails.", ["challenging", "scenic", "popular", "diverse", "beautiful"][rng.gen_range(0..5)])),
                    "Lake" => Some(format!("A {} lake perfect for outdoor activities.", ["serene", "vast", "picturesque", "clear", "beautiful"][rng.gen_range(0..5)])),
                    "Forest" => Some(format!("A {} forest with diverse flora and fauna.", ["dense", "ancient", "magical", "lush", "protected"][rng.gen_range(0..5)])),
                    _ => Some(format!("A {} historical site with rich heritage.", ["fascinating", "well-preserved", "ancient", "significant", "mysterious"][rng.gen_range(0..5)])),
                };
                
                let new_poi = NewPointOfInterest {
                    route_option_id: route_option_id.to_string(),
                    name: poi_name,
                    description,
                    category: Some(category.to_string()),
                    coordinates: coords,
                };
                
                let poi = Self::create(conn, &new_poi)?;
                pois.push(poi);
            }
            
            info!("Generated {} random points of interest for route option ID: {}", count, route_option_id);
            Ok(pois)
        } else {
            info!("No route option found with ID: {}", route_option_id);
            Ok(Vec::new())
        }
    }
}