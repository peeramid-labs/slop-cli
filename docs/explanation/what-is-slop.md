# What "AI slop" actually means

"Slop" is the residue an LLM leaves around the working code it
produces. The function does the right thing. The 30 % around it does
not.

## Five buckets

1. **Editorial wrapping.** Self-congratulatory verbs ("comprehensive",
   "robust", "battle-tested"), narrative comments that restate the
   next line, polite hedges that soften nothing. Tokens, no information.

2. **Defensive crud.** `if x is None: x = None`. `try/except` around
   imports. Null-checks on values that are typed non-nullable. The
   model can't tell which guards are real and ships all of them.

3. **Unfinished stubs.** `def foo(): pass`, `def foo(): ...`,
   `throw new NotImplementedException()`, `[YOUR_NAME]`. Plausible
   shape, no body.

4. **Hallucinated references.** Imports of packages that don't exist.
   Variable names referring to things never defined. URLs to
   `example.com`. Identifiers that look like UUIDs but aren't.

5. **Untested branches.** New `if` / `else` / `match` arms that ship
   without a paired test. The control flow is real; the verification
   is not.

## Why the model produces it

LLMs predict the next plausible token. Plausible ≠ correct. A
defensive guard is plausible — it looks like what production code does.
A narrative comment is plausible — every blog post the model trained
on had one. An import of `numpyz` is plausible — `numpy` exists.

Slop isn't a hallucination flaw, it's the load-bearing characteristic
of next-token prediction at scale. The model is doing its job. The job
just doesn't end where committing code ends.

## Why a human can't catch all of it

Reviewers eyeball a diff for correctness. They don't have the budget
to read 30 % more lines per commit looking for narrative comments and
defensive guards. The slop slips past, accumulates, and shows up
months later as "the codebase feels bloated."

## What sloppoke counts as slop

Anything in the [catalog reference](../reference/catalog.md). Every
category there has been distilled from real diffs across real
LLM-assisted repos. The catalog is regex-driven + AST-aware where
syntax matters (Rust `#[cfg(test)]` boundaries, Python test-file
conventions, TypeScript branch counting).

The list grows. New entries land as we see them in the wild.

## What sloppoke does *not* count as slop

- Stylistic preferences (tab vs space, brace position) — a linter's job
- Correctness bugs (wrong algorithm, race condition) — a test's job
- Architectural drift — code review's job
- Type errors — `tsc` / `mypy` / `cargo check`'s job

Sloppoke sits between the linter and the human reviewer. It catches
the residue both miss.
