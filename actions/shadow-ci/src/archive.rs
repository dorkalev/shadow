// shadow-ci archive — the post-merge evidence record.
//
// On every merged PR: assemble a complete JSON+MD archive (PR metadata, reviews,
// comments, check runs, files, commits, the shadow-ci compliance comment) plus a
// BYPASS ANALYSIS — required status checks are read from the *live* branch ruleset,
// and any merge that landed with a required check failed/missing is flagged
// `is_bypass: true`. Records are committed to the append-only `compliance-archives`
// branch. Bypasses are never forbidden — only detected, recorded, and announced.
use crate::util::{commit_to_archives, curl_post, env_or, gh_json, utc_date};
use serde_json::{json, Value};

/// Outcome classification for one required-or-observed check:
/// failure/cancelled/missing on a required check means the merge bypassed it.
pub fn classify(conclusion: Option<&str>, required: bool) -> &'static str {
    match conclusion {
        Some("success") | Some("skipped") | Some("neutral") => "passed",
        _ if required => "bypassed", // failure, cancelled, timed_out, action_required, or never ran
        _ => "informational",
    }
}

pub fn analyze_bypass(
    check_runs: &[(String, Option<String>)], // (name, conclusion)
    statuses: &[(String, Option<String>)],   // (context, state-as-conclusion)
    required: &[String],
) -> (bool, Vec<String>, Vec<String>, Vec<String>) {
    let lookup = |name: &str| -> Option<Option<String>> {
        check_runs
            .iter()
            .chain(statuses.iter())
            .find(|(n, _)| n == name)
            .map(|(_, c)| c.clone())
    };
    let mut passed = Vec::new();
    let mut bypassed = Vec::new();
    let mut informational = Vec::new();

    for req in required {
        match lookup(req) {
            Some(conclusion) => match classify(conclusion.as_deref(), true) {
                "passed" => passed.push(req.clone()),
                _ => bypassed.push(format!("{req} ({})", conclusion.as_deref().unwrap_or("no conclusion"))),
            },
            None => bypassed.push(format!("{req} (never ran)")),
        }
    }
    for (name, conclusion) in check_runs.iter().chain(statuses.iter()) {
        if required.contains(name) {
            continue;
        }
        if classify(conclusion.as_deref(), false) == "informational" {
            informational.push(format!("{name} ({})", conclusion.as_deref().unwrap_or("none")));
        }
    }
    (!bypassed.is_empty(), passed, bypassed, informational)
}

/// Returns Ok(contexts) when the required set is KNOWN, Err when it could not be
/// determined. The caller must fail-closed on Err — treating an unreadable
/// ruleset as "nothing required" would silently disable bypass detection exactly
/// when it matters most.
fn required_contexts(repo: &str, base: &str) -> Result<Vec<String>, String> {
    if let Ok(over) = std::env::var("REQUIRED_CHECKS_OVERRIDE") {
        if !over.is_empty() {
            return Ok(over.split(',').map(|s| s.trim().to_string()).collect());
        }
    }
    // live ruleset so the archive can't drift from what GitHub actually enforces
    let v = gh_json(&["api", &format!("repos/{repo}/rules/branches/{base}")])
        .map_err(|e| format!("ruleset unreadable: {e}"))?;
    let arr = v.as_array().ok_or("ruleset response was not an array")?;
    Ok(arr
        .iter()
        .filter(|r| r["type"].as_str() == Some("required_status_checks"))
        .flat_map(|r| {
            r["parameters"]["required_status_checks"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|c| c["context"].as_str().map(String::from))
        .collect())
}

pub fn run_archive() -> Result<i32, String> {
    let repo = std::env::var("REPO")
        .or_else(|_| std::env::var("GITHUB_REPOSITORY"))
        .map_err(|_| "REPO or GITHUB_REPOSITORY required")?;
    let pr_number = std::env::var("PR_NUMBER").map_err(|_| "PR_NUMBER required")?;

    let pr = gh_json(&[
        "pr", "view", &pr_number, "--repo", &repo,
        "--json",
        "number,title,url,author,body,labels,createdAt,mergedAt,mergedBy,headRefName,baseRefName,headRefOid,additions,deletions,files,commits,reviews",
    ])?;
    if pr["mergedAt"].is_null() {
        println!("PR #{pr_number} not merged; nothing to archive");
        return Ok(0);
    }
    let sha = pr["headRefOid"].as_str().unwrap_or("");
    let base = pr["baseRefName"].as_str().unwrap_or("staging");

    let comments = gh_json(&["api", &format!("repos/{repo}/issues/{pr_number}/comments"), "--paginate"])
        .unwrap_or(Value::Array(vec![]));
    let review_comments = gh_json(&["api", &format!("repos/{repo}/pulls/{pr_number}/comments"), "--paginate"])
        .unwrap_or(Value::Array(vec![]));
    let check_runs_raw = gh_json(&[
        "api", &format!("repos/{repo}/commits/{sha}/check-runs"), "--paginate", "--jq", ".check_runs",
    ])
    .unwrap_or(Value::Array(vec![]));
    let statuses_raw = gh_json(&["api", &format!("repos/{repo}/commits/{sha}/status")])
        .unwrap_or(json!({"statuses": []}));

    let check_runs: Vec<(String, Option<String>)> = check_runs_raw
        .as_array()
        .map(|a| {
            a.iter()
                .map(|c| (
                    c["name"].as_str().unwrap_or("").to_string(),
                    c["conclusion"].as_str().map(String::from),
                ))
                .collect()
        })
        .unwrap_or_default();
    let statuses: Vec<(String, Option<String>)> = statuses_raw["statuses"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|s| (
                    s["context"].as_str().unwrap_or("").to_string(),
                    s["state"].as_str().map(|st| if st == "success" { "success".into() } else { st.to_string() }),
                ))
                .collect()
        })
        .unwrap_or_default();

    // Fail-closed: if the required set is unknown, we cannot certify "not a bypass".
    // Record it as an unknown requiring manual review, not a clean merge.
    let (required, required_known) = match required_contexts(&repo, base) {
        Ok(r) => (r, true),
        Err(e) => {
            eprintln!("bypass detection degraded: {e}");
            (Vec::new(), false)
        }
    };
    let (bypass_detected, passed, bypassed, informational) = analyze_bypass(&check_runs, &statuses, &required);
    // unknown required set ⇒ treat as bypass-suspect (true), not clean
    let is_bypass = bypass_detected || !required_known;

    // last shadow-ci compliance comment, embedded verbatim
    let compliance_comment = comments
        .as_array()
        .and_then(|a| {
            a.iter()
                .filter(|c| c["body"].as_str().unwrap_or("").contains("shadow-ci:"))
                .last()
        })
        .and_then(|c| c["body"].as_str())
        .unwrap_or("")
        .to_string();

    let ticket_pattern = env_or("TICKET_PATTERN", r"[A-Z]{2,6}-[0-9]+|#[0-9]+");
    let tickets = crate::check::extract_tickets(
        pr["title"].as_str().unwrap_or(""),
        pr["body"].as_str().unwrap_or(""),
        &ticket_pattern,
    );
    let first_ticket = tickets.first().map(String::as_str).unwrap_or("noticket").to_lowercase();
    let date = utc_date("%Y%m%d");
    let archive_id = format!("pr-{pr_number}-{first_ticket}-{date}");

    let record = json!({
        "archive_version": "3.0",
        "archived_at": utc_date("%Y-%m-%dT%H:%M:%SZ"),
        "archive_id": archive_id,
        "pr": pr,
        "tickets": tickets,
        "bypass_merge": {
            "is_bypass": is_bypass,
            "required_checks_known": required_known,
            "required_checks": required,
            "passed": passed,
            "bypassed": bypassed,
            "informational": informational,
        },
        "compliance_comment": compliance_comment,
        "issue_comments": comments,
        "review_comments": review_comments,
        "check_runs": check_runs_raw,
        "statuses": statuses_raw["statuses"],
    });

    let md = render_md(&record);

    let branch = env_or("ARCHIVES_BRANCH", "compliance-archives");
    commit_to_archives(
        &branch,
        &[
            (format!("{archive_id}.json"), serde_json::to_string_pretty(&record).unwrap()),
            (format!("{archive_id}.md"), md.clone()),
        ],
        &format!("archive: {archive_id}"),
    )?;

    if let Ok(hook) = std::env::var("SLACK_WEBHOOK_URL") {
        if !hook.is_empty() {
            let title = pr["title"].as_str().unwrap_or("");
            let url = pr["url"].as_str().unwrap_or("");
            let head = if is_bypass {
                format!(":rotating_light: MANUAL MERGE — CHECKS BYPASSED\n#{pr_number} {title}")
            } else {
                format!(":white_check_mark: #{pr_number} merged to {base}\n{title}")
            };
            let _ = curl_post(&hook, &[("Content-Type", "application/json")],
                &json!({"text": format!("{head}\n{url} · archive {archive_id}")}).to_string());
        }
    }

    println!("archived {archive_id} (bypass: {is_bypass})");
    Ok(0)
}

fn render_md(r: &Value) -> String {
    let pr = &r["pr"];
    let b = &r["bypass_merge"];
    let mut s = format!(
        "# Compliance Archive: PR #{}\n\n> {}\n> Archived at: {}\n\n",
        pr["number"], pr["title"].as_str().unwrap_or(""), r["archived_at"].as_str().unwrap_or("")
    );
    if b["is_bypass"].as_bool() == Some(true) {
        s.push_str("## 🚨 MANUAL MERGE — REQUIRED CHECKS BYPASSED\n\n");
        for x in b["bypassed"].as_array().unwrap_or(&vec![]) {
            s.push_str(&format!("- {}\n", x.as_str().unwrap_or("")));
        }
        s.push_str("\nAn incident ticket + backport PR are required (hotfix procedure).\n\n");
    }
    s.push_str(&format!(
        "## Overview\n\n| Field | Value |\n|---|---|\n| PR | [#{}]({}) |\n| Author | {} |\n| Tickets | {} |\n| Base | {} |\n| Merged by | {} |\n| Merged at | {} |\n| +/− | +{} −{} |\n| Bypass | {} |\n\n",
        pr["number"], pr["url"].as_str().unwrap_or(""),
        pr["author"]["login"].as_str().unwrap_or(""),
        r["tickets"].as_array().map(|a| a.iter().filter_map(|t| t.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default(),
        pr["baseRefName"].as_str().unwrap_or(""),
        pr["mergedBy"]["login"].as_str().unwrap_or(""),
        pr["mergedAt"].as_str().unwrap_or(""),
        pr["additions"], pr["deletions"],
        if b["is_bypass"].as_bool() == Some(true) { "YES" } else { "no" },
    ));
    s.push_str("## Required checks\n\n| Check | Outcome |\n|---|---|\n");
    for x in b["passed"].as_array().unwrap_or(&vec![]) {
        s.push_str(&format!("| {} | ✅ passed |\n", x.as_str().unwrap_or("")));
    }
    for x in b["bypassed"].as_array().unwrap_or(&vec![]) {
        s.push_str(&format!("| {} | 🚨 bypassed |\n", x.as_str().unwrap_or("")));
    }
    s.push_str("\n## Files changed\n\n");
    for f in pr["files"].as_array().unwrap_or(&vec![]) {
        s.push_str(&format!("- {}\n", f["path"].as_str().unwrap_or("")));
    }
    let cc = r["compliance_comment"].as_str().unwrap_or("");
    if !cc.is_empty() {
        s.push_str("\n## Compliance report (as posted on the PR)\n\n");
        s.push_str(cc);
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> String {
        v.to_string()
    }

    #[test]
    fn classify_outcomes() {
        assert_eq!(classify(Some("success"), true), "passed");
        assert_eq!(classify(Some("skipped"), true), "passed");
        assert_eq!(classify(Some("failure"), true), "bypassed");
        assert_eq!(classify(None, true), "bypassed");
        assert_eq!(classify(Some("failure"), false), "informational");
    }

    #[test]
    fn bypass_detection() {
        let runs = vec![(s("ci"), Some(s("success"))), (s("compliance-audit"), Some(s("failure")))];
        let statuses = vec![(s("review-bot"), Some(s("success")))];
        let required = vec![s("ci"), s("compliance-audit"), s("review-bot"), s("never-ran")];
        let (bypass, passed, bypassed, _) = analyze_bypass(&runs, &statuses, &required);
        assert!(bypass);
        assert_eq!(passed, vec![s("ci"), s("review-bot")]);
        assert_eq!(bypassed, vec![s("compliance-audit (failure)"), s("never-ran (never ran)")]);
    }

    #[test]
    fn clean_merge_is_not_bypass() {
        let runs = vec![(s("ci"), Some(s("success")))];
        let (bypass, _, bypassed, _) = analyze_bypass(&runs, &[], &[s("ci")]);
        assert!(!bypass);
        assert!(bypassed.is_empty());
    }
}
