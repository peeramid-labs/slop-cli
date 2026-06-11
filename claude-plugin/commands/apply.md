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
