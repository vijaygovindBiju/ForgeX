use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use forgex_core::{
    Activity, ActivityCategory, Commitment, CommitmentStatus, ForgeConfig,
    RecurrenceRule, SaveDay,
};
use forgex_scoring::{AvoidanceAnalyzer, CharacterCalculator, PenaltyCalculator, RecoveryEngine};
use forgex_storage::ForgeDb;

#[derive(Clone)]
pub struct AppState {
    pub db: ForgeDb,
    pub config: ForgeConfig,
    pub pairing_token: String,
}

#[derive(Deserialize)]
pub struct AuthQuery {
    pub token: Option<String>,
}

fn check_auth(headers: &HeaderMap, query: &AuthQuery, expected_token: &str) -> bool {
    if let Some(auth) = headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            if val.trim().strip_prefix("Bearer ").map(|s| s.trim()) == Some(expected_token) {
                return true;
            }
        }
    }
    if let Some(ref t) = query.token {
        if t == expected_token {
            return true;
        }
    }
    false
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/status", get(get_status))
        .route("/api/commitments", get(list_commitments).post(create_commitment))
        .route("/api/commitments/:id/start", post(start_commitment))
        .route("/api/commitments/:id/complete", post(complete_commitment))
        .route("/api/commitments/:id/miss", post(miss_commitment))
        .route("/api/activities", post(log_activity))
        .route("/api/character", get(get_character))
        .route("/api/recover", get(get_recovery))
        .route("/api/sync", post(sync_state))
        .layer(cors)
        .with_state(Arc::new(state))
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "online", "system": "ForgeX Sync Daemon" })))
}

// ----------------------------------------------------
// Handlers
// ----------------------------------------------------

async fn get_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized: invalid pairing token" }))).into_response();
    }

    let all_commitments = state.db.list_commitments().unwrap_or_default();
    let all_activities = state.db.list_activities().unwrap_or_default();
    let active_penalties = state.db.get_active_penalties().unwrap_or_default();
    let available_save_days = state.db.get_available_save_days().unwrap_or_default();

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

    let recent_penalties = state.db.list_penalties().unwrap_or_default();
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

    (StatusCode::OK, Json(json!({
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
    }))).into_response()
}

async fn list_commitments(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    let all = state.db.list_commitments().unwrap_or_default();
    (StatusCode::OK, Json(json!(all))).into_response()
}

#[derive(Deserialize)]
pub struct CreateCommitmentRequest {
    pub title: String,
    pub start: Option<String>,
    pub duration: u32,
    pub importance: u8,
    pub effort: u8,
    pub category: Option<String>,
    pub recurrence: Option<String>,
    pub description: Option<String>,
}

async fn create_commitment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    Json(payload): Json<CreateCommitmentRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    let start = payload.start.as_deref().and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let recurrence = match payload.recurrence.as_deref().unwrap_or("daily").to_lowercase().as_str() {
        "once" => RecurrenceRule::Once,
        "weekdays" => RecurrenceRule::Weekdays,
        _ => RecurrenceRule::Daily,
    };

    let mut commitment = match Commitment::new(
        payload.title,
        payload.importance,
        payload.effort,
        start,
        payload.duration,
        recurrence,
        payload.category.unwrap_or_else(|| "General".into()),
    ) {
        Ok(c) => c,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
    };

    commitment.description = payload.description;

    if let Err(e) = state.db.insert_commitment(&commitment) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
    }

    (StatusCode::CREATED, Json(json!(commitment))).into_response()
}

async fn start_commitment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    let all = state.db.list_commitments().unwrap_or_default();
    let mut commitment = match all.into_iter().find(|c| c.id.to_string().starts_with(&id)) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Commitment not found" }))).into_response(),
    };

    if let Err(e) = commitment.start(Utc::now()) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response();
    }

    let _ = state.db.update_commitment(&commitment);
    (StatusCode::OK, Json(json!(commitment))).into_response()
}

async fn complete_commitment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    let all = state.db.list_commitments().unwrap_or_default();
    let mut commitment = match all.into_iter().find(|c| c.id.to_string().starts_with(&id)) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Commitment not found" }))).into_response(),
    };

    let now = Utc::now();
    if let Err(e) = commitment.complete(now) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response();
    }
    let _ = state.db.update_commitment(&commitment);

    let active_penalties = state.db.get_active_penalties().unwrap_or_default();
    let assessment = RecoveryEngine::evaluate_task_completion(&commitment, active_penalties.first());

    let mut reduced_mins = 0;
    if let Some(mut penalty) = active_penalties.into_iter().next() {
        reduced_mins = assessment.penalty_reduction_mins;
        penalty.reduce(reduced_mins, now);
        let _ = state.db.update_penalty(&penalty);
    }

    // Streak check
    let all_commitments = state.db.list_commitments().unwrap_or_default();
    let completed_streak = all_commitments.iter().filter(|c| c.status == CommitmentStatus::Completed).count() as u32;
    let available_save_days = state.db.get_available_save_days().unwrap_or_default().len() as u32;

    let mut earned_save_day = false;
    if RecoveryEngine::should_earn_save_day(available_save_days, completed_streak, &state.config) {
        let new_save_day = SaveDay::new_earned(now);
        let _ = state.db.insert_save_day(&new_save_day);
        earned_save_day = true;
    }

    (StatusCode::OK, Json(json!({
        "commitment": commitment,
        "reduced_penalty_mins": reduced_mins,
        "screen_time_bonus_mins": assessment.screen_time_bonus_mins,
        "earned_save_day": earned_save_day
    }))).into_response()
}

#[derive(Deserialize)]
pub struct MissCommitmentRequest {
    pub force_use_save_day: Option<bool>,
    pub force_no_save_day: Option<bool>,
}

async fn miss_commitment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    Json(payload): Json<Option<MissCommitmentRequest>>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    let all = state.db.list_commitments().unwrap_or_default();
    let mut commitment = match all.into_iter().find(|c| c.id.to_string().starts_with(&id)) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Commitment not found" }))).into_response(),
    };

    let now = Utc::now();
    if let Err(e) = commitment.miss(now) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response();
    }
    let _ = state.db.update_commitment(&commitment);

    let window_start = commitment.scheduled_start - Duration::minutes(AvoidanceAnalyzer::WINDOW_BUFFER_MINS);
    let window_end = commitment.scheduled_end() + Duration::minutes(AvoidanceAnalyzer::WINDOW_BUFFER_MINS);
    let activities = state.db.list_activities_for_window(window_start, window_end).unwrap_or_default();

    let avoidance = AvoidanceAnalyzer::assess(&commitment, &activities);
    let active_penalties = state.db.get_active_penalties().unwrap_or_default();
    let current_restricted_days = active_penalties.iter().map(|p| p.restricted_days).max().unwrap_or(0);
    let prev_hit_max = active_penalties.iter().any(|p| p.actual_penalty_mins >= state.config.max_daily_penalty_mins);

    let calc = PenaltyCalculator::calculate(
        &commitment,
        &avoidance,
        &state.config,
        current_restricted_days,
        prev_hit_max,
    );

    let req = payload.unwrap_or(MissCommitmentRequest { force_use_save_day: None, force_no_save_day: None });
    let available_save_days = state.db.get_available_save_days().unwrap_or_default();
    let should_shield = RecoveryEngine::should_shield_missed_task(
        &commitment,
        available_save_days.len() as u32,
        &state.config,
        req.force_use_save_day.unwrap_or(false),
        req.force_no_save_day.unwrap_or(false),
    );

    if should_shield {
        let _ = state.db.consume_oldest_save_day(commitment.id, now);
    }

    let penalty = PenaltyCalculator::to_penalty_record(&commitment, &calc, should_shield);
    let _ = state.db.insert_penalty(&penalty);

    (StatusCode::OK, Json(json!({
        "commitment": commitment,
        "raw_penalty_mins": calc.raw_penalty_mins,
        "actual_penalty_mins": penalty.actual_penalty_mins,
        "restricted_days": penalty.restricted_days,
        "shielded": should_shield,
        "avoidance_score": calc.avoidance_score
    }))).into_response()
}

#[derive(Deserialize)]
pub struct LogActivityRequest {
    pub application: String,
    pub detail: Option<String>,
    pub duration_mins: u32,
    pub category: String,
    pub started_at: Option<String>,
    pub device: Option<String>,
}

async fn log_activity(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    Json(payload): Json<LogActivityRequest>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    let cat: ActivityCategory = payload.category.parse().unwrap_or(ActivityCategory::MediumEntertainment);
    let started = payload.started_at.as_deref().and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - Duration::minutes(payload.duration_mins as i64));

    let act = match Activity::new(
        payload.device.unwrap_or_else(|| "android-phone".into()),
        payload.application,
        payload.detail,
        cat,
        started,
        payload.duration_mins,
    ) {
        Ok(a) => a,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
    };

    if let Err(e) = state.db.insert_activity(&act) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
    }

    (StatusCode::CREATED, Json(json!(act))).into_response()
}

async fn get_character(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    let commitments = state.db.list_commitments().unwrap_or_default();
    let penalties = state.db.list_penalties().unwrap_or_default();
    let profile = CharacterCalculator::calculate(&commitments, &penalties);

    (StatusCode::OK, Json(json!(profile))).into_response()
}

async fn get_recovery(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    let active_penalties = state.db.get_active_penalties().unwrap_or_default();
    let total_remaining_mins: u32 = active_penalties.iter().map(|p| p.remaining_penalty_mins).sum();
    let max_restricted_days: u32 = active_penalties.iter().map(|p| p.restricted_days).max().unwrap_or(0);

    let all = state.db.list_commitments().unwrap_or_default();
    let available_tasks: Vec<_> = all.into_iter().filter(|c| c.status == CommitmentStatus::Planned || c.status == CommitmentStatus::Active).collect();

    (StatusCode::OK, Json(json!({
        "remaining_penalty_mins": total_remaining_mins,
        "restricted_days": max_restricted_days,
        "available_recovery_tasks": available_tasks
    }))).into_response()
}

#[derive(Deserialize, Serialize)]
pub struct SyncPayload {
    pub client_device: String,
    pub activities: Option<Vec<Activity>>,
}

async fn sync_state(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    Json(payload): Json<SyncPayload>,
) -> impl IntoResponse {
    if !check_auth(&headers, &query, &state.pairing_token) {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }

    // Ingest any offline activities logged on mobile
    if let Some(acts) = payload.activities {
        for a in acts {
            let _ = state.db.insert_activity(&a);
        }
    }

    let commitments = state.db.list_commitments().unwrap_or_default();
    let penalties = state.db.get_active_penalties().unwrap_or_default();
    let save_days = state.db.get_available_save_days().unwrap_or_default();

    (StatusCode::OK, Json(json!({
        "synced": true,
        "commitments": commitments,
        "active_penalties": penalties,
        "available_save_days": save_days.len()
    }))).into_response()
}
