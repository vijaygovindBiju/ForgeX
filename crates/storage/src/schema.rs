use rusqlite::Connection;
use forgex_core::{ForgeError, Result};

pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS commitments (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    importance INTEGER NOT NULL,
    expected_effort INTEGER NOT NULL,
    scheduled_start TEXT NOT NULL,
    scheduled_duration_mins INTEGER NOT NULL,
    recurrence TEXT NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL,
    actual_started_at TEXT,
    actual_ended_at TEXT,
    consecutive_skips INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_commitments_status ON commitments(status);
CREATE INDEX IF NOT EXISTS idx_commitments_scheduled_start ON commitments(scheduled_start);

CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY,
    device TEXT NOT NULL,
    application TEXT NOT NULL,
    domain_or_detail TEXT,
    category TEXT NOT NULL,
    started_at TEXT NOT NULL,
    duration_mins INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_activities_started_at ON activities(started_at);
CREATE INDEX IF NOT EXISTS idx_activities_category ON activities(category);

CREATE TABLE IF NOT EXISTS penalties (
    id TEXT PRIMARY KEY,
    commitment_id TEXT NOT NULL,
    raw_penalty_mins INTEGER NOT NULL,
    actual_penalty_mins INTEGER NOT NULL,
    remaining_penalty_mins INTEGER NOT NULL,
    restricted_days INTEGER NOT NULL,
    avoidance_score INTEGER NOT NULL,
    shielded_by_save_day INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(commitment_id) REFERENCES commitments(id)
);

CREATE INDEX IF NOT EXISTS idx_penalties_status ON penalties(status);

CREATE TABLE IF NOT EXISTS save_days (
    id TEXT PRIMARY KEY,
    earned_at TEXT NOT NULL,
    consumed_at TEXT,
    consumed_for_commitment_id TEXT,
    status TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_save_days_status ON save_days(status);

CREATE TABLE IF NOT EXISTS events_log (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    payload_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS system_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| ForgeError::Storage(format!("Failed to initialize schema: {}", e)))?;
    Ok(())
}
