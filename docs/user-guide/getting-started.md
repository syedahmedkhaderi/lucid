# Getting started with Lucid

Lucid is a desktop meeting assistant built on Tauri, Rust, and React. This checkout is an Apple Silicon macOS development build. Windows is the intended production platform, but a Windows build and audio test have not been run.

## Open this build

The project produces an app bundle at `src-tauri/target/release/bundle/macos/Lucid.app` and a disk image at `Lucid_0.1.0_macOS_arm64.dmg` in the project root. The bundle is ad hoc signed and the disk image is not notarized; Gatekeeper behavior has not been verified.

## First launch

1. Open Lucid and complete the setup wizard.
2. Lucid starts in **Settings → Privacy → Private Local**. Choose offline speech models for **You** and **Them** in **Settings → Audio & Devices** and **Settings → STT Providers**. Download the selected models first. Web Speech, Windows Speech dictation, cloud providers, subscription CLIs, and remote model endpoints are blocked in this mode. Windows dictation is blocked because its dictation constraint can use Microsoft's online speech service ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/apps/develop/input/specify-the-speech-recognizer-language)).
3. Choose your microphone for **You** and your system output for **Them**. Grant macOS microphone and screen/audio capture permissions if prompted.
4. To use a cloud provider, switch to **Cloud processing allowed** in **Settings → Privacy** after the meeting and current AI requests finish. Cloud processing can send meeting content to the selected provider and may incur charges. For Gemini Live, select **Gemini Live Transcribe**, enter a key in the password field, then choose **Save & Test**. The key is stored through the operating system credential store.
5. Add files to the workspace you will use. Current uploads support PDF, DOCX, PPTX, TXT, and Markdown; files must be under 50 MB each, with up to ten document entries and 500 MiB total source-file size per workspace. These limits are currently workspace-wide rather than meeting-specific.
6. Wait for indexing to finish, then choose **Start Meeting**.

## Speech providers

Gemini Live streams 16 kHz audio to Google's cloud service and supports interim and final transcripts plus automatic language detection. Google documents a 10-minute session limit; Lucid refreshes the session before that limit. The provider has not been tested against a real Gemini account or real meeting audio in this checkout.

Google's pricing page says content submitted on the free tier may be used to improve its products. Do not send confidential meeting audio to the free tier. Gemini quotas are applied per project, not per key, so Lucid does not rotate keys to get around quota errors. See [Google's Live Transcription guide](https://ai.google.dev/gemini-api/docs/live-api/live-transcribe), [pricing](https://ai.google.dev/gemini-api/docs/pricing), and [rate limits](https://ai.google.dev/gemini-api/docs/rate-limits).

Private Local also restricts LLM, translation, and document embedding requests to local providers. The app blocks a privacy-mode change while a meeting, audio capture, or content-bearing AI request is active. A local provider alone does not guarantee privacy if the software you install on this device sends data elsewhere.

## Use documents

Open a workspace, choose **Add** in the Library, and select files or a folder. Lucid extracts and indexes supported file text locally. PPTX import preserves slide numbers, slide text, and basic table rows; it does not extract speaker notes, charts, or text embedded in images. Legacy DOC/PPT and scanned-PDF OCR are not supported yet. See [Using Context Intelligence](rag-context.md) for scope and extraction details.

## Start and end a meeting

Open the workspace you want, confirm its audio devices and speech providers, then select **Start Meeting**. The overlay shows the live transcript and meeting assistance. Use its controls to pause or end the meeting. Meeting history and transcripts are stored locally in the app's data directory.

## Before using real meeting material

This development build has not been benchmarked for accented English or Mandarin accuracy, and the private mode has not received an independent security audit. The free Gemini tier may use submitted content to improve Google's products; confirm provider terms and your organization's policy before enabling cloud transcription.

## More guides

- [AI providers](ai-providers.md)
- [Audio setup](audio-setup.md)
- [Workspaces](workspaces.md)
- [Using Context Intelligence](rag-context.md)
- [Troubleshooting](troubleshooting.md)
- [Keyboard shortcuts](keyboard-shortcuts.md)
