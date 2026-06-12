# Why pre-commit is the right boundary

You could catch slop in a lot of places: in the editor as you type,
inside an LLM's review step, on a pull request, in CI, on the
production branch. Sloppoke runs against the staged diff right
before `git commit`. This is deliberate.

## The cost graph

The cost of removing slop scales with how far down the pipeline you
remove it:

```
edit (free)           ──────────────────  $1
staged diff           ──────────────────  $1
pull request          ───────────────────  $10
merged commit         ──────────────────   $50
production            ──────────────────   $500
new-hire reading it   ──────────────────   "why is this here?"
```

Numbers are illustrative. Direction is not. Each step right adds:

- A reviewer's attention
- A CI cycle
- A force-push to "clean up"
- Time between you and the original intent

Pre-commit is the last cheap step. Anything earlier requires either an
editor plugin (per editor, per language) or hooking the LLM itself
(works only when the LLM is one specific tool). Pre-commit works
regardless of which assistant produced the code, which editor you
edited it in, or which CI you run.

## Why not at the LLM layer

LLM-side guards are the obvious move and they're fine, but:

- They only cover *that* LLM
- They run *during* generation, which means the slop you do ship is
  by definition slop the guard missed
- They run on every token, not every commit — orders of magnitude more
  expensive

Pre-commit is one verdict per intent-to-ship. Cheap, decisive, vendor-
neutral.

## Why not in CI

CI is a fine *second* gate. The
[CI how-to](../how-to/gate-a-pull-request-in-ci.md) walks through it.
But by the time CI fires:

- The slop is already in the git history
- A reviewer has to read it
- A force-push is needed to clean it up
- Or the slop ships, because force-pushing got contentious

CI catches what slipped past the local gate. It doesn't replace it.

## What gets sacrificed

Pre-commit gating means a milliseconds-fast check on every commit. We
can't run a 10-second LLM call there. The detector is a regex + AST
engine that returns under 10 ms on a typical patch. That constraint
shapes the catalog: every entry is mechanically detectable from the
diff alone, no model in the loop.

The downside: there are slop classes that need actual reasoning to
catch (e.g. "this entire function duplicates one ten files away"). For
those, the [NSED Orchestrator](https://sloppoke.me/) async backend
sharpens the same engine over time from your `slop learn` feedback.
You stay on the fast path.
