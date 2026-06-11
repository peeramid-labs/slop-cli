#!/usr/bin/env sh
# Pre-commit slop poke gate. Fires before every Bash tool call;
# only intercepts `git commit` variants. Everything else passes
# through untouched.
#
# Bypass once: SLOP_SKIP_HOOK=1 git commit -m "..."
# Bypass globally: export SLOP_SKIP_HOOK=1 in your shell rc

set -u

payload=$(cat)

if [ "${SLOP_SKIP_HOOK:-0}" = "1" ]; then
  exit 0
fi

extract_command() {
  if command -v jq >/dev/null 2>&1; then
    printf '%s' "$payload" | jq -r '.tool_input.command // empty'
  else
    printf '%s' "$payload" \
      | sed -n 's/.*"command"[[:space:]]*:[[:space:]]*"\(\([^\\"]\|\\.\)*\)".*/\1/p' \
      | head -1
  fi
}

cmd=$(extract_command)

case "$cmd" in
  *"git commit"*|*"git "*"commit"*) ;;
  *) exit 0 ;;
esac

if ! command -v slop >/dev/null 2>&1; then
  exit 0
fi

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  exit 0
fi

output=$(slop poke --staged 2>&1)

case "$output" in
  *LGTM*) exit 0 ;;
esac

{
  printf 'slop poke flagged the staged diff before commit:\n\n'
  printf '%s\n\n' "$output"
  printf 'Run /slop:apply to splice the suggested TODOs, then retry the commit.\n'
  printf '(Bypass once with SLOP_SKIP_HOOK=1 git commit ...)\n'
} >&2
exit 2
