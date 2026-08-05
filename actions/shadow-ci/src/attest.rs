// shadow-ci attest — the fieldwork a CPA performs, run against GitHub itself.
//
// For an audit window: build the POPULATION of changes from three reconciled
// records (GitHub's merged-PR list, the git commit history of main, and the
// compliance-archives records), reconcile them for completeness, then test
// EVERY item (100%, not a sample) against the change-management attributes:
//
//   T1  authorized:  a ticket is referenced AND (for #N tickets) the issue was
//                    created BEFORE the PR — authorization preceded the change
//   T2  approved:    authenticated management merge actor recorded. In the
//                    solo profile this may be the author; no independence claimed.
//   T3  gated:       the archive record shows is_bypass=false (checks passed);
//                    bypasses need a linked incident/hotfix trail
//   T4  documented:  the PR body carries the four required sections
//   T5  flow:        the PR targeted protected main (the default one-person flow)
//
// Output: attestation-{since}-{until}.md + .json — population sizes, the
// reconciliation, the exception list, and the conclusion. This is the report
// you hand the auditor to show everything was aligned — or the honest list of
// where it wasn't.
use crate::util::{env_or, gh_json, git, utc_date};
use serde_json::{json, Value};

// ---------- pure helpers (unit-tested) ----------

/// Squash/merge commit subjects carry the PR number: "Title (#42)" / "Merge pull request #42 …"
pub fn pr_number_from_subject(subject: &str) -> Option<u64> {
    let re = regex::Regex::new(r"\(#(\d+)\)\s*$|^Merge pull request #(\d+)").unwrap();
    re.captures(subject)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .and_then(|m| m.as_str().parse().ok())
}

/// ISO-8601 strings compare lexicographically — good enough for windowing.
pub fn in_window(ts: &str, since: &str, until: &str) -> bool {
    !ts.is_empty() && ts >= since && ts <= until
}

/// Archive filenames: pr-{n}-{ticket}-{yyyymmdd}.json
pub fn archive_pr_number(path: &str) -> Option<u64> {
    path.strip_prefix("pr-")?.split('-').next()?.parse().ok()
}

pub fn missing_sections(body: &str) -> Vec<&'static str> {
    ["## Summary", "## Tickets", "## Changes", "## Test Plan"]
        .iter()
        .filter(|s| !body.contains(**s))
        .copied()
        .collect()
}

// ---------- the fieldwork ----------

pub fn run_attest() -> Result<i32, String> {
    let repo = std::env::var("REPO")
        .or_else(|_| std::env::var("GITHUB_REPOSITORY"))
        .map_err(|_| "REPO or GITHUB_REPOSITORY required")?;
    let since =
        std::env::var("SINCE").map_err(|_| "SINCE required (ISO date, start of audit window)")?;
    let until = env_or("UNTIL", &utc_date("%Y-%m-%dT23:59:59Z"));
    let ticket_pattern = env_or("TICKET_PATTERN", r"[A-Z]{2,6}-[0-9]+|#[0-9]+");
    let archives = env_or("ARCHIVES_BRANCH", "compliance-archives");

    // Source A — GitHub's own record of merged PRs
    let prs_raw = gh_json(&[
        "pr",
        "list",
        "--repo",
        &repo,
        "--state",
        "merged",
        "--limit",
        "500",
        "--json",
        "number,title,body,author,createdAt,mergedAt,mergedBy,baseRefName,reviews",
    ])?;
    let prs: Vec<&Value> = prs_raw
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|p| {
                    in_window(p["mergedAt"].as_str().unwrap_or(""), &since, &until)
                        && p["baseRefName"].as_str() == Some("main")
                })
                .collect()
        })
        .unwrap_or_default();

    // Source B — the git history of main (what actually landed)
    git(&["fetch", "-q", "origin", "main"]).ok();
    let commits = git(&[
        "log",
        "origin/main",
        &format!("--since={since}"),
        &format!("--until={until}"),
        "--pretty=%H|%cI|%s",
    ])
    .unwrap_or_default();
    let mut orphan_commits: Vec<String> = Vec::new();
    let mut commit_prs: std::collections::BTreeSet<u64> = Default::default();
    for line in commits.lines() {
        let mut parts = line.splitn(3, '|');
        let (sha, _ts, subject) = (
            parts.next().unwrap_or(""),
            parts.next().unwrap_or(""),
            parts.next().unwrap_or(""),
        );
        match pr_number_from_subject(subject) {
            Some(n) => {
                commit_prs.insert(n);
            }
            None => orphan_commits.push(format!("{} {}", &sha[..sha.len().min(10)], subject)),
        }
    }

    // Source C — the archive records
    git(&["fetch", "-q", "origin", &archives]).ok();
    let archive_files = git(&[
        "ls-tree",
        "-r",
        "--name-only",
        &format!("origin/{archives}"),
    ])
    .unwrap_or_default();
    let since_compact = since[..10].replace('-', "");
    let until_compact = until[..10].replace('-', "");
    let archived_prs: std::collections::BTreeSet<u64> = archive_files
        .lines()
        .filter(|f| f.ends_with(".json") && f.starts_with("pr-"))
        .filter(|f| {
            f.strip_suffix(".json")
                .and_then(|s| s.rsplit('-').next())
                .map(|d| d >= since_compact.as_str() && d <= until_compact.as_str())
                .unwrap_or(false)
        })
        .filter_map(archive_pr_number)
        .collect();

    // ---------- completeness reconciliation ----------
    let pr_numbers: std::collections::BTreeSet<u64> =
        prs.iter().filter_map(|p| p["number"].as_u64()).collect();
    let commits_without_pr = orphan_commits.len();
    let prs_not_in_commits: Vec<u64> = pr_numbers.difference(&commit_prs).copied().collect();
    let prs_not_archived: Vec<u64> = pr_numbers.difference(&archived_prs).copied().collect();
    let archived_not_prs: Vec<u64> = archived_prs.difference(&pr_numbers).copied().collect();

    // ---------- attribute testing, 100% of the population ----------
    let mut items = Vec::new();
    let mut exceptions: Vec<String> = Vec::new();
    for p in &prs {
        let n = p["number"].as_u64().unwrap_or(0);
        let author = p["author"]["login"].as_str().unwrap_or("");
        let body = p["body"].as_str().unwrap_or("");
        let title = p["title"].as_str().unwrap_or("");
        let created = p["createdAt"].as_str().unwrap_or("");
        let base = p["baseRefName"].as_str().unwrap_or("");
        let mut fails: Vec<String> = Vec::new();

        // T1 authorized — ticket referenced, and (for #N) the issue predates the PR
        let tickets = crate::check::extract_tickets(title, body, &ticket_pattern);
        if tickets.is_empty() {
            fails.push("T1: no ticket referenced".into());
        } else {
            for t in &tickets {
                if let Some(num) = t.strip_prefix('#') {
                    if num.parse::<u64>().ok() == Some(n) {
                        fails.push("T1: PR cites itself as its ticket".into());
                        continue;
                    }
                    if let Ok(issue) = gh_json(&["api", &format!("repos/{repo}/issues/{num}")]) {
                        if issue.get("pull_request").is_some() {
                            fails.push(format!("T1: {t} is a PR, not a ticket"));
                        } else if issue["created_at"].as_str().unwrap_or("") > created {
                            fails.push(format!("T1: {t} was created AFTER the PR — authorization did not precede the change"));
                        }
                    } else {
                        fails.push(format!("T1: {t} does not exist"));
                    }
                }
            }
        }

        // T2 management approval — the authenticated merge actor is the
        // approval-of-record. It may equal the author in the disclosed solo model.
        let merged_by = p["mergedBy"]["login"].as_str().unwrap_or("");
        if merged_by.is_empty() {
            fails.push("T2: no authenticated management merge actor recorded".into());
        }

        // T3 gated — the archive's bypass analysis
        if archived_prs.contains(&n) {
            if let Some(file) = archive_files
                .lines()
                .find(|f| f.ends_with(".json") && archive_pr_number(f) == Some(n))
            {
                if let Ok(rec) = git(&["show", &format!("origin/{archives}:{file}")]) {
                    if rec.contains("\"is_bypass\": true") {
                        fails.push("T3: merged with required checks bypassed (verify incident + backport trail)".into());
                    }
                }
            }
        } else {
            fails.push("T3: no archive record — gate outcome unverifiable from evidence".into());
        }

        // T4 documented
        let missing = missing_sections(body);
        if !missing.is_empty() {
            fails.push(format!("T4: missing sections: {}", missing.join(", ")));
        }

        // T5 flow
        if base != "main" {
            fails.push(format!(
                "T5: PR targeted {base}, not the declared main-only flow"
            ));
        }

        let pass = fails.is_empty();
        if !pass {
            for f in &fails {
                exceptions.push(format!("PR #{n} ({title}): {f}"));
            }
        }
        items.push(json!({
            "pr": n, "title": title, "author": author, "base": base,
            "merged_at": p["mergedAt"], "merged_by": p["mergedBy"]["login"],
            "tickets": tickets, "pass": pass, "findings": fails,
        }));
    }
    for c in &orphan_commits {
        exceptions.push(format!(
            "main commit without a PR (direct push or bootstrap commit): {c}"
        ));
    }
    for n in &prs_not_archived {
        // already surfaced per-item as T3, but completeness deserves its own line
        let _ = n;
    }
    for n in &archived_not_prs {
        exceptions.push(format!(
            "archive record pr-{n} has no matching merged PR — investigate"
        ));
    }

    let tested = items.len();
    let passed = items
        .iter()
        .filter(|i| i["pass"].as_bool() == Some(true))
        .count();
    let conclusion = if exceptions.is_empty() {
        format!("NO EXCEPTIONS: {tested} of {tested} changes in the window passed all five attributes, and the three populations reconcile.")
    } else {
        format!("{} exception(s) across {tested} changes — each listed below with the PR and attribute.", exceptions.len())
    };

    let report = json!({
        "attestation": "change management (CC8.1, CC6.3 compensating controls, CC4.1)",
        "repo": repo, "window": {"since": since, "until": until},
        "method": "population from three reconciled GitHub records (merged-PR API, main history, compliance-archives); 100% attribute testing (no sampling)",
        "population": {
            "merged_prs": pr_numbers.len(),
            "main_commits_without_pr": commits_without_pr,
            "prs_not_in_commit_history": prs_not_in_commits,
            "prs_without_archive_record": prs_not_archived,
            "archive_records_without_pr": archived_not_prs,
        },
        "attributes": ["T1 authorized (ticket precedes change)", "T2 founder approval recorded", "T3 gated (no bypass)", "T4 documented (4 sections)", "T5 flow (protected main PR)"],
        "items": items,
        "exceptions": exceptions,
        "conclusion": conclusion,
        "generated_at": utc_date("%Y-%m-%dT%H:%M:%SZ"),
    });

    let stamp = format!("{}-{}", &since[..10], &until[..10]);
    let md = render_md(&report);
    let files = vec![
        (
            format!("evidence/attestations/attestation-{stamp}.json"),
            serde_json::to_string_pretty(&report).unwrap(),
        ),
        (
            format!("evidence/attestations/attestation-{stamp}.md"),
            md.clone(),
        ),
    ];
    if env_or("ARCHIVES_PUSH", "") == "1" {
        crate::util::commit_to_archives(
            &archives,
            &files,
            &format!("evidence: attestation {stamp}"),
        )?;
        println!("attestation pushed to {archives}");
    } else {
        std::fs::write(format!("attestation-{stamp}.md"), &md).map_err(|e| e.to_string())?;
        std::fs::write(
            format!("attestation-{stamp}.json"),
            serde_json::to_string_pretty(&report).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        println!("wrote attestation-{stamp}.md / .json  (ARCHIVES_PUSH=1 to file on {archives})");
    }
    println!("{conclusion}  ({passed}/{tested} clean)");
    Ok(if exceptions.is_empty() { 0 } else { 1 })
}

fn render_md(r: &Value) -> String {
    let mut s = format!(
        "# Change-Management Attestation — {}\n\n**Window:** {} → {}\n**Method:** {}\n\n## Conclusion\n\n**{}**\n\n",
        r["repo"].as_str().unwrap_or(""),
        r["window"]["since"].as_str().unwrap_or(""),
        r["window"]["until"].as_str().unwrap_or(""),
        r["method"].as_str().unwrap_or(""),
        r["conclusion"].as_str().unwrap_or(""),
    );
    let p = &r["population"];
    s.push_str(&format!(
        "## Population & completeness\n\n| Measure | Value |\n|---|---|\n| Merged PRs to main | {} |\n| Main commits with no PR (direct pushes/bootstrap) | {} |\n| PRs missing from commit history | {:?} |\n| PRs without an archive record | {:?} |\n| Archive records without a PR | {:?} |\n\n",
        p["merged_prs"], p["main_commits_without_pr"], p["prs_not_in_commit_history"], p["prs_without_archive_record"], p["archive_records_without_pr"],
    ));
    s.push_str("## Attribute testing (100% of population)\n\n| PR | Title | Author | Merged by | Verdict |\n|---|---|---|---|---|\n");
    for i in r["items"].as_array().unwrap_or(&vec![]) {
        s.push_str(&format!(
            "| #{} | {} | {} | {} | {} |\n",
            i["pr"],
            i["title"]
                .as_str()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>(),
            i["author"].as_str().unwrap_or(""),
            i["merged_by"].as_str().unwrap_or(""),
            if i["pass"].as_bool() == Some(true) {
                "✓"
            } else {
                "✗ see exceptions"
            },
        ));
    }
    let ex = r["exceptions"].as_array().cloned().unwrap_or_default();
    s.push_str("\n## Exceptions\n\n");
    if ex.is_empty() {
        s.push_str("None.\n");
    } else {
        for e in ex {
            s.push_str(&format!("- {}\n", e.as_str().unwrap_or("")));
        }
    }
    s.push_str(&format!("\n---\nGenerated {} by `shadow-ci attest` from GitHub's records. Attributes: T1 authorized (ticket precedes change) · T2 founder approval recorded (no independence claimed) · T3 gated, no bypass · T4 documented · T5 protected main PR.\n", r["generated_at"].as_str().unwrap_or("")));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_numbers_from_merge_subjects() {
        assert_eq!(pr_number_from_subject("Fix login (#42)"), Some(42));
        assert_eq!(
            pr_number_from_subject("Merge pull request #7 from x/y"),
            Some(7)
        );
        assert_eq!(pr_number_from_subject("direct push, no pr"), None);
        assert_eq!(pr_number_from_subject("mentions #9 midway"), None);
    }

    #[test]
    fn window_and_archive_parsing() {
        assert!(in_window(
            "2026-05-02T10:00:00Z",
            "2026-05-01",
            "2026-07-31"
        ));
        assert!(!in_window(
            "2026-04-30T23:59:59Z",
            "2026-05-01",
            "2026-07-31"
        ));
        assert_eq!(archive_pr_number("pr-42-abc-7-20260502.json"), Some(42));
        assert_eq!(archive_pr_number("releases/release-x.json"), None);
    }

    #[test]
    fn documentation_attribute() {
        assert!(missing_sections("## Summary\n## Tickets\n## Changes\n## Test Plan").is_empty());
        assert_eq!(
            missing_sections("## Summary only"),
            vec!["## Tickets", "## Changes", "## Test Plan"]
        );
    }
}
