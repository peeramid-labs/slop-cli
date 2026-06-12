---
description: Report which slop hook layers are active on this machine (git pre-commit + Claude Code PreToolUse). Defense-in-depth checkpoint.
allowed-tools: Bash
---

Run `slop status` and surface the two-layer defense-in-depth report:

- **Git pre-commit hook** — gates terminal `git commit` invocations
- **Claude Code PreToolUse hook** — gates `git commit` Bash calls made by Claude (this plugin)

Both should be ACTIVE so an agent has to defeat two independent gates
(`--no-verify` for the git hook, `SLOP_SKIP_HOOK=1` for the Claude
hook). If only one is active, surface the missing-layer install command
verbatim.

Never run any `slop install-hook` variants on the user's behalf without
explicit consent — installing global git hooks affects every repo on
the machine.
