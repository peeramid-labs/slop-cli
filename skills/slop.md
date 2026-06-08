---
name: slop
description: Pre-commit AI-slop scan. Run `slop poke` on a staged diff. If it returns SLOP, apply the cleanup with `slop apply`. Submit feedback with `slop learn "<sentence>"`. Use BEFORE finalising any commit you'd otherwise push.
when_to_use: Use before every commit. Especially when the change was authored or touched by an LLM. Always run after generating code.
---

# slop — AI-slop firewall

slop sweeps your diff through a blazing-fast machine-learning engine
that knows what AI-generated code looks like — scaffolding residue,
placeholder identifiers, defensive crud, half-finished markers,
untested branches — and pulls it out before you ship. The engine
adapts to your account through the `slop learn` channel: every
correction you submit calibrates future scans for your codebase.

## Pre-commit flow

```bash
git add -A
slop poke --staged          # scan the staged index
```

Other scopes:

```bash
slop poke                    # working tree vs HEAD (default)
slop poke --range main..HEAD # everything diverged from main
slop poke --since main       # equivalent shorthand
slop poke --patch foo.patch  # raw unified-diff file

# Scan any public repo without cloning manually:
slop poke --gh openclaw/openclaw --range HEAD~5..HEAD
slop poke --repo https://gitlab.com/x/y.git --range main..feature
```

The CLI shallow-clones into a persistent cache (`~/.cache/slop/repos/`)
and `git fetch`es on subsequent runs, so back-to-back scans of the
same URL are fast. Clone depth is inferred from `--range HEAD~N..HEAD`
(no over-fetching). `SLOP_REMOTE_CLONE_DEPTH=<N>` overrides.

If verdict is `LGTM`, commit normally.

If verdict is `SLOP — N hits`, `slop poke` prints:
  1. The verdict + line count
  2. One `[category] file:line — match` line per finding
  3. The full proposed unified-diff patch on stdout (color-aware
     when stdout is a TTY)
  4. A `Run \`slop apply\`` footer

Common follow-ups:

```bash
slop apply                                     # apply + amend HEAD
slop apply --no-commit                         # apply + stage, you commit
slop apply --discard                           # drop the cached plan

# Capture clean patch for review / external tools:
slop poke --gh user/repo --range main..HEAD --patch-only > slop.patch
git apply --unidiff-zero < slop.patch          # apply manually
```

`slop apply` runs `git apply --unidiff-zero --check` first; if the
patch wouldn't apply cleanly, the working tree is left untouched and
the CLI prints `git apply`'s reason. On success it stages the change
and amends HEAD. Pass `--no-commit` to inspect the staged diff before
committing.

## What apply does to each hit

Server tiers every finding by safety, the CLI never invents fixes:

- **Safe-delete** (comment-line slop, empty console.log) → the line is
  removed.
- **Todo** (anything semantically loaded — naming, branches without
  tests, off-by-one bounds, redundant null checks) → a
  `// TODO(slop): …` comment is spliced above the line in the file's
  native comment syntax. Code is left intact. You decide.
- **Flag-only** (TODO/FIXME placeholders) → surfaced in the verdict
  but no patch change. Nothing to mechanically fix.

If apply's preflight refuses the patch, the working tree is byte-
identical to its pre-apply state. Re-running is safe.

## When the model disagrees with a finding

```bash
slop learn "false positive on 'process_data' — this is a deliberate
            verb-led getter, not naming slop. file: src/foo.rs:42"
```

That signal teaches the engine for your account and project
specifically. Over the next scans the same false positive should stop
firing. No raw text is retained beyond the learning step.

## Bootstrap

```bash
slop login
# Asks `ssh -G <server-host>` which key OpenSSH would pick for the
# server (same resolution `git` already observes). Honours per-host
# `IdentityFile` blocks in ~/.ssh/config. Caches the identity in
# ~/.config/slop/.
```

If `slop poke` returns 402, the response includes a Stripe Checkout
URL. Open it, subscribe, retry.

## Quick reference

| command | does |
|---|---|
| `slop poke` | scan working tree vs HEAD; cache plan to `.slop/last-poke.json` |
| `slop poke --staged` | scan the staged index |
| `slop poke --range X..Y` | scan an explicit git diff range |
| `slop poke --since REF` | scan everything since REF |
| `slop poke --patch FILE` | scan a unified-diff file directly |
| `slop poke --gh org/repo` | scan a public github repo (any range, persistent cache) |
| `slop poke --repo URL` | scan any git URL (gitlab, bitbucket, self-hosted) |
| `slop poke --patch-only` | emit ONLY the unified diff on stdout — pairs with `> slop.patch` |
| `slop apply` | preflight + `git apply --unidiff-zero` + `git commit --amend` |
| `slop apply --no-commit` | preflight + `git apply --index`; you commit |
| `slop apply --show` | print cached plan (patch + actions) |
| `slop apply --discard` | drop cached plan, no patch action |
| `slop learn "<text>"` | shape future scans |
| `slop billing tier` | quota + usage this cycle |
| `slop billing portal` | open Stripe portal |

Knobs: `SLOPPOKE_SERVER`, `SLOP_NO_VERSION_CHECK=1`,
`SLOP_REMOTE_CLONE_DEPTH=<N>`, `SLOP_CACHE_DIR=<path>`,
`SLOP_NO_COLOR=1` / `NO_COLOR=1`.

## Don't bypass it

If `slop poke` flags something the author thinks is fine, the right
move is `slop learn` (a one-line note explaining why) — NOT
silencing the finding or committing without a scan. The engine only
gets sharper if real signal flows back.
