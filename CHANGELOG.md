## [unreleased]

### 🚀 Features

- *(cli)* Poke --repo / --gh — scan any git URL without manual clone
- *(cli)* Friendly version notice — checks GitHub Releases with 24h cache
- *(cli)* Poke prints the proposed patch on stdout alongside the line summary
- *(cli)* Infer clone depth from --range, cache repos, colorize patch, add --patch-only

### 🐛 Bug Fixes

- *(cli)* Clippy — char-array split sugar in version parser
- *(cli)* --patch-only routes through TTY-aware emitter so terminal output stays colored
- *(cli)* Drop per-line findings dump — it leaked detector keywords
- *(cli)* Mark PokeResponse.findings as #[serde(default)]
- *(cli)* Drop --patch-only flag — redundant with stdout/stderr split

### 📚 Documentation

- *(magic)* Replace detector enumeration with high-level capability copy
- *(rename)* README title + repository URL → peeramid-labs/sloppoke
- *(free-tier)* Surface 50 pokes/month free in README + skill
- *(pricing)* Drop monthly quota + dollar references from README + skill
- *(cli)* Surface git apply --unidiff-zero in the manual-apply hint
- *(skill)* Cover --gh/--repo, --patch-only, persistent cache, color, knobs

### 🎨 Styling

- Cargo fmt — collapse parse_version chain

### ⚙️ Miscellaneous Tasks

- *(release)* V0.4.1
- *(domain)* Switch DEFAULT_SERVER + README badge + ssh_resolve fixtures to sloppoke.me
- *(homebrew)* Rename homepage URL slop-cli → sloppoke
- *(release)* V0.5.0
## [0.4.0] - 2026-06-08

### 🚀 Features

- *(cli)* Apply now handles insert_above CleanupAction
- *(cli)* Prefer server-rendered patch — git apply replaces in-house line editor

### 📚 Documentation

- *(readme)* List supported languages + add branch-without-test row
- *(skill)* Refresh detector list, document tier-aware apply + ssh-G login

### ⚙️ Miscellaneous Tasks

- *(release)* V0.4.0
## [0.3.0] - 2026-06-08

### 🚀 Features

- *(cli)* Ssh-config-aware key resolution via ssh -G

### 📚 Documentation

- *(readme)* Drop brew/apt install paths until release pipeline stabilises
- *(readme)* Add Homebrew install for slop v0.2.3

### ⚙️ Miscellaneous Tasks

- *(release)* V0.3.0
## [0.2.3] - 2026-06-07

### 🐛 Bug Fixes

- *(release-workflows)* Pin artifact actions to v3 for forgejo GHES compat
- *(cli)* Derive pubkey from --key, recompute fingerprint per request

### 🎨 Styling

- Cargo fmt

### ⚙️ Miscellaneous Tasks

- *(release)* V0.2.3
## [0.2.2] - 2026-06-07

### 🚜 Refactor

- *(cli)* Rename forgejo_api → api, drop dead forgejo_user + orgs fields

### ⚙️ Miscellaneous Tasks

- *(release)* V0.2.2
## [0.2.1] - 2026-06-07

### 🐛 Bug Fixes

- *(release-workflows)* Replace cross with cargo-zigbuild for linux targets

### ⚙️ Miscellaneous Tasks

- *(release)* V0.2.1
## [0.2.0] - 2026-06-07

### 🚀 Features

- *(cli)* Default poke to git diff, add --staged/--range/--since/--patch

### 🐛 Bug Fixes

- *(release-workflows)* Read version from [workspace.package], not crate Cargo.toml
- *(release-workflows)* Use forgejo REST API instead of gh CLI
- *(release-workflows)* Recover from half-failed runs by reusing branch
- *(release-workflows)* Fail loud when cargo set-version no-ops
- *(api)* Base64-encode armored SSH signature for Authorization header
- *(release-workflows)* Stage explicit paths + emit pre/post status
- *(release-workflows)* Drop Cargo.lock from staging — repo gitignores it

### 📚 Documentation

- Restore centered mascot block (HTML <p align=center> with width=500)
- ML-driven learning copy + drop implementation details
- Drop CI badge — actions disabled on github mirror, forgejo runs CI

### ⚙️ Miscellaneous Tasks

- Copy nsed release workflows 1:1 (verbatim)
- Adapt nsed workflows for slop-cli single-binary layout
- Run integration tests too — drop --bins flag, slop-cli has tests/
- *(release)* V0.2.0
