---
description: Send feedback to the slop learning loop via `slop learn`
allowed-tools: Bash
---

Submit a short feedback note to the sloppoke server so the engine calibrates for the user's account and project.

Use this when:
- The most recent scan flagged something that is NOT slop in the user's codebase context (false positive)
- The most recent scan MISSED something the user expected it to catch (false negative)
- The user wants to share project-specific intent ("we always name our top-level handler `Manager` on purpose")

Run:
```
slop learn "<one-sentence note explaining the FP / FN / intent. include file/line if relevant.>"
```

Keep the note short and specific. Quote the exact identifier or pattern at issue. Do not paraphrase code blocks into the message.

Don't auto-submit feedback the user didn't actually ask for. Always confirm the wording before sending.
