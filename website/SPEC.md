# Website Spec — the One-Pager Gauge

> **Status: built.** Implementation lives in [`app/`](app/) (Rust, axum + rusqlite, server-rendered, zero client JS — even the gauge needle is computed in Rust and animated with CSS). Screenshots in [`screenshots/`](screenshots/). Build & run:
> ```
> cd website/app
> cargo build --release
> ./target/release/shadow seed --criteria ../../criteria --procedures ../../procedures/PROCEDURES.md --db shadow.db
> SHADOW_ORG="my company" SHADOW_TOKEN=... ./target/release/shadow serve --db shadow.db --port 8300
> ```
> `demo.db` holds a fabricated mid-audit state (used to exercise rendering); `shadow.db` starts at the honest 0.0% until agent/03 takes the first reading.

## Purpose

One page with one primary internal metric above the full 61-criterion checklist **and the machinery section** — the agents, workflows, webhooks, scanners, and registers installed on the project, sourced from [`../procedures/PROCEDURES.md`](../procedures/PROCEDURES.md). The gauge is **weighted, in-scope criterion evidence maturity**: `Σ(weight × credit) / Σ(weight)`, where verified is 100% credit, implemented/design-only is 60%, and failing or not-started is 0%. It is not a probability of passing an examination.

Automated observation results are rendered separately with an explicit denominator: pass, fail, unknown, and not-applicable. The page always says that it is an automated point-in-time assessment, not a SOC 2 report or CPA opinion; neither Type I nor Type II status is inferred. Type II requires an elapsed examination period and auditor-selected operating samples. Provenance names the repository, commit, workflow run, generator, and whether the JSON report itself has a cryptographic signature.

Everything renders on one sheet — no tabs, no view flips (they contradicted the one-pager paradigm and were removed):

- **Section II, The Machinery — territory cards.** The ten program territories render as cards side by side (CSS columns, 2-up): *The Perimeter* (identity wall, charter, front door), *The Loop* (the SDLC — every change funnels through here), *The Sirens* (detect → respond → learn), *The People* (join → recertify → leave), *The Custody* (classify → back up → prove restore → dispose), *The Counterparties* (vendor lifecycle + cloud-inherited CC6.4), *The Risk*, *The Product* (PI, dimmed when out of scope), *The Subjects* (privacy, dimmed when out of scope), *The Clock*. Each card lists its stations one line each, with procedure pins right-aligned as status-colored chips (green verified, amber installed, red failing, grey not installed). Nothing is explained in visible text — hovering opens the same CSS-only modal used by the criteria matrix: a station shows `name · criteria` plus its description; a pin shows `id · category · status`, the procedure name, the detection command, and serves/install/last-checked. A `:has()` rule suppresses the station modal while a pin inside it is hovered. Machinery-less stations read "procedural."
- **Section III, The Criteria — the card matrix.** All 61 criteria as a grid of small cards, each carrying: the ID (`CC6.1`, status-colored, with a matching left border), a **two-word essence** ("Access Security", "Change Management", "Breach Notification"…), the category tag (`security`), and a **nature icon** in the corner — ⚙︎ technical (evidence is system state; 29 criteria) vs ¶ document-only (evidence is human-authored paper — policies, registers, minutes, attestations; 32 criteria). The nature comes from a `nature: technical|document` frontmatter field in each criterion file; machinery pins get the same treatment in their modal header (paper/cadence procedures are ¶). Hovering opens a CSS-only modal (no JS) containing the verbatim TSP §100 criterion text and nothing more, headed by `id · category · status`; clicking goes to the evidence page (`/criteria/{id}`). Out-of-scope criteria render dimmed/italic. The two-word labels live in `LABELS` in `app/src/render.rs`, and a third test — `labels_cover_all_criteria` — asserts every criterion has exactly one label of exactly two words.

**Coverage invariants, enforced by `cargo test`:** `map_covers_all_criteria` expands every station's criteria tag (including ranges) and asserts all 61 criteria in `criteria/` appear across the cards; `every_procedure_is_pinned_once` asserts the pinned set exactly matches the procedure IDs defined in `procedures/PROCEDURES.md` (34 today), each pinned once. Adding a criterion file or procedure without placing it on the map fails the build. The station→pin mapping is `MAP` in `app/src/render.rs`; unpinned DB procedures fall into a visible "Unpinned" card rather than disappearing.

## Stack decision

**Rust, single static binary.** `axum` (or `tiny_http`) + `rusqlite`, server-rendered HTML via one `format!`-style template (or `maud`), zero JS build step, zero frontend framework — the only JS is ~30 lines inline to draw the gauge arc on a `<canvas>`/SVG. Ships as one binary + one `shadow.db` file.

Rationale: the app is read-mostly with a single ingest endpoint; "one binary, one file, no runtime deps" matches the shadow-auditor ethos and the owner's Rust preference. Fallback: single-file Python/Flask (~150 lines) if iteration speed ever dominates — this spec is stack-agnostic; target ≤300 lines either way.

## Data model (SQLite: `shadow.db`)

```sql
CREATE TABLE criteria (        -- one row per TSC criterion, seeded from criteria/*.md frontmatter
  id TEXT PRIMARY KEY,         -- 'CC6.1'
  family TEXT NOT NULL,        -- 'CC6 — Logical and Physical Access Controls'
  category TEXT NOT NULL,      -- 'security' | 'availability' | 'confidentiality' | 'processing_integrity' | 'privacy'
  text TEXT NOT NULL,          -- verbatim criterion text
  weight INTEGER NOT NULL,     -- 1..3
  in_scope INTEGER NOT NULL DEFAULT 1,
  status TEXT NOT NULL DEFAULT 'not_started',  -- not_started|in_progress|implemented|verified|failing
  credit REAL NOT NULL DEFAULT 0.0,            -- 0 / 0.25 / 0.6 / 1.0
  updated_at TEXT
);

CREATE TABLE checks (          -- individual automated shadow checks, upserted by Runbook 03
  id INTEGER PRIMARY KEY,
  criterion_id TEXT NOT NULL REFERENCES criteria(id),
  name TEXT NOT NULL,
  verdict TEXT NOT NULL,       -- pass|fail|unknown|n/a
  evidence TEXT,               -- trimmed command output / link
  last_run TEXT NOT NULL,
  UNIQUE(criterion_id, name)
);

CREATE TABLE attestations (    -- manual evidence for organizational criteria
  id INTEGER PRIMARY KEY,
  criterion_id TEXT NOT NULL REFERENCES criteria(id),
  note TEXT NOT NULL,
  evidence_link TEXT,
  attested_by TEXT NOT NULL,
  attested_at TEXT NOT NULL,
  expires_at TEXT              -- default +12 months; expiry ⇒ decay to in_progress
);

CREATE TABLE gauge_history (   -- one row per verify run
  ts TEXT PRIMARY KEY,
  gauge REAL NOT NULL,         -- weighted in-scope criterion evidence maturity
  cap REAL,                    -- retained for schema compatibility; currently NULL
  cap_reason TEXT              -- retained for schema compatibility
);

CREATE TABLE procedures (      -- the machinery ledger, seeded from procedures/PROCEDURES.md
  id TEXT PRIMARY KEY,         -- 'post-merge-archive'
  name TEXT NOT NULL,
  category TEXT NOT NULL,      -- github-org|repo-gates|agents|evidence|scanners|paper|monitoring|cloud|cadence|identity
  criteria TEXT NOT NULL,      -- criteria served, display string
  install TEXT NOT NULL,       -- which runbook step installs it
  detect TEXT NOT NULL,        -- how runbook 03 re-detects it
  status TEXT NOT NULL DEFAULT 'not_installed',  -- not_installed|installed|verified|failing
  last_checked TEXT
);
```

## Micro board (`/micro`)

The one-pager's dense sibling: one small box per criterion (ID + status glyph, status-colored, category headers, out-of-scope dimmed), with weighted criterion maturity in the corner. **Clicking a box runs that criterion's checks right now**: the click is a form POST (`/run/{id}`, zero JS) that spawns the verifier — the `claude` CLI if found on PATH (built-in single-criterion prompt: execute the criterion file's "Automated shadow checks" table, POST results to `/ingest`), or any command set in `SHADOW_RUNNER` (invoked via `sh -c` with `CRITERION`, `CRITERION_FILE`, `SHADOW_URL` env). While anything runs, boxes pulse ⟳ and the page polls via a meta-refresh. Two honesty rules: single-box runs never write a gauge entry (the primary metric moves only on the full verify), and with no verifier available the board renders read-only and says so. The site never computes or claims compliance; it records evidence state.

## Routes

| Route | Method | Behavior |
|---|---|---|
| `/` | GET | The one page (below). |
| `/micro` | GET | The micro board. |
| `/run/{id}` | POST | Spawn the single-criterion verifier (form post from the micro board). |
| `/ingest` | POST | JSON body = Runbook 03 output (`checks`, `criteria`, `gauge_history` rows). Bearer token from env `SHADOW_TOKEN`. Upserts, then 204. (Optional — the runbook may also write `shadow.db` directly and skip HTTP.) |
| `/criteria/{id}` | GET | Detail: verbatim text, all checks with verdicts + evidence, attestations. Server-rendered. |
| `/db` | GET | Download `shadow.db` (evidence export for the real auditor). Same token. |

No auth beyond the token; deploy behind Tailscale/localhost — this is an internal instrument.

Responsive: below 900px the sheet goes single-column (cards stack, header stacks, criteria grid tightens) and all hover modals reposition as fixed bottom-sheets. The desk background is a distinctly darker shade than the sheet so the paper reads as paper.

## The page (top to bottom)

1. **Criterion maturity gauge** — fixed semicircular arc, 0–100%, needle at weighted in-scope evidence maturity. The formula and credits are printed beside it. Below the needle: the comparable trend from `gauge_history`; a metric-version migration discards incompatible legacy history.
2. **Evidence summary and category chips** — verified/implemented/not-started/failing criterion counts; applicable automated pass/fail/unknown counts with n/a separated; Security, Availability, Confidentiality, PI, and Privacy weighted sub-scores; out-of-scope categories greyed with "not in scope".
3. **The Machinery** — the ten territory cards (see above).
4. **The Criteria** — the 61-cell checkbox matrix (see above), mirroring [CHECKLIST.md](../CHECKLIST.md) content via hover.
5. **Footer** — last verify run time, count of `unknown` checks ("blind spots"), link to `/db`.

Honest-rendering rules: a stale verify run (>48h) banners the whole page ("state is stale — monitor may be dead"); `unknown` never displays as pass and not-applicable never inflates the applicable denominator; the gauge is always shown with its computation date, formula, and denominator. A perfect maturity score only means every in-scope criterion received full credit under this internal evidence rubric. It is never an auditor's opinion, a prediction of examination outcome, or a substitute for either a Type I CPA evaluation or a Type II observation period.

## Seeding

A `seed` subcommand (or `--seed criteria/`) parses the frontmatter + verbatim blockquote from each `criteria/*.md` and populates the `criteria` table. The markdown corpus stays the single source of truth; the DB is a projection.
