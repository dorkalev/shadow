//! Daily, deterministic verification of high-signal GitHub controls.
//!
//! This intentionally reports `unknown` for unavailable APIs rather than
//! treating unavailable data as a passing control. It is a cheap daily drift
//! detector; judgment-only criteria remain attested by humans or a manually
//! authorized deep review.
use crate::util::gh_json;
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Verdict {
    Pass,
    Fail,
    Unknown,
}

impl Verdict {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Unknown => "unknown",
        }
    }
}

fn result(id: &str, verdict: Verdict, evidence: String) -> Value {
    json!({"id": id, "verdict": verdict.as_str(), "evidence": evidence})
}

fn bool_control(id: &str, value: Result<bool, String>) -> Value {
    match value {
        Ok(true) => result(id, Verdict::Pass, "live API returned true".into()),
        Ok(false) => result(id, Verdict::Fail, "live API returned false".into()),
        Err(e) => result(id, Verdict::Unknown, format!("could not query live API: {e}")),
    }
}

fn array_control(id: &str, value: Result<Value, String>) -> Value {
    match value {
        Ok(v) => match v.as_array() {
            Some(items) if items.is_empty() => result(id, Verdict::Pass, "no open findings".into()),
            Some(items) => result(id, Verdict::Fail, format!("{} open finding(s)", items.len())),
            None => result(id, Verdict::Unknown, "API response was not an array".into()),
        },
        Err(e) => result(id, Verdict::Unknown, format!("could not query live API: {e}")),
    }
}

fn protected_branch(id: &str, value: Result<Value, String>) -> Value {
    match value {
        Ok(_) => result(id, Verdict::Pass, "branch protection endpoint is configured".into()),
        Err(e) if e.contains("HTTP 404") => result(id, Verdict::Fail, "branch is not protected".into()),
        Err(e) => result(id, Verdict::Unknown, format!("could not query live API: {e}")),
    }
}

pub fn run_verify() -> Result<i32, String> {
    let repo = std::env::var("REPO").or_else(|_| std::env::var("GITHUB_REPOSITORY"))
        .map_err(|_| "REPO or GITHUB_REPOSITORY is required".to_string())?;
    let (owner, _) = repo.split_once('/').ok_or("REPO must be owner/name")?;
    let branches = std::env::var("SHADOW_BRANCHES").unwrap_or_else(|_| "main".into());

    let org = gh_json(&["api", &format!("orgs/{owner}")]);
    let two_factor = org.map(|v| v["two_factor_requirement_enabled"].as_bool().unwrap_or(false));
    let mut checks = vec![bool_control("github.org_2fa_required", two_factor)];

    for branch in branches.split(',').map(str::trim).filter(|b| !b.is_empty()) {
        checks.push(protected_branch(
            &format!("github.branch_protection.{branch}"),
            gh_json(&["api", &format!("repos/{repo}/branches/{branch}/protection")]),
        ));
    }
    checks.push(array_control(
        "github.open_dependabot_alerts",
        gh_json(&["api", &format!("repos/{repo}/dependabot/alerts?state=open&per_page=100")]),
    ));
    checks.push(array_control(
        "github.open_code_scanning_alerts",
        gh_json(&["api", &format!("repos/{repo}/code-scanning/alerts?state=open&per_page=100")]),
    ));
    checks.push(array_control(
        "github.open_secret_scanning_alerts",
        gh_json(&["api", &format!("repos/{repo}/secret-scanning/alerts?state=open&per_page=100")]),
    ));

    let failures = checks.iter().filter(|c| c["verdict"] == "fail").count();
    let report = json!({"schema_version": 1, "repo": repo, "checks": checks, "failures": failures});
    std::fs::create_dir_all("shadow").map_err(|e| e.to_string())?;
    let date = crate::util::utc_date("%F");
    std::fs::write(format!("shadow/verify-{date}.json"), serde_json::to_string_pretty(&report).unwrap())
        .map_err(|e| e.to_string())?;
    println!("{} deterministic check(s), {} failure(s)", report["checks"].as_array().unwrap().len(), failures);
    Ok(if failures == 0 { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_controls_are_never_silently_passing() {
        assert_eq!(bool_control("x", Ok(true))["verdict"], "pass");
        assert_eq!(bool_control("x", Ok(false))["verdict"], "fail");
        assert_eq!(bool_control("x", Err("denied".into()))["verdict"], "unknown");
    }

    #[test]
    fn alerts_and_missing_protection_fail() {
        assert_eq!(array_control("x", Ok(json!([])))["verdict"], "pass");
        assert_eq!(array_control("x", Ok(json!([{}])))["verdict"], "fail");
        assert_eq!(protected_branch("x", Err("HTTP 404".into()))["verdict"], "fail");
    }
}
