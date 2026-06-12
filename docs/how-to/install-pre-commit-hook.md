# How to install the pre-commit hook

The hook runs `slop poke --staged` before every commit and blocks the
commit on a SLOP verdict. Two scopes: per-repo or global.

## Per-repo (default)

From inside the repo:

```sh
slop install-hook
```

This writes `.git/hooks/pre-commit`. Already-configured hooks are
preserved — `slop` only appends its block, marked by comments.

## Globally (every repo)

```sh
slop install-hook --global
```

This writes to `~/.config/slop/git-hooks/` and points
`core.hooksPath` at it. Existing per-repo hooks still run first.

## Bypass for one commit

```sh
git commit --no-verify -m "..."
```

Use this when you intentionally want the slop in the commit (e.g.
recording a placeholder you'll remove in the next commit).

## Force-overwrite an existing hook

```sh
slop install-hook --force
```

Drops any existing hook content. Lose-history risk — only use when you
have nothing to preserve.

## Uninstall

```sh
rm .git/hooks/pre-commit
# Or globally:
git config --global --unset core.hooksPath
```
