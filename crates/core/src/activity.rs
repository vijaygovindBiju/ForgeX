use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{ForgeError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityCategory {
    Productive,
    Essential,
    MediumEntertainment,
    HighEntertainment,
    Unknown,
}

impl ActivityCategory {
    pub fn is_entertainment(&self) -> bool {
        matches!(self, ActivityCategory::MediumEntertainment | ActivityCategory::HighEntertainment)
    }

    pub fn entertainment_weight(&self) -> f32 {
        match self {
            ActivityCategory::Productive => 0.0,
            ActivityCategory::Essential => 0.0,
            ActivityCategory::Unknown => 0.2,
            ActivityCategory::MediumEntertainment => 1.0,
            ActivityCategory::HighEntertainment => 1.5,
        }
    }
}

impl std::fmt::Display for ActivityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityCategory::Productive => write!(f, "Productive"),
            ActivityCategory::Essential => write!(f, "Essential"),
            ActivityCategory::MediumEntertainment => write!(f, "Medium Entertainment"),
            ActivityCategory::HighEntertainment => write!(f, "High Entertainment"),
            ActivityCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::str::FromStr for ActivityCategory {
    type Err = ForgeError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().replace(['-', '_', ' '], "").as_str() {
            "productive" | "prod" => Ok(ActivityCategory::Productive),
            "essential" | "ess" => Ok(ActivityCategory::Essential),
            "mediumentertainment" | "medium" | "medent" => Ok(ActivityCategory::MediumEntertainment),
            "highentertainment" | "high" | "highent" => Ok(ActivityCategory::HighEntertainment),
            "unknown" => Ok(ActivityCategory::Unknown),
            _ => Err(ForgeError::Validation(format!("Invalid activity category: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Uuid,
    pub device: String,
    pub application: String,
    pub domain_or_detail: Option<String>,
    pub category: ActivityCategory,
    pub started_at: DateTime<Utc>,
    pub duration_mins: u32,
    pub created_at: DateTime<Utc>,
}

impl Activity {
    pub fn new(
        device: impl Into<String>,
        application: impl Into<String>,
        domain_or_detail: Option<String>,
        category: ActivityCategory,
        started_at: DateTime<Utc>,
        duration_mins: u32,
    ) -> Result<Self> {
        let application = application.into().trim().to_string();
        if application.is_empty() {
            return Err(ForgeError::Validation("Application name cannot be empty".to_string()));
        }

        if duration_mins == 0 {
            return Err(ForgeError::Validation("Activity duration must be > 0".to_string()));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            device: device.into().trim().to_string(),
            application,
            domain_or_detail,
            category,
            started_at,
            duration_mins,
            created_at: now,
        })
    }

    pub fn ended_at(&self) -> DateTime<Utc> {
        self.started_at + chrono::Duration::minutes(self.duration_mins as i64)
    }

    pub fn overlaps_window(&self, window_start: DateTime<Utc>, window_end: DateTime<Utc>) -> bool {
        let act_start = self.started_at;
        let act_end = self.ended_at();
        act_start < window_end && act_end > window_start
    }
}
