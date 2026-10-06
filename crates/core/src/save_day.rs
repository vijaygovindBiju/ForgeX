use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveDayStatus {
    Available,
    Consumed,
}

impl std::fmt::Display for SaveDayStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveDayStatus::Available => write!(f, "AVAILABLE"),
            SaveDayStatus::Consumed => write!(f, "CONSUMED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDay {
    pub id: Uuid,
    pub earned_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub consumed_for_commitment_id: Option<Uuid>,
    pub status: SaveDayStatus,
}

impl SaveDay {
    pub fn new_earned(now: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            earned_at: now,
            consumed_at: None,
            consumed_for_commitment_id: None,
            status: SaveDayStatus::Available,
        }
    }

    pub fn consume(&mut self, commitment_id: Uuid, now: DateTime<Utc>) {
        self.consumed_at = Some(now);
        self.consumed_for_commitment_id = Some(commitment_id);
        self.status = SaveDayStatus::Consumed;
    }
}
