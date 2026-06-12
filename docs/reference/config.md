# Configuration reference

All sloppoke client state lives under `~/.config/slop/`.

## File paths

| Path                                       | Contents                                          |
|--------------------------------------------|---------------------------------------------------|
| `~/.config/slop/config.toml`               | Server URL + identity cached from `slop login`    |
| `~/.config/slop/news-cache.json`           | Last server news feed (24 h TTL)                  |
| `~/.config/slop/news-seen.json`            | Per-machine list of news entry IDs already shown  |
| `~/.config/slop/version-check.json`        | Cached GitHub Releases tag (24 h TTL)             |
| `<repo>/.slop/last-poke.json`              | Cached cleanup plan, consumed by `slop apply`     |

## Environment variables

### Identity + transport

| Variable                       | Default                  | Effect                                       |
|--------------------------------|--------------------------|----------------------------------------------|
| `SLOPPOKE_SERVER`              | `https://sloppoke.me`    | API base URL used by every subcommand        |
| `SLOP_SSH_KEY`                 | `~/.ssh/id_*`            | Override the SSH key used to sign requests   |
| `SLOP_CONFIG_DIR`              | `~/.config/slop`         | Override the config directory (CI / tests)   |

### Pre-commit hook

| Variable                       | Effect                                           |
|--------------------------------|--------------------------------------------------|
| `SLOP_SKIP_HOOK=1`             | Bypass the pre-commit hook for this `git commit` |

### News + version-check throttle

| Variable                       | Default | Effect                                                 |
|--------------------------------|---------|--------------------------------------------------------|
| `SLOP_VERSION_CHECK_TTL_HOURS` | `24`    | Hours between GitHub-Releases checks                   |
| `SLOP_NO_VERSION_CHECK=1`      | unset   | Suppress the "version X is out" stderr line entirely   |
| `SLOP_GITHUB_RELEASES_URL`     | upstream| Override the release-feed URL (used in tests / mirrors)|

### Server-side (only relevant if you self-host)

| Variable                       | Effect                                              |
|--------------------------------|-----------------------------------------------------|
| `SLOPPOKE_BIND`                | Listener bind address (default `0.0.0.0:8765`)      |
| `SLOPPOKE_CACHE_DIR`           | Disk cache for public-scorer scans                  |
| `SLOPPOKE_THROTTLE_PER_MINUTE` | Per-IP rate limit (default `30`)                    |
| `SLOPPOKE_ADMIN_TOKEN`         | Shared secret for `/api/v1/admin/*` endpoints       |
| `GITHUB_TOKEN`                 | Lift the public-scorer's GitHub API rate-limit to 5000/hr |
| `STRIPE_SECRET_KEY`            | Enables Stripe billing flow                         |
| `RUST_LOG`                     | Log level (`info,sloppoke_server=info`)             |
