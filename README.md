# Lucid

Lucid is a desktop meeting assistant for checking facts against documents you select before a meeting and preparing concise, cited answers to questions asked aloud. Meeting transcripts, uploaded documents, and answers are stored locally by default.

## Current status

Lucid is an active development build. The macOS app launches on Apple Silicon. Windows packaging and audio capture still need verification on a Windows machine. Local speech models have been exercised with synthesized speech through a live system-audio meeting session; a human-participant call has not been tested. See [PROGRESS.md](PROGRESS.md) and [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md).

## What it does

- Captures your microphone and meeting audio as separate channels.
- Lets you choose up to ten documents for a meeting, then answers with citations from that scope and says when it cannot verify an answer.
- Transcribes and answers in English.
- Offers Whisper Large v3 Turbo or NVIDIA Parakeet TDT 0.6B v3 as in-app local downloads on a Mac or a detected GPU. Delete downloaded models from Speech & Models to reclaim disk space.
- Offers Groq Whisper when local transcription is unavailable or you prefer cloud speech. Cloud use requires a Groq key and must be enabled in Privacy settings.
- Stores provider secrets in the operating system credential store. Gemini LLM keys can be added and managed as separate profiles in the AI Answers settings.

## Set up

Install Node.js 22.22.2+ or 24.15.0+, Rust stable, CMake, and your platform's desktop build tools. Then run one command from the project directory:

    npm run setup

On macOS this builds and installs Lucid.app in /Applications. On Windows it builds the NSIS installer in src-tauri/target/release/bundle/nsis/. Use npm run setup -- --dry-run to check prerequisites first.

## Privacy and credentials

Lucid starts in **Private Local** mode, which blocks cloud speech and AI requests. **Cloud processing allowed** may send selected audio, transcript text, or document excerpts to the configured service and may incur charges. Check the provider's current limits and your organization's data rules before enabling it.

Google applies Gemini quotas at the project level, so multiple keys from one project do not multiply its quota. Lucid does not rotate keys automatically after quota failures. Keys pasted into the original request were not copied into this repository or tested; revoke them at their provider and replace them with new credentials.

## Project docs

- [Getting started](docs/user-guide/getting-started.md)
- [Speech options and local model management](docs/user-guide/ai-providers.md)
- [Audio setup](docs/user-guide/audio-setup.md)
- [Development](DEVELOPMENT.md)
- [Progress and test results](PROGRESS.md)
- [Known limitations](KNOWN_LIMITATIONS.md)
- [Security](SECURITY.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)
