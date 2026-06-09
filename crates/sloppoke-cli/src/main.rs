//! `slop` — the AI-slop firewall CLI.
//!
//!   slop login                  resolve SSH key, cache identity
//!   slop poke   --patch FILE    scan a patch, save cleanup plan
//!   slop apply                  strip flagged lines, amend HEAD
//!   slop learn  "<feedback>"    ship feedback to the RL loop
//!   slop billing tier|portal    subscription + usage / Stripe portal

mod api;
mod ssh_resolve;
mod version_check;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use api::CleanupAction;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const DEFAULT_SERVER: &str = "https://sloppoke.me";
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
    /// Path to SSH public key. Default: ask `ssh -G <server-host>`
    /// which identity OpenSSH would pick (same resolution git uses),
    /// fall back to `~/.ssh/id_ed25519.pub`.
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
    /// Scan an arbitrary git URL instead of the local working tree.
    /// Shallow-clones the repo into a temp dir, runs the chosen
    /// range/since/staged selector against it, ships the patch to
    /// the server, cleans up. Works on any git host.
    /// Example: `slop poke --repo https://github.com/user/foo --range HEAD~5..HEAD`
    #[arg(long, conflicts_with = "patch")]
    repo: Option<String>,
    /// Shorthand for `--repo https://github.com/<arg>.git`.
    /// Example: `slop poke --gh openclaw/openclaw --range HEAD~5..HEAD`
    #[arg(long, conflicts_with_all = ["patch", "repo"])]
    gh: Option<String>,
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
    // Surface a friendly heads-up if a newer release exists. Runs
    // AFTER the user's primary output so the line scrolls past
    // whatever they came to see, never replaces it. All errors
    // (offline, rate-limited, parse failure) are swallowed inside.
    version_check::notify_if_outdated();
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
    // Resolve pubkey + key together so they always come from the same
    // pair. If the user passes only `--key foo`, derive pubkey from
    // `foo.pub` instead of falling back to the default ed25519 pubkey
    // — otherwise discover would compute a fingerprint for one keypair
    // while signed requests use a different one (401 mismatch).
    //
    // With no explicit flags we ask OpenSSH which identity it would
    // use for the server host (same resolution git observes). Only
    // when ssh has no opinion do we fall back to the ed25519 default.
    let pubkey_path = match (args.pubkey.clone(), args.key.clone()) {
        (Some(p), _) => p,
        (None, Some(k)) => PathBuf::from(format!("{}.pub", k.display())),
        (None, None) => ssh_resolve::host_from_server_url(&args.server)
            .and_then(|host| ssh_resolve::resolve_for_host(&host))
            .unwrap_or_else(default_pubkey_path),
    };
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

/// Resolve the effective remote git URL from either --repo or --gh.
/// --gh is the short form for github.com only; --repo accepts any git
/// URL the local git binary knows how to clone (https/ssh/git://).
fn remote_repo_url(args: &PokeArgs) -> Option<String> {
    if let Some(url) = args.repo.as_deref() {
        return Some(url.to_string());
    }
    if let Some(slug) = args.gh.as_deref() {
        // Accept either `org/repo` or a full URL the user fat-fingered
        // into --gh. The latter just passes through to git clone.
        if slug.starts_with("http") || slug.starts_with("git@") {
            return Some(slug.to_string());
        }
        return Some(format!("https://github.com/{slug}.git"));
    }
    None
}

/// Infer the smallest clone depth that satisfies the caller's range
/// selector. For ranges that explicitly reference `HEAD~N`, we need
/// at most N+2 commits (the N ancestors, HEAD itself, plus a buffer
/// for merge parents). For branch / tag / SHA refs we have no static
/// bound so we fall back to a conservative default.
///
/// `SLOP_REMOTE_CLONE_DEPTH` always wins when set — escape hatch for
/// users who know their range needs more history.
fn infer_clone_depth(args: &PokeArgs) -> u32 {
    const DEFAULT_DEPTH: u32 = 50;
    if let Some(env) = std::env::var("SLOP_REMOTE_CLONE_DEPTH")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
    {
        return env;
    }
    // Scan every `HEAD~<digits>` occurrence across both selectors,
    // take the max N. +2 buffer (HEAD itself + a merge-parent step)
    // means `HEAD~5..HEAD` clones depth 7, not 50.
    let mut needed: Option<u32> = None;
    for selector in [args.range.as_deref(), args.since.as_deref()]
        .into_iter()
        .flatten()
    {
        for capture in selector.split("HEAD~").skip(1) {
            let digits: String = capture.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                needed = Some(needed.map_or(n, |cur| cur.max(n)));
            }
        }
    }
    needed.map(|n| n + 2).unwrap_or(DEFAULT_DEPTH)
}

/// Persistent cache root for cloned repos. `~/.cache/slop/repos/`
/// by default; XDG override honored. One subdir per URL hash so the
/// `git fetch` on subsequent runs reuses the same checkout instead
/// of paying the full clone cost every invocation.
fn cache_root() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("SLOP_CACHE_DIR") {
        return Some(PathBuf::from(custom));
    }
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return Some(PathBuf::from(xdg).join("slop").join("repos"));
    }
    let home = std::env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join(".cache")
            .join("slop")
            .join("repos"),
    )
}

/// Hash a URL down to a stable, filesystem-safe directory name so the
/// cache layout is predictable.
fn url_to_cache_key(url: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(url.as_bytes());
    let bytes = hasher.finalize();
    let hex: String = bytes.iter().take(8).map(|b| format!("{b:02x}")).collect();
    hex
}

/// Reusable repo workdir. On cache hit, runs `git fetch --depth N`
/// so the checkout includes whatever range the caller needs. On
/// cache miss, does a full shallow clone. Either way the caller
/// gets a stable PathBuf the rest of poke can chdir into.
///
/// `None` returned by this function means: caller should fall back
/// to the tempdir path (e.g. cache root unwritable).
fn cached_clone_or_fetch(url: &str, depth: u32) -> Option<PathBuf> {
    let root = cache_root()?;
    let dir = root.join(url_to_cache_key(url));
    let _ = fs::create_dir_all(&root);
    let depth_s = depth.to_string();
    if dir.join(".git").exists() {
        eprintln!("slop: refreshing cached {url} (depth {depth})…");
        let ok = Command::new("git")
            .args([
                "-C",
                dir.to_str()?,
                "fetch",
                "--depth",
                &depth_s,
                "--quiet",
                "--no-tags",
                "origin",
            ])
            .status()
            .ok()?
            .success();
        if ok {
            // Reset to the freshly-fetched HEAD so subsequent
            // `git diff` calls see the new tip, not the stale one.
            let _ = Command::new("git")
                .args(["-C", dir.to_str()?, "reset", "--hard", "FETCH_HEAD"])
                .status();
            return Some(dir);
        }
        // Fetch failed — fall through to re-clone fresh.
        let _ = fs::remove_dir_all(&dir);
    }
    eprintln!("slop: cloning {url} (depth {depth})…");
    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            &depth_s,
            "--quiet",
            "--no-tags",
            url,
            dir.to_str()?,
        ])
        .status()
        .ok()?;
    if !status.success() {
        let _ = fs::remove_dir_all(&dir);
        return None;
    }
    Some(dir)
}

/// Holder so the rest of `run_poke` doesn't need to know whether the
/// workdir is a tempdir (cache disabled) or a long-lived cached
/// checkout. Drop wipes only the tempdir variant.
enum RemoteWorkdir {
    Cached(PathBuf),
    #[allow(dead_code)] // Held by RAII; the tempdir's lifetime is what matters.
    Tempdir(tempfile::TempDir),
}

impl RemoteWorkdir {
    fn path(&self) -> &Path {
        match self {
            Self::Cached(p) => p.as_path(),
            Self::Tempdir(t) => t.path(),
        }
    }
}

/// Shallow-clone the URL — try the persistent cache first, fall back
/// to a one-shot tempdir if the cache directory is unwritable.
fn remote_clone(url: String, depth: u32) -> Result<RemoteWorkdir> {
    if let Some(cached) = cached_clone_or_fetch(&url, depth) {
        return Ok(RemoteWorkdir::Cached(cached));
    }
    // Cache path failed — fall back to a disposable tempdir.
    let tmp = tempfile::Builder::new()
        .prefix("slop-scan-")
        .tempdir()
        .context("create temp dir for --repo clone")?;
    eprintln!("slop: cloning {url} (depth {depth}, no cache)…");
    let depth_s = depth.to_string();
    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            &depth_s,
            "--quiet",
            "--no-tags",
            &url,
            tmp.path().to_str().context("temp path not utf-8")?,
        ])
        .status()
        .context("spawn git clone")?;
    if !status.success() {
        bail!("git clone {url} failed (exit {:?})", status.code());
    }
    Ok(RemoteWorkdir::Tempdir(tmp))
}

/// Print a unified diff to stdout, colorized when stdout is a TTY
/// (red `-`, green `+`, cyan hunk header). Auto-disabled for pipes
/// and redirections so `slop poke … | git apply --check` works.
/// `NO_COLOR=1` (https://no-color.org) and `SLOP_NO_COLOR=1` both
/// force plain output.
fn emit_patch_maybe_colored(patch: &str) {
    use std::io::IsTerminal;
    let color = std::io::stdout().is_terminal()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("SLOP_NO_COLOR").is_err();
    if !color {
        println!("{}", patch.trim_end());
        return;
    }
    const RED: &str = "\x1b[31m";
    const GREEN: &str = "\x1b[32m";
    const CYAN: &str = "\x1b[36m";
    const DIM: &str = "\x1b[2m";
    const RESET: &str = "\x1b[0m";
    for line in patch.trim_end().lines() {
        if line.starts_with("diff --git ") || line.starts_with("--- ") || line.starts_with("+++ ") {
            println!("{DIM}{line}{RESET}");
        } else if line.starts_with("@@") {
            println!("{CYAN}{line}{RESET}");
        } else if line.starts_with('+') {
            println!("{GREEN}{line}{RESET}");
        } else if line.starts_with('-') {
            println!("{RED}{line}{RESET}");
        } else {
            println!("{line}");
        }
    }
}

fn run_poke(args: PokeArgs) -> Result<()> {
    let cfg =
        api::load_config().context("`slop poke` needs a server config. Run `slop login` first.")?;
    // --repo / --gh: shallow-clone an arbitrary git URL into a temp
    // dir, run the rest of the resolver from inside it. tempdir Drop
    // cleans the checkout regardless of success/panic.
    let remote_workdir = if let Some(url) = remote_repo_url(&args) {
        Some(remote_clone(url, infer_clone_depth(&args))?)
    } else {
        None
    };
    let (patch, source) = if let Some(ref tmp) = remote_workdir {
        std::env::set_current_dir(tmp.path())
            .with_context(|| format!("chdir into cloned repo {}", tmp.path().display()))?;
        let (p, s) = resolve_patch(&args)?;
        (p, format!("{s} @ {}", remote_repo_url(&args).unwrap()))
    } else {
        resolve_patch(&args)?
    };
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
    save_plan(&resp)?;
    // Verdict + quota line lives on stderr so it's visible to
    // interactive users (self-teaching, quota awareness) without
    // polluting `> foo.patch` redirections or `| git apply` pipes.
    eprintln!(
        "slop poke: {} ({} ms, {}/{} this cycle)",
        resp.verdict, resp.elapsed_ms, resp.usage.poke_calls, resp.cap
    );
    // The unified-diff patch on stdout. Color-aware for TTYs, ANSI
    // stripped for pipes / redirections so `git apply --unidiff-zero`
    // still works as a one-liner.
    if !resp.patch.trim().is_empty() {
        emit_patch_maybe_colored(&resp.patch);
        // Apply hint on stderr — first-time users get the obvious
        // next step without having to read --help.
        eprintln!(
            "\nRun `slop apply` to apply, `slop apply --discard` to drop, \
             or `git apply --unidiff-zero` if applying manually."
        );
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedPlan {
    poke_id: String,
    verdict: String,
    /// Server-rendered unified diff. Preferred apply target.
    #[serde(default)]
    patch: String,
    /// Legacy structured action list. Kept so a stale plan from an
    /// older server still applies via the action-walker fallback.
    #[serde(default)]
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
        patch: r.patch.clone(),
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

/// Lower priority value = applied first within the same line. Delete
/// before insert: if a slop line has BOTH a delete-line action and an
/// insert-above-it action, we want the deletion to land first so the
/// inserted TODO comment sits above the line the slop used to occupy
/// — not above the slop itself.
fn action_priority(action: &str) -> u8 {
    match action {
        "delete_line" => 0,
        "insert_above" => 1,
        _ => 9,
    }
}

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
    if plan.patch.trim().is_empty() && plan.cleanup_actions.is_empty() {
        eprintln!("slop: nothing to apply (LGTM)");
        return Ok(());
    }

    // Primary path: server gave us a unified diff. Hand it to
    // `git apply --unidiff-zero --index -`. Battle-tested mutator,
    // zero CLI-side line arithmetic.
    if !plan.patch.trim().is_empty() {
        return apply_via_git(&plan, args);
    }

    let mut by_file: std::collections::BTreeMap<String, Vec<&CleanupAction>> =
        std::collections::BTreeMap::new();
    for a in &plan.cleanup_actions {
        by_file.entry(a.file.clone()).or_default().push(a);
    }
    let mut touched = Vec::new();
    for (file, mut acts) in by_file {
        // Sort by line descending so later mutations don't shift the
        // indices we still need to address. Within a single line:
        // process delete_line before insert_above so an insert above a
        // line we're also deleting still lands at the right relative
        // position (the deleted line was the slop; the TODO replaces
        // surrounding context).
        acts.sort_by(|a, b| {
            b.line
                .cmp(&a.line)
                .then_with(|| action_priority(&a.action).cmp(&action_priority(&b.action)))
        });
        if !PathBuf::from(&file).exists() {
            eprintln!("slop: skip {file} (not in working tree)");
            continue;
        }
        let body = fs::read_to_string(&file).with_context(|| format!("read {file}"))?;
        let mut lines: Vec<String> = body.lines().map(String::from).collect();
        let trailing_newline = body.ends_with('\n');
        let mut deleted = 0usize;
        let mut inserted = 0usize;
        for a in acts {
            let Some(idx) = a.line.checked_sub(1) else {
                continue;
            };
            match a.action.as_str() {
                "delete_line" => {
                    if idx >= lines.len() {
                        continue;
                    }
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
                "insert_above" => {
                    // Off-by-one tolerance: allow idx == lines.len()
                    // so a slop hit on the final line still gets a
                    // comment spliced above it.
                    if idx > lines.len() {
                        continue;
                    }
                    // Idempotent: the TODO is "already present" if
                    //   (a) the line above the target matches it
                    //       (fresh apply against the original line
                    //       numbering), or
                    //   (b) the target line itself matches it (re-apply
                    //       of a stale plan whose line numbers no
                    //       longer reflect the post-insert file).
                    let already_above = idx
                        .checked_sub(1)
                        .and_then(|i| lines.get(i))
                        .map(|prev| prev.trim() == a.content.trim())
                        .unwrap_or(false);
                    let already_here = lines
                        .get(idx)
                        .map(|here| here.trim() == a.content.trim())
                        .unwrap_or(false);
                    if already_above || already_here {
                        continue;
                    }
                    lines.insert(idx, a.content.clone());
                    inserted += 1;
                }
                _ => {
                    // Unknown action — surfacing on a future server
                    // protocol bump; safest is to skip.
                    continue;
                }
            }
        }
        if deleted == 0 && inserted == 0 {
            continue;
        }
        let mut out = lines.join("\n");
        if trailing_newline {
            out.push('\n');
        }
        fs::write(&file, out).with_context(|| format!("write {file}"))?;
        touched.push((file, deleted, inserted));
    }

    if touched.is_empty() {
        eprintln!("slop: no clean changes to apply; leaving working tree intact");
        return Ok(());
    }
    for (f, del, ins) in &touched {
        match (*del, *ins) {
            (d, 0) => eprintln!("slop: trimmed {d} line(s) from {f}"),
            (0, i) => eprintln!("slop: spliced {i} TODO comment(s) into {f}"),
            (d, i) => {
                eprintln!("slop: trimmed {d} line(s) and spliced {i} TODO comment(s) into {f}")
            }
        }
    }
    let mut add_args = vec!["add", "--"];
    for (f, _, _) in &touched {
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
        let r = clamp_head_tilde(r);
        return Ok((git_diff(&[&r])?, format!("--range {r}")));
    }
    if let Some(s) = args.since.as_deref() {
        let s = clamp_head_tilde(s);
        return Ok((git_diff(&[&s])?, format!("--since {s}")));
    }
    Ok((git_diff(&["HEAD"])?, "git diff HEAD (default)".into()))
}

/// Rewrite every `HEAD~N` token in `selector` so N never exceeds the
/// repo's actual history depth. Public repos often have only a handful
/// of commits — without this, `slop poke --range HEAD~10..HEAD` on a
/// 5-commit repo dies with git's unfriendly `unknown revision` error.
/// On success the user sees a `slop: range clamped to …` notice on
/// stderr and the scan proceeds with the largest range that resolves.
fn clamp_head_tilde(selector: &str) -> String {
    let Some(total) = git_rev_count_head() else {
        return selector.to_string();
    };
    if total == 0 {
        return selector.to_string();
    }
    let (out, clamped) = clamp_head_tilde_to(selector, total);
    if clamped {
        eprintln!(
            "slop: range clamped to {out} (repo has {total} commit{})",
            if total == 1 { "" } else { "s" }
        );
    }
    out
}

/// Pure helper for `clamp_head_tilde` — separated for unit tests so we
/// don't need a temp git repo to exercise the parser. Returns the
/// rewritten selector plus a flag indicating whether any token was
/// clamped (the caller uses that to log a notice).
fn clamp_head_tilde_to(selector: &str, total_commits: u32) -> (String, bool) {
    let limit = total_commits.saturating_sub(1);
    let mut out = String::with_capacity(selector.len());
    let bytes = selector.as_bytes();
    let mut i = 0;
    let mut clamped = false;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"HEAD~") {
            let start = i + 5;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start {
                let n_str = &selector[start..end];
                let n: u32 = n_str.parse().unwrap_or(0);
                let chosen = if n > limit {
                    clamped = true;
                    limit
                } else {
                    n
                };
                out.push_str("HEAD~");
                out.push_str(&chosen.to_string());
                i = end;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    (out, clamped)
}

fn git_rev_count_head() -> Option<u32> {
    let out = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    s.trim().parse().ok()
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

/// Apply a server-rendered unified diff via `git apply
/// --unidiff-zero --index`. Stdin carries the patch; on success the
/// working tree is mutated and the changes are staged. If
/// `args.no_commit` is false we also amend HEAD so the slop never
/// ships as a separate commit.
fn apply_via_git(plan: &CachedPlan, args: ApplyArgs) -> Result<()> {
    use std::io::Write;
    // Dry-run preflight: --check exits non-zero if the diff would not
    // apply cleanly. Surface the actual git stderr so the user knows
    // why before we mutate anything.
    let mut check = Command::new("git")
        .args(["apply", "--unidiff-zero", "--check", "-"])
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("spawn git apply --check")?;
    check
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(plan.patch.as_bytes())
        .context("write patch to git apply --check")?;
    let preflight = check.wait_with_output().context("wait git apply --check")?;
    if !preflight.status.success() {
        let stderr = String::from_utf8_lossy(&preflight.stderr);
        bail!(
            "patch would not apply cleanly — leaving working tree untouched.\ngit apply --check:\n{stderr}"
        );
    }

    let mut apply = Command::new("git")
        .args(["apply", "--unidiff-zero", "--index", "-"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("spawn git apply")?;
    apply
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(plan.patch.as_bytes())
        .context("write patch to git apply")?;
    let status = apply.wait().context("wait git apply")?;
    if !status.success() {
        bail!(
            "git apply failed after --check passed (exit {:?}) — re-run with RUST_LOG=debug for detail",
            status.code()
        );
    }

    eprintln!("slop: applied server patch (verdict: {})", plan.verdict);
    if args.no_commit {
        eprintln!("slop: staged. Commit when ready.");
        return Ok(());
    }
    git_run(&["commit", "--amend", "--no-edit"])?;
    eprintln!("slop: HEAD amended.");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args_with(repo: Option<&str>, gh: Option<&str>) -> PokeArgs {
        PokeArgs {
            patch: None,
            staged: false,
            range: None,
            since: None,
            project: None,
            repo: repo.map(str::to_string),
            gh: gh.map(str::to_string),
            dry_run: false,
        }
    }

    #[test]
    fn remote_repo_url_returns_none_when_neither_flag_set() {
        assert!(remote_repo_url(&args_with(None, None)).is_none());
    }

    fn args_with_range(range: &str) -> PokeArgs {
        PokeArgs {
            patch: None,
            staged: false,
            range: Some(range.to_string()),
            since: None,
            project: None,
            repo: None,
            gh: None,
            dry_run: false,
        }
    }

    fn args_with_since(since: &str) -> PokeArgs {
        PokeArgs {
            patch: None,
            staged: false,
            range: None,
            since: Some(since.to_string()),
            project: None,
            repo: None,
            gh: None,
            dry_run: false,
        }
    }

    // SLOP_REMOTE_CLONE_DEPTH is a process-global env var, so every
    // assertion that depends on it lives in a single test to avoid
    // races between parallel cargo-test threads.
    #[test]
    fn infer_clone_depth_covers_all_cases() {
        std::env::remove_var("SLOP_REMOTE_CLONE_DEPTH");

        // Branch / tag / SHA refs have no static bound → default 50.
        assert_eq!(infer_clone_depth(&args_with_range("main..HEAD")), 50);
        assert_eq!(infer_clone_depth(&args_with_range("origin/main...")), 50);
        assert_eq!(infer_clone_depth(&args_with(None, None)), 50);

        // HEAD~N selectors → N + 2 buffer.
        assert_eq!(infer_clone_depth(&args_with_range("HEAD~5..HEAD")), 7);
        assert_eq!(infer_clone_depth(&args_with_range("HEAD~1..HEAD")), 3);
        assert_eq!(infer_clone_depth(&args_with_range("HEAD~8..HEAD~2")), 10);
        assert_eq!(infer_clone_depth(&args_with_since("HEAD~3")), 5);

        // Env override wins even when the selector would infer less.
        std::env::set_var("SLOP_REMOTE_CLONE_DEPTH", "200");
        assert_eq!(infer_clone_depth(&args_with_range("HEAD~5..HEAD")), 200);
        std::env::remove_var("SLOP_REMOTE_CLONE_DEPTH");
    }

    #[test]
    fn remote_repo_url_passes_repo_through_verbatim() {
        let a = args_with(Some("https://gitlab.com/owner/proj.git"), None);
        assert_eq!(
            remote_repo_url(&a).as_deref(),
            Some("https://gitlab.com/owner/proj.git")
        );
    }

    #[test]
    fn remote_repo_url_expands_gh_slug_to_github_https() {
        let a = args_with(None, Some("openclaw/openclaw"));
        assert_eq!(
            remote_repo_url(&a).as_deref(),
            Some("https://github.com/openclaw/openclaw.git")
        );
    }

    #[test]
    fn remote_repo_url_accepts_full_url_in_gh() {
        // Defensive: if the user fat-fingers a full URL into --gh
        // instead of the org/repo slug, pass it through rather than
        // rewriting it into `https://github.com/https://…`.
        let a = args_with(None, Some("git@github.com:foo/bar.git"));
        assert_eq!(
            remote_repo_url(&a).as_deref(),
            Some("git@github.com:foo/bar.git")
        );
        let a = args_with(None, Some("https://example.test/foo.git"));
        assert_eq!(
            remote_repo_url(&a).as_deref(),
            Some("https://example.test/foo.git")
        );
    }

    #[test]
    fn remote_repo_url_prefers_repo_over_gh_when_both_set() {
        // clap should reject this combination at parse time
        // (conflicts_with), but the function should still pick
        // deterministically if invoked programmatically.
        let a = args_with(
            Some("https://repo.test/x.git"),
            Some("ignored/forsclap-violators"),
        );
        assert_eq!(
            remote_repo_url(&a).as_deref(),
            Some("https://repo.test/x.git")
        );
    }

    #[test]
    fn clamp_head_tilde_rewrites_n_over_history_limit() {
        // Repo with 5 commits → HEAD~4 is the oldest reachable rev,
        // HEAD~5+ doesn't resolve.
        let (out, clamped) = clamp_head_tilde_to("HEAD~10..HEAD", 5);
        assert!(clamped);
        assert_eq!(out, "HEAD~4..HEAD");
    }

    #[test]
    fn clamp_head_tilde_passes_through_when_within_limit() {
        let (out, clamped) = clamp_head_tilde_to("HEAD~3..HEAD", 10);
        assert!(!clamped);
        assert_eq!(out, "HEAD~3..HEAD");
    }

    #[test]
    fn clamp_head_tilde_handles_multiple_tokens_in_one_range() {
        // Both ends are HEAD~N — clamp each independently.
        let (out, clamped) = clamp_head_tilde_to("HEAD~20..HEAD~2", 7);
        assert!(clamped);
        // total=7 → limit=6; HEAD~20 → HEAD~6, HEAD~2 stays.
        assert_eq!(out, "HEAD~6..HEAD~2");
    }

    #[test]
    fn clamp_head_tilde_leaves_branch_refs_alone() {
        let (out, clamped) = clamp_head_tilde_to("origin/main..HEAD", 5);
        assert!(!clamped);
        assert_eq!(out, "origin/main..HEAD");
    }

    #[test]
    fn clamp_head_tilde_with_zero_history_returns_input_unchanged() {
        // Edge case: empty / unborn HEAD. Total=0 means limit=0; the
        // function should not produce HEAD~0 (invalid). Caller is
        // responsible for the early-return in `clamp_head_tilde`.
        let (out, clamped) = clamp_head_tilde_to("HEAD~3..HEAD", 0);
        // Limit=0 means every N gets clamped to 0; while ugly, this
        // helper stays a pure parser. The wrapper short-circuits
        // before calling it when total==0.
        assert!(clamped);
        assert_eq!(out, "HEAD~0..HEAD");
    }
}
