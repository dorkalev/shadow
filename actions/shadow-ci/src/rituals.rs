// shadow-ci rituals — automation for the Clock's mundane human work.
//
//   access-review   assemble the quarterly access-review packet from live systems
//                   (GitHub org/repos/keys + GCP IAM/service accounts). The human's
//                   job shrinks to reading a diff and signing. CC6.2 / CC6.3.
//                   With USER_FILTER=<login-or-email>: a per-person grant report —
//                   run before and after offboarding to prove revocation.
//   mgmt-packet     assemble the quarterly management-review packet: gauge trend,
//                   bypasses, open security alerts, incidents. Pre-filled agenda +
//                   minutes template. CC1.2 / CC4.2.
//   release-record  write the release record (commits, PRs, tickets, stats between
//                   origin/main and origin/staging) to compliance-archives. CC8.1.
//
// Output: local files by default; ARCHIVES_PUSH=1 commits them to the
// compliance-archives branch under evidence/{year}/{quarter}/ (or releases/).
use crate::util::{commit_to_archives, env_or, gh, gh_json, git, run, utc_date};
use serde_json::Value;

/// (year, quarter-label, quarter-start-date) from a "YYYY-MM" string.
pub fn quarter_of(year_month: &str) -> (String, String, String) {
    let (y, m) = year_month.split_once('-').unwrap_or(("1970", "01"));
    let month: u32 = m.parse().unwrap_or(1);
    let q = (month - 1) / 3 + 1;
    let start_month = (q - 1) * 3 + 1;
    (y.to_string(), format!("Q{q}"), format!("{y}-{start_month:02}-01"))
}

fn quarter_now() -> (String, String, String) {
    quarter_of(utc_date("%Y-%m").trim())
}

fn emit(kind: &str, files: Vec<(String, String)>) -> Result<(), String> {
    if env_or("ARCHIVES_PUSH", "") == "1" {
        let branch = env_or("ARCHIVES_BRANCH", "compliance-archives");
        commit_to_archives(&branch, &files, &format!("evidence: {kind}"))?;
        println!("pushed {} file(s) to {branch}", files.len());
    } else {
        for (path, content) in &files {
            let local = path.rsplit('/').next().unwrap_or(path);
            std::fs::write(local, content).map_err(|e| e.to_string())?;
            println!("wrote {local}  (set ARCHIVES_PUSH=1 to commit to the archives branch)");
        }
    }
    Ok(())
}

fn section_or_err<F: FnOnce() -> Result<String, String>>(title: &str, f: F) -> String {
    match f() {
        Ok(body) if body.trim().is_empty() => format!("### {title}\n\n_none_\n\n"),
        Ok(body) => format!("### {title}\n\n{body}\n"),
        Err(e) => format!("### {title}\n\n_unavailable: {}_\n\n", e.lines().next().unwrap_or("error")),
    }
}

fn matches_filter(row: &str, filter: &Option<String>) -> bool {
    match filter {
        None => true,
        Some(f) => row.to_lowercase().contains(&f.to_lowercase()),
    }
}

// ---------- access-review ----------

pub fn run_access_review() -> Result<i32, String> {
    let org = std::env::var("ORG").or_else(|_| {
        std::env::var("GITHUB_REPOSITORY").map(|r| r.split('/').next().unwrap_or("").to_string())
    })
    .map_err(|_| "ORG or GITHUB_REPOSITORY required")?;
    let filter = std::env::var("USER_FILTER").ok().filter(|s| !s.is_empty());
    let gcp_projects: Vec<String> = env_or("GCP_PROJECTS", "")
        .split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let (year, q, _) = quarter_now();
    let date = utc_date("%Y-%m-%d");

    let mut md = format!(
        "# Access Review — {org} — {year} {q}\n\nGenerated {date} by `shadow-ci access-review` from live system state.{}\nCriteria: CC6.2, CC6.3. The reviewer's job: confirm every grant below is still appropriate, list revocations, sign.\n\n",
        filter.as_deref().map(|f| format!(" **Filtered to user: {f}** (offboarding evidence)")).unwrap_or_default()
    );

    // GitHub org membership
    md.push_str(&section_or_err("GitHub org admins", || {
        let v = gh_json(&["api", &format!("orgs/{org}/members?role=admin"), "--paginate"])?;
        Ok(logins_table(&v, &filter))
    }));
    md.push_str(&section_or_err("GitHub org members", || {
        let v = gh_json(&["api", &format!("orgs/{org}/members?role=member"), "--paginate"])?;
        Ok(logins_table(&v, &filter))
    }));
    md.push_str(&section_or_err("Outside collaborators", || {
        let v = gh_json(&["api", &format!("orgs/{org}/outside_collaborators"), "--paginate"])?;
        Ok(logins_table(&v, &filter))
    }));

    // per-repo direct collaborators + deploy keys
    md.push_str(&section_or_err("Repo-level direct grants & deploy keys", || {
        let repos = gh_json(&["repo", "list", &org, "--limit", "200", "--json", "name"])?;
        let mut out = String::from("| Repo | Grant | Detail |\n|---|---|---|\n");
        for r in repos.as_array().unwrap_or(&vec![]) {
            let name = r["name"].as_str().unwrap_or("");
            if let Ok(collabs) = gh_json(&["api", &format!("repos/{org}/{name}/collaborators?affiliation=direct"), "--paginate"]) {
                for c in collabs.as_array().unwrap_or(&vec![]) {
                    let row = format!("| {name} | collaborator: {} | {} |",
                        c["login"].as_str().unwrap_or(""), c["role_name"].as_str().unwrap_or(""));
                    if matches_filter(&row, &filter) { out.push_str(&row); out.push('\n'); }
                }
            }
            if let Ok(keys) = gh_json(&["api", &format!("repos/{org}/{name}/keys")]) {
                for k in keys.as_array().unwrap_or(&vec![]) {
                    let row = format!("| {name} | deploy key: {} | read_only: {} |",
                        k["title"].as_str().unwrap_or(""), k["read_only"]);
                    if matches_filter(&row, &filter) { out.push_str(&row); out.push('\n'); }
                }
            }
        }
        Ok(out)
    }));

    // GCP IAM + service-account keys
    for project in &gcp_projects {
        md.push_str(&section_or_err(&format!("GCP IAM — {project}"), || {
            let out = run("gcloud", &["projects", "get-iam-policy", project, "--format=json"])?;
            let v: Value = serde_json::from_str(&out).map_err(|e| e.to_string())?;
            let mut t = String::from("| Member | Role |\n|---|---|\n");
            for b in v["bindings"].as_array().unwrap_or(&vec![]) {
                let role = b["role"].as_str().unwrap_or("");
                for m in b["members"].as_array().unwrap_or(&vec![]) {
                    let row = format!("| {} | {role} |", m.as_str().unwrap_or(""));
                    if matches_filter(&row, &filter) { t.push_str(&row); t.push('\n'); }
                }
            }
            Ok(t)
        }));
        md.push_str(&section_or_err(&format!("GCP user-managed service-account keys — {project}"), || {
            let out = run("gcloud", &["iam", "service-accounts", "list", "--project", project, "--format=json"])?;
            let v: Value = serde_json::from_str(&out).map_err(|e| e.to_string())?;
            let mut t = String::from("| Service account | Key | Created |\n|---|---|---|\n");
            for sa in v.as_array().unwrap_or(&vec![]) {
                let email = sa["email"].as_str().unwrap_or("");
                if let Ok(keys) = run("gcloud", &["iam", "service-accounts", "keys", "list",
                    "--iam-account", email, "--managed-by=user", "--project", project, "--format=json"]) {
                    let kv: Value = serde_json::from_str(&keys).unwrap_or(Value::Array(vec![]));
                    for k in kv.as_array().unwrap_or(&vec![]) {
                        let row = format!("| {email} | {} | {} |",
                            k["name"].as_str().unwrap_or("").rsplit('/').next().unwrap_or(""),
                            k["validAfterTime"].as_str().unwrap_or(""));
                        if matches_filter(&row, &filter) { t.push_str(&row); t.push('\n'); }
                    }
                }
            }
            Ok(t)
        }));
    }
    if gcp_projects.is_empty() {
        md.push_str("### GCP\n\n_skipped: set GCP_PROJECTS=proj1,proj2_\n\n");
    }

    md.push_str("### Not automatable here (attach manually)\n\n- Google Workspace users + 2SV report (Admin console export)\n- Tracker (Linear) seats\n- Any system without API access\n\n");
    md.push_str(&sign_off_block("access review"));
    md.push_str("\n> Diffing: this packet lives on the compliance-archives branch — `git diff` against last quarter's packet shows exactly what changed.\n");

    let stamp = if filter.is_some() { format!("user-{}", utc_date("%Y%m%d-%H%M%S")) } else { date.clone() };
    emit("access-review", vec![(format!("evidence/{year}/{q}/access-review-{stamp}.md"), md)])?;
    Ok(0)
}

fn logins_table(v: &Value, filter: &Option<String>) -> String {
    let mut t = String::from("| Login |\n|---|\n");
    for m in v.as_array().unwrap_or(&vec![]) {
        let row = format!("| {} |", m["login"].as_str().unwrap_or(""));
        if matches_filter(&row, filter) { t.push_str(&row); t.push('\n'); }
    }
    t
}

fn sign_off_block(what: &str) -> String {
    format!(
        "## Review decision\n\n- [ ] Every grant above is appropriate for the holder's current role\n- Revocations required (list, or \"none\"): \n- Reviewer: \n- Date: \n\nSigning = committing this file with the boxes filled. That commit is the {what} evidence.\n"
    )
}

// ---------- mgmt-packet ----------

pub fn run_mgmt_packet() -> Result<i32, String> {
    let repo = std::env::var("REPO")
        .or_else(|_| std::env::var("GITHUB_REPOSITORY"))
        .map_err(|_| "REPO or GITHUB_REPOSITORY required")?;
    let (year, q, q_start) = quarter_now();
    let date = utc_date("%Y-%m-%d");
    let db = env_or("SHADOW_DB", "shadow/shadow.db");

    let mut md = format!(
        "# Management Review — {year} {q}\n\nGenerated {date} by `shadow-ci mgmt-packet`. Criteria: CC1.2, CC4.2.\nThe meeting's job: read the numbers, decide, record decisions below. The packet is pre-filled; the minutes are not.\n\n"
    );

    md.push_str(&section_or_err("Gauge trend (last 13 readings)", || {
        if !std::path::Path::new(&db).exists() {
            return Err(format!("{db} not found"));
        }
        let out = run("sqlite3", &[&db, "SELECT ts, printf('%.1f', gauge), COALESCE(cap_reason,'') FROM gauge_history ORDER BY ts DESC LIMIT 13"])?;
        let mut t = String::from("| Reading | Gauge | Cap reason |\n|---|---|---|\n");
        for line in out.lines() {
            let cells: Vec<&str> = line.split('|').collect();
            if cells.len() >= 2 {
                t.push_str(&format!("| {} | {}% | {} |\n", cells[0], cells[1], cells.get(2).unwrap_or(&"")));
            }
        }
        Ok(t)
    }));

    md.push_str(&section_or_err("Bypass merges this quarter", || {
        let branch = env_or("ARCHIVES_BRANCH", "compliance-archives");
        git(&["fetch", "origin", &branch])?;
        let hits = git(&["grep", "-l", "\"is_bypass\": true", &format!("origin/{branch}"), "--", "*.json"])
            .unwrap_or_default();
        let mut t = String::new();
        for h in hits.lines() {
            let file = h.rsplit(':').next().unwrap_or(h);
            // pr-{n}-{ticket}-YYYYMMDD.json — keep only this quarter's
            if let Some(datepart) = file.strip_suffix(".json").and_then(|s| s.rsplit('-').next()) {
                if datepart >= q_start.replace('-', "").as_str() {
                    t.push_str(&format!("- `{file}` — verify a linked incident ticket + backport PR exists\n"));
                }
            }
        }
        Ok(t)
    }));

    md.push_str(&section_or_err("Open security alerts", || {
        let count = |path: &str| -> String {
            gh(&["api", &format!("repos/{repo}/{path}"), "--paginate", "--jq", "length"])
                .map(|s| s.lines().map(|l| l.trim().parse::<i64>().unwrap_or(0)).sum::<i64>().to_string())
                .unwrap_or_else(|_| "n/a".into())
        };
        Ok(format!(
            "| Source | Open |\n|---|---|\n| Dependabot | {} |\n| Code scanning | {} |\n| Secret scanning | {} |\n",
            count("dependabot/alerts?state=open"),
            count("code-scanning/alerts?state=open"),
            count("secret-scanning/alerts?state=open"),
        ))
    }));

    md.push_str(&section_or_err("Incidents this quarter (label: incident)", || {
        let out = gh(&["issue", "list", "--repo", &repo, "--label", "incident", "--state", "all",
            "--search", &format!("created:>={q_start}"), "--json", "number,title,state",
            "--jq", ".[] | \"- #\\(.number) [\\(.state)] \\(.title)\""])?;
        Ok(out)
    }));

    md.push_str(&section_or_err("Open shadow regression tickets (label: shadow)", || {
        let out = gh(&["issue", "list", "--repo", &repo, "--label", "shadow", "--state", "open",
            "--json", "number,title", "--jq", ".[] | \"- #\\(.number) \\(.title)\""])?;
        Ok(out)
    }));

    md.push_str(
        "## Agenda\n\n- [ ] Gauge trend reviewed — direction and caps explained\n- [ ] Every bypass merge has its incident ticket + backport\n- [ ] Security alerts within policy SLAs\n- [ ] Incidents reviewed; postmortems filed\n- [ ] Risk register: anything to add from this quarter?\n- [ ] Upcoming attestation expiries assigned\n\n## Decisions & action items\n\n| Decision / action | Owner | Ticket |\n|---|---|---|\n| | | |\n\n## Minutes\n\n(what was discussed, by whom)\n\n- Attendees: \n- Date held: \n\nSigning = committing this file completed. That commit is the CC1.2/CC4.2 oversight evidence.\n",
    );

    emit("management-review", vec![(format!("evidence/{year}/{q}/management-review-{date}.md"), md)])?;
    Ok(0)
}

// ---------- release-record ----------

pub fn run_release_record() -> Result<i32, String> {
    git(&["fetch", "origin", "main", "staging"]).ok();
    let commits = git(&["log", "origin/main..origin/staging", "--oneline", "--no-merges"])?;
    if commits.trim().is_empty() {
        println!("staging is not ahead of main; nothing to record");
        return Ok(0);
    }
    let full_log = git(&["log", "origin/main..origin/staging", "--pretty=%s"])?;
    let ticket_pattern = env_or("TICKET_PATTERN", r"[A-Z]{2,6}-[0-9]+|#[0-9]+");
    let tickets = crate::check::extract_tickets("", &full_log, &ticket_pattern);
    let prs: Vec<String> = {
        let re = regex::Regex::new(r"#(\d+)").unwrap();
        let mut seen = std::collections::HashSet::new();
        re.captures_iter(&full_log)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .filter(|n| seen.insert(n.clone()))
            .collect()
    };
    let stats = git(&["diff", "--shortstat", "origin/main...origin/staging"]).unwrap_or_default();
    let ts = utc_date("%Y%m%d-%H%M%S");
    let released_by = env_or("RELEASED_BY", &env_or("GITHUB_ACTOR", "unknown"));

    let record = serde_json::json!({
        "release_id": format!("release-{ts}"),
        "timestamp": utc_date("%Y-%m-%dT%H:%M:%SZ"),
        "released_by": released_by,
        "from_ref": "origin/main",
        "to_ref": "origin/staging",
        "commits": commits.lines().count(),
        "pull_requests": prs,
        "tickets": tickets,
        "diff_shortstat": stats.trim(),
    });
    let md = format!(
        "# Release {ts}\n\n| Field | Value |\n|---|---|\n| Released by | {released_by} |\n| Commits | {} |\n| PRs | {} |\n| Tickets | {} |\n| Diff | {} |\n\n## Commits\n\n```\n{}```\n",
        commits.lines().count(),
        record["pull_requests"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).map(|p| format!("#{p}")).collect::<Vec<_>>().join(", ")).unwrap_or_default(),
        tickets.join(", "),
        stats.trim(),
        commits
    );

    emit("release-record", vec![
        (format!("releases/release-{ts}.json"), serde_json::to_string_pretty(&record).unwrap()),
        (format!("releases/release-{ts}.md"), md),
    ])?;
    println!("release-{ts}: {} commits, {} PRs, {} tickets", commits.lines().count(), prs.len(), tickets.len());
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_math() {
        assert_eq!(quarter_of("2026-01"), ("2026".into(), "Q1".into(), "2026-01-01".into()));
        assert_eq!(quarter_of("2026-07"), ("2026".into(), "Q3".into(), "2026-07-01".into()));
        assert_eq!(quarter_of("2026-12"), ("2026".into(), "Q4".into(), "2026-10-01".into()));
    }

    #[test]
    fn user_filter_matching() {
        let f = Some("jane".to_string());
        assert!(matches_filter("| Jane Doe | admin |", &f));
        assert!(!matches_filter("| alice | member |", &f));
        assert!(matches_filter("| anyone |", &None));
    }
}
