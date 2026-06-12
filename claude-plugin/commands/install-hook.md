---
description: Install the slop git pre-commit hook in the current repo (or globally) so every terminal commit is gated too
allowed-tools: Bash
---

Install the slop pre-commit hook so commits made directly in the
terminal (outside Claude Code) are also gated by `slop poke --staged`.

Pick the scope based on what the user wants:
- Current repo only → `slop install-hook`
- Every repo on the machine → `slop install-hook --global`
- Overwrite an existing hook → add `--force`

After running:
- On success the CLI prints the install path. Reply with one line
  confirming where it landed and how to bypass once
  (`SLOP_SKIP_HOOK=1 git commit ...`).
- On error surface the message verbatim — the most common failures
  are 'already installed' (suggest `--force`) and 'not in a git
  repo' (cd into one first).

NEVER pass `--force` without asking the user, except when they
explicitly request to overwrite. An existing hook may belong to
another tool (husky, pre-commit framework, lefthook).
