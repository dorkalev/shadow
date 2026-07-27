// shadow-ci — deterministic SOC 2 compliance gates for GitHub, in one binary.
//
//   shadow-ci check     per-PR compliance audit (run on pull_request events)
//   shadow-ci archive   post-merge evidence record + bypass detection (run on PR close)
//
// No LLM in CI: these gates are deterministic and reproducible. The judgment
// half of the platform (scan, setup, periodic shadow audit) lives in the
// markdown runbooks under agent/, executed by Claude or another LLM.
//
// Requirements at runtime: gh (authenticated), git, curl — all present on
// GitHub Actions runners. Configuration is via environment variables; see
// actions/README.md for the full contract.
mod archive;
mod attest;
mod check;
mod rituals;
mod util;

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    let result = match cmd.as_str() {
        "check" => check::run_check(),
        "archive" => archive::run_archive(),
        "access-review" => rituals::run_access_review(),
        "mgmt-packet" => rituals::run_mgmt_packet(),
        "release-record" => rituals::run_release_record(),
        "attest" => attest::run_attest(),
        _ => {
            eprintln!("usage: shadow-ci <check|archive|access-review|mgmt-packet|release-record|attest>");
            eprintln!("  check    per-PR compliance audit   (env: REPO, PR_NUMBER, TICKET_PATTERN, REVIEW_PHASE,");
            eprintln!("           CONFIDENCE_THRESHOLD, REQUIRED_REVIEWERS, EXPECTED_REVIEWERS, TEST_EXCLUDE_PATHS, LINEAR_API_KEY)");
            eprintln!("  archive  post-merge evidence record (env: REPO, PR_NUMBER, TICKET_PATTERN, ARCHIVES_BRANCH,");
            eprintln!("           REQUIRED_CHECKS_OVERRIDE, SLACK_WEBHOOK_URL)");
            eprintln!("  access-review   quarterly access packet  (env: ORG, GCP_PROJECTS, USER_FILTER, ARCHIVES_PUSH)");
            eprintln!("  mgmt-packet     quarterly mgmt packet    (env: REPO, SHADOW_DB, ARCHIVES_PUSH)");
            eprintln!("  release-record  release evidence record  (env: TICKET_PATTERN, RELEASED_BY, ARCHIVES_PUSH)");
            eprintln!("  attest          CPA-style change attestation (env: REPO, SINCE, UNTIL, TICKET_PATTERN, ARCHIVES_PUSH)");
            std::process::exit(2);
        }
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("shadow-ci {cmd}: {e}");
            std::process::exit(2);
        }
    }
}
