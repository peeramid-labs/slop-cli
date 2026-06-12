# Privacy + identity model

Sloppoke processes your patch text to score it and stores parts of it
to improve the engine. This page explains what flows where, what we
retain, and what your identity is.

## Identity = SSH key fingerprint

The `slop login` flow asks `ssh-agent` (or reads `~/.ssh/id_*.pub`)
for your public key, computes its SHA-256 fingerprint, and registers
it server-side as `slop-fp-<short>`. Every subsequent API call signs
its body with the matching private key. The server verifies the
signature against the registered fingerprint.

Consequences:

- **No email, no password, no OAuth scope.** Your existing git
  identity is enough.
- **One key = one billable identity.** A team that shares one bot key
  shares one quota.
- **Rotating keys = rotating identity.** A new fingerprint is a new
  account; the old learning history doesn't carry over until you `slop
  learn` it back.

## What flows over the wire

Every `slop poke` POSTs:

- The full patch text
- The optional `project` label
- A signature header proving the request came from your fingerprint

That's it. No file paths outside the patch, no commit messages, no
working-tree state, no environment variables.

Every `slop learn` POSTs:

- Your one-line feedback text
- An optional 8 KB attachment from the last cached poke (id + verdict
  + patch slice)
- The fingerprint signature

## What the server retains

- **PokeLog** — the patch + findings JSON, used by the offline RL
  loop. Pruned by the learning loop once consumed.
- **LearnLog** — your feedback rows, keyed by fingerprint + period.
  Drained on the same cadence as PokeLog.
- **MonthlyCounters** — `(slop_org, period) → poke_calls, slop_hits,
  review_tokens` for billing + the public counter. Never deleted.
- **PublicScoreLog** — anonymous public-scorer scans, retained 30 d.

Nothing else. No telemetry pings, no analytics SDK, no error tracker
that captures your patch text.

## What never leaves the server

- The mapping from `slop-fp-<short>` back to your raw SSH fingerprint
  is server-only.
- Patch text never appears in metrics, log lines, or Stripe payloads.
- The learn loop's intermediate state lives entirely inside the
  server's sled tree.

## Where the server lives

Production hosts in Germany under GDPR. EU data residency, per-
account purge on request (email
[engineering@peeramid.xyz](mailto:engineering@peeramid.xyz)).

## Optional confidential compute

Enterprise tier offers an AMD SEV-SNP TEE deployment. Patch text is
decrypted only inside the guest's encrypted memory; the operator (us)
cannot read your diffs even with host root. Remote attestation runs
before any data ships. Intel TDX / AWS Nitro on request.

## What `slop` runs locally

The CLI itself:

- Reads your patch from `git diff`
- Reads your config from `~/.config/slop/`
- Reads the cached plan from `<repo>/.slop/last-poke.json`
- Writes a TODO splice when you `slop apply`
- Talks to the server over HTTPS

It does not:

- Run a model locally
- Scan files outside the patch
- Phone home outside the explicit calls listed above
- Modify any file other than `<repo>/.slop/last-poke.json` and (on
  `slop apply`) the files the patch touches
