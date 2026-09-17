use std::sync::Arc;

use chrono::{DateTime, Utc};
use leona_core::Error;
use leona_db::Db;
use sqlx::{AssertSqlSafe, prelude::FromRow};

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    NotSpecified,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserProfile {
    pub name: String,
    pub gender: Gender,
    pub date_of_birth: DateTime<Utc>,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub db_client: Arc<Db>,
}

impl User {
    pub async fn new(db_client: Arc<Db>) -> Result<Self, Error> {
        Self::migrate(db_client.clone()).await?;
        Ok(Self { db_client })
    }

    pub fn table() -> String {
        "leona_user".to_string()
    }

    /*
     * Migrate
     * Put your sql queries that modifies the user schema.
     * Make sure that is in sequence.
     */
    async fn migrate(db_client: Arc<Db>) -> Result<(), Error> {
        let _ = Self::create_table(db_client.clone()).await?;
        Ok(())
    }

    async fn create_table(db_client: Arc<Db>) -> Result<(), Error> {
        let mut db = db_client.get_connection().await?;
        let query_stmt = format!(
            r#"
                   CREATE TABLE IF NOT EXISTS {} (
                       id            INTEGER PRIMARY KEY AUTOINCREMENT,
                       name          TEXT NOT NULL,
                       gender        TEXT,
                       date_of_birth DATETIME,
                       avatar        TEXT,
                       created_at    DATETIME NOT NULL,
                       updated_at    DATETIME NOT NULL
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

#[cfg(test)]
pub mod test_user {
    use super::*;

    #[tokio::test]
    async fn test_migrate() {
        let db = Db::new("/Users/jp-mac/Codes/leona-api/storage/test.db")
            .await
            .expect("expecting a db instance");
        let db_client = Arc::new(db);
        let user = User::new(db_client.clone())
            .await
            .expect("expecting a user instance");
    }
}
