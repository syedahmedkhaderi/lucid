# Security and privacy

Lucid is an active local-first desktop project. This document records current handling rules; it is not a security certification.

## Secrets

- Keep API keys in macOS Keychain or Windows Credential Manager through the app's credential integration.
- Never place credentials in source, configuration files, logs, bug reports, tests, screenshots, or Git history.
- Revoke any key that was pasted into chat or otherwise exposed. Keys from the originating request were not copied into this checkout or probed.
- Gemini API quotas are project-level. Do not rotate keys from the same project to evade a quota response.

## Meeting data

- Meeting documents, transcripts, and recordings can contain sensitive personal or business information. Use synthetic fixtures during development.
- The upstream foundation supports both local and cloud providers. Cloud inference transmits the content included in a request; choose a local provider for confidential material unless cloud use is explicitly approved.
- A Lucid-specific privacy-mode gate has not yet been completed. Until it is, do not assume that selecting a workspace alone prevents cloud transmission.
- Do not enable recording or transcription for a meeting unless participants and applicable policies allow it.

## Reporting

No Lucid-owned public security intake is configured yet. Do not send sensitive meeting data to the upstream NexQ issue tracker. For local evaluation, keep reports and reproduction material on the device until a private reporting channel is established.

## Release checks still required

Run secret scanning, dependency review, updater-signing review, privacy-boundary review, and Windows/macOS permission tests before any distribution. The upstream NexQ updater endpoint must remain disabled until Lucid has its own signing key and release feed.
