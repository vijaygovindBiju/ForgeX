use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{ForgeError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitmentStatus {
    Planned,
    Active,
    Completed,
    Missed,
    Excused,
}

impl std::fmt::Display for CommitmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitmentStatus::Planned => write!(f, "PLANNED"),
            CommitmentStatus::Active => write!(f, "ACTIVE"),
            CommitmentStatus::Completed => write!(f, "COMPLETED"),
            CommitmentStatus::Missed => write!(f, "MISSED"),
            CommitmentStatus::Excused => write!(f, "EXCUSED"),
        }
    }
}

impl std::str::FromStr for CommitmentStatus {
    type Err = ForgeError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "planned" => Ok(CommitmentStatus::Planned),
            "active" => Ok(CommitmentStatus::Active),
            "completed" => Ok(CommitmentStatus::Completed),
            "missed" => Ok(CommitmentStatus::Missed),
            "excused" => Ok(CommitmentStatus::Excused),
            _ => Err(ForgeError::Validation(format!("Invalid commitment status: {}", s))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecurrenceRule {
    Once,
    Daily,
    Weekdays,
    TimesPerWeek(u8),
    Custom(String),
}

impl std::fmt::Display for RecurrenceRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecurrenceRule::Once => write!(f, "Once"),
            RecurrenceRule::Daily => write!(f, "Daily"),
            RecurrenceRule::Weekdays => write!(f, "Weekdays"),
            RecurrenceRule::TimesPerWeek(n) => write!(f, "{}x/week", n),
            RecurrenceRule::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub importance: u8,
    pub expected_effort: u8,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_duration_mins: u32,
    pub recurrence: RecurrenceRule,
    pub category: String,
    pub status: CommitmentStatus,
    pub actual_started_at: Option<DateTime<Utc>>,
    pub actual_ended_at: Option<DateTime<Utc>>,
    pub consecutive_skips: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Commitment {
    pub fn new(
        title: impl Into<String>,
        importance: u8,
        expected_effort: u8,
        scheduled_start: DateTime<Utc>,
        scheduled_duration_mins: u32,
        recurrence: RecurrenceRule,
        category: impl Into<String>,
    ) -> Result<Self> {
        let title = title.into().trim().to_string();
        if title.is_empty() {
            return Err(ForgeError::Validation("Title cannot be empty".to_string()));
        }

        if !(1..=5).contains(&importance) {
            return Err(ForgeError::Validation("Importance must be between 1 and 5".to_string()));
        }

        if !(1..=5).contains(&expected_effort) {
            return Err(ForgeError::Validation("Expected effort must be between 1 and 5".to_string()));
        }

        if scheduled_duration_mins == 0 {
            return Err(ForgeError::Validation("Duration must be greater than 0 minutes".to_string()));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            importance,
            expected_effort,
            scheduled_start,
            scheduled_duration_mins,
            recurrence,
            category: category.into().trim().to_string(),
            status: CommitmentStatus::Planned,
            actual_started_at: None,
            actual_ended_at: None,
            consecutive_skips: 0,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn start(&mut self, now: DateTime<Utc>) -> Result<()> {
        match self.status {
            CommitmentStatus::Planned => {
                self.status = CommitmentStatus::Active;
                self.actual_started_at = Some(now);
                self.updated_at = now;
                Ok(())
            }
            _ => Err(ForgeError::InvalidTransition(format!(
                "Cannot start commitment from status {}",
                self.status
            ))),
        }
    }

    pub fn complete(&mut self, now: DateTime<Utc>) -> Result<()> {
        match self.status {
            CommitmentStatus::Planned | CommitmentStatus::Active => {
                self.status = CommitmentStatus::Completed;
                self.actual_ended_at = Some(now);
                self.consecutive_skips = 0; // reset consecutive skip streak on success
                self.updated_at = now;
                Ok(())
            }
            _ => Err(ForgeError::InvalidTransition(format!(
                "Cannot complete commitment from status {}",
                self.status
            ))),
        }
    }

    pub fn miss(&mut self, now: DateTime<Utc>) -> Result<()> {
        match self.status {
            CommitmentStatus::Planned | CommitmentStatus::Active => {
                self.status = CommitmentStatus::Missed;
                self.actual_ended_at = Some(now);
                self.consecutive_skips += 1;
                self.updated_at = now;
                Ok(())
            }
            _ => Err(ForgeError::InvalidTransition(format!(
                "Cannot mark commitment missed from status {}",
                self.status
            ))),
        }
    }

    pub fn excuse(&mut self, now: DateTime<Utc>) -> Result<()> {
        match self.status {
            CommitmentStatus::Planned | CommitmentStatus::Active => {
                self.status = CommitmentStatus::Excused;
                self.actual_ended_at = Some(now);
                self.updated_at = now;
                Ok(())
            }
            _ => Err(ForgeError::InvalidTransition(format!(
                "Cannot excuse commitment from status {}",
                self.status
            ))),
        }
    }

    pub fn scheduled_end(&self) -> DateTime<Utc> {
        self.scheduled_start + chrono::Duration::minutes(self.scheduled_duration_mins as i64)
    }

    pub fn is_overdue(&self, now: DateTime<Utc>, grace_period_mins: u32) -> bool {
        let deadline = self.scheduled_end() + chrono::Duration::minutes(grace_period_mins as i64);
        (self.status == CommitmentStatus::Planned || self.status == CommitmentStatus::Active)
            && now > deadline
    }
}
