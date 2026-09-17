use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use leona_core::Error;
use leona_db::Db;
use sqlx::{AssertSqlSafe, prelude::FromRow};

#[derive(Debug, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    NotSpecified,
}

#[derive(Debug, Clone, FromRow)]
pub struct CreateUserProfile {
    pub name: String,
    pub gender: Gender,
    pub date_of_birth: NaiveDate,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UpdateUserProfile {
    pub name: String,
    pub gender: Gender,
    pub date_of_birth: NaiveDate,
    pub avatar: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct UserProfile {
    pub id: u64,
    pub name: String,
    pub gender: Gender,
    pub date_of_birth: NaiveDate,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
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
        {
            let mut db = db_client.get_connection().await?;
            let query_stmt = format!(
                r#"
                   CREATE TABLE IF NOT EXISTS {} (
                       id            INTEGER PRIMARY KEY AUTOINCREMENT,
                       name          TEXT NOT NULL,
                       gender        TEXT,
                       date_of_birth DATE,
                       avatar        TEXT NULL,
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

    pub async fn is_exists(&self) -> Result<bool, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("SELECT COUNT(1) FROM {} WHERE id = 1", Self::table());

            match sqlx::query(AssertSqlSafe(query_stmt))
                .fetch_optional(&mut *connection)
                .await
            {
                Ok(result) => Ok(result.is_some()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn get(&self) -> Result<UserProfile, Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!("SELECT * FROM {} WHERE id = 1", Self::table());

            match sqlx::query_as::<_, UserProfile>(AssertSqlSafe(query_stmt))
                .fetch_one(&mut *connection)
                .await
            {
                Ok(result) => Ok(result),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }

    pub async fn create(&self, data: CreateUserProfile) -> Result<(), Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "INSERT INTO {}(id, name, gender, avatar, date_of_birth, created_at) VALUES(1,?,?,?,?,?)",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.name)
                .bind(data.gender)
                .bind(data.avatar)
                .bind(data.date_of_birth.to_string())
                .bind(data.created_at)
                .execute(&mut *connection)
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
        }
    }
    pub async fn update(&self, data: UpdateUserProfile) -> Result<(), Error> {
        {
            let mut connection = self.db_client.get_connection().await?;

            let query_stmt = format!(
                "UPDATE {} SET name = ?, gender = ?, avatar = ?, date_of_birth = ?, updated_at = ? WHERE id = 1",
                Self::table()
            );

            match sqlx::query(AssertSqlSafe(query_stmt))
                .bind(data.name)
                .bind(data.gender)
                .bind(data.avatar)
                .bind(data.date_of_birth.to_string())
                .bind(data.updated_at)
                .execute(&mut *connection)
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => Err(Error::DbQueryErr(error.to_string())),
            }
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

        let is_exists = user
            .is_exists()
            .await
            .expect("unable to check user if exists");

        if !is_exists {
            user.create(CreateUserProfile {
                name: "Juan dela Cruz".to_string(),
                gender: Gender::Male,
                date_of_birth: NaiveDate::default(),
                avatar: None,
                created_at: Utc::now(),
            })
            .await
            .expect("unable to create user");
        }

        let user_profile = user.get().await.expect("unable to get user profile data");
        assert_eq!(user_profile.id, 1);
        assert!(user_profile.updated_at.is_none());

        let _ = user
            .update(UpdateUserProfile {
                name: "Juan dela Cruz".to_string(),
                gender: Gender::Male,
                date_of_birth: NaiveDate::default(),
                avatar: None,
                updated_at: Utc::now(),
            })
            .await
            .expect("unable to update user profile");

        let user_profile = user.get().await.expect("unable to get user profile data");
        assert_eq!(user_profile.id, 1);
        assert!(user_profile.updated_at.is_some());
    }
}
