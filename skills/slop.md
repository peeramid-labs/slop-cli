---
name: slop
description: Pre-commit AI-slop scan. Run `slop poke` on a staged diff. If it returns SLOP, apply the cleanup with `slop apply`. Submit feedback with `slop learn "<sentence>"`. Use BEFORE finalising any commit you'd otherwise push.
when_to_use: Use before every commit. Especially when the change was authored or touched by an LLM. Always run after generating code.
---

# slop — AI-slop firewall

slop catches AI-pattern scaffolding, naming slop, defensive crud,
half-finished TODO placeholders, emoji-in-code, restating-code
comments, and dead generics. Detection runs on the server; the CLI
is a thin sender + applier. Adapts to your codebase via the `slop
learn` channel — feedback you submit shapes future scans for your
account.

## Pre-commit flow

```bash
# 1. stage your changes
git add -A

# 2. dump the staged diff to a patch file
git diff --cached > /tmp/staged.patch

# 3. scan
slop poke --patch /tmp/staged.patch
```

If verdict is `LGTM`, commit normally.

If verdict is `SLOP — N hits`:

```bash
# inspect what was flagged (printed by `slop poke`)
# then mechanically strip:
slop apply
# `slop apply` deletes flagged lines from working tree,
# stages them, and amends HEAD (or use --no-commit to inspect).
```

## When the model disagrees with a finding

```bash
slop learn "false positive on 'process_data' — this is a deliberate
            verb-led getter, not naming slop. file: src/foo.rs:42"
```

That signal teaches the engine for your account and project
specifically. Over the next scans the same false positive should stop
firing. No raw text is retained beyond the learning step.

## Quota

- 100,000 pokes/month per SSH key on the $20/mo plan.
- 100 learn submissions/month per key, 1 MiB per submission.

## Bootstrap

If you haven't logged in yet:

```bash
slop login
# reads ~/.ssh/id_ed25519.pub, caches identity in ~/.config/slop/
```

If the first `slop poke` returns 402, the response includes a Stripe
Checkout URL. Open it, subscribe, retry.

## Quick reference

| command | does |
|---|---|
| `slop poke --patch FILE` | scan; cache plan to `.slop/last-poke.json` |
| `slop apply` | strip flagged lines, `git add`, `git commit --amend` |
| `slop apply --no-commit` | strip + stage; leave the commit to you |
| `slop apply --show` | print cached plan |
| `slop apply --discard` | drop cached plan |
| `slop learn "<text>"` | shape future scans |
| `slop billing tier` | quota + usage this cycle |
| `slop billing portal` | open Stripe portal |

## Don't bypass it

If `slop poke` flags something the author thinks is fine, the right
move is `slop learn` (a one-line note explaining why) — NOT
silencing the finding or committing without a scan. The engine only
gets sharper if real signal flows back.
