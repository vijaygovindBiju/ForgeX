use chrono::Duration;
use forgex_core::{Activity, Commitment};

#[derive(Debug, Clone, PartialEq)]
pub struct AvoidanceAssessment {
    pub total_entertainment_mins: f32,
    pub entertainment_factor: u8, // 1 to 5
    pub avoidance_score: u32,      // 0 to 100
    pub overlapping_activities_count: usize,
}

pub struct AvoidanceAnalyzer;

impl AvoidanceAnalyzer {
    pub const WINDOW_BUFFER_MINS: i64 = 30;

    /// Evaluates entertainment activity during and around (±30 mins) the commitment's window.
    pub fn assess(commitment: &Commitment, activities: &[Activity]) -> AvoidanceAssessment {
        let window_start = commitment.scheduled_start - Duration::minutes(Self::WINDOW_BUFFER_MINS);
        let window_end = commitment.scheduled_end() + Duration::minutes(Self::WINDOW_BUFFER_MINS);

        let mut total_weighted_mins = 0.0f32;
        let mut count = 0;

        for activity in activities {
            if !activity.category.is_entertainment() {
                continue;
            }

            if activity.overlaps_window(window_start, window_end) {
                // Compute overlapping duration within the buffer window
                let act_start = activity.started_at;
                let act_end = activity.ended_at();

                let effective_start = act_start.max(window_start);
                let effective_end = act_end.min(window_end);

                let overlap_duration = (effective_end - effective_start).num_minutes().max(0) as f32;
                let weighted = overlap_duration * activity.category.entertainment_weight();

                total_weighted_mins += weighted;
                count += 1;
            }
        }

        let entertainment_factor = Self::calculate_entertainment_factor(total_weighted_mins);
        let avoidance_score = Self::calculate_avoidance_score(
            total_weighted_mins,
            commitment.importance,
            commitment.consecutive_skips,
        );

        AvoidanceAssessment {
            total_entertainment_mins: total_weighted_mins,
            entertainment_factor,
            avoidance_score,
            overlapping_activities_count: count,
        }
    }

    /// Factor bounded [1, 5]
    /// 0 mins -> 1
    /// 1..=15 mins -> 2
    /// 16..=30 mins -> 3
    /// 31..=60 mins -> 4
    /// >60 mins -> 5
    pub fn calculate_entertainment_factor(entertainment_mins: f32) -> u8 {
        if entertainment_mins <= 0.0 {
            1
        } else if entertainment_mins <= 15.0 {
            2
        } else if entertainment_mins <= 30.0 {
            3
        } else if entertainment_mins <= 60.0 {
            4
        } else {
            5
        }
    }

    /// Avoidance score from 0 to 100 based on entertainment minutes, importance, and skip frequency
    pub fn calculate_avoidance_score(
        entertainment_mins: f32,
        importance: u8,
        consecutive_skips: u32,
    ) -> u32 {
        if entertainment_mins <= 0.0 {
            return 0;
        }

        // Base intensity from minutes (capped at 60 mins -> 60 points)
        let min_points = (entertainment_mins.min(60.0) / 60.0) * 50.0;
        // Importance factor adds up to 30 points
        let imp_points = ((importance as f32) / 5.0) * 30.0;
        // Skips factor adds up to 20 points
        let skip_points = ((consecutive_skips.min(5) as f32) / 5.0) * 20.0;

        let total = (min_points + imp_points + skip_points).round() as u32;
        total.min(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use forgex_core::{ActivityCategory, RecurrenceRule};

    #[test]
    fn test_avoidance_factor_mapping() {
        assert_eq!(AvoidanceAnalyzer::calculate_entertainment_factor(0.0), 1);
        assert_eq!(AvoidanceAnalyzer::calculate_entertainment_factor(10.0), 2);
        assert_eq!(AvoidanceAnalyzer::calculate_entertainment_factor(25.0), 3);
        assert_eq!(AvoidanceAnalyzer::calculate_entertainment_factor(45.0), 4);
        assert_eq!(AvoidanceAnalyzer::calculate_entertainment_factor(90.0), 5);
    }

    #[test]
    fn test_avoidance_assessment_with_activities() {
        let now = Utc::now();
        let commitment = Commitment::new(
            "Study Rust",
            4,
            3,
            now,
            60,
            RecurrenceRule::Daily,
            "Study",
        ).unwrap();

        let activities = vec![
            Activity::new(
                "laptop",
                "Firefox",
                Some("youtube.com".to_string()),
                ActivityCategory::HighEntertainment,
                now + Duration::minutes(10),
                30,
            ).unwrap(),
        ];

        let assessment = AvoidanceAnalyzer::assess(&commitment, &activities);
        assert!(assessment.total_entertainment_mins > 0.0);
        assert!(assessment.entertainment_factor >= 3);
        assert!(assessment.avoidance_score > 0);
        assert_eq!(assessment.overlapping_activities_count, 1);
    }
}
