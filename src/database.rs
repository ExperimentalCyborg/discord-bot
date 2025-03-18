use std::fmt::Display;
use std::num::NonZeroU64;
use sqlx::{
    sqlite::{SqlitePool, SqlitePoolOptions},
    Error as SqlxError, Row,
};
use std::sync::Arc;
use log::{debug, info, warn, error};
use tokio::sync::Mutex;

/// Database connection pool wrapper for key-value storage
pub struct Database {
    pool: Arc<SqlitePool>,
}

impl Database {
    /// Initialize a new database connection and create necessary tables
    pub async fn new(db_path: &str) -> Result<Self, SqlxError> {
        // Create empty file if it doesn't exist
        let db_file_path = std::path::Path::new(db_path);
        if !db_file_path.exists() {
            std::fs::File::create(db_file_path)
                .map_err(|e| SqlxError::Configuration(format!("Failed to create database file: {}", e).into()))?;
            warn!("Created new database file at {}", db_path);
        }

        info!("Connecting to database at {}", db_path);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", db_path))
            .await?;

        // Create tables if they don't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS guild_kv (
                guild_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, key)
            )"
        )
            .execute(&pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS user_kv (
                user_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (user_id, key)
            )"
        )
            .execute(&pool)
            .await?;

        info!("Database initialized successfully");
        Ok(Self { pool: Arc::new(pool) })
    }

    /// Get the database connection pool
    pub fn pool(&self) -> Arc<SqlitePool> {
        self.pool.clone()
    }

    // Guild-specific key-value methods

    /// Set a key-value pair for a specific guild
    pub async fn set_guild_value<G, K, V>(&self, guild_id: &G, key: &K, value: &V) -> Result<(), SqlxError>
    where
        G: Display + Send + Sync + ?Sized,
        K: Display + Send + Sync + ?Sized,
        V: Display + Send + Sync + ?Sized,
    {

        // First, check if the key exists and get its current value
        let existing_value = self.get_guild_value(guild_id, key).await?;

        // Only update if the value has changed or the key didn't exist
        if existing_value.as_deref() != Some(value.to_string().as_str()) {
            debug!("Setting guild value: guild_id={}, key={}", guild_id, key);
            sqlx::query(
                "INSERT OR REPLACE INTO guild_kv (guild_id, key, value, updated_at)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
            )
                .bind(guild_id.to_string())
                .bind(key.to_string())
                .bind(value.to_string())
                .execute(&*self.pool)
                .await?;
        } else {
            // Key exists with same value - no update needed
            debug!("Value unchanged for guild_id={}, key={}, skipping update", guild_id, key);
        }

        Ok(())
    }

    /// Get a value for a specific guild and key
    pub async fn get_guild_value<G, K>(&self, guild_id: &G, key: &K) -> Result<Option<String>, SqlxError>
    where
        G: Display + Send + Sync + ?Sized,
        K: Display + Send + Sync + ?Sized,
    {
        debug!("Getting guild value: guild_id={}, key={}", guild_id, key);

        let result = sqlx::query(
            "SELECT value FROM guild_kv
             WHERE guild_id = ? AND key = ?"
        )
            .bind(guild_id.to_string())
            .bind(key.to_string())
            .fetch_optional(&*self.pool)
            .await?;

        Ok(result.map(|row| row.get("value")))
    }

    /// Delete a key-value pair for a specific guild
    pub async fn delete_guild_value<G, K>(&self, guild_id: &G, key: &K) -> Result<bool, SqlxError>
    where
        G: Display + Send + Sync + ?Sized,
        K: Display + Send + Sync + ?Sized,
    {
        debug!("Deleting guild value: guild_id={}, key={}", guild_id, key);

        let result = sqlx::query(
            "DELETE FROM guild_kv
             WHERE guild_id = ? AND key = ?"
        )
            .bind(guild_id.to_string())
            .bind(key.to_string())
            .execute(&*self.pool)
            .await?;

        // Return true if a row was deleted
        Ok(result.rows_affected() > 0)
    }

    /// Get all key-value pairs for a specific guild
    pub async fn get_all_guild_values<G>(&self, guild_id: &G) -> Result<Vec<(String, String)>, SqlxError>
    where
        G: Display + Send + Sync + ?Sized,
    {
        debug!("Getting all guild values: guild_id={}", guild_id);

        let rows = sqlx::query(
            "SELECT key, value FROM guild_kv
             WHERE guild_id = ?"
        )
            .bind(guild_id.to_string())
            .fetch_all(&*self.pool)
            .await?;

        let mut values = Vec::with_capacity(rows.len());
        for row in rows {
            values.push((row.get("key"), row.get("value")));
        }

        Ok(values)
    }

    // User-specific key-value methods

    /// Set a key-value pair for a specific user
    pub async fn set_user_value<U, K, V>(&self, user_id: &U, key: &K, value: &V) -> Result<(), SqlxError>
    where
        U: Display + Send + Sync + ?Sized,
        K: Display + Send + Sync + ?Sized,
        V: Display + Send + Sync + ?Sized,
    {
        // First, check if the key exists and get its current value
        let existing_value = self.get_user_value(user_id, key).await?;

        // Only update if the value has changed or the key didn't exist
        if existing_value.as_deref() != Some(value.to_string().as_str()) {
            debug!("Setting user value: guild_id={}, key={}", user_id, key);
            sqlx::query(
                "INSERT OR REPLACE INTO user_kv (guild_id, key, value, updated_at)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
            )
                .bind(user_id.to_string())
                .bind(key.to_string())
                .bind(value.to_string())
                .execute(&*self.pool)
                .await?;
        } else {
            // Key exists with same value - no update needed
            debug!("Value unchanged for user_id={}, key={}, skipping update", user_id, key);
        }

        Ok(())
    }

    /// Get a value for a specific user and key
    pub async fn get_user_value<U, K>(&self, user_id: &U, key: &K) -> Result<Option<String>, SqlxError>
    where
        U: Display + Send + Sync + ?Sized,
        K: Display + Send + Sync + ?Sized,
    {

        debug!("Getting user value: user_id={}, key={}", user_id, key);

        let result = sqlx::query(
            "SELECT value FROM user_kv
             WHERE user_id = ? AND key = ?"
        )
            .bind(user_id.to_string())
            .bind(key.to_string())
            .fetch_optional(&*self.pool)
            .await?;

        Ok(result.map(|row| row.get("value")))
    }

    /// Delete a key-value pair for a specific user
    pub async fn delete_user_value<U, K>(&self, user_id: &U, key: &K) -> Result<bool, SqlxError>
    where
        U: Display + Send + Sync + ?Sized,
        K: Display + Send + Sync + ?Sized,
    {
        debug!("Deleting user value: user_id={}, key={}", user_id, key);

        let result = sqlx::query(
            "DELETE FROM user_kv
             WHERE user_id = ? AND key = ?"
        )
            .bind(user_id.to_string())
            .bind(key.to_string())
            .execute(&*self.pool)
            .await?;

        // Return true if a row was deleted
        Ok(result.rows_affected() > 0)
    }

    /// Get all key-value pairs for a specific user
    pub async fn get_all_user_values<U>(&self, user_id: &U) -> Result<Vec<(String, String)>, SqlxError>
    where
        U: Display + Send + Sync + ?Sized,
    {
        debug!("Getting all user values: user_id={}", user_id);

        let rows = sqlx::query(
            "SELECT key, value FROM user_kv
             WHERE user_id = ?"
        )
            .bind(user_id.to_string())
            .fetch_all(&*self.pool)
            .await?;

        let mut values = Vec::with_capacity(rows.len());
        for row in rows {
            values.push((row.get("key"), row.get("value")));
        }

        Ok(values)
    }

    pub fn to_snowflake(&self, text: &str) -> Option<NonZeroU64> {
        text.parse::<NonZeroU64>().ok()
    }
}
