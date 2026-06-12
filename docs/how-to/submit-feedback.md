# How to submit feedback to the learning loop

Every `slop learn` submission feeds the NSED reinforcement loop on the
server. The next scan from your account weights findings differently
based on what you reported.

## Report a false positive

```sh
slop learn "the rust_pub_no_doc warning on src/main.rs:42 is wrong — the public re-export already has docs on the source item"
```

The last poke is auto-attached as context (id + verdict + patch, up to
8 KB) so you don't have to paste the diff.

## Skip auto-attach

```sh
slop learn --no-attach "the heuristic for X is too broad"
```

## Report a missed slop

```sh
slop learn "missed: defensive try/except around imported function call in app.py — no chance it raises"
```

## Tag a feedback batch

```sh
slop learn --project my-org/my-repo "the python_print_debug rule fires on legitimate CLI tools — gate on entrypoint detection"
```

The `project` field partitions your feedback so the learning loop can
weight by project shape later.

## Per-period cap

Each account is capped at 100 submissions per calendar month and
8 KB per submission. The CLI prints `queued <uuid> (N/100)` so you
know where you stand.

## What happens next

The submitted text + attached context lands in the server's `LearnLog`
sled tree. The offline RL workflow consumes it out-of-band and tunes
the per-account weighting. Expect 24–48 h before changes show up.
