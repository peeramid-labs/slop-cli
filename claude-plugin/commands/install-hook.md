---
description: Install the slop git pre-commit hook so terminal commits are also gated (defense-in-depth alongside this plugin's PreToolUse hook)
allowed-tools: Bash
---

Install the slop pre-commit hook so commits made directly in the
terminal (outside Claude Code) are gated by `slop poke --staged`. This
plugin's PreToolUse hook only catches `git commit` invocations Claude
makes via the Bash tool — a coding agent that shells out, or a human
running `git commit` in a real terminal, bypasses it. Installing the
git-level hook adds the second layer.

Pick the scope based on what the user wants:
- Current repo only → `slop install-hook`
- Every repo on the machine → `slop install-hook --global`
- Overwrite an existing hook → add `--force`

`slop install-hook` ends by printing the defense-in-depth status. Two
ACTIVE lines is the target — agents have to defeat BOTH gates
(`--no-verify` for the git hook AND `SLOP_SKIP_HOOK=1` for the Claude
hook) to slip slop past.

After running:
- On success surface the install path + the printed status block.
- On error surface the message verbatim — common failures are
  'already installed' (suggest `--force`) and 'not in a git repo'
  (cd into one first).

NEVER pass `--force` without explicit consent — an existing hook may
belong to another tool (husky, pre-commit framework, lefthook).
