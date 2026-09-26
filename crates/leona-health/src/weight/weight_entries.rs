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
    user_id: i64,
    weight: f32,
    unit: WeightUnit,
    notes: Option<String>,
    created_at: Option<DateTime<Utc>>,
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

    pub async fn list(&self, user_id: i64) -> Result<Vec<WeightEntryData>, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "SELECT * FROM {} WHERE user_id = $1 ORDER BY created_at DESC",
                Self::table()
            );

            match sqlx::query_as::<_, WeightEntryData>(AssertSqlSafe(query_stmt))
                .bind(user_id)
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

    pub async fn create(&self, data: WeightEntryData) -> Result<i64, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "INSERT INTO {}(
                    user_id,
                    weight,
                    unit,
                    notes,
                    created_at
                ) VALUES(?,?,?,?,?)",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.user_id)
                .bind(data.weight)
                .bind(data.unit)
                .bind(data.notes)
                .bind(data.created_at)
                .execute(&mut *connection)
                .await
            {
                Ok(result) => Ok(result.last_insert_rowid()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn update(&self, data: WeightEntryData) -> Result<bool, Error> {
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
                Ok(result) => {
                    if result.rows_affected() == 0 {
                        return Err(Error::DbQueryErr("nothing is updated".to_string()));
                    }
                    Ok(true)
                }
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

#[cfg(test)]
pub mod case {
    use super::*;

    #[tokio::test]
    async fn test_weight_entry() {
        let db = Db::new("/home/jp-laptop/Codes/leona-api/storage/test.db")
            .await
            .expect("expecting a db instance");
        let db_client = Arc::new(db);
        let we = WeightEntries::new(db_client)
            .await
            .expect("expecting a weight entries instance");
        let create_we = WeightEntryData {
            user_id: 1,
            weight: 172.0,
            unit: WeightUnit::Lb,
            notes: Some("Lorem ipsum".to_string()),
            created_at: Some(Utc::now()),
            updated_at: None,
            id: None,
        };
        let created_weight_entry_id = we
            .create(create_we)
            .await
            .expect("expecting the created data");

        let created_weight_entry_data = we
            .get(created_weight_entry_id)
            .await
            .expect("expecting a weight entry data");
        println!("Created");

        let weight_entries_data = we.list(1).await.expect("expecting weight entries data");
        assert!(!weight_entries_data.is_empty());
        println!("Listed");

        let update_we = WeightEntryData {
            user_id: 1,
            weight: 173.0,
            unit: WeightUnit::Lb,
            notes: Some("Updated Lorem ipsum".to_string()),
            created_at: None,
            updated_at: Some(Utc::now()),
            id: created_weight_entry_data.id.clone(),
        };

        let _updated_we_status = we
            .update(update_we)
            .await
            .expect("expecting updated weight entry data");

        let updated_weight_entry_data = we
            .get(created_weight_entry_id.clone())
            .await
            .expect("expecting a weight entry data");

        assert_eq!(updated_weight_entry_data.weight, 173.0);
        println!("Updated");
        let deleted_we_result = we.delete(created_weight_entry_id.clone()).await;
        assert!(deleted_we_result.is_ok());

        let deleted_weight_entry_data = we.get(created_weight_entry_id.clone()).await;
        assert!(deleted_weight_entry_data.is_err());
        println!("Deleted")
    }
}
