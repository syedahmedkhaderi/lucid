// Tauri commands for managing local STT model downloads.

use tauri::{command, AppHandle, Manager};

use crate::state::AppState;

const SUPPORTED_MODELS: &[(&str, &str)] = &[
    ("whisper_cpp", "large-v3-turbo"),
    ("parakeet_tdt", "parakeet-tdt-0.6b-v3-int8"),
];

fn is_supported_model(engine: &str, model_id: &str) -> bool {
    SUPPORTED_MODELS.contains(&(engine, model_id))
}

/// Report whether this machine is eligible to download and use the local speech models.
#[command]
pub async fn get_local_stt_capabilities() -> Result<String, String> {
    serde_json::to_string(&crate::stt::local_engines::capabilities::current())
        .map_err(|e| format!("Failed to serialize local STT capabilities: {e}"))
}

/// List all local STT engines with their models and download status.
#[command]
pub async fn list_local_stt_engines(app: AppHandle) -> Result<String, String> {
    let state = app.state::<AppState>();
    let model_mgr = state
        .model_manager
        .as_ref()
        .ok_or("Model manager not initialized")?;
    let mgr = model_mgr
        .lock()
        .map_err(|_| "Model manager lock poisoned".to_string())?;

    let capabilities = crate::stt::local_engines::capabilities::current();
    let engines = if capabilities.local_models_available {
        mgr.list_engines_with_status()
            .into_iter()
            .filter_map(|mut engine| {
                engine.models.retain(|model| {
                    is_supported_model(model.definition.engine, model.definition.model_id)
                });
                (!engine.models.is_empty()).then_some(engine)
            })
            .collect()
    } else {
        Vec::new()
    };
    serde_json::to_string(&engines)
        .map_err(|e| format!("Failed to serialize engine list: {}", e))
}

/// Start downloading a local STT model. Progress emitted via `model_download_progress` events.
#[command]
pub async fn download_local_stt_model(
    app: AppHandle,
    engine: String,
    model_id: String,
) -> Result<(), String> {
    if !crate::stt::local_engines::capabilities::current().local_models_available {
        return Err(
            "Local STT models require a Mac or a detected GPU. Use Groq Whisper on this device."
                .into(),
        );
    }
    if !is_supported_model(&engine, &model_id) {
        return Err(
            "Only Whisper Large v3 Turbo and NVIDIA Parakeet TDT 0.6B v3 are supported.".into(),
        );
    }
    let state = app.state::<AppState>();
    let model_mgr = state
        .model_manager
        .as_ref()
        .ok_or("Model manager not initialized")?;

    let mut mgr = model_mgr
        .lock()
        .map_err(|_| "Model manager lock poisoned".to_string())?;

    mgr.download_model(&engine, &model_id, app.clone())
}

/// Cancel an active model download.
#[command]
pub async fn cancel_model_download(
    app: AppHandle,
    engine: String,
    model_id: String,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let model_mgr = state
        .model_manager
        .as_ref()
        .ok_or("Model manager not initialized")?;

    let mut mgr = model_mgr
        .lock()
        .map_err(|_| "Model manager lock poisoned".to_string())?;

    mgr.cancel_download(&engine, &model_id);
    Ok(())
}

/// Delete a downloaded local STT model.
#[command]
pub async fn delete_local_stt_model(
    app: AppHandle,
    engine: String,
    model_id: String,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let model_mgr = state
        .model_manager
        .as_ref()
        .ok_or("Model manager not initialized")?;

    let mut mgr = model_mgr
        .lock()
        .map_err(|_| "Model manager lock poisoned".to_string())?;

    mgr.delete_model(&engine, &model_id)
}

#[cfg(test)]
mod tests {
    use super::is_supported_model;

    #[test]
    fn only_requested_local_models_are_downloadable() {
        assert!(is_supported_model("whisper_cpp", "large-v3-turbo"));
        assert!(is_supported_model("parakeet_tdt", "parakeet-tdt-0.6b-v3-int8"));
        assert!(!is_supported_model("whisper_cpp", "base"));
        assert!(!is_supported_model("parakeet_tdt", "parakeet-tdt-ctc-110m-int8"));
        assert!(!is_supported_model("sherpa_onnx", "streaming-zipformer-en-20M"));
    }
}
