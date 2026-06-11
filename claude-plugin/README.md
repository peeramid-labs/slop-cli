# sloppoke — Claude Code plugin

Adds three slash commands to Claude Code that wrap the `slop` CLI:

| command | does |
|---|---|
| `/slop:poke` | runs `slop poke` against the current diff, reports the verdict |
| `/slop:apply` | applies the cached cleanup patch (`--no-commit` by default to avoid amending pushed commits) |
| `/slop:learn` | sends a one-line feedback note to tune detection for your account |

Also bundles the canonical `slop.md` skill so the same "what does the verdict mean" context is available regardless of which command the user invokes.

## Install

```
/plugin install github.com/peeramid-labs/sloppoke
```

The plugin assumes the `slop` binary is already on your `$PATH`. Install it first:

```
curl -fsSL https://sloppoke.me/install.sh | sh
slop login
```

## Why a plugin and not just the skill

The skill (`skills/slop.md`) is portable across any agent that reads a skill directory — Cursor, Continue, Codex, plain Claude Code. The plugin adds:

- Slash-command shortcuts for the common verbs
- Plugin-manager update lifecycle (no `curl | sh` to refresh)
- Future hooks (pre-commit `slop poke` gate) and MCP exposure of the public-score endpoint

Use whichever fits your setup. The skill alone works fine — the plugin just adds shortcuts.

## License

MIT — same as the rest of the sloppoke project.
