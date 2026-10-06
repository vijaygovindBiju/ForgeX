use forgex_core::{Commitment, ForgeConfig, PenaltyRecord, PenaltyStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryAssessment {
    pub penalty_reduction_mins: u32,
    pub screen_time_bonus_mins: u32,
    pub penalty_cleared: bool,
    pub restricted_days_reduced: bool,
}

pub struct RecoveryEngine;

impl RecoveryEngine {
    /// Evaluates recovery rewards when a commitment is successfully completed
    pub fn evaluate_task_completion(
        commitment: &Commitment,
        active_penalty: Option<&PenaltyRecord>,
    ) -> RecoveryAssessment {
        let effort = commitment.expected_effort as u32;
        let penalty_reduction_mins = effort * 15;
        let screen_time_bonus_mins = effort * 5;

        let mut penalty_cleared = false;
        let mut restricted_days_reduced = false;

        if let Some(penalty) = active_penalty {
            if penalty.status == PenaltyStatus::Active {
                if penalty_reduction_mins >= penalty.remaining_penalty_mins {
                    penalty_cleared = true;
                    if penalty.restricted_days > 0 {
                        restricted_days_reduced = true;
                    }
                }
            }
        }

        RecoveryAssessment {
            penalty_reduction_mins,
            screen_time_bonus_mins,
            penalty_cleared,
            restricted_days_reduced,
        }
    }

    /// Checks if a missed commitment qualifies for Save Day shielding
    pub fn should_shield_missed_task(
        commitment: &Commitment,
        available_save_days: u32,
        config: &ForgeConfig,
        force_use: bool,
        force_skip: bool,
    ) -> bool {
        if available_save_days == 0 || force_skip {
            return false;
        }

        if force_use {
            return true;
        }

        commitment.importance >= config.auto_save_day_importance_threshold
    }

    /// Evaluates if consistency streak warrants earning a new Save Day
    pub fn should_earn_save_day(
        current_save_days: u32,
        consecutive_successful_days: u32,
        config: &ForgeConfig,
    ) -> bool {
        if current_save_days >= config.save_days_max {
            return false;
        }

        consecutive_successful_days > 0
            && consecutive_successful_days % config.save_day_streak_required == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use forgex_core::RecurrenceRule;

    #[test]
    fn test_recovery_evaluation() {
        let commitment = Commitment::new(
            "Hard Coding Session",
            4,
            4,
            Utc::now(),
            90,
            RecurrenceRule::Daily,
            "Deep Work",
        ).unwrap();

        let penalty = PenaltyRecord::new(
            uuid::Uuid::new_v4(),
            50,
            50,
            0,
            20,
            false,
        );

        let rec = RecoveryEngine::evaluate_task_completion(&commitment, Some(&penalty));
        assert_eq!(rec.penalty_reduction_mins, 60); // 4 * 15
        assert_eq!(rec.screen_time_bonus_mins, 20); // 4 * 5
        assert!(rec.penalty_cleared); // 60 >= 50
    }

    #[test]
    fn test_save_day_shielding_threshold() {
        let config = ForgeConfig::default(); // threshold = 3

        let high_imp = Commitment::new("Exam Prep", 4, 3, Utc::now(), 60, RecurrenceRule::Once, "Study").unwrap();
        let low_imp = Commitment::new("Casual Read", 2, 1, Utc::now(), 15, RecurrenceRule::Once, "Reading").unwrap();

        assert!(RecoveryEngine::should_shield_missed_task(&high_imp, 1, &config, false, false));
        assert!(!RecoveryEngine::should_shield_missed_task(&low_imp, 1, &config, false, false));
        assert!(RecoveryEngine::should_shield_missed_task(&low_imp, 1, &config, true, false));
        assert!(!RecoveryEngine::should_shield_missed_task(&high_imp, 1, &config, false, true));
        assert!(!RecoveryEngine::should_shield_missed_task(&high_imp, 0, &config, false, false));
    }
}
