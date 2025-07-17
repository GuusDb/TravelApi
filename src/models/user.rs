use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Utc};
use log::{error, info};
use rusqlite::{Connection, Result, Row, params};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            email: row.get(3)?,
            created_at: row.get(4)?,
        })
    }

    pub fn create(conn: &Connection, new_user: &NewUser) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let password_hash = hash(&new_user.password, DEFAULT_COST)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        let now = Utc::now();

        conn.execute(
            "INSERT INTO users (id, username, password_hash, email, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, new_user.username, password_hash, new_user.email, now],
        )?;

        info!("Created new user: {}", new_user.username);

        Ok(User {
            id,
            username: new_user.username.clone(),
            password_hash,
            email: new_user.email.clone(),
            created_at: now,
        })
    }

    #[allow(dead_code)]
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, email, created_at FROM users WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_username(conn: &Connection, username: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, email, created_at FROM users WHERE username = ?1",
        )?;

        let mut rows = stmt.query(params![username])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub fn authenticate(conn: &Connection, credentials: &LoginCredentials) -> Result<Option<Self>> {
        if let Some(user) = Self::find_by_username(conn, &credentials.username)? {
            let password_matches = verify(&credentials.password, &user.password_hash)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            if password_matches {
                info!("User authenticated successfully: {}", credentials.username);
                return Ok(Some(user));
            }
        }

        error!(
            "Authentication failed for username: {}",
            credentials.username
        );
        Ok(None)
    }

    #[allow(dead_code)]
    pub fn get_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt =
            conn.prepare("SELECT id, username, password_hash, email, created_at FROM users")?;

        let user_iter = stmt.query_map([], |row| Self::from_row(row))?;

        let mut users = Vec::new();
        for user_result in user_iter {
            users.push(user_result?);
        }

        Ok(users)
    }

    #[allow(dead_code)]
    pub fn update(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "UPDATE users SET username = ?1, email = ?2 WHERE id = ?3",
            params![self.username, self.email, self.id],
        )?;

        info!("Updated user: {}", self.username);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
        let rows_affected = conn.execute("DELETE FROM users WHERE id = ?1", params![id])?;

        if rows_affected > 0 {
            info!("Deleted user with ID: {}", id);
            Ok(true)
        } else {
            info!("No user found with ID: {}", id);
            Ok(false)
        }
    }
}
