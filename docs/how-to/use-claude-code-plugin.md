# How to use the Claude Code plugin

The plugin gives Claude Code four slash commands plus two hooks: a
PreToolUse hook that auto-gates every `git commit` Claude attempts,
and a SessionStart tip that nudges you to install the terminal-level
hook too.

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

| Command                | Effect                                                                |
|------------------------|-----------------------------------------------------------------------|
| `/slop:poke`           | Runs `slop poke` against the current diff, prints verdict             |
| `/slop:apply`          | Applies the cached cleanup patch (`--no-commit` by default)           |
| `/slop:learn`          | Sends one-line feedback to the learning loop                          |
| `/slop:install-hook`   | Installs the slop git pre-commit hook (`--global` for every repo)     |

## How commits get gated

Two surfaces, two hooks:

| Surface                | Hook                       | What runs                                   |
|------------------------|----------------------------|---------------------------------------------|
| Claude Code `git commit` | Plugin PreToolUse on Bash | `slop poke --staged` before the commit runs |
| Terminal `git commit`  | `.git/hooks/pre-commit`    | Same — once `/slop:install-hook` is run     |

The PreToolUse hook ships enabled and intercepts every Bash tool call
Claude makes. It ignores everything except `git commit*`, parses any
`cd <path> &&` prefix so it lands in the right repo, runs
`slop poke --staged`, and blocks the commit on SLOP. Claude sees the
verdict + patch and can call `/slop:apply` then retry.

The terminal hook is opt-in. The SessionStart tip prints one line
when the hook is missing:

```
tip: run /slop:install-hook so terminal git commit is gated too.
SLOP_HIDE_HOOK_TIP=1 to hide.
```

Run `/slop:install-hook` (current repo) or `/slop:install-hook --global`
(every repo on the machine) when you're ready. See
[install-pre-commit-hook](install-pre-commit-hook.md) for scope details.

## Bypass for one commit

```sh
SLOP_SKIP_HOOK=1 git commit -m "..."
```

Works for both the plugin's PreToolUse hook and the terminal hook.

## Hide the SessionStart tip

```sh
export SLOP_HIDE_HOOK_TIP=1   # in your shell rc
```

## Update to a new version

```text
/plugin marketplace update peeramid-labs
/plugin install sloppoke@peeramid-labs
```

If the marketplace shows a new version but the installed copy stays
on the old one, uninstall then install:

```text
/plugin uninstall sloppoke@peeramid-labs
/plugin install sloppoke@peeramid-labs
```

## Disable the hook entirely

In Claude Code settings, toggle the sloppoke plugin off, or set
`SLOP_SKIP_HOOK=1` in your shell rc so the gate never engages.
