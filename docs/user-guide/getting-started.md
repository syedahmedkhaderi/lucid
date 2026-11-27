# Getting started

Lucid helps you use selected documents to check facts and prepare answers during online meetings. Transcription and generated answers are English-only.

## First setup

1. Open Lucid and complete the setup wizard.
2. Choose Whisper Large v3 Turbo or NVIDIA Parakeet TDT 0.6B v3 for local speech when Lucid detects a Mac or a GPU. Download and activate a model in the wizard or Speech & Models settings. Delete it there later to reclaim disk space.
3. On a device without supported local hardware, choose Groq Whisper and add your Groq API key in the setup screen. Groq's account limits vary; its [published free plan](https://console.groq.com/docs/rate-limits) currently lists up to eight audio hours per day.
4. Select your microphone for **You** and the meeting app's output device for **Them**. Grant microphone and system-audio permissions if macOS asks.
5. Add the documents you expect to use. Lucid supports PDF, DOCX, PPTX, TXT, and Markdown; choose no more than ten documents for a meeting.
6. Start the meeting. Ask Lucid to verify a point or suggest a reply; answers use the selected document and meeting scope and include citations when evidence is available.

## Privacy

Lucid starts in **Private Local** mode. Local speech models keep speech on the device; Private Local also blocks cloud speech and cloud AI requests. To use Groq or a cloud AI provider, switch to **Cloud processing allowed** before a meeting. The app will show when a service sends meeting audio or text to a provider.

## Current limits

The Mac app has been launched on Apple Silicon. Windows packaging and audio capture have not been tested on a Windows machine. Local speech was tested with synthesized utterances in a live private session; a human-participant meeting has not been tested. Gemini Live speech is not offered in the current speech setup. See [Progress](../../PROGRESS.md) and [Known limitations](../../KNOWN_LIMITATIONS.md).

## More

- [Speech options and local model management](ai-providers.md)
- [Audio setup](audio-setup.md)
- [Using documents](rag-context.md)
- [Troubleshooting](troubleshooting.md)
