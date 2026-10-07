use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use chrono::{Duration, Local, Utc};
use serde_json::json;

use forgex_core::{
    Activity, ActivityCategory, Commitment, CommitmentStatus, ForgeConfig,
    RecurrenceRule, SaveDay,
};
use forgex_scoring::{AvoidanceAnalyzer, CharacterCalculator, PenaltyCalculator, RecoveryEngine};
use forgex_storage::ForgeDb;

pub struct FfiContext {
    pub db: ForgeDb,
    pub config: ForgeConfig,
}

// ----------------------------------------------------
// Memory Management
// ----------------------------------------------------
fn c_str_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }
}

fn to_c_string(json_val: serde_json::Value) -> *mut c_char {
    let s = json_val.to_string();
    CString::new(s).unwrap_or_default().into_raw()
}

fn to_c_error(msg: &str) -> *mut c_char {
    to_c_string(json!({ "success": false, "error": msg }))
}

#[no_mangle]
pub extern "C" fn forgex_ffi_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

// ----------------------------------------------------
// Lifecycle
// ----------------------------------------------------
#[no_mangle]
pub extern "C" fn forgex_ffi_open(db_path: *const c_char) -> *mut FfiContext {
    let path_str = c_str_to_str(db_path);

    let db_res = match path_str {
        Some(p) if !p.trim().is_empty() => ForgeDb::open(p),
        _ => ForgeDb::open_in_memory(),
    };

    match db_res {
        Ok(db) => {
            let ctx = Box::new(FfiContext {
                db,
                config: ForgeConfig::default(),
            });
            Box::into_raw(ctx)
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn forgex_ffi_close(ctx: *mut FfiContext) {
    if !ctx.is_null() {
        unsafe {
            let _ = Box::from_raw(ctx);
        }
    }
}

// ----------------------------------------------------
// Dashboard & Status
// ----------------------------------------------------
#[no_mangle]
pub extern "C" fn forgex_ffi_status(ctx: *mut FfiContext) -> *mut c_char {
    if ctx.is_null() {
        return to_c_error("Null context pointer");
    }
    let ctx = unsafe { &*ctx };

    let all_commitments = match ctx.db.list_commitments() {
        Ok(c) => c,
        Err(e) => return to_c_error(&e.to_string()),
    };
    let all_activities = ctx.db.list_activities().unwrap_or_default();
    let active_penalties = ctx.db.get_active_penalties().unwrap_or_default();
    let available_save_days = ctx.db.get_available_save_days().unwrap_or_default();

    let today_local = Local::now().date_naive();

    let today_commitments: Vec<_> = all_commitments
        .iter()
        .filter(|c| c.scheduled_start.with_timezone(&Local).date_naive() == today_local)
        .collect();

    let today_entertainment_mins: u32 = all_activities
        .iter()
        .filter(|a| {
            a.started_at.with_timezone(&Local).date_naive() == today_local
                && a.category.is_entertainment()
        })
        .map(|a| a.duration_mins)
        .sum();

    let max_consecutive_skips = all_commitments
        .iter()
        .map(|c| c.consecutive_skips)
        .max()
        .unwrap_or(0);

    let recent_penalties = ctx.db.list_penalties().unwrap_or_default();
    let latest_avoidance_score = recent_penalties
        .first()
        .map(|p| p.avoidance_score)
        .unwrap_or(0);

    let total_remaining_penalty_mins: u32 = active_penalties
        .iter()
        .map(|p| p.remaining_penalty_mins)
        .sum();
    let max_restricted_days: u32 = active_penalties
        .iter()
        .map(|p| p.restricted_days)
        .max()
        .unwrap_or(0);

    let completed_this_week = all_commitments
        .iter()
        .filter(|c| {
            let days_ago = (Utc::now() - c.scheduled_start).num_days();
            (0..=7).contains(&days_ago) && c.status == CommitmentStatus::Completed
        })
        .count();
    let scheduled_this_week = all_commitments
        .iter()
        .filter(|c| {
            let days_ago = (Utc::now() - c.scheduled_start).num_days();
            (0..=7).contains(&days_ago)
        })
        .count();

    let consistency_ratio = if scheduled_this_week > 0 {
        (completed_this_week as f32 / scheduled_this_week as f32).min(1.0)
    } else {
        1.0
    };
    let consistency_days_approx = (consistency_ratio * 7.0).round() as u32;

    to_c_string(json!({
        "success": true,
        "today_commitments": today_commitments,
        "skip_streak": max_consecutive_skips,
        "today_entertainment_mins": today_entertainment_mins,
        "avoidance_score": latest_avoidance_score,
        "penalty": {
            "remaining_mins": total_remaining_penalty_mins,
            "restricted_days": max_restricted_days
        },
        "save_days": {
            "available": available_save_days.len(),
            "weekly_consistency": format!("{}/7", consistency_days_approx)
        }
    }))
}

// ----------------------------------------------------
// Tasks / Commitments
// ----------------------------------------------------
#[no_mangle]
pub extern "C" fn forgex_ffi_task_add(
    ctx: *mut FfiContext,
    title: *const c_char,
    start_rfc3339: *const c_char,
    duration_mins: u32,
    importance: u8,
    effort: u8,
    category: *const c_char,
) -> *mut c_char {
    if ctx.is_null() {
        return to_c_error("Null context pointer");
    }
    let ctx = unsafe { &*ctx };

    let title_str = match c_str_to_str(title) {
        Some(s) => s,
        None => return to_c_error("Missing title"),
    };
    let cat_str = c_str_to_str(category).unwrap_or("General");

    let scheduled_start = if let Some(s) = c_str_to_str(start_rfc3339) {
        chrono::DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    let commitment = match Commitment::new(
        title_str,
        importance,
        effort,
        scheduled_start,
        duration_mins,
        RecurrenceRule::Daily,
        cat_str,
    ) {
        Ok(c) => c,
        Err(e) => return to_c_error(&e.to_string()),
    };

    if let Err(e) = ctx.db.insert_commitment(&commitment) {
        return to_c_error(&e.to_string());
    }

    to_c_string(json!({
        "success": true,
        "id": commitment.id.to_string(),
        "title": commitment.title
    }))
}

#[no_mangle]
pub extern "C" fn forgex_ffi_task_complete(
    ctx: *mut FfiContext,
    id_prefix: *const c_char,
) -> *mut c_char {
    if ctx.is_null() {
        return to_c_error("Null context pointer");
    }
    let ctx = unsafe { &*ctx };

    let prefix = match c_str_to_str(id_prefix) {
        Some(s) => s,
        None => return to_c_error("Missing ID"),
    };

    let all = match ctx.db.list_commitments() {
        Ok(c) => c,
        Err(e) => return to_c_error(&e.to_string()),
    };

    let mut commitment = match all.into_iter().find(|c| c.id.to_string().starts_with(prefix)) {
        Some(c) => c,
        None => return to_c_error("Commitment not found"),
    };

    let now = Utc::now();
    if let Err(e) = commitment.complete(now) {
        return to_c_error(&e.to_string());
    }
    let _ = ctx.db.update_commitment(&commitment);

    let active_penalties = ctx.db.get_active_penalties().unwrap_or_default();
    let assessment = RecoveryEngine::evaluate_task_completion(&commitment, active_penalties.first());

    let mut reduced_mins = 0;
    if let Some(mut penalty) = active_penalties.into_iter().next() {
        reduced_mins = assessment.penalty_reduction_mins;
        penalty.reduce(reduced_mins, now);
        let _ = ctx.db.update_penalty(&penalty);
    }

    // Check streak
    let all_commitments = ctx.db.list_commitments().unwrap_or_default();
    let completed_streak = all_commitments.iter().filter(|c| c.status == CommitmentStatus::Completed).count() as u32;
    let available_save_days = ctx.db.get_available_save_days().unwrap_or_default().len() as u32;

    let mut earned_save_day = false;
    if RecoveryEngine::should_earn_save_day(available_save_days, completed_streak, &ctx.config) {
        let new_save_day = SaveDay::new_earned(now);
        let _ = ctx.db.insert_save_day(&new_save_day);
        earned_save_day = true;
    }

    to_c_string(json!({
        "success": true,
        "reduced_penalty_mins": reduced_mins,
        "screen_time_bonus_mins": assessment.screen_time_bonus_mins,
        "earned_save_day": earned_save_day
    }))
}

#[no_mangle]
pub extern "C" fn forgex_ffi_task_miss(
    ctx: *mut FfiContext,
    id_prefix: *const c_char,
    force_use_save_day: i32,
    force_no_save_day: i32,
) -> *mut c_char {
    if ctx.is_null() {
        return to_c_error("Null context pointer");
    }
    let ctx = unsafe { &*ctx };

    let prefix = match c_str_to_str(id_prefix) {
        Some(s) => s,
        None => return to_c_error("Missing ID"),
    };

    let all = match ctx.db.list_commitments() {
        Ok(c) => c,
        Err(e) => return to_c_error(&e.to_string()),
    };

    let mut commitment = match all.into_iter().find(|c| c.id.to_string().starts_with(prefix)) {
        Some(c) => c,
        None => return to_c_error("Commitment not found"),
    };

    let now = Utc::now();
    if let Err(e) = commitment.miss(now) {
        return to_c_error(&e.to_string());
    }
    let _ = ctx.db.update_commitment(&commitment);

    let window_start = commitment.scheduled_start - Duration::minutes(AvoidanceAnalyzer::WINDOW_BUFFER_MINS);
    let window_end = commitment.scheduled_end() + Duration::minutes(AvoidanceAnalyzer::WINDOW_BUFFER_MINS);
    let activities = ctx.db.list_activities_for_window(window_start, window_end).unwrap_or_default();

    let avoidance = AvoidanceAnalyzer::assess(&commitment, &activities);
    let active_penalties = ctx.db.get_active_penalties().unwrap_or_default();
    let current_restricted_days = active_penalties.iter().map(|p| p.restricted_days).max().unwrap_or(0);
    let prev_hit_max = active_penalties.iter().any(|p| p.actual_penalty_mins >= ctx.config.max_daily_penalty_mins);

    let calc = PenaltyCalculator::calculate(
        &commitment,
        &avoidance,
        &ctx.config,
        current_restricted_days,
        prev_hit_max,
    );

    let available_save_days = ctx.db.get_available_save_days().unwrap_or_default();
    let should_shield = RecoveryEngine::should_shield_missed_task(
        &commitment,
        available_save_days.len() as u32,
        &ctx.config,
        force_use_save_day == 1,
        force_no_save_day == 1,
    );

    if should_shield {
        let _ = ctx.db.consume_oldest_save_day(commitment.id, now);
    }

    let penalty = PenaltyCalculator::to_penalty_record(&commitment, &calc, should_shield);
    let _ = ctx.db.insert_penalty(&penalty);

    to_c_string(json!({
        "success": true,
        "raw_penalty_mins": calc.raw_penalty_mins,
        "actual_penalty_mins": penalty.actual_penalty_mins,
        "restricted_days": penalty.restricted_days,
        "shielded": should_shield,
        "avoidance_score": calc.avoidance_score
    }))
}

// ----------------------------------------------------
// Activity
// ----------------------------------------------------
#[no_mangle]
pub extern "C" fn forgex_ffi_activity_log(
    ctx: *mut FfiContext,
    application: *const c_char,
    domain_or_detail: *const c_char,
    duration_mins: u32,
    category_str: *const c_char,
) -> *mut c_char {
    if ctx.is_null() {
        return to_c_error("Null context pointer");
    }
    let ctx = unsafe { &*ctx };

    let app = match c_str_to_str(application) {
        Some(s) => s,
        None => return to_c_error("Missing application name"),
    };
    let detail = c_str_to_str(domain_or_detail).map(|s| s.to_string());
    let cat: ActivityCategory = c_str_to_str(category_str)
        .unwrap_or("medium")
        .parse()
        .unwrap_or(ActivityCategory::MediumEntertainment);

    let now = Utc::now();
    let act = match Activity::new(
        "android-mobile",
        app,
        detail,
        cat,
        now - Duration::minutes(duration_mins as i64),
        duration_mins,
    ) {
        Ok(a) => a,
        Err(e) => return to_c_error(&e.to_string()),
    };

    if let Err(e) = ctx.db.insert_activity(&act) {
        return to_c_error(&e.to_string());
    }

    to_c_string(json!({ "success": true, "id": act.id.to_string() }))
}

// ----------------------------------------------------
// Character
// ----------------------------------------------------
#[no_mangle]
pub extern "C" fn forgex_ffi_character_profile(ctx: *mut FfiContext) -> *mut c_char {
    if ctx.is_null() {
        return to_c_error("Null context pointer");
    }
    let ctx = unsafe { &*ctx };

    let commitments = ctx.db.list_commitments().unwrap_or_default();
    let penalties = ctx.db.list_penalties().unwrap_or_default();

    let profile = CharacterCalculator::calculate(&commitments, &penalties);

    to_c_string(json!({
        "success": true,
        "consistency": profile.consistency,
        "reliability": profile.reliability,
        "resilience": profile.resilience,
        "self_control": profile.self_control,
        "discipline": profile.discipline,
        "overall_level": profile.overall_level
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_ffi_lifecycle_and_status() {
        let ctx = forgex_ffi_open(std::ptr::null());
        assert!(!ctx.is_null());

        let status_ptr = forgex_ffi_status(ctx);
        assert!(!status_ptr.is_null());

        let json_str = unsafe { CStr::from_ptr(status_ptr).to_str().unwrap() };
        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed["success"], true);

        forgex_ffi_free_string(status_ptr);
        forgex_ffi_close(ctx);
    }
}
