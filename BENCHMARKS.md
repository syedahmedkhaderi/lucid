# Benchmarks

Updated: 2026-11-27. Measurements below were collected on this development Mac and are engineering checks, not a controlled customer benchmark.

## Test environment

- Hardware: Apple M5, 24 GB memory.
- Platform: macOS, ARM64.
- Search benchmark: Rust's Cargo test profile, SQLite temporary database, local synthetic corpus. The main run compiled `libsqlite3-sys` with test-profile optimization level 3 (as the benchmark recommends); the application Rust code itself remained in the Cargo test profile.
- No cloud provider, API key, meeting audio, or private document was used.

## Retrieval measurements

| Measurement | Corpus and method | Result |
|---|---|---:|
| Meeting retrieval hit@1 / hit@8 | 42 synthetic meetings (32 English, 6 Japanese, 4 Chinese); 30 questions; gold meeting known | 100% / 100% |
| Search latency p50 / p95 / max | 500 synthetic transcripts × 10,000 words; 24,000 passages; 250-meeting scope; 120 timed searches after warm-up; SQLite optimized | 87.9 / 97.2 / 99.6 ms |
| Corpus indexing time | Same search benchmark corpus; SQLite optimized | 6.2 s |
| Search latency baseline p50 / p95 / max | Same corpus and test profile without SQLite package optimization | 88.8 / 109.9 / 114.7 ms |

The retrieval accuracy fixture measures whether search returns the correct meeting; it does not grade generated answers, evidence citations, numeric reasoning, or abstention. The latency run measures local text search, not embeddings, document extraction, AI generation, or full meeting workflow. It ran in the Cargo test profile and should not be treated as a release-build or ten-document production benchmark.

## Acceptance targets not measured

| Measure | Target | Verified result |
|---|---:|---|
| Cached factual answer | ≤300 ms | Not measured |
| Typical AI answer | 1–2.5 s | Not measured |
| Initial live transcription | ≤500 ms | Not measured |
| Factual answer accuracy | ≥98% | Not measured |
| Citation correctness | ≥99% | Not measured |
| Unsupported-number rate | 0 accepted unsupported figures | Not measured |
| Unanswerable-question abstention | ≥95% | Not measured |
| Accented English / Mandarin ASR | Curated speech benchmark | Not measured |
| Four-hour meeting stability | No crash or lost audio | Not measured |

## Build and regression verification

| Check | Result |
|---|---|
| Frontend tests | 3,931 passed; 2 skipped (255 files) |
| Rust tests | 2,006 passed; 24 ignored across unit and integration targets |
| Frontend production build | Passed; existing Tailwind and ineffective dynamic-import warnings remain |
| macOS app bundle | ARM64; 85.91 MiB; ad-hoc signature verification passed |
| macOS installer image | 27 MB HFS+ DMG; `hdiutil verify` passed; mounted and confirmed `Lucid.app` plus Applications shortcut |

The current DMG SHA-256 is recorded in [PROGRESS.md](PROGRESS.md). Notarization, Gatekeeper behavior, Windows runtime/audio, and real-provider speech quality remain untested.
