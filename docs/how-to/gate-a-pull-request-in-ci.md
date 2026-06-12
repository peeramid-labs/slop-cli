# How to gate a pull request in CI

Run `slop poke` against the PR diff. Exit code is non-zero on SLOP →
the job fails → the PR is gated. No GitHub App, no install scope.

## GitHub Actions

```yaml
# .github/workflows/slop.yml
name: slop
on: pull_request

jobs:
  poke:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }   # need history to diff against base

      - name: Install slop
        run: curl -fsSL https://sloppoke.me/install.sh | sh

      - name: Log in with the bot key
        env:
          SLOP_PRIVATE_KEY: ${{ secrets.SLOP_PRIVATE_KEY }}
        run: |
          mkdir -p ~/.ssh
          echo "$SLOP_PRIVATE_KEY" > ~/.ssh/slop_bot && chmod 600 ~/.ssh/slop_bot
          SLOP_SSH_KEY=~/.ssh/slop_bot slop login

      - name: Scan PR diff
        run: slop poke --range "$GITHUB_BASE_REF..HEAD"
```

`SLOP_PRIVATE_KEY` is a service-account SSH key minted just for CI. The
fingerprint counts as one identity for billing.

## GitLab CI

```yaml
slop:
  image: rust:latest
  script:
    - curl -fsSL https://sloppoke.me/install.sh | sh
    - mkdir -p ~/.ssh && echo "$SLOP_PRIVATE_KEY" > ~/.ssh/slop_bot
    - chmod 600 ~/.ssh/slop_bot
    - SLOP_SSH_KEY=~/.ssh/slop_bot slop login
    - slop poke --range "$CI_MERGE_REQUEST_DIFF_BASE_SHA..HEAD"
  only: [merge_requests]
```

## Forgejo Actions

Same shape as GitHub Actions; the variable names differ:

```yaml
- run: slop poke --range "${{ gitea.base_ref }}..HEAD"
```

## Jenkins

```groovy
stage('slop') {
  steps {
    sh 'curl -fsSL https://sloppoke.me/install.sh | sh'
    withCredentials([sshUserPrivateKey(credentialsId: 'slop-bot',
                                       keyFileVariable: 'KEY')]) {
      sh 'SLOP_SSH_KEY=$KEY slop login'
      sh "slop poke --range origin/${env.CHANGE_TARGET}..HEAD"
    }
  }
}
```

## Soft-gate (warn, don't fail)

Drop `set -e` and ignore the exit code:

```sh
slop poke --range "$BASE..HEAD" || echo "::warning::slop found hits"
```

## Cap noise on big PRs

Restrict the range to the latest commit only:

```sh
slop poke --range "HEAD~1..HEAD"
```

This stops the gate from being a wall of historical TODO splices and
forces every new commit to clean itself up.
