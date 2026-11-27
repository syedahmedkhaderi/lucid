# Benchmarks

No Lucid performance measurements have been taken yet. Targets below come from the product specification and remain acceptance goals, not results.

| Measure | Target | Verified result |
|---|---:|---|
| Cached factual answer | ≤300 ms | Not measured |
| Typical AI answer | 1–2.5 s | Not measured |
| Initial live transcription | ≤500 ms | Not measured |
| Factual answer accuracy | ≥98% | Not measured |
| Frontend test suite | Pass | 3,926 passed, 2 skipped (254 files) |
| Frontend production build | Pass | Node 24.1.0; existing Tailwind and dynamic-import warnings |
| Rust backend test suite | Pass | 2,001 passed, 24 ignored across all targets |
| macOS native app bundle | Pass | ARM64 `.app`; ad-hoc signature verifies |
| macOS installer image | Pass | 30 MB compressed HFS+ DMG; checksum verified and mounted with app and Applications shortcut; notarization untested |

Future measurements must record hardware, platform, model/provider, corpus, sample count, and calculation method. Never represent targets as achieved results.
