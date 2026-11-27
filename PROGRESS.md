# Lucid progress

Updated: 2026-11-27

## Completed

- Created the project at `/Users/syed/Downloads/summer project` from NexQ `v3.0.0`, preserving history and MIT license. Pinned commit: `d83db586024270bfd0bcf7221a55275b104c4246`; branch `lucid/main`; upstream remote retained.
- Rebranded the app package, Tauri product, Rust crate, visible UI, installer strings, icons, and user-data identity (`com.lucid.desktop`). NexQ user data is isolated and is not automatically migrated.
- Removed the upstream update endpoint/key and disabled update artifacts until Lucid owns a signed release channel.
- Added a compact triangular top-center overlay with monitor-aware positioning and restoration of the expanded window bounds.
- Added tests for compact placement across scaled, offset, and missing-work-area monitors.
- Added bounded PPTX slide-text and table-row extraction, numeric slide ordering, slide markers, and local retrieval indexing. Tests cover entities, table rows, unreadable archives, expansion limits, and folder search.
- Added a hard ten-document cap per workspace scope, a 50 MB upload limit per file, early rejection of empty files, and cleanup when persistence fails. Existing workspaces are not migrated if they already exceed ten.
- Added a 500 MiB aggregate source-file limit per workspace, enforced transactionally alongside the ten-document cap.
- Added Gemini 3.5 Transcribe Live through Google's bidirectional WebSocket API, including 16 kHz audio streaming, interim/final transcript handling, language auto-detection, bounded queueing, safe error messages, and session refresh before the documented maximum. Unit and local mock-WebSocket tests pass; no real Gemini credentials or meeting audio were used.
- Added backend-enforced Private Local and Cloud processing modes. Private Local is the runtime default and blocks cloud speech, cloud LLMs, subscription CLIs, remote translation and embeddings; only offline STT and loopback local AI endpoints are allowed. Mode changes are refused during meetings or in-flight content requests. The active mode is visible in the dashboard and meeting footer.
- Excluded ignored `work/` packaging output from Vitest discovery so generated app resources do not appear as tests.
- Added project guidance, architecture, decisions, known limitations, security, build, progress, benchmark, and third-party notice documents.
- Added a reproducible SPDX/dependency metadata inventory generator. It covers 165 production npm and 711 Cargo.lock packages; license texts and legal classification still require review.
- Did not copy, test, or store the Gemini API keys pasted in the request.

## Verified on this Apple Silicon Mac

- `npm ci`: passed; 396 packages audited, 0 vulnerabilities. Node 24.1.0 emits a declared engine warning from jsdom (requires 24.15.0+ or 22.22.2+).
- `npm test`: 255 files passed; 3,931 tests passed; 2 skipped.
- `npm run build`: passed. Existing Tailwind and ineffective dynamic-import warnings remain.
- `cargo test` with the Xcode `LIBCLANG_PATH` and Unix Makefiles CMake generator: 2,006 passed; 24 ignored; 0 failed across unit and integration targets.
- Local retrieval evaluation: 100% hit@1 and hit@8 across 30 questions and 42 synthetic English/Japanese/Chinese meetings. Search benchmark: 500 × 10,000-word synthetic transcripts, 24,000 passages, p50 87.9 ms / p95 97.2 ms / max 99.6 ms over 120 warm searches; corpus indexing 6.2 s with SQLite optimized at level 3. Unoptimized SQLite baseline p95 was 109.9 ms. These do not measure generated answer accuracy or end-to-end latency.
- `./build-mac.sh --no-install --bundles app`: passed for the final source. The ARM64 bundle is 85.94 MiB; its `com.lucid.desktop` identifier and `0.1.0` version were checked, and `codesign --verify --deep --strict` passed.
- Rebuilt `src-tauri/target/release/bundle/dmg/Lucid_0.1.0_macOS_arm64.dmg` as a 30 MB compressed HFS+ disk image. `hdiutil verify` passed; mounting confirmed `Lucid.app` and the `Applications` shortcut. SHA-256: `2a940bf60cdebc485be77852d52fb1a72b1a197be594d845d78d7153525fbe4b`.
- Gemini Live provider tests use a local mock WebSocket server; the provider has not been authenticated against Google's live endpoint.
- A secret scan of non-generated repository files found none of the pasted Gemini key pattern.
- `npm run license:inventory`: passed; 0 Rust packages lack declared license metadata; all 98 npm packages without declared SPDX expressions had a license-related file in the installed package tree. No legal review or license-text collection has been completed.
- The test harness paths were updated for Cargo 1.100's build output layout.

## Still required for the specified product

- A separate per-meeting document picker; the current hard ten-document cap is per workspace, which serves as the meeting scope in the existing app.
- Legacy DOC/PPT extraction, OCR, speaker notes, chart and image interpretation, and robust page/slide citations. PPTX text extraction currently covers slide text and basic table rows with slide markers.
- Technical vocabulary, verified-fact cache, and strict evidence checks.
- Gemini Live real-account and real-audio verification, performance testing, clearer disconnect recovery, and credential lifecycle tooling. Keys are not rotated because quotas are project-level, not key-level.
- A task-based model registry and policy-driven provider selection.
- Independent privacy/security audit and encryption at rest. Private Local cannot verify the behavior of local software you install.
- Windows build/installer/audio validation and native overlay runtime testing.
- Real synthetic/safely licensed speech benchmark and latency measurements.
- Legal classification and notice collection/release check for the dependency inventory, signing, notarization, and Lucid update channel.

Google's [official Live API guide](https://ai.google.dev/gemini-api/docs/live-api/live-transcribe) documents `gemini-3.5-transcribe-live` and a ten-minute session limit. Lucid refreshes the session before that limit, but model access, account eligibility, billing, real-audio quality, and latency have not been confirmed with a live account. Google's [pricing page](https://ai.google.dev/gemini-api/docs/pricing) says free-tier content may be used to improve products, so it is not suitable for confidential meetings.

See [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) for the current release-readiness boundary.
