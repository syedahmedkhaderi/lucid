# Lucid architecture

Lucid retains the Tauri 2, Rust, Tokio, React, TypeScript, SQLite, and Zustand architecture of NexQ v3.0.0. The detailed subsystem map remains in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Runtime

- The Rust backend owns audio capture, speech providers, AI providers, retrieval, persistence, credentials, and native windows.
- The React frontend owns the launcher, meeting overlay, settings, and state stores.
- The overlay can shrink to a triangular top-center marker and restore its previous position and size.
- Typed Tauri IPC commands carry requests to Rust; typed events carry live transcript and generation updates back to the frontend.
- Meeting history and document indexes are local SQLite data. Provider keys belong in the OS credential store.
- Cloud providers are optional integrations. Local recognition and Ollama/LM Studio are supported by the upstream foundation; Lucid must keep the selected privacy mode visible and never silently change it.

## Lucid-specific direction

Lucid uses the `com.lucid.desktop` application identity and a separate data namespace; it does not automatically import NexQ data. Work should prioritize the preparation workflow, a per-meeting ten-document scope, the requested Office formats, Gemini streaming transcription, quota-aware credential selection, and evidence validation. These gaps are tracked in [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md).

## Upstream pin

- Repository: <https://github.com/naxhq/NexQ>
- Release tag: `v3.0.0`
- Commit: `d83db586024270bfd0bcf7221a55275b104c4246`
- License: MIT; preserve [`LICENSE`](LICENSE) and attribution in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
