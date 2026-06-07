---
name: slop
description: Pre-commit AI-slop scan. Run `slop poke` on a staged diff. If it returns SLOP, apply the cleanup with `slop apply`. Submit feedback with `slop learn "<sentence>"`. Use BEFORE finalising any commit you'd otherwise push.
when_to_use: Use before every commit. Especially when the change was authored or touched by an LLM. Always run after generating code.
---

# slop — AI-slop firewall

`slop` is a fast remote scanner that flags AI-pattern scaffolding,
naming slop, defensive crud, TODO placeholders, emoji-in-code,
restating-code comments, and unused generics. The detector runs on
the server; the CLI is a thin sender + applier.

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

The server commits the feedback to the org's RL feed. Over the next
release cycle the regex/AST catalog gets tuned.

## Quota

- 100,000 pokes/month per SSH key on the $20/mo plan.
- 30 requests/minute per-IP throttle.
- 100 learn submissions per month per key, 1 MiB per submission.

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
| `slop learn "<text>"` | send feedback to RL loop |
| `slop billing tier` | quota + usage this cycle |
| `slop billing portal` | open Stripe portal |

## Don't bypass it

If `slop poke` flags something the author thinks is fine, the right
move is `slop learn` (a one-line note explaining why) — NOT
silencing the finding or committing without a scan. The RL loop only
works if real customer signal flows back.
