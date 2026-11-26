# Known limitations

# Known limitations

The current code is a rebranded, tested adaptation of NexQ v3.0.0. It is not yet a production-ready Lucid release.

- Meeting prep does not yet provide a meeting-specific ten-document scope. Retrieval currently uses the selected workspace's existing sources.
- Supported imported file types are PDF, DOCX, TXT, and Markdown. Legacy DOC/PPT, PPTX extraction, scanned-PDF OCR, image/table interpretation, and robust slide-level citations are missing or unverified.
- Gemini 3.5 Transcribe Live is not integrated. Existing speech providers are inherited from NexQ; Mandarin/technical-term accuracy and live latency were not tested with real audio.
- Gemini 4 Hydrogen has not been verified as an eligible official Developer API model. Model availability and entitlement need a documented registry rather than an assumption.
- Multiple credentials can be stored through existing provider settings, but secure bulk import, key labels, project association, project-aware quota accounting, and lifecycle management are not implemented.
- Privacy modes are not enforced end to end. Do not use the app with confidential material or allow cloud inference for private meetings until this boundary is implemented and verified.
- The new triangular compact overlay placement math has unit tests, but the interactive native resize, monitor handling, and cross-platform visual behavior have not been runtime-tested.
- The performance goals (cached answer ≤300 ms, typical answer 1–2.5 s, initial transcript ≤500 ms, accuracy ≥98%) are unmeasured.
- Windows is the primary production target, but no Windows build, installer, or audio capture test has been run. Tauri's macOS DMG layout script timed out while scripting Finder; a standalone HFS+ DMG was assembled and verified, but its Gatekeeper/notarization behavior is untested.
- App Store/notarized distribution and automatic updates are not configured. Updates remain disabled because Lucid does not own a signing key or release feed.
- A complete third-party license inventory and release security review have not been completed.
- The host Node version is 24.1.0; the pinned jsdom package declares Node 24.15.0+ or 22.22.2+, so npm reports an engine warning even though build and tests pass.
