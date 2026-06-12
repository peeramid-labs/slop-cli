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

You should see something like `slop 0.7.0`.

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

## What now

You now have the basics. Pick a how-to next:

- [Install the pre-commit hook](../how-to/install-pre-commit-hook.md)
  so every commit runs `slop poke` automatically.
- [Gate a pull request in CI](../how-to/gate-a-pull-request-in-ci.md)
  to catch slop your team didn't.
- [Use the Claude Code plugin](../how-to/use-claude-code-plugin.md) to
  gate every `git commit` your agent attempts.
