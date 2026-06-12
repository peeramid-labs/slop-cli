#!/usr/bin/env sh
# Pre-commit slop poke gate. Fires before every Bash tool call;
# only intercepts `git commit` variants. Everything else passes
# through untouched.
#
# Bypass once: SLOP_SKIP_HOOK=1 git commit -m "..."
# Bypass globally: export SLOP_SKIP_HOOK=1 in your shell rc

set -u

# Trace marker so the operator can confirm whether Claude Code is
# actually invoking the hook at all. Writes one line per
# invocation regardless of branch taken. Suppress with
# SLOP_NO_HOOK_TRACE=1 once the hook is verified live.
if [ "${SLOP_NO_HOOK_TRACE:-0}" != "1" ]; then
  {
    printf '%s pid=%s sloppoke pre-commit hook invoked\n' "$(date '+%Y-%m-%dT%H:%M:%S%z')" "$$"
  } >> /tmp/sloppoke-hook-trace.log 2>/dev/null || true
fi

payload=$(cat)

if [ "${SLOP_SKIP_HOOK:-0}" = "1" ]; then
  exit 0
fi

# Extract tool_input.command from the PreToolUse JSON payload.
# Prefer jq; fall back to python3 (universally present on macOS +
# every modern Linux). The earlier pure-sed fallback was BSD-sed
# incompatible so the hook silently no-op'd on macOS without jq.
extract_command() {
  if command -v jq >/dev/null 2>&1; then
    printf '%s' "$payload" | jq -r '.tool_input.command // empty'
    return
  fi
  if command -v python3 >/dev/null 2>&1; then
    printf '%s' "$payload" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('command',''))"
    return
  fi
  # Last-ditch grep for the command field. Tolerates simple quoting
  # but not embedded escaped quotes — those should be rare in the
  # git-commit path we actually care about.
  printf '%s' "$payload" \
    | tr ',' '\n' \
    | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' \
    | head -1 \
    | sed -e 's/^"command"[[:space:]]*:[[:space:]]*"//' -e 's/"$//'
}

cmd=$(extract_command)

case "$cmd" in
  *"git commit"*|*"git "*"commit"*) ;;
  *) exit 0 ;;
esac

if ! command -v slop >/dev/null 2>&1; then
  exit 0
fi

# Claude Code fires PreToolUse from the SESSION cwd, not from the
# command's effective cwd. Commands shaped 'cd /path && git commit'
# would otherwise run slop poke in the wrong directory and find
# nothing staged. Walk a leading 'cd X &&' / 'cd X ;' prefix and
# chdir into it before invoking slop.
target_dir=$(printf '%s' "$cmd" | sed -n 's|^[[:space:]]*cd[[:space:]]\{1,\}\([^[:space:]&;|]\{1,\}\).*|\1|p' | head -1)
if [ -n "$target_dir" ]; then
  # Strip surrounding single / double quotes.
  target_dir=$(printf '%s' "$target_dir" | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
  if [ -d "$target_dir" ]; then
    cd "$target_dir" || exit 0
  fi
fi

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  exit 0
fi

# slop poke --staged on v0.7+ writes the patch to stdout (or empty
# on LGTM) and the verdict to stderr. Exit code is 0 either way; the
# stdout content is the signal. Older builds also stuffed a verdict
# line into stdout, so accept either: empty stdout → LGTM. Anything
# else → SLOP and block.
patch=$(slop poke --staged 2>/dev/null)
slop_rc=$?

if [ $slop_rc -ne 0 ]; then
  # Real error (network / auth / quota) — don't block, slop poke
  # already wrote to stderr.
  exit 0
fi

if [ -z "${patch}" ]; then
  exit 0
fi

{
  printf 'slop poke flagged the staged diff before commit:\n\n'
  printf '%s\n\n' "$patch"
  printf 'Run /slop:apply to splice the suggested TODOs, then retry the commit.\n'
  printf '(Bypass once with SLOP_SKIP_HOOK=1 git commit ...)\n'
} >&2
exit 2
