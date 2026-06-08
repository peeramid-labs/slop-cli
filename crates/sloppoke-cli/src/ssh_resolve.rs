//! SSH-config-aware public-key resolution.
//!
//! Defers to OpenSSH's own config parser — we shell out to
//! `ssh -G <host>`, which dumps the fully resolved per-host config
//! the ssh binary would use (Host * + per-host + Include + wildcards,
//! same precedence git itself observes since git just shells out to
//! ssh too).
//!
//! Returns the first `identityfile` whose matching `.pub` exists on
//! disk. The caller derives the private key from the pubkey path by
//! stripping the `.pub` suffix, so the pair is always consistent.

use std::path::PathBuf;
use std::process::Command;

/// Extract the hostname from a server URL. Strips scheme, port, and
/// path. `https://sloppoke.me:443/api` → `sloppoke.me`.
pub fn host_from_server_url(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let host_port = after_scheme.split('/').next()?;
    let host = host_port.split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

/// Parse `ssh -G` output into the ordered list of `identityfile` paths
/// it announced. Lines look like `identityfile ~/.ssh/id_rsa`; we
/// expand a leading `~/` against `$HOME` because the dump uses the
/// tilde verbatim.
pub fn parse_ssh_g_output(text: &str, home: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else {
            continue;
        };
        // ssh -G lower-cases all option names.
        if key != "identityfile" {
            continue;
        }
        let Some(raw) = parts.next() else {
            continue;
        };
        let expanded = if let Some(rest) = raw.strip_prefix("~/") {
            format!("{home}/{rest}")
        } else if raw == "~" {
            home.to_string()
        } else {
            raw.to_string()
        };
        out.push(PathBuf::from(expanded));
    }
    out
}

/// Ask OpenSSH to resolve which identity it would use for `host`,
/// then return the first `<identity>.pub` that exists on disk.
/// Returns None if `ssh` is not installed, the host has no
/// configured identity, or none of the announced identities have a
/// matching public key on disk.
pub fn resolve_for_host(host: &str) -> Option<PathBuf> {
    let out = Command::new("ssh").args(["-G", host]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let home = std::env::var("HOME").ok()?;
    for ident in parse_ssh_g_output(&text, &home) {
        // ssh -G dumps the private-key path (id_rsa, id_ed25519, …).
        // Our auth uses ssh-keygen -Y sign which needs the private key
        // + the matching pubkey. Require the .pub to actually be on
        // disk before claiming this identity.
        let pub_path = PathBuf::from(format!("{}.pub", ident.display()));
        if pub_path.exists() {
            return Some(pub_path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_from_server_url_strips_scheme_and_port() {
        assert_eq!(
            host_from_server_url("https://sloppoke.me").as_deref(),
            Some("sloppoke.me")
        );
        assert_eq!(
            host_from_server_url("https://sloppoke.me:443/api/v1").as_deref(),
            Some("sloppoke.me")
        );
        assert_eq!(
            host_from_server_url("http://192.168.1.135:3000").as_deref(),
            Some("192.168.1.135")
        );
        // No scheme — treat the input as a bare host.
        assert_eq!(
            host_from_server_url("forge.local:8080").as_deref(),
            Some("forge.local")
        );
    }

    #[test]
    fn host_from_server_url_rejects_empty() {
        assert!(host_from_server_url("").is_none());
        assert!(host_from_server_url("https://").is_none());
    }

    #[test]
    fn parse_ssh_g_extracts_identityfile_lines_in_order() {
        let dump = "\
host sloppoke.me
user tim
identityfile ~/.ssh/id_rsa
identityfile ~/.ssh/id_ed25519
port 22
loglevel info
";
        let paths = parse_ssh_g_output(dump, "/Users/tim");
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/Users/tim/.ssh/id_rsa"),
                PathBuf::from("/Users/tim/.ssh/id_ed25519"),
            ]
        );
    }

    #[test]
    fn parse_ssh_g_passes_through_absolute_paths() {
        let dump = "identityfile /opt/keys/deploy_key\n";
        let paths = parse_ssh_g_output(dump, "/Users/tim");
        assert_eq!(paths, vec![PathBuf::from("/opt/keys/deploy_key")]);
    }

    #[test]
    fn parse_ssh_g_ignores_other_options() {
        let dump = "\
hostname forge.local
controlmaster auto
identityfile ~/.ssh/forgejo_key
";
        let paths = parse_ssh_g_output(dump, "/h");
        assert_eq!(paths, vec![PathBuf::from("/h/.ssh/forgejo_key")]);
    }
}
