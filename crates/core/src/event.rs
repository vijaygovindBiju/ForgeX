use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ForgeEventPayload {
    TaskCreated { commitment_id: Uuid, title: String },
    TaskStarted { commitment_id: Uuid },
    TaskCompleted { commitment_id: Uuid },
    TaskMissed { commitment_id: Uuid, penalty_mins: u32, shielded: bool },
    ActivityLogged { activity_id: Uuid, application: String, duration_mins: u32 },
    PenaltyReduced { penalty_id: Uuid, reduced_by_mins: u32, remaining_mins: u32 },
    SaveDayEarned { save_day_id: Uuid },
    SaveDayConsumed { save_day_id: Uuid, commitment_id: Uuid },
    ScreenTimeGranted { minutes: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: ForgeEventPayload,
}

impl ForgeEvent {
    pub fn new(payload: ForgeEventPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            payload,
        }
    }
}
