# slop

[![Release](https://img.shields.io/github/v/release/peeramid-labs/slop-cli?label=release)](https://github.com/peeramid-labs/slop-cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Homebrew](https://img.shields.io/badge/brew-peeramid--labs%2Ftap%2Fslop-orange.svg)](https://github.com/peeramid-labs/homebrew-tap)
[![APT](https://img.shields.io/badge/apt-peeramid--labs%2Fapt--repo-blue.svg)](https://github.com/peeramid-labs/apt-repo)
[![Server: slop.peeramid.xyz](https://img.shields.io/badge/server-slop.peeramid.xyz-green.svg)](https://slop.peeramid.xyz/)

<p align="center">
  <img src="./assets/mascot.png" alt="slop mascot" width="500"/>
</p>

**Blazing-fast AI-slop firewall for your git workflow.** Sub-10ms
verdict per patch. Learns from every correction you ship.

```
slop poke                  # scan working tree vs HEAD (default)
slop poke --staged         # scan the staged index
slop poke --range main..HEAD
slop poke --since main     # everything that diverged from main
slop poke --patch foo.patch
slop apply                 # auto-clean flagged lines, amend HEAD
slop learn "this was a false positive on src/foo.rs"
```

## What it catches

| kind | what it is |
|---|---|
| AI scaffolding   | Numbered "Step N" / "First, …" / "Now we will …" comments that document the act of writing the code instead of explaining the code. |
| Naming slop      | Vague verb-led functions and Manager/Helper/Util class names — placeholder identifiers an LLM reaches for when the real name would require thought. |
| Defensive crud   | Empty exception swallows and redundant null guards added "so it doesn't crash" instead of fixing the underlying assumption. |
| Half-finished    | Unfinished-business markers: TODO/FIXME asking the next reader to implement the actual logic. |
| Emoji-in-code    | Emoji embedded in source. Almost never deliberate in a real codebase; nearly always an LLM autograph. |
| Restating code   | Comments that paraphrase the line below them instead of explaining WHY. |
| Dead generics    | Type parameters declared but never referenced — speculative abstraction. |

The detection engine improves continuously, server-side, so the
catalog you scan against today is always the latest one — no
re-install needed.

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
Needs rust `1.86+`. One binary, no runtime daemons, no native deps
beyond `ssh-keygen` for request signing.

### Pre-built tarballs
Grab the matching archive for your platform from [Releases](https://github.com/peeramid-labs/slop-cli/releases),
unpack, drop `slop` into your `$PATH`.

## Get started

```
slop login                     # SSH-key handshake; cache identity
slop poke                      # first call: 402 + Stripe Checkout URL
                               # pay → next call lands findings + plan
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

Per-account learn intake: **100 submissions/month, 1 MiB per
submission.** Plenty for a small team; if you need more, ping
[engineering@peeramid.xyz](mailto:engineering@peeramid.xyz).

## Auth model

- SSH key = identity. No accounts, no signups beyond Stripe checkout.
- `slop login` reads `~/.ssh/id_ed25519.pub`, derives a canonical
  fingerprint, caches it locally.
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
