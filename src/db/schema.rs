use rusqlite::{Connection, Result};
use log::info;

pub fn initialize_database(conn: &Connection) -> Result<()> {
    info!("Initializing database schema...");
    
    // Create users table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create travel_plans table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS travel_plans (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            start_location TEXT NOT NULL,
            end_location TEXT NOT NULL,
            start_date TIMESTAMP,
            end_date TIMESTAMP,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )",
        [],
    )?;

    // Create route_options table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS route_options (
            id TEXT PRIMARY KEY,
            travel_plan_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            distance REAL,
            duration INTEGER,
            start_coordinates TEXT NOT NULL,
            end_coordinates TEXT NOT NULL,
            waypoints TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (travel_plan_id) REFERENCES travel_plans (id)
        )",
        [],
    )?;

    // Create points_of_interest table for storing attractions along routes
    conn.execute(
        "CREATE TABLE IF NOT EXISTS points_of_interest (
            id TEXT PRIMARY KEY,
            route_option_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            category TEXT,
            coordinates TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (route_option_id) REFERENCES route_options (id)
        )",
        [],
    )?;

    info!("Database schema initialized successfully");
    Ok(())
}