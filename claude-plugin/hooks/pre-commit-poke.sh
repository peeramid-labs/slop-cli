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

# Token-walk to detect a real `git commit` invocation, not just any
# command that happens to contain the word "commit" in a flag value
# (e.g. `slop apply --no-commit`, `git config -e commit.template`).
# Find the first `git` token; skip option-style tokens (anything
# starting with `-`); the next non-option token must be `commit`.
is_git_commit() {
  # shellcheck disable=SC2086
  set -- $1
  found_git=0
  while [ $# -gt 0 ]; do
    if [ "$found_git" -eq 0 ]; then
      case "$1" in
        git) found_git=1 ;;
      esac
      shift
      continue
    fi
    case "$1" in
      -*) shift; continue ;;
      commit) return 0 ;;
      *) return 1 ;;
    esac
  done
  return 1
}

if ! is_git_commit "$cmd"; then
  exit 0
fi

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

# Three verdict tiers from `slop poke`:
#   LGTM   — no findings. Pass.
#   MARKED — findings exist but every TODO splice is already in source
#            (render_patch produced no actionable hunks). Nothing to
#            fix; pass.
#   SLOP   — findings + a real patch to apply. Block.
#
# Server v0.7+ ships the verdict tier on stderr as
#   "slop poke: LGTM (…)"  /  "slop poke: MARKED — N hit… (…)"  /
#   "slop poke: SLOP — N hits (…)"
# Patch on stdout is empty for LGTM, empty for MARKED, non-empty for
# SLOP. Match the tier explicitly so a future render_patch quirk
# (header-only emission, etc.) doesn't cause a MARKED to look like
# a SLOP and block the commit.
verdict_tmp=$(mktemp -t sloppoke-verdict.XXXXXX)
patch=$(slop poke --staged 2>"$verdict_tmp")
slop_rc=$?
verdict=$(cat "$verdict_tmp" 2>/dev/null)
rm -f "$verdict_tmp"

if [ $slop_rc -ne 0 ]; then
  # Real error (network / auth / quota) — don't block, slop poke
  # already wrote to stderr.
  exit 0
fi

case "$verdict" in
  *"slop poke: LGTM"*|*"slop poke: MARKED"*)
    exit 0
    ;;
esac

# Fall through covers SLOP plus any verdict we don't recognise. Only
# block when there's an actionable patch to surface.
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
