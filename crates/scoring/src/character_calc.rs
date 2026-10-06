use forgex_core::{CharacterProfile, Commitment, CommitmentStatus, PenaltyRecord};

pub struct CharacterCalculator;

impl CharacterCalculator {
    pub fn calculate(
        commitments: &[Commitment],
        penalties: &[PenaltyRecord],
    ) -> CharacterProfile {
        if commitments.is_empty() {
            return CharacterProfile::default();
        }

        let completed = commitments.iter().filter(|c| c.status == CommitmentStatus::Completed).count();
        let missed = commitments.iter().filter(|c| c.status == CommitmentStatus::Missed).count();
        let total_resolved = completed + missed;

        // Consistency: percentage of resolved tasks that were completed
        let consistency = if total_resolved == 0 {
            50
        } else {
            ((completed as f32 / total_resolved as f32) * 100.0).round() as u32
        };

        // Reliability: percentage of tasks that were completed with 0 prior skips
        let first_try_completed = commitments.iter().filter(|c| {
            c.status == CommitmentStatus::Completed && c.consecutive_skips == 0
        }).count();
        let reliability = if completed == 0 {
            50
        } else {
            ((first_try_completed as f32 / completed as f32) * 100.0).round() as u32
        };

        // Self-Control: 100 minus average avoidance score from penalties
        let self_control = if penalties.is_empty() {
            80
        } else {
            let avg_avoidance: f32 = penalties.iter().map(|p| p.avoidance_score as f32).sum::<f32>() / penalties.len() as f32;
            (100.0 - avg_avoidance).clamp(10.0, 100.0).round() as u32
        };

        // Resilience: percentage of misses that were followed by recovery / completed commitments
        let resilience = if missed == 0 {
            75
        } else {
            let recovered_penalties = penalties.iter().filter(|p| p.remaining_penalty_mins == 0).count();
            ((recovered_penalties as f32 / penalties.len().max(1) as f32) * 100.0).round() as u32
        };

        // Discipline: weighted completion of high importance / effort tasks (importance >= 4 or effort >= 4)
        let hard_commitments: Vec<&Commitment> = commitments.iter().filter(|c| c.importance >= 4 || c.expected_effort >= 4).collect();
        let discipline = if hard_commitments.is_empty() {
            consistency
        } else {
            let hard_completed = hard_commitments.iter().filter(|c| c.status == CommitmentStatus::Completed).count();
            ((hard_completed as f32 / hard_commitments.len() as f32) * 100.0).round() as u32
        };

        CharacterProfile::new(consistency, reliability, resilience, self_control, discipline)
    }
}
