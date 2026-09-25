use sqlx::prelude::Type;

#[derive(Debug, Clone, PartialEq, Eq, Type)]
#[sqlx(rename_all = "lowercase")]
pub enum WeightUnit {
    Lb,
    Kg,
}
