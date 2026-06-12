---
description: Apply the last cached `slop poke` patch via `slop apply`
allowed-tools: Bash
---

Apply the cached slop-poke patch produced by the most recent `/slop:poke` invocation.

Default behaviour:
```
slop apply --no-commit
```

`--no-commit` is the safer default for plugin use: it applies + stages the patch and leaves committing to the user (and their normal workflow), so we never amend a HEAD that has already been pushed.

If the user explicitly asks to amend HEAD ("squash it in", "amend the last commit"), drop the flag:
```
slop apply
```

If apply fails its preflight (`git apply --check`), the working tree is byte-identical. Surface the reason verbatim. Don't loop, don't retry, don't mutate anything else.

When the user wants to drop the cached plan instead of applying it, run `slop apply --discard`.

## After apply: the TODO(slop) markers are an action queue

Once `slop apply` succeeds, **immediately** run:

```
git grep -n "TODO(slop)"
```

Each hit is a precise (file, line, category) flag the catalog matched but did not auto-fix. Triage every one:

- **In-scope + small** (off-by-one, redundant guard, missing test for a branch the user just added) → fix it in the same change. The marker disappears on the next `/slop:poke`. Don't ask permission for trivial in-scope fixes; just do them and report.
- **Out-of-scope or larger refactor** → file a followup. Use whatever issue convention this repo already has (`.issues/open/TXXX-…md` if the repo uses the convention, otherwise `gh issue create`). Leave the TODO(slop) line in the code. Mention the new ticket reference next to it.

**Never** delete a TODO(slop) marker as cleanup. The next poke will re-emit it. If the user thinks it's a false positive, run `slop learn "<why>"` instead.

Pattern: `apply → grep → triage → either fix or file → poke again`. Treat the markers as work, not noise.
