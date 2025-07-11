use crate::db::schema;
use log::{error, info};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::fmt;
use std::path::Path;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

// Define a custom error type that can handle both r2d2 and database errors
#[derive(Debug)]
pub enum DbError {
    PoolError(r2d2::Error),
    InitError(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DbError::PoolError(e) => write!(f, "Database pool error: {}", e),
            DbError::InitError(e) => write!(f, "Database initialization error: {}", e),
        }
    }
}

impl std::error::Error for DbError {}

impl From<r2d2::Error> for DbError {
    fn from(err: r2d2::Error) -> Self {
        DbError::PoolError(err)
    }
}

pub fn create_pool(db_path: &str) -> Result<DbPool, DbError> {
    info!("Creating database connection pool for: {}", db_path);

    let db_exists = Path::new(db_path).exists();

    // Create a connection manager
    let manager = SqliteConnectionManager::file(db_path);

    // Create the pool
    let pool = Pool::new(manager)?;

    // Initialize the database if it doesn't exist
    if !db_exists {
        info!("Database file does not exist. Creating new database.");
        let conn = pool.get()?;
        if let Err(e) = schema::initialize_database(&conn) {
            error!("Failed to initialize database: {}", e);
            return Err(DbError::InitError(e.to_string()));
        }
    }

    info!("Database connection pool created successfully");
    Ok(pool)
}

pub fn get_pool() -> Result<DbPool, DbError> {
    // In a real application, you might want to get the database path from an environment variable
    let db_path = "travel_api.db";
    create_pool(db_path)
}

#[cfg(test)]
pub fn get_test_pool() -> Result<DbPool, DbError> {
    // Use an in-memory database for tests
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::new(manager)?;

    // Initialize the database schema
    let conn = pool.get()?;
    if let Err(e) = schema::initialize_database(&conn) {
        error!("Failed to initialize test database: {}", e);
        return Err(DbError::InitError(e.to_string()));
    }

    Ok(pool)
}
