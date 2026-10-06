use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PenaltyStatus {
    Active,
    Cleared,
    Expired,
}

impl std::fmt::Display for PenaltyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PenaltyStatus::Active => write!(f, "ACTIVE"),
            PenaltyStatus::Cleared => write!(f, "CLEARED"),
            PenaltyStatus::Expired => write!(f, "EXPIRED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyRecord {
    pub id: Uuid,
    pub commitment_id: Uuid,
    pub raw_penalty_mins: u32,
    pub actual_penalty_mins: u32,
    pub remaining_penalty_mins: u32,
    pub restricted_days: u32,
    pub avoidance_score: u32,
    pub shielded_by_save_day: bool,
    pub status: PenaltyStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PenaltyRecord {
    pub fn new(
        commitment_id: Uuid,
        raw_penalty_mins: u32,
        actual_penalty_mins: u32,
        restricted_days: u32,
        avoidance_score: u32,
        shielded_by_save_day: bool,
    ) -> Self {
        let now = Utc::now();
        let remaining = if shielded_by_save_day { 0 } else { actual_penalty_mins };
        let status = if shielded_by_save_day {
            PenaltyStatus::Cleared
        } else {
            PenaltyStatus::Active
        };

        Self {
            id: Uuid::new_v4(),
            commitment_id,
            raw_penalty_mins,
            actual_penalty_mins,
            remaining_penalty_mins: remaining,
            restricted_days: if shielded_by_save_day { 0 } else { restricted_days },
            avoidance_score,
            shielded_by_save_day,
            status,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn reduce(&mut self, minutes: u32, now: DateTime<Utc>) {
        if self.status != PenaltyStatus::Active {
            return;
        }

        if minutes >= self.remaining_penalty_mins {
            self.remaining_penalty_mins = 0;
            if self.restricted_days > 0 {
                self.restricted_days = self.restricted_days.saturating_sub(1);
            }
            if self.restricted_days == 0 {
                self.status = PenaltyStatus::Cleared;
            }
        } else {
            self.remaining_penalty_mins -= minutes;
        }
        self.updated_at = now;
    }
}
