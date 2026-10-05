use std::sync::Arc;

use leona_core::Error;
use leona_db::Db;

use crate::weight::{TargetWeight, TargetWeightData, WeightEntries, WeightEntryData};

#[derive(Debug, Clone, Default)]
pub struct TargetWeightProgressData {
    pub weight: f32,
    pub completion: f32,
}

#[derive(Debug, Clone)]
pub struct WeightProgress {
    pub current_weight: f32,
    pub targets: Vec<TargetWeightProgressData>,
}

impl WeightProgress {
    pub fn calculate(user_id: &i64, db_client: Arc<Db>) -> Result<WeightProgress, Error> {
        let we = WeightEntries::new(db_client).await?;

        Ok(WeightProgress {
            current_weight: 0.0,
            targets: vec![],
        })
    }
}
