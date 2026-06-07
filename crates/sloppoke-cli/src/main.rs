//! `slop` — the AI-slop firewall CLI.
//!
//!   slop login                  resolve SSH key, cache identity
//!   slop poke   --patch FILE    scan a patch, save cleanup plan
//!   slop apply                  strip flagged lines, amend HEAD
//!   slop learn  "<feedback>"    ship feedback to the RL loop
//!   slop billing tier|portal    subscription + usage / Stripe portal

mod api;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use api::CleanupAction;
use serde::{Deserialize, Serialize};

const DEFAULT_SERVER: &str = "https://slop.peeramid.xyz";
const CACHED_PLAN: &str = ".slop/last-poke.json";

#[derive(Parser, Debug)]
#[command(version, about = "Blazing-fast AI-slop firewall.")]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Resolve your SSH key's server identity and cache it locally.
    Login(LoginArgs),
    /// Scan a patch for slop. Saves a cleanup plan to
    /// `.slop/last-poke.json` for `slop apply`.
    Poke(PokeArgs),
    /// Apply the cleanup plan from the most recent poke: strip
    /// flagged lines, stage, and amend HEAD.
    Apply(ApplyArgs),
    /// Submit free-text feedback. Server commits it to your slop-org
    /// learn feed; the offline RL loop turns it into catalog updates.
    Learn(LearnArgs),
    /// Subscription + usage commands.
    #[command(subcommand)]
    Billing(BillingCmd),
}

#[derive(Parser, Debug, Clone)]
struct LoginArgs {
    #[arg(long, env = "SLOPPOKE_SERVER", default_value = DEFAULT_SERVER)]
    server: String,
    /// Path to SSH public key (default `~/.ssh/id_ed25519.pub`).
    #[arg(long)]
    pubkey: Option<PathBuf>,
    /// Path to matching private key (default: strip `.pub` suffix).
    #[arg(long)]
    key: Option<PathBuf>,
}

#[derive(Parser, Debug, Clone)]
struct PokeArgs {
    /// Unified-diff file. Overrides the default `git diff` capture.
    #[arg(long, conflicts_with_all = ["staged", "range", "since"])]
    patch: Option<PathBuf>,
    /// Scan the staged index instead of the working tree
    /// (equivalent to `git diff --cached`).
    #[arg(long, conflicts_with_all = ["patch", "range", "since"])]
    staged: bool,
    /// Custom git diff range, passed verbatim to `git diff`.
    /// Examples: `main..HEAD`, `HEAD~3..HEAD`, `origin/main...`.
    #[arg(long, conflicts_with_all = ["patch", "staged", "since"])]
    range: Option<String>,
    /// Scan only changes since a given ref (equivalent to
    /// `git diff <ref>`). Handy for "what's new since main".
    #[arg(long, conflicts_with_all = ["patch", "staged", "range"])]
    since: Option<String>,
    /// Optional `<source-org>/<project>` tag the server uses to
    /// bucket the row in the learn store. Not billing-relevant.
    #[arg(long)]
    project: Option<String>,
    /// Print the request JSON and exit without contacting the server.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Parser, Debug, Clone)]
struct ApplyArgs {
    /// Print the cached plan instead of applying.
    #[arg(long)]
    show: bool,
    /// Delete the cached plan without applying.
    #[arg(long)]
    discard: bool,
    /// Skip the `git commit --amend` step — leave changes staged.
    #[arg(long)]
    no_commit: bool,
}

#[derive(Parser, Debug, Clone)]
struct LearnArgs {
    /// One sentence (or paragraph) of feedback.
    feedback: String,
    /// Optional `<source-org>/<project>` scope.
    #[arg(long)]
    project: Option<String>,
    /// Optional anchoring context (file path, code excerpt, error).
    #[arg(long)]
    context: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum BillingCmd {
    /// Print plan + entitlements + this-cycle usage.
    Tier,
    /// Open the Stripe-hosted billing portal in $BROWSER.
    Portal {
        /// Print the URL instead of opening a browser.
        #[arg(long)]
        print: bool,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let rc = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("slop: {e:#}");
            1
        }
    };
    std::process::exit(rc);
}

fn run(cli: Cli) -> Result<()> {
    match cli.mode {
        Mode::Login(a) => run_login(a),
        Mode::Poke(a) => run_poke(a),
        Mode::Apply(a) => run_apply(a),
        Mode::Learn(a) => run_learn(a),
        Mode::Billing(c) => run_billing(c),
    }
}

// ── login ────────────────────────────────────────────────────────

fn run_login(args: LoginArgs) -> Result<()> {
    let pubkey_path = args.pubkey.unwrap_or_else(default_pubkey_path);
    let pubkey_line = fs::read_to_string(&pubkey_path)
        .with_context(|| format!("read {}", pubkey_path.display()))?
        .trim()
        .to_string();
    let key_path = args
        .key
        .unwrap_or_else(|| derive_key_from_pubkey(&pubkey_path));
    let resp = api::discover(&args.server, &pubkey_line)?;
    let cfg = api::SavedConfig {
        server_url: args.server,
        fingerprint: resp.fingerprint.clone(),
        ssh_key_path: key_path,
        slop_org: resp.slop_org.clone(),
    };
    api::save_config(&cfg)?;
    println!(
        "slop: logged in as {} ({})",
        if resp.slop_org.is_empty() {
            "(anonymous)".to_string()
        } else {
            resp.slop_org.clone()
        },
        resp.fingerprint
    );
    Ok(())
}

fn default_pubkey_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".ssh")
        .join("id_ed25519.pub")
}

fn derive_key_from_pubkey(p: &Path) -> PathBuf {
    let s = p.display().to_string();
    if let Some(rest) = s.strip_suffix(".pub") {
        PathBuf::from(rest)
    } else {
        p.to_path_buf()
    }
}

// ── poke ─────────────────────────────────────────────────────────

fn run_poke(args: PokeArgs) -> Result<()> {
    let cfg = api::load_config()
        .context("`slop poke` needs a server config. Run `slop login` first.")?;
    let (patch, source) = resolve_patch(&args)?;
    if patch.trim().is_empty() {
        bail!("nothing to scan ({source})");
    }
    if args.dry_run {
        let preview = serde_json::json!({
            "project": args.project,
            "patch_bytes": patch.len(),
            "source": source,
        });
        println!("{}", serde_json::to_string_pretty(&preview)?);
        return Ok(());
    }

    let resp = api::poke(&cfg, args.project.as_deref(), &patch)?;
    eprintln!(
        "slop poke: {} ({} ms, {}/{} this cycle)",
        resp.verdict, resp.elapsed_ms, resp.usage.poke_calls, resp.cap
    );
    for f in &resp.findings {
        println!("[{}] {}:{} — {}", f.category, f.file, f.line, f.matched);
    }
    save_plan(&resp)?;
    if !resp.cleanup_actions.is_empty() {
        eprintln!(
            "slop: {} line(s) flagged. Run `slop apply` to strip them.",
            resp.cleanup_actions.len()
        );
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedPlan {
    poke_id: String,
    verdict: String,
    cleanup_actions: Vec<CleanupAction>,
}

fn save_plan(r: &api::PokeResponse) -> Result<()> {
    let dir = Path::new(".slop");
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("slop: warning — could not create .slop dir: {e}");
        return Ok(());
    }
    let plan = CachedPlan {
        poke_id: r.poke_id.clone(),
        verdict: r.verdict.clone(),
        cleanup_actions: r
            .cleanup_actions
            .iter()
            .map(|a| CleanupAction {
                file: a.file.clone(),
                line: a.line,
                action: a.action.clone(),
                content: a.content.clone(),
            })
            .collect(),
    };
    fs::write(CACHED_PLAN, serde_json::to_string_pretty(&plan)?)?;
    Ok(())
}

// ── apply ────────────────────────────────────────────────────────

fn run_apply(args: ApplyArgs) -> Result<()> {
    let path = PathBuf::from(CACHED_PLAN);
    if args.discard {
        if path.exists() {
            fs::remove_file(&path)?;
            eprintln!("slop: discarded {CACHED_PLAN}");
        } else {
            eprintln!("slop: no cached plan");
        }
        return Ok(());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read {CACHED_PLAN} (run `slop poke` first)"))?;
    let plan: CachedPlan = serde_json::from_str(&raw)?;
    if args.show {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }
    if plan.cleanup_actions.is_empty() {
        eprintln!("slop: nothing to apply (LGTM)");
        return Ok(());
    }

    let mut by_file: std::collections::BTreeMap<String, Vec<&CleanupAction>> =
        std::collections::BTreeMap::new();
    for a in &plan.cleanup_actions {
        by_file.entry(a.file.clone()).or_default().push(a);
    }
    let mut touched = Vec::new();
    for (file, mut acts) in by_file {
        acts.sort_by_key(|b| std::cmp::Reverse(b.line));
        if !PathBuf::from(&file).exists() {
            eprintln!("slop: skip {file} (not in working tree)");
            continue;
        }
        let body = fs::read_to_string(&file).with_context(|| format!("read {file}"))?;
        let mut lines: Vec<String> = body.lines().map(String::from).collect();
        let trailing_newline = body.ends_with('\n');
        let mut deleted = 0usize;
        for a in acts {
            if a.action != "delete_line" {
                continue;
            }
            let idx = match a.line.checked_sub(1) {
                Some(n) if n < lines.len() => n,
                _ => continue,
            };
            let actual = lines[idx].trim_end_matches('\r');
            if actual != a.content.trim_end_matches('\r') {
                eprintln!(
                    "slop: skip {file}:{} — content drifted (expected {:?}, got {:?})",
                    a.line, a.content, actual
                );
                continue;
            }
            lines.remove(idx);
            deleted += 1;
        }
        if deleted == 0 {
            continue;
        }
        let mut out = lines.join("\n");
        if trailing_newline {
            out.push('\n');
        }
        fs::write(&file, out).with_context(|| format!("write {file}"))?;
        touched.push((file, deleted));
    }

    if touched.is_empty() {
        eprintln!("slop: no clean deletes (content drift); leaving working tree intact");
        return Ok(());
    }
    for (f, n) in &touched {
        eprintln!("slop: trimmed {n} line(s) from {f}");
    }
    let mut add_args = vec!["add", "--"];
    for (f, _) in &touched {
        add_args.push(f);
    }
    git_run(&add_args)?;
    if args.no_commit {
        eprintln!("slop: staged. Commit when ready.");
        return Ok(());
    }
    git_run(&["commit", "--amend", "--no-edit"])?;
    eprintln!("slop: HEAD amended.");
    Ok(())
}

/// Choose what to send to `slop poke`. Precedence (clap enforces
/// mutual exclusion at parse time): `--patch` > `--staged` >
/// `--range` > `--since` > default `git diff HEAD` (working tree
/// versus HEAD). Returns the raw patch + a short human label of
/// where it came from for error / dry-run output.
fn resolve_patch(args: &PokeArgs) -> Result<(String, String)> {
    if let Some(p) = args.patch.as_ref() {
        let body =
            fs::read_to_string(p).with_context(|| format!("read patch file {}", p.display()))?;
        return Ok((body, format!("--patch {}", p.display())));
    }
    if args.staged {
        return Ok((
            git_diff(&["--cached"])?,
            "--staged (git diff --cached)".into(),
        ));
    }
    if let Some(r) = args.range.as_deref() {
        return Ok((git_diff(&[r])?, format!("--range {r}")));
    }
    if let Some(s) = args.since.as_deref() {
        return Ok((git_diff(&[s])?, format!("--since {s}")));
    }
    Ok((git_diff(&["HEAD"])?, "git diff HEAD (default)".into()))
}

fn git_diff(extra: &[&str]) -> Result<String> {
    let mut argv: Vec<&str> = vec!["diff", "--no-color"];
    argv.extend_from_slice(extra);
    let out = Command::new("git")
        .args(&argv)
        .output()
        .with_context(|| format!("spawn git {}", argv.join(" ")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!(
            "git {} failed (exit {:?}): {}",
            argv.join(" "),
            out.status.code(),
            stderr.trim()
        );
    }
    Ok(String::from_utf8(out.stdout)?)
}

fn git_run(args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .status()
        .with_context(|| format!("spawn git {}", args.join(" ")))?;
    if !status.success() {
        bail!("git {} failed (exit {:?})", args.join(" "), status.code());
    }
    Ok(())
}

// ── learn ────────────────────────────────────────────────────────

fn run_learn(args: LearnArgs) -> Result<()> {
    let cfg = api::load_config()
        .context("`slop learn` needs a server config. Run `slop login` first.")?;
    let resp = api::learn(
        &cfg,
        &args.feedback,
        args.context.as_deref(),
        args.project.as_deref(),
    )?;
    eprintln!(
        "slop learn: queued {} ({}/{}) — {} bytes",
        resp.entry_id, resp.queued, resp.monthly_cap, resp.bytes
    );
    Ok(())
}

// ── billing ──────────────────────────────────────────────────────

fn run_billing(cmd: BillingCmd) -> Result<()> {
    let cfg = api::load_config()?;
    match cmd {
        BillingCmd::Tier => {
            let resp = api::billing_tier(&cfg)?;
            println!(
                "{}: poke {}/{} | review_tokens {}/{}",
                resp.slop_org,
                resp.usage.poke_calls,
                resp.entitlements.poke_calls_cap,
                resp.usage.review_tokens,
                resp.entitlements.review_token_cap
            );
            Ok(())
        }
        BillingCmd::Portal { print } => {
            let resp = api::billing_portal(&cfg)?;
            if print {
                println!("{}", resp.url);
                return Ok(());
            }
            eprintln!("slop: opening Stripe portal — {}", resp.url);
            let _ = Command::new("open").arg(&resp.url).status();
            Ok(())
        }
    }
}
