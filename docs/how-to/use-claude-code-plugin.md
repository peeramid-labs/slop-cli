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

## Mental model: slop is a debt MARKER, not a rewriter

Past agents have mistaken `slop apply` for a `clippy --fix` /
`rustfmt` / `eslint --fix` class tool. It is not. `slop apply` does
**not** rewrite logic, rename identifiers, or reformat bodies. It
splices `// TODO(slop): …` comments **above** the flagged line and
deletes lines only at SafeDelete tier (empty comment slop,
hallucinated `console.log`, dead emoji, etc.). The flagged line
itself stays exactly as written.

If the diff looks "noisy" (many TODOs added), the catalog has
converted debt that was hiding in the file volume into precise
(file, line, category) breadcrumbs. **The markers are the value**,
not the cost. Stripping them as cleanup defeats the entire purpose.

## Why the `TODO(slop):` markers matter

When `slop apply` runs, two things happen:

1. **Safe-delete findings** (slop comments, redundant `console.log`,
   etc.) are stripped out of the diff.
2. **Semantic findings** (questionable naming, untested branches,
   suspicious bounds, redundant guards) get a `// TODO(slop): …`
   marker spliced above the line. The code itself is left intact.

Those markers are not noise. They are precise (file, line, category)
hints — the catalog already pattern-matched, the agent just needs to
act. After every apply, the assistant should:

```sh
git grep -n "TODO(slop)"
```

Then triage each hit. Two paths:

- **Fix in this change.** Small, local, in-scope (off-by-one,
  redundant null check, missing test for a branch you just wrote).
  Fix it. The TODO line vanishes on the next poke.
- **File as backlog.** Larger refactor, out-of-scope, cross-file.
  Add to `.issues/open/TXXX-…md` if the repo uses that convention,
  or open a real ticket (`gh issue create`, Linear, JIRA, whatever).
  **Leave the TODO(slop) line in place.** Removing it without either
  fixing the code or filing a followup loses the signal — and the
  next poke will splice it back in anyway.

If a marker is genuinely wrong for your codebase, that is what
`slop learn "<why>"` is for. The catalog gets sharper from
calibrated feedback, not from stripped markers.

The plugin's prompt already nudges Claude through this triage after
`/slop:apply`. The flow is: **apply → grep → triage → fix or file →
poke again**.

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
