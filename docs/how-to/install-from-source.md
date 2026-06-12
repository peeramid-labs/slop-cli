# How to install from source

Use this when the install script is blocked, when you need a specific
commit, or when packaging for an unsupported distro.

Prerequisites: Rust `1.86+`, git, `ssh-keygen`.

## Clone and build

```sh
git clone https://github.com/peeramid-labs/sloppoke.git
cd sloppoke
cargo install --path crates/sloppoke-cli
```

`cargo install` places the binary at `~/.cargo/bin/slop`. Add that
directory to your `$PATH` if it isn't already.

## Pin to a specific version

```sh
git clone https://github.com/peeramid-labs/sloppoke.git
cd sloppoke
git checkout v0.7.0
cargo install --path crates/sloppoke-cli
```

## Build a release binary directly

```sh
cargo build --release -p sloppoke-cli
./target/release/slop --version
```

The release binary has no runtime dependencies beyond `ssh-keygen` for
request signing.

## Cross-compile for another target

```sh
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu -p sloppoke-cli
```

## Build the homebrew formula locally

```sh
brew install --build-from-source peeramid-labs/tap/slop
```

## Run tests

```sh
cargo test -p sloppoke-cli
```

The test suite is hermetic — no network calls, no real SSH operations
(`SLOP_CONFIG_DIR` is redirected to a tempdir per test).
