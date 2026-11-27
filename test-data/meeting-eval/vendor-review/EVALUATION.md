# Lucid vendor review evaluation

Updated: 2026-11-27

## Fixture

This is a synthetic, deterministic meeting test. It contains no real vendors, employees, balances, or customer data.

- `accounts.csv`: 500 fake supplier records. The target `ACCT-0427` is the 428th data record (CSV line 429 including the header).
- `policy-brief.md`: synthetic risk definitions, renewal gates, and approval roles.
- `meeting-dialogue.md`: a six-turn manager/employee script with expected answers.
- Optional meeting context specified which source controls amounts versus policy and instructed the assistant to match account IDs exactly and not turn a recommendation into approval.

## Expected facts

For `ACCT-0427`: $18,420 total open balance; $12,600 overdue more than 30 days; 82% on-time delivery; 4 late deliveries; 3 recent SLA breaches; risk score 48 (High); hold renewal; review route variance weekly.

Before a renewal can be approved at the 30-day gate: on-time delivery must be at least 95%; the amount overdue more than 30 days must be zero; there can be no more than one SLA breach during the rolling 30-day period; Finance must reconcile the balance; and the Procurement Operations manager must sign in writing.

## In-app runs

The app was opened from `/Applications/Lucid.app`; each run used the actual meeting interface, Gemini `gemini-3.5-flash-lite`, the selected workspace documents, and the macOS Parakeet TDT 0.6B v3 transcription profile. The fixtures were spoken by local macOS synthesized voices and played through system audio. No audio or document was sent to a speech provider; selected excerpts and questions were sent to the configured Gemini text-answer provider under Cloud processing allowed.

| Session | Result |
| --- | --- |
| 2026-11-27 18:13 | The multi-part account question was transcribed. Gemini returned the correct balance, overdue portion, delivery rate, late deliveries, SLA breaches, risk score and band, citing `accounts.csv` row 428. Response appeared in about 1.6 s. |
| 2026-11-27 18:13 | A synthetic employee explanation was transcribed as a separate speech segment. |
| 2026-11-27 18:24 | The policy question was initially truncated by speech recognition. After fixing duration text such as `30-day` being misidentified as a record ID, Gemini answered the 95% delivery, zero-over-30-days overdue, maximum-one-breach, reconciliation, and written sign-off requirements from `policy-brief.md` in about 1.8 s. |
| 2026-11-27 18:24 | “What should we do immediately?” returned a weekly route-variance/recovery review and kept renewal on hold, citing the policy brief, in about 1.5 s. |
| 2026-11-27 18:30 | With Auto-Answer enabled, the first account question was answered without selecting the response card, in about 1.2 s. It included overdue and risk drivers but omitted the question’s first requested clause (the total open balance). A follow-up code change now expands a partial detected question using the full transcript utterance when that utterance includes a valid account ID. Unit tests cover this case, but this exact live utterance has not yet passed a post-fix retest. |
| 2026-11-27 18:36 and 18:42 | After restarting Lucid, both audio meters stayed at 0%, device selectors were disabled at “Loading…”, and no transcript appeared during another synthetic manager prompt. macOS `system_profiler` still listed the built-in microphone/speakers; unified logs showed CoreAudio HAL server errors (`0x10004003`). |
| 2026-11-27 18:46 | Outside a meeting, Lucid settings successfully enumerated the built-in microphone and speakers after relaunch. A fresh meeting then started, but both meters again stayed at 0% and replaying the manager prompt produced no transcript. The app log recorded `Starting output-tap recording failed (CoreAudio status 268451843)` and recommended checking System Audio Recording permission. |

## Interpretation and limits

- The selected-document Gemini Q&A path, multi-part account lookup, policy lookup, optional context, and Auto-Answer worked in live app sessions. The response times above are single-run observations, not a latency benchmark.
- This exercises a 500-row exact-ID lookup, not every row, every CSV shape, or every possible natural-language phrasing. The row lookup and partial-question expansion have automated Rust tests.
- Synthetic speech through a shared output route did not provide real speaker separation: both Jordan and Priya were labeled “Them” in the transcript, and one unrelated “Okay” appeared on “You”. Human speakers, an actual conference application, and separate speaker inputs have not been tested.
- The final audio failure occurred after earlier successful capture sessions. Idle device enumeration recovered, but live capture did not; the failing CoreAudio output tap points to System Audio Recording access or tap startup. The app bundle includes `NSAudioCaptureUsageDescription`. The permission's state was not confirmed or changed during this test. Check macOS System Settings → Privacy & Security → Screen & System Audio Recording, allow Lucid if it is off, then restart the app and repeat the meeting test. Apple documents the required usage string and first-use system prompt [here](https://developer.apple.com/documentation/CoreAudio/capturing-system-audio-with-core-audio-taps). Model transcription and meeting recording cannot be called reliable on this host until that retest passes.
- The latest prompt now asks Gemini to include every clause and cite the filename plus CSV row instead of only a generic source number. Its exact filename/row citation change has not yet been rechecked against a successful post-fix live audio run.
- The latest source code passed the complete Rust test suite (2,052 passed, 24 ignored) and a macOS application build. These checks do not replace Windows device tests, a human meeting, or a successful final audio retest.
