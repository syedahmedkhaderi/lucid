use std::sync::atomic::Ordering;

use tauri::{command, State};

use crate::{
    privacy::{self, PrivacyMode},
    state::AppState,
};

#[command]
pub async fn set_privacy_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    let mode = PrivacyMode::parse(&mode)?;
    if privacy::current_mode() == mode {
        return Ok(());
    }
    // Keep the audio lock until after the policy changes. Capture startup does
    // its final STT policy check under the same lock, closing the check/start race.
    let audio = state
        .audio
        .lock()
        .map_err(|_| "Audio state lock poisoned".to_string())?;
    let capturing = audio.as_ref().is_some_and(|manager| manager.is_capturing());
    let chat_running = state.workspace_chat.running() > 0;
    let ai_running = state.ai_runs.as_ref().is_some_and(|runs| runs.is_running());
    if state.meeting_active.load(Ordering::SeqCst) || capturing || chat_running || ai_running {
        return Err("Privacy mode can only be changed after the meeting, audio capture, and current AI responses have finished.".into());
    }
    privacy::set_mode(mode)?;
    drop(audio);
    log::info!("Privacy mode set to {}", mode.as_str());
    Ok(())
}

#[command]
pub async fn get_privacy_mode() -> String {
    privacy::current_mode().as_str().to_string()
}
