use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use forgex_core::{
    Activity, ActivityCategory, Commitment, CommitmentStatus,
    ForgeError, ForgeEvent, ForgeEventPayload, PenaltyRecord, PenaltyStatus,
    RecurrenceRule, Result, SaveDay, SaveDayStatus,
};
use crate::schema::initialize_schema;

#[derive(Clone)]
pub struct ForgeDb {
    conn: Arc<Mutex<Connection>>,
}

impl ForgeDb {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| ForgeError::Storage(format!("Failed to open sqlite database: {}", e)))?;
        initialize_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| ForgeError::Storage(format!("Failed to open in-memory database: {}", e)))?;
        initialize_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // ----------------------------------------------------
    // Commitments
    // ----------------------------------------------------
    pub fn insert_commitment(&self, c: &Commitment) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let recurrence_json = serde_json::to_string(&c.recurrence)
            .map_err(|e| ForgeError::Serialization(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO commitments (
                id, title, description, importance, expected_effort,
                scheduled_start, scheduled_duration_mins, recurrence, category,
                status, actual_started_at, actual_ended_at, consecutive_skips,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
            params![
                c.id.to_string(),
                c.title,
                c.description,
                c.importance,
                c.expected_effort,
                c.scheduled_start.to_rfc3339(),
                c.scheduled_duration_mins,
                recurrence_json,
                c.category,
                c.status.to_string(),
                c.actual_started_at.map(|t| t.to_rfc3339()),
                c.actual_ended_at.map(|t| t.to_rfc3339()),
                c.consecutive_skips,
                c.created_at.to_rfc3339(),
                c.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| ForgeError::Storage(format!("Failed to insert commitment: {}", e)))?;

        Ok(())
    }

    pub fn update_commitment(&self, c: &Commitment) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let recurrence_json = serde_json::to_string(&c.recurrence)
            .map_err(|e| ForgeError::Serialization(e.to_string()))?;

        conn.execute(
            r#"
            UPDATE commitments SET
                title = ?1,
                description = ?2,
                importance = ?3,
                expected_effort = ?4,
                scheduled_start = ?5,
                scheduled_duration_mins = ?6,
                recurrence = ?7,
                category = ?8,
                status = ?9,
                actual_started_at = ?10,
                actual_ended_at = ?11,
                consecutive_skips = ?12,
                updated_at = ?13
            WHERE id = ?14
            "#,
            params![
                c.title,
                c.description,
                c.importance,
                c.expected_effort,
                c.scheduled_start.to_rfc3339(),
                c.scheduled_duration_mins,
                recurrence_json,
                c.category,
                c.status.to_string(),
                c.actual_started_at.map(|t| t.to_rfc3339()),
                c.actual_ended_at.map(|t| t.to_rfc3339()),
                c.consecutive_skips,
                c.updated_at.to_rfc3339(),
                c.id.to_string(),
            ],
        ).map_err(|e| ForgeError::Storage(format!("Failed to update commitment: {}", e)))?;

        Ok(())
    }

    pub fn get_commitment(&self, id: Uuid) -> Result<Option<Commitment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, title, description, importance, expected_effort,
                scheduled_start, scheduled_duration_mins, recurrence, category,
                status, actual_started_at, actual_ended_at, consecutive_skips,
                created_at, updated_at
            FROM commitments WHERE id = ?1
            "#,
        ).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let res = stmt.query_row(params![id.to_string()], |row| {
            Ok(Self::row_to_commitment(row))
        }).optional().map_err(|e| ForgeError::Storage(e.to_string()))?;

        match res {
            Some(res_c) => Ok(Some(res_c?)),
            None => Ok(None),
        }
    }

    pub fn list_commitments(&self) -> Result<Vec<Commitment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, title, description, importance, expected_effort,
                scheduled_start, scheduled_duration_mins, recurrence, category,
                status, actual_started_at, actual_ended_at, consecutive_skips,
                created_at, updated_at
            FROM commitments
            ORDER BY scheduled_start ASC
            "#,
        ).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            Ok(Self::row_to_commitment(row))
        }).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| ForgeError::Storage(e.to_string()))??);
        }
        Ok(list)
    }

    pub fn delete_commitment(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM commitments WHERE id = ?1", params![id.to_string()])
            .map_err(|e| ForgeError::Storage(e.to_string()))?;
        Ok(())
    }

    fn row_to_commitment(row: &rusqlite::Row) -> Result<Commitment> {
        let id_str: String = row.get(0).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let title: String = row.get(1).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let description: Option<String> = row.get(2).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let importance: u8 = row.get(3).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let expected_effort: u8 = row.get(4).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let scheduled_start_str: String = row.get(5).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let scheduled_duration_mins: u32 = row.get(6).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let recurrence_str: String = row.get(7).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let category: String = row.get(8).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let status_str: String = row.get(9).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let started_str: Option<String> = row.get(10).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let ended_str: Option<String> = row.get(11).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let consecutive_skips: u32 = row.get(12).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let created_at_str: String = row.get(13).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let updated_at_str: String = row.get(14).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let id = Uuid::parse_str(&id_str).map_err(|e| ForgeError::Validation(e.to_string()))?;
        let scheduled_start = DateTime::parse_from_rfc3339(&scheduled_start_str)
            .map_err(|e| ForgeError::Validation(e.to_string()))?
            .with_timezone(&Utc);
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| ForgeError::Validation(e.to_string()))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| ForgeError::Validation(e.to_string()))?
            .with_timezone(&Utc);
        let actual_started_at = started_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s).ok().map(|t| t.with_timezone(&Utc))
        });
        let actual_ended_at = ended_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s).ok().map(|t| t.with_timezone(&Utc))
        });
        let status = CommitmentStatus::from_str(&status_str)?;
        let recurrence: RecurrenceRule = serde_json::from_str(&recurrence_str)
            .unwrap_or(RecurrenceRule::Daily);

        Ok(Commitment {
            id,
            title,
            description,
            importance,
            expected_effort,
            scheduled_start,
            scheduled_duration_mins,
            recurrence,
            category,
            status,
            actual_started_at,
            actual_ended_at,
            consecutive_skips,
            created_at,
            updated_at,
        })
    }

    // ----------------------------------------------------
    // Activities
    // ----------------------------------------------------
    pub fn insert_activity(&self, a: &Activity) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO activities (
                id, device, application, domain_or_detail, category,
                started_at, duration_mins, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                a.id.to_string(),
                a.device,
                a.application,
                a.domain_or_detail,
                a.category.to_string(),
                a.started_at.to_rfc3339(),
                a.duration_mins,
                a.created_at.to_rfc3339(),
            ],
        ).map_err(|e| ForgeError::Storage(format!("Failed to insert activity: {}", e)))?;

        Ok(())
    }

    pub fn list_activities(&self) -> Result<Vec<Activity>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, device, application, domain_or_detail, category, started_at, duration_mins, created_at
            FROM activities
            ORDER BY started_at DESC
            "#,
        ).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            Ok(Self::row_to_activity(row))
        }).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| ForgeError::Storage(e.to_string()))??);
        }
        Ok(list)
    }

    pub fn list_activities_for_window(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Activity>> {
        let all = self.list_activities()?;
        Ok(all.into_iter().filter(|a| a.overlaps_window(start, end)).collect())
    }

    fn row_to_activity(row: &rusqlite::Row) -> Result<Activity> {
        let id_str: String = row.get(0).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let device: String = row.get(1).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let application: String = row.get(2).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let domain_or_detail: Option<String> = row.get(3).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let category_str: String = row.get(4).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let started_at_str: String = row.get(5).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let duration_mins: u32 = row.get(6).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let created_at_str: String = row.get(7).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let id = Uuid::parse_str(&id_str).map_err(|e| ForgeError::Validation(e.to_string()))?;
        let started_at = DateTime::parse_from_rfc3339(&started_at_str)
            .map_err(|e| ForgeError::Validation(e.to_string()))?
            .with_timezone(&Utc);
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| ForgeError::Validation(e.to_string()))?
            .with_timezone(&Utc);
        let category = ActivityCategory::from_str(&category_str)?;

        Ok(Activity {
            id,
            device,
            application,
            domain_or_detail,
            category,
            started_at,
            duration_mins,
            created_at,
        })
    }

    // ----------------------------------------------------
    // Penalties
    // ----------------------------------------------------
    pub fn insert_penalty(&self, p: &PenaltyRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO penalties (
                id, commitment_id, raw_penalty_mins, actual_penalty_mins,
                remaining_penalty_mins, restricted_days, avoidance_score,
                shielded_by_save_day, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                p.id.to_string(),
                p.commitment_id.to_string(),
                p.raw_penalty_mins,
                p.actual_penalty_mins,
                p.remaining_penalty_mins,
                p.restricted_days,
                p.avoidance_score,
                if p.shielded_by_save_day { 1 } else { 0 },
                p.status.to_string(),
                p.created_at.to_rfc3339(),
                p.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| ForgeError::Storage(format!("Failed to insert penalty: {}", e)))?;

        Ok(())
    }

    pub fn update_penalty(&self, p: &PenaltyRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            UPDATE penalties SET
                remaining_penalty_mins = ?1,
                restricted_days = ?2,
                status = ?3,
                updated_at = ?4
            WHERE id = ?5
            "#,
            params![
                p.remaining_penalty_mins,
                p.restricted_days,
                p.status.to_string(),
                p.updated_at.to_rfc3339(),
                p.id.to_string(),
            ],
        ).map_err(|e| ForgeError::Storage(format!("Failed to update penalty: {}", e)))?;

        Ok(())
    }

    pub fn list_penalties(&self) -> Result<Vec<PenaltyRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, commitment_id, raw_penalty_mins, actual_penalty_mins,
                remaining_penalty_mins, restricted_days, avoidance_score,
                shielded_by_save_day, status, created_at, updated_at
            FROM penalties
            ORDER BY created_at DESC
            "#,
        ).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            Ok(Self::row_to_penalty(row))
        }).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| ForgeError::Storage(e.to_string()))??);
        }
        Ok(list)
    }

    pub fn get_active_penalties(&self) -> Result<Vec<PenaltyRecord>> {
        let all = self.list_penalties()?;
        Ok(all.into_iter().filter(|p| p.status == PenaltyStatus::Active).collect())
    }

    fn row_to_penalty(row: &rusqlite::Row) -> Result<PenaltyRecord> {
        let id_str: String = row.get(0).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let comm_str: String = row.get(1).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let raw_penalty_mins: u32 = row.get(2).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let actual_penalty_mins: u32 = row.get(3).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let remaining_penalty_mins: u32 = row.get(4).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let restricted_days: u32 = row.get(5).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let avoidance_score: u32 = row.get(6).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let shielded_int: i32 = row.get(7).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let status_str: String = row.get(8).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let created_at_str: String = row.get(9).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let updated_at_str: String = row.get(10).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let id = Uuid::parse_str(&id_str).map_err(|e| ForgeError::Validation(e.to_string()))?;
        let commitment_id = Uuid::parse_str(&comm_str).map_err(|e| ForgeError::Validation(e.to_string()))?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| ForgeError::Validation(e.to_string()))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| ForgeError::Validation(e.to_string()))?
            .with_timezone(&Utc);

        let status = match status_str.to_uppercase().as_str() {
            "ACTIVE" => PenaltyStatus::Active,
            "CLEARED" => PenaltyStatus::Cleared,
            "EXPIRED" => PenaltyStatus::Expired,
            _ => PenaltyStatus::Active,
        };

        Ok(PenaltyRecord {
            id,
            commitment_id,
            raw_penalty_mins,
            actual_penalty_mins,
            remaining_penalty_mins,
            restricted_days,
            avoidance_score,
            shielded_by_save_day: shielded_int == 1,
            status,
            created_at,
            updated_at,
        })
    }

    // ----------------------------------------------------
    // Save Days
    // ----------------------------------------------------
    pub fn insert_save_day(&self, s: &SaveDay) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO save_days (id, earned_at, consumed_at, consumed_for_commitment_id, status)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                s.id.to_string(),
                s.earned_at.to_rfc3339(),
                s.consumed_at.map(|t| t.to_rfc3339()),
                s.consumed_for_commitment_id.map(|u| u.to_string()),
                s.status.to_string(),
            ],
        ).map_err(|e| ForgeError::Storage(format!("Failed to insert save day: {}", e)))?;

        Ok(())
    }

    pub fn list_save_days(&self) -> Result<Vec<SaveDay>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, earned_at, consumed_at, consumed_for_commitment_id, status
            FROM save_days
            ORDER BY earned_at ASC
            "#,
        ).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            Ok(Self::row_to_save_day(row))
        }).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| ForgeError::Storage(e.to_string()))??);
        }
        Ok(list)
    }

    pub fn get_available_save_days(&self) -> Result<Vec<SaveDay>> {
        let all = self.list_save_days()?;
        Ok(all.into_iter().filter(|s| s.status == SaveDayStatus::Available).collect())
    }

    pub fn consume_oldest_save_day(&self, commitment_id: Uuid, now: DateTime<Utc>) -> Result<Option<SaveDay>> {
        let mut available = self.get_available_save_days()?;
        if available.is_empty() {
            return Ok(None);
        }

        let mut oldest = available.remove(0);
        oldest.consume(commitment_id, now);

        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            UPDATE save_days SET
                consumed_at = ?1,
                consumed_for_commitment_id = ?2,
                status = ?3
            WHERE id = ?4
            "#,
            params![
                oldest.consumed_at.map(|t| t.to_rfc3339()),
                oldest.consumed_for_commitment_id.map(|u| u.to_string()),
                oldest.status.to_string(),
                oldest.id.to_string(),
            ],
        ).map_err(|e| ForgeError::Storage(format!("Failed to consume save day: {}", e)))?;

        Ok(Some(oldest))
    }

    fn row_to_save_day(row: &rusqlite::Row) -> Result<SaveDay> {
        let id_str: String = row.get(0).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let earned_at_str: String = row.get(1).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let consumed_at_str: Option<String> = row.get(2).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let comm_str: Option<String> = row.get(3).map_err(|e| ForgeError::Storage(e.to_string()))?;
        let status_str: String = row.get(4).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let id = Uuid::parse_str(&id_str).map_err(|e| ForgeError::Validation(e.to_string()))?;
        let earned_at = DateTime::parse_from_rfc3339(&earned_at_str)
            .map_err(|e| ForgeError::Validation(e.to_string()))?
            .with_timezone(&Utc);
        let consumed_at = consumed_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s).ok().map(|t| t.with_timezone(&Utc))
        });
        let consumed_for_commitment_id = comm_str.and_then(|s| Uuid::parse_str(&s).ok());
        let status = match status_str.to_uppercase().as_str() {
            "AVAILABLE" => SaveDayStatus::Available,
            "CONSUMED" => SaveDayStatus::Consumed,
            _ => SaveDayStatus::Available,
        };

        Ok(SaveDay {
            id,
            earned_at,
            consumed_at,
            consumed_for_commitment_id,
            status,
        })
    }

    // ----------------------------------------------------
    // Events Log
    // ----------------------------------------------------
    pub fn log_event(&self, event: &ForgeEvent) -> Result<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| ForgeError::Serialization(e.to_string()))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO events_log (id, timestamp, payload_json) VALUES (?1, ?2, ?3)",
            params![event.id.to_string(), event.timestamp.to_rfc3339(), payload_json],
        ).map_err(|e| ForgeError::Storage(e.to_string()))?;

        Ok(())
    }

    pub fn list_events(&self) -> Result<Vec<ForgeEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, timestamp, payload_json FROM events_log ORDER BY timestamp DESC LIMIT 50")
            .map_err(|e| ForgeError::Storage(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let ts_str: String = row.get(1)?;
            let json_str: String = row.get(2)?;
            Ok((id_str, ts_str, json_str))
        }).map_err(|e| ForgeError::Storage(e.to_string()))?;

        let mut events = Vec::new();
        for r in rows {
            let (id_s, ts_s, json_s) = r.map_err(|e| ForgeError::Storage(e.to_string()))?;
            let id = Uuid::parse_str(&id_s).map_err(|e| ForgeError::Validation(e.to_string()))?;
            let timestamp = DateTime::parse_from_rfc3339(&ts_s)
                .map_err(|e| ForgeError::Validation(e.to_string()))?
                .with_timezone(&Utc);
            let payload: ForgeEventPayload = serde_json::from_str(&json_s)
                .map_err(|e| ForgeError::Serialization(e.to_string()))?;
            events.push(ForgeEvent { id, timestamp, payload });
        }
        Ok(events)
    }

    // ----------------------------------------------------
    // System State / Key-Value
    // ----------------------------------------------------
    pub fn get_state(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM system_state WHERE key = ?1")
            .map_err(|e| ForgeError::Storage(e.to_string()))?;
        let res: Option<String> = stmt.query_row(params![key], |row| row.get(0))
            .optional().map_err(|e| ForgeError::Storage(e.to_string()))?;
        Ok(res)
    }

    pub fn set_state(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO system_state (key, value) VALUES (?1, ?2)",
            params![key, value],
        ).map_err(|e| ForgeError::Storage(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_in_memory_db_commitment_crud() {
        let db = ForgeDb::open_in_memory().unwrap();
        let now = Utc::now();
        let mut c = Commitment::new(
            "Read Architecture Book",
            4,
            3,
            now,
            60,
            RecurrenceRule::Daily,
            "Learning",
        ).unwrap();

        db.insert_commitment(&c).unwrap();
        let fetched = db.get_commitment(c.id).unwrap().unwrap();
        assert_eq!(fetched.title, "Read Architecture Book");
        assert_eq!(fetched.status, CommitmentStatus::Planned);

        c.complete(now + Duration::minutes(50)).unwrap();
        db.update_commitment(&c).unwrap();
        let updated = db.get_commitment(c.id).unwrap().unwrap();
        assert_eq!(updated.status, CommitmentStatus::Completed);
    }

    #[test]
    fn test_save_days_management() {
        let db = ForgeDb::open_in_memory().unwrap();
        let now = Utc::now();
        let save_day = SaveDay::new_earned(now);
        db.insert_save_day(&save_day).unwrap();

        let available = db.get_available_save_days().unwrap();
        assert_eq!(available.len(), 1);

        let comm_id = Uuid::new_v4();
        let consumed = db.consume_oldest_save_day(comm_id, now).unwrap().unwrap();
        assert_eq!(consumed.status, SaveDayStatus::Consumed);

        let available_after = db.get_available_save_days().unwrap();
        assert_eq!(available_after.len(), 0);
    }
}
