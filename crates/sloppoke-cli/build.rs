//! Build-time provenance: bakes the git commit + build timestamp into
//! the binary so `slop --version` surfaces what code was actually
//! shipped. Lets users tie a downloaded binary to a specific commit
//! when verifying supply-chain provenance.

use std::process::Command;

fn git_output(args: &[&str]) -> Option<Vec<u8>> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| o.stdout)
}

fn main() {
    let commit = git_output(&["rev-parse", "--short=12", "HEAD"])
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let dirty = git_output(&["status", "--porcelain"])
        .map(|b| !b.is_empty())
        .unwrap_or(false);

    let commit_label = format!("{commit}{}", if dirty { "-dirty" } else { "" });

    let build_epoch = std::env::var("SOURCE_DATE_EPOCH").unwrap_or_else(|_| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string())
    });

    println!("cargo:rustc-env=SLOP_BUILD_COMMIT={commit_label}");
    println!("cargo:rustc-env=SLOP_BUILD_EPOCH={build_epoch}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
}
