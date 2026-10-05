use std::sync::Arc;

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
    pub id: Option<i64>,
    pub user_id: i64,
    pub target_weight: f32,
    pub unit: WeightUnit,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
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
                       target_weight        FLOAT NOT NULL,
                       unit          TEXT NOT NULL,
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

    pub async fn list(&self, user_id: i64) -> Result<Vec<TargetWeightData>, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "SELECT * FROM {} WHERE user_id = $1 ORDER BY created_at DESC",
                Self::table()
            );

            match sqlx::query_as::<_, TargetWeightData>(AssertSqlSafe(query_stmt))
                .bind(user_id)
                .fetch_all(&mut *connection)
                .await
            {
                Ok(result) => Ok(result),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn get(&self, id: i64) -> Result<TargetWeightData, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("SELECT * FROM {} WHERE id = $1", Self::table());

            match sqlx::query_as::<_, TargetWeightData>(AssertSqlSafe(query_stmt))
                .bind(id)
                .fetch_one(&mut *connection)
                .await
            {
                Ok(result) => Ok(result),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn create(&self, data: TargetWeightData) -> Result<i64, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "INSERT INTO {}(
                    user_id,
                    target_weight,
                    unit,
                    created_at
                ) VALUES(?,?,?,?)",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.user_id)
                .bind(data.target_weight)
                .bind(data.unit)
                .bind(Utc::now())
                .execute(&mut *connection)
                .await
            {
                Ok(result) => Ok(result.last_insert_rowid()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn update(&self, data: TargetWeightData) -> Result<bool, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "UPDATE {} SET target_weight = ?, unit = ?,  updated_at = ? WHERE id = ?",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.target_weight)
                .bind(data.unit)
                .bind(Utc::now())
                .bind(data.id)
                .execute(&mut *connection)
                .await
            {
                Ok(result) => Ok(result.rows_affected() > 0),
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
    async fn test_target_weight() {
        let db = Db::new("/Users/jp-mac/Codes/leona-api/storage/test.db")
            .await
            .expect("expecting a db instance");
        let db_client = Arc::new(db);
        let tw = TargetWeight::new(db_client)
            .await
            .expect("expecting a target weight entries instance");
        let create_tw = TargetWeightData {
            user_id: 1,
            target_weight: 143.0,
            unit: WeightUnit::Lb,
            created_at: Some(Utc::now()),
            updated_at: None,
            id: None,
        };
        let created_tw_entry_id = tw
            .create(create_tw)
            .await
            .expect("expecting the created data");

        let _tw_entry_data = tw
            .get(created_tw_entry_id)
            .await
            .expect("expecting a target weight entry data");
        println!("Created");

        let tw_entries_data = tw
            .list(1)
            .await
            .expect("expecting target weight entries data");
        assert!(!tw_entries_data.is_empty());
        println!("Listed");

        let update_we = TargetWeightData {
            user_id: 1,
            target_weight: 143.2,
            unit: WeightUnit::Lb,
            created_at: None,
            updated_at: Some(Utc::now()),
            id: Some(created_tw_entry_id.clone()),
        };

        let _updated_we_status = tw
            .update(update_we)
            .await
            .expect("expecting updated target weight entry data");

        let updated_tw_entry_data = tw
            .get(created_tw_entry_id.clone())
            .await
            .expect("expecting a target weight entry data");

        assert_eq!(updated_tw_entry_data.target_weight, 143.2);
        println!("Updated");
        let deleted_tw_entry_result = tw.delete(created_tw_entry_id.clone()).await;
        assert!(deleted_tw_entry_result.is_ok());

        let deleted_weight_entry_data = tw.get(created_tw_entry_id.clone()).await;
        assert!(deleted_weight_entry_data.is_err());
        println!("Deleted")
    }
}
