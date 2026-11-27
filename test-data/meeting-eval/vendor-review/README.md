# Vendor review meeting evaluation

This fixture tests a live assistant session with a manager asking questions, an employee explaining the documents, and Lucid preparing concise source-grounded answers for the employee.

## Files to attach

Attach both files in this directory to the meeting:

- `accounts.csv`: 500 synthetic supplier records. ACCT-0427 is the target row.
- `policy-brief.md`: definitions, risk bands, renewal gate and the target case note.

Suggested optional meeting context:

> Q4 supplier renewal review. The CSV is authoritative for account figures; the policy brief is authoritative for definitions and approval gates. Match account IDs exactly, distinguish total open balance from the overdue portion, and do not present a recommendation as an approval.

## Dialogue

Read `meeting-dialogue.md` in sequence, using two distinct voices for Jordan (manager) and Priya (employee). For each Jordan question, verify that Lucid detects it and that Assist answers from the attached files. In particular, confirm the exact ACCT-0427 figures and the three recovery thresholds plus sign-off requirements.

The data and dialogue are fictional and contain no real customer or employee information. Audio can be generated locally with macOS `say`; no dataset audio recording needs to be committed.

## Test report

See [EVALUATION.md](EVALUATION.md) for the actual in-app Gemini meeting runs, response times, corrected lookup behavior, and the final macOS audio failure. The test demonstrates a real Lucid meeting workflow with synthetic role-play audio; it is not a meeting with human participants.
