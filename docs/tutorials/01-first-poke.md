# Your first slop poke

By the end of this tutorial, you will have installed `slop`, run a
scan on a real staged diff, and applied the suggested cleanup. This
takes about five minutes.

You will need:

- macOS or Linux (x86_64 or arm64)
- A git repository with at least one uncommitted change
- An SSH key already configured for git (the same key you push commits
  with)

## 1. Install the CLI

Run:

```sh
curl -fsSL https://sloppoke.me/install.sh | sh
```

You should see:

```
sloppoke installed at /usr/local/bin/slop
```

Verify:

```sh
slop --version
```

You should see something like `slop 0.8.1 (commit abc1234, built 2026-06-11)`.

## 2. Log in

The first run resolves your SSH key into a server-side identity. No
email, no password.

```sh
slop login
```

You should see:

```
slop login: identity cached as slop-fp-<short-fingerprint>
```

## 3. Stage a change

Move into any git repo. Make a small edit — add a function with a
narrative comment, leave a `TODO`, write an `unwrap()` outside a
test, anything an LLM would emit.

Stage it:

```sh
git add path/to/your-file
```

## 4. Scan the staged diff

```sh
slop poke --staged
```

If the diff is clean you will see:

```
slop poke: LGTM (3 ms, 5/100000 this cycle)
```

If slop found something you will see a verdict like:

```
slop poke: SLOP — 2 hits (11 ms, 6/100000 this cycle)
─── proposed patch (1 hunk) ───
diff --git a/path/to/your-file b/path/to/your-file
+// TODO(slop): comment narrates what the code does — let the code speak
```

The patch above the prompt is the cleanup `slop` recommends. The plan
is cached at `.slop/last-poke.json` for the next step.

## 5. Apply the cleanup

```sh
slop apply --no-commit
```

You should see:

```
slop apply: 2 actions applied, staged for commit
```

Inspect the staged changes:

```sh
git diff --staged
```

You should see the `TODO(slop): …` comments spliced above the
flagged lines.

## 6. Commit

Drop `--no-commit` to let `slop apply` amend HEAD for you, or commit
manually:

```sh
git commit -m "your message"
```

## 7. Install the git pre-commit hook

Manual scans work, but they only fire when *you* remember. The
pre-commit hook runs `slop poke --staged` on every `git commit` —
whether you typed it, your IDE typed it, or a coding agent shelled
out and typed it. This is the gate.

For the current repo:

```sh
slop install-hook
```

You should see:

```
slop install-hook: installed at .git/hooks/pre-commit
defense-in-depth status:
  [ACTIVE] git pre-commit hook (.git/hooks/pre-commit)
  [MISSING] Claude Code PreToolUse hook
```

For every repo on the machine (recommended once you trust it):

```sh
slop install-hook --global
```

Try it — make another small edit, stage it, and commit. The hook
will print the verdict. A clean diff commits silently; slop blocks
the commit until you fix it or pass `--no-verify` (don't).

## 8. Add the Claude Code layer

If you let Claude Code drive `git commit` through the Bash tool, the
git hook still catches it — but you want the verdict *before* Claude
spends tokens drafting the commit message. Install the plugin:

```
/plugin install sloppoke@peeramid-labs
```

Then verify both gates are live:

```sh
slop status
```

You should see:

```
defense-in-depth status:
  [ACTIVE] git pre-commit hook
  [ACTIVE] Claude Code PreToolUse hook
```

Two ACTIVE lines is the target. Coding agents have to defeat **both**
gates (`--no-verify` for git AND `SLOP_SKIP_HOOK=1` for Claude) to
ship slop past you.

## What now

You now have the basics. Pick a how-to next:

- [Gate a pull request in CI](../how-to/gate-a-pull-request-in-ci.md)
  to catch slop your team didn't.
- [Score a public GitHub repo](../how-to/score-a-public-repo.md) to
  see how `slop` reads code at scale.
- [Submit feedback to the learning loop](../how-to/submit-feedback.md)
  when slop is wrong — the next catalog release will be sharper.
