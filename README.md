# Lucid

Lucid is a local-first desktop meeting assistant for live transcription, document-grounded answers, and searchable meeting notes. It is adapted from the MIT-licensed NexQ v3.0.0 foundation.

## Status

This checkout is an active development build, not a professional-pilot release. The adapted frontend passes 3,925 tests, all Rust targets pass 1,986 tests, and the production frontend and macOS installer image build. See [`PROGRESS.md`](PROGRESS.md) and [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) for verified status and gaps.

## Current capabilities inherited from NexQ

- Tauri desktop shell with a launcher and always-on-top meeting window, including a compact top-center triangle.
- Microphone and system audio capture paths, live transcripts, question detection, and streamed AI answers.
- Local SQLite meeting history and workspace-scoped document retrieval.
- Local speech/model options and configurable cloud providers.
- OS credential-store integration for provider secrets.

These features are inherited and selectively rebranded. The compact overlay has unit-tested placement math but still needs interactive runtime verification. Do not infer production readiness from the upstream README.

## Build from source

See [`DEVELOPMENT.md`](DEVELOPMENT.md). On macOS, install Node.js 22.22.2+ or 24.15.0+, Rust stable, Xcode Command Line Tools, and CMake, then:

```sh
npm ci
npm test
npm run build
source scripts/macos-env.sh
npx tauri dev
```

The repository's original `build-mac.sh` creates a macOS development package after native dependencies are staged. Windows packaging must be run and tested on Windows; it has not been verified from this Mac.

## Privacy and credentials

Cloud inference sends selected transcript or document context to the configured provider. The app does not yet enforce the required private-local boundary, so do not use it for confidential meetings. Provider keys belong in the app's OS credential store and must never be committed. The keys pasted in the originating request were not copied into this repository or tested; treat them as exposed and revoke or rotate them at their provider.

Gemini quotas are project-level. Multiple keys from one project share its quota, so Lucid will not use key cycling to bypass limits. Multi-key project-aware management is not implemented yet. See [`DECISIONS.md`](DECISIONS.md).

## Project docs

- [`ARCHITECTURE.md`](ARCHITECTURE.md)
- [`DEVELOPMENT.md`](DEVELOPMENT.md)
- [`SECURITY.md`](SECURITY.md)
- [`PROGRESS.md`](PROGRESS.md)
- [`BENCHMARKS.md`](BENCHMARKS.md)
- [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md)
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
