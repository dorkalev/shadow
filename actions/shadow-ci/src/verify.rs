//! Continuous, deterministic readiness verification.
//!
//! The JSON report emitted here is the canonical, vendor-neutral evidence
//! snapshot. SQLite is only a dashboard render cache. Every observation names
//! the procedures and criteria it supports, its evidence dimension, source,
//! and timestamp so a later Drata/Vanta/auditor adapter never has to infer
//! what a boolean meant.
use crate::util::{commit_to_archives, gh_json, run};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const FIREBASE_RULE_TEST_PATHS: &[&str] = &[
    "app/test/firestore.rules.test.ts",
    "test/firestore.rules.test.ts",
    "tests/firestore.rules.test.ts",
    "functions/test/firestore.rules.test.ts",
];

const DEPENDENCY_LOCK_PATHS: &[&str] = &[
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "Cargo.lock",
    "app/package-lock.json",
    "app/pnpm-lock.yaml",
    "app/yarn.lock",
    "functions/package-lock.json",
    "functions/pnpm-lock.yaml",
    "functions/yarn.lock",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Pass,
    Fail,
    Unknown,
    NotApplicable,
}

impl Verdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Unknown => "unknown",
            Self::NotApplicable => "n/a",
        }
    }
}

fn observation(
    id: &str,
    procedure: Option<&str>,
    criteria: &[&str],
    dimension: &str,
    verdict: Verdict,
    evidence: impl Into<String>,
    source: impl Into<String>,
    observed_at: &str,
) -> Value {
    json!({
        "id": id,
        "procedure_id": procedure,
        "criteria": criteria,
        "dimension": dimension,
        "verdict": verdict.as_str(),
        "evidence": evidence.into(),
        "source": source.into(),
        "observed_at": observed_at,
        "expires_at": null
    })
}

fn api_array(
    id: &str,
    procedure: &str,
    criteria: &[&str],
    value: Result<Value, String>,
    source: &str,
    observed_at: &str,
) -> Value {
    match value {
        Ok(v) => match v.as_array() {
            Some(items) if items.is_empty() => observation(
                id,
                Some(procedure),
                criteria,
                "technical",
                Verdict::Pass,
                "no open findings",
                source,
                observed_at,
            ),
            Some(items) => observation(
                id,
                Some(procedure),
                criteria,
                "technical",
                Verdict::Fail,
                format!("{} open finding(s)", items.len()),
                source,
                observed_at,
            ),
            None => observation(
                id,
                Some(procedure),
                criteria,
                "technical",
                Verdict::Unknown,
                "API response was not an array",
                source,
                observed_at,
            ),
        },
        Err(e) => observation(
            id,
            Some(procedure),
            criteria,
            "technical",
            Verdict::Unknown,
            format!("could not query live API: {e}"),
            source,
            observed_at,
        ),
    }
}

fn branch_protection(repo: &str, branch: &str, observed_at: &str) -> Value {
    let endpoint = format!("repos/{repo}/branches/{branch}/protection");
    match gh_json(&["api", &endpoint]) {
        Ok(_) => observation(
            &format!("github.branch_protection.{branch}"),
            Some("branch-rulesets"),
            &["CC8.1", "CC6.3"],
            "technical",
            Verdict::Pass,
            "branch protection endpoint is configured",
            format!("github:{endpoint}"),
            observed_at,
        ),
        Err(e) if e.contains("HTTP 404") => observation(
            &format!("github.branch_protection.{branch}"),
            Some("branch-rulesets"),
            &["CC8.1", "CC6.3"],
            "technical",
            Verdict::Fail,
            "branch is not protected",
            format!("github:{endpoint}"),
            observed_at,
        ),
        Err(e) => observation(
            &format!("github.branch_protection.{branch}"),
            Some("branch-rulesets"),
            &["CC8.1", "CC6.3"],
            "technical",
            Verdict::Unknown,
            format!("could not query live API: {e}"),
            format!("github:{endpoint}"),
            observed_at,
        ),
    }
}

fn first_existing(root: &Path, paths: &[&str]) -> Option<PathBuf> {
    paths.iter().map(|p| root.join(p)).find(|p| p.is_file())
}

fn any_under(root: &Path, relative: &str, needle: &str) -> bool {
    fn walk(dir: &Path, needle: &str) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else { return false };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.to_lowercase().contains(needle))
            {
                return true;
            }
            if path.is_dir() && walk(&path, needle) {
                return true;
            }
        }
        false
    }
    walk(&root.join(relative), &needle.to_lowercase())
}

fn attestation_observations(root: &Path, observed_at: &str) -> Vec<Value> {
    let dir = root.join("evidence/attestations");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut latest: BTreeMap<String, (String, Value, PathBuf)> = BTreeMap::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(body) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&body) else {
            continue;
        };
        let Some(criterion) = value["criterion"].as_str() else {
            continue;
        };
        let attested_at = value["attested_at"].as_str().unwrap_or_default().to_string();
        let replace = latest
            .get(criterion)
            .map_or(true, |(current, _, _)| attested_at > *current);
        if replace {
            latest.insert(criterion.to_string(), (attested_at, value, path));
        }
    }

    latest
        .into_iter()
        .map(|(criterion, (_, value, path))| {
            let expires_at = value["expires_at"].as_str().unwrap_or_default();
            let attested_by = value["attested_by"].as_str().unwrap_or("unknown");
            let note = value["note"].as_str().unwrap_or("human control attestation");
            let valid = !expires_at.is_empty() && expires_at >= observed_at;
            let procedure = value["procedure_id"].as_str();
            let mut result = observation(
                &format!("attestation.{criterion}"),
                procedure,
                &[criterion.as_str()],
                "operating",
                if valid { Verdict::Pass } else { Verdict::Fail },
                if valid {
                    format!("attested by {attested_by}: {note}")
                } else {
                    format!("attestation by {attested_by} is missing an expiry or expired at {expires_at}")
                },
                format!("repo:{}", path.display()),
                observed_at,
            );
            result["expires_at"] = if expires_at.is_empty() { Value::Null } else { json!(expires_at) };
            result
        })
        .collect()
}

fn file_check(
    root: &Path,
    id: &str,
    procedure: Option<&str>,
    criteria: &[&str],
    paths: &[&str],
    observed_at: &str,
) -> Value {
    match first_existing(root, paths) {
        Some(path) => observation(
            id,
            procedure,
            criteria,
            "design",
            Verdict::Pass,
            format!("required artifact exists: {}", path.display()),
            format!("repo:{}", path.display()),
            observed_at,
        ),
        None => observation(
            id,
            procedure,
            criteria,
            "design",
            Verdict::Fail,
            format!("none of the required artifacts exist: {}", paths.join(", ")),
            "repository",
            observed_at,
        ),
    }
}

fn approved_document_check(
    root: &Path,
    id: &str,
    procedure: Option<&str>,
    criteria: &[&str],
    paths: &[&str],
    observed_at: &str,
) -> Value {
    let Some(path) = first_existing(root, paths) else {
        return observation(
            id,
            procedure,
            criteria,
            "design",
            Verdict::Fail,
            format!("none of the required documents exist: {}", paths.join(", ")),
            "repository",
            observed_at,
        );
    };
    let body = std::fs::read_to_string(&path).unwrap_or_default();
    let problem = document_problem(&body);
    observation(
        id,
        procedure,
        criteria,
        "design",
        if problem.is_none() { Verdict::Pass } else { Verdict::Fail },
        problem.map_or_else(
            || format!("approved control document is present: {}", path.display()),
            |reason| format!("{} {reason}", path.display()),
        ),
        format!("repo:{}", path.display()),
        observed_at,
    )
}

fn document_problem(body: &str) -> Option<String> {
    let upper = body.to_uppercase();
    let placeholders = ["TODO", "TBD", "OPEN —", "PLACEHOLDER", "REPLACE_ME", "DRAFT —"];
    if let Some(marker) = placeholders.iter().find(|p| upper.contains(*p)) {
        Some(format!("still contains placeholder marker {marker}"))
    } else if body.trim().len() < 80 {
        Some("is too short to establish the control design".into())
    } else {
        None
    }
}

fn file_contains_check(
    root: &Path,
    id: &str,
    procedure: &str,
    criteria: &[&str],
    path: &str,
    required: &[&str],
    observed_at: &str,
) -> Value {
    let full = root.join(path);
    let body = std::fs::read_to_string(&full);
    match body {
        Ok(body) if required.iter().all(|s| body.contains(s)) => observation(
            id,
            Some(procedure),
            criteria,
            "design",
            Verdict::Pass,
            format!("{path} contains all required sections"),
            format!("repo:{path}"),
            observed_at,
        ),
        Ok(_) => observation(
            id,
            Some(procedure),
            criteria,
            "design",
            Verdict::Fail,
            format!("{path} is missing one or more required sections: {}", required.join(", ")),
            format!("repo:{path}"),
            observed_at,
        ),
        Err(e) => observation(
            id,
            Some(procedure),
            criteria,
            "design",
            Verdict::Fail,
            format!("cannot read {path}: {e}"),
            format!("repo:{path}"),
            observed_at,
        ),
    }
}

fn firestore_rules_check(root: &Path, observed_at: &str) -> Value {
    let path = root.join("firestore.rules");
    match std::fs::read_to_string(&path) {
        Ok(body)
            if body.contains("rules_version")
                && !body.contains("allow read, write: if true")
                && !body.contains("allow read: if true") => observation(
            "firebase.firestore_rules",
            None,
            &["CC6.1", "CC6.3", "C1.1"],
            "design",
            Verdict::Pass,
            "Firestore rules are versioned and do not contain an unconditional allow",
            "repo:firestore.rules",
            observed_at,
        ),
        Ok(_) => observation(
            "firebase.firestore_rules",
            None,
            &["CC6.1", "CC6.3", "C1.1"],
            "design",
            Verdict::Fail,
            "firestore.rules is missing a version or contains an unconditional allow",
            "repo:firestore.rules",
            observed_at,
        ),
        Err(e) => observation(
            "firebase.firestore_rules",
            None,
            &["CC6.1", "CC6.3", "C1.1"],
            "design",
            Verdict::Fail,
            format!("cannot read firestore.rules: {e}"),
            "repo:firestore.rules",
            observed_at,
        ),
    }
}

fn gcp_observations(projects: &[String], observed_at: &str) -> Vec<Value> {
    if projects.is_empty() {
        return vec![
            observation(
                "gcp.uptime_alerting",
                Some("uptime-alerting"),
                &["CC7.2", "A1.1"],
                "technical",
                Verdict::Unknown,
                "GCP_PROJECTS is not configured",
                "gcp",
                observed_at,
            ),
            observation(
                "gcp.firestore_pitr",
                Some("backups-pitr"),
                &["A1.2"],
                "technical",
                Verdict::Unknown,
                "GCP_PROJECTS is not configured",
                "gcp",
                observed_at,
            ),
            observation(
                "gcp.audit_logging",
                Some("audit-logging"),
                &["CC7.2", "CC2.1"],
                "technical",
                Verdict::Unknown,
                "GCP_PROJECTS is not configured",
                "gcp",
                observed_at,
            ),
            observation(
                "gcp.firestore_delete_protection",
                Some("backups-pitr"),
                &["A1.2"],
                "technical",
                Verdict::Unknown,
                "GCP_PROJECTS is not configured",
                "gcp",
                observed_at,
            ),
            observation(
                "gcp.primitive_iam_bindings",
                Some("access-register"),
                &["CC6.2", "CC6.3"],
                "technical",
                Verdict::Unknown,
                "GCP_PROJECTS is not configured",
                "gcp",
                observed_at,
            ),
        ];
    }

    let mut out = Vec::new();
    for project in projects {
        let db = run("gcloud", &["firestore", "databases", "describe", "--project", project, "--format=json"]);
        let pitr_pass = db
            .as_ref()
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v["pointInTimeRecoveryEnablement"].as_str().map(str::to_owned))
            .is_some_and(|v| v.contains("ENABLED"));
        out.push(observation(
            &format!("gcp.firestore_pitr.{project}"),
            Some("backups-pitr"),
            &["A1.2"],
            "technical",
            if pitr_pass { Verdict::Pass } else if db.is_ok() { Verdict::Fail } else { Verdict::Unknown },
            db.as_ref().err().cloned().unwrap_or_else(|| format!("Firestore PITR enabled={pitr_pass}")),
            format!("gcp:{project}:firestore"),
            observed_at,
        ));

        let delete_protected = db
            .as_ref()
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v["deleteProtectionState"].as_str().map(str::to_owned))
            .is_some_and(|v| v.contains("ENABLED"));
        out.push(observation(
            &format!("gcp.firestore_delete_protection.{project}"),
            Some("backups-pitr"),
            &["A1.2"],
            "technical",
            if delete_protected { Verdict::Pass } else if db.is_ok() { Verdict::Fail } else { Verdict::Unknown },
            format!("Firestore delete protection enabled={delete_protected}"),
            format!("gcp:{project}:firestore"),
            observed_at,
        ));

        let uptime = run("gcloud", &["monitoring", "uptime", "list-configs", "--project", project, "--format=json"]);
        let uptime_count = uptime.as_ref().ok().and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v.as_array().map(Vec::len));
        out.push(observation(
            &format!("gcp.uptime_alerting.{project}"),
            Some("uptime-alerting"),
            &["CC7.2", "A1.1"],
            "technical",
            match uptime_count { Some(n) if n > 0 => Verdict::Pass, Some(_) => Verdict::Fail, None => Verdict::Unknown },
            uptime.as_ref().err().cloned().unwrap_or_else(|| format!("{} uptime configuration(s)", uptime_count.unwrap_or(0))),
            format!("gcp:{project}:monitoring"),
            observed_at,
        ));

        let audit = run(
            "gcloud",
            &[
                "logging",
                "read",
                "logName:\"cloudaudit.googleapis.com\"",
                "--project",
                project,
                "--limit=1",
                "--format=json",
            ],
        );
        let audit_count = audit
            .as_ref()
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v.as_array().map(Vec::len));
        out.push(observation(
            &format!("gcp.audit_activity.{project}"),
            Some("audit-logging"),
            &["CC7.2", "CC2.1"],
            "technical",
            match audit_count { Some(n) if n > 0 => Verdict::Pass, Some(_) => Verdict::Unknown, None => Verdict::Unknown },
            audit.as_ref().err().cloned().unwrap_or_else(|| format!("{0} recent Cloud Audit Log record(s) sampled", audit_count.unwrap_or(0))),
            format!("gcp:{project}:logging"),
            observed_at,
        ));

        let iam = run("gcloud", &["projects", "get-iam-policy", project, "--format=json"]);
        let risky: Option<Vec<String>> = iam
            .as_ref()
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v["bindings"].as_array().cloned())
            .map(|bindings| {
                bindings
                    .iter()
                    .filter(|b| matches!(b["role"].as_str(), Some("roles/owner" | "roles/editor")))
                    .flat_map(|b| {
                        let role = b["role"].as_str().unwrap_or("primitive-role");
                        b["members"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(Value::as_str)
                            .map(move |member| format!("{role}:{member}"))
                    })
                    .collect()
            });
        out.push(observation(
            &format!("gcp.primitive_iam_bindings.{project}"),
            Some("access-register"),
            &["CC6.2", "CC6.3"],
            "technical",
            match risky.as_ref() { Some(items) if items.is_empty() => Verdict::Pass, Some(_) => Verdict::Fail, None => Verdict::Unknown },
            iam.as_ref().err().cloned().unwrap_or_else(|| format!("{} primitive owner/editor binding(s)", risky.as_ref().map(Vec::len).unwrap_or(0))),
            format!("gcp:{project}:iam"),
            observed_at,
        ));
    }
    out
}

fn summary(checks: &[Value]) -> Value {
    let mut dimensions: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    let mut evidenced = BTreeSet::new();
    let mut operating = BTreeSet::new();
    for check in checks {
        let dimension = check["dimension"].as_str().unwrap_or("unknown").to_string();
        let entry = dimensions.entry(dimension.clone()).or_default();
        match check["verdict"].as_str().unwrap_or("unknown") {
            "pass" => {
                entry.0 += 1;
                if let Some(criteria) = check["criteria"].as_array() {
                    for criterion in criteria.iter().filter_map(Value::as_str) {
                        evidenced.insert(criterion.to_string());
                        if dimension == "technical" || dimension == "operating" {
                            operating.insert(criterion.to_string());
                        }
                    }
                }
            }
            "fail" => entry.1 += 1,
            "n/a" => {}
            _ => entry.2 += 1,
        }
    }
    let dimensions: BTreeMap<String, Value> = dimensions
        .into_iter()
        .map(|(name, (pass, fail, unknown))| {
            let denominator = pass + fail + unknown;
            let percent = if denominator == 0 { 0.0 } else { pass as f64 * 100.0 / denominator as f64 };
            (name, json!({"pass": pass, "fail": fail, "unknown": unknown, "percent": percent}))
        })
        .collect();
    json!({
        "dimensions": dimensions,
        "criteria_with_evidence": evidenced.len(),
        "criteria_with_operating_evidence": operating.len()
    })
}

pub fn run_verify() -> Result<i32, String> {
    let repo = std::env::var("REPO").or_else(|_| std::env::var("GITHUB_REPOSITORY"))
        .map_err(|_| "REPO or GITHUB_REPOSITORY is required".to_string())?;
    let root = PathBuf::from(std::env::var("SHADOW_ROOT").unwrap_or_else(|_| ".".into()));
    let (owner, _) = repo.split_once('/').ok_or("REPO must be owner/name")?;
    let branches = std::env::var("SHADOW_BRANCHES").unwrap_or_else(|_| "main,staging".into());
    let gcp_projects: Vec<String> = std::env::var("GCP_PROJECTS").unwrap_or_default()
        .split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned).collect();
    let observed_at = crate::util::utc_date("%Y-%m-%dT%H:%M:%SZ");
    let mut checks = Vec::new();

    let repo_info = gh_json(&["api", &format!("repos/{repo}")]);
    let owner_type = repo_info.as_ref().ok().and_then(|v| v["owner"]["type"].as_str());
    checks.push(match owner_type {
        Some("Organization") => match gh_json(&["api", &format!("orgs/{owner}")]) {
            Ok(v) if v["two_factor_requirement_enabled"].as_bool() == Some(true) => observation("github.org_2fa_required", Some("org-2fa"), &["CC6.1", "CC6.2"], "technical", Verdict::Pass, "organization requires 2FA", format!("github:orgs/{owner}"), &observed_at),
            Ok(_) => observation("github.org_2fa_required", Some("org-2fa"), &["CC6.1", "CC6.2"], "technical", Verdict::Fail, "organization does not require 2FA", format!("github:orgs/{owner}"), &observed_at),
            Err(e) => observation("github.org_2fa_required", Some("org-2fa"), &["CC6.1", "CC6.2"], "technical", Verdict::Unknown, e, format!("github:orgs/{owner}"), &observed_at),
        },
        Some("User") => observation("github.org_2fa_required", Some("org-2fa"), &["CC6.1", "CC6.2"], "technical", Verdict::NotApplicable, "repository is user-owned; account MFA requires attestation", format!("github:repos/{repo}"), &observed_at),
        _ => observation("github.org_2fa_required", Some("org-2fa"), &["CC6.1", "CC6.2"], "technical", Verdict::Unknown, "could not determine owner type", format!("github:repos/{repo}"), &observed_at),
    });

    for branch in branches.split(',').map(str::trim).filter(|b| !b.is_empty()) {
        checks.push(branch_protection(&repo, branch, &observed_at));
    }
    checks.push(api_array("github.open_dependabot_alerts", "dependabot", &["CC7.1"], gh_json(&["api", &format!("repos/{repo}/dependabot/alerts?state=open&per_page=100")]), &format!("github:repos/{repo}/dependabot/alerts"), &observed_at));
    checks.push(api_array("github.open_code_scanning_alerts", "codeql", &["CC7.1"], gh_json(&["api", &format!("repos/{repo}/code-scanning/alerts?state=open&per_page=100")]), &format!("github:repos/{repo}/code-scanning/alerts"), &observed_at));
    checks.push(api_array("github.open_secret_scanning_alerts", "secret-scanning", &["CC6.1", "CC7.1"], gh_json(&["api", &format!("repos/{repo}/secret-scanning/alerts?state=open&per_page=100")]), &format!("github:repos/{repo}/secret-scanning/alerts"), &observed_at));

    checks.push(file_contains_check(&root, "repo.pr_template", "pr-template", &["CC8.1"], ".github/pull_request_template.md", &["Summary", "Tickets", "Changes", "Test Plan"], &observed_at));
    checks.push(file_check(&root, "repo.ci_workflow", Some("ci-tests"), &["CC8.1"], &[".github/workflows/ci.yml", ".github/workflows/test.yml"], &observed_at));
    checks.push(file_check(&root, "repo.compliance_gate", Some("compliance-audit-agent"), &["CC8.1", "CC4.1"], &[".github/workflows/compliance.yml"], &observed_at));
    checks.push(file_check(&root, "repo.review_gate", Some("compliance-review-gate"), &["CC8.1", "CC4.1"], &[".github/workflows/review.yml"], &observed_at));
    checks.push(file_check(&root, "repo.post_merge_archive", Some("post-merge-archive"), &["CC8.1", "CC4.1", "CC2.1"], &[".github/workflows/post-merge-archive.yml"], &observed_at));
    checks.push(file_check(&root, "repo.daily_verify", Some("daily-verify"), &["CC4.1"], &[".github/workflows/deterministic-verify.yml", ".github/workflows/deterministic-dashboard.yml"], &observed_at));
    checks.push(file_check(&root, "repo.quarterly_rituals", Some("quarterly-access-review"), &["CC6.2", "CC6.3", "CC1.2", "CC4.2"], &[".github/workflows/quarterly-rituals.yml"], &observed_at));

    checks.push(approved_document_check(&root, "evidence.policy_canon", Some("policies-repo"), &["CC5.3", "CC1.1", "CC2.2"], &["policies/README.md", ".shadow/policies/README.md"], &observed_at));
    checks.push(approved_document_check(&root, "evidence.risk_register", Some("risk-register"), &["CC3.1", "CC3.2", "CC3.3", "CC3.4", "CC9.1"], &["policies/risk-register.md", "compliance/risk-register.md", "risk-register.md"], &observed_at));
    checks.push(approved_document_check(&root, "evidence.vendor_register", Some("vendor-register"), &["CC9.2", "P6.4", "P6.5"], &["policies/vendor-register.md", "compliance/vendor-register.md", "vendor-register.md"], &observed_at));
    checks.push(approved_document_check(&root, "evidence.access_register", Some("access-register"), &["CC6.2", "CC6.3"], &["policies/access-register.md", "compliance/access-register.md", "access-register.md"], &observed_at));
    checks.push(approved_document_check(&root, "repo.onboarding_runbook", Some("onboard-offboard"), &["CC6.2", "CC1.4"], &["runbooks/onboarding.md", "policies/onboarding-offboarding.md"], &observed_at));
    checks.push(approved_document_check(&root, "repo.offboarding_runbook", Some("onboard-offboard"), &["CC6.2", "CC1.4"], &["runbooks/offboarding.md", "policies/onboarding-offboarding.md"], &observed_at));
    checks.push(approved_document_check(&root, "repo.incident_runbook", Some("incident-runbook"), &["CC7.3", "CC7.4", "CC7.5"], &["runbooks/incident-response.md", "policies/incident-response.md"], &observed_at));
    checks.push(approved_document_check(&root, "repo.hotfix_runbook", Some("hotfix-runbook"), &["CC8.1", "CC7.4"], &["runbooks/hotfix.md", "policies/runbooks/hotfix.md"], &observed_at));
    checks.push(approved_document_check(&root, "repo.ai_policy", Some("ai-policy"), &["CC8.1", "CC6.3", "CC9.2"], &["policies/ai-agent-use.md", "policies/ai-development.md"], &observed_at));
    checks.push(approved_document_check(&root, "evidence.system_description", None, &["CC2.1", "CC2.2", "CC2.3"], &["SYSTEM-DESCRIPTION.md", "docs/system-description.md"], &observed_at));
    checks.push(approved_document_check(&root, "evidence.data_inventory", None, &["C1.1", "P3.1", "P4.1"], &["compliance/data-inventory.yaml", "compliance/data-inventory.yml", "docs/data-inventory.md"], &observed_at));
    checks.push(approved_document_check(&root, "evidence.privacy_notice", None, &["P1.1", "P2.1"], &["app/PRIVACY-NOTICE.md", "PRIVACY.md", "docs/privacy-notice.md"], &observed_at));
    checks.push(file_check(&root, "repo.privacy_tests", None, &["P2.1", "P3.1", "P3.2", "P4.2", "P4.3", "P5.1", "P5.2", "PI1.2", "PI1.4", "PI1.5"], &["app/test/privacy.test.ts", "test/privacy.test.ts"], &observed_at));
    if root.join("firebase.json").is_file()
        || std::env::var("SHADOW_PROFILE").ok().as_deref() == Some("firebase")
    {
        checks.push(file_check(&root, "firebase.configuration", None, &["CC5.2", "CC8.1"], &["firebase.json"], &observed_at));
        checks.push(firestore_rules_check(&root, &observed_at));
        checks.push(file_check(&root, "firebase.rules_tests", Some("ci-tests"), &["CC6.1", "CC6.3", "CC8.1"], FIREBASE_RULE_TEST_PATHS, &observed_at));
    } else {
        for (id, criteria) in [
            ("firebase.configuration", &["CC5.2", "CC8.1"][..]),
            ("firebase.firestore_rules", &["CC6.1", "CC6.3", "C1.1"][..]),
            ("firebase.rules_tests", &["CC6.1", "CC6.3", "CC8.1"][..]),
        ] {
            checks.push(observation(id, None, criteria, "design", Verdict::NotApplicable, "Firebase profile is not selected", "repository", &observed_at));
        }
    }
    checks.push(file_contains_check(&root, "gcp.keyless_deploy", "ci-tests", &["CC6.1", "CC6.3", "CC8.1"], ".github/workflows/deploy.yml", &["google-github-actions/auth", "workload_identity_provider", "service_account"], &observed_at));
    checks.push(file_check(&root, "repo.dependency_lock", Some("dependabot"), &["CC7.1", "CC8.1"], DEPENDENCY_LOCK_PATHS, &observed_at));

    let restore_present = any_under(&root, "evidence", "restore");
    checks.push(observation("evidence.restore_test", Some("restore-test"), &["A1.3"], "operating", if restore_present { Verdict::Pass } else { Verdict::Fail }, if restore_present { "restore-test evidence exists" } else { "no restore-test evidence found" }, "repo:evidence", &observed_at));
    let access_review_present = any_under(&root, "evidence", "access-review");
    checks.push(observation("evidence.access_review", Some("quarterly-access-review"), &["CC6.2", "CC6.3"], "operating", if access_review_present { Verdict::Pass } else { Verdict::Fail }, if access_review_present { "access-review evidence exists" } else { "no access-review evidence found" }, "repo:evidence", &observed_at));
    let oversight_present = any_under(&root, "evidence", "oversight") || any_under(&root, "evidence", "management-review");
    checks.push(observation("evidence.management_review", Some("quarterly-mgmt-review"), &["CC1.2", "CC4.2"], "operating", if oversight_present { Verdict::Pass } else { Verdict::Fail }, if oversight_present { "management-review evidence exists" } else { "no management-review evidence found" }, "repo:evidence", &observed_at));
    let tabletop_present = any_under(&root, "evidence", "tabletop");
    checks.push(observation("evidence.tabletop", Some("annual-rituals"), &["CC7.5", "CC3.1"], "operating", if tabletop_present { Verdict::Pass } else { Verdict::Fail }, if tabletop_present { "tabletop evidence exists" } else { "no tabletop evidence found" }, "repo:evidence", &observed_at));

    checks.extend(gcp_observations(&gcp_projects, &observed_at));
    checks.extend(attestation_observations(&root, &observed_at));

    let failures = checks.iter().filter(|c| c["verdict"] == "fail").count();
    let unknowns = checks.iter().filter(|c| c["verdict"] == "unknown").count();
    let report_summary = summary(&checks);
    let report = json!({
        "schema_version": 2,
        "subject": {"repository": repo, "gcp_projects": gcp_projects},
        "observed_at": observed_at,
        "checks": checks,
        "summary": report_summary,
        "failures": failures,
        "unknowns": unknowns
    });
    let body = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    std::fs::create_dir_all("shadow").map_err(|e| e.to_string())?;
    let date = crate::util::utc_date("%F");
    let stamp = crate::util::utc_date("%Y%m%dT%H%M%SZ");
    std::fs::write(format!("shadow/verify-{date}.json"), &body).map_err(|e| e.to_string())?;
    std::fs::write("shadow/readiness-latest.json", &body).map_err(|e| e.to_string())?;
    std::fs::write(format!("shadow/readiness-{stamp}.json"), &body).map_err(|e| e.to_string())?;

    if std::env::var("ARCHIVES_PUSH").as_deref() == Ok("1") {
        let branch = std::env::var("ARCHIVES_BRANCH").unwrap_or_else(|_| "compliance-archives".into());
        let month = crate::util::utc_date("%Y/%m");
        commit_to_archives(
            &branch,
            &[
                (format!("readiness/{month}/{stamp}.json"), body.clone()),
                ("readiness/latest.json".into(), body.clone()),
            ],
            &format!("readiness: {stamp}"),
        )?;
    }

    println!("{} deterministic observation(s), {failures} failure(s), {unknowns} unknown(s)", report["checks"].as_array().map(Vec::len).unwrap_or(0));
    Ok(if failures == 0 { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_documents_never_receive_design_credit() {
        assert_eq!(
            document_problem("approved"),
            Some("is too short to establish the control design".into())
        );
    }

    #[test]
    fn reviewed_documents_pass_design_validation() {
        let body = "# Risk register\n\nOwner: security\nApproved: 2026-08-05\nReview by: 2027-08-05\n\nRisk R-1 is accepted with quarterly review and a named accountable owner.";
        assert_eq!(document_problem(body), None);
    }

    #[test]
    fn draft_documents_never_receive_design_credit() {
        assert!(document_problem(
            "# Risk register\n\nDRAFT — TODO replace this with a reviewed risk assessment and treatment register."
        )
        .unwrap()
        .contains("placeholder"));
    }

    #[test]
    fn summary_keeps_dimensions_separate() {
        let checks = vec![
            observation("a", None, &["CC1.1"], "design", Verdict::Pass, "ok", "repo", "now"),
            observation("b", None, &["CC1.1"], "technical", Verdict::Fail, "bad", "api", "now"),
        ];
        let s = summary(&checks);
        assert_eq!(s["dimensions"]["design"]["percent"], 100.0);
        assert_eq!(s["dimensions"]["technical"]["percent"], 0.0);
    }

    #[test]
    fn api_arrays_preserve_pass_fail_and_unknown() {
        let pass = api_array("a", "p", &["CC7.1"], Ok(json!([])), "api", "now");
        let fail = api_array("b", "p", &["CC7.1"], Ok(json!([{"id": 1}])), "api", "now");
        let malformed = api_array("c", "p", &["CC7.1"], Ok(json!({"items": []})), "api", "now");
        let unavailable = api_array("d", "p", &["CC7.1"], Err("denied".into()), "api", "now");
        assert_eq!(pass["verdict"], "pass");
        assert_eq!(fail["verdict"], "fail");
        assert_eq!(malformed["verdict"], "unknown");
        assert_eq!(unavailable["verdict"], "unknown");
    }

    #[test]
    fn unconfigured_gcp_is_a_visible_blind_spot() {
        let checks = gcp_observations(&[], "now");
        assert_eq!(checks.len(), 5);
        assert!(checks.iter().all(|check| check["verdict"] == "unknown"));
        assert!(checks.iter().all(|check| check["source"] == "gcp"));
    }

    #[test]
    fn common_monorepo_control_artifacts_are_supported() {
        assert!(FIREBASE_RULE_TEST_PATHS.contains(&"app/test/firestore.rules.test.ts"));
        assert!(DEPENDENCY_LOCK_PATHS.contains(&"app/package-lock.json"));
        assert!(DEPENDENCY_LOCK_PATHS.contains(&"functions/package-lock.json"));
    }
}
