# sloppoke — Claude Code plugin

Adds three slash commands to Claude Code that wrap the `slop` CLI:

| command | does |
|---|---|
| `/slop:poke` | runs `slop poke` against the current diff, reports the verdict |
| `/slop:apply` | applies the cached cleanup patch (`--no-commit` by default to avoid amending pushed commits) |
| `/slop:learn` | sends a one-line feedback note to tune detection for your account |

Also bundles the canonical `slop.md` skill so the same "what does the verdict mean" context is available regardless of which command the user invokes.

Plus a **PreToolUse hook** (`hooks/pre-commit-poke.sh`) that intercepts every `Bash` tool call Claude makes, lets non-commit commands through, and runs `slop poke --staged` before any `git commit*`. Non-LGTM verdicts surface to Claude with the suggested patch — the model can call `/slop:apply` and retry the commit, or you can bypass once with `SLOP_SKIP_HOOK=1 git commit ...`.

## Install

```
/plugin marketplace add peeramid-labs/sloppoke
/plugin install sloppoke@peeramid-labs
```

The first command registers this repo as a marketplace (it ships a
`.claude-plugin/marketplace.json` at root). The second installs the
`sloppoke` plugin from it.

The plugin assumes the `slop` binary is already on your `$PATH`. Install it first:

```
curl -fsSL https://sloppoke.me/install.sh | sh
slop login
```

## Why a plugin and not just the skill

The skill (`skills/slop.md`) is portable across any agent that reads a skill directory — Cursor, Continue, Codex, plain Claude Code. The plugin adds:

- Slash-command shortcuts for the common verbs
- Plugin-manager update lifecycle (no `curl | sh` to refresh)
- PreToolUse hook auto-gating every `git commit` Claude attempts
- Future: MCP exposure of the public-score endpoint, `slop doctor`

To pull a new plugin version after an update lands:

```
/plugin marketplace update peeramid-labs
/plugin install sloppoke@peeramid-labs
```

Use whichever fits your setup. The skill alone works fine — the plugin just adds shortcuts.

## License

MIT — same as the rest of the sloppoke project.
