#!/usr/bin/env sh
# Plugin SessionStart hook. Detects whether the user has installed
# the git-level pre-commit hook (`slop install-hook` or
# `slop install-hook --global`). If not, emits a one-line status
# nudge so terminal commits get the same gate Claude Code commits
# get — without forcing an install behind the user's back.
#
# Silent when the hook is already in place. Silent when the user
# sets SLOP_HIDE_HOOK_TIP=1.

set -u

if [ "${SLOP_HIDE_HOOK_TIP:-0}" = "1" ]; then
  exit 0
fi

if ! command -v slop >/dev/null 2>&1; then
  exit 0
fi

# Global install: ~/.config/slop/git-hooks/pre-commit + core.hooksPath.
global_hook="$HOME/.config/slop/git-hooks/pre-commit"
if [ -x "$global_hook" ]; then
  if git config --global --get core.hooksPath 2>/dev/null \
      | grep -q "/.config/slop/git-hooks"; then
    exit 0
  fi
fi

# Per-repo install: .git/hooks/pre-commit with our marker. Cheap
# grep; if either fails we just treat the hook as not installed.
if git rev-parse --git-dir >/dev/null 2>&1; then
  hook_path="$(git rev-parse --git-dir)/hooks/pre-commit"
  if [ -x "$hook_path" ] && grep -q "slop poke --staged" "$hook_path" 2>/dev/null; then
    exit 0
  fi
fi

cat <<'EOF'
{"continue": true, "statusMessage": "tip: run /slop:install-hook so terminal `git commit` is gated too. SLOP_HIDE_HOOK_TIP=1 to hide."}
EOF
