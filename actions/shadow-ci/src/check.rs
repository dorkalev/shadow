// shadow-ci check — the per-PR compliance audit.
//
// Deterministic policy layer (no LLM in CI; judgment lives in the platform's
// agent runbooks):
//   1. Ticket traceability   — pattern over PR title+body; optional Linear verification. HARD GATE.
//   2. Description           — PR body >= 20 chars, 4 required sections.        HARD GATE (length).
//   3. Change traceability   — every changed file mentioned in the PR body.     -10 each.
//   4. Test coverage         — changed source files have tests.                 -5 each.
//   5. Review gate           — unresolved CRITICAL/MAJOR bot findings.          HARD GATE.
//                              required reviewers present (post-review phase).  -5 + HARD GATE.
// Score starts at 100; fail below CONFIDENCE_THRESHOLD or on any hard gate.
// Posts/updates one PR comment per phase, keyed by an HTML marker.
use crate::util::{curl_post, env_or, gh, gh_json, git, run};
use regex::Regex;
use serde_json::{json, Value};

pub struct Config {
    pub repo: String,
    pub pr: String,
    pub ticket_pattern: String,
    pub phase: String, // awaiting-review | post-review
    pub agent_key: String,
    pub threshold: i64,
    pub required_reviewers: Vec<String>,
    pub expected_reviewers: Vec<String>,
    pub test_exclude_paths: Vec<String>,
    pub linear_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Config, String> {
        let phase = env_or("REVIEW_PHASE", "awaiting-review");
        let csv = |k: &str| -> Vec<String> {
            env_or(k, "")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        Ok(Config {
            repo: std::env::var("REPO")
                .or_else(|_| std::env::var("GITHUB_REPOSITORY"))
                .map_err(|_| "REPO or GITHUB_REPOSITORY required")?,
            pr: std::env::var("PR_NUMBER").map_err(|_| "PR_NUMBER required")?,
            ticket_pattern: env_or("TICKET_PATTERN", r"[A-Z]{2,6}-[0-9]+|#[0-9]+"),
            agent_key: env_or(
                "AGENT_KEY",
                if phase == "post-review" {
                    "review-gate"
                } else {
                    "audit"
                },
            ),
            phase,
            threshold: env_or("CONFIDENCE_THRESHOLD", "70").parse().unwrap_or(70),
            required_reviewers: csv("REQUIRED_REVIEWERS"),
            expected_reviewers: csv("EXPECTED_REVIEWERS"),
            test_exclude_paths: csv("TEST_EXCLUDE_PATHS"),
            linear_key: std::env::var("LINEAR_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),
        })
    }
}

// ---------- pure policy functions (unit-tested below) ----------

/// Deterministic, order-preserving, de-duplicated ticket extraction from PR
/// title+body ONLY (crash-safe: never derived from the diff or comments).
pub fn extract_tickets(title: &str, body: &str, pattern: &str) -> Vec<String> {
    let re = match Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for m in re.find_iter(&format!("{title}\n{body}")) {
        if seen.insert(m.as_str().to_string()) {
            out.push(m.as_str().to_string());
        }
    }
    out
}

/// A changed file is traceable if its full path or basename appears in the PR body.
pub fn is_traceable(path: &str, body_lower: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    body_lower.contains(&path.to_lowercase()) || body_lower.contains(&base.to_lowercase())
}

/// Housekeeping files that never need their own spec line.
pub fn is_exempt_minor(path: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    matches!(
        base,
        "Cargo.lock"
            | "package-lock.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "poetry.lock"
            | "Gemfile.lock"
            | ".gitignore"
            | ".gitattributes"
    )
}

const SOURCE_EXT: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "rb", "java", "kt", "swift", "c", "cc", "cpp",
];

pub fn is_source_file(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .map_or(false, |e| SOURCE_EXT.contains(&e))
}

pub fn is_test_file(path: &str) -> bool {
    let p = path.to_lowercase();
    let base = p.rsplit('/').next().unwrap_or(&p);
    p.contains("/tests/")
        || p.contains("/__tests__/")
        || p.starts_with("tests/")
        || base.starts_with("test_")
        || base.contains("_test.")
        || base.contains(".test.")
        || base.contains(".spec.")
}

/// Does any file in `candidates` look like a test for `src_path`?
pub fn has_test_for(src_path: &str, candidates: &[String]) -> bool {
    let base = src_path.rsplit('/').next().unwrap_or(src_path);
    let stem = base
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(base)
        .to_lowercase();
    if stem.is_empty() {
        return false;
    }
    candidates
        .iter()
        .any(|c| is_test_file(c) && c.to_lowercase().contains(&stem))
}

/// Severity classification for review-bot findings.
pub fn severity_of(text: &str) -> Option<&'static str> {
    let t = text.to_lowercase();
    if Regex::new(r"\bcritical\b").unwrap().is_match(&t) {
        return Some("critical");
    }
    if Regex::new(r"\bmajor\b|\bpotential issue\b")
        .unwrap()
        .is_match(&t)
    {
        return Some("major");
    }
    None
}

/// Reviewers are configured by their actual GitHub login (REQUIRED_REVIEWERS),
/// except the built-in `shadow-reviewer`, whose presence is marker-based
/// (it posts as github-actions[bot], the same login as the compliance bot,
/// which must not satisfy the requirement itself).
pub fn bot_login_for(name: &str) -> String {
    match name {
        "shadow-reviewer" => "github-actions".into(),
        other => other.into(),
    }
}

/// A completed semantic review carries this exact marker. Unavailable, skipped,
/// timed-out, or failed review runs use different markers and never satisfy a
/// configured reviewer requirement.
pub const SHADOW_REVIEW_MARKER: &str = "<!-- shadow-review:complete -->";

/// The marker counts ONLY when a bot posted it — an author pasting the marker
/// string into a normal comment must not fake a completed bot run. `author` is
/// the PR author's login, excluded even if it somehow ends in [bot].
pub fn shadow_review_posted_by_bot(comments: &Value, author: &str) -> bool {
    comments
        .as_array()
        .map(|a| {
            a.iter().any(|c| {
                let login = c["user"]["login"].as_str().unwrap_or("");
                c["body"]
                    .as_str()
                    .unwrap_or("")
                    .contains(SHADOW_REVIEW_MARKER)
                    && login.ends_with("[bot]")
                    && login != author
            })
        })
        .unwrap_or(false)
}

/// Attestation-side presence check (no author to exclude): marker from a bot.
#[cfg(test)]
pub fn shadow_review_posted(comments: &Value) -> bool {
    comments
        .as_array()
        .map(|a| {
            a.iter().any(|c| {
                c["body"]
                    .as_str()
                    .unwrap_or("")
                    .contains(SHADOW_REVIEW_MARKER)
                    && c["user"]["login"].as_str().unwrap_or("").ends_with("[bot]")
            })
        })
        .unwrap_or(false)
}

pub fn calculate_score(
    invalid_tickets: usize,
    unspecced: usize,
    untested: usize,
    missing_reviewers: usize,
) -> i64 {
    let score = 100i64
        - 10 * invalid_tickets as i64
        - 10 * unspecced as i64
        - 5 * untested as i64
        - 5 * missing_reviewers as i64;
    score.clamp(0, 100)
}

// ---------- data gathering ----------

/// GitHub Issues as the tracker: the ticket is real if the number exists,
/// is an ISSUE (not a PR — the issues API returns both, PRs carry a
/// `pull_request` key), and is not this PR itself (a change cannot authorize
/// itself). 404 = invalid; other API failures = unknown (falls back to
/// extraction-as-evidence, never silently invalid).
fn github_issue_valid(repo: &str, num: &str, pr_number: &str) -> Option<bool> {
    if num == pr_number {
        return Some(false); // self-authorization
    }
    match crate::util::gh(&["api", &format!("repos/{repo}/issues/{num}")]) {
        Ok(out) => {
            let v: Value = serde_json::from_str(&out).ok()?;
            Some(v.get("pull_request").is_none() && v["number"].is_i64())
        }
        Err(e) if e.contains("404") || e.contains("Not Found") => Some(false),
        Err(_) => None,
    }
}

fn linear_ticket_exists(key: &str, id: &str) -> Option<bool> {
    let query = json!({
        "query": "query($id: String!) { issue(id: $id) { identifier title } }",
        "variables": { "id": id }
    });
    let resp = curl_post(
        "https://api.linear.app/graphql",
        &[("Content-Type", "application/json"), ("Authorization", key)],
        &query.to_string(),
    )
    .ok()?;
    let v: Value = serde_json::from_str(&resp).ok()?;
    Some(v["data"]["issue"]["identifier"].is_string())
}

fn unresolved_findings(cfg: &Config) -> Result<Vec<String>, String> {
    let (owner, name) = cfg.repo.split_once('/').ok_or("bad REPO")?;
    let query = "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100){nodes{isResolved comments(first:1){nodes{author{login} body}}}}}}}";
    let v = gh_json(&[
        "api",
        "graphql",
        "-f",
        &format!("query={query}"),
        "-f",
        &format!("owner={owner}"),
        "-f",
        &format!("name={name}"),
        "-F",
        &format!("number={}", cfg.pr),
    ])?;
    let mut out = Vec::new();
    if let Some(nodes) = v["data"]["repository"]["pullRequest"]["reviewThreads"]["nodes"].as_array()
    {
        for t in nodes {
            if t["isResolved"].as_bool() == Some(true) {
                continue;
            }
            let Some(c) = t["comments"]["nodes"].get(0) else {
                continue;
            };
            let login = c["author"]["login"].as_str().unwrap_or("");
            let body = c["body"].as_str().unwrap_or("");
            let configured: bool = cfg
                .required_reviewers
                .iter()
                .chain(cfg.expected_reviewers.iter())
                .any(|r| login.contains(&bot_login_for(r)));
            let is_bot = login.ends_with("[bot]") || configured;
            if !is_bot {
                continue;
            }
            if let Some(sev) = severity_of(body) {
                let first_line: String = body
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(120)
                    .collect();
                out.push(format!("[{sev}] {login}: {first_line}"));
            }
        }
    }
    Ok(out)
}

fn reviewer_posted(cfg: &Config, pr_json: &Value, comments: &Value, name: &str) -> bool {
    let _ = cfg;
    let author = pr_json["author"]["login"].as_str().unwrap_or("");
    if name == "shadow-reviewer" {
        return shadow_review_posted_by_bot(comments, author);
    }
    // Exact login match (tolerating the [bot] suffix) — a substring match would let
    // an unrelated actor whose login merely contains the configured name satisfy it.
    let want = bot_login_for(name);
    let is_reviewer = |login: &str| login == want || login == format!("{want}[bot]");
    let in_reviews = pr_json["reviews"]
        .as_array()
        .map(|a| {
            a.iter().any(|r| {
                is_reviewer(r["author"]["login"].as_str().unwrap_or(""))
                    && r["author"]["login"].as_str() != Some(author)
            })
        })
        .unwrap_or(false);
    let in_comments = comments
        .as_array()
        .map(|a| {
            a.iter()
                .any(|c| is_reviewer(c["user"]["login"].as_str().unwrap_or("")))
        })
        .unwrap_or(false);
    in_reviews || in_comments
}

fn upsert_comment(cfg: &Config, marker: &str, body: &str) -> Result<(), String> {
    let comments = gh_json(&[
        "api",
        &format!("repos/{}/issues/{}/comments", cfg.repo, cfg.pr),
        "--paginate",
    ])?;
    let existing = comments
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|c| c["body"].as_str().unwrap_or("").contains(marker))
        })
        .and_then(|c| c["id"].as_i64());
    match existing {
        Some(id) => gh(&[
            "api",
            "-X",
            "PATCH",
            &format!("repos/{}/issues/comments/{id}", cfg.repo),
            "-f",
            &format!("body={body}"),
        ])
        .map(|_| ()),
        None => gh(&[
            "api",
            "-X",
            "POST",
            &format!("repos/{}/issues/{}/comments", cfg.repo, cfg.pr),
            "-f",
            &format!("body={body}"),
        ])
        .map(|_| ()),
    }
}

// ---------- the audit ----------

pub fn run_check() -> Result<i32, String> {
    let cfg = Config::from_env()?;

    let pr = gh_json(&[
        "pr",
        "view",
        &cfg.pr,
        "--repo",
        &cfg.repo,
        "--json",
        "title,body,labels,files,reviews,isDraft",
    ])?;
    let title = pr["title"].as_str().unwrap_or("");
    let body = pr["body"].as_str().unwrap_or("");
    let body_lower = body.to_lowercase();
    let comments = gh_json(&[
        "api",
        &format!("repos/{}/issues/{}/comments", cfg.repo, cfg.pr),
        "--paginate",
    ])
    .unwrap_or(Value::Array(vec![]));

    // exempt label short-circuits (still leaves a visible record)
    let exempt = pr["labels"]
        .as_array()
        .map(|a| {
            a.iter()
                .any(|l| l["name"].as_str() == Some("compliance:exempt"))
        })
        .unwrap_or(false);

    let changed: Vec<String> = pr["files"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|f| f["path"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // 1. tickets. FAIL-CLOSED: a ticket we could not verify is `unverified`, NOT
    // valid — "couldn't determine authorization" must never satisfy the gate.
    let tickets = extract_tickets(title, body, &cfg.ticket_pattern);
    let mut valid = Vec::new();
    let mut invalid = Vec::new();
    let mut unverified = Vec::new();
    for t in &tickets {
        let verdict = if let Some(num) = t.strip_prefix('#') {
            github_issue_valid(&cfg.repo, num, &cfg.pr)
        } else if cfg.linear_key.is_some() {
            cfg.linear_key
                .as_deref()
                .and_then(|k| linear_ticket_exists(k, t))
        } else {
            None // Linear-style ticket but no key configured → cannot verify
        };
        match verdict {
            Some(true) => valid.push(t.clone()),
            Some(false) => invalid.push(t.clone()),
            None => unverified.push(t.clone()),
        }
    }

    // 2. description
    let body_len_ok = body.trim().len() >= 20;
    let missing_sections: Vec<&str> = ["## Summary", "## Tickets", "## Changes", "## Test Plan"]
        .iter()
        .filter(|s| !body.contains(**s))
        .copied()
        .collect();

    // 3. change traceability
    let unspecced: Vec<String> = changed
        .iter()
        .filter(|p| !is_exempt_minor(p) && !is_traceable(p, &body_lower))
        .cloned()
        .collect();

    // 4. test coverage
    let repo_files: Vec<String> = git(&["ls-files"])
        .map(|s| s.lines().map(String::from).collect())
        .unwrap_or_default();
    let mut candidates = repo_files;
    candidates.extend(changed.iter().cloned());
    let untested: Vec<String> = changed
        .iter()
        .filter(|p| {
            is_source_file(p)
                && !is_test_file(p)
                && !cfg.test_exclude_paths.iter().any(|x| p.starts_with(x))
                && !has_test_for(p, &candidates)
                && !std::fs::read_to_string(p)
                    .map(|src| src.contains("#[cfg(test)]") || src.contains("mod tests"))
                    .unwrap_or(false)
        })
        .cloned()
        .collect();

    // 5. review gate. FAIL-CLOSED: if the findings query errors we cannot claim
    // "no unresolved findings" — treat the evaluation as failed, not clean.
    let findings_err;
    let findings = match unresolved_findings(&cfg) {
        Ok(f) => {
            findings_err = false;
            f
        }
        Err(e) => {
            eprintln!("review-findings query failed: {e}");
            findings_err = true;
            Vec::new()
        }
    };
    let reviewers_to_check = if cfg.phase == "post-review" {
        &cfg.required_reviewers
    } else {
        &cfg.expected_reviewers
    };
    let missing_reviewers: Vec<String> = reviewers_to_check
        .iter()
        .filter(|r| !reviewer_posted(&cfg, &pr, &comments, r))
        .cloned()
        .collect();
    let reviewers_block = cfg.phase == "post-review" && !missing_reviewers.is_empty();

    // policy. invalid tickets deduct; unverified tickets deduct AND, if no valid
    // ticket exists, hard-gate (authorization could not be established).
    let score = calculate_score(
        invalid.len() + unverified.len(),
        unspecced.len(),
        untested.len(),
        missing_reviewers.len(),
    );
    let mut hard_gates: Vec<String> = Vec::new();
    if valid.is_empty() {
        hard_gates.push(if tickets.is_empty() {
            "MANDATORY: no ticket referenced in PR title or description".into()
        } else if !invalid.is_empty() && unverified.is_empty() {
            "MANDATORY: all referenced tickets are invalid — no verified authorization".into()
        } else {
            format!(
                "MANDATORY: no ticket could be verified (unverified: {}) — authorization not established. Configure LINEAR_API_KEY, or reference a real GitHub issue",
                unverified.join(", ")
            )
        });
    }
    if !body_len_ok {
        hard_gates.push("MANDATORY: PR description is empty or too brief (min 20 chars)".into());
    }
    if findings_err {
        hard_gates.push(
            "MANDATORY: could not evaluate review findings (API error) — failing closed".into(),
        );
    }
    if !findings.is_empty() {
        hard_gates.push(format!(
            "MANDATORY: {} unresolved critical/major review finding(s)",
            findings.len()
        ));
    }
    if reviewers_block {
        hard_gates.push(format!(
            "MANDATORY: required reviewer(s) not posted: {}",
            missing_reviewers.join(", ")
        ));
    }
    // `compliant:exempt` waives the SCORE threshold only — it can NEVER waive a
    // hard gate (missing ticket, empty body, unresolved Critical/Major, missing
    // required reviewer). A label is not authorization to ship unsafe changes.
    let compliant = hard_gates.is_empty() && (exempt || score >= cfg.threshold);

    let report = json!({
        "compliant": compliant,
        "exempt": exempt,
        "phase": cfg.phase,
        "score": score,
        "threshold": cfg.threshold,
        "tickets": valid,
        "invalid_tickets": invalid,
        "unverified_tickets": unverified,
        "unspecced_changes": unspecced,
        "untested_files": untested,
        "missing_sections": missing_sections,
        "missing_reviewers": missing_reviewers,
        "unresolved_findings": findings,
        "hard_gates": hard_gates,
    });
    std::fs::write(
        "compliance_report.json",
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&report).unwrap());

    // PR comment
    let marker = format!("<!-- shadow-ci:{} -->", cfg.agent_key);
    let comment = render_comment(&marker, &cfg, &report);
    if let Err(e) = upsert_comment(&cfg, &marker, &comment) {
        eprintln!("warning: could not post PR comment: {e}");
    }

    if let Ok(out_path) = std::env::var("GITHUB_OUTPUT") {
        let _ = run(
            "bash",
            &["-c", &format!("echo 'compliant={compliant}' >> {out_path}")],
        );
    }
    Ok(if compliant { 0 } else { 1 })
}

fn render_comment(marker: &str, cfg: &Config, r: &Value) -> String {
    let ok = r["compliant"].as_bool().unwrap_or(false);
    let (icon, verdict) = if ok {
        ("✅", "Passed")
    } else {
        ("❌", "Failed")
    };
    let phase_name = if cfg.phase == "post-review" {
        "Review Gate"
    } else {
        "Audit"
    };
    let mut s = format!(
        "{marker}\n## {icon} Shadow Compliance — {phase_name}: {verdict} ({}%)\n\n",
        r["score"]
    );
    if r["exempt"].as_bool() == Some(true) {
        s.push_str("> `compliance:exempt` label present — checks recorded but not enforced.\n\n");
    }
    let list = |v: &Value| -> String {
        v.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    };
    s.push_str(&format!(
        "| Check | Result |\n|---|---|\n| Tickets | {} |\n| Invalid tickets | {} |\n| Unspecced changes | {} |\n| Untested files | {} |\n| Missing PR sections | {} |\n| Missing reviewers | {} |\n| Unresolved findings | {} |\n",
        if list(&r["tickets"]).is_empty() { "none".into() } else { list(&r["tickets"]) },
        or_dash(&list(&r["invalid_tickets"])),
        or_dash(&list(&r["unspecced_changes"])),
        or_dash(&list(&r["untested_files"])),
        or_dash(&list(&r["missing_sections"])),
        or_dash(&list(&r["missing_reviewers"])),
        r["unresolved_findings"].as_array().map(|a| a.len()).unwrap_or(0),
    ));
    if let Some(gates) = r["hard_gates"].as_array() {
        if !gates.is_empty() {
            s.push_str("\n**Hard gates:**\n");
            for g in gates {
                s.push_str(&format!("- {}\n", g.as_str().unwrap_or("")));
            }
        }
    }
    if !ok {
        s.push_str("\nFix with the `fix-compliance` command: every changed file must appear in the PR body's Changes section, every ticket must exist, changed source files need tests, and critical/major review findings must be resolved.\n");
    }
    let run_id = env_or("GITHUB_RUN_ID", "local");
    s.push_str(&format!(
        "\n<sub>shadow-ci · phase {} · run {run_id}</sub>\n",
        cfg.phase
    ));
    s
}

fn or_dash(s: &str) -> String {
    if s.is_empty() {
        "—".into()
    } else {
        s.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tickets_from_title_and_body_deduped_ordered() {
        let t = extract_tickets(
            "ABC-12: fix",
            "closes ABC-12, relates ABC-7",
            r"[A-Z]{2,6}-[0-9]+",
        );
        assert_eq!(t, vec!["ABC-12", "ABC-7"]);
        assert!(extract_tickets("no tickets here", "", r"[A-Z]{2,6}-[0-9]+").is_empty());
        // the default pattern accepts both tracker styles
        let both = extract_tickets(
            "#42: fix login",
            "relates ABC-7 and #42",
            r"[A-Z]{2,6}-[0-9]+|#[0-9]+",
        );
        assert_eq!(both, vec!["#42", "ABC-7"]);
    }

    #[test]
    fn traceability_matches_path_or_basename() {
        let body = "## Changes\n- src/auth/login.rs — new login flow".to_lowercase();
        assert!(is_traceable("src/auth/login.rs", &body));
        assert!(is_traceable("deep/other/login.rs", &body)); // basename match
        assert!(!is_traceable("src/db/pool.rs", &body));
        assert!(is_exempt_minor("Cargo.lock"));
        assert!(!is_exempt_minor("src/lock.rs"));
    }

    #[test]
    fn test_detection() {
        assert!(is_source_file("src/gauge.rs"));
        assert!(!is_source_file("README.md"));
        assert!(is_test_file("tests/gauge_test.rs"));
        assert!(is_test_file("src/foo.spec.ts"));
        let candidates = vec![
            "tests/test_gauge.rs".to_string(),
            "src/gauge.rs".to_string(),
        ];
        assert!(has_test_for("src/gauge.rs", &candidates));
        assert!(!has_test_for("src/needle.rs", &candidates));
    }

    #[test]
    fn shadow_reviewer_presence_requires_bot_author() {
        let by_bot = serde_json::json!([{"user":{"login":"github-actions[bot]"},"body":"<!-- shadow-review:complete -->\nno blocking findings"}]);
        let unavailable = serde_json::json!([{"user":{"login":"github-actions[bot]"},"body":"<!-- shadow-review:unavailable -->\nreview did not run"}]);
        let legacy = serde_json::json!([{"user":{"login":"github-actions[bot]"},"body":"<!-- shadow-review -->\nambiguous legacy marker"}]);
        let by_author = serde_json::json!([{"user":{"login":"mallory"},"body":"<!-- shadow-review:complete -->\nfaking it"}]);
        let other_comment = serde_json::json!([{"user":{"login":"github-actions[bot]"},"body":"<!-- shadow-ci:audit -->\ncompliance report"}]);
        assert!(shadow_review_posted_by_bot(&by_bot, "mallory"));
        assert!(!shadow_review_posted_by_bot(&unavailable, "mallory"));
        assert!(!shadow_review_posted_by_bot(&legacy, "mallory"));
        assert!(!shadow_review_posted_by_bot(&by_author, "mallory")); // author cannot paste the marker
        assert!(!shadow_review_posted_by_bot(&other_comment, "mallory")); // wrong marker
        assert!(shadow_review_posted(&by_bot));
        assert!(!shadow_review_posted(&by_author)); // non-bot marker doesn't count
        assert_eq!(bot_login_for("shadow-reviewer"), "github-actions");
    }

    #[test]
    fn severity_classification() {
        assert_eq!(severity_of("Critical: SQL injection"), Some("critical"));
        assert_eq!(
            severity_of("this is a Potential issue with retries"),
            Some("major")
        );
        assert_eq!(severity_of("nitpick: rename this"), None);
    }

    #[test]
    fn scoring_matches_policy() {
        assert_eq!(calculate_score(0, 0, 0, 0), 100);
        assert_eq!(calculate_score(1, 2, 3, 1), 100 - 10 - 20 - 15 - 5);
        assert_eq!(calculate_score(9, 9, 9, 9), 0); // clamped
    }
}
