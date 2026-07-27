// Thin wrappers over the CLIs guaranteed on GitHub Actions runners: gh, git, curl.
use std::process::Command;

pub fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!(
            "{program} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

pub fn gh(args: &[&str]) -> Result<String, String> {
    run("gh", args)
}

pub fn gh_json(args: &[&str]) -> Result<serde_json::Value, String> {
    let out = gh(args)?;
    serde_json::from_str(&out).map_err(|e| format!("bad json from gh {}: {e}", args.join(" ")))
}

pub fn git(args: &[&str]) -> Result<String, String> {
    run("git", args)
}

pub fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn utc_date(format: &str) -> String {
    run("date", &["-u", &format!("+{format}")])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown-date".into())
}

/// POST a JSON body with curl (Slack webhooks, Linear GraphQL).
pub fn curl_post(url: &str, headers: &[(&str, &str)], body: &str) -> Result<String, String> {
    let mut args: Vec<String> = vec![
        "-sS".into(),
        "-X".into(),
        "POST".into(),
        "--max-time".into(),
        "30".into(),
    ];
    for (k, v) in headers {
        args.push("-H".into());
        args.push(format!("{k}: {v}"));
    }
    args.push("-d".into());
    args.push(body.to_string());
    args.push(url.to_string());
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run("curl", &arg_refs)
}

/// Commit a set of (relative_path, content) files to the append-only archives
/// branch via a temporary worktree. Creates the orphan branch on first use.
/// Retries once through a rebase if a concurrent writer won the push race.
pub fn commit_to_archives(branch: &str, files: &[(String, String)], msg: &str) -> Result<(), String> {
    let dir = format!("/tmp/shadow-archives-{}", std::process::id());
    let _ = git(&["worktree", "remove", "--force", &dir]);

    if git(&["fetch", "origin", branch]).is_ok() {
        git(&["worktree", "add", &dir, &format!("origin/{branch}")])?;
        git(&["-C", &dir, "checkout", "-B", branch, &format!("origin/{branch}")])?;
    } else {
        git(&["worktree", "add", "--detach", &dir])?;
        git(&["-C", &dir, "checkout", "--orphan", branch])?;
        git(&["-C", &dir, "rm", "-rf", "--ignore-unmatch", "."])?;
        std::fs::write(
            format!("{dir}/README.md"),
            "# compliance-archives\n\nAppend-only evidence branch written by shadow-ci. PR records, release records, and quarterly evidence packets. Never merge this branch anywhere.\n",
        )
        .map_err(|e| e.to_string())?;
    }

    for (path, content) in files {
        let full = format!("{dir}/{path}");
        if let Some(parent) = std::path::Path::new(&full).parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&full, content).map_err(|e| e.to_string())?;
    }

    git(&["-C", &dir, "add", "-A"])?;
    git(&["-C", &dir, "-c", "user.name=shadow-ci", "-c", "user.email=shadow-ci@noreply.local",
          "commit", "-m", msg])?;
    if git(&["-C", &dir, "push", "origin", &format!("HEAD:{branch}")]).is_err() {
        git(&["-C", &dir, "pull", "--rebase", "origin", branch])?;
        git(&["-C", &dir, "push", "origin", &format!("HEAD:{branch}")])?;
    }
    let _ = git(&["worktree", "remove", "--force", &dir]);
    Ok(())
}
