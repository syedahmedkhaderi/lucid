# Speech options and local models

Lucid transcribes meeting audio in English. Setup shows only two local speech models and Groq Whisper.

## Local speech

Local downloads are available when Lucid detects macOS or a supported GPU. Download and activate a model from the setup wizard or **Settings → Speech & Models**:

- **Whisper Large v3 Turbo** — about 1.6 GB.
- **NVIDIA Parakeet TDT 0.6B v3** — about 465 MB.

After download, choose the model for your microphone or meeting audio. You can change the active model at any time. To reclaim storage, choose **Delete model** next to a downloaded model and confirm. Lucid shows the size it will remove and prevents deletion while a meeting is recording.

## Groq Whisper

Use Groq Whisper when this device does not support local speech or you prefer cloud processing. Add the Groq API key in setup or **Settings → Speech & Models**. It is stored in the operating system's credential store.

Groq's [published free-plan rate-limit table](https://console.groq.com/docs/rate-limits) currently lists 28,800 audio seconds per day (eight hours) for Whisper Large v3 Turbo. The table says actual limits can vary by organization; check your account's current limits. A key from the same organization does not create a separate quota.

Cloud speech is blocked while **Private Local** mode is active. Switch to **Cloud processing allowed** before sending meeting audio to Groq.

## AI answer keys

Gemini keys are for AI answers, not meeting transcription. Add or replace keys under **Settings → AI Answers**. Lucid stores key values in the operating system's credential store and lets you label and explicitly activate profiles. It does not automatically switch keys after quota errors.

## Scope and evidence

Select up to ten documents before starting a meeting. Lucid uses only the meeting's selected document scope for document-backed answers, cites available sources, and should say when it cannot verify a fact. Review the cited material before using an answer in a consequential conversation.
