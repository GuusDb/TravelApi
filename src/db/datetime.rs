use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use rusqlite::{types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef}, Result};
use std::ops::{Deref, DerefMut};

// Create a newtype wrapper for DateTime<Utc> to satisfy the orphan rule
// The orphan rule prevents implementing external traits for external types
#[derive(Debug, Clone, PartialEq)]
pub struct SqlDateTime(pub DateTime<Utc>);

impl Deref for SqlDateTime {
    type Target = DateTime<Utc>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SqlDateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<DateTime<Utc>> for SqlDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        SqlDateTime(dt)
    }
}

impl FromSql for SqlDateTime {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(text) => {
                let text_str = std::str::from_utf8(text)
                    .map_err(|_| FromSqlError::InvalidType)?;
                
                DateTime::parse_from_rfc3339(text_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|_| {
                        // Try parsing as a naive datetime if RFC3339 fails
                        NaiveDateTime::parse_from_str(text_str, "%Y-%m-%d %H:%M:%S")
                            .map(|ndt| Utc.from_utc_datetime(&ndt))
                    })
                    .map(SqlDateTime)
                    .map_err(|_| FromSqlError::InvalidType)
            },
            ValueRef::Integer(i) => {
                // Handle Unix timestamp (seconds since epoch)
                // Use DateTime::from_timestamp instead of the deprecated from_timestamp_opt
                let dt = DateTime::from_timestamp(i, 0)
                    .ok_or(FromSqlError::InvalidType)?;
                Ok(SqlDateTime(dt))
            },
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for SqlDateTime {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        let formatted = self.0.to_rfc3339();
        Ok(ToSqlOutput::from(formatted))
    }
}

// Create a newtype wrapper for Option<SqlDateTime> to satisfy the orphan rule
#[derive(Debug, Clone, PartialEq)]
pub struct NullableSqlDateTime(pub Option<SqlDateTime>);

impl Deref for NullableSqlDateTime {
    type Target = Option<SqlDateTime>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NullableSqlDateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Option<SqlDateTime>> for NullableSqlDateTime {
    fn from(opt: Option<SqlDateTime>) -> Self {
        NullableSqlDateTime(opt)
    }
}

impl From<Option<DateTime<Utc>>> for NullableSqlDateTime {
    fn from(opt: Option<DateTime<Utc>>) -> Self {
        NullableSqlDateTime(opt.map(SqlDateTime))
    }
}

impl FromSql for NullableSqlDateTime {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Null => Ok(NullableSqlDateTime(None)),
            _ => SqlDateTime::column_result(value).map(|dt| NullableSqlDateTime(Some(dt))),
        }
    }
}