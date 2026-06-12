# sloppoke

[![slop](https://sloppoke.me/badge/peeramid-labs/sloppoke.svg)](https://sloppoke.me/?repo=peeramid-labs/sloppoke)
[![Release](https://img.shields.io/github/v/release/peeramid-labs/sloppoke?label=release)](https://github.com/peeramid-labs/sloppoke/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Server: sloppoke.me](https://img.shields.io/badge/server-sloppoke.me-green.svg)](https://sloppoke.me/)

<p align="center">
  <img src="./assets/mascot.png" alt="slop mascot" width="500"/>
</p>

**Blazing-fast AI-slop firewall for your git workflow.** Sub-10ms
verdict per patch. Learns from every correction you ship.

```
slop poke                                # working tree vs HEAD (default)
slop poke --staged                       # the staged index
slop poke --range main..HEAD             # everything diverged from main
slop poke --since main                   # shorthand for the same
slop poke --patch foo.patch              # raw unified-diff file
slop poke --gh org/repo --range X..Y     # any public github repo, no clone needed
slop poke --repo URL  --range X..Y       # any git URL (gitlab, bitbucket, self-hosted)
slop apply                               # apply the cached patch, amend HEAD
slop learn "false positive on src/foo.rs"
```

Output goes to **stdout** as a unified-diff patch — apply hint and
verdict line on stderr — so piping is trivial:

```
slop poke > slop.patch                   # save the patch
slop poke | git apply --unidiff-zero     # apply directly
slop poke | delta                        # view in a pager
```

## How do we characterize slop?

Slop is what writing-by-suggestion leaves in source. Three flavours,
easier to feel than to define:

- **Wordy nothing** — comments that restate the next line, names
  that say less than nothing, prose that drifts.
- **Defensive theatre** — guards for impossible cases, empty
  catches, retries with no upstream, asserts that can't fire.
- **Unfinished work shipped** — placeholder brackets, untested
  branches, stub markers, AI-authorship trailers committed instead
  of staying in metadata.

We don't publish the catalog, and it isn't static. Every `slop
learn "…"` you submit calibrates the engine for your account; the
set firing on your repo on day 30 isn't the set firing today.

## What it does

slop sweeps your diff through a blazing-fast machine-learning engine
that knows what AI-generated code looks like — and pulls it out before
you ship.

- **Catches the artefacts an LLM leaves behind.** Scaffolding
  comments, placeholder identifiers, defensive crud, half-finished
  markers, untested branches — the residue that survives "looks fine"
  review but rots six months later.
- **Adapts to your codebase.** Every `slop learn` you submit tunes the
  engine for your account and your project. The catalogue you scan
  against on day 30 isn't the catalogue you started with — it's the
  one calibrated to your team's idioms.
- **Fixes what's safe, flags what isn't.** Mechanical noise is
  stripped automatically; anything that could change behaviour gets a
  TODO comment spliced into your diff for you to decide on. The CLI
  refuses to mutate your tree unless `git apply` agrees the change is
  clean.

Detection runs server-side and updates continuously — no re-install
needed. Works on every language; the deepest analysis lights up first
for Rust, TypeScript/JavaScript, Python, and Go.

## Install

### Homebrew (macOS + Linux)

```
brew install peeramid-labs/tap/slop
```

Prebuilt binaries for macOS (arm64 + x86_64) and Linux (arm64 + x86_64).
APT repo still pending.

### Build from source

```
git clone https://github.com/peeramid-labs/sloppoke.git
cd sloppoke
cargo install --path crates/sloppoke-cli
```

Needs rust `1.86+`. One binary, no runtime daemons, no native deps
beyond `ssh-keygen` for request signing. `cargo install` places `slop`
in `~/.cargo/bin/` — make sure that's on your `$PATH`.

### Claude Code plugin

Two-step install in any Claude Code session:

```text
/plugin marketplace add peeramid-labs/sloppoke
/plugin install sloppoke@peeramid-labs
```

Ships `/slop:poke`, `/slop:apply`, `/slop:learn` slash commands plus
matching skills, so Claude Code calls the CLI on its own before
committing. Requires `slop` on PATH (Homebrew or source install above)
and `slop login` once.

## Get started

```
slop login                     # SSH-key handshake; cache identity
slop poke                      # scan; 402 returns a Stripe Checkout URL
slop apply                     # auto-strip flagged lines, amend HEAD
slop learn "false positive"    # shapes future scans
```

`slop apply` runs `git apply` + `git commit --amend` locally. Use
`--no-commit` if you want to inspect the staged diff first.

## Learns as you go

slop adapts to **your** code and **your** preferences through machine
learning. Every `slop learn "…"` you submit teaches the engine — for
your account and your project specifically. False positives get
quieter. Real misses get caught next time. The catalog you experience
on day 30 is calibrated to your team's idioms in ways the day-1
catalog could not be.

> slop does not retain user information. Learning signals are folded
> into the model and the raw inputs do not persist.

Need more headroom or have an unusual workload? Ping
[engineering@peeramid.xyz](mailto:engineering@peeramid.xyz).

## Auth

SSH key = identity. `slop login` picks the key OpenSSH would use for
the server host (same resolution git observes), caches the fingerprint
locally, and signs each request with `ssh-keygen -Y sign`. No accounts,
no signups beyond Stripe checkout.

## Use in CI

Drop into any GitHub Actions job to gate merges on a clean scan:

```yaml
jobs:
  slop:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0                  # need history for range diff
      - name: install slop
        run: |
          curl -fsSL https://github.com/peeramid-labs/sloppoke/releases/latest/download/sloppoke-cli-x86_64-unknown-linux-gnu.tar.gz \
            | tar -xz -C /usr/local/bin
      - name: configure identity
        env:
          SLOP_SSH_KEY: ${{ secrets.SLOP_SSH_KEY }}
        run: |
          mkdir -p ~/.ssh
          echo "$SLOP_SSH_KEY" > ~/.ssh/id_ed25519
          chmod 600 ~/.ssh/id_ed25519
          slop login
      - name: scan PR
        run: |
          slop poke --range origin/${{ github.base_ref }}..HEAD > slop.patch
          if [ -s slop.patch ]; then
            echo "::error::sloppoke flagged AI-slop in this PR"
            cat slop.patch
            exit 1
          fi
```

Mechanics:

- **SSH key as secret.** `SLOP_SSH_KEY` holds the private key whose
  fingerprint owns your subscription. Generate a dedicated CI key, run
  `slop login` once locally to register the fingerprint, then store
  the private half in GitHub Secrets. Same auth model as your laptop —
  no separate token to manage.
- **Scope to the PR diff.** `--range origin/${base_ref}..HEAD` keeps
  the scan to changed lines only, so the run burns one poke per PR
  instead of one per file.
- **Exit code is the gate.** Empty stdout = clean = pass. Patch on
  stdout = slop = fail the job, optionally print the patch in logs so
  reviewers see the suggested fix.

For monorepos or trunk-based flows where `origin/main..HEAD` is too
broad, swap in `HEAD~1..HEAD` (last commit only) or a specific tag.

## Claude skill

`/slop` is bundled as a Claude skill in `skills/slop.md` — wires the
CLI into your agent so it pokes before every commit.

## License

MIT. See `LICENSE`.

---

Built by [Peeramid Labs](https://peeramid.xyz/).
