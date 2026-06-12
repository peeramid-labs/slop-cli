# How to score a public GitHub repo

The public scorer scans the last 100 commits of any public GitHub repo
and returns a 0–100 score with a trend sparkline. No install required.

## From the browser

Visit [sloppoke.me](https://sloppoke.me), paste the repo URL or
`org/repo` into the scorer, hit **Score it →**.

You get back:

- A score 0–100 (higher is cleaner)
- The number of commits scanned and changed lines hit
- Top categories and worst files
- A trend sparkline + verdict tier (PRISTINE / CLEAN / DRIFTING /
  SLOPPY / ROTTING)
- A shareable `/s/<id>` permalink

## From the CLI

```sh
slop poke --gh paradigmxyz/reth --range HEAD~100..HEAD
```

The flag scopes the scan to a remote repo via the GitHub API (no clone).

## Share a result

Every score has a permalink, e.g. `https://sloppoke.me/s/abc123`. The
**Poke maintainer on X** button on the result card composes a tweet
that @-mentions the GitHub owner with the verdict and the link.

## Cap repo size

Repos larger than 2 GB return a polite refusal. Anything inside that
ceiling scans in under 30 seconds.

## Rate limit

Anonymous: 30 scans per IP per minute. Set the `GITHUB_TOKEN` env on
your own server-side scorer deploy to lift GitHub's own 60/hr cap to
5000/hr.
