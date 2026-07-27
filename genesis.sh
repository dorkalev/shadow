#!/usr/bin/env bash
# genesis.sh — from nothing to a proven-compliant repo, for real, in front of you.
#
#   ./genesis.sh                                    create shadow-genesis-<ts> (private), run the full story
#   ./genesis.sh --name myrepo --public             pick name/visibility
#   ./genesis.sh --gcp-project P --alert-email E    ALSO provision the Firestore runtime (terraform apply)
#   ./genesis.sh --only CC6                         limit the criteria half (tames the LLM bill)
#
# The story (nothing is simulated — synthetic content, real controls):
#   1. create a brand-new GitHub repo, main + staging
#   2. install the shadow (gates, archive, board, criteria corpus, judgment.sh)
#   3. (optional) terraform-apply the Firestore runtime, WIF-bound to this repo
#   4. open a REAL issue, branch, commit, push
#   5. non-compliant PR → gate REJECTS (negative control) → fix → gate PASSES → merge → archive
#   6. JUDGMENT: verify ALL criteria against the new repo (+ Firestore project);
#      failures open real gh issues; the board goes green/red in your browser
#
# Afterward, from inside the new repo, ./judgment.sh re-runs the route + all criteria any time.
#
# Needs: gh (authenticated), git, cargo, sqlite3, curl, claude; +terraform+gcloud for --gcp-project.
set -euo pipefail
cd "$(dirname "$0")"

NAME="shadow-genesis-$(date -u +%Y%m%d-%H%M%S)"; VIS="--private"
GCP_PROJECT=""; REGION="us-central1"; ALERT_EMAIL=""; ONLY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --name) NAME="$2"; shift ;;
    --public) VIS="--public" ;;
    --gcp-project) GCP_PROJECT="$2"; shift ;;
    --region) REGION="$2"; shift ;;
    --alert-email) ALERT_EMAIL="$2"; shift ;;
    --only) ONLY="$2"; shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done
if [ -n "$GCP_PROJECT" ] && [ -z "$ALERT_EMAIL" ]; then
  echo "--gcp-project requires --alert-email (CC7.2 needs a monitored inbox)" >&2; exit 2
fi

if [ -t 1 ]; then G=$'\033[32m'; R=$'\033[31m'; Y=$'\033[33m'; B=$'\033[1m'; D=$'\033[2m'; N=$'\033[0m'; else G=""; R=""; Y=""; B=""; D=""; N=""; fi
log()  { printf '%s[%s]%s %s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$*"; }
step() { printf '%s[%s]%s %s▶ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$B" "$*" "$N"; }
ok()   { printf '%s[%s]%s   %s✓ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$G" "$*" "$N"; }
warn() { printf '%s[%s]%s   %s⚠ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$Y" "$*" "$N"; }
bad()  { printf '%s[%s]%s   %s✗ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$R" "$*" "$N"; }

# ---------- preflight doctor: fail fast with the whole shopping list ----------
step "preflight — checking prerequisites before touching your account"
MISSING=""
for dep in gh git cargo sqlite3 curl claude jq; do
  command -v "$dep" >/dev/null || MISSING="$MISSING $dep"
done
if [ -n "$MISSING" ]; then
  bad "missing tools:$MISSING"
  info "install: rust (cargo) via rustup · gh via brew · claude via 'npm i -g @anthropic-ai/claude-code' · jq via brew"
  exit 1
fi
gh auth status >/dev/null 2>&1 || { bad "gh not authenticated — run: gh auth login (choose HTTPS + browser so the token gets 'workflow' scope)"; exit 1; }
gh auth setup-git >/dev/null 2>&1 || true  # ensure git can auth to github non-interactively (HTTPS or SSH)
# the token MUST carry 'workflow' scope or pushing .github/workflows/*.yml is rejected
if ! gh auth status 2>&1 | grep -q "'workflow'"; then
  warn "your gh token may lack the 'workflow' scope — pushing workflow files can be rejected"
  info "if the install push fails, run: gh auth refresh -s workflow  and re-run"
fi
command -v claude >/dev/null && claude --version >/dev/null 2>&1 || warn "claude is installed but 'claude --version' failed — ensure it is logged in (the criteria half needs it)"
[ -n "${ANTHROPIC_API_KEY:-}" ] || warn "ANTHROPIC_API_KEY not exported — daily-verify/shadow-agent won't run until set; the demo still works (reviewer falls back to a presence marker)"
case "$VIS" in
  --private) warn "PRIVATE repo: secret scanning & branch rulesets need a paid plan / GH Advanced Security — they'll warn-and-skip. Use --public for a fully-green demo on a free account." ;;
esac
ok "preflight passed"
if [ -n "$GCP_PROJECT" ]; then
  for dep in terraform gcloud; do command -v "$dep" >/dev/null || { bad "$dep is required for --gcp-project"; exit 1; }; done
  gcloud auth list --filter=status:ACTIVE --format='value(account)' | grep -q . || { bad "gcloud not authenticated"; exit 1; }
fi
OWNER=$(gh api user -q .login)
PLATFORM="$(pwd)"

# ---------- 1. the repo ----------
step "creating $OWNER/$NAME ($VIS)"
WORK=$(mktemp -d)/repo
gh repo create "$NAME" $VIS --clone -- "$WORK" >/dev/null 2>&1 || gh repo create "$NAME" $VIS >/dev/null
[ -d "$WORK" ] || git clone -q "https://github.com/$OWNER/$NAME" "$WORK"
cd "$WORK"
git config user.name "genesis"; git config user.email "genesis@noreply.local"
ok "repo created"

# ---------- 2. install the shadow (vendored mode) ----------
step "installing the shadow (vendored: gates + archive + board + corpus + judgment)"
mkdir -p .shadow/ci .shadow/site .shadow/criteria .shadow/procedures .github/workflows
cp -R "$PLATFORM/actions/shadow-ci/Cargo.toml" "$PLATFORM/actions/shadow-ci/src" .shadow/ci/
cp -R "$PLATFORM/website/app/Cargo.toml" "$PLATFORM/website/app/src" .shadow/site/
cp "$PLATFORM"/criteria/*.md .shadow/criteria/
cp "$PLATFORM/procedures/PROCEDURES.md" .shadow/procedures/
mkdir -p .shadow/agent .shadow/commands
cp "$PLATFORM"/agent/*.md .shadow/agent/
cp "$PLATFORM"/commands/*.md .shadow/commands/
cp "$PLATFORM/judgment.sh" ./judgment.sh && chmod +x judgment.sh
cp "$PLATFORM/testimony.sh" ./testimony.sh && chmod +x testimony.sh
cp "$PLATFORM/atonement.sh" ./atonement.sh && chmod +x atonement.sh
mkdir -p .shadow/provision && cp "$PLATFORM/provision/guided.mjs" .shadow/provision/guided.mjs
# the Clock — the long-term machinery, not just the birth certificate:
cp "$PLATFORM/actions/workflows/daily-verify.yml"      .github/workflows/   # drift detector (24h)
cp "$PLATFORM/actions/workflows/quarterly-rituals.yml" .github/workflows/   # evidence packets + interviews
cp "$PLATFORM/actions/workflows/shadow-agent.yml"      .github/workflows/   # async interviews + reminders
cp "$PLATFORM/actions/workflows/drill.yml"             .github/workflows/   # quarterly gate self-test
cp "$PLATFORM/actions/workflows/compliance.yml" .github/workflows/   # requires shadow-reviewer by default
cp "$PLATFORM/actions/workflows/review.yml"     .github/workflows/   # the built-in reviewer (LLM key from secrets)
cp "$PLATFORM/actions/workflows/post-merge-archive.yml" .github/workflows/
cat > .github/pull_request_template.md <<'EOF'
## Summary

## Tickets
| Ticket | Title | Status |
|---|---|---|

## Changes

## Test Plan
EOF
cat > .gitignore <<'EOF'
.shadow/ci/target/
.shadow/site/target/
shadow/*.db
provision/**/.terraform/
provision/**/*.tfstate*
provision/**/tfplan
.shadow-evidence/
EOF
mkdir -p policies/runbooks
for reg in risk-register vendor-register access-register; do
  printf '# %s\n\nOPEN — populated by the shadow rituals (ritual-risks / ritual-vendors / ritual-access).\n' "$reg" > "policies/$reg.md"
done
printf '# Hotfix procedure\n\nDirect pushes to main are allowed only as documented emergencies: incident ticket + backport PR through the normal gates. The bypass detector bills undocumented ones.\n' > policies/runbooks/hotfix.md
cat > README.md <<EOF
# $NAME

This repo has a compliant CHANGE PIPELINE and the scaffolding for everything
else — installed by genesis.sh from the compliance-shadow platform. It is
readiness, not a SOC 2 report: a report requires a licensed CPA and evidence
accrued over months. Run \`./judgment.sh\` to re-prove the pipeline and re-test
criteria; \`./testimony.sh --since <date>\` for the change-management attestation.
EOF
git add -A && git commit -qm "Install the compliance shadow (gates + archive + board + judgment)"
git branch -M main
git push -qu origin main
git checkout -qb staging && git push -qu origin staging
gh label create shadow --description "opened by the compliance shadow" --color 9e2b25 >/dev/null 2>&1 || true
gh label create shadow-drill --color d5cab0 >/dev/null 2>&1 || true
gh label create incident --color b60205 >/dev/null 2>&1 || true
ok "installed and pushed (main + staging)"

step "scanners + intake (CC7.1, CC6.8, CC2.3)"
gh api -X PUT "repos/$OWNER/$NAME/vulnerability-alerts" >/dev/null 2>&1 && ok "dependabot alerts on" || warn "dependabot alerts: could not enable"
gh api -X PUT "repos/$OWNER/$NAME/automated-security-fixes" >/dev/null 2>&1 && ok "dependabot security fixes on" || true
gh api -X PATCH "repos/$OWNER/$NAME" --input - >/dev/null 2>&1 <<'JSON' && ok "secret scanning + push protection on" || warn "secret scanning: needs GH Advanced Security on private repos (free on public)"
{"security_and_analysis":{"secret_scanning":{"status":"enabled"},"secret_scanning_push_protection":{"status":"enabled"}}}
JSON
gh api -X PUT "repos/$OWNER/$NAME/private-vulnerability-reporting" >/dev/null 2>&1 && ok "private vulnerability reporting on (the front door)" || true

step "wiring secrets for the Clock"
if [ -n "${ANTHROPIC_API_KEY:-}" ]; then
  gh secret set ANTHROPIC_API_KEY --body "$ANTHROPIC_API_KEY" && ok "ANTHROPIC_API_KEY set — daily verify + shadow agent are armed"
else
  warn "ANTHROPIC_API_KEY not in env — daily-verify and shadow-agent will fail until: gh secret set ANTHROPIC_API_KEY"
fi
warn "drill.yml needs a fine-grained PAT: gh secret set DRILL_TOKEN (gates cannot be triggered by GITHUB_TOKEN)"

# ---------- 3. optional: the Firestore runtime ----------
if [ -n "$GCP_PROJECT" ]; then
  step "provisioning the Firestore runtime in $GCP_PROJECT (terraform apply)"
  cp -R "$PLATFORM/provision" ./provision
  cp "$PLATFORM/actions/workflows/deploy.yml" .github/workflows/deploy.yml
  sed -e "s/GCP_PROJECT: my-project/GCP_PROJECT: $GCP_PROJECT/" -e "s/LOCATION: us-central1/LOCATION: $REGION/" \
    "$PLATFORM/actions/workflows/restore-test.yml" > .github/workflows/restore-test.yml
  terraform -chdir=provision/gcp init -input=false >/dev/null
  terraform -chdir=provision/gcp apply -input=false -auto-approve \
    -var "project_id=$GCP_PROJECT" -var "region=$REGION" \
    -var "github_repo=$OWNER/$NAME" -var "alert_email=$ALERT_EMAIL" \
    -var "service_name=app" 2>&1 | tail -3
  gh variable set GCP_WIF_PROVIDER --body "$(terraform -chdir=provision/gcp output -raw wif_provider)"
  gh variable set GCP_DEPLOY_SA    --body "$(terraform -chdir=provision/gcp output -raw deploy_service_account)"
  gh variable set GCP_ARTIFACT_REPO --body "$(terraform -chdir=provision/gcp output -raw artifact_repo)"
  gh variable set GCP_REGION --body "$REGION"
  gh variable set GCP_SERVICE --body "app"
  mkdir -p shadow
  cat > shadow/scope.json <<EOF
{"org": "$OWNER", "repos": ["$NAME"], "cloud": "gcp", "gcp_projects": ["$GCP_PROJECT"],
 "categories": ["security", "availability", "confidentiality"]}
EOF
  # with a real cloud runtime, Availability + Confidentiality controls exist (backups, PITR, TLS, audit logs)
  git add -A && git commit -qm "Provision the Firestore runtime (#infra: WIF, backups+PITR, audit logs, uptime)" && git push -q
  ok "runtime live; WIF bound to $OWNER/$NAME; scope.json points the verifier at $GCP_PROJECT"
else
  mkdir -p shadow
  cat > shadow/scope.json <<EOF
{"org": "$OWNER", "repos": ["$NAME"], "cloud": "none",
 "categories": ["security"]}
EOF
  git add -A && git commit -qm "Shadow scope (security only — no cloud runtime yet)" && git push -q
  warn "no --gcp-project: scope is SECURITY-ONLY. Availability & Confidentiality need cloud"
  warn "controls (backups/PITR/TLS/audit logs) — re-run with --gcp-project to add them, then"
  warn "widen scope.json. Scoping them in now would show green with nothing behind it."
fi

# ---------- 4. judgment: the route (three controls) + every criterion ----------
step "JUDGMENT — negative/positive controls on the route, then all criteria${GCP_PROJECT:+ against $GCP_PROJECT}"
./judgment.sh ${ONLY:+--only "$ONLY"}

# ---------- 5. lock the doors (the check contexts now exist) ----------
step "locking the doors: branch rulesets (now that the check contexts exist)"
gh api -X POST "repos/$OWNER/$NAME/rulesets" --input - >/dev/null <<'JSON' && ok "staging ruleset: PR + green compliance checks required, no force push" || warn "staging ruleset failed"
{"name":"shadow-staging","target":"branch","enforcement":"active",
 "conditions":{"ref_name":{"include":["refs/heads/staging"],"exclude":[]}},
 "bypass_actors":[{"actor_id":5,"actor_type":"RepositoryRole","bypass_mode":"always"}],
 "rules":[{"type":"pull_request","parameters":{"required_approving_review_count":0,"dismiss_stale_reviews_on_push":false,"require_code_owner_review":false,"require_last_push_approval":false,"required_review_thread_resolution":true}},
          {"type":"required_status_checks","parameters":{"strict_required_status_checks_policy":false,"required_status_checks":[{"context":"compliance-audit"},{"context":"compliance-review-gate"}]}},
          {"type":"non_fast_forward"},{"type":"deletion"}]}
JSON
gh api -X POST "repos/$OWNER/$NAME/rulesets" --input - >/dev/null <<'JSON' && ok "main ruleset: no direct pushes, no force, no deletion (admin bypass stays possible — and billed by the archive)" || warn "main ruleset failed"
{"name":"shadow-main","target":"branch","enforcement":"active",
 "conditions":{"ref_name":{"include":["refs/heads/main"],"exclude":[]}},
 "bypass_actors":[{"actor_id":5,"actor_type":"RepositoryRole","bypass_mode":"always"}],
 "rules":[{"type":"pull_request","parameters":{"required_approving_review_count":0,"dismiss_stale_reviews_on_push":false,"require_code_owner_review":false,"require_last_push_approval":false,"required_review_thread_resolution":false}},
          {"type":"non_fast_forward"},{"type":"deletion"}]}
JSON

# ---------- 6. judgment: every criterion, tested ----------
echo
step "genesis complete — a compliant change pipeline, not a finished audit"
echo "  repo:     https://github.com/$OWNER/$NAME   (clone: $WORK)"
echo "  route:    the judgment PR was rejected twice (unauthorized, self-authorized), then accepted — all in its checks history"
echo "  archive:  https://github.com/$OWNER/$NAME/tree/compliance-archives"
echo "  rerun:    cd $WORK && ./judgment.sh   ·   attest: ./testimony.sh --since $(date -u +%Y-%m-01)"
echo
warn "NOT established by genesis — run ./atonement.sh to fix these interactively"
warn "(auto-runs what has an API, deep-links + Playwright-guides the console-only ones):"
echo "    • Org-wide 2FA enforcement (gh api -X PATCH /orgs/$OWNER -f two_factor_requirement_enabled=true) — hard-gate item"
echo "    • Google Workspace / IdP 2SV enforcement + admin-role review"
echo "    • The policy pack, risk/vendor/access registers (stubs only — run the ritual interviews to fill them)"
echo "    • Background-check + security-training evidence per person (CC1.4)"
echo "    • A human approver on production if you add teammates (solo founders: this is a disclosed SoD limitation — see below)"
echo "    • Vendor DPAs / subprocessor SOC 2 reports on file (CC9.2)"
echo
warn "SoD note: review here is an automated agent, not an independent human. For a solo"
warn "founder this is a defensible COMPENSATING control (gates + immutable archives +"
warn "release confirmation), but it is a disclosed limitation, not satisfied segregation."
warn "Readiness ≠ a clean Type II — evidence must accrue over your audit window (months)."
log "teardown when done admiring: gh repo delete $OWNER/$NAME --yes${GCP_PROJECT:+  (and terraform -chdir=$WORK/provision/gcp destroy)}"
