use forgex_core::{Commitment, ForgeConfig, PenaltyRecord};
use crate::avoidance::AvoidanceAssessment;

#[derive(Debug, Clone, PartialEq)]
pub struct PenaltyCalculationResult {
    pub raw_penalty_mins: u32,
    pub actual_penalty_mins: u32,
    pub restricted_days: u32,
    pub avoidance_score: u32,
    pub repetition_factor: u32,
    pub entertainment_factor: u8,
    pub hit_max_penalty: bool,
}

pub struct PenaltyCalculator;

impl PenaltyCalculator {
    pub fn calculate(
        commitment: &Commitment,
        avoidance: &AvoidanceAssessment,
        config: &ForgeConfig,
        current_restricted_days: u32,
        previous_day_hit_max: bool,
    ) -> PenaltyCalculationResult {
        let importance = commitment.importance as u32;
        let effort = commitment.expected_effort as u32;
        // Repetition bounded [1, 5] based on consecutive skips
        let repetition = commitment.consecutive_skips.clamp(1, 5);
        let entertainment = avoidance.entertainment_factor as u32;

        let raw_penalty = importance * effort * repetition * entertainment;
        let max_daily = config.max_daily_penalty_mins;

        let hit_max = raw_penalty >= max_daily;
        let actual_penalty = raw_penalty.min(max_daily);

        // Persistent penalty logic: if consecutive max penalties occur, extend restricted days
        let mut restricted_days = current_restricted_days;
        if hit_max {
            if previous_day_hit_max {
                restricted_days = restricted_days.saturating_add(1).max(2);
            } else if restricted_days == 0 {
                restricted_days = 1;
            }
        }

        PenaltyCalculationResult {
            raw_penalty_mins: raw_penalty,
            actual_penalty_mins: actual_penalty,
            restricted_days,
            avoidance_score: avoidance.avoidance_score,
            repetition_factor: repetition,
            entertainment_factor: avoidance.entertainment_factor,
            hit_max_penalty: hit_max,
        }
    }

    pub fn to_penalty_record(
        commitment: &Commitment,
        calc: &PenaltyCalculationResult,
        shielded_by_save_day: bool,
    ) -> PenaltyRecord {
        PenaltyRecord::new(
            commitment.id,
            calc.raw_penalty_mins,
            calc.actual_penalty_mins,
            calc.restricted_days,
            calc.avoidance_score,
            shielded_by_save_day,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use forgex_core::RecurrenceRule;

    #[test]
    fn test_penalty_formula_and_capping() {
        let commitment = Commitment::new(
            "Workout",
            4,
            3,
            Utc::now(),
            45,
            RecurrenceRule::Daily,
            "Health",
        ).unwrap();

        let avoidance = AvoidanceAssessment {
            total_entertainment_mins: 45.0,
            entertainment_factor: 4,
            avoidance_score: 75,
            overlapping_activities_count: 1,
        };

        let config = ForgeConfig {
            max_daily_penalty_mins: 120,
            ..Default::default()
        };

        // Raw: 4 * 3 * 1 * 4 = 48 mins (first skip: consecutive_skips = 1)
        let mut first_miss = commitment.clone();
        first_miss.consecutive_skips = 1;
        let res = PenaltyCalculator::calculate(&first_miss, &avoidance, &config, 0, false);
        assert_eq!(res.raw_penalty_mins, 48);
        assert_eq!(res.actual_penalty_mins, 48);
        assert!(!res.hit_max_penalty);

        // With high repetition (consecutive skips = 4 -> repetition = 4):
        let mut skipped_commitment = commitment.clone();
        skipped_commitment.consecutive_skips = 4;
        // Raw: 4 * 3 * 4 * 4 = 192 mins -> capped at 120
        let res2 = PenaltyCalculator::calculate(&skipped_commitment, &avoidance, &config, 0, false);
        assert_eq!(res2.raw_penalty_mins, 192);
        assert_eq!(res2.actual_penalty_mins, 120);
        assert!(res2.hit_max_penalty);
        assert_eq!(res2.restricted_days, 1);
    }
}
