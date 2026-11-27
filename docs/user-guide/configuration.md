# Settings

Lucid keeps its common setup short. **Settings** has six main sections; less common controls are grouped under **Advanced**.

## Audio & Devices

Choose your microphone for **You** and the output device that plays the meeting for **Them**. Set each side to a downloaded local model or Groq Whisper. Transcription language is fixed to English.

## Speech & Models

Choose one of the supported speech options:

- **Whisper Large v3 Turbo** or **NVIDIA Parakeet TDT 0.6B v3** for local transcription on a Mac or a supported GPU.
- **Groq Whisper** for cloud transcription.

Download, activate, and delete local models here. Deleting a model removes its files from this device to free space. Add the Groq key here when you choose the cloud provider.

## AI Answers

Choose an answer provider and model. API keys are saved in the operating system credential store. Gemini keys can be saved as labeled profiles and explicitly activated; quota failures do not trigger automatic key rotation.

## Documents

Add files to your workspace, wait for indexing, and select up to ten sources when preparing a meeting. Answers use the selected meeting scope and include citations when source evidence is available.

Supported uploads are PDF, DOCX, PPTX, TXT, and Markdown. Legacy DOC/PPT and OCR for scanned PDFs are not available.

## Privacy

**Private Local** is the default and blocks cloud speech and AI requests. **Cloud processing allowed** must be selected before sending meeting audio, transcript text, or document excerpts to a cloud provider.

## Advanced

Less common options stay collapsed until selected. The app does not offer translation or non-English transcription.
