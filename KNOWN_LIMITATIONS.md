# Known limitations

The current code is a rebranded, tested adaptation of NexQ v3.0.0. It is not yet a production-ready Lucid release.

- The hard ten-document cap is per workspace. There is no separate per-meeting document picker yet; meetings use the sources in their selected workspace. Older workspaces that already exceed ten are not migrated.
- Uploaded files are capped at 50 MB each and empty files are rejected. Supported imported file types are PDF, DOCX, PPTX, TXT, and Markdown. PPTX extraction preserves slide numbers and basic table rows. Legacy DOC/PPT, scanned-PDF OCR, speaker notes, chart/image interpretation, and robust page/slide citations are missing or unverified.
- Gemini 3.5 Transcribe Live is implemented with a local mock-WebSocket test suite, but it has not been authenticated against a real Google account or tested with real meeting audio. Unexpected connection loss requires the user to restart transcription or choose another provider; automatic retry is limited to planned session refresh. Accuracy, latency, model/account access, billing and Mandarin/technical-term behavior are unverified.
- Google's [current model catalog](https://ai.google.dev/gemini-api/docs/models) lists Gemini 4 Hydrogen as `gemini-4-hydrogen`; this app has not confirmed account eligibility or free-tier access, and does not yet have a documented task-based model registry.
- Secure bulk import, key labels, project association, project-aware quota accounting, and credential lifecycle management are not implemented. Lucid does not rotate keys: Google's rate limits are project-level, so rotating keys from the same project does not create additional quota.
- Privacy modes are not enforced end to end. Do not use the app with confidential material or allow cloud inference for private meetings until this boundary is implemented and verified.
- The new triangular compact overlay placement math has unit tests, but the interactive native resize, monitor handling, and cross-platform visual behavior have not been runtime-tested.
- The performance goals (cached answer ≤300 ms, typical answer 1–2.5 s, initial transcript ≤500 ms, accuracy ≥98%) are unmeasured.
- Google's [pricing page](https://ai.google.dev/gemini-api/docs/pricing) says Gemini 3.5 Transcribe Live free-tier content may be used to improve products. It must not be used for confidential meetings. The app does not enforce a complete private-local boundary yet.
- Windows is the primary production target, but no Windows build, installer, or audio capture test has been run. Tauri's macOS Finder-based DMG layout step timed out; the current HFS+ DMG was assembled separately, checksum-verified, and mounted successfully. Gatekeeper and notarization behavior are untested.
- App Store/notarized distribution and automatic updates are not configured. Updates remain disabled because Lucid does not own a signing key or release feed.
- A complete third-party license inventory and release security review have not been completed.
- The host Node version is 24.1.0; the pinned jsdom package declares Node 24.15.0+ or 22.22.2+, so npm reports an engine warning even though build and tests pass.
