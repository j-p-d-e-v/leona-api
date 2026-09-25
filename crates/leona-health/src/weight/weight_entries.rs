use std::sync::Arc;

use chrono::{DateTime, Utc};
use leona_core::Error;
use leona_db::Db;
use sqlx::{AssertSqlSafe, prelude::FromRow};

use crate::weight::WeightUnit;

#[derive(Debug, Clone)]
pub struct WeightEntries {
    db_client: Arc<Db>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WeightEntryData {
    id: Option<i64>,
    weight: f32,
    unit: WeightUnit,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

impl WeightEntries {
    pub async fn new(db_client: Arc<Db>) -> Result<Self, Error> {
        Self::migrate(db_client.clone()).await?;
        Ok(Self { db_client })
    }

    pub fn table() -> String {
        "leona_weight_entries".to_string()
    }

    /*
     * Migrate
     * Put your sql queries that modifies the weight entries schema.
     * Make sure that is in sequence.
     */
    async fn migrate(db_client: Arc<Db>) -> Result<(), Error> {
        let _ = Self::create_table(db_client.clone()).await?;
        Ok(())
    }

    async fn create_table(db_client: Arc<Db>) -> Result<(), Error> {
        {
            let mut db = db_client.get_connection().await?;
            let query_stmt = format!(
                r#"
                   CREATE TABLE IF NOT EXISTS {} (
                       id            INTEGER PRIMARY KEY AUTOINCREMENT,
                       weight        FLOAT NOT NULL,
                       unit          TEXT NOT NULL,
                       notes         TEXT NULL,
                       created_at    DATETIME NOT NULL,
                       updated_at    DATETIME NULL
                   );
               "#,
                Self::table()
            );
            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(Self::table())
                .execute(&mut *db)
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn list(&self) -> Result<Vec<WeightEntryData>, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("SELECT * FROM {}", Self::table());

            match sqlx::query_as::<_, WeightEntryData>(AssertSqlSafe(query_stmt))
                .fetch_all(&mut *connection)
                .await
            {
                Ok(result) => Ok(result),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn get(&self, id: i64) -> Result<WeightEntryData, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("SELECT * FROM {} WHERE id = $1", Self::table());

            match sqlx::query_as::<_, WeightEntryData>(AssertSqlSafe(query_stmt))
                .bind(id)
                .fetch_one(&mut *connection)
                .await
            {
                Ok(result) => Ok(result),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn create(&self, data: WeightEntryData) -> Result<(), Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "INSERT INTO {}(
                    weight,
                    unit,
                    notes
                    created_at
                ) VALUES(?,?,?,?)",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.weight)
                .bind(data.unit)
                .bind(data.notes)
                .bind(data.created_at)
                .execute(&mut *connection)
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn update(&self, data: WeightEntryData) -> Result<(), Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "UPDATE {} SET weight = ?, unit = ?, notes = ?, updated_at = ? WHERE id = ?",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.weight)
                .bind(data.unit)
                .bind(data.notes)
                .bind(data.updated_at)
                .bind(data.id)
                .execute(&mut *connection)
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn delete(&self, id: i64) -> Result<bool, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("DELETE FROM {} WHERE id = $1", Self::table());

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(id)
                .execute(&mut *connection)
                .await
            {
                Ok(_) => Ok(true),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }
}
