# Decisions

## ADR-001: Adapt NexQ rather than rewrite

**Status:** Accepted

Lucid starts from NexQ `v3.0.0` at `d83db586024270bfd0bcf7221a55275b104c4246`, MIT licensed. The release already has Tauri/Rust desktop windows, audio capture, local retrieval, provider routing, meeting history, and a large tested frontend. Retaining it avoids replacing functioning infrastructure. Preserve its upstream remote and license notices.

## ADR-002: Do not rotate keys to bypass a project's quota

**Status:** Accepted

Gemini limits apply at project level. Multiple API keys for one project share that project's quota, so cycling them does not increase allowance and is not a valid fallback. Credentials may be stored securely and selected by provider/project identity; a separate project may be used only when it is independently configured and permitted. On quota exhaustion, back off or use an explicitly selected local provider. Never switch a private meeting to cloud inference silently.

## ADR-003: Keep upstream updates disabled until Lucid owns a release channel

**Status:** Accepted

The pinned source points its signed updater at NexQ's release server and public key. Those artifacts are not Lucid releases. Remove that endpoint before shipping a Lucid build; re-enable updates only after Lucid has its own signed update key and release manifest.
