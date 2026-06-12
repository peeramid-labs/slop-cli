# CLI reference

Authoritative list of every command, subcommand, and flag.

## Global

```
slop [-v | -V | --version] [--help] <COMMAND> [ARGS...]
```

| Flag                   | Effect                                |
|------------------------|---------------------------------------|
| `-v`, `-V`, `--version`| Print version and exit                |
| `--help`               | Print help for any command            |

## `slop login`

Resolve the SSH key into a server-side identity. Required once per
machine.

```
slop login [--server URL] [--key PATH]
```

| Flag         | Default                  | Effect                                  |
|--------------|--------------------------|-----------------------------------------|
| `--server`   | `https://sloppoke.me`    | API base URL                            |
| `--key`      | `$SLOP_SSH_KEY` or `~/.ssh/id_*` | SSH private key used to sign requests |

## `slop poke`

Scan a patch for slop. Saves a cleanup plan to `.slop/last-poke.json`
for `slop apply`.

```
slop poke [--staged | --range BASE..HEAD | --patch FILE | --gh ORG/REPO]
          [--project NAME] [--no-color]
```

| Flag        | Effect                                                |
|-------------|-------------------------------------------------------|
| `--staged`  | Scan `git diff --staged` (default if no flag is given when staged changes exist) |
| `--range`   | Scan a git range (`HEAD~3..HEAD`, `main..HEAD`)       |
| `--patch`   | Scan an arbitrary unified-diff file                   |
| `--gh`      | Scan a remote GitHub repo via the public API          |
| `--project` | Override the project label written to the server log  |
| `--no-color`| Disable ANSI colour even on a TTY                     |

Exit code: `0` on LGTM, `1` on SLOP, `2` on usage error, `>2` on
network / quota failures (see [exit-codes](exit-codes.md)).

## `slop apply`

Apply the cached cleanup plan from the most recent poke.

```
slop apply [--no-commit | --discard | --force]
```

| Flag         | Effect                                               |
|--------------|------------------------------------------------------|
| `--no-commit`| Stage the cleanup but don't amend HEAD               |
| `--discard`  | Drop the cached plan without applying                |
| `--force`    | Apply even when `git apply --check` reports drift    |

## `slop learn`

Send one-line feedback to the learning loop.

```
slop learn [--no-attach] [--project NAME] "<text>"
```

| Flag          | Effect                                                |
|---------------|-------------------------------------------------------|
| `--no-attach` | Skip auto-attaching the last cached poke as context   |
| `--project`   | Tag the feedback with a project label                 |

## `slop billing`

```
slop billing tier         # show current tier + monthly cap usage
slop billing portal       # open the Stripe customer portal URL
```

## `slop install-hook`

Drop a git pre-commit hook that runs `slop poke --staged` and blocks
SLOP commits.

```
slop install-hook [--global] [--force]
```

| Flag       | Effect                                                |
|------------|-------------------------------------------------------|
| `--global` | Write to `~/.config/slop/git-hooks/` and set `core.hooksPath` |
| `--force`  | Overwrite an existing hook (preserves nothing)        |

## `slop news`

Show announcement entries served by `/api/v1/news`.

```
slop news            # show unseen entries, mark them seen
slop news --all      # show the full back-catalog
slop news --ack      # mark every cached entry as seen
```

## See also

- [Configuration](config.md) — env vars and config file paths
- [Exit codes](exit-codes.md) — what every numeric exit means
- [Detection categories](catalog.md) — every category the engine knows
