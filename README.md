# slop

![mascot](assets/mascot.png)

[![CI](https://github.com/peeramid-labs/slop-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/peeramid-labs/slop-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/peeramid-labs/slop-cli?label=release)](https://github.com/peeramid-labs/slop-cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Homebrew](https://img.shields.io/badge/brew-peeramid--labs%2Ftap%2Fslop-orange.svg)](https://github.com/peeramid-labs/homebrew-tap)
[![APT](https://img.shields.io/badge/apt-peeramid--labs%2Fapt--repo-blue.svg)](https://github.com/peeramid-labs/apt-repo)
[![Server: slop.peeramid.xyz](https://img.shields.io/badge/server-slop.peeramid.xyz-green.svg)](https://slop.peeramid.xyz/)

**Blazing-fast AI-slop firewall for your git workflow.** One command,
~10ms per patch, learns from every false-positive you flag.

```
slop poke --patch <file>   # scan: regex + AST, server-side
slop apply                 # strip flagged lines, amend HEAD
slop learn "this was a false positive on src/foo.rs"
```

## What it catches

| category | what it kills |
|---|---|
| `ai_scaffolding`  | `// Step 1:` / `// Initialize the X` / `// Helper function for …` |
| `naming_slop`     | `process_data`, `class XManager`, `let data1 = …`, `getStuff()` |
| `defensive_crud`  | `except: pass`, `} catch (Exception) {}`, redundant null guards |
| `todo_implement`  | `TODO: implement`, `unimplemented!()`, `raise NotImplementedError` |
| `emoji_in_code`   | any emoji in source — almost always LLM signature |
| `what_filler`     | comments that just restate the next line of code |
| `unused_generic`  | `fn foo<T>(x: i32)` — generic declared but never used (tree-sitter) |

Detector lives on the server. The CLI is a thin sender + applier.
Algorithms ship server-side so the catalog can evolve without
re-publishing the binary — every customer gets the latest detectors
on their next poke.

## Install

### Homebrew (macOS / Linux)
```
brew install peeramid-labs/tap/slop
```

### APT (Debian / Ubuntu)
```
curl -fsSL https://raw.githubusercontent.com/peeramid-labs/apt-repo/main/KEY.gpg \
    | sudo gpg --dearmor -o /usr/share/keyrings/peeramid.gpg
echo "deb [signed-by=/usr/share/keyrings/peeramid.gpg] https://raw.githubusercontent.com/peeramid-labs/apt-repo/main stable main" \
    | sudo tee /etc/apt/sources.list.d/peeramid.list
sudo apt update && sudo apt install slop
```

### From source
```
git clone https://github.com/peeramid-labs/slop-cli.git
cd slop-cli
cargo install --path crates/sloppoke-cli
```
Needs rust `1.86+`. Build is one binary, no LLM runtime, no native
deps beyond `ssh-keygen` for request signing.

### Pre-built tarballs
Grab the matching archive for your platform from [Releases](https://github.com/peeramid-labs/slop-cli/releases),
unpack, drop `slop` into your `$PATH`.

## Get started

```
slop login                     # SSH-key handshake; cache identity
slop poke --patch <file>       # first call: 402 + Stripe Checkout URL
                               # pay → next call lands findings + plan
slop apply                     # auto-strip flagged lines, amend HEAD
slop learn "false positive"    # ship feedback → RL loop learns
```

`slop apply` runs `git apply` + `git commit --amend` locally. Use
`--no-commit` if you want to inspect the staged diff first.

## Pricing

| plan | price | quota |
|---|---|---|
| Slop Poke | $20 / month | 100,000 pokes |

Quota resets on the first of each month. Per-IP throttle 30
requests/min by default — bump on the server if your IDE legitimately
bursts harder.

## How it learns

Every poke gets persisted server-side (the patch + verdict + findings).
Every `slop learn "…"` lands in the same store. The operator drains
the queue periodically and feeds it through an offline LLM workflow
that proposes catalog tweaks; new regex/AST detectors flow back into
the next server release. You feel the catalog get smarter over time
without re-installing anything.

Per-fingerprint learn cap: **100 submissions/month, 1 MiB per submit.**

## Auth model

- SSH key fingerprint = identity. No accounts, no signups beyond the
  Stripe checkout.
- `slop login` reads `~/.ssh/id_ed25519.pub`, sends it to the server,
  caches the canonical `SHA256:…` fingerprint locally.
- Each request signs `{method}\n{path}\n{timestamp}\n{sha256(body)}`
  via `ssh-keygen -Y sign`. The server verifies the signature with
  the in-band pubkey.

## Claude skill

`/slop` is bundled as a Claude skill in `skills/slop.md` — wires the
CLI into your agent so it pokes before every commit.

## License

MIT. See `LICENSE`.

---

Built by [Peeramid Labs](https://peeramid.xyz/).
