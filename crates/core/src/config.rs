use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeConfig {
    pub max_daily_penalty_mins: u32,
    pub save_days_max: u32,
    pub save_day_streak_required: u32,
    pub auto_save_day_importance_threshold: u8,
    pub default_entertainment_allowance_mins: u32,
    pub grace_period_mins: u32,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            max_daily_penalty_mins: 120,
            save_days_max: 3,
            save_day_streak_required: 7,
            auto_save_day_importance_threshold: 3,
            default_entertainment_allowance_mins: 60,
            grace_period_mins: 15,
        }
    }
}
