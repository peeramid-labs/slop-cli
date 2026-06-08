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
        assert!(
            stdout.contains(sub),
            "missing subcommand {sub} in help: {stdout}"
        );
    }
}

#[test]
fn slop_version_renders() {
    let mut cmd = Command::cargo_bin("slop").unwrap();
    let out = cmd.arg("--version").assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("slop"),
        "expected slop in version: {stdout}"
    );
}

#[test]
fn poke_without_config_fails_with_login_hint() {
    let dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.env("SLOP_CONFIG_DIR", dir.path())
        .args(["poke"])
        .assert()
        .failure()
        .stderr(contains("slop login"));
}

#[test]
fn poke_rejects_mutually_exclusive_sources() {
    let dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.env("SLOP_CONFIG_DIR", dir.path())
        .args(["poke", "--staged", "--range", "main..HEAD"])
        .assert()
        .failure();
}

#[test]
fn poke_dry_run_without_login_fails_with_config_message() {
    let dir = tempfile::tempdir().unwrap();
    let patch = dir.path().join("p.patch");
    let mut f = std::fs::File::create(&patch).unwrap();
    f.write_all(b"diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -0,0 +1 @@\n+hi\n")
        .unwrap();
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

/// Initialise a throwaway repo, write `src/foo.ts` + a cached plan
/// asking for an insert_above on line 3, then run `slop apply
/// --no-commit`. Asserts the comment was spliced and staged.
#[test]
fn apply_insert_above_splices_todo_comment() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // Minimal git repo so `slop apply` can `git add` the result.
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "t@example.com"])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(root)
        .status()
        .unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    let src = "function compute() {\n    return 1;\n    if (always) return 2;\n}\n";
    std::fs::write(root.join("src/foo.ts"), src).unwrap();
    std::process::Command::new("git")
        .args(["add", "src/foo.ts"])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-q", "-m", "init"])
        .current_dir(root)
        .status()
        .unwrap();

    std::fs::create_dir_all(root.join(".slop")).unwrap();
    let plan = r#"{
        "poke_id":"x",
        "verdict":"SLOP",
        "cleanup_actions":[
            {"file":"src/foo.ts","line":3,"action":"insert_above","content":"    // TODO(slop): add test for new `if` branch"}
        ]
    }"#;
    std::fs::write(root.join(".slop/last-poke.json"), plan).unwrap();

    let mut cmd = Command::cargo_bin("slop").unwrap();
    cmd.current_dir(root)
        .args(["apply", "--no-commit"])
        .assert()
        .success()
        .stderr(contains("spliced"));
    let after = std::fs::read_to_string(root.join("src/foo.ts")).unwrap();
    assert!(
        after.contains("// TODO(slop)"),
        "TODO not present in file:\n{after}"
    );
    let line3 = after.lines().nth(2).unwrap();
    assert!(
        line3.contains("TODO(slop)"),
        "TODO should sit on the line above the original line 3 (now row 3): got {line3:?}"
    );
}

/// Running `slop apply` twice with the same insert_above plan must
/// not duplicate the TODO comment — second run is a no-op on that
/// file.
#[test]
fn apply_insert_above_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "t@example.com"])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(root)
        .status()
        .unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/foo.ts"), "let x = 1;\nlet y = 2;\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "src/foo.ts"])
        .current_dir(root)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-q", "-m", "init"])
        .current_dir(root)
        .status()
        .unwrap();

    std::fs::create_dir_all(root.join(".slop")).unwrap();
    let plan = r#"{
        "poke_id":"x","verdict":"SLOP",
        "cleanup_actions":[
            {"file":"src/foo.ts","line":2,"action":"insert_above","content":"// TODO(slop): review"}
        ]
    }"#;
    std::fs::write(root.join(".slop/last-poke.json"), plan).unwrap();

    Command::cargo_bin("slop")
        .unwrap()
        .current_dir(root)
        .args(["apply", "--no-commit"])
        .assert()
        .success();
    let after_first = std::fs::read_to_string(root.join("src/foo.ts")).unwrap();
    let todo_count = after_first.matches("TODO(slop)").count();
    assert_eq!(todo_count, 1, "first apply should add exactly one TODO");

    Command::cargo_bin("slop")
        .unwrap()
        .current_dir(root)
        .args(["apply", "--no-commit"])
        .assert()
        .success();
    let after_second = std::fs::read_to_string(root.join("src/foo.ts")).unwrap();
    assert_eq!(
        after_second.matches("TODO(slop)").count(),
        1,
        "second apply must not duplicate the TODO"
    );
}
