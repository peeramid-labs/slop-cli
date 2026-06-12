# How to verify the binary

Every Sloppoke release ships three independent verification handles:

1. **Per-archive SHA-256 sidecar** — `<archive>.sha256` next to each
   tarball on the GitHub release page
2. **Consolidated `SHA256SUMS`** — one file listing every release
   artifact with its hash, signed by the same git tag
3. **Build provenance attestation** — GitHub-signed Sigstore bundle
   proving the binary came from the release workflow on a specific
   commit

Use whichever fits your threat model.

## Per-archive checksum

Quickest path. Download the binary + its sidecar, run `sha256sum -c`:

```sh
TAG=v0.7.0
TARGET=x86_64-unknown-linux-gnu
URL=https://github.com/peeramid-labs/sloppoke/releases/download/$TAG

curl -fsSLO "$URL/slop-$TAG-$TARGET.tar.gz"
curl -fsSLO "$URL/slop-$TAG-$TARGET.tar.gz.sha256"
sha256sum -c "slop-$TAG-$TARGET.tar.gz.sha256"
```

Expected output: `slop-v0.7.0-x86_64-unknown-linux-gnu.tar.gz: OK`

## Consolidated SHA256SUMS

Verify multiple downloads in one shot:

```sh
TAG=v0.7.0
URL=https://github.com/peeramid-labs/sloppoke/releases/download/$TAG

curl -fsSLO "$URL/SHA256SUMS"
sha256sum -c --ignore-missing SHA256SUMS
```

`--ignore-missing` skips entries you didn't download.

## Build provenance via GitHub Sigstore attestation

Strongest — proves the binary came from a specific commit, built in
the official GitHub Actions workflow, with no replay or in-flight
tamper.

```sh
gh attestation verify slop-v0.7.0-x86_64-unknown-linux-gnu.tar.gz \
  --owner peeramid-labs
```

Expected output: `Loaded digest sha256:… ✓ Verification succeeded`

`gh attestation verify` checks the Sigstore bundle attached to the
release, matches the artifact's SHA-256 to the bundle's subject, and
confirms the bundle was signed by a workflow inside the
`peeramid-labs/sloppoke` repo. No public key to rotate.

## What the binary reports about itself

`slop --version` embeds the git commit hash + build epoch the binary
was compiled from:

```
slop 0.7.0 (commit 14980b9a5807, built 1781261029)
```

Cross-reference the commit against the [release page on
GitHub](https://github.com/peeramid-labs/sloppoke/releases) — the tag
should point at the same commit.

A `-dirty` suffix on the commit means the working tree had
uncommitted changes when the binary was built. Official release
binaries never carry `-dirty`.

## Install script verification

The `curl | sh` installer at <https://sloppoke.me/install.sh>
fetches the matching sidecar and runs `sha256sum -c` before
installing. If the hash doesn't match, the installer aborts before
writing anything to `$PATH`.

You can read the installer before running it:

```sh
curl -fsSL https://sloppoke.me/install.sh | less
```

## Why three handles instead of one

- **Sidecar** — convenient for one-off downloads, but only as strong
  as the integrity of the release page
- **SHA256SUMS** — useful for verifying batches, same trust root as
  the sidecar
- **Sigstore attestation** — independent trust root (Sigstore +
  GitHub workflow identity), survives even a hypothetical compromise
  of the release page

Reproducible binary builds are not yet shipped. The
`SOURCE_DATE_EPOCH` environment variable is honoured during
compilation, so determinism is wired — what's missing is the
verifier-rebuilds-from-source script. Planned for a future release.
