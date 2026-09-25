use crate::gender::Gender;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;

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
pub struct UserProfileData {
    pub id: u64,
    pub name: String,
    pub gender: Gender,
    pub date_of_birth: NaiveDate,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
