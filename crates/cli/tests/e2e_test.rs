use std::fs;
use chrono::Utc;
use forgex_core::{ActivityCategory, PenaltyStatus, SaveDayStatus};
use forgex_storage::ForgeDb;
use forgex_scoring::{AvoidanceAnalyzer, CharacterCalculator, PenaltyCalculator, RecoveryEngine};

#[test]
fn test_end_to_end_behavioral_cycle() {
    let temp_dir = std::env::temp_dir().join(format!("forgex_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();
    let db_path = temp_dir.join("test.db");

    let db = ForgeDb::open(&db_path).unwrap();
    let now = Utc::now();

    // 1. Create a commitment: "Deep Work" (Importance 5, Effort 4)
    let commitment = forgex_core::Commitment::new(
        "Deep Work: Core Engine",
        5,
        4,
        now,
        60,
        forgex_core::RecurrenceRule::Daily,
        "Coding",
    ).unwrap();
    db.insert_commitment(&commitment).unwrap();

    // 2. Log entertainment activity during the commitment window (avoidance behavior)
    let avoidance_activity = forgex_core::Activity::new(
        "linux-laptop",
        "Firefox",
        Some("youtube.com/shorts".to_string()),
        ActivityCategory::HighEntertainment,
        now + chrono::Duration::minutes(15),
        40,
    ).unwrap();
    db.insert_activity(&avoidance_activity).unwrap();

    // 3. User misses the commitment
    let mut missed_task = commitment.clone();
    missed_task.miss(now + chrono::Duration::minutes(70)).unwrap();
    db.update_commitment(&missed_task).unwrap();

    // 4. Avoidance detection & Penalty calculation
    let activities = db.list_activities().unwrap();
    let avoidance = AvoidanceAnalyzer::assess(&missed_task, &activities);

    assert!(avoidance.total_entertainment_mins > 0.0);
    assert_eq!(avoidance.entertainment_factor, 4); // 40 mins * 1.5 = 60 mins -> factor 4
    assert!(avoidance.avoidance_score > 50);

    let config = forgex_core::ForgeConfig::default();
    let penalty_calc = PenaltyCalculator::calculate(&missed_task, &avoidance, &config, 0, false);

    // Formula: 5 (imp) * 4 (effort) * 1 (rep) * 4 (ent) = 80 mins
    assert_eq!(penalty_calc.raw_penalty_mins, 80);
    assert_eq!(penalty_calc.actual_penalty_mins, 80);

    // Insert penalty without Save Day
    let penalty = PenaltyCalculator::to_penalty_record(&missed_task, &penalty_calc, false);
    db.insert_penalty(&penalty).unwrap();

    let active_penalties = db.get_active_penalties().unwrap();
    assert_eq!(active_penalties.len(), 1);
    assert_eq!(active_penalties[0].remaining_penalty_mins, 80);

    // 5. Recovery: User commits to and completes a recovery task (Effort 4)
    let recovery_task = forgex_core::Commitment::new(
        "Recovery Exercise",
        4,
        4, // Effort 4 -> 4 * 15 = 60m recovery reduction
        now + chrono::Duration::hours(2),
        30,
        forgex_core::RecurrenceRule::Once,
        "Health",
    ).unwrap();
    db.insert_commitment(&recovery_task).unwrap();

    let mut completed_task = recovery_task.clone();
    completed_task.complete(now + chrono::Duration::hours(3)).unwrap();
    db.update_commitment(&completed_task).unwrap();

    let recovery_assessment = RecoveryEngine::evaluate_task_completion(&completed_task, Some(&active_penalties[0]));
    assert_eq!(recovery_assessment.penalty_reduction_mins, 60);

    let mut current_penalty = active_penalties[0].clone();
    current_penalty.reduce(recovery_assessment.penalty_reduction_mins, now);
    db.update_penalty(&current_penalty).unwrap();

    // Remaining penalty should now be 20m (80 - 60)
    assert_eq!(current_penalty.remaining_penalty_mins, 20);
    assert_eq!(current_penalty.status, PenaltyStatus::Active);

    // 6. Complete another task to clear the remaining 20m
    let final_task = forgex_core::Commitment::new(
        "Final Task",
        3,
        2, // Effort 2 -> 30m reduction
        now + chrono::Duration::hours(4),
        20,
        forgex_core::RecurrenceRule::Once,
        "Chores",
    ).unwrap();
    let mut final_completed = final_task.clone();
    final_completed.complete(now + chrono::Duration::hours(5)).unwrap();
    db.insert_commitment(&final_completed).unwrap();

    current_penalty.reduce(30, now);
    db.update_penalty(&current_penalty).unwrap();

    assert_eq!(current_penalty.remaining_penalty_mins, 0);
    assert_eq!(current_penalty.status, PenaltyStatus::Cleared);

    // 7. Verify Character Profile calculation
    let all_commitments = db.list_commitments().unwrap();
    let all_penalties = db.list_penalties().unwrap();
    let profile = CharacterCalculator::calculate(&all_commitments, &all_penalties);

    assert!(profile.consistency >= 60);
    assert!(profile.resilience >= 70); // Recovered from miss!

    // Cleanup
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_save_day_shielding_e2e() {
    let temp_dir = std::env::temp_dir().join(format!("forgex_test_save_day_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();
    let db_path = temp_dir.join("test.db");
    let db = ForgeDb::open(&db_path).unwrap();
    let now = Utc::now();

    // Award 1 Save Day
    let save_day = forgex_core::SaveDay::new_earned(now);
    db.insert_save_day(&save_day).unwrap();
    assert_eq!(db.get_available_save_days().unwrap().len(), 1);

    // Critical commitment (importance 5 >= threshold 3)
    let commitment = forgex_core::Commitment::new(
        "Critical Exam",
        5,
        5,
        now,
        60,
        forgex_core::RecurrenceRule::Once,
        "Academics",
    ).unwrap();
    db.insert_commitment(&commitment).unwrap();

    // Miss commitment
    let mut missed = commitment.clone();
    missed.miss(now + chrono::Duration::hours(2)).unwrap();
    db.update_commitment(&missed).unwrap();

    let config = forgex_core::ForgeConfig::default();
    let should_shield = RecoveryEngine::should_shield_missed_task(&missed, 1, &config, false, false);
    assert!(should_shield);

    // Consume Save Day
    let consumed = db.consume_oldest_save_day(missed.id, now).unwrap().unwrap();
    assert_eq!(consumed.status, SaveDayStatus::Consumed);
    assert_eq!(db.get_available_save_days().unwrap().len(), 0);

    // Shielded penalty has 0 remaining minutes and Cleared status
    let penalty = forgex_core::PenaltyRecord::new(missed.id, 100, 100, 0, 0, true);
    assert_eq!(penalty.remaining_penalty_mins, 0);
    assert_eq!(penalty.status, PenaltyStatus::Cleared);
    assert!(penalty.shielded_by_save_day);

    let _ = fs::remove_dir_all(temp_dir);
}
