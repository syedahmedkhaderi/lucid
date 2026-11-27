use tauri::{command, State};

use crate::db::credentials::{self, CredentialProfile};
use crate::llm::{create_provider, ProviderConfig};
use crate::state::AppState;

#[command]
pub async fn store_api_key(
    provider: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let cred_mgr = state
        .credentials
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let cred = cred_mgr
        .lock()
        .map_err(|e| format!("Failed to lock credential manager: {}", e))?;

    cred.store_key(&provider, &key)
}

/// Read a non-secret provider setting. Secret credential slots are never
/// returned over IPC; this allowlist is limited to region, endpoint, and model.
#[command]
pub async fn get_credential_setting(
    setting: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    if !is_readable_credential_setting(&setting) {
        return Err("That credential slot is not a readable setting".into());
    }
    let cred_mgr = state
        .credentials
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let cred = cred_mgr
        .lock()
        .map_err(|e| format!("Failed to lock credential manager: {}", e))?;

    cred.get_key(&setting)
}

fn is_readable_credential_setting(setting: &str) -> bool {
    matches!(
        setting,
        "translation_microsoft_region"
            | "azure_speech_region"
            | crate::stt::azure_mai_streaming::REGION_KEY
            | crate::stt::whisper_api::BASE_URL_CREDENTIAL_KEY
            | crate::stt::whisper_api::MODEL_CREDENTIAL_KEY
    )
}

#[command]
pub async fn delete_api_key(provider: String, state: State<'_, AppState>) -> Result<(), String> {
    let cred_mgr = state
        .credentials
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let cred = cred_mgr
        .lock()
        .map_err(|e| format!("Failed to lock credential manager: {}", e))?;

    cred.delete_key(&provider)
}

#[command]
pub async fn has_api_key(provider: String, state: State<'_, AppState>) -> Result<bool, String> {
    let cred_mgr = state
        .credentials
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let cred = cred_mgr
        .lock()
        .map_err(|e| format!("Failed to lock credential manager: {}", e))?;

    cred.has_key(&provider)
}

/// Validate a provider key and store it as an independently named OS-keychain
/// profile. Profile metadata is stored in SQLite; the key is never persisted
/// there or returned to the frontend.
#[command]
pub async fn add_credential_profile(
    provider: String,
    key: String,
    label: String,
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<CredentialProfile, String> {
    let provider = provider.trim().to_lowercase();
    let allowed = matches!(
        provider.as_str(),
        "openai" | "anthropic" | "groq" | "gemini" | "openrouter"
    );
    if !allowed {
        return Err("Choose a supported cloud provider for this credential profile".into());
    }
    let key = key.trim();
    if key.is_empty() || key.len() > 4096 || key.chars().any(char::is_control) {
        return Err("Enter a valid API key value".into());
    }
    let label = label.trim();
    if label.is_empty() || label.len() > 80 || label.chars().any(char::is_control) {
        return Err("Enter a profile label of 1 to 80 characters".into());
    }
    let project_id = project_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if project_id
        .as_ref()
        .is_some_and(|value| value.len() > 128 || value.chars().any(char::is_control))
    {
        return Err("Project association must be 128 characters or fewer".into());
    }

    let profiles = {
        let db = state
            .database
            .as_ref()
            .ok_or_else(|| "Database not initialized".to_string())?;
        let db = db
            .lock()
            .map_err(|e| format!("Failed to lock database: {e}"))?;
        credentials::list(db.connection(), None)?
    };
    // Compare within Rust only. No key fingerprints or raw key values are
    // stored as metadata; even imported duplicates are rejected before use.
    {
        let manager = state
            .credentials
            .as_ref()
            .ok_or_else(|| "Credential manager not initialized".to_string())?;
        let manager = manager
            .lock()
            .map_err(|e| format!("Failed to lock credential manager: {e}"))?;
        for profile in &profiles {
            if manager.get_profile_key(&profile.id)?.as_deref() == Some(key) {
                return Err(format!("This key is already saved as {}", profile.label));
            }
        }
        for known_provider in ["openai", "anthropic", "groq", "gemini", "openrouter"] {
            if manager.get_key(known_provider)?.as_deref() == Some(key) {
                return Err(format!(
                    "This key is already configured for {}",
                    known_provider
                ));
            }
        }
    }

    let config = ProviderConfig {
        provider_type: provider.clone(),
        api_key: Some(key.to_string()),
        base_url: None,
        auth_type: None,
        auth_value: None,
        auth_header: None,
        thinking: Default::default(),
        extra_request_json: None,
    };
    let provider_client = create_provider(&config).map_err(|e| {
        format!(
            "Could not prepare the provider check: {}",
            e.to_string().replace(key, "[redacted]")
        )
    })?;
    let test_result = provider_client.test_connection().await.map_err(|e| {
        format!(
            "Credential check failed: {}",
            e.to_string().replace(key, "[redacted]")
        )
    })?;
    if !test_result {
        return Err(
            "The provider did not accept this key. The existing credential remains unchanged."
                .into(),
        );
    }

    let id = uuid::Uuid::new_v4().to_string();
    // Recheck under the same DB -> credential-store lock order used by profile
    // activation/removal. That closes the duplicate-import race across windows.
    let db = state
        .database
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let db = db
        .lock()
        .map_err(|e| format!("Failed to lock database: {e}"))?;
    let profiles = credentials::list(db.connection(), None)?;
    let manager = state
        .credentials
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;
    let manager = manager
        .lock()
        .map_err(|e| format!("Failed to lock credential manager: {e}"))?;
    for profile in &profiles {
        if manager.get_profile_key(&profile.id)?.as_deref() == Some(key) {
            return Err(format!("This key is already saved as {}", profile.label));
        }
    }
    for known_provider in ["openai", "anthropic", "groq", "gemini", "openrouter"] {
        if manager.get_key(known_provider)?.as_deref() == Some(key) {
            return Err(format!(
                "This key is already configured for {}",
                known_provider
            ));
        }
    }
    manager.store_profile_key(&id, key)?;
    if let Err(error) = credentials::insert(
        db.connection(),
        &id,
        &provider,
        label,
        project_id.as_deref(),
        key,
        chrono::Utc::now().timestamp_millis(),
    ) {
        let _ = manager.delete_profile_key(&id);
        return Err(error);
    }
    credentials::find(db.connection(), &id)?.ok_or_else(|| "Saved profile could not be read".into())
}

#[command]
pub fn list_credential_profiles(
    provider: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<CredentialProfile>, String> {
    let db = state
        .database
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let db = db
        .lock()
        .map_err(|e| format!("Failed to lock database: {e}"))?;
    credentials::list(db.connection(), provider.as_deref())
}

/// A deliberate user action switches one provider to this validated profile.
/// Quota/rate-limit errors never invoke this command or rotate credentials.
#[command]
pub fn activate_credential_profile(
    id: String,
    state: State<'_, AppState>,
) -> Result<CredentialProfile, String> {
    let db = state
        .database
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let db = db
        .lock()
        .map_err(|e| format!("Failed to lock database: {e}"))?;
    let profile = credentials::find(db.connection(), &id)?
        .ok_or_else(|| "Credential profile not found".to_string())?;
    if !profile.enabled {
        return Err("Enable this credential before making it active".into());
    }
    let manager = state
        .credentials
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;
    let manager = manager
        .lock()
        .map_err(|e| format!("Failed to lock credential manager: {e}"))?;
    let key = manager.get_profile_key(&id)?.ok_or_else(|| {
        "The saved key is missing from the operating system credential store".to_string()
    })?;
    let previous = manager.get_key(&profile.provider)?;
    manager.store_key(&profile.provider, &key)?;
    match credentials::set_active(db.connection(), &id) {
        Ok(profile) => Ok(profile),
        Err(error) => {
            match previous {
                Some(old) => {
                    let _ = manager.store_key(&profile.provider, &old);
                }
                None => {
                    let _ = manager.delete_key(&profile.provider);
                }
            }
            Err(error)
        }
    }
}

#[command]
pub fn set_credential_profile_enabled(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<CredentialProfile, String> {
    let db = state
        .database
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let db = db
        .lock()
        .map_err(|e| format!("Failed to lock database: {e}"))?;
    credentials::set_enabled(db.connection(), &id, enabled)
}

#[command]
pub fn delete_credential_profile(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state
        .database
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let db = db
        .lock()
        .map_err(|e| format!("Failed to lock database: {e}"))?;
    let profile = credentials::find(db.connection(), &id)?
        .ok_or_else(|| "Credential profile not found".to_string())?;
    if profile.active {
        return Err("Activate another credential before removing the active one".into());
    }
    let manager = state
        .credentials
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;
    let manager = manager
        .lock()
        .map_err(|e| format!("Failed to lock credential manager: {e}"))?;
    let saved = manager.get_profile_key(&id)?;
    manager.delete_profile_key(&id)?;
    if let Err(error) = credentials::delete(db.connection(), &id) {
        if let Some(key) = saved {
            let _ = manager.store_profile_key(&id, &key);
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod readable_setting_tests {
    use super::is_readable_credential_setting;

    #[test]
    fn only_non_secret_provider_settings_are_readable() {
        for setting in [
            "translation_microsoft_region",
            "azure_speech_region",
            crate::stt::azure_mai_streaming::REGION_KEY,
            crate::stt::whisper_api::BASE_URL_CREDENTIAL_KEY,
            crate::stt::whisper_api::MODEL_CREDENTIAL_KEY,
        ] {
            assert!(is_readable_credential_setting(setting));
        }
        for secret in [
            "openai",
            "gemini",
            "translation_google",
            crate::stt::azure_mai_streaming::ID,
            crate::stt::whisper_api::OPENAI_KEY_CREDENTIAL_KEY,
            crate::stt::whisper_api::CUSTOM_KEY_CREDENTIAL_KEY,
        ] {
            assert!(!is_readable_credential_setting(secret));
        }
    }
}
