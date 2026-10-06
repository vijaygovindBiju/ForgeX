pub mod activity;
pub mod character;
pub mod commitment;
pub mod config;
pub mod error;
pub mod event;
pub mod penalty;
pub mod save_day;

pub use activity::{Activity, ActivityCategory};
pub use character::CharacterProfile;
pub use commitment::{Commitment, CommitmentStatus, RecurrenceRule};
pub use config::ForgeConfig;
pub use error::{ForgeError, Result};
pub use event::{ForgeEvent, ForgeEventPayload};
pub use penalty::{PenaltyRecord, PenaltyStatus};
pub use save_day::{SaveDay, SaveDayStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_commitment_lifecycle() {
        let now = Utc::now();
        let mut commitment = Commitment::new(
            "Morning Workout",
            4,
            3,
            now,
            45,
            RecurrenceRule::Daily,
            "Health",
        ).unwrap();

        assert_eq!(commitment.status, CommitmentStatus::Planned);
        assert_eq!(commitment.importance, 4);

        commitment.start(now).unwrap();
        assert_eq!(commitment.status, CommitmentStatus::Active);

        commitment.complete(now + Duration::minutes(40)).unwrap();
        assert_eq!(commitment.status, CommitmentStatus::Completed);
    }

    #[test]
    fn test_penalty_reduction() {
        let mut penalty = PenaltyRecord::new(
            uuid::Uuid::new_v4(),
            120,
            120,
            1,
            50,
            false,
        );
        assert_eq!(penalty.remaining_penalty_mins, 120);
        assert_eq!(penalty.restricted_days, 1);

        penalty.reduce(60, Utc::now());
        assert_eq!(penalty.remaining_penalty_mins, 60);
        assert_eq!(penalty.status, PenaltyStatus::Active);

        penalty.reduce(60, Utc::now());
        assert_eq!(penalty.remaining_penalty_mins, 0);
        assert_eq!(penalty.restricted_days, 0);
        assert_eq!(penalty.status, PenaltyStatus::Cleared);
    }
}
