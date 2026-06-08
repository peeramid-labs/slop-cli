//! Background freshness check against GitHub Releases.
//!
//! Every CLI invocation may emit a single stderr line if a newer
//! release is available. The check itself is best-effort: we cache
//! the answer for 24h, time HTTP out at 2s, and swallow every error
//! so a slow or absent GitHub never blocks a poke. Stderr-only so
//! machine-readable stdout is untouched.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 24 hours by default — overridable via `SLOP_VERSION_CHECK_TTL_HOURS`
/// so CI runs can pin a shorter window, or paranoid users a longer one.
fn ttl_seconds() -> u64 {
    let hours: u64 = std::env::var("SLOP_VERSION_CHECK_TTL_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24);
    hours.saturating_mul(3600)
}

#[derive(Debug, Serialize, Deserialize)]
struct Cache {
    checked_at_unix: u64,
    latest_version: String,
}

fn cache_path() -> Option<PathBuf> {
    let dir = if let Ok(custom) = std::env::var("SLOP_CONFIG_DIR") {
        PathBuf::from(custom)
    } else {
        let home = std::env::var("HOME").ok()?;
        PathBuf::from(home).join(".config").join("slop")
    };
    Some(dir.join("version-check.json"))
}

fn read_cache() -> Option<Cache> {
    let path = cache_path()?;
    let text = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_cache(latest: &str) {
    let Some(path) = cache_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let entry = Cache {
        checked_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        latest_version: latest.to_string(),
    };
    let _ = serde_json::to_string(&entry).map(|s| fs::write(&path, s));
}

fn cache_is_fresh(c: &Cache) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now.saturating_sub(c.checked_at_unix) < ttl_seconds()
}

/// Fetch the latest release tag from GitHub Releases. 2s timeout —
/// the user is waiting on the actual command; we never block them.
fn fetch_latest_tag() -> Option<String> {
    let url = std::env::var("SLOP_GITHUB_RELEASES_URL").unwrap_or_else(|_| {
        "https://api.github.com/repos/peeramid-labs/sloppoke/releases/latest".to_string()
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .user_agent(concat!("slop-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;
    let resp = client.get(&url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().ok()?;
    let tag = v.get("tag_name")?.as_str()?.to_string();
    Some(tag)
}

/// Normalise a version string for compare. Strips a leading `v`, then
/// splits on `.` and parses each segment as u32. Pre-release suffixes
/// (`-rc1`, `+build`) are ignored — we just stop parsing at the first
/// non-numeric segment. Returns None on garbage rather than panicking.
fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let trimmed = s.trim().trim_start_matches('v').trim_start_matches('V');
    let head = trimmed.split(['-', '+', '_']).next()?;
    let mut parts = head.split('.').filter_map(|p| p.parse::<u32>().ok());
    let major = parts.next()?;
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    Some((major, minor, patch))
}

/// True when `latest` is strictly newer than `local`. Garbage on
/// either side resolves to false so we never nag on parse failure.
fn is_newer(local: &str, latest: &str) -> bool {
    match (parse_version(local), parse_version(latest)) {
        (Some(a), Some(b)) => b > a,
        _ => false,
    }
}

/// Resolve the version to advertise as "available". Returns the
/// cached tag if fresh; otherwise hits GitHub, caches the result,
/// and returns it. None on any failure so callers stay silent.
fn latest_known_version() -> Option<String> {
    if let Some(c) = read_cache() {
        if cache_is_fresh(&c) {
            return Some(c.latest_version);
        }
    }
    let tag = fetch_latest_tag()?;
    write_cache(&tag);
    Some(tag)
}

/// Emit a friendly heads-up to stderr if a newer release exists.
/// Suppressed by `SLOP_NO_VERSION_CHECK=1` so CI and scripted users
/// stay quiet.
pub fn notify_if_outdated() {
    if std::env::var("SLOP_NO_VERSION_CHECK")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        return;
    }
    let local = env!("CARGO_PKG_VERSION");
    let Some(latest) = latest_known_version() else {
        return;
    };
    if !is_newer(local, &latest) {
        return;
    }
    let normalised = latest.trim_start_matches('v').trim_start_matches('V');
    eprintln!(
        "sloppoke v{normalised} is out — you're on {local}. \
         https://github.com/peeramid-labs/sloppoke/releases/tag/v{normalised}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_handles_leading_v() {
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("V0.4.1"), Some((0, 4, 1)));
        assert_eq!(parse_version("0.4.1"), Some((0, 4, 1)));
    }

    #[test]
    fn parse_version_handles_two_segment() {
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
        assert_eq!(parse_version("7"), Some((7, 0, 0)));
    }

    #[test]
    fn parse_version_strips_pre_release_suffix() {
        assert_eq!(parse_version("0.4.1-rc1"), Some((0, 4, 1)));
        assert_eq!(parse_version("0.4.1+build"), Some((0, 4, 1)));
        assert_eq!(parse_version("0.4.1_alpha"), Some((0, 4, 1)));
    }

    #[test]
    fn parse_version_rejects_garbage() {
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("hello"), None);
        assert_eq!(parse_version("v"), None);
    }

    #[test]
    fn is_newer_compares_semver_numerically() {
        assert!(is_newer("0.4.0", "0.4.1"));
        assert!(is_newer("0.4.0", "0.5.0"));
        assert!(is_newer("0.4.0", "1.0.0"));
        assert!(is_newer("0.4.9", "0.4.10"));
        assert!(!is_newer("0.4.1", "0.4.0"));
        assert!(!is_newer("0.4.0", "0.4.0"));
        // Pre-release tag suffix is ignored — same numeric triple wins.
        assert!(!is_newer("0.4.0", "0.4.0-rc1"));
    }

    #[test]
    fn is_newer_returns_false_on_garbage() {
        // Never nag on parse failure.
        assert!(!is_newer("0.4.0", ""));
        assert!(!is_newer("", "0.4.0"));
        assert!(!is_newer("hello", "0.4.0"));
        assert!(!is_newer("0.4.0", "world"));
    }

    #[test]
    fn cache_is_fresh_respects_ttl() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let fresh = Cache {
            checked_at_unix: now,
            latest_version: "0.4.0".into(),
        };
        let stale = Cache {
            checked_at_unix: 0,
            latest_version: "0.4.0".into(),
        };
        assert!(cache_is_fresh(&fresh));
        assert!(!cache_is_fresh(&stale));
    }
}
