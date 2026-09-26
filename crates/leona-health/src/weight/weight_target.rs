use std::{result, sync::Arc};

use chrono::{DateTime, Utc};
use leona_core::Error;
use leona_db::Db;
use sqlx::{AssertSqlSafe, prelude::FromRow};

use crate::weight::WeightUnit;

#[derive(Debug, Clone)]
pub struct TargetWeight {
    db_client: Arc<Db>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TargetWeightData {
    id: Option<i64>,
    user_id: i64,
    weight: f32,
    unit: WeightUnit,
    notes: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl TargetWeight {
    pub async fn new(db_client: Arc<Db>) -> Result<Self, Error> {
        Self::migrate(db_client.clone()).await?;
        Ok(Self { db_client })
    }

    pub fn table() -> String {
        "leona_target_weight".to_string()
    }

    /*
     * Migrate
     * Put your sql queries that modifies the target weight schema.
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
                       user_id       INTEGER NOT NULL,
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

    pub async fn get(&self, user_id: i64) -> Result<TargetWeightData, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("SELECT * FROM {} WHERE user_id = $1", Self::table());

            match sqlx::query_as::<_, TargetWeightData>(AssertSqlSafe(query_stmt))
                .bind(user_id)
                .fetch_one(&mut *connection)
                .await
            {
                Ok(result) => Ok(result),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn is_exists(&self, user_id: i64) -> Result<bool, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("SELECT 1 FROM {} WHERE user_id = $1", Self::table());

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(user_id)
                .fetch_optional(&mut *connection)
                .await
            {
                Ok(result) => Ok(result.is_some()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn save(&self, data: TargetWeightData) -> Result<TargetWeightData, Error> {
        let user_id = data.user_id.clone();
        let is_exists = self.is_exists(user_id).await?;
        if is_exists {
            let _ = self.create(data).await?;
        } else {
            let _ = self.update(data).await?;
        };
        Ok(self.get(user_id).await?)
    }

    pub async fn create(&self, data: TargetWeightData) -> Result<(), Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "INSERT INTO {}(
                    user_id,
                    weight,
                    unit,
                    notes,
                    created_at
                ) VALUES(?,?,?,?)",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.user_id)
                .bind(data.weight)
                .bind(data.unit)
                .bind(data.notes)
                .bind(Utc::now())
                .execute(&mut *connection)
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn update(&self, data: TargetWeightData) -> Result<bool, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "UPDATE {} SET weight = ?, unit = ?, notes = ?, updated_at = ? WHERE user_id = ?",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.weight)
                .bind(data.unit)
                .bind(data.notes)
                .bind(Utc::now())
                .bind(data.user_id)
                .execute(&mut *connection)
                .await
            {
                Ok(result) => Ok(result.rows_affected() > 0),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn delete(&self, user_id: i64) -> Result<bool, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("DELETE FROM {} WHERE user_id = $1", Self::table());

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(user_id)
                .execute(&mut *connection)
                .await
            {
                Ok(_) => Ok(true),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }
}
