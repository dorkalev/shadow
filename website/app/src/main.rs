// shadow — the compliance shadow one-pager.
// The site renders state; the agent computes it (agent/03-verify-compliance.md).
//   shadow seed  --criteria ../criteria --procedures ../procedures/PROCEDURES.md [--db shadow.db]
//   shadow serve [--db shadow.db] [--port 8300]
mod render;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS criteria (
  id TEXT PRIMARY KEY, family TEXT NOT NULL, category TEXT NOT NULL,
  text TEXT NOT NULL, weight INTEGER NOT NULL,
  in_scope INTEGER NOT NULL DEFAULT 1,
  status TEXT NOT NULL DEFAULT 'not_started',
  credit REAL NOT NULL DEFAULT 0.0, updated_at TEXT,
  automatable TEXT NOT NULL DEFAULT 'partial',
  nature TEXT NOT NULL DEFAULT 'technical');
CREATE TABLE IF NOT EXISTS checks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  criterion_id TEXT NOT NULL REFERENCES criteria(id),
  name TEXT NOT NULL, verdict TEXT NOT NULL, evidence TEXT,
  last_run TEXT NOT NULL, procedure_id TEXT, dimension TEXT NOT NULL DEFAULT 'technical',
  source TEXT, expires_at TEXT, UNIQUE(criterion_id, name));
CREATE TABLE IF NOT EXISTS attestations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  criterion_id TEXT NOT NULL, note TEXT NOT NULL, evidence_link TEXT,
  attested_by TEXT NOT NULL, attested_at TEXT NOT NULL, expires_at TEXT);
CREATE TABLE IF NOT EXISTS gauge_history (
  ts TEXT PRIMARY KEY, gauge REAL NOT NULL, cap REAL, cap_reason TEXT);
CREATE TABLE IF NOT EXISTS readiness_history (
  ts TEXT PRIMARY KEY, design REAL NOT NULL, technical REAL NOT NULL,
  evidence REAL NOT NULL, operating REAL NOT NULL);
CREATE TABLE IF NOT EXISTS procedures (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, category TEXT NOT NULL,
  criteria TEXT NOT NULL, install TEXT NOT NULL, detect TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'not_installed', last_checked TEXT);
";

fn ensure_schema(conn: &Connection) {
    conn.execute_batch(SCHEMA).expect("schema");
    // Forward-only migrations keep local dashboard caches readable. Failures
    // mean the column already exists and are intentionally harmless.
    let _ = conn.execute("ALTER TABLE checks ADD COLUMN procedure_id TEXT", []);
    let _ = conn.execute("ALTER TABLE checks ADD COLUMN dimension TEXT NOT NULL DEFAULT 'technical'", []);
    let _ = conn.execute("ALTER TABLE checks ADD COLUMN source TEXT", []);
    let _ = conn.execute("ALTER TABLE checks ADD COLUMN expires_at TEXT", []);
}

enum Runner {
    None,
    Shell(String), // SHADOW_RUNNER, run via sh -c with CRITERION/CRITERION_FILE/SHADOW_URL env
    Claude,        // claude CLI found on PATH: built-in single-criterion verifier prompt
}

struct App {
    db: Mutex<Connection>,
    db_path: String,
    token: Option<String>,
    org: String,
    running: Mutex<std::collections::HashSet<String>>,
    runner: Runner,
    criteria_dir: String,
    port: u16,
}

fn arg(args: &[String], flag: &str, default: &str) -> String {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let db_path = arg(&args, "--db", "shadow.db");
    match args.get(1).map(String::as_str) {
        Some("seed") => {
            let criteria_dir = arg(&args, "--criteria", "../criteria");
            let procedures_md = arg(&args, "--procedures", "../procedures/PROCEDURES.md");
            seed(&db_path, &criteria_dir, &procedures_md);
        }
        Some("serve") => {
            let port: u16 = arg(&args, "--port", "8300").parse().expect("bad --port");
            serve(db_path, port);
        }
        Some("render") => {
            let out = arg(&args, "--out", "dist");
            render_static(&db_path, &out);
        }
        Some("import-verify") => {
            let report = arg(&args, "--report", "shadow/verify.json");
            import_verify(&db_path, &report);
        }
        Some("apply-scope") => {
            let scope = arg(&args, "--scope", "shadow/scope.json");
            apply_scope(&db_path, &scope);
        }
        _ => {
            eprintln!("usage: shadow seed --criteria DIR --procedures FILE [--db PATH]");
            eprintln!("       shadow serve  [--db PATH] [--port 8300]");
            eprintln!("       shadow render [--db PATH] [--out dist]   # static export (index + criteria pages)");
            eprintln!("       shadow import-verify --db PATH --report verify.json");
            eprintln!("       shadow apply-scope --db PATH --scope shadow/scope.json");
            std::process::exit(2);
        }
    }
}

#[derive(Deserialize)]
struct ScopeConfig {
    categories: Vec<String>,
}

fn apply_scope(db_path: &str, scope_path: &str) {
    let config: ScopeConfig = serde_json::from_str(
        &std::fs::read_to_string(scope_path).expect("read scope configuration"),
    )
    .expect("parse scope configuration");
    let conn = Connection::open(db_path).expect("open db");
    ensure_schema(&conn);
    conn.execute("UPDATE criteria SET in_scope=0", []).expect("reset scope");
    for category in &config.categories {
        let normalized = match category.as_str() {
            "processing" | "processing-integrity" => "processing_integrity",
            other => other,
        };
        conn.execute("UPDATE criteria SET in_scope=1 WHERE category=?1", [normalized])
            .expect("apply scope category");
    }
    println!("applied {} scope category(s) from {scope_path}", config.categories.len());
}

// ---------- deterministic verifier import ----------

#[derive(Deserialize)]
struct VerifyReport {
    #[serde(default)]
    schema_version: u8,
    #[serde(default)]
    observed_at: String,
    checks: Vec<VerifyCheck>,
}

#[derive(Deserialize)]
struct VerifyCheck {
    id: String,
    verdict: String,
    #[serde(default)]
    evidence: String,
    #[serde(default)]
    criteria: Vec<String>,
    #[serde(default)]
    procedure_id: Option<String>,
    #[serde(default = "default_dimension")]
    dimension: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    observed_at: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
}

fn default_dimension() -> String {
    "technical".into()
}

fn criteria_for_legacy_verify_check(check: &str) -> Vec<String> {
    match check {
        "github.org_2fa_required" => vec!["CC6.1".into(), "CC6.2".into()],
        id if id.starts_with("github.branch_protection.") => vec!["CC8.1".into()],
        "github.open_dependabot_alerts" | "github.open_code_scanning_alerts" | "github.open_secret_scanning_alerts" => vec!["CC7.1".into()],
        _ => vec![],
    }
}

#[derive(Default)]
struct CriterionEvidence {
    pass_design: bool,
    pass_evidence: bool,
    pass_operating: bool,
    failed: bool,
}

fn status_for_evidence(state: &CriterionEvidence) -> (&'static str, f64) {
    if state.failed {
        ("failing", 0.0)
    } else if state.pass_operating {
        ("verified", 1.0)
    } else if state.pass_design || state.pass_evidence {
        ("implemented", 0.6)
    } else {
        ("not_started", 0.0)
    }
}

fn readiness_metrics(conn: &Connection) -> (f64, f64, f64, f64) {
    let total: f64 = conn
        .query_row("SELECT COALESCE(SUM(weight), 0) FROM criteria WHERE in_scope=1", [], |r| r.get(0))
        .unwrap_or(0.0);
    let weighted = |dimension_sql: &str| -> f64 {
        if total == 0.0 {
            return 0.0;
        }
        let sql = format!(
            "SELECT COALESCE(SUM(c.weight), 0) * 100.0 / ?1 FROM criteria c
             WHERE c.in_scope=1 AND EXISTS (
               SELECT 1 FROM checks ch WHERE ch.criterion_id=c.id AND ch.verdict='pass' AND {dimension_sql}
             ) AND NOT EXISTS (
               SELECT 1 FROM checks ch WHERE ch.criterion_id=c.id AND ch.verdict='fail' AND {dimension_sql}
             )"
        );
        conn.query_row(&sql, [total], |r| r.get(0)).unwrap_or(0.0)
    };
    let design = weighted("ch.dimension='design'");
    let evidence = weighted("1=1");
    let operating = weighted("ch.dimension IN ('technical','operating')");
    let technical: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN verdict='pass' THEN 1.0 ELSE 0.0 END) * 100.0 / NULLIF(COUNT(*),0), 0)
             FROM checks WHERE dimension='technical' AND verdict != 'n/a'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);
    (design, technical, evidence, operating)
}

fn import_verify(db_path: &str, report_path: &str) {
    let report: VerifyReport = serde_json::from_str(
        &std::fs::read_to_string(report_path).expect("read deterministic verify report"),
    )
    .expect("parse deterministic verify report");
    let check_count = report.checks.len();
    let conn = Connection::open(db_path).expect("open db");
    ensure_schema(&conn);
    let fallback_ts = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    let ts = if report.observed_at.is_empty() { fallback_ts } else { report.observed_at.clone() };
    let mut evidence_by_criterion: HashMap<String, CriterionEvidence> = HashMap::new();
    let mut procedure_state: HashMap<String, (&str, u8)> = HashMap::new();
    let mut cap_reason = None;

    // A readiness report is a complete current snapshot. Clearing the render
    // cache prevents retired checks from surviving forever as ghost evidence.
    conn.execute("DELETE FROM checks", []).expect("clear old checks");
    conn.execute("UPDATE criteria SET status='not_started', credit=0.0, updated_at=?1", [&ts])
        .expect("reset criterion cache");

    for check in &report.checks {
        if (check.id == "github.org_2fa_required" || check.id.starts_with("github.branch_protection."))
            && check.verdict == "fail"
        {
            cap_reason = Some(check.id.clone());
        }
        let criteria = if check.criteria.is_empty() {
            criteria_for_legacy_verify_check(&check.id)
        } else {
            check.criteria.clone()
        };
        let observed = check.observed_at.as_deref().unwrap_or(&ts);
        for criterion in criteria {
            conn.execute(
                "INSERT INTO checks (criterion_id, name, verdict, evidence, last_run, procedure_id, dimension, source, expires_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                 ON CONFLICT(criterion_id, name) DO UPDATE SET verdict=?3, evidence=?4, last_run=?5,
                   procedure_id=?6, dimension=?7, source=?8, expires_at=?9",
                params![criterion, &check.id, &check.verdict, &check.evidence, observed,
                    &check.procedure_id, &check.dimension, &check.source, &check.expires_at],
            )
            .expect("upsert check");
            let state = evidence_by_criterion.entry(criterion).or_default();
            match check.verdict.as_str() {
                "fail" => state.failed = true,
                "pass" if check.dimension == "design" => state.pass_design = true,
                "pass" if check.dimension == "evidence" => state.pass_evidence = true,
                "pass" if check.dimension == "technical" || check.dimension == "operating" => state.pass_operating = true,
                _ => {}
            }
        }
        if let Some(procedure) = &check.procedure_id {
            let next = match check.verdict.as_str() {
                "fail" => Some(("failing", 3)),
                "pass" if check.dimension == "design" => Some(("installed", 1)),
                "pass" => Some(("verified", 2)),
                _ => None,
            };
            if let Some(next) = next {
                if procedure_state.get(procedure).map_or(true, |(_, rank)| *rank < next.1) {
                    procedure_state.insert(procedure.clone(), next);
                }
            }
        }
    }
    for (criterion, state) in evidence_by_criterion {
        let (status, credit) = status_for_evidence(&state);
        conn.execute(
            "UPDATE criteria SET status=?2, credit=?3, updated_at=?4 WHERE id=?1",
            params![criterion, status, credit, ts],
        )
        .expect("update criterion");
    }
    conn.execute("UPDATE procedures SET status='not_installed', last_checked=?1", [&ts])
        .expect("reset procedures");
    for (procedure, (status, _)) in procedure_state {
        conn.execute(
            "UPDATE procedures SET status=?2, last_checked=?3 WHERE id=?1",
            params![procedure, status, ts],
        )
        .expect("update procedure");
    }
    let gauge: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(weight * credit) * 100.0 / NULLIF(SUM(weight), 0), 0) FROM criteria WHERE in_scope=1",
            [],
            |r| r.get(0),
        )
        .expect("compute gauge");
    let cap = cap_reason.as_ref().map(|_| 79.0);
    conn.execute(
        "INSERT OR REPLACE INTO gauge_history (ts, gauge, cap, cap_reason) VALUES (?1,?2,?3,?4)",
        params![ts, gauge, cap, cap_reason],
    )
    .expect("record gauge");
    let (design, technical, evidence, operating) = readiness_metrics(&conn);
    conn.execute(
        "INSERT OR REPLACE INTO readiness_history (ts, design, technical, evidence, operating) VALUES (?1,?2,?3,?4,?5)",
        params![ts, design, technical, evidence, operating],
    )
    .expect("record readiness dimensions");
    println!("imported schema v{}: {check_count} checks; gauge {gauge:.1}%; design {design:.1}%; technical {technical:.1}%; evidence {evidence:.1}%; operating {operating:.1}%", report.schema_version);
}

// ---------- seeding: the markdown corpus is the source of truth ----------

fn frontmatter_value(fm: &str, key: &str) -> Option<String> {
    fm.lines()
        .find(|l| l.starts_with(&format!("{key}:")))
        .map(|l| l[key.len() + 1..].trim().trim_matches('"').to_string())
}

fn category_key(raw: &str) -> &'static str {
    let r = raw.to_lowercase();
    if r.contains("security") {
        "security"
    } else if r.contains("availability") {
        "availability"
    } else if r.contains("confidentiality") {
        "confidentiality"
    } else if r.contains("processing") {
        "processing_integrity"
    } else {
        "privacy"
    }
}

fn parse_criterion_md(body: &str) -> Option<(String, String, String, i64, String, String, String)> {
    let fm_end = body[3..].find("---")? + 3;
    let fm = &body[3..fm_end];
    let id = frontmatter_value(fm, "id")?;
    let family = frontmatter_value(fm, "family")?;
    let category = category_key(&frontmatter_value(fm, "category")?).to_string();
    let weight: i64 = frontmatter_value(fm, "weight")?.parse().ok()?;
    let automatable = frontmatter_value(fm, "automatable").unwrap_or_else(|| "partial".into());
    let nature = frontmatter_value(fm, "nature").unwrap_or_else(|| "technical".into());
    let after = &body[body.find("## Criterion")?..];
    let text: String = after
        .lines()
        .skip(1)
        .take_while(|l| l.starts_with('>'))
        .map(|l| l.trim_start_matches('>').trim())
        .collect::<Vec<_>>()
        .join(" ");
    Some((id, family, category, weight, text, automatable, nature))
}

fn seed(db_path: &str, criteria_dir: &str, procedures_md: &str) {
    let conn = Connection::open(db_path).expect("open db");
    ensure_schema(&conn);
    // migrate pre-existing DBs; harmless failure if the column already exists
    let _ = conn.execute("ALTER TABLE criteria ADD COLUMN automatable TEXT NOT NULL DEFAULT 'partial'", []);
    let _ = conn.execute("ALTER TABLE criteria ADD COLUMN nature TEXT NOT NULL DEFAULT 'technical'", []);

    let criteria_root = std::fs::canonicalize(criteria_dir).expect("criteria dir");
    if !criteria_root.is_dir() {
        panic!("criteria path is not a directory");
    }
    let mut n = 0;
    for entry in std::fs::read_dir(&criteria_root).expect("criteria dir") {
        let path = entry.expect("entry").path().canonicalize().expect("canonical criterion");
        // Never follow a symlink or traversal outside the configured corpus.
        if !path.starts_with(&criteria_root) {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let body = std::fs::read_to_string(&path).expect("read");
        let Some((id, family, category, weight, text, automatable, nature)) = parse_criterion_md(&body) else {
            eprintln!("skipping unparseable {}", path.display());
            continue;
        };
        // upsert descriptive fields; never clobber live status/credit
        conn.execute(
            "INSERT INTO criteria (id, family, category, text, weight, automatable, nature) VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(id) DO UPDATE SET family=?2, category=?3, text=?4, weight=?5, automatable=?6, nature=?7",
            params![id, family, category, text, weight, automatable, nature],
        )
        .expect("upsert criterion");
        n += 1;
    }
    println!("seeded {n} criteria from {criteria_dir}");

    let md = std::fs::read_to_string(procedures_md).expect("procedures md");
    let mut p = 0;
    for line in md.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() < 6 || cells[0] == "ID" || cells[0].starts_with("---") || cells[0].starts_with(":-") {
            continue;
        }
        let strip = |s: &str| s.replace('`', "");
        conn.execute(
            "INSERT INTO procedures (id, name, category, criteria, install, detect) VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(id) DO UPDATE SET name=?2, category=?3, criteria=?4, install=?5, detect=?6",
            params![strip(cells[0]), cells[1], cells[2], cells[3], cells[4], strip(cells[5])],
        )
        .expect("upsert procedure");
        p += 1;
    }
    println!("seeded {p} procedures from {procedures_md}");
}

// ---------- ingest: what agent/03 POSTs (or writes directly to the db) ----------

#[derive(Deserialize)]
struct Ingest {
    #[serde(default)]
    criteria: Vec<CritUp>,
    #[serde(default)]
    checks: Vec<CheckUp>,
    #[serde(default)]
    procedures: Vec<ProcUp>,
    gauge: Option<GaugeUp>,
    #[serde(default)]
    attestations: Vec<AttUp>,
    #[serde(default)]
    scope: Vec<ScopeUp>,
}
#[derive(Deserialize)]
struct CritUp {
    id: String,
    status: String,
    credit: f64,
    updated_at: Option<String>,
}
#[derive(Deserialize)]
struct CheckUp {
    criterion: String,
    name: String,
    verdict: String,
    evidence: Option<String>,
    last_run: String,
}
#[derive(Deserialize)]
struct ProcUp {
    id: String,
    status: String,
    last_checked: Option<String>,
}
#[derive(Deserialize)]
struct GaugeUp {
    ts: String,
    gauge: f64,
    cap: Option<f64>,
    cap_reason: Option<String>,
}
#[derive(Deserialize)]
struct AttUp {
    criterion: String,
    note: String,
    evidence_link: Option<String>,
    attested_by: String,
    attested_at: String,
    expires_at: Option<String>,
}
#[derive(Deserialize)]
struct ScopeUp {
    category: String,
    in_scope: bool,
}

fn authorized(app: &App, headers: &HeaderMap) -> bool {
    match &app.token {
        None => true, // localhost / Tailscale deployment; set SHADOW_TOKEN to require auth
        Some(t) => headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == format!("Bearer {t}"))
            .unwrap_or(false),
    }
}

async fn ingest(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    if !authorized(&app, &headers) {
        return (StatusCode::UNAUTHORIZED, "bad token").into_response();
    }
    let up: Ingest = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("bad json: {e}")).into_response(),
    };
    let conn = app.db.lock().unwrap();
    for c in &up.criteria {
        conn.execute(
            "UPDATE criteria SET status=?2, credit=?3, updated_at=COALESCE(?4, datetime('now')) WHERE id=?1",
            params![c.id, c.status, c.credit, c.updated_at],
        )
        .ok();
    }
    for ch in &up.checks {
        conn.execute(
            "INSERT INTO checks (criterion_id, name, verdict, evidence, last_run) VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(criterion_id, name) DO UPDATE SET verdict=?3, evidence=?4, last_run=?5",
            params![ch.criterion, ch.name, ch.verdict, ch.evidence, ch.last_run],
        )
        .ok();
    }
    for p in &up.procedures {
        conn.execute(
            "UPDATE procedures SET status=?2, last_checked=COALESCE(?3, datetime('now')) WHERE id=?1",
            params![p.id, p.status, p.last_checked],
        )
        .ok();
    }
    for s in &up.scope {
        conn.execute(
            "UPDATE criteria SET in_scope=?2 WHERE category=?1",
            params![s.category, s.in_scope as i64],
        )
        .ok();
    }
    for a in &up.attestations {
        conn.execute(
            "INSERT INTO attestations (criterion_id, note, evidence_link, attested_by, attested_at, expires_at)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![a.criterion, a.note, a.evidence_link, a.attested_by, a.attested_at, a.expires_at],
        )
        .ok();
    }
    if let Some(g) = &up.gauge {
        conn.execute(
            "INSERT OR REPLACE INTO gauge_history (ts, gauge, cap, cap_reason) VALUES (?1,?2,?3,?4)",
            params![g.ts, g.gauge, g.cap, g.cap_reason],
        )
        .ok();
    }
    StatusCode::NO_CONTENT.into_response()
}

// ---------- read model ----------

fn load_model(conn: &Connection, org: &str) -> render::Model {
    let mut criteria = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, family, category, text, weight, in_scope, status, credit, nature FROM criteria")
            .unwrap();
        let rows = stmt
            .query_map([], |r| {
                Ok(render::Crit {
                    id: r.get(0)?,
                    family: r.get(1)?,
                    category: r.get(2)?,
                    text: r.get(3)?,
                    weight: r.get(4)?,
                    in_scope: r.get::<_, i64>(5)? != 0,
                    status: r.get(6)?,
                    credit: r.get(7)?,
                    nature: r.get(8)?,
                    failing: Vec::new(),
                })
            })
            .unwrap();
        for row in rows {
            criteria.push(row.unwrap());
        }
    }
    {
        let mut stmt = conn
            .prepare("SELECT criterion_id, name FROM checks WHERE verdict='fail' ORDER BY name")
            .unwrap();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).unwrap();
        for row in rows {
            let (cid, name) = row.unwrap();
            if let Some(c) = criteria.iter_mut().find(|c| c.id == cid) {
                c.failing.push(name);
            }
        }
    }

    let mut procedures = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, name, category, criteria, install, detect, status, last_checked FROM procedures ORDER BY rowid")
            .unwrap();
        let rows = stmt
            .query_map([], |r| {
                Ok(render::Proc {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    category: r.get(2)?,
                    criteria: r.get(3)?,
                    install: r.get(4)?,
                    detect: r.get(5)?,
                    status: r.get(6)?,
                    last_checked: r.get(7)?,
                })
            })
            .unwrap();
        for row in rows {
            procedures.push(row.unwrap());
        }
    }

    // authoritative gauge = latest verify run; fallback = recompute from criteria table
    let latest: Option<(String, f64, Option<f64>, Option<String>, f64)> = conn
        .query_row(
            "SELECT ts, gauge, cap, cap_reason, (julianday('now') - julianday(ts)) * 24.0
             FROM gauge_history ORDER BY ts DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .ok();
    let mut history = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT gauge FROM (SELECT ts, gauge FROM gauge_history ORDER BY ts DESC LIMIT 90) ORDER BY ts ASC")
            .unwrap();
        let rows = stmt.query_map([], |r| r.get::<_, f64>(0)).unwrap();
        for row in rows {
            history.push(row.unwrap());
        }
    }
    let computed = {
        let scoped: Vec<&render::Crit> = criteria.iter().filter(|c| c.in_scope).collect();
        let wsum: f64 = scoped.iter().map(|c| c.weight as f64).sum();
        if wsum > 0.0 {
            scoped.iter().map(|c| c.weight as f64 * c.credit).sum::<f64>() / wsum * 100.0
        } else {
            0.0
        }
    };
    let gauge = match latest {
        Some((ts, g, cap, reason, hours)) => render::Gauge {
            value: cap.map_or(g, |c| g.min(c)),
            cap,
            cap_reason: reason,
            ts: Some(ts),
            history,
            stale_hours: Some(hours),
        },
        None => render::Gauge {
            value: computed,
            cap: None,
            cap_reason: None,
            ts: None,
            history,
            stale_hours: None,
        },
    };

    let unknown_checks: i64 = conn
        .query_row("SELECT COUNT(*) FROM checks WHERE verdict='unknown'", [], |r| r.get(0))
        .unwrap_or(0);

    let readiness = conn
        .query_row(
            "SELECT design, technical, evidence, operating FROM readiness_history ORDER BY ts DESC LIMIT 1",
            [],
            |r| Ok(render::Readiness { design: r.get(0)?, technical: r.get(1)?, evidence: r.get(2)?, operating: r.get(3)? }),
        )
        .unwrap_or_else(|_| {
            let (design, technical, evidence, operating) = readiness_metrics(conn);
            render::Readiness { design, technical, evidence, operating }
        });

    render::Model { org: org.to_string(), gauge, readiness, criteria, procedures, unknown_checks }
}

// ---------- routes ----------

async fn index(State(app): State<Arc<App>>) -> Html<String> {
    let conn = app.db.lock().unwrap();
    Html(render::index(&load_model(&conn, &app.org)))
}

async fn micro(
    State(app): State<Arc<App>>,
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let model = {
        let conn = app.db.lock().unwrap();
        load_model(&conn, &app.org)
    };
    let running = app.running.lock().unwrap().clone();
    let runner_ok = !matches!(app.runner, Runner::None);
    Html(render::micro(&model, &running, runner_ok, q.get("err").map(String::as_str)))
}

// Click-to-test: spawn the verifier for one criterion. The site never computes
// compliance — the verifier runs the criterion's check table and POSTs to /ingest.
async fn run_criterion(State(app): State<Arc<App>>, Path(id): Path<String>) -> impl IntoResponse {
    use axum::response::Redirect;
    let exists: bool = {
        let conn = app.db.lock().unwrap();
        conn.query_row("SELECT in_scope FROM criteria WHERE id=?1", [&id], |r| r.get::<_, i64>(0))
            .map(|v| v != 0)
            .unwrap_or(false)
    };
    if !exists {
        return Redirect::to("/micro").into_response();
    }
    if matches!(app.runner, Runner::None) {
        return Redirect::to("/micro?err=norunner").into_response();
    }
    {
        let mut running = app.running.lock().unwrap();
        if !running.insert(id.clone()) {
            return Redirect::to("/micro").into_response(); // already running
        }
    }
    let app2 = app.clone();
    std::thread::spawn(move || {
        let file = format!("{}/{}.md", app2.criteria_dir, id);
        let url = format!("http://localhost:{}", app2.port);
        let status = match &app2.runner {
            Runner::Shell(cmd) => std::process::Command::new("sh")
                .args(["-c", cmd])
                .env("CRITERION", &id)
                .env("CRITERION_FILE", &file)
                .env("SHADOW_URL", &url)
                .status(),
            Runner::Claude => {
                let prompt = format!(
                    "You are the compliance shadow's single-criterion verifier. Criterion: {id}. Read {file} and execute each row of its 'Automated shadow checks' table (skip rows marked MANUAL) using gh / gcloud / file checks. Scope config: shadow/scope.json if present, else infer the org and repo from `gh repo view`. Then POST the results with curl to {url}/ingest as JSON: {{\"checks\":[{{\"criterion\":\"{id}\",\"name\":\"<check>\",\"verdict\":\"pass|fail|unknown\",\"evidence\":\"<trimmed output>\",\"last_run\":\"<utc now>\"}}],\"criteria\":[{{\"id\":\"{id}\",\"status\":\"verified|implemented|in_progress|failing\",\"credit\":1.0}}]}}. Credit rules: all checks pass and evidence fresh = verified/1.0; controls exist, evidence partial = implemented/0.6; some pass = in_progress/0.25; failures = failing/0.0. unknown is never treated as pass. Do NOT write a gauge entry (single-criterion runs must not move the official gauge). Be quick; no commentary."
                );
                std::process::Command::new("claude")
                    .args(["-p", &prompt, "--allowedTools", "Bash,Read,Glob,Grep", "--max-turns", "40"])
                    .status()
            }
            Runner::None => unreachable!(),
        };
        if let Err(e) = status {
            eprintln!("runner for {id} failed to start: {e}");
        }
        app2.running.lock().unwrap().remove(&id);
    });
    Redirect::to("/micro").into_response()
}

async fn criterion(State(app): State<Arc<App>>, Path(id): Path<String>) -> impl IntoResponse {
    let conn = app.db.lock().unwrap();
    let model = load_model(&conn, &app.org);
    let Some(c) = model.criteria.iter().find(|c| c.id == id) else {
        return (StatusCode::NOT_FOUND, "no such criterion").into_response();
    };
    let mut checks = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT name, verdict, evidence, last_run, dimension, source, expires_at FROM checks WHERE criterion_id=?1 ORDER BY name")
            .unwrap();
        let rows = stmt
            .query_map([&id], |r| {
                Ok(render::CheckRow { name: r.get(0)?, verdict: r.get(1)?, evidence: r.get(2)?, last_run: r.get(3)?, dimension: r.get(4)?, source: r.get(5)?, expires_at: r.get(6)? })
            })
            .unwrap();
        for row in rows {
            checks.push(row.unwrap());
        }
    }
    let mut atts = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT note, evidence_link, attested_by, attested_at, expires_at FROM attestations WHERE criterion_id=?1 ORDER BY attested_at DESC")
            .unwrap();
        let rows = stmt
            .query_map([&id], |r| {
                Ok(render::Attestation {
                    note: r.get(0)?,
                    evidence_link: r.get(1)?,
                    attested_by: r.get(2)?,
                    attested_at: r.get(3)?,
                    expires_at: r.get(4)?,
                })
            })
            .unwrap();
        for row in rows {
            atts.push(row.unwrap());
        }
    }
    Html(render::detail(c, &checks, &atts)).into_response()
}

async fn export_db(State(app): State<Arc<App>>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&app, &headers) {
        return (StatusCode::UNAUTHORIZED, "bad token").into_response();
    }
    match std::fs::read(&app.db_path) {
        Ok(bytes) => (
            [
                ("content-type", "application/octet-stream".to_string()),
                ("content-disposition", "attachment; filename=\"shadow.db\"".to_string()),
            ],
            bytes,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn serve(db_path: String, port: u16) {
    let conn = Connection::open(&db_path).expect("open db");
    ensure_schema(&conn);
    let runner = match std::env::var("SHADOW_RUNNER") {
        Ok(cmd) if !cmd.is_empty() => Runner::Shell(cmd),
        _ => {
            let has_claude = std::process::Command::new("claude")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if has_claude { Runner::Claude } else { Runner::None }
        }
    };
    let app = Arc::new(App {
        db: Mutex::new(conn),
        db_path,
        token: std::env::var("SHADOW_TOKEN").ok(),
        org: std::env::var("SHADOW_ORG").unwrap_or_else(|_| "unnamed org".into()),
        running: Mutex::new(std::collections::HashSet::new()),
        runner,
        criteria_dir: std::env::var("SHADOW_CRITERIA_DIR").unwrap_or_else(|_| "../../criteria".into()),
        port,
    });
    let router = Router::new()
        .route("/", get(index))
        .route("/micro", get(micro))
        .route("/run/{id}", post(run_criterion))
        .route("/criteria/{id}", get(criterion))
        .route("/ingest", post(ingest))
        .route("/db", get(export_db))
        .with_state(app);
    let rt = tokio::runtime::Runtime::new().expect("tokio");
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("bind");
        println!("shadow listening on http://localhost:{port}");
        axum::serve(listener, router).await.expect("serve");
    });
}

// Static export: the same pages the server renders, written to disk.
// The site renders state; a static render of the same state is byte-equivalent
// evidence — publishable as an Actions artifact or (opt-in) GitHub Pages.
fn render_static(db_path: &str, out: &str) {
    let conn = Connection::open(db_path).expect("open db");
    ensure_schema(&conn);
    let org = std::env::var("SHADOW_ORG").unwrap_or_else(|_| "unnamed org".into());
    let model = load_model(&conn, &org);

    std::fs::create_dir_all(format!("{out}/criteria")).expect("mkdir");

    // index: rewrite app routes to static paths
    let mut index = render::index(&model);
    for c in &model.criteria {
        index = index.replace(
            &format!("href=\"/criteria/{}\"", c.id),
            &format!("href=\"criteria/{}.html\"", c.id),
        );
    }
    index = index.replace("<a href=\"/db\">export shadow.db</a>", "static render — served copy has /db export");
    std::fs::write(format!("{out}/index.html"), index).expect("write index");

    for c in &model.criteria {
        let mut checks = Vec::new();
        let mut stmt = conn
            .prepare("SELECT name, verdict, evidence, last_run, dimension, source, expires_at FROM checks WHERE criterion_id=?1 ORDER BY name")
            .unwrap();
        let rows = stmt
            .query_map([&c.id], |r| {
                Ok(render::CheckRow { name: r.get(0)?, verdict: r.get(1)?, evidence: r.get(2)?, last_run: r.get(3)?, dimension: r.get(4)?, source: r.get(5)?, expires_at: r.get(6)? })
            })
            .unwrap();
        for row in rows {
            checks.push(row.unwrap());
        }
        let mut atts = Vec::new();
        let mut stmt = conn
            .prepare("SELECT note, evidence_link, attested_by, attested_at, expires_at FROM attestations WHERE criterion_id=?1 ORDER BY attested_at DESC")
            .unwrap();
        let rows = stmt
            .query_map([&c.id], |r| {
                Ok(render::Attestation {
                    note: r.get(0)?,
                    evidence_link: r.get(1)?,
                    attested_by: r.get(2)?,
                    attested_at: r.get(3)?,
                    expires_at: r.get(4)?,
                })
            })
            .unwrap();
        for row in rows {
            atts.push(row.unwrap());
        }
        let page = render::detail(c, &checks, &atts).replace("href=\"/\"", "href=\"../index.html\"");
        std::fs::write(format!("{out}/criteria/{}.html", c.id), page).expect("write detail");
    }
    println!("rendered {out}/index.html + {} criteria pages", model.criteria.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_checks_map_only_to_owned_criteria() {
        assert_eq!(criteria_for_legacy_verify_check("github.org_2fa_required"), ["CC6.1", "CC6.2"]);
        assert_eq!(criteria_for_legacy_verify_check("github.branch_protection.main"), ["CC8.1"]);
        assert!(criteria_for_legacy_verify_check("unknown").is_empty());
    }

    #[test]
    fn evidence_credit_is_conservative() {
        assert_eq!(status_for_evidence(&CriterionEvidence::default()), ("not_started", 0.0));
        let design = CriterionEvidence { pass_design: true, ..CriterionEvidence::default() };
        assert_eq!(status_for_evidence(&design), ("implemented", 0.6));
        let operating = CriterionEvidence { pass_operating: true, ..CriterionEvidence::default() };
        assert_eq!(status_for_evidence(&operating), ("verified", 1.0));
        let conflict = CriterionEvidence { pass_operating: true, failed: true, ..CriterionEvidence::default() };
        assert_eq!(status_for_evidence(&conflict), ("failing", 0.0));
    }
}
