# Lucid contributor guide

Lucid is a local-first desktop meeting assistant derived from the MIT-licensed NexQ v3.0.0 release. Preserve upstream notices and history.

## Rules

- Keep all source and project data in this repository. Never check in API keys, meeting documents, transcripts, recordings, model weights, databases, or build output.
- Store provider credentials only through the operating-system credential store. Do not log credential values or transcript content.
- Respect project-level provider quotas. Do not rotate keys from one project to evade quotas. Switch only after an explicit quota failure, and only to a key configured for a distinct project with its own quota.
- Make cloud processing opt-in and visible. Do not silently send content from a private meeting to a cloud provider.
- Ground answers in scoped meeting material, cite source locations, and abstain when evidence is missing or conflicting.
- Keep workspace and meeting source scope explicit. Do not broaden a request to other workspaces by default.
- Before changing a subsystem, inspect its Rust implementation, frontend wrapper, event contract, and tests. Update the Rust and TypeScript contract together.
- Run focused tests for each code change, then `npm test`, `npm run build`, and `cargo test --lib` at milestones. Record results in `PROGRESS.md`; do not claim unrun checks.
- Keep Windows packaging claims separate from macOS results. Windows builds and audio capture need Windows-host verification.
- No publishing or pushing without explicit authorization.

## Project map

- `src/`: React and TypeScript interface.
- `src-tauri/src/`: Rust desktop, audio, speech, retrieval, database, and provider code.
- `docs/ARCHITECTURE.md`: upstream subsystem map.
- `docs/DEVELOPMENT.md`: toolchain and build commands.
- `PROGRESS.md`: verified implementation status and next work.
- `KNOWN_LIMITATIONS.md`: uncovered product and platform gaps.
