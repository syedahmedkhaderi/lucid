# Security and privacy

Lucid is an active local-first desktop project. This document records current handling rules; it is not a security certification.

## Secrets

- Keep API keys in macOS Keychain or Windows Credential Manager through the app's credential integration.
- Never place credentials in source, configuration files, logs, bug reports, tests, screenshots, or Git history.
- Revoke any key that was pasted into chat or otherwise exposed. Keys from the originating request were not copied into this checkout or probed.
- Gemini API quotas are project-level. Do not rotate keys from the same project to evade a quota response.

## Meeting data

- Meeting documents, transcripts, and recordings can contain sensitive personal or business information. Use synthetic fixtures during development.
- Lucid starts in **Private Local**. The Rust backend blocks cloud speech, cloud LLMs, subscription CLIs, remote translations and embeddings, Gemini context uploads, and non-loopback model endpoints. It allows offline STT engines and local LLM endpoints on loopback addresses.
- **Cloud processing allowed** requires an explicit settings change. It may transmit audio, transcript, document, note, or image content to configured providers and may incur provider charges.
- Privacy mode cannot be changed during an active meeting, audio capture, chat, or content-bearing AI request. Capture startup rechecks the selected STT provider under the same lock as policy changes.
- The privacy gate is unit- and integration-tested, but it has not received an independent security audit. Private Local cannot verify the behavior of software installed on the device, and meeting records are not encrypted at rest.
- Do not enable recording or transcription for a meeting unless participants and applicable policies allow it.

## Reporting

No Lucid-owned public security intake is configured yet. Do not send sensitive meeting data to the upstream NexQ issue tracker. For local evaluation, keep reports and reproduction material on the device until a private reporting channel is established.

## Release checks still required

Run secret scanning, dependency review, updater-signing review, independent privacy-boundary review, and Windows/macOS permission tests before any distribution. The upstream NexQ updater endpoint must remain disabled until Lucid has its own signing key and release feed.
