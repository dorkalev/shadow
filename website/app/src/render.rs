// Server-side rendering: the auditor's working paper. No client JS.
use std::fmt::Write as _;

pub struct Crit {
    pub id: String,
    pub family: String,
    pub category: String,
    pub text: String,
    pub weight: i64,
    pub in_scope: bool,
    pub status: String,
    pub credit: f64,
    pub nature: String, // technical | document
    pub failing: Vec<String>,
}

pub struct Proc {
    pub id: String,
    pub name: String,
    pub category: String,
    pub criteria: String,
    pub install: String,
    pub detect: String,
    pub status: String,
    pub last_checked: Option<String>,
}

pub struct Gauge {
    pub value: f64,
    pub cap: Option<f64>,
    pub cap_reason: Option<String>,
    pub ts: Option<String>,
    pub history: Vec<f64>,
    pub stale_hours: Option<f64>,
}

pub struct Model {
    pub org: String,
    pub gauge: Gauge,
    pub readiness: Readiness,
    pub criteria: Vec<Crit>,
    pub procedures: Vec<Proc>,
    pub unknown_checks: i64,
}

pub struct Readiness {
    pub design: f64,
    pub technical: f64,
    pub evidence: f64,
    pub operating: f64,
}

pub struct CheckRow {
    pub name: String,
    pub verdict: String,
    pub evidence: Option<String>,
    pub last_run: String,
    pub dimension: String,
    pub source: Option<String>,
    pub expires_at: Option<String>,
}

pub struct Attestation {
    pub note: String,
    pub evidence_link: Option<String>,
    pub attested_by: String,
    pub attested_at: String,
    pub expires_at: Option<String>,
}

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

const CATEGORIES: [(&str, &str); 5] = [
    ("security", "Security"),
    ("availability", "Availability"),
    ("confidentiality", "Confidentiality"),
    ("processing_integrity", "Processing Integrity"),
    ("privacy", "Privacy"),
];

fn cat_rank(c: &str) -> usize {
    CATEGORIES.iter().position(|(k, _)| *k == c).unwrap_or(9)
}

fn crit_glyph(status: &str) -> (&'static str, &'static str) {
    match status {
        "verified" => ("\u{2611}", "ok"),      // ☑
        "implemented" => ("\u{25D5}", "mid"),  // ◕
        "in_progress" => ("\u{25D4}", "mid"),  // ◔
        "failing" => ("\u{2612}", "bad"),      // ☒
        _ => ("\u{2610}", "dim"),              // ☐
    }
}

// technical control (evidence is system state) vs document-only (evidence is paper)
fn nature_glyph(nature: &str) -> (&'static str, &'static str) {
    if nature == "document" {
        ("\u{00B6}", "document-only")
    } else {
        ("\u{2699}\u{fe0e}", "technical")
    }
}

fn proc_glyph(status: &str) -> (&'static str, &'static str) {
    match status {
        "verified" => ("●", "ok"),      // ●
        "installed" => ("◐", "mid"),    // ◐
        "failing" => ("✕", "bad"),      // ✕
        _ => ("○", "dim"),              // ○
    }
}

// Point on the dial. pct 0 → left (180°), pct 100 → right (0°). SVG y-down.
fn dial_pt(pct: f64, r: f64) -> (f64, f64) {
    let th = std::f64::consts::PI * (1.0 - pct / 100.0);
    (200.0 + r * th.cos(), 200.0 - r * th.sin())
}

fn arc(from: f64, to: f64, r: f64) -> String {
    let (x1, y1) = dial_pt(from, r);
    let (x2, y2) = dial_pt(to, r);
    format!("M {x1:.1} {y1:.1} A {r:.1} {r:.1} 0 0 1 {x2:.1} {y2:.1}")
}

fn gauge_svg(g: &Gauge) -> String {
    let mut s = String::new();
    s.push_str(r#"<svg viewBox="0 0 400 232" class="dial" role="img" aria-label="compliance gauge">"#);
    s.push_str(r##"<defs><pattern id="hatch" width="7" height="7" patternTransform="rotate(45)" patternUnits="userSpaceOnUse"><line x1="0" y1="0" x2="0" y2="7" stroke="#9e2b25" stroke-width="2.2"/></pattern></defs>"##);
    // colour bands
    for (a, b, c) in [
        (0.0, 50.0, "#b3542e"),
        (50.0, 80.0, "#b07d10"),
        (80.0, 95.0, "#2c6e49"),
        (95.0, 100.0, "#1b4d3e"),
    ] {
        let _ = write!(s, r#"<path d="{}" fill="none" stroke="{}" stroke-width="15" opacity="0.85"/>"#, arc(a, b, 150.0), c);
    }
    // hard-gate cap: hatch out the unreachable region
    if let Some(cap) = g.cap {
        let _ = write!(s, r#"<path d="{}" fill="none" stroke="url(#hatch)" stroke-width="19"/>"#, arc(cap, 100.0, 150.0));
    }
    // ticks + labels
    for i in 0..=20 {
        let pct = i as f64 * 5.0;
        let major = i % 4 == 0;
        let (r1, r2) = if major { (126.0, 139.0) } else { (132.0, 139.0) };
        let (x1, y1) = dial_pt(pct, r1);
        let (x2, y2) = dial_pt(pct, r2);
        let _ = write!(s, r##"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="#211c14" stroke-width="{}" opacity="0.7"/>"##, if major { 1.6 } else { 0.8 });
        if major {
            let (tx, ty) = dial_pt(pct, 110.0);
            let _ = write!(s, r##"<text x="{tx:.1}" y="{:.1}" class="tick">{}</text>"##, ty + 3.0, pct as i64);
        }
    }
    // needle, drawn at 0% and rotated into place by CSS (sweep animation)
    let deg = g.value.clamp(0.0, 100.0) * 1.8;
    let _ = write!(s, r##"<g id="needle" style="transform-origin:200px 200px;transform:rotate({deg:.2}deg)"><line x1="200" y1="200" x2="76" y2="200" stroke="#211c14" stroke-width="2.6"/><line x1="200" y1="200" x2="224" y2="200" stroke="#211c14" stroke-width="5"/><circle cx="200" cy="200" r="7.5" fill="#211c14"/><circle cx="200" cy="200" r="2.6" fill="#f7f3e8"/></g>"##);
    s.push_str("</svg>");
    s
}

fn sparkline(history: &[f64]) -> String {
    if history.len() < 2 {
        return String::new();
    }
    let pts: Vec<String> = history
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = 4.0 + (i as f64 / (history.len() - 1) as f64) * 212.0;
            let y = 34.0 - (v / 100.0) * 30.0;
            format!("{x:.1},{y:.1}")
        })
        .collect();
    format!(
        r##"<svg viewBox="0 0 220 38" class="spark" aria-label="gauge trend"><polyline points="{}" fill="none" stroke="#2c6e49" stroke-width="1.8"/></svg>"##,
        pts.join(" ")
    )
}

fn category_chips(m: &Model) -> String {
    let mut s = String::from(r#"<div class="chips">"#);
    for (key, label) in CATEGORIES {
        let of_cat: Vec<&Crit> = m.criteria.iter().filter(|c| c.category == key).collect();
        let scoped: Vec<&&Crit> = of_cat.iter().filter(|c| c.in_scope).collect();
        if scoped.is_empty() {
            let _ = write!(s, r#"<div class="chip out"><span class="chip-name">{label}</span><span class="chip-score">not in scope</span><span class="chip-n">{} criteria</span></div>"#, of_cat.len());
        } else {
            let wsum: f64 = scoped.iter().map(|c| c.weight as f64).sum();
            let score = if wsum > 0.0 {
                scoped.iter().map(|c| c.weight as f64 * c.credit).sum::<f64>() / wsum * 100.0
            } else {
                0.0
            };
            let _ = write!(s, r#"<div class="chip"><span class="chip-name">{label}</span><span class="chip-score">{score:.0}%</span><span class="chip-n">{} of {} in scope</span></div>"#, scoped.len(), of_cat.len());
        }
    }
    s.push_str("</div>");
    s
}

fn readiness_cards(r: &Readiness) -> String {
    let items = [
        ("Design readiness", r.design, "documented controls"),
        ("Technical health", r.technical, "live automated checks"),
        ("Evidence coverage", r.evidence, "criteria with current proof"),
        ("Operating maturity", r.operating, "controls proven in operation"),
    ];
    let mut s = String::from(r#"<div class="readiness" aria-label="readiness dimensions">"#);
    for (label, value, note) in items {
        let _ = write!(
            s,
            r#"<div class="rmetric"><span class="rvalue">{value:.1}%</span><span class="rlabel">{label}</span><span class="rnote">{note}</span></div>"#
        );
    }
    s.push_str("</div>");
    s
}

// ---------- the workflow map: the SDLC as territory, procedures as pins ----------

enum MapItem {
    Group {
        label: &'static str,
        category: Option<&'static str>, // dim the segment when this TSC category is out of scope
    },
    Station {
        label: &'static str,
        note: &'static str,
        crit: &'static str,
        pins: &'static [&'static str],
    },
}

// The whole program as one territory. The Loop is the center of gravity:
// every side process (incidents, risk, people, vendors) funnels back into it
// as tickets, and everything deposits into the same evidence store.
// COVERAGE INVARIANT: every one of the 61 TSC criteria appears in some station's
// `crit` tag — enforced by the map_covers_all_criteria test below.
const MAP: &[MapItem] = &[
    MapItem::Group { label: "The Perimeter — always on", category: None },
    MapItem::Station {
        label: "Identity wall",
        note: "one door: MFA everywhere, least privilege by default",
        crit: "CC6.1 · CC6.6",
        pins: &["org-2fa", "org-base-perms", "workspace-mfa"],
    },
    MapItem::Station {
        label: "The charter",
        note: "code of conduct, roles + reporting lines, accountability; the policy pack everyone attests to",
        crit: "CC1.1 · CC1.3 · CC1.5 · CC2.2",
        pins: &["policies-repo"],
    },
    MapItem::Station {
        label: "The front door",
        note: "security.txt, status page, subprocessor list, support inbox — commitments communicated outward",
        crit: "CC2.3",
        pins: &[],
    },
    MapItem::Group { label: "The Loop — every change funnels through here", category: None },
    MapItem::Station {
        label: "Ticket",
        note: "no change without intent on record; agents get a ticket, not a vibe",
        crit: "CC8.1",
        pins: &["ai-policy"],
    },
    MapItem::Station {
        label: "Branch + draft PR",
        note: "{TICKET-ID}-slug off staging; the PR is the audit artifact from minute one",
        crit: "CC8.1",
        pins: &["staging-topology", "pr-template"],
    },
    MapItem::Station {
        label: "Spec on ticket",
        note: "the plan is approved before substantive code lands",
        crit: "CC8.1",
        pins: &[],
    },
    MapItem::Station {
        label: "Implement + tests",
        note: "scanners watch every line as it is written",
        crit: "CC7.1 · CC6.8",
        pins: &["dependabot", "secret-scanning", "codeql"],
    },
    MapItem::Station {
        label: "Gates",
        note: "required checks — merging is impossible until all are green; the general controls over technology, deployed",
        crit: "CC8.1 · CC4.1 · CC5.1 · CC5.2",
        pins: &["ci-tests", "review-bot", "compliance-audit-agent", "compliance-review-gate"],
    },
    MapItem::Station {
        label: "Merge to staging",
        note: "protected; a bypass is never forbidden, only detected and billed",
        crit: "CC8.1 · CC6.3",
        pins: &["branch-rulesets"],
    },
    MapItem::Station {
        label: "Archive",
        note: "JSON+MD evidence per merged PR, on its own branch, forever",
        crit: "CC8.1 · CC2.1",
        pins: &["post-merge-archive", "bypass-detection", "archives-branch", "slack-webhook"],
    },
    MapItem::Station {
        label: "Release → main",
        note: "fast-forward only, human-confirmed, with a release record and ticket",
        crit: "CC8.1",
        pins: &[],
    },
    MapItem::Station {
        label: "Hotfix (emergency valve)",
        note: "direct push to main is allowed but billed: incident ticket + backport PR back through the gates; an undocumented bypass is the loudest alarm the shadow has",
        crit: "CC8.1 · CC7.4",
        pins: &["hotfix-runbook"],
    },
    MapItem::Group { label: "The Sirens — when something breaks", category: None },
    MapItem::Station {
        label: "Detect",
        note: "telemetry, alerts, audit trails, and capacity watch production around the clock",
        crit: "CC7.2 · A1.1",
        pins: &["uptime-alerting", "error-monitoring", "audit-logging"],
    },
    MapItem::Station {
        label: "Respond",
        note: "triage → incident ticket → contain, remediate; the fix re-enters the Loop as a ticketed change",
        crit: "CC7.3 · CC7.4",
        pins: &["incident-runbook"],
    },
    MapItem::Station {
        label: "Learn",
        note: "postmortem filed as evidence; recovery rehearsed in the annual tabletop",
        crit: "CC7.5",
        pins: &[],
    },
    MapItem::Group { label: "The People — joiners, movers, leavers", category: None },
    MapItem::Station {
        label: "Join",
        note: "checklist ticket: least-privilege grants, MFA verified, policy attestation within 2 weeks",
        crit: "CC6.2 · CC1.4",
        pins: &["onboard-offboard"],
    },
    MapItem::Station {
        label: "Recertify",
        note: "quarterly: every grant on every system reviewed against the register, signed",
        crit: "CC6.2 · CC6.3",
        pins: &["quarterly-access-review", "access-register"],
    },
    MapItem::Station {
        label: "Leave",
        note: "same-day revocation, enumerated in the offboarding ticket — the auditor's favorite sample",
        crit: "CC6.2",
        pins: &[],
    },
    MapItem::Group { label: "The Custody — data through its life", category: None },
    MapItem::Station {
        label: "Classify + protect",
        note: "data classification policy; TLS in transit, encryption at rest, scoped LLM-vendor flows",
        crit: "C1.1 · CC6.7",
        pins: &[],
    },
    MapItem::Station {
        label: "Back up",
        note: "automated, point-in-time recovery on the primary datastore",
        crit: "A1.2",
        pins: &["backups-pitr"],
    },
    MapItem::Station {
        label: "Prove restore",
        note: "a backup that was never restored is a hope, not a control",
        crit: "A1.3",
        pins: &["restore-test"],
    },
    MapItem::Station {
        label: "Dispose",
        note: "retention schedule enforced; end-of-life erasure logged",
        crit: "C1.2 · CC6.5",
        pins: &[],
    },
    MapItem::Group { label: "The Counterparties — vendors & partners", category: None },
    MapItem::Station {
        label: "Vendor lifecycle",
        note: "assess → DPA + breach-notice commitments → approve → re-review annually (LLM providers included)",
        crit: "CC9.2 · P6.4 · P6.5",
        pins: &["vendor-register"],
    },
    MapItem::Station {
        label: "Inherited from the cloud",
        note: "physical security + environmental protections are the provider's controls (CSOC) — we carry their SOC 2, reviewed at vendor re-review",
        crit: "CC6.4",
        pins: &[],
    },
    MapItem::Group { label: "The Risk — what could hurt us", category: None },
    MapItem::Station {
        label: "Risk loop",
        note: "assess annually → treat via tickets into the Loop → the shadow watches drift daily",
        crit: "CC3.1–CC3.4 · CC9.1",
        pins: &["risk-register"],
    },
    MapItem::Group { label: "The Product — processing integrity", category: Some("processing_integrity") },
    MapItem::Station {
        label: "Processing lifecycle",
        note: "inputs → processing → outputs → storage: definitions published, inputs validated for completeness + accuracy, outputs delivered and stored to spec",
        crit: "PI1.1 · PI1.2 · PI1.3 · PI1.4 · PI1.5",
        pins: &[],
    },
    MapItem::Group { label: "The Subjects — privacy", category: Some("privacy") },
    MapItem::Station {
        label: "Notice + consent",
        note: "privacy notice current and versioned; consent obtained before collection, for the stated purpose only",
        crit: "P1.1 · P2.1 · P3.1 · P3.2",
        pins: &[],
    },
    MapItem::Station {
        label: "Use, retain, dispose",
        note: "use limited to stated purposes; retention schedule enforced; secure disposal logged",
        crit: "P4.1 · P4.2 · P4.3",
        pins: &[],
    },
    MapItem::Station {
        label: "Access + correction",
        note: "DSARs tracked as tickets: subjects can see, copy, and correct their data — denials explained",
        crit: "P5.1 · P5.2",
        pins: &[],
    },
    MapItem::Station {
        label: "Disclosure records",
        note: "consent before disclosure; a log of every authorized and unauthorized disclosure (breaches included); subjects and regulators notified",
        crit: "P6.1 · P6.2 · P6.3 · P6.6 · P6.7",
        pins: &[],
    },
    MapItem::Station {
        label: "Quality + complaints",
        note: "personal data kept accurate; privacy@ inbox with tracked resolution",
        crit: "P7.1 · P8.1",
        pins: &[],
    },
    MapItem::Group { label: "The Clock — the calendar that drives it all", category: None },
    MapItem::Station {
        label: "Daily",
        note: "the shadow re-audits every process above and files regressions as tickets",
        crit: "CC4.1",
        pins: &["daily-verify"],
    },
    MapItem::Station {
        label: "Quarterly",
        note: "management review: gauge trend, incidents, exceptions — minutes filed",
        crit: "CC1.2 · CC4.2",
        pins: &["quarterly-mgmt-review"],
    },
    MapItem::Station {
        label: "Annually",
        note: "risk refresh, policy re-approval + staff attestation, restore test, incident tabletop",
        crit: "CC3.4 · CC5.3",
        pins: &["annual-rituals"],
    },
];

// Split "The Loop — subtitle" into (name, subtitle)
fn split_label(label: &str) -> (&str, &str) {
    match label.split_once(" — ") {
        Some((a, b)) => (a, b),
        None => (label, ""),
    }
}

fn render_pin(s: &mut String, id: &str, by_id: &std::collections::HashMap<&str, &Proc>) {
    let Some(p) = by_id.get(id) else {
        let (g, cls) = proc_glyph("not_installed");
        let _ = write!(s, r#"<span class="pin {cls}">{g}&nbsp;{}</span>"#, esc(id));
        return;
    };
    let (g, cls) = proc_glyph(&p.status);
    let nature = if p.category == "paper" || p.category == "cadence" { "\u{00B6} document-only" } else { "\u{2699}\u{fe0e} technical" };
    let _ = write!(
        s,
        r#"<span class="pin {cls}">{g}&nbsp;{}<span class="pop"><span class="pop-head">{} · {} · {} · {nature}</span><span class="pop-text">{}</span><span class="pop-meta">detect: {}</span><span class="pop-meta">serves {} · {} · last checked {}</span></span></span>"#,
        esc(id),
        esc(id),
        esc(&p.category),
        esc(&p.status),
        esc(&p.name),
        esc(&p.detect),
        esc(&p.criteria),
        esc(&p.install),
        esc(p.last_checked.as_deref().unwrap_or("never")),
    );
}

fn machinery_cards(m: &Model) -> String {
    let by_id: std::collections::HashMap<&str, &Proc> =
        m.procedures.iter().map(|p| (p.id.as_str(), p)).collect();
    let pinned: std::collections::HashSet<&str> = MAP
        .iter()
        .filter_map(|i| match i {
            MapItem::Station { pins, .. } => Some(pins.iter().copied()),
            _ => None,
        })
        .flatten()
        .collect();
    let extras: Vec<&str> = m
        .procedures
        .iter()
        .filter(|p| !pinned.contains(p.id.as_str()))
        .map(|p| p.id.as_str())
        .collect();
    // a category with zero in-scope criteria dims its whole card
    let scoped_cats: std::collections::HashSet<&str> =
        m.criteria.iter().filter(|c| c.in_scope).map(|c| c.category.as_str()).collect();

    let done = m.procedures.iter().filter(|p| p.status == "verified").count();
    let mut s = String::new();
    let _ = write!(
        s,
        r#"<section class="sect" style="animation-delay:.15s"><h2><span class="roman">II.</span> The Machinery <span class="sect-note">the program as territory · {done} of {} procedures verified · hover anything</span></h2><div class="cards">"#,
        m.procedures.len()
    );

    let mut open = false;
    for item in MAP {
        match item {
            MapItem::Group { label, category } => {
                if open {
                    s.push_str("</div>");
                }
                let dim = category.map_or(false, |c| !scoped_cats.contains(c));
                let (name, sub) = split_label(label);
                let oos = if dim { r#"<span class="card-oos">not in scope</span>"# } else { "" };
                let _ = write!(
                    s,
                    r#"<div class="card{}"><div class="card-h">{}{oos}<span class="card-sub">{}</span></div>"#,
                    if dim { " out" } else { "" },
                    esc(name),
                    esc(sub)
                );
                open = true;
            }
            MapItem::Station { label, note, crit, pins } => {
                let _ = write!(
                    s,
                    r#"<div class="st"><span class="st-name">{}</span><span class="st-pins">"#,
                    esc(label)
                );
                if pins.is_empty() {
                    s.push_str(r#"<span class="st-none">procedural</span>"#);
                } else {
                    for id in *pins {
                        render_pin(&mut s, id, &by_id);
                    }
                }
                let _ = write!(
                    s,
                    r#"</span><span class="pop st-pop"><span class="pop-head">{} · {}</span><span class="pop-text">{}</span></span></div>"#,
                    esc(label),
                    esc(crit),
                    esc(note)
                );
            }
        }
    }
    if !extras.is_empty() {
        if open {
            s.push_str("</div>");
            open = false;
        }
        s.push_str(r#"<div class="card"><div class="card-h">Unpinned<span class="card-sub">not yet placed on the map</span></div><div class="st"><span class="st-pins">"#);
        for id in &extras {
            render_pin(&mut s, id, &by_id);
        }
        s.push_str("</span></div></div>");
    }
    if open {
        s.push_str("</div>");
    }
    s.push_str(r#"</div><div class="legend"><span class="pin ok">● verified</span><span class="pin mid">◐ installed</span><span class="pin bad">✕ failing</span><span class="pin dim">○ not installed</span></div></section>"#);
    s
}


// Two-word essence of each criterion — shown on the matrix cards.
// COVERAGE INVARIANT: every criterion has exactly one two-word label
// (enforced by the labels_cover_all_criteria test below).
const LABELS: &[(&str, &str)] = &[
    ("CC1.1", "Ethical Values"),
    ("CC1.2", "Board Oversight"),
    ("CC1.3", "Reporting Lines"),
    ("CC1.4", "Competent People"),
    ("CC1.5", "Accountability Enforced"),
    ("CC2.1", "Quality Information"),
    ("CC2.2", "Internal Communication"),
    ("CC2.3", "External Communication"),
    ("CC3.1", "Clear Objectives"),
    ("CC3.2", "Risk Identification"),
    ("CC3.3", "Fraud Consideration"),
    ("CC3.4", "Change Assessment"),
    ("CC4.1", "Ongoing Monitoring"),
    ("CC4.2", "Deficiency Remediation"),
    ("CC5.1", "Control Selection"),
    ("CC5.2", "Technology Controls"),
    ("CC5.3", "Policy Deployment"),
    ("CC6.1", "Access Security"),
    ("CC6.2", "User Provisioning"),
    ("CC6.3", "Least Privilege"),
    ("CC6.4", "Physical Access"),
    ("CC6.5", "Asset Disposal"),
    ("CC6.6", "Boundary Protection"),
    ("CC6.7", "Transmission Protection"),
    ("CC6.8", "Malware Prevention"),
    ("CC7.1", "Vulnerability Detection"),
    ("CC7.2", "Anomaly Monitoring"),
    ("CC7.3", "Incident Evaluation"),
    ("CC7.4", "Incident Response"),
    ("CC7.5", "Incident Recovery"),
    ("CC8.1", "Change Management"),
    ("CC9.1", "Disruption Mitigation"),
    ("CC9.2", "Vendor Management"),
    ("A1.1", "Capacity Planning"),
    ("A1.2", "Recovery Infrastructure"),
    ("A1.3", "Recovery Testing"),
    ("C1.1", "Confidential Identification"),
    ("C1.2", "Confidential Disposal"),
    ("PI1.1", "Data Definitions"),
    ("PI1.2", "Input Accuracy"),
    ("PI1.3", "Processing Controls"),
    ("PI1.4", "Output Delivery"),
    ("PI1.5", "Storage Integrity"),
    ("P1.1", "Privacy Notice"),
    ("P2.1", "Consent Choices"),
    ("P3.1", "Limited Collection"),
    ("P3.2", "Explicit Consent"),
    ("P4.1", "Limited Use"),
    ("P4.2", "Data Retention"),
    ("P4.3", "Secure Disposal"),
    ("P5.1", "Subject Access"),
    ("P5.2", "Data Correction"),
    ("P6.1", "Consented Disclosure"),
    ("P6.2", "Disclosure Records"),
    ("P6.3", "Breach Records"),
    ("P6.4", "Vendor Commitments"),
    ("P6.5", "Vendor Notification"),
    ("P6.6", "Breach Notification"),
    ("P6.7", "Disclosure Accounting"),
    ("P7.1", "Data Quality"),
    ("P8.1", "Complaint Handling"),
];

fn label_for(id: &str) -> &'static str {
    LABELS.iter().find(|(k, _)| *k == id).map(|(_, v)| *v).unwrap_or("")
}

fn criteria_matrix(m: &Model) -> String {
    let mut crits: Vec<&Crit> = m.criteria.iter().collect();
    crits.sort_by(|a, b| (cat_rank(&a.category), &a.id).cmp(&(cat_rank(&b.category), &b.id)));
    let in_scope = crits.iter().filter(|c| c.in_scope).count();
    let verified = crits.iter().filter(|c| c.in_scope && c.status == "verified").count();
    let failing = crits.iter().filter(|c| c.in_scope && c.status == "failing").count();

    let mut s = String::new();
    let _ = write!(
        s,
        r#"<section class="sect" style="animation-delay:.3s"><h2><span class="roman">III.</span> The Criteria <span class="sect-note">TSP §100 · {in_scope} in scope · {verified} verified · {failing} failing · ⚙︎ technical / ¶ document-only — hover for the verbatim criterion</span></h2><div class="matrix">"#
    );
    for c in &crits {
        let cls = if !c.in_scope {
            "out"
        } else {
            match c.status.as_str() {
                "verified" => "ok",
                "implemented" | "in_progress" => "mid",
                "failing" => "bad",
                _ => "todo",
            }
        };
        let status_txt = if c.in_scope { c.status.replace('_', " ") } else { "out of scope".into() };
        let cat = CATEGORIES
            .iter()
            .find(|(k, _)| *k == c.category)
            .map(|(_, l)| *l)
            .unwrap_or(c.category.as_str());
        let (nat_glyph, nat_name) = nature_glyph(&c.nature);
        let _ = write!(
            s,
            r#"<a class="ccard {cls}" href="/criteria/{id}"><span class="ccid">{id}<span class="cicon" aria-label="{nat}">{ng}</span></span><span class="clabel">{label}</span><span class="ctag">{cat}</span><span class="pop"><span class="pop-head">{id} · {cat} · {status} · {ng} {nat}</span><span class="pop-text">{text}</span></span></a>"#,
            id = esc(&c.id),
            ng = nat_glyph,
            nat = nat_name,
            label = label_for(&c.id),
            cat = esc(cat),
            status = esc(&status_txt),
            text = esc(&c.text),
        );
    }
    s.push_str("</div></section>");
    s
}

const CSS: &str = r#"
:root{--paper:#ddd2b6;--sheet:#f6f1e4;--ink:#211c14;--faint:#6f6754;--rule:#d5cab0;--red:#9e2b25;--green:#2c6e49;--deep:#1b4d3e;--amber:#b07d10;--rust:#b3542e}
*{box-sizing:border-box;margin:0}
html{background:var(--paper)}
body{font-family:"Fraunces",Georgia,serif;color:var(--ink);
  background:
    radial-gradient(ellipse at 18% -8%, rgba(255,255,255,.5), transparent 55%),
    repeating-linear-gradient(0deg, transparent 0 3px, rgba(33,28,20,.012) 3px 4px),
    var(--paper);
  padding:2.4rem 1rem 5rem}
.mono{font-family:"IBM Plex Mono",ui-monospace,monospace}
.sheet{max-width:1120px;margin:0 auto;background:var(--sheet);border:1px solid var(--rule);
  box-shadow:0 1px 2px rgba(59,48,28,.14),0 24px 60px -20px rgba(59,48,28,.5);
  padding:52px 60px 60px 108px;position:relative}
.sheet::before{content:"";position:absolute;top:0;bottom:0;left:72px;width:1px;background:rgba(158,43,37,.4)}
.sheet::after{content:"";position:absolute;inset:0;pointer-events:none;
  background:repeating-linear-gradient(90deg, transparent 0 140px, rgba(33,28,20,.006) 140px 141px)}
header{display:flex;justify-content:space-between;align-items:baseline;border-bottom:2px solid var(--ink);padding-bottom:14px}
.kicker{font-family:"IBM Plex Mono",monospace;font-size:11px;letter-spacing:.22em;text-transform:uppercase;color:var(--faint)}
h1{font-size:44px;font-weight:600;letter-spacing:-.01em;margin-top:6px;font-variation-settings:"opsz" 60}
h1 .org{font-style:italic;font-weight:400;color:var(--faint)}
.hd-right{text-align:right;font-family:"IBM Plex Mono",monospace;font-size:11px;color:var(--faint);letter-spacing:.14em;text-transform:uppercase;line-height:2}
.banner{margin:18px 0 0;padding:10px 16px;border:1.5px solid var(--amber);color:#7a5a0c;background:rgba(176,125,16,.07);
  font-family:"IBM Plex Mono",monospace;font-size:12px;letter-spacing:.06em;text-transform:uppercase}
.banner.dead{border-color:var(--red);color:var(--red);background:rgba(158,43,37,.06)}
.instrument{display:grid;grid-template-columns:minmax(320px,460px) 1fr;gap:44px;align-items:center;padding:36px 0 8px}
.dialwrap{position:relative}
.dial{width:100%;display:block}
.dial .tick{font-family:"IBM Plex Mono",monospace;font-size:10.5px;fill:var(--faint);text-anchor:middle}
#needle{animation:sweep 1.3s cubic-bezier(.25,.9,.3,1.05) both}
@keyframes sweep{from{transform:rotate(0deg)}}
.reading{text-align:center;margin-top:-8px}
.reading .big{font-size:60px;font-weight:600;letter-spacing:-.02em;font-variation-settings:"opsz" 72}
.reading .delta{font-family:"IBM Plex Mono",monospace;font-size:12px;color:var(--faint);letter-spacing:.08em}
.reading .delta .up{color:var(--green)} .reading .delta .down{color:var(--red)}
.stamp{position:absolute;top:8%;right:-2%;transform:rotate(-6deg);border:2.5px double var(--red);color:var(--red);
  padding:7px 13px;font-family:"IBM Plex Mono",monospace;font-size:11px;font-weight:600;letter-spacing:.14em;
  text-transform:uppercase;background:rgba(246,241,228,.85);max-width:240px;text-align:center}
.spark{width:220px;display:block;margin:10px auto 0;opacity:.9}
.chips{display:flex;flex-direction:column;gap:0;border-top:1px solid var(--rule)}
.readiness{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-bottom:18px}
.rmetric{border:1px solid var(--rule);padding:11px 12px;background:rgba(33,28,20,.015)}
.rvalue{display:block;font-family:"IBM Plex Mono",monospace;font-size:20px;color:var(--deep)}
.rlabel{display:block;font-size:14px;font-weight:600;margin-top:3px}
.rnote{display:block;font-family:"IBM Plex Mono",monospace;font-size:8.5px;line-height:1.4;color:var(--faint);letter-spacing:.06em;text-transform:uppercase;margin-top:3px}
.chip{display:grid;grid-template-columns:1fr auto;grid-template-rows:auto auto;padding:12px 4px;border-bottom:1px solid var(--rule)}
.chip-name{font-size:19px;font-weight:600}
.chip-score{font-family:"IBM Plex Mono",monospace;font-size:19px;grid-row:span 2;align-self:center}
.chip-n{font-family:"IBM Plex Mono",monospace;font-size:10.5px;color:var(--faint);letter-spacing:.08em;text-transform:uppercase}
.chip.out .chip-name,.chip.out .chip-score{color:var(--faint);font-style:italic}
.sect{margin-top:44px;animation:rise .7s cubic-bezier(.2,.8,.2,1) both}
@keyframes rise{from{opacity:0;transform:translateY(14px)}}
h2{font-size:26px;font-weight:600;border-bottom:1.5px solid var(--ink);padding-bottom:8px;margin-bottom:2px}
h2 .roman{font-style:italic;font-weight:400;color:var(--faint);margin-right:8px}
.sect-note{font-family:"IBM Plex Mono",monospace;font-size:11px;font-weight:400;color:var(--faint);letter-spacing:.1em;text-transform:uppercase;margin-left:12px}
.cards{column-count:2;column-gap:20px;margin-top:18px}
.card{break-inside:avoid;border:1px solid var(--rule);background:rgba(33,28,20,.015);padding:14px 16px 10px;margin-bottom:20px}
.card.out{opacity:.45}
.card-h{font-size:20px;font-weight:600;border-bottom:1px solid var(--ink);padding-bottom:7px;margin-bottom:6px}
.card-sub{display:block;font-family:"IBM Plex Mono",monospace;font-size:9.5px;color:var(--faint);letter-spacing:.14em;text-transform:uppercase;font-weight:400;margin-top:3px}
.card-oos{float:right;font-family:"IBM Plex Mono",monospace;font-size:9.5px;color:var(--rust);letter-spacing:.1em;text-transform:uppercase;font-weight:400;font-style:italic;margin-top:6px}
.st{display:flex;gap:12px;align-items:baseline;padding:6px 0;border-bottom:1px solid var(--rule);cursor:help;position:relative}
.st:hover{z-index:70}
.st:hover:not(:has(.pin:hover))>.st-pop{display:block}
.st:last-child{border-bottom:none}
.st-name{font-weight:600;font-size:13.5px;white-space:nowrap}
.st-pins{display:flex;flex-wrap:wrap;gap:4px 5px;margin-left:auto;justify-content:flex-end}
.pin{font-family:"IBM Plex Mono",monospace;font-size:9.5px;border:1px solid;border-radius:3px;padding:1.5px 6px;white-space:nowrap;cursor:help;position:relative}
.pin:hover{z-index:80}
.pin:hover>.pop{display:block}
.pin.ok{color:var(--green);border-color:var(--green)}
.pin.mid{color:#8a6309;border-color:var(--amber)}
.pin.bad{color:var(--red);border-color:var(--red)}
.pin.dim{color:var(--faint);border-color:var(--rule)}
.st-none{font-family:"IBM Plex Mono",monospace;font-size:9px;color:var(--faint);font-style:italic;letter-spacing:.08em}
.legend{display:flex;gap:8px;margin-top:2px}
.matrix{display:grid;grid-template-columns:repeat(auto-fill,minmax(158px,1fr));gap:10px;margin-top:18px}
.ccard{position:relative;display:block;border:1px solid var(--rule);border-left:3px solid var(--rule);background:rgba(33,28,20,.015);padding:9px 11px 8px;text-decoration:none;cursor:help}
.ccard:hover{border-color:var(--ink);border-left-width:3px}
.ccid{display:flex;justify-content:space-between;font-family:"IBM Plex Mono",monospace;font-size:10px;letter-spacing:.08em;color:var(--faint)}
.cicon{font-size:11px;opacity:.75}
.clabel{display:block;font-size:15px;font-weight:600;color:var(--ink);line-height:1.2;margin:3px 0 7px}
.ctag{display:block;font-family:"IBM Plex Mono",monospace;font-size:8.5px;letter-spacing:.16em;text-transform:uppercase;color:var(--faint)}
.ccard.ok{border-left-color:var(--green)} .ccard.ok .ccid{color:var(--green)}
.ccard.mid{border-left-color:var(--amber)} .ccard.mid .ccid{color:#8a6309}
.ccard.bad{border-left-color:var(--red)} .ccard.bad .ccid{color:var(--red)}
.ccard.todo{border-left-color:var(--rule)}
.ccard.out .ccid,.ccard.out .clabel,.ccard.out .ctag{opacity:.45}
.ccard.out .clabel{font-style:italic;font-weight:400}
.ccard:hover{z-index:70}
.pop{display:none;position:absolute;left:50%;bottom:calc(100% + 10px);transform:translateX(-50%);width:360px;max-width:82vw;background:var(--sheet);border:1.5px solid var(--ink);box-shadow:0 2px 4px rgba(33,28,20,.12),0 16px 40px -12px rgba(33,28,20,.4);padding:13px 16px 14px;z-index:60;white-space:normal;text-align:left;cursor:default}
.pop-meta{display:block;font-family:"IBM Plex Mono",monospace;font-size:10px;color:var(--faint);letter-spacing:.02em;text-transform:none;margin-top:6px;line-height:1.5}
.pop::after{content:"";position:absolute;top:100%;left:50%;transform:translateX(-50%);border:7px solid transparent;border-top-color:var(--ink)}
.ccard:hover .pop{display:block}
.pop-head{display:block;font-family:"IBM Plex Mono",monospace;font-size:9.5px;letter-spacing:.14em;text-transform:uppercase;color:var(--faint);border-bottom:1px solid var(--rule);padding-bottom:6px;margin-bottom:8px}
.pop-text{display:block;font-family:"Fraunces",Georgia,serif;font-size:13.5px;font-style:italic;font-weight:400;line-height:1.55;color:var(--ink);letter-spacing:0;text-transform:none}
table.ledger{width:100%;border-collapse:collapse;font-size:14.5px}
.ledger td{padding:8px 10px 8px 0;border-bottom:1px solid var(--rule);vertical-align:top}
.ledger tr.fam td{font-family:"IBM Plex Mono",monospace;font-size:11px;letter-spacing:.18em;text-transform:uppercase;
  color:var(--faint);padding-top:26px;padding-bottom:5px;border-bottom:1px solid var(--ink)}
.ledger tr:hover:not(.fam) td{background:rgba(33,28,20,.025)}
.glyph{width:30px;font-size:16px}
.glyph.ok{color:var(--green)} .glyph.bad{color:var(--red)} .glyph.mid{color:var(--amber)} .glyph.dim{color:var(--faint)}
.when{width:118px;font-size:11px;color:var(--faint);white-space:nowrap;text-align:right}
footer{margin-top:52px;padding-top:14px;border-top:2px solid var(--ink);display:flex;justify-content:space-between;
  font-family:"IBM Plex Mono",monospace;font-size:11px;color:var(--faint);letter-spacing:.1em;text-transform:uppercase}
footer a{color:var(--faint)}
.hd-right a{color:var(--faint)}
.micro-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(64px,1fr));gap:6px;margin-top:20px}
form.mrun{display:contents}
.mbox{display:flex;flex-direction:column;gap:3px;align-items:center;font-family:"IBM Plex Mono",monospace;font-size:9.5px;letter-spacing:.03em;padding:9px 2px 7px;border:1px solid var(--rule);border-left:3px solid var(--rule);background:rgba(33,28,20,.015);color:var(--ink);text-align:center;cursor:pointer;text-decoration:none}
span.mbox{cursor:default}
.mbox .mg{font-size:12px;line-height:1}
.mbox.ok{border-left-color:var(--green);background:rgba(44,110,73,.10)}
.mbox.ok .mg{color:var(--green)}
.mbox.mid{border-left-color:var(--amber);background:rgba(176,125,16,.08)}
.mbox.mid .mg{color:#8a6309}
.mbox.bad{border-left-color:var(--red);background:rgba(158,43,37,.10)}
.mbox.bad .mg{color:var(--red)}
.mbox.todo:hover,.mbox.ok:hover,.mbox.mid:hover,.mbox.bad:hover{border-color:var(--ink)}
.mbox.out{opacity:.35}
.mbox.running{border-left-color:var(--ink);animation:mpulse 1.2s ease-in-out infinite}
@keyframes mpulse{50%{opacity:.45}}
.mcat-mini{grid-column:1/-1;font-family:"IBM Plex Mono",monospace;font-size:9px;letter-spacing:.2em;text-transform:uppercase;color:var(--faint);border-bottom:1px solid var(--rule);padding:10px 0 3px}
blockquote{border-left:3px solid var(--red);padding:6px 0 6px 18px;font-size:19px;font-style:italic;line-height:1.6;margin:18px 0}
.meta{font-family:"IBM Plex Mono",monospace;font-size:12px;color:var(--faint);letter-spacing:.06em}
pre.ev{font-family:"IBM Plex Mono",monospace;font-size:11.5px;background:rgba(33,28,20,.04);border:1px solid var(--rule);
  padding:8px 10px;white-space:pre-wrap;max-height:180px;overflow:auto;margin-top:6px}
a.back{font-family:"IBM Plex Mono",monospace;font-size:11px;letter-spacing:.14em;text-transform:uppercase;color:var(--faint);text-decoration:none}
@media(max-width:900px){
 body{padding:1rem .6rem 3rem}
 .sheet{padding:26px 16px 36px 38px}
 .sheet::before{left:22px}
 header{flex-direction:column;align-items:flex-start;gap:10px}
 .hd-right{text-align:left;line-height:1.8}
 h1{font-size:30px}
 .instrument{grid-template-columns:1fr;gap:26px;padding:24px 0 4px}
 .reading .big{font-size:46px}
 .cards{column-count:1}
 .st{flex-wrap:wrap}
 .st-name{white-space:normal}
 .st-pins{margin-left:0;justify-content:flex-start}
 .matrix{grid-template-columns:repeat(auto-fill,minmax(136px,1fr));gap:8px}
 .pop{position:fixed;left:10px;right:10px;bottom:10px;top:auto;transform:none;width:auto;max-width:none}
 .pop::after{display:none}
 footer{flex-direction:column;gap:8px}
}
"#;

fn head(title: &str) -> String {
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{}</title>
<link rel="icon" href="data:,">
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,400;9..144,600;9..144,400&family=Fraunces:ital,opsz,wght@1,9..144,400&family=IBM+Plex+Mono:wght@400;500;600&display=swap" rel="stylesheet">
<style>{CSS}</style></head><body>"#,
        esc(title)
    )
}

pub fn index(m: &Model) -> String {
    let mut s = head("Shadow Audit — SOC 2 readiness");
    s.push_str(r#"<div class="sheet">"#);

    // header
    let ts = m.gauge.ts.as_deref().unwrap_or("no verify run recorded");
    let _ = write!(
        s,
        r#"<header><div><div class="kicker">Compliance Shadow — Working Papers</div><h1>Shadow Audit <span class="org">/ {}</span></h1></div>
<div class="hd-right">SOC 2 · TSP §100<br>61 criteria · <a href="/micro">micro board</a><br>last verify: {}</div></header>"#,
        esc(&m.org),
        esc(ts)
    );

    // staleness / no-data banners
    match m.gauge.stale_hours {
        None => s.push_str(r#"<div class="banner">No verify run recorded — the needle shows seeded state. Run agent/03-verify-compliance.md to take the first reading.</div>"#),
        Some(h) if h > 48.0 => {
            let _ = write!(s, r#"<div class="banner dead">State is stale — last verify {h:.0} hours ago. The monitor may be dead (that is itself a CC4.1 regression).</div>"#);
        }
        _ => {}
    }

    // I. instrument
    s.push_str(r#"<section class="sect"><h2><span class="roman">I.</span> The Instrument <span class="sect-note">would you pass an examination today?</span></h2><div class="instrument"><div class="dialwrap">"#);
    s.push_str(&gauge_svg(&m.gauge));
    if let (Some(cap), Some(reason)) = (m.gauge.cap, m.gauge.cap_reason.as_deref()) {
        let _ = write!(s, r#"<div class="stamp">Capped {cap:.0}% — {}</div>"#, esc(reason));
    }
    let delta = if m.gauge.history.len() >= 2 {
        let prev = m.gauge.history[m.gauge.history.len() - 2];
        let d = m.gauge.value - prev;
        if d.abs() < 0.05 {
            r#"<span>— unchanged since previous reading</span>"#.to_string()
        } else if d > 0.0 {
            format!(r#"<span class="up">▲ {d:+.1} since previous reading</span>"#)
        } else {
            format!(r#"<span class="down">▼ {d:+.1} since previous reading</span>"#)
        }
    } else {
        String::new()
    };
    let _ = write!(
        s,
        r#"<div class="reading"><div class="big">{:.1}%</div><div class="delta">{delta}</div>{}</div></div>"#,
        m.gauge.value,
        sparkline(&m.gauge.history)
    );
    s.push_str("<div>");
    s.push_str(&readiness_cards(&m.readiness));
    s.push_str(&category_chips(m));
    s.push_str("</div>");
    s.push_str("</div></section>");

    // II + III
    s.push_str(&machinery_cards(m));
    s.push_str(&criteria_matrix(m));

    // footer
    let _ = write!(
        s,
        r#"<footer><span>{} unknown checks (blind spots)</span><span>state renders; the agent computes — <a href="/db">export shadow.db</a></span></footer>"#,
        m.unknown_checks
    );
    s.push_str("</div></body></html>");
    s
}


// ---------- the micro board: one box per criterion, click = run its checks ----------

pub fn micro(m: &Model, running: &std::collections::HashSet<String>, runner_ok: bool, err: Option<&str>) -> String {
    let mut s = head("Shadow Audit — micro board");
    // zero-JS live updates: the board always polls by reloading, so greens
    // appear in front of your eyes no matter who drives the checks (a click,
    // the daily verify, or showtime.sh)
    s = s.replace("</head>", r#"<meta http-equiv="refresh" content="5"></head>"#);
    s.push_str(r#"<div class="sheet">"#);
    let _ = write!(
        s,
        r#"<header><div><div class="kicker">Compliance Shadow — Micro Board</div><h1>{:.1}% <span class="org">/ {}</span></h1></div>
<div class="hd-right"><a href="/">full working papers →</a><br>click a box to run its checks now<br>the official gauge moves on the next full verify</div></header>"#,
        m.gauge.value,
        esc(&m.org)
    );
    if let Some(e) = err {
        let msg = if e == "norunner" {
            "No verifier configured: install the claude CLI or set SHADOW_RUNNER — boxes render, but click-to-test is disabled."
        } else {
            "The verifier could not be started."
        };
        let _ = write!(s, r#"<div class="banner">{msg}</div>"#);
    } else if !runner_ok {
        s.push_str(r#"<div class="banner">Read-only: no verifier found (install the claude CLI or set SHADOW_RUNNER to enable click-to-test).</div>"#);
    }

    let mut crits: Vec<&Crit> = m.criteria.iter().collect();
    crits.sort_by(|a, b| (cat_rank(&a.category), &a.id).cmp(&(cat_rank(&b.category), &b.id)));
    s.push_str(r#"<div class="micro-grid">"#);
    let mut cat = "";
    for c in &crits {
        if c.category != cat {
            cat = &c.category;
            let label = CATEGORIES.iter().find(|(k, _)| *k == cat).map(|(_, l)| *l).unwrap_or(cat);
            let oos = if crits.iter().any(|x| x.category == cat && x.in_scope) { "" } else { " · not in scope" };
            let _ = write!(s, r#"<div class="mcat-mini">{label}{oos}</div>"#);
        }
        let is_running = running.contains(&c.id);
        let cls = if is_running {
            "running"
        } else if !c.in_scope {
            "out"
        } else {
            match c.status.as_str() {
                "verified" => "ok",
                "implemented" | "in_progress" => "mid",
                "failing" => "bad",
                _ => "todo",
            }
        };
        let (g, _) = crit_glyph(if c.in_scope { &c.status } else { "not_started" });
        let glyph = if is_running { "⟳" } else { g };
        let tip = format!(
            "{} — {} [{}] · click for the check log & evidence",
            c.id,
            label_for(&c.id),
            if c.in_scope { c.status.replace('_', " ") } else { "out of scope".into() }
        );
        // every box links to its evidence detail page (the check log). While a
        // verification is actively running, show the pulsing state instead.
        if is_running {
            let _ = write!(
                s,
                r#"<span class="mbox {cls}" title="{}"><span>{}</span><span class="mg">{glyph}</span></span>"#,
                esc(&tip), esc(&c.id),
            );
        } else {
            let _ = write!(
                s,
                r#"<a class="mbox {cls}" href="/criteria/{id}" title="{tip}"><span>{id}</span><span class="mg">{glyph}</span></a>"#,
                id = esc(&c.id),
                tip = esc(&tip),
            );
        }
    }
    let _ = runner_ok;
    s.push_str("</div>");
    let _ = write!(
        s,
        r#"<footer><span>{} running</span><span>boxes test on demand; evidence lands in the same ledger — <a href="/">details</a></span></footer>"#,
        running.len()
    );
    s.push_str("</div></body></html>");
    s
}

pub fn detail(c: &Crit, checks: &[CheckRow], atts: &[Attestation]) -> String {
    let mut s = head(&format!("{} — Shadow Audit", c.id));
    s.push_str(r#"<div class="sheet">"#);
    let (g, cls) = crit_glyph(&c.status);
    let _ = write!(
        s,
        r#"<a class="back" href="/">← working papers</a>
<header style="margin-top:14px"><div><div class="kicker">{}</div><h1><span class="glyph {cls}" style="font-size:30px">{g}</span> {}</h1></div>
<div class="hd-right">weight {} · {}<br>status: {}</div></header>
<blockquote>{}</blockquote>
<p class="meta">Verbatim, AICPA TSP Section 100. In-depth reference: criteria/{}.md — meaning, points of focus, PBC list, controls, evidence.</p>"#,
        esc(&c.family), esc(&c.id), c.weight,
        if c.in_scope { "in scope" } else { "out of scope" },
        esc(&c.status), esc(&c.text), esc(&c.id)
    );

    s.push_str(r#"<section class="sect"><h2><span class="roman">a.</span> Automated shadow checks</h2><table class="ledger">"#);
    if checks.is_empty() {
        s.push_str(r#"<tr><td class="meta">no checks recorded yet — run agent/03</td></tr>"#);
    }
    for ch in checks {
        let cls = match ch.verdict.as_str() { "pass" => "ok", "fail" => "bad", _ => "dim" };
        let glyph = match ch.verdict.as_str() { "pass" => "●", "fail" => "✕", _ => "?" };
        let ev = ch.evidence.as_deref().map(|e| format!(r#"<pre class="ev">{}</pre>"#, esc(e))).unwrap_or_default();
        let source = ch.source.as_deref().map(|v| format!(" · source {}", esc(v))).unwrap_or_default();
        let expiry = ch.expires_at.as_deref().map(|v| format!(" · expires {}", esc(v))).unwrap_or_default();
        let _ = write!(
            s,
            r#"<tr><td class="glyph {cls}">{glyph}</td><td><strong>{}</strong> <span class="meta">{} · {}{source}{expiry}</span>{ev}</td><td class="mono when">{}</td></tr>"#,
            esc(&ch.name), esc(&ch.verdict), esc(&ch.dimension), esc(&ch.last_run)
        );
    }
    s.push_str("</table></section>");

    s.push_str(r#"<section class="sect"><h2><span class="roman">b.</span> Attestations</h2><table class="ledger">"#);
    if atts.is_empty() {
        s.push_str(r#"<tr><td class="meta">none on file</td></tr>"#);
    }
    for a in atts {
        let link = a.evidence_link.as_deref().map(|l| format!(r#" — <a href="{}">evidence</a>"#, esc(l))).unwrap_or_default();
        let exp = a.expires_at.as_deref().map(|e| format!(" · expires {}", esc(e))).unwrap_or_default();
        let _ = write!(
            s,
            r#"<tr><td>{}{link}</td><td class="mono when">{} · {}{exp}</td></tr>"#,
            esc(&a.note), esc(&a.attested_by), esc(&a.attested_at)
        );
    }
    s.push_str("</table></section></div></body></html>");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus_path(source: &'static str, vendored: &'static str) -> &'static str {
        if std::path::Path::new(source).exists() { source } else { vendored }
    }

    // "CC3.1–CC3.4" (en dash range) → CC3.1, CC3.2, CC3.3, CC3.4; plain ids pass through
    fn expand(tag: &str) -> Vec<String> {
        let tag = tag.trim();
        if tag.is_empty() {
            return vec![];
        }
        if let Some((a, b)) = tag.split_once('\u{2013}') {
            let (a, b) = (a.trim(), b.trim());
            let dot = a.rfind('.').expect("range start has no dot");
            let prefix = &a[..=dot];
            let start: u32 = a[dot + 1..].parse().expect("range start");
            let end: u32 = b[b.rfind('.').expect("range end has no dot") + 1..].parse().expect("range end");
            (start..=end).map(|n| format!("{prefix}{n}")).collect()
        } else {
            vec![tag.to_string()]
        }
    }

    #[test]
    fn map_covers_all_criteria() {
        // ground truth: the criterion files themselves
        let mut all: std::collections::BTreeSet<String> =
            std::fs::read_dir(corpus_path("../../criteria", "../criteria"))
            .expect("criteria corpus not found in source or vendored layout")
            .filter_map(|e| {
                let name = e.ok()?.file_name().into_string().ok()?;
                name.strip_suffix(".md").map(str::to_string)
            })
            .collect();
        assert_eq!(all.len(), 61, "expected the 61 TSC criteria in criteria/");

        for item in MAP {
            if let MapItem::Station { crit, .. } = item {
                for part in crit.split('·') {
                    for id in expand(part) {
                        all.remove(&id);
                    }
                }
            }
        }
        assert!(
            all.is_empty(),
            "criteria missing from the map (no station carries them): {:?}",
            all
        );
    }

    #[test]
    fn labels_cover_all_criteria() {
        let all: std::collections::BTreeSet<String> =
            std::fs::read_dir(corpus_path("../../criteria", "../criteria"))
            .expect("criteria corpus not found in source or vendored layout")
            .filter_map(|e| {
                let name = e.ok()?.file_name().into_string().ok()?;
                name.strip_suffix(".md").map(str::to_string)
            })
            .collect();
        let labeled: std::collections::BTreeSet<String> = LABELS.iter().map(|(k, _)| k.to_string()).collect();
        assert_eq!(all, labeled, "LABELS must cover exactly the 61 criteria");
        for (id, label) in LABELS {
            assert_eq!(
                label.split_whitespace().count(),
                2,
                "label for {id} must be exactly two words: {label:?}"
            );
        }
    }

    #[test]
    fn every_procedure_is_pinned_once() {
        let mut seen = std::collections::BTreeSet::new();
        for item in MAP {
            if let MapItem::Station { pins, .. } = item {
                for pin in *pins {
                    assert!(seen.insert(pin.to_string()), "procedure pinned twice on the map: {pin}");
                }
            }
        }
        // ground truth: the procedure IDs in procedures/PROCEDURES.md
        let md = std::fs::read_to_string(corpus_path(
            "../../procedures/PROCEDURES.md",
            "../procedures/PROCEDURES.md",
        ))
        .expect("procedure corpus not found in source or vendored layout");
        let defined: std::collections::BTreeSet<String> = md
            .lines()
            .filter_map(|l| {
                let l = l.trim().strip_prefix('|')?;
                let cell = l.split('|').next()?.trim().replace('`', "");
                if cell.is_empty() || cell == "ID" || cell.starts_with("---") || cell.starts_with(":-") {
                    None
                } else {
                    Some(cell)
                }
            })
            .collect();
        assert_eq!(defined, seen, "map pins must match PROCEDURES.md exactly");
    }
}
