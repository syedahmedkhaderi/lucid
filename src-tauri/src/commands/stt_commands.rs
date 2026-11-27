use tauri::{command, AppHandle, Manager};

use crate::state::AppState;
use crate::stt::provider::{self, STTProviderType};

/// Switch the active STT provider.
///
/// `provider` is one of: "windows_native", "deepgram", "whisper_api", "azure_speech", "groq_whisper", "cartesia"
#[command]
pub async fn set_stt_provider(app: AppHandle, provider: String) -> Result<(), String> {
    let provider_type = STTProviderType::from_str(&provider)
        .ok_or_else(|| format!("Unknown STT provider: {}", provider))?;
    crate::privacy::ensure_stt_allowed(&provider_type)?;

    let state = app.state::<AppState>();

    // Initialize STTRouter if not yet created
    ensure_stt_initialized(&app, &state)?;

    let stt_arc = state
        .stt
        .as_ref()
        .ok_or("STT router not initialized")?
        .clone();

    // We need to use a tokio task since set_provider is async
    // but we're holding a std::sync::Mutex
    let mut router = stt_arc
        .lock()
        .map_err(|_| "STT state lock poisoned".to_string())?;

    // Load API key from credentials for the selected provider
    if let Some(ref cred_arc) = state.credentials {
        if let Ok(cred) = cred_arc.lock() {
            match provider_type {
                STTProviderType::Deepgram => {
                    if let Ok(Some(key)) = cred.get_key("deepgram") {
                        router.set_deepgram_api_key(&key);
                    }
                }
                STTProviderType::WhisperApi => {
                    load_whisper_api_credentials(&mut router, &cred);
                }
                STTProviderType::AzureSpeech => {
                    if let Ok(Some(key)) = cred.get_key("azure_speech") {
                        router.set_azure_speech_key(&key);
                    }
                    if let Ok(Some(region)) = cred.get_key("azure_speech_region") {
                        router.set_azure_speech_region(&region);
                    }
                }
                STTProviderType::AzureMaiStreaming => {
                    // Per-party routing reads these credentials directly when
                    // it creates each provider; do not cache the MAI key here.
                }
                STTProviderType::GroqWhisper => {
                    if let Ok(Some(key)) = cred.get_key("groq_whisper") {
                        router.set_groq_whisper_api_key(&key);
                    }
                }
                STTProviderType::Cartesia => {
                    if let Ok(Some(key)) = cred.get_key("cartesia") {
                        router.set_cartesia_api_key(&key);
                    }
                }
                STTProviderType::GeminiLive => {
                    if let Ok(Some(key)) = cred.get_key("gemini_transcribe_live") {
                        router.set_gemini_live_api_key(&key);
                    }
                }
                STTProviderType::WindowsNative | STTProviderType::WhisperCpp => {
                    // No API key needed for local providers
                }
                STTProviderType::WebSpeech => {
                    // Frontend-only — no API key needed
                }
                STTProviderType::SherpaOnnx
                | STTProviderType::OrtStreaming
                | STTProviderType::ParakeetTdt => {
                    // Local providers — no API key needed
                }
            }
        }
    }

    // For async operations on the router, we use block_in_place
    // since the Mutex is std::sync, not tokio
    tokio::task::block_in_place(|| {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { router.set_provider(provider_type).await })
    })?;

    log::info!("STT provider set to: {}", provider);
    Ok(())
}

/// Test whether a given STT provider is available and configured correctly.
#[command]
pub async fn test_stt_connection(
    app: AppHandle,
    provider: String,
    api_key: Option<String>,
    replace_saved_key: bool,
    region: Option<String>,
) -> Result<bool, String> {
    let provider_type = STTProviderType::from_str(&provider)
        .ok_or_else(|| format!("Unknown STT provider: {}", provider))?;
    crate::privacy::ensure_stt_allowed(&provider_type)?;

    let state = app.state::<AppState>();

    // MAI's test opens a short-lived SDK bridge session. Read its credentials
    // fresh from secure storage and avoid holding the blocking STTRouter mutex
    // across the asynchronous SDK probe.
    if provider_type == STTProviderType::AzureMaiStreaming {
        let (key, region) = get_azure_mai_credentials(&state)?;
        return crate::stt::azure_mai_streaming::test_azure_mai_connection(app, key, region).await;
    }

    // Initialize STTRouter if needed
    ensure_stt_initialized(&app, &state)?;

    let stt_arc = state
        .stt
        .as_ref()
        .ok_or("STT router not initialized")?
        .clone();

    let mut router = stt_arc
        .lock()
        .map_err(|_| "STT state lock poisoned".to_string())?;

    // Load saved API keys before testing. A typed replacement is only held
    // for the duration of this probe and is written after a successful test.
    if let Some(ref cred_arc) = state.credentials {
        if let Ok(cred) = cred_arc.lock() {
            match provider_type {
                STTProviderType::Deepgram => {
                    if let Ok(Some(key)) = cred.get_key("deepgram") {
                        router.set_deepgram_api_key(&key);
                    }
                }
                STTProviderType::WhisperApi => {
                    load_whisper_api_credentials(&mut router, &cred);
                }
                STTProviderType::AzureSpeech => {
                    if let Ok(Some(key)) = cred.get_key("azure_speech") {
                        router.set_azure_speech_key(&key);
                    }
                    if let Ok(Some(region)) = cred.get_key("azure_speech_region") {
                        router.set_azure_speech_region(&region);
                    }
                }
                STTProviderType::AzureMaiStreaming => {
                    // Handled above without holding the router lock.
                }
                STTProviderType::GroqWhisper => {
                    if let Ok(Some(key)) = cred.get_key("groq_whisper") {
                        router.set_groq_whisper_api_key(&key);
                    }
                }
                STTProviderType::Cartesia => {
                    if let Ok(Some(key)) = cred.get_key("cartesia") {
                        router.set_cartesia_api_key(&key);
                    }
                }
                STTProviderType::GeminiLive => {
                    if let Ok(Some(key)) = cred.get_key("gemini_transcribe_live") {
                        router.set_gemini_live_api_key(&key);
                    }
                }
                STTProviderType::WindowsNative | STTProviderType::WhisperCpp => {}
                STTProviderType::WebSpeech => {}
                STTProviderType::SherpaOnnx
                | STTProviderType::OrtStreaming
                | STTProviderType::ParakeetTdt => {
                    // Local providers — no API key needed
                }
            }
        }
    }

    if replace_saved_key {
        let typed_key = api_key.as_deref().unwrap_or("");
        match provider_type {
            STTProviderType::Deepgram => router.set_deepgram_api_key(typed_key),
            STTProviderType::WhisperApi => {
                if let Some(ref cred_arc) = state.credentials {
                    if let Ok(cred) = cred_arc.lock() {
                        load_whisper_api_credentials_with_override(
                            &mut router,
                            &cred,
                            Some(typed_key),
                            true,
                        );
                    }
                }
            }
            STTProviderType::AzureSpeech => router.set_azure_speech_key(typed_key),
            STTProviderType::GroqWhisper => router.set_groq_whisper_api_key(typed_key),
            STTProviderType::Cartesia => router.set_cartesia_api_key(typed_key),
            STTProviderType::GeminiLive => router.set_gemini_live_api_key(typed_key),
            STTProviderType::AzureMaiStreaming
            | STTProviderType::WindowsNative
            | STTProviderType::WhisperCpp
            | STTProviderType::WebSpeech
            | STTProviderType::SherpaOnnx
            | STTProviderType::OrtStreaming
            | STTProviderType::ParakeetTdt => {}
        }
    }
    if provider_type == STTProviderType::AzureSpeech {
        if let Some(region) = region.as_deref().filter(|value| !value.trim().is_empty()) {
            router.set_azure_speech_region(region.trim());
        }
    }

    let result = tokio::task::block_in_place(|| {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { router.test_provider_connection(&provider_type).await })
    });

    if !matches!(result, Ok(true)) && replace_saved_key {
        if let Some(credentials) = state.credentials.as_ref() {
            if let Ok(credentials) = credentials.lock() {
                restore_test_provider_credentials(&mut router, &credentials, provider_type.clone());
            }
        }
    }
    let result = result?;

    if result {
        if replace_saved_key {
            let slot = match provider_type {
                STTProviderType::Deepgram => Some("deepgram"),
                STTProviderType::WhisperApi => {
                    let credentials = state.credentials.as_ref();
                    credentials.and_then(|credentials| {
                        let credentials = credentials.lock().ok()?;
                        let base = credentials
                            .get_key(crate::stt::whisper_api::BASE_URL_CREDENTIAL_KEY)
                            .ok()
                            .flatten();
                        let model = credentials
                            .get_key(crate::stt::whisper_api::MODEL_CREDENTIAL_KEY)
                            .ok()
                            .flatten();
                        let (base, _) = crate::stt::whisper_api::resolve_endpoint(
                            base.as_deref(),
                            model.as_deref(),
                        )
                        .ok()?;
                        Some(if base == crate::stt::whisper_api::DEFAULT_BASE_URL {
                            crate::stt::whisper_api::OPENAI_KEY_CREDENTIAL_KEY
                        } else {
                            crate::stt::whisper_api::CUSTOM_KEY_CREDENTIAL_KEY
                        })
                    })
                }
                STTProviderType::AzureSpeech => Some("azure_speech"),
                STTProviderType::GroqWhisper => Some("groq_whisper"),
                STTProviderType::Cartesia => Some("cartesia"),
                STTProviderType::GeminiLive => Some("gemini_transcribe_live"),
                _ => None,
            };
            if let (Some(slot), Some(credentials)) = (slot, state.credentials.as_ref()) {
                let credentials = credentials
                    .lock()
                    .map_err(|_| "Credential store lock poisoned".to_string())?;
                if let Some(key) = api_key.as_deref().filter(|key| !key.trim().is_empty()) {
                    if let Err(error) = credentials.store_key(slot, key.trim()) {
                        restore_test_provider_credentials(&mut router, &credentials, provider_type);
                        return Err(error);
                    }
                } else {
                    if let Err(error) = credentials.delete_key(slot) {
                        restore_test_provider_credentials(&mut router, &credentials, provider_type);
                        return Err(error);
                    }
                }
            }
        }
        if provider_type == STTProviderType::AzureSpeech {
            if let (Some(region), Some(credentials)) =
                (region.as_deref(), state.credentials.as_ref())
            {
                let credentials = credentials
                    .lock()
                    .map_err(|_| "Credential store lock poisoned".to_string())?;
                credentials.store_key("azure_speech_region", region.trim())?;
            }
        }
    }

    Ok(result)
}

/// Load the Whisper API settings into the router: the OpenAI key, the optional
/// key for a custom endpoint, and the optional custom endpoint and model.
/// Missing values clear the cached ones so a removed value does not linger.
fn load_whisper_api_credentials(
    router: &mut crate::stt::STTRouter,
    cred: &crate::credentials::CredentialManager,
) {
    load_whisper_api_credentials_with_override(router, cred, None, false);
}

fn load_whisper_api_credentials_with_override(
    router: &mut crate::stt::STTRouter,
    cred: &crate::credentials::CredentialManager,
    replacement: Option<&str>,
    replace_saved_key: bool,
) {
    use crate::stt::whisper_api as w;
    let get = |key: &str| {
        cred.get_key(key)
            .ok()
            .flatten()
            .filter(|v| !v.trim().is_empty())
    };
    let mut openai_key = get(w::OPENAI_KEY_CREDENTIAL_KEY);
    let mut custom_key = get(w::CUSTOM_KEY_CREDENTIAL_KEY);
    let base_url = get(w::BASE_URL_CREDENTIAL_KEY);
    let model = get(w::MODEL_CREDENTIAL_KEY);
    if replace_saved_key {
        let is_custom = w::resolve_endpoint(base_url.as_deref(), model.as_deref())
            .map(|(base, _)| base != w::DEFAULT_BASE_URL)
            .unwrap_or(false);
        let replacement = replacement
            .filter(|key| !key.trim().is_empty())
            .map(str::to_string);
        if is_custom {
            custom_key = replacement;
        } else {
            openai_key = replacement;
        }
    }
    router.set_whisper_api_config(
        openai_key.as_deref(),
        custom_key.as_deref(),
        base_url.as_deref(),
        model.as_deref(),
    );
}

fn restore_test_provider_credentials(
    router: &mut crate::stt::STTRouter,
    cred: &crate::credentials::CredentialManager,
    provider: STTProviderType,
) {
    let get = |slot: &str| cred.get_key(slot).ok().flatten();
    match provider {
        STTProviderType::Deepgram => {
            router.set_deepgram_api_key(get("deepgram").as_deref().unwrap_or(""));
        }
        STTProviderType::WhisperApi => load_whisper_api_credentials(router, cred),
        STTProviderType::AzureSpeech => {
            router.set_azure_speech_key(get("azure_speech").as_deref().unwrap_or(""));
            router.set_azure_speech_region(get("azure_speech_region").as_deref().unwrap_or(""));
        }
        STTProviderType::GroqWhisper => {
            router.set_groq_whisper_api_key(get("groq_whisper").as_deref().unwrap_or(""));
        }
        STTProviderType::Cartesia => {
            router.set_cartesia_api_key(get("cartesia").as_deref().unwrap_or(""));
        }
        STTProviderType::GeminiLive => {
            router.set_gemini_live_api_key(get("gemini_transcribe_live").as_deref().unwrap_or(""));
        }
        STTProviderType::AzureMaiStreaming
        | STTProviderType::WindowsNative
        | STTProviderType::WhisperCpp
        | STTProviderType::WebSpeech
        | STTProviderType::SherpaOnnx
        | STTProviderType::OrtStreaming
        | STTProviderType::ParakeetTdt => {}
    }
}

/// Read Azure MAI credentials directly from secure storage. There is
/// intentionally no environment-variable or STTRouter cache fallback.
fn get_azure_mai_credentials(state: &AppState) -> Result<(String, String), String> {
    let credentials = state
        .credentials
        .as_ref()
        .ok_or("No credential store available for Azure MAI Streaming")?;
    let credentials = credentials
        .lock()
        .map_err(|_| "Credential store lock poisoned".to_string())?;
    let key = credentials
        .get_key(crate::stt::azure_mai_streaming::ID)
        .map_err(|e| format!("Failed to read Azure MAI Streaming key: {}", e))?
        .ok_or("No Azure MAI Streaming key configured")?;
    let region = credentials
        .get_key(crate::stt::azure_mai_streaming::REGION_KEY)
        .map_err(|e| format!("Failed to read Azure MAI Streaming region: {}", e))?
        .ok_or("No Azure MAI Streaming region configured")?;
    Ok((key, region))
}

/// Update whisper.cpp dual-pass configuration. Takes effect immediately,
/// even during an active meeting.
#[command]
pub async fn update_whisper_dual_pass_config(
    app: AppHandle,
    short_chunk_secs: f32,
    long_chunk_secs: f32,
    pause_secs: f32,
) -> Result<(), String> {
    use crate::stt::provider::DualPassConfig;

    let new_config = DualPassConfig {
        short_chunk_secs: short_chunk_secs.clamp(0.3, 5.0),
        long_chunk_secs: long_chunk_secs.clamp(1.0, 10.0),
        pause_secs: pause_secs.clamp(0.3, 5.0),
    };

    // Write to the shared config — WhisperCppSTT reads this on each feed_audio call.
    let config_arc = app.state::<AppState>().whisper_config.clone();
    let result = {
        match config_arc.write() {
            Ok(mut cfg) => {
                *cfg = new_config;
                log::info!(
                    "Whisper dual-pass config updated: fast={}s, correction={}s, pause={}s",
                    cfg.short_chunk_secs,
                    cfg.long_chunk_secs,
                    cfg.pause_secs
                );
                Ok(())
            }
            Err(_) => Err("Config lock poisoned".to_string()),
        }
    };
    result
}

/// List available STT providers with their metadata.
#[command]
pub async fn get_available_stt_providers() -> Result<String, String> {
    let providers = provider::list_available_providers();
    serde_json::to_string(&providers).map_err(|e| format!("Failed to serialize providers: {}", e))
}

/// Ensure the STTRouter is initialized in the app state.
/// This is a helper that initializes lazily on first use.
fn ensure_stt_initialized(_app: &AppHandle, state: &AppState) -> Result<(), String> {
    // The AppState.stt field is Option<Arc<Mutex<STTRouter>>>.
    // Since AppState is behind Tauri's managed state (immutable reference),
    // we can't directly mutate it here. The STTRouter should be initialized
    // in lib.rs setup. For now, if it's None, we log a warning.
    //
    // In practice, lib.rs should initialize it during setup just like LLMRouter.
    if state.stt.is_none() {
        log::warn!("STT router not initialized — it should be initialized in app setup");
        return Err("STT router not initialized. Please restart the application.".to_string());
    }
    Ok(())
}

/// Update the Deepgram model/feature configuration.
/// The config takes effect on the next Deepgram connection (not mid-stream).
#[command]
pub async fn update_deepgram_config(app: AppHandle, config_json: String) -> Result<(), String> {
    let config: crate::stt::deepgram::DeepgramConfig = serde_json::from_str(&config_json)
        .map_err(|e| format!("Invalid Deepgram config JSON: {}", e))?;

    let state = app.state::<AppState>();
    if let Some(ref stt_arc) = state.stt {
        let mut router = stt_arc
            .lock()
            .map_err(|_| "STT state lock poisoned".to_string())?;
        router.set_deepgram_config(config);
        log::info!("Deepgram config updated via IPC");
    }
    Ok(())
}

/// Update the Groq Whisper configuration.
/// Takes effect immediately — the running provider reads the shared config
/// on each API call, so settings changes apply to the very next batch.
#[command]
pub async fn update_groq_config(app: AppHandle, config_json: String) -> Result<(), String> {
    let config: crate::stt::groq_whisper::GroqConfig = serde_json::from_str(&config_json)
        .map_err(|e| format!("Invalid Groq config JSON: {}", e))?;

    let state = app.state::<AppState>();

    // Preserve the router's current language — config updates must not clobber it.
    let current_language = if let Some(ref stt_arc) = state.stt {
        let router = stt_arc
            .lock()
            .map_err(|_| "STT state lock poisoned".to_string())?;
        router.language.clone()
    } else {
        config.language.clone()
    };

    // Update the shared config Arc — running providers read this on each API call.
    // Apply the preserved language so the shared Arc stays in sync with the router.
    {
        let mut cfg = state
            .shared_groq_config
            .write()
            .map_err(|_| "Groq config lock poisoned".to_string())?;
        *cfg = config.clone();
        cfg.language = current_language.clone();
    }

    // Also update the STTRouter's copy (for new provider creation)
    if let Some(ref stt_arc) = state.stt {
        let mut router = stt_arc
            .lock()
            .map_err(|_| "STT state lock poisoned".to_string())?;
        let mut config_with_lang = config;
        config_with_lang.language = current_language;
        router.set_groq_config(config_with_lang);
    }

    log::info!("Groq config updated via IPC (applies immediately to running provider)");
    Ok(())
}

/// Keep the recognition language set to English for all STT providers.
#[command]
pub async fn set_stt_language(app: AppHandle, language: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    // Keep old clients and saved preferences from enabling non-English meeting
    // transcription. The UI no longer exposes a language selector.
    let _requested_language = language;
    let language = "en-US".to_string();

    // Keep shared_groq_config in sync — Groq reads it on every API call via current_config().
    {
        let mut cfg = state
            .shared_groq_config
            .write()
            .map_err(|_| "Groq config lock poisoned".to_string())?;
        cfg.language = language.clone();
    }

    if let Some(ref stt_arc) = state.stt {
        let mut router = stt_arc
            .lock()
            .map_err(|_| "STT state lock poisoned".to_string())?;
        router.set_language(&language);
        log::info!("STT language set to: {}", language);
    }
    Ok(())
}

/// Set the universal pause threshold for transcript line-breaking.
/// Takes effect immediately for system audio STT (the spawned task reads
/// the atomic value on each result). Value is in milliseconds.
#[command]
pub async fn set_pause_threshold(app: AppHandle, ms: u64) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    let state = app.state::<AppState>();
    state.pause_threshold_ms.store(ms, Ordering::Relaxed);
    log::info!("Pause threshold updated to {}ms", ms);
    Ok(())
}

/// Get the current pause threshold (ms).
#[command]
pub async fn get_pause_threshold(app: AppHandle) -> Result<u64, String> {
    use std::sync::atomic::Ordering;
    let state = app.state::<AppState>();
    Ok(state.pause_threshold_ms.load(Ordering::Relaxed))
}

/// Estimate Deepgram cost for a given meeting duration.
/// Returns JSON: { "cost_usd": 0.46, "streams": 2, "rate_per_min": 0.0043 }
#[command]
pub async fn estimate_deepgram_cost(duration_minutes: f32) -> Result<String, String> {
    let streams = 2u32; // typically You + Them
    let (cost, rate) = crate::stt::deepgram::estimate_cost(duration_minutes, streams);
    let result = serde_json::json!({
        "cost_usd": (cost * 100.0).round() / 100.0,
        "streams": streams,
        "rate_per_min": rate,
    });
    serde_json::to_string(&result).map_err(|e| format!("Failed to serialize cost estimate: {}", e))
}
