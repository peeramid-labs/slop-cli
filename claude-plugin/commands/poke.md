---
description: Run `slop poke` against the current diff and report the verdict
allowed-tools: Bash
---

Scan the current git changes through sloppoke and report the verdict.

Pick the scope based on context:
- Staged-only context → `slop poke --staged`
- Working-tree (default review) → `slop poke`
- Specific range → `slop poke --range <BASE>..<HEAD>`
- Remote repo → `slop poke --gh <org/repo> --range HEAD~5..HEAD`

After the scan:
- If exit code is 0 with empty stdout → reply with `LGTM` and one short line of context.
- If stdout has a patch → show the patch (or its hunk count) to the user, and offer `/slop:apply` as the next step.
- If stderr has an error → surface it verbatim, do not try to fix it silently.

NEVER commit, push, or amend HEAD on the user's behalf. The verdict is informational; the user runs `/slop:apply` when they decide.
