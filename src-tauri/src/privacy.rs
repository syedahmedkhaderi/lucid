//! Runtime privacy gate for content-bearing provider calls.
//!
//! The safe mode is active before the frontend loads. Cloud access must be
//! explicitly enabled from Settings and is checked at the backend boundary.

use std::sync::{
    atomic::{AtomicU8, AtomicUsize, Ordering},
    Mutex,
};

use crate::llm::{ProviderConfig, ProviderType};
use crate::stt::provider::STTProviderType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyMode {
    PrivateLocal = 0,
    CloudAllowed = 1,
}

impl PrivacyMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "private_local" => Ok(Self::PrivateLocal),
            "cloud_allowed" => Ok(Self::CloudAllowed),
            _ => Err("Unknown privacy mode. Choose private_local or cloud_allowed.".into()),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PrivateLocal => "private_local",
            Self::CloudAllowed => "cloud_allowed",
        }
    }
}

// Runtime starts private. Unit tests use cloud mode by default so existing
// provider mock tests remain isolated from process-global runtime settings.
#[cfg(test)]
const DEFAULT_MODE: u8 = PrivacyMode::CloudAllowed as u8;
#[cfg(not(test))]
const DEFAULT_MODE: u8 = PrivacyMode::PrivateLocal as u8;
static MODE: AtomicU8 = AtomicU8::new(DEFAULT_MODE);
static MODE_LOCK: Mutex<()> = Mutex::new(());
static ACTIVE_CONTENT_REQUESTS: AtomicUsize = AtomicUsize::new(0);

/// Held for one content-bearing provider request. It prevents a mode change
/// from racing a request that has already passed the policy check.
pub struct ContentRequestGuard;

impl Drop for ContentRequestGuard {
    fn drop(&mut self) {
        ACTIVE_CONTENT_REQUESTS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn current_mode() -> PrivacyMode {
    if MODE.load(Ordering::SeqCst) == PrivacyMode::CloudAllowed as u8 {
        PrivacyMode::CloudAllowed
    } else {
        PrivacyMode::PrivateLocal
    }
}

pub fn set_mode(mode: PrivacyMode) -> Result<(), String> {
    let _lock = MODE_LOCK
        .lock()
        .map_err(|_| "Privacy policy lock poisoned".to_string())?;
    if current_mode() != mode && ACTIVE_CONTENT_REQUESTS.load(Ordering::SeqCst) > 0 {
        return Err(
            "Wait for current AI and translation requests to finish before changing privacy mode."
                .into(),
        );
    }
    MODE.store(mode as u8, Ordering::SeqCst);
    Ok(())
}

pub fn is_private_local() -> bool {
    current_mode() == PrivacyMode::PrivateLocal
}

/// Local provider hosts must be explicit loopback addresses. `localhost.evil`
/// and non-HTTP transports are intentionally rejected.
pub fn is_loopback_url(raw: &str) -> bool {
    let Ok(url) = url::Url::parse(raw.trim()) else {
        return false;
    };
    if url.scheme() != "http" || !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    match url.host() {
        Some(url::Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(host)) => host.is_loopback(),
        Some(url::Host::Ipv6(host)) => host.is_loopback(),
        None => false,
    }
}

pub fn ensure_llm_config_allowed(config: &ProviderConfig) -> Result<(), String> {
    let provider = ProviderType::from_str(&config.provider_type).map_err(|e| e.to_string())?;
    ensure_llm_allowed(&provider, config.base_url.as_deref())
}

pub fn begin_llm_request(
    provider: &ProviderType,
    base_url: Option<&str>,
) -> Result<ContentRequestGuard, String> {
    let _lock = MODE_LOCK
        .lock()
        .map_err(|_| "Privacy policy lock poisoned".to_string())?;
    ensure_llm_allowed_for(current_mode(), provider, base_url)?;
    ACTIVE_CONTENT_REQUESTS.fetch_add(1, Ordering::SeqCst);
    Ok(ContentRequestGuard)
}

pub fn begin_translation_request(provider: &str) -> Result<ContentRequestGuard, String> {
    let _lock = MODE_LOCK
        .lock()
        .map_err(|_| "Privacy policy lock poisoned".to_string())?;
    ensure_translation_allowed_for(current_mode(), provider)?;
    ACTIVE_CONTENT_REQUESTS.fetch_add(1, Ordering::SeqCst);
    Ok(ContentRequestGuard)
}

pub fn begin_embedder_request(url: &str) -> Result<ContentRequestGuard, String> {
    let _lock = MODE_LOCK
        .lock()
        .map_err(|_| "Privacy policy lock poisoned".to_string())?;
    ensure_embedder_allowed_for(current_mode(), url)?;
    ACTIVE_CONTENT_REQUESTS.fetch_add(1, Ordering::SeqCst);
    Ok(ContentRequestGuard)
}

pub fn ensure_llm_allowed(provider: &ProviderType, base_url: Option<&str>) -> Result<(), String> {
    ensure_llm_allowed_for(current_mode(), provider, base_url)
}

fn ensure_llm_allowed_for(
    mode: PrivacyMode,
    provider: &ProviderType,
    base_url: Option<&str>,
) -> Result<(), String> {
    if mode == PrivacyMode::CloudAllowed {
        return Ok(());
    }
    let url = base_url
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| provider.default_base_url());
    let permitted = match provider {
        ProviderType::Ollama | ProviderType::LmStudio | ProviderType::Custom => {
            is_loopback_url(url)
        }
        // CLI agents run locally but send prompts to their vendors' services.
        _ => false,
    };
    if permitted {
        Ok(())
    } else {
        Err(format!(
            "Private Local mode blocked {}. Choose Ollama or LM Studio on this device, or enable Cloud processing in Privacy settings.",
            provider.display_name()
        ))
    }
}

pub fn ensure_stt_allowed(provider: &STTProviderType) -> Result<(), String> {
    ensure_stt_allowed_for(current_mode(), provider)
}

fn ensure_stt_allowed_for(mode: PrivacyMode, provider: &STTProviderType) -> Result<(), String> {
    if mode == PrivacyMode::CloudAllowed {
        return Ok(());
    }
    if matches!(
        provider,
        STTProviderType::WhisperCpp
            | STTProviderType::SherpaOnnx
            | STTProviderType::OrtStreaming
            | STTProviderType::ParakeetTdt
    ) {
        Ok(())
    } else {
        Err(format!(
            "Private Local mode blocked {} speech recognition. Choose an offline speech model, or enable Cloud processing in Privacy settings.",
            provider.as_str()
        ))
    }
}

pub fn ensure_translation_allowed(provider: &str) -> Result<(), String> {
    ensure_translation_allowed_for(current_mode(), provider)
}

fn ensure_translation_allowed_for(mode: PrivacyMode, provider: &str) -> Result<(), String> {
    if mode == PrivacyMode::CloudAllowed || matches!(provider, "opus-mt" | "llm") {
        Ok(())
    } else {
        Err(format!(
            "Private Local mode blocked {} translation. Choose Opus-MT or a local LLM, or enable Cloud processing in Privacy settings.",
            provider
        ))
    }
}

pub fn ensure_embedder_allowed(url: &str) -> Result<(), String> {
    ensure_embedder_allowed_for(current_mode(), url)
}

fn ensure_embedder_allowed_for(mode: PrivacyMode, url: &str) -> Result<(), String> {
    if mode == PrivacyMode::CloudAllowed || is_loopback_url(url) {
        Ok(())
    } else {
        Err("Private Local mode blocked the remote embeddings endpoint. Use a local Ollama endpoint, or enable Cloud processing in Privacy settings.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_http_loopback_hosts() {
        for url in [
            "http://localhost:11434",
            "http://127.0.0.1:1234/v1",
            "http://[::1]:11434",
        ] {
            assert!(is_loopback_url(url), "{url}");
        }
        for url in [
            "https://localhost:11434",
            "http://localhost.example.com",
            "http://192.168.1.10:11434",
            "http://user:pass@localhost:11434",
            "not a url",
        ] {
            assert!(!is_loopback_url(url), "{url}");
        }
    }

    #[test]
    fn privacy_mode_names_are_explicit() {
        assert_eq!(
            PrivacyMode::parse("private_local").unwrap(),
            PrivacyMode::PrivateLocal
        );
        assert_eq!(
            PrivacyMode::parse("cloud_allowed").unwrap(),
            PrivacyMode::CloudAllowed
        );
        assert!(PrivacyMode::parse("automatic").is_err());
    }

    #[test]
    fn private_mode_allows_only_local_content_providers() {
        let local = PrivacyMode::PrivateLocal;
        assert!(ensure_llm_allowed_for(local, &ProviderType::Ollama, None).is_ok());
        assert!(ensure_llm_allowed_for(local, &ProviderType::LmStudio, None).is_ok());
        assert!(ensure_llm_allowed_for(
            local,
            &ProviderType::Custom,
            Some("http://127.0.0.1:8080/v1")
        )
        .is_ok());
        assert!(ensure_llm_allowed_for(
            local,
            &ProviderType::Custom,
            Some("https://example.com/v1")
        )
        .is_err());
        assert!(ensure_llm_allowed_for(local, &ProviderType::Gemini, None).is_err());
        assert!(ensure_llm_allowed_for(local, &ProviderType::CodexCli, None).is_err());
        assert!(ensure_embedder_allowed_for(local, "http://localhost:11434").is_ok());
        assert!(ensure_embedder_allowed_for(local, "http://192.168.1.4:11434").is_err());
        assert!(ensure_translation_allowed_for(local, "opus-mt").is_ok());
        assert!(ensure_translation_allowed_for(local, "google").is_err());
    }

    #[test]
    fn private_mode_allows_only_offline_speech_recognition() {
        assert!(
            ensure_stt_allowed_for(PrivacyMode::PrivateLocal, &STTProviderType::ParakeetTdt)
                .is_ok()
        );
        assert!(
            ensure_stt_allowed_for(PrivacyMode::PrivateLocal, &STTProviderType::WebSpeech).is_err()
        );
        assert!(
            ensure_stt_allowed_for(PrivacyMode::PrivateLocal, &STTProviderType::GeminiLive)
                .is_err()
        );
        assert!(
            ensure_stt_allowed_for(PrivacyMode::CloudAllowed, &STTProviderType::GeminiLive).is_ok()
        );
    }
}
