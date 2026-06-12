# Exit codes

Stable across versions. CI scripts can rely on these.

| Code | Meaning                                                     |
|------|-------------------------------------------------------------|
| `0`  | Success — `LGTM` verdict on `slop poke`, no work to do      |
| `1`  | `SLOP` verdict — the scan surfaced at least one hit         |
| `2`  | Usage error — bad flag, missing argument, malformed input   |
| `3`  | Authentication failure — `slop login` not run, key mismatch |
| `4`  | Server error — 5xx response, network timeout                |
| `5`  | Quota exceeded — monthly poke-call cap hit                  |
| `6`  | Patch apply failed — `git apply --check` reported drift     |

Hook helpers (the bundled pre-commit hook) translate `1` to a blocking
exit `2` so git rejects the commit.
