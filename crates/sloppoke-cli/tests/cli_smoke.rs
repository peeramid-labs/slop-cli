//! End-to-end CLI smoke tests. No network — we drive the binary
//! through clap and check its surface (help text, dry-run shape,
//! exit codes on missing input).

use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;

#[test]
fn slop_no_args_prints_usage() {
    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.assert().failure().stderr(contains("Usage: slop"));
}

#[test]
fn slop_help_lists_every_subcommand() {
    let mut cmd = Command::cargo_bin("slop").unwrap();
    let out = cmd.arg("--help").assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    for sub in ["login", "poke", "apply", "learn", "billing"] {
        assert!(stdout.contains(sub), "missing subcommand {sub} in help: {stdout}");
    }
}

#[test]
fn slop_version_renders() {
    let mut cmd = Command::cargo_bin("slop").unwrap();
    let out = cmd.arg("--version").assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("slop"), "expected slop in version: {stdout}");
}

#[test]
fn poke_requires_patch_file() {
    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.args(["poke"]).assert().failure();
}

#[test]
fn poke_dry_run_without_login_fails_with_config_message() {
    let dir = tempfile::tempdir().unwrap();
    let patch = dir.path().join("p.patch");
    let mut f = std::fs::File::create(&patch).unwrap();
    f.write_all(b"diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -0,0 +1 @@\n+hi\n").unwrap();
    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.env("SLOP_CONFIG_DIR", dir.path())
        .args(["poke", "--patch"])
        .arg(&patch)
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(contains("slop login"));
}

#[test]
fn apply_without_cached_plan_fails() {
    let dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.current_dir(dir.path())
        .args(["apply"])
        .assert()
        .failure()
        .stderr(contains("slop poke"));
}

#[test]
fn apply_show_with_empty_plan_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".slop")).unwrap();
    let plan = r#"{"poke_id":"x","verdict":"LGTM","cleanup_actions":[]}"#;
    std::fs::write(dir.path().join(".slop/last-poke.json"), plan).unwrap();
    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.current_dir(dir.path())
        .args(["apply", "--show"])
        .assert()
        .success()
        .stdout(contains("LGTM"));
}

#[test]
fn apply_discard_removes_plan() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".slop")).unwrap();
    let plan_path = dir.path().join(".slop/last-poke.json");
    std::fs::write(&plan_path, "{}").unwrap();
    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.current_dir(dir.path())
        .args(["apply", "--discard"])
        .assert()
        .success();
    assert!(!plan_path.exists(), "plan should have been deleted");
}
