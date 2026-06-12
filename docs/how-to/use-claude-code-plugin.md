# How to use the Claude Code plugin

The plugin gives Claude Code three slash commands plus a PreToolUse
hook that auto-gates every `git commit` Claude attempts.

Prerequisites: `slop` already installed (see
[install-from-source](install-from-source.md) or
[the tutorial](../tutorials/01-first-poke.md)) and `slop login` run
once.

## Install

In any Claude Code session:

```text
/plugin marketplace add peeramid-labs/sloppoke
/plugin install sloppoke@peeramid-labs
```

## Use the slash commands

| Command       | Effect                                                    |
|---------------|-----------------------------------------------------------|
| `/slop:poke`  | Runs `slop poke` against the current diff, prints verdict |
| `/slop:apply` | Applies the cached cleanup patch (`--no-commit` by default) |
| `/slop:learn` | Sends one-line feedback to the learning loop              |

## Let the hook auto-gate commits

The PreToolUse hook ships enabled. It intercepts every Bash tool call,
ignores everything except `git commit*`, runs `slop poke --staged`,
and blocks the commit if SLOP is detected. Claude sees the verdict +
patch and can call `/slop:apply` then retry.

## Bypass the hook for one commit

```sh
SLOP_SKIP_HOOK=1 git commit -m "..."
```

## Update to a new version

```text
/plugin marketplace update peeramid-labs
/plugin install sloppoke@peeramid-labs
```

## Disable the hook entirely

In Claude Code settings, toggle the sloppoke plugin off, or set
`SLOP_SKIP_HOOK=1` in your shell rc so the gate never engages.
