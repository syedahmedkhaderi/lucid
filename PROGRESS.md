# Lucid progress

Updated: 2026-11-27

## Completed

- Created the project at `the Lucid source directory` from NexQ `v3.0.0`, preserving history and MIT license. Pinned commit: `d83db586024270bfd0bcf7221a55275b104c4246`; branch `lucid/main`; upstream remote retained.
- Rebranded the app package, Tauri product, Rust crate, visible UI, installer strings, icons, and user-data identity (`com.lucid.desktop`). NexQ user data is isolated and is not automatically migrated.
- Removed the upstream update endpoint/key and disabled update artifacts until Lucid owns a signed release channel.
- Added a compact triangular top-center overlay with monitor-aware positioning and restoration of the expanded window bounds.
- Added tests for compact placement across scaled, offset, and missing-work-area monitors.
- Added bounded PPTX slide-text and table-row extraction, numeric slide ordering, slide markers, and local retrieval indexing. Tests cover entities, table rows, unreadable archives, expansion limits, and folder search.
- Added a hard ten-document cap per workspace scope, a 50 MB upload limit per file, early rejection of empty files, and cleanup when persistence fails. Existing workspaces are not migrated if they already exceed ten.
- Added a 500 MiB aggregate source-file limit per workspace, enforced transactionally alongside the ten-document cap.
- The meeting-scoped live brief now requires an exact excerpt from its cited passage. It checks that the fact's names and numbers occur in the quote and that at least half of the fact's distinct content words overlap. The quote is visible in the panel and included in live-answer context. This lexical filter cannot prove meaning or replace a structured/versioned fact database.
- Added a local structured source-fact cache (workspace migrations v20–v21) for explicitly labeled numeric values, ISO dates, model/equipment identifiers, suppliers, materials, parties, acronyms, requirements, obligations, and payment terms. Each candidate keeps its subject, attribute, exact value and unit, trailing conditions, source id/location, supporting quote, indexed source hash, extraction method, and `source_exact` status. Migration v21 backfills the expanded field set from existing indexed passages; source changes and purges replace/remove matching cache rows. Workspace chat injects only attribute matches from the active source scope and flags conflicting values when subject, attribute, and captured conditions match. Precise single-source numeric/date lookups can return a cited local answer without a generation request; ambiguous questions and text-field lookups use the evidence-grounded model path. The cache confirms exact source text, not semantic meaning or document precedence.
- Workspace chat now places mandatory evidence and citation rules after user-configured workspace instructions: source-based claims must cite provided labels, missing/conflicting evidence must trigger abstention, previous AI answers are not evidence, and calculations must show cited inputs. Citation labels are validated against the context plan; semantic support remains unproven.
- Added a per-meeting document picker with a ten-document limit across the home, borrowed, and included General workspaces. The selection is committed and checked against the effective workspace scope before the AI session starts. Legacy meetings and shortcut starts retain their existing all-in-scope behavior.
- Added bounded, cached PDF page extraction and page-scoped workspace search passages. Chat citations, including pinned PDF material, now show the matching page; previously loaded PDFs are backfilled on the next index pass when their copied source file is available.
- Added Gemini 3.5 Transcribe Live through Google's bidirectional WebSocket API, including 16 kHz audio streaming, interim/final transcript handling, language auto-detection, bounded queueing, safe error messages, and session refresh before the documented maximum. Unit and local mock-WebSocket tests pass; no real Gemini credentials or meeting audio were used.
- Added backend-enforced Private Local and Cloud processing modes. Private Local is the runtime default and blocks cloud speech, cloud LLMs, subscription CLIs, remote translation and embeddings; only offline STT and loopback local AI endpoints are allowed. Mode changes are refused during meetings or in-flight content requests. The active mode is visible in the dashboard and meeting footer.
- Added LLM credential profiles with individual and tab-separated bulk import, provider validation before storage, exact duplicate detection, user-entered labels/project IDs, masked previews, enable/disable, explicit activation, and local deletion. Secret values live in the OS credential store; migration v22 stores metadata only. Profile switches are explicit and never occur automatically after rate-limit or quota errors. The regular LLM key replacement flow also now tests before replacing a previously stored key.
- Excluded ignored `work/` packaging output from Vitest discovery so generated app resources do not appear as tests.
- Added project guidance, architecture, decisions, known limitations, security, build, progress, benchmark, and third-party notice documents.
- Added a reproducible SPDX/dependency metadata inventory generator. It covers 165 production npm and 711 Cargo.lock packages; license texts and legal classification still require review.
- Did not copy, test, or store the Gemini API keys pasted in the request.

## Verified on this Apple Silicon Mac

- `npm ci`: passed; 396 packages audited, 0 vulnerabilities. Node 24.1.0 emits a declared engine warning from jsdom (requires 24.15.0+ or 22.22.2+).
- `npm test -- --reporter=dot`: 258 files passed; 3,942 tests passed; 2 skipped.
- `npm run build`: passed. Existing Tailwind and ineffective dynamic-import warnings remain.
- `source scripts/macos-env.sh; export CMAKE_MAKE_PROGRAM="$(command -v ninja)"; cargo test --quiet`: 2,037 passed; 24 ignored; 0 failed across all Rust targets. The explicit Ninja path is needed for the nested Cargo build in the ACP integration tests on this host.
- Dependency scans: `npm audit --omit=dev` found 0 vulnerabilities. `cargo-audit` 0.23.0 found 0 vulnerable Rust packages after updating `cookie` to 0.18.3, `url` to 2.5.9, and `rustix` to 1.1.6; nine warnings remain for unmaintained/yanked crates and Linux GTK's `glib` unsoundness advisory.
- Local retrieval evaluation: 100% hit@1 and hit@8 across 30 questions and 42 synthetic English/Japanese/Chinese meetings. Search benchmark: 500 × 10,000-word synthetic transcripts, 24,000 passages, p50 87.9 ms / p95 97.2 ms / max 99.6 ms over 120 warm searches; corpus indexing 6.2 s with SQLite optimized at level 3. Unoptimized SQLite baseline p95 was 109.9 ms. These do not measure generated answer accuracy or end-to-end latency.
- `./build-mac.sh --no-install --bundles app`: passed after the credential-profile, write-only secret IPC, and updater-config startup fixes. The installed `/Applications/Lucid.app` was launched and reached the Lucid Setup welcome screen. The ARM64 bundle is 86.36 MiB; its `com.lucid.desktop` identifier and `0.1.0` version were checked, and `codesign --verify --deep --strict` passed. Notarization is unavailable because signing credentials are not configured.
- Added the cross-platform `npm run setup` bootstrap. Its macOS prerequisite/plan path passed with `npm run setup -- --dry-run`; Windows setup is implemented but cannot be exercised on this host.
- Tauri's Finder-based DMG layout timed out. Built `src-tauri/target/release/bundle/dmg/Lucid_0.1.0_aarch64_secure_credentials_hfsplus.dmg` from the current app as a 28.5 MB compressed HFS+ image. `hdiutil verify` and read-only mounting passed; the image contains the validly signed `Lucid.app` and `Applications` shortcut. SHA-256: `5145590aeb46eabef5288f8e4baf4654c0c792594e8e11537ddbf7b2a9fdfd75`.
- Gemini Live provider tests use a local mock WebSocket server; the provider has not been authenticated against Google's live endpoint.
- A secret scan of non-generated repository files found none of the pasted Gemini key pattern.
- `npm run license:inventory`: passed; 0 Rust packages lack declared license metadata; all 98 npm packages without declared SPDX expressions had a license-related file in the installed package tree. No legal review or license-text collection has been completed.
- The test harness paths were updated for Cargo 1.100's build output layout.
- Per-meeting document selection focused tests: 16 frontend tests and 2 Rust tests passed; TypeScript typecheck passed. PDF page extraction, page-aware indexing, page persistence, citation labels, source-exact fact extraction, v21 fact-cache rebuild, and no-generation direct numeric lookup have dedicated tests. Full frontend and Rust suites pass with these features included.

## Still required for the specified product

- Legacy DOC/PPT extraction, OCR, speaker notes, chart and image interpretation, and page-specific navigation inside the PDF viewer. PDF source citations now include page numbers; PPTX text extraction currently covers slide text and basic table rows with slide markers.
- Broader fact extraction for unlabeled prose, entities, parties, materials, acronyms, clauses, technical requirements, and non-ISO dates; semantic verification and document-precedence rules.
- Gemini Live real-account and real-audio verification, performance testing, clearer disconnect recovery, provider-reported quota status, and profile last-use/runtime-error tracking. Keys are not rotated automatically because quotas are project-level, not key-level.
- A task-based model registry and policy-driven provider selection.
- Independent privacy/security audit and encryption at rest. Private Local cannot verify the behavior of local software you install.
- Windows build/installer/audio validation and native overlay runtime testing.
- Real synthetic/safely licensed speech benchmark and latency measurements.
- Legal classification and notice collection/release check for the dependency inventory, signing, notarization, and Lucid update channel.

Google's [official Live API guide](https://ai.google.dev/gemini-api/docs/live-api/live-transcribe) documents `gemini-3.5-transcribe-live` and a ten-minute session limit. Lucid refreshes the session before that limit, but model access, account eligibility, billing, real-audio quality, and latency have not been confirmed with a live account. Google's [pricing page](https://ai.google.dev/gemini-api/docs/pricing) says free-tier content may be used to improve products, so it is not suitable for confidential meetings.

See [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) for the current release-readiness boundary.
