//! Gemini 3.5 Transcribe Live over Google's bidirectional WebSocket API.
//!
//! The provider streams 16 kHz mono PCM16 and converts interim/final
//! transcription events to Lucid transcript results. The API key is only read
//! from the OS credential store and is never included in logs or errors.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::audio::AudioChunk;
use crate::stt::deepgram::ConnectionStatusEvent;
use crate::stt::provider::{reported_language, STTProvider, STTProviderType, TranscriptResult};

type GeminiSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const DEFAULT_WS_URL: &str = "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent";
const MODEL: &str = "gemini-3.5-transcribe-live";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(12);
const FINAL_RESULTS_WAIT: Duration = Duration::from_secs(5);
const AUDIO_QUEUE_CHUNKS: usize = 128;
const SESSION_MAX_DURATION: Duration = Duration::from_secs(10 * 60);
const SESSION_ROLLOVER: Duration = Duration::from_secs(9 * 60 + 30);
const RECONNECT_ATTEMPTS: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TranscriptEvent {
    SetupComplete,
    Interim(String),
    Final(String),
    Error(String),
    Other,
}

fn parse_event(message: &str) -> TranscriptEvent {
    let Ok(value) = serde_json::from_str::<Value>(message) else {
        return TranscriptEvent::Other;
    };
    if value.get("setupComplete").is_some() {
        return TranscriptEvent::SetupComplete;
    }
    if value.get("error").is_some() {
        let error = value.get("error").unwrap_or(&Value::Null);
        let code = error
            .get("code")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let status = error.get("status").and_then(Value::as_str).unwrap_or("");
        let description = if code == 429 || status == "RESOURCE_EXHAUSTED" {
            "Gemini project quota is exhausted. Wait for the quota window or choose another configured provider.".to_string()
        } else if code == 401 || code == 403 || status == "PERMISSION_DENIED" {
            "Gemini rejected the credential or denied access to this model. Check the key and project access in Settings.".to_string()
        } else {
            "Gemini Live reported a session error. Check the provider status and network connection.".to_string()
        };
        return TranscriptEvent::Error(description);
    }
    let Some(content) = value.get("serverContent") else {
        return TranscriptEvent::Other;
    };
    if let Some(text) = content
        .get("interimInputTranscription")
        .and_then(|v| v.get("text"))
        .and_then(Value::as_str)
    {
        return TranscriptEvent::Interim(text.to_string());
    }
    if let Some(text) = content
        .get("inputTranscription")
        .and_then(|v| v.get("text"))
        .and_then(Value::as_str)
    {
        return TranscriptEvent::Final(text.to_string());
    }
    TranscriptEvent::Other
}

fn gemini_language_code(language: &str) -> Option<String> {
    let trimmed = language.trim();
    match trimmed.to_ascii_lowercase().as_str() {
        "" | "auto" => None,
        // The app's shared picker uses generic UI locales. Gemini documents a
        // different valid BCP-47 code for Spanish, Simplified Chinese, and Arabic.
        "es-es" => Some("es-419".to_string()),
        "zh-cn" => Some("cmn-Hans-CN".to_string()),
        "ar-sa" => Some("ar-EG".to_string()),
        _ => Some(trimmed.to_string()),
    }
}

fn setup_message(language: &str) -> Value {
    let language_codes: Vec<String> = gemini_language_code(language).into_iter().collect();
    json!({
        "setup": {
            "model": format!("models/{MODEL}"),
            "generationConfig": { "responseModalities": ["TEXT"] },
            "inputAudioTranscription": {
                "languageCodes": language_codes,
                "mode": "VERBATIM"
            }
        }
    })
}

fn build_ws_url(endpoint: &str, api_key: &str) -> Result<Url, String> {
    let mut url = Url::parse(endpoint).map_err(|_| "Invalid Gemini Live endpoint".to_string())?;
    url.query_pairs_mut().append_pair("key", api_key);
    Ok(url)
}

fn pcm_to_bytes(samples: &[i16]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect()
}

fn epoch_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn transcript_result(
    text: String,
    is_final: bool,
    timestamp_ms: u64,
    duration_ms: Option<u64>,
    party: &str,
    language: &str,
) -> TranscriptResult {
    TranscriptResult {
        text,
        is_final,
        confidence: 1.0,
        timestamp_ms,
        speaker: Some(party.to_string()),
        language: reported_language(language),
        segment_id: None,
        duration_ms,
        speaker_tag: None,
        turn: None,
    }
}

async fn connect_and_setup(
    endpoint: &str,
    api_key: &str,
    language: &str,
) -> Result<GeminiSocket, String> {
    if api_key.trim().is_empty() {
        return Err("Gemini API key not configured. Add it in Settings → STT.".to_string());
    }
    let url = build_ws_url(endpoint, api_key)?;
    let (mut socket, _) = tokio::time::timeout(CONNECT_TIMEOUT, connect_async(url.as_str()))
        .await
        .map_err(|_| {
            "Gemini Live connection timed out. Check your network and API key.".to_string()
        })?
        .map_err(|_| {
            "Gemini Live connection failed. Check your network and API key.".to_string()
        })?;

    socket
        .send(Message::Text(setup_message(language).to_string().into()))
        .await
        .map_err(|_| "Gemini Live could not start the transcription session.".to_string())?;

    let handshake = tokio::time::timeout(CONNECT_TIMEOUT, async {
        while let Some(message) = socket.next().await {
            match message {
                Ok(Message::Text(text)) => match parse_event(&text) {
                    TranscriptEvent::SetupComplete => return Ok(()),
                    TranscriptEvent::Error(description) => return Err(description),
                    _ => {}
                },
                Ok(Message::Close(_)) => break,
                Err(_) => return Err("Gemini Live connection closed during setup.".to_string()),
                _ => {}
            }
        }
        Err("Gemini Live closed the connection before setup completed.".to_string())
    })
    .await
    .map_err(|_| {
        "Gemini Live setup timed out. Check model access and project quota.".to_string()
    })?;
    if let Err(message) = handshake {
        let _ = socket.close(None).await;
        return Err(message);
    }
    Ok(socket)
}

async fn run_writer(
    mut sink: futures::stream::SplitSink<GeminiSocket, Message>,
    mut audio_rx: mpsc::Receiver<Vec<i16>>,
    party: String,
    endpoint: String,
    api_key: String,
    language: String,
    result_tx: mpsc::Sender<TranscriptResult>,
    app_handle: Option<tauri::AppHandle>,
    reader_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
) {
    let mut rollover_at = Instant::now() + SESSION_ROLLOVER;
    loop {
        tokio::select! {
            samples = audio_rx.recv() => {
                let Some(samples) = samples else { break; };
                if samples.is_empty() {
                    continue;
                }
                let message = json!({
                    "realtimeInput": {
                        "audio": {
                            "data": BASE64.encode(pcm_to_bytes(&samples)),
                            "mimeType": "audio/pcm;rate=16000"
                        }
                    }
                });
                if sink.send(Message::Text(message.to_string().into())).await.is_err() {
                    log::warn!("GeminiLiveSTT: audio stream ended after a send failure ({party})");
                    emit_status(&app_handle, &party, "disconnected", Some("Gemini Live connection ended. Restart transcription or choose another provider.".to_string()));
                    return;
                }
            }
            _ = tokio::time::sleep_until(rollover_at) => {
                // Google's documented live transcription session limit is 10 minutes.
                // Refresh before that boundary, keeping the same credential and bounded retries.
                let end_message = json!({ "realtimeInput": { "audioStreamEnd": true } });
                let _ = sink.send(Message::Text(end_message.to_string().into())).await;
                let _ = sink.close().await;
                emit_status(&app_handle, &party, "reconnecting", Some("Refreshing the Gemini Live session.".to_string()));

                let mut next_socket = None;
                let mut last_error = "Gemini Live session refresh failed.".to_string();
                for attempt in 0..RECONNECT_ATTEMPTS {
                    if attempt > 0 {
                        tokio::time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
                    }
                    match connect_and_setup(&endpoint, &api_key, &language).await {
                        Ok(socket) => {
                            next_socket = Some(socket);
                            break;
                        }
                        Err(error) => last_error = error,
                    }
                }
                let Some(socket) = next_socket else {
                    emit_status(&app_handle, &party, "error", Some(last_error));
                    return;
                };
                let (next_sink, next_stream) = socket.split();
                sink = next_sink;
                let reader = tokio::spawn(run_reader(
                    next_stream,
                    result_tx.clone(),
                    party.clone(),
                    language.clone(),
                    app_handle.clone(),
                ));
                if let Ok(mut tasks) = reader_tasks.lock() {
                    tasks.push(reader);
                } else {
                    reader.abort();
                }
                rollover_at = Instant::now() + SESSION_ROLLOVER;
                debug_assert!(SESSION_ROLLOVER < SESSION_MAX_DURATION);
                emit_status(&app_handle, &party, "connected", None);
            }
        }
    }

    let end_message = json!({ "realtimeInput": { "audioStreamEnd": true } });
    if sink
        .send(Message::Text(end_message.to_string().into()))
        .await
        .is_err()
    {
        log::debug!("GeminiLiveSTT: stream end signal could not be sent ({party})");
    }
    let _ = sink.close().await;
}

async fn run_reader(
    mut stream: futures::stream::SplitStream<GeminiSocket>,
    result_tx: mpsc::Sender<TranscriptResult>,
    party: String,
    language: String,
    app_handle: Option<tauri::AppHandle>,
) {
    let mut speech_start_ms: Option<u64> = None;
    while let Some(message) = stream.next().await {
        match message {
            Ok(Message::Text(text)) => match parse_event(&text) {
                TranscriptEvent::Interim(text) => {
                    if text.trim().is_empty() {
                        continue;
                    }
                    let now = epoch_now_ms();
                    let start = *speech_start_ms.get_or_insert(now);
                    if result_tx
                        .send(transcript_result(
                            text, false, start, None, &party, &language,
                        ))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                TranscriptEvent::Final(text) => {
                    let now = epoch_now_ms();
                    let start = speech_start_ms.take().unwrap_or(now);
                    if !text.trim().is_empty()
                        && result_tx
                            .send(transcript_result(
                                text,
                                true,
                                start,
                                Some(now.saturating_sub(start)),
                                &party,
                                &language,
                            ))
                            .await
                            .is_err()
                    {
                        return;
                    }
                }
                TranscriptEvent::Error(message) => {
                    log::warn!("GeminiLiveSTT: provider reported an error ({party})");
                    emit_status(&app_handle, &party, "error", Some(message));
                }
                TranscriptEvent::SetupComplete | TranscriptEvent::Other => {}
            },
            Ok(Message::Close(_)) => break,
            Err(_) => {
                log::warn!("GeminiLiveSTT: connection ended unexpectedly ({party})");
                emit_status(
                    &app_handle,
                    &party,
                    "disconnected",
                    Some("Gemini Live connection ended. Restart transcription or choose another provider.".to_string()),
                );
                break;
            }
            _ => {}
        }
    }
}

fn emit_status(
    app_handle: &Option<tauri::AppHandle>,
    party: &str,
    status: &str,
    message: Option<String>,
) {
    if let Some(handle) = app_handle {
        let payload = ConnectionStatusEvent {
            provider: "gemini_live".to_string(),
            party: party.to_string(),
            status: status.to_string(),
            message,
        };
        let _ = tauri::Emitter::emit(handle, "stt_connection_status", &payload);
    }
}

/// A Gemini transcription stream for a single party. Audio is held in a
/// bounded queue; capture never waits on network writes and stale audio is
/// dropped if the provider cannot keep pace.
pub struct GeminiLiveSTT {
    api_key: String,
    language: String,
    party: String,
    endpoint: String,
    app_handle: Option<tauri::AppHandle>,
    audio_tx: Option<mpsc::Sender<Vec<i16>>>,
    result_tx: Option<mpsc::Sender<TranscriptResult>>,
    writer_task: Option<JoinHandle<()>>,
    reader_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
    is_streaming: bool,
    muted: bool,
    dropped_chunks: u64,
}

impl GeminiLiveSTT {
    pub fn new() -> Self {
        Self {
            api_key: String::new(),
            language: "en-US".to_string(),
            party: "Them".to_string(),
            endpoint: DEFAULT_WS_URL.to_string(),
            app_handle: None,
            audio_tx: None,
            result_tx: None,
            writer_task: None,
            reader_tasks: Arc::new(Mutex::new(Vec::new())),
            is_streaming: false,
            muted: false,
            dropped_chunks: 0,
        }
    }

    pub fn with_api_key(api_key: &str) -> Self {
        let mut provider = Self::new();
        provider.api_key = api_key.trim().to_string();
        provider
    }

    pub fn set_api_key(&mut self, api_key: &str) {
        self.api_key = api_key.trim().to_string();
    }

    pub fn set_party(&mut self, party: &str) {
        self.party = party.to_string();
    }

    pub fn set_app_handle(&mut self, handle: tauri::AppHandle) {
        self.app_handle = Some(handle);
    }

    #[cfg(test)]
    fn for_endpoint(api_key: &str, endpoint: &str) -> Self {
        let mut provider = Self::with_api_key(api_key);
        provider.endpoint = endpoint.to_string();
        provider
    }
}

impl Default for GeminiLiveSTT {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl STTProvider for GeminiLiveSTT {
    fn provider_name(&self) -> &str {
        "Gemini Live Transcribe"
    }

    fn provider_type(&self) -> STTProviderType {
        STTProviderType::GeminiLive
    }

    async fn start_stream(
        &mut self,
        result_tx: mpsc::Sender<TranscriptResult>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_streaming {
            return Err("Gemini Live stream is already active".into());
        }
        emit_status(&self.app_handle, &self.party, "connecting", None);
        let socket = connect_and_setup(&self.endpoint, &self.api_key, &self.language)
            .await
            .map_err(|message| {
                emit_status(
                    &self.app_handle,
                    &self.party,
                    "error",
                    Some(message.clone()),
                );
                message
            })?;

        let (sink, stream) = socket.split();
        let (audio_tx, audio_rx) = mpsc::channel(AUDIO_QUEUE_CHUNKS);
        let reader = tokio::spawn(run_reader(
            stream,
            result_tx.clone(),
            self.party.clone(),
            self.language.clone(),
            self.app_handle.clone(),
        ));
        if let Ok(mut tasks) = self.reader_tasks.lock() {
            tasks.push(reader);
        } else {
            reader.abort();
            return Err("Gemini Live could not start its transcript reader".into());
        }
        self.writer_task = Some(tokio::spawn(run_writer(
            sink,
            audio_rx,
            self.party.clone(),
            self.endpoint.clone(),
            self.api_key.clone(),
            self.language.clone(),
            result_tx.clone(),
            self.app_handle.clone(),
            self.reader_tasks.clone(),
        )));
        self.audio_tx = Some(audio_tx);
        self.result_tx = Some(result_tx);
        self.is_streaming = true;
        emit_status(&self.app_handle, &self.party, "connected", None);
        Ok(())
    }

    async fn feed_audio(
        &mut self,
        chunk: AudioChunk,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_streaming || self.muted || chunk.pcm_data.is_empty() {
            return Ok(());
        }
        let Some(sender) = self.audio_tx.as_ref() else {
            return Ok(());
        };
        match sender.try_send(chunk.pcm_data) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                self.dropped_chunks += 1;
                if self.dropped_chunks.is_power_of_two() {
                    log::warn!(
                        "GeminiLiveSTT: realtime audio queue is behind ({}) ; audio chunks dropped for {}",
                        self.dropped_chunks,
                        self.party
                    );
                }
            }
            Err(TrySendError::Closed(_)) => {
                self.audio_tx = None;
                self.is_streaming = false;
                emit_status(
                    &self.app_handle,
                    &self.party,
                    "disconnected",
                    Some("Gemini Live writer stopped. Restart transcription or choose another provider.".to_string()),
                );
            }
        }
        Ok(())
    }

    async fn stop_stream(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.audio_tx = None;
        if let Some(writer) = self.writer_task.take() {
            let abort = writer.abort_handle();
            if tokio::time::timeout(FINAL_RESULTS_WAIT, writer)
                .await
                .is_err()
            {
                abort.abort();
            }
        }
        let readers = self
            .reader_tasks
            .lock()
            .map(|mut tasks| std::mem::take(&mut *tasks))
            .unwrap_or_default();
        for reader in readers {
            let abort = reader.abort_handle();
            if tokio::time::timeout(FINAL_RESULTS_WAIT, reader)
                .await
                .is_err()
            {
                abort.abort();
                log::warn!(
                    "GeminiLiveSTT: final transcript wait expired ({})",
                    self.party
                );
            }
        }
        self.result_tx = None;
        self.is_streaming = false;
        emit_status(&self.app_handle, &self.party, "disconnected", None);
        Ok(())
    }

    async fn test_connection(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut socket = connect_and_setup(&self.endpoint, &self.api_key, &self.language)
            .await
            .map_err(|message| -> Box<dyn std::error::Error + Send + Sync> { message.into() })?;
        let _ = socket.close(None).await;
        Ok(true)
    }

    fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        emit_status(
            &self.app_handle,
            &self.party,
            if muted { "paused" } else { "connected" },
            muted.then(|| "Muted: audio is not being sent to Gemini".to_string()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    #[test]
    fn setup_uses_auto_language_and_verbatim_transcription() {
        let setup = setup_message("auto");
        assert_eq!(setup["setup"]["model"], format!("models/{MODEL}"));
        assert_eq!(
            setup["setup"]["generationConfig"]["responseModalities"][0],
            "TEXT"
        );
        assert_eq!(
            setup["setup"]["inputAudioTranscription"]["languageCodes"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            setup["setup"]["inputAudioTranscription"]["mode"],
            "VERBATIM"
        );
    }

    #[test]
    fn setup_uses_a_selected_bcp47_language() {
        assert_eq!(
            setup_message("zh-CN")["setup"]["inputAudioTranscription"]["languageCodes"][0],
            "cmn-Hans-CN"
        );
        assert_eq!(
            setup_message("es-ES")["setup"]["inputAudioTranscription"]["languageCodes"][0],
            "es-419"
        );
        assert_eq!(
            setup_message("ar-SA")["setup"]["inputAudioTranscription"]["languageCodes"][0],
            "ar-EG"
        );
    }

    #[test]
    fn session_rolls_over_before_the_documented_ten_minute_limit() {
        assert!(SESSION_ROLLOVER < SESSION_MAX_DURATION);
        assert!(SESSION_MAX_DURATION - SESSION_ROLLOVER >= Duration::from_secs(30));
    }

    #[test]
    fn parses_interim_final_quota_and_auth_events() {
        assert_eq!(
            parse_event(r#"{"serverContent":{"interimInputTranscription":{"text":"Testing"}}}"#),
            TranscriptEvent::Interim("Testing".to_string())
        );
        assert_eq!(
            parse_event(r#"{"serverContent":{"inputTranscription":{"text":"Testing one two."}}}"#),
            TranscriptEvent::Final("Testing one two.".to_string())
        );
        assert!(matches!(
            parse_event(r#"{"error":{"code":429,"status":"RESOURCE_EXHAUSTED"}}"#),
            TranscriptEvent::Error(message) if message.contains("quota")
        ));
        assert!(matches!(
            parse_event(r#"{"error":{"code":403,"status":"PERMISSION_DENIED"}}"#),
            TranscriptEvent::Error(message) if message.contains("credential")
        ));
    }

    #[test]
    fn websocket_url_encodes_the_api_key_in_query_without_formatting_it() {
        let url = build_ws_url("ws://localhost:1234/live", "a key&value").unwrap();
        assert_eq!(
            url.query_pairs().find(|(key, _)| key == "key").unwrap().1,
            "a key&value"
        );
    }

    #[test]
    fn pcm_samples_are_little_endian_and_base64_round_trips() {
        let samples = [-2i16, 0, 258];
        let bytes = pcm_to_bytes(&samples);
        assert_eq!(bytes, vec![254, 255, 0, 0, 2, 1]);
        assert_eq!(BASE64.decode(BASE64.encode(&bytes)).unwrap(), bytes);
    }

    #[tokio::test]
    async fn test_connection_performs_setup_handshake_without_sending_audio() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(stream).await.unwrap();
            let request = socket.next().await.unwrap().unwrap().into_text().unwrap();
            let value: Value = serde_json::from_str(&request).unwrap();
            assert_eq!(value["setup"]["model"], format!("models/{MODEL}"));
            assert!(value["setup"]["inputAudioTranscription"].is_object());
            socket
                .send(Message::Text(r#"{"setupComplete":{}}"#.into()))
                .await
                .unwrap();
            let _close = socket.next().await.unwrap().unwrap();
        });
        let provider = GeminiLiveSTT::for_endpoint("test-key", &format!("ws://{address}/live"));
        assert!(provider.test_connection().await.unwrap());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn streams_pcm_and_maps_interim_and_final_server_events() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(stream).await.unwrap();
            let _setup = socket.next().await.unwrap().unwrap();
            socket
                .send(Message::Text(r#"{"setupComplete":{}}"#.into()))
                .await
                .unwrap();
            let audio = socket.next().await.unwrap().unwrap().into_text().unwrap();
            let value: Value = serde_json::from_str(&audio).unwrap();
            assert_eq!(
                value["realtimeInput"]["audio"]["mimeType"],
                "audio/pcm;rate=16000"
            );
            assert_eq!(
                BASE64
                    .decode(value["realtimeInput"]["audio"]["data"].as_str().unwrap())
                    .unwrap(),
                vec![1, 0, 2, 0]
            );
            socket
                .send(Message::Text(
                    r#"{"serverContent":{"interimInputTranscription":{"text":"hello"}}}"#.into(),
                ))
                .await
                .unwrap();
            socket
                .send(Message::Text(
                    r#"{"serverContent":{"inputTranscription":{"text":"hello there"}}}"#.into(),
                ))
                .await
                .unwrap();
            let _end = socket.next().await.unwrap().unwrap();
        });

        let mut provider = GeminiLiveSTT::for_endpoint("test-key", &format!("ws://{address}/live"));
        provider.set_party("Them");
        let (tx, mut rx) = mpsc::channel(4);
        provider.start_stream(tx).await.unwrap();
        provider
            .feed_audio(AudioChunk {
                pcm_data: vec![1, 2],
                source: crate::audio::AudioSource::System,
                timestamp_ms: epoch_now_ms(),
                is_speech: true,
            })
            .await
            .unwrap();
        let interim = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(interim.text, "hello");
        assert!(!interim.is_final);
        let final_result = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(final_result.text, "hello there");
        assert!(final_result.is_final);
        assert_eq!(final_result.speaker.as_deref(), Some("Them"));
        provider.stop_stream().await.unwrap();
        server.await.unwrap();
    }
}
