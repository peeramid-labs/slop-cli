# How detection works under the hood

People assume an LLM-based code analyzer must call an LLM. Sloppoke
deliberately doesn't, on the hot path. The fast scanner is a
deterministic regex + AST engine. The "learning" lives in a slow async
loop behind it.

## The fast path

```
your patch ──► slop CLI ──► HTTPS POST /api/v1/poke
                            │
                            ▼
                   LocalEngine in the server
                            │
                ┌───────────┴───────────────┐
                ▼                           ▼
         compiled regex                AST visitors
         categories                    (per-language)
                │                           │
                └─────────┬─────────────────┘
                          ▼
              hit list with category +
              suggested cleanup
```

Steps:

1. The CLI reads your patch (`git diff --staged`, a range, a file).
2. It signs the request with your SSH key and POSTs it to
   `/api/v1/poke`.
3. The server loads a pre-compiled corpus of ~50 categories. Each
   category is a regex + a fixability declaration.
4. For each added line of the diff, every regex runs. AST visitors
   walk per-language modules for the structural categories that
   pure-text patterns can't reach (branch counting, doc-comment
   detection, `#[cfg(test)]` boundaries).
5. Hits are deduplicated, scored, and returned with a `cleanup_actions`
   array.

Sub-10 ms verdict per typical patch. No LLM. No model load. No GPU.

## What's in the catalog

See the [catalog reference](../reference/catalog.md) for the full list.
Five buckets cover most of the surface:

- Language-agnostic text patterns (self-congratulatory verbs,
  defensive crud, narrative comments)
- Language-specific structural rules (Python `bare_except`, Rust
  `unwrap` outside tests, TS `as unknown as`)
- SQL anti-patterns
- AST-driven cross-file checks (untested branches)
- Comment-marker hunters (FIXME / HACK / XXX / TODO)

Every rule has been distilled from real LLM-assisted diffs, not from
academic taxonomies. The list grows as new patterns surface in the
wild.

## The slow path — where ML lives

Detection accuracy improves over time via NSED Orchestrator, an
asynchronous reinforcement-learning backend.

```
your `slop learn` feedback ──► LearnLog (sled tree on the server)
                                       │
                                       ▼
                          ┌────────────────────────────┐
                          │ NSED async RL pipeline      │
                          │ ──────────────────────────  │
                          │ • multi-model deliberation │
                          │ • category weight tuning   │
                          │ • per-account corpus diffs │
                          └──────────────┬──────────────┘
                                         │
                                         ▼
                          per-account catalog updates
                          land on the fast path next deploy
```

What this means for you:

- Every false positive you report (`slop learn "the X warning is wrong
  because…"`) feeds the loop.
- The loop processes feedback out-of-band — no synchronous latency.
- The model fleet that runs deliberation is what powers the rest of
  Peeramid Labs' multi-agent products. You get the benefit of the
  full orchestration backend, but you only ever hit the fast
  regex/AST scanner from the CLI.

## Why this split is the right one

Detection benefits from determinism: same patch → same verdict, every
time, no temperature. Tuning benefits from ML: surfacing patterns no
human wrote a regex for yet.

Mixing them on the hot path would mean an LLM call per commit, which
would:

- Add seconds of latency to every `git commit`
- Cost real money per commit
- Make every verdict non-reproducible

Splitting them keeps `slop poke` boring, fast, and free of mystery.
The intelligence lives in the catalog, not in the request.
