#[derive(Debug, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    NotSpecified,
}
