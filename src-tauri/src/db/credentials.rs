//! Non-secret metadata for user-managed API credential profiles.
//!
//! Credential material itself always lives in the OS credential store. This
//! table deliberately contains no API-key or token column.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialProfile {
    pub id: String,
    pub provider: String,
    pub label: String,
    pub project_id: Option<String>,
    /// Last four characters only; the full credential is never returned.
    pub key_suffix: String,
    pub enabled: bool,
    pub active: bool,
    pub verified_at: i64,
    pub last_used_at: Option<i64>,
    /// Safe, user-facing status text. Never store raw provider response bodies.
    pub last_error: Option<String>,
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS credential_profiles (
            id TEXT PRIMARY KEY NOT NULL,
            provider TEXT NOT NULL,
            label TEXT NOT NULL,
            project_id TEXT,
            key_suffix TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
            active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0, 1)),
            verified_at INTEGER NOT NULL,
            last_used_at INTEGER,
            last_error TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_credential_profiles_provider
            ON credential_profiles(provider, created_at);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_credential_profiles_one_active
            ON credential_profiles(provider) WHERE active = 1;",
    )
}

pub fn list(conn: &Connection, provider: Option<&str>) -> Result<Vec<CredentialProfile>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, provider, label, project_id, key_suffix, enabled, active,
                    verified_at, last_used_at, last_error
             FROM credential_profiles
             WHERE (?1 IS NULL OR provider = ?1)
             ORDER BY provider, active DESC, created_at, id",
        )
        .map_err(|e| format!("Could not read credential profiles: {e}"))?;
    let rows = stmt
        .query_map([provider], |row| {
            Ok(CredentialProfile {
                id: row.get(0)?,
                provider: row.get(1)?,
                label: row.get(2)?,
                project_id: row.get(3)?,
                key_suffix: row.get(4)?,
                enabled: row.get::<_, i64>(5)? != 0,
                active: row.get::<_, i64>(6)? != 0,
                verified_at: row.get(7)?,
                last_used_at: row.get(8)?,
                last_error: row.get(9)?,
            })
        })
        .map_err(|e| format!("Could not read credential profiles: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Could not read credential profiles: {e}"))
}

pub fn find(conn: &Connection, id: &str) -> Result<Option<CredentialProfile>, String> {
    Ok(list(conn, None)?
        .into_iter()
        .find(|profile| profile.id == id))
}

pub fn insert(
    conn: &Connection,
    id: &str,
    provider: &str,
    label: &str,
    project_id: Option<&str>,
    secret: &str,
    verified_at: i64,
) -> Result<(), String> {
    let suffix: String = secret
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let now = now_ms();
    conn.execute(
        "INSERT INTO credential_profiles
            (id, provider, label, project_id, key_suffix, enabled, active,
             verified_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, 0, ?6, ?7, ?7)",
        params![id, provider, label, project_id, suffix, verified_at, now],
    )
    .map(|_| ())
    .map_err(|e| format!("Could not save credential profile: {e}"))
}

pub fn set_active(conn: &Connection, id: &str) -> Result<CredentialProfile, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Could not update credential profile: {e}"))?;
    let (provider, enabled): (String, i64) = tx
        .query_row(
            "SELECT provider, enabled FROM credential_profiles WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| format!("Could not find credential profile: {e}"))?
        .ok_or_else(|| "Credential profile not found".to_string())?;
    if enabled == 0 {
        return Err("Enable this credential before making it active".into());
    }
    tx.execute(
        "UPDATE credential_profiles SET active = 0, updated_at = ?2 WHERE provider = ?1",
        params![provider, now_ms()],
    )
    .map_err(|e| format!("Could not update credential profile: {e}"))?;
    let now = now_ms();
    tx.execute(
        "UPDATE credential_profiles SET active = 1, updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )
    .map_err(|e| format!("Could not activate credential profile: {e}"))?;
    tx.commit()
        .map_err(|e| format!("Could not activate credential profile: {e}"))?;
    find(conn, id)?.ok_or_else(|| "Credential profile not found after activation".into())
}

pub fn set_enabled(
    conn: &Connection,
    id: &str,
    enabled: bool,
) -> Result<CredentialProfile, String> {
    if !enabled {
        let active: bool = conn
            .query_row(
                "SELECT active FROM credential_profiles WHERE id = ?1",
                [id],
                |row| row.get::<_, i64>(0).map(|value| value != 0),
            )
            .optional()
            .map_err(|e| format!("Could not read credential profile: {e}"))?
            .ok_or_else(|| "Credential profile not found".to_string())?;
        if active {
            return Err("Activate another credential before disabling the active one".into());
        }
    }
    let changed = conn
        .execute(
            "UPDATE credential_profiles SET enabled = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, enabled as i64, now_ms()],
        )
        .map_err(|e| format!("Could not update credential profile: {e}"))?;
    if changed == 0 {
        return Err("Credential profile not found".into());
    }
    find(conn, id)?.ok_or_else(|| "Credential profile not found after update".into())
}

pub fn delete(conn: &Connection, id: &str) -> Result<CredentialProfile, String> {
    let profile = find(conn, id)?.ok_or_else(|| "Credential profile not found".to_string())?;
    if profile.active {
        return Err("Activate another credential before removing the active one".into());
    }
    conn.execute("DELETE FROM credential_profiles WHERE id = ?1", [id])
        .map_err(|e| format!("Could not remove credential profile: {e}"))?;
    Ok(profile)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn add(conn: &Connection, id: &str, label: &str) {
        insert(
            conn,
            id,
            "gemini",
            label,
            Some("project-alpha"),
            "secret-1234",
            10,
        )
        .unwrap();
    }

    #[test]
    fn migration_has_no_secret_column_and_metadata_masks_the_key() {
        let conn = db();
        add(&conn, "one", "Primary");
        let profile = find(&conn, "one").unwrap().unwrap();
        assert_eq!(profile.key_suffix, "1234");
        let columns = conn
            .prepare("PRAGMA table_info(credential_profiles)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(!columns
            .iter()
            .any(|column| column.contains("secret") || column.contains("api_key")));
    }

    #[test]
    fn profiles_are_scoped_by_provider_and_activation_is_exclusive() {
        let conn = db();
        add(&conn, "first", "First");
        add(&conn, "second", "Second");
        insert(&conn, "openai", "openai", "OpenAI", None, "other-key", 10).unwrap();
        set_active(&conn, "first").unwrap();
        set_active(&conn, "second").unwrap();
        let profiles = list(&conn, Some("gemini")).unwrap();
        assert_eq!(profiles.len(), 2);
        assert!(
            !profiles
                .iter()
                .find(|profile| profile.id == "first")
                .unwrap()
                .active
        );
        assert!(
            profiles
                .iter()
                .find(|profile| profile.id == "second")
                .unwrap()
                .active
        );
        assert!(list(&conn, None)
            .unwrap()
            .iter()
            .any(|profile| profile.provider == "openai"));
    }

    #[test]
    fn active_profile_must_be_replaced_before_disable_or_delete() {
        let conn = db();
        add(&conn, "first", "First");
        set_active(&conn, "first").unwrap();
        assert!(set_enabled(&conn, "first", false)
            .unwrap_err()
            .contains("Activate another"));
        assert!(delete(&conn, "first")
            .unwrap_err()
            .contains("Activate another"));
    }
}
