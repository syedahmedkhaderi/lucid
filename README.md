# Lucid

Lucid is a local-first desktop meeting assistant for live transcription, document-grounded answers, and searchable meeting notes. It is adapted from the MIT-licensed NexQ v3.0.0 foundation.

## Status

This checkout is an active development build, not a professional-pilot release. The frontend passes 3,940 tests (2 skipped), all Rust targets pass 2,037 tests (24 ignored), and the installed macOS app launches into the meeting dashboard. Parakeet completed a private, live system-audio smoke test using two synthesized utterances; Whisper's live test returned text but had noticeable recognition errors. Windows packaging and launch checks are configured in GitHub Actions; no Windows runner result is available yet. Gemini Live Transcribe remains unverified against a live provider account. See [`PROGRESS.md`](PROGRESS.md) and [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) for details.

## Current capabilities inherited from NexQ

- Tauri desktop shell with a launcher and always-on-top meeting window, including a compact top-center triangle.
- Microphone and system audio capture paths, live transcripts, question detection, and streamed AI answers.
- Local SQLite meeting history and workspace-scoped document retrieval.
- A scope-bound live brief that caches up to six source-linked facts and rejects facts whose cited passage lacks the stated numbers or names.
- Local speech/model options and configurable cloud providers.
- In-app download and activation for only Whisper Large v3 Turbo and NVIDIA Parakeet TDT 0.6B v3 on supported macOS/GPU systems, with Groq Whisper as the cloud alternative on unsupported hardware.
- Gemini Live multilingual speech transcription with interim/final text, automatic language detection, and pre-limit session refresh.
- Local PPTX slide text and basic table-row extraction with slide markers.
- Individually validated and tab-separated bulk credential profiles stored in the OS credential store, with manual selection and no quota-triggered rotation.

These features are inherited and selectively rebranded. The compact overlay has unit-tested placement math but still needs interactive runtime verification. Do not infer production readiness from the upstream README.

## Build from source

Install Node.js 22.22.2+ or 24.15.0+, Rust stable, CMake, and the platform toolchain (Xcode Command Line Tools on macOS; Visual Studio C++ Build Tools on Windows). On Windows, also install LLVM/Clang, NSIS, and the WebView2 Runtime. Then, from the project directory, one command builds and installs the app or produces the Windows installer:

```sh
npm run setup
```

On macOS this installs `Lucid.app` in `/Applications`. On Windows it builds an NSIS installer in `src-tauri/target/release/bundle/nsis/`. Use `npm run setup -- --dry-run` to check prerequisites and see the planned commands without building. For development and individual test commands, see [`DEVELOPMENT.md`](DEVELOPMENT.md).

## Privacy and credentials

Lucid starts in **Private Local** mode. It blocks cloud speech, cloud LLM and translation requests, subscription CLIs, remote model endpoints, Gemini context uploads, and remote embeddings. It permits offline speech engines and Ollama or LM Studio bound to a loopback address. Change this in **Settings → Privacy** after the meeting and any current AI requests finish. **Cloud processing allowed** may send audio, transcripts, document excerpts, notes, or images to the selected provider and may incur charges. Check provider and organizational data policies before enabling it. Private Local cannot verify the behavior of software you install on your device, and it does not encrypt meeting records stored on disk.

Provider keys belong in the app's OS credential store and must never be committed. Gemini keys can be added one at a time or as tab-separated profiles in **Settings → LLM Provider**; each is validated before storage and can be explicitly selected. Saved values are write-only from the web UI. The keys pasted in the originating request were not copied into this repository or tested; treat them as exposed and revoke or rotate them at their provider.

Gemini quotas are project-level. Multiple keys from one project share its quota, so Lucid does not switch keys automatically after a quota error. See [`DECISIONS.md`](DECISIONS.md).

## Project docs

- [`ARCHITECTURE.md`](ARCHITECTURE.md)
- [`DEVELOPMENT.md`](DEVELOPMENT.md)
- [`SECURITY.md`](SECURITY.md)
- [`PROGRESS.md`](PROGRESS.md)
- [`BENCHMARKS.md`](BENCHMARKS.md)
- [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md)
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
