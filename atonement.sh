#!/usr/bin/env bash
# atonement.sh — fix what genesis could not. Walks the "NOT established" list and,
# per item, does the most it safely can:
#   • auto    — an API exists (gh/gcloud): show the exact command, confirm, run it
#   • paper   — autogenerate the artifact as a real PR through the gate
#   • guided  — console-only (no API): deep-link + exact click steps, and with
#               --guided, open the real page in Playwright with a step overlay
# Nothing destructive runs without an explicit typed confirmation. Run from inside
# a shadow-installed repo (genesis installs this file at the repo root).
#
#   ./atonement.sh              walk every item, interactively
#   ./atonement.sh --guided     open console-only items in a guided browser
#   ./atonement.sh --only org-2fa   fix one item
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

GUIDED=0; ONLY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --guided) GUIDED=1 ;;
    --only) ONLY="$2"; shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

if [ -t 1 ]; then G=$'\033[32m'; R=$'\033[31m'; Y=$'\033[33m'; B=$'\033[1m'; D=$'\033[2m'; N=$'\033[0m'; else G=""; R=""; Y=""; B=""; D=""; N=""; fi
step(){ printf '%s[%s]%s %s▶ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$B" "$*" "$N"; }
ok(){ printf '   %s✓ %s%s\n' "$G" "$*" "$N"; }
warn(){ printf '   %s⚠ %s%s\n' "$Y" "$*" "$N"; }
info(){ printf '   %s· %s%s\n' "$D" "$*" "$N"; }
link(){ printf '   %s→ %s%s\n' "$B" "$*" "$N"; }

command -v gh >/dev/null || { echo "gh required" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "gh not authenticated" >&2; exit 1; }
OWNER=$(gh api user -q .login)
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
REPO_OWNER="${REPO%%/*}"
# org items only apply when the repo lives under a GitHub ORGANIZATION, not a
# personal account (/orgs/<user> 404s for personal accounts).
IS_ORG=0; gh api "/orgs/$REPO_OWNER" >/dev/null 2>&1 && IS_ORG=1
want(){ [ -z "$ONLY" ] || [ "$ONLY" = "$1" ]; }

confirm(){ # $1 = prompt
  printf '   %s%s%s [type yes]: ' "$Y" "$1" "$N"; read -r a; [ "$a" = "yes" ]
}

open_guided(){ # $1=url  $2=title  $3=steps(newlines)
  link "$1"
  printf '%s' "$3" | sed 's/^/     /'
  if [ "$GUIDED" = 1 ]; then
    if command -v node >/dev/null && [ -f .shadow/provision/guided.mjs ]; then
      URL="$1" TITLE="$2" STEPS="$3" node .shadow/provision/guided.mjs || \
        (command -v open >/dev/null && open "$1") || true
    else
      (command -v open >/dev/null && open "$1") || (command -v xdg-open >/dev/null && xdg-open "$1") || true
    fi
  fi
}

# ---------- org-2fa (auto via API, with ejection preflight) ----------
if want org-2fa && [ "$IS_ORG" = 1 ]; then
  step "Org-wide 2FA enforcement (CC6.1/CC6.2 — hard-gate item)"
  OWNER="$REPO_OWNER"
  CUR=$(gh api "/orgs/$OWNER" -q .two_factor_requirement_enabled 2>/dev/null || echo "unknown")
  if [ "$CUR" = "true" ]; then ok "already enforced"
  else
    NO2FA=$(gh api "/orgs/$OWNER/members?filter=2fa_disabled" --jq 'length' 2>/dev/null || echo "?")
    if [ "$NO2FA" != "0" ] && [ "$NO2FA" != "?" ]; then
      warn "$NO2FA member(s) WITHOUT 2FA would be REMOVED from the org when you enforce"
      gh api "/orgs/$OWNER/members?filter=2fa_disabled" --jq '.[].login' 2>/dev/null | sed 's/^/     - /' || true
    fi
    info "command: gh api -X PATCH /orgs/$OWNER -f two_factor_requirement_enabled=true"
    if confirm "enforce org 2FA now?"; then
      gh api -X PATCH "/orgs/$OWNER" -f two_factor_requirement_enabled=true >/dev/null && ok "enforced" \
        || { warn "API failed (need org-admin scope). Do it in the UI:"; \
             open_guided "https://github.com/organizations/$OWNER/settings/security" "Org 2FA" \
             "1. Under 'Two-factor authentication'"$'\n'"2. Check 'Require two-factor authentication…'"$'\n'"3. Save"; }
    else
      open_guided "https://github.com/organizations/$OWNER/settings/security" "Org 2FA" \
        "1. 'Require two-factor authentication for everyone in the <org>'"$'\n'"2. Save (removes members without 2FA)"
    fi
  fi
fi

# ---------- org base perms (auto) ----------
if want org-base-perms && [ "$IS_ORG" = 1 ]; then
  step "Default member permission = read, no public repo creation (CC6.3)"
  OWNER="$REPO_OWNER"
  info "command: gh api -X PATCH /orgs/$OWNER -f default_repository_permission=read -f members_can_create_public_repositories=false"
  if confirm "apply base permissions now?"; then
    gh api -X PATCH "/orgs/$OWNER" -f default_repository_permission=read -f members_can_create_public_repositories=false >/dev/null \
      && ok "applied" || warn "API failed (org-admin scope needed)"
  fi
fi

# ---------- workspace 2sv (guided — no API in most tiers) ----------
if want workspace-2sv; then
  step "Google Workspace / IdP 2SV enforcement (CC6.1)"
  warn "console-only — no API in standard Workspace tiers"
  open_guided "https://admin.google.com/ac/security/2sv" "Workspace 2SV enforcement" \
    "1. Security → Authentication → 2-step verification"$'\n'"2. Enforcement: On for everyone (allow a grace/enrollment window)"$'\n'"3. Save, then export the enrollment report as evidence"
  info "file the enrollment-report export under evidence/ when done"
fi

# ---------- policy pack + registers (autogenerate as a PR through the gate) ----------
if want policies; then
  step "Policy pack + registers (CC5.3, CC1.1, CC3.x, CC9.2) — autogenerating as a PR"
  if gh issue view --json number >/dev/null 2>&1; then :; fi
  ISSUE=$(gh issue create --title "Establish the policy pack and registers" \
    --body "Autogenerated draft policy pack + registers for founder review and approval (CC5.3). Each policy carries lifecycle frontmatter; approving = merging this PR." \
    | grep -oE '[0-9]+$' | head -1)
  BR="$ISSUE-policy-pack"
  git fetch -q origin staging && git checkout -q staging && git pull -q
  git checkout -qb "$BR"
  mkdir -p policies/runbooks
  TODAY=$(date -u +%F); REVIEW=$(date -u -v+12m +%F 2>/dev/null || date -u -d "+12 months" +%F)
  gen(){ # $1=file $2=title $3=criteria $4=body
    cat > "policies/$1.md" <<EOF
---
owner: $OWNER
version: 0.1.0-draft
approved_by: OPEN
approved_at: OPEN
review_by: $REVIEW
criteria: $3
---
# $2

> DRAFT — review, adjust to how this company actually operates, then approve by merging.

$4
EOF
  }
  gen information-security "Information Security Policy" "CC1.1, CC5.3" "Scope, roles, acceptable use, and the commitment to protect customer data. Points at the SDLC and the other policies."
  gen access-control "Access Control Policy" "CC6.1-CC6.3" "MFA everywhere, least privilege, quarterly access reviews, onboarding/offboarding grant+revoke."
  gen secure-development "Secure Development Policy" "CC8.1" "The dictated SDLC: ticket → branch → gated PR → archive → release. See sdlc/SDLC.md."
  gen change-management "Change Management Policy" "CC8.1" "Every change is authorized (ticket), reviewed, tested, and archived; emergencies follow the hotfix runbook."
  gen incident-response "Incident Response Plan" "CC7.3-CC7.5" "Detect → triage → contain → remediate → postmortem; breach notification path; severity tiers."
  gen business-continuity "Business Continuity & DR Policy" "CC9.1, A1.2, A1.3" "Backups + PITR, restore testing cadence, recovery objectives."
  gen risk-management "Risk Management Policy" "CC3.1-CC3.4, CC9.1" "Annual risk assessment incl. fraud consideration; treatments tracked as tickets."
  gen vendor-management "Vendor Management Policy" "CC9.2" "Assess → DPA → approve → annual re-review; LLM providers included."
  gen data-classification "Data Classification & Handling Policy" "C1.1, CC6.7" "Data tiers, encryption in transit/at rest, handling rules per tier."
  gen data-retention "Data Retention & Disposal Policy" "C1.2, CC6.5" "Retention schedule per data type; secure disposal, logged."
  gen encryption "Encryption & Key Management Policy" "CC6.1, CC6.7" "TLS everywhere, at-rest encryption, secret storage, rotation."
  gen acceptable-use "Acceptable Use & Code of Conduct" "CC1.1, CC1.5" "Expected conduct, device rules, reporting obligations."
  gen vulnerability-management "Vulnerability Management Policy" "CC7.1" "Scanner coverage, remediation SLAs by severity, exception process."
  gen ai-development "AI Development & Agent Use Policy" "CC8.1, CC6.3, CC9.2" "Agent identities are scoped and inventoried; no self-approval; prompts/specs live on the ticket; LLM-vendor data handling."
  for reg in risk-register vendor-register access-register; do
    [ -f "policies/$reg.md" ] && grep -q "^# " "policies/$reg.md" 2>/dev/null || \
      printf '# %s\n\nOPEN — populated by the shadow rituals (ritual-risks / ritual-vendors / ritual-access).\n' "$reg" > "policies/$reg.md"
  done
  git add policies && git commit -qm "#$ISSUE: Autogenerate draft policy pack + registers"
  git push -qu origin "$BR"
  gh pr create --base staging --title "#$ISSUE: Draft policy pack + registers" --body "## Summary
Autogenerated 14-policy draft pack + register scaffolds for founder review (#$ISSUE).

## Tickets
| Ticket | Title | Status |
|---|---|---|
| #$ISSUE | Establish the policy pack and registers | In Progress |

## Changes
$(git diff --name-only origin/staging..HEAD | sed 's/^/- /')

## Test Plan
- [x] Review each policy, adjust to reality, approve by merging (that merge is the CC5.3 approval evidence)" >/dev/null \
    && ok "policy-pack PR opened (#$ISSUE) — review, edit to reality, merge to approve" \
    || warn "PR creation failed — branch $BR is pushed; open the PR manually"
  git checkout -q staging
fi

# ---------- human approver on production (auto when a teammate exists) ----------
if want human-approver; then
  step "Human approver on production (CC8.1 segregation of duties)"
  MEMBERS=$(gh api "/repos/$REPO/collaborators" --jq 'length' 2>/dev/null || echo 1)
  if [ "$MEMBERS" -gt 1 ]; then
    info "you have >1 collaborator — require 1 approving review on main"
    if confirm "add required human approval to the main ruleset now?"; then
      gh api -X POST "repos/$REPO/rulesets" --input - >/dev/null <<'JSON' && ok "main now requires 1 human approval" || warn "ruleset add failed (may already exist — edit it in Settings → Rules)"
{"name":"shadow-main-approval","target":"branch","enforcement":"active",
 "conditions":{"ref_name":{"include":["refs/heads/main"],"exclude":[]}},
 "rules":[{"type":"pull_request","parameters":{"required_approving_review_count":1,"dismiss_stale_reviews_on_push":true,"require_code_owner_review":false,"require_last_push_approval":true,"required_review_thread_resolution":true}}]}
JSON
    fi
  else
    warn "solo — a required human approver would deadlock you (GitHub forbids self-approval)."
    info "This is the disclosed SoD limitation. Add the approval rule the day a second person joins."
  fi
fi

# ---------- background checks + training evidence (scaffold + point at ritual) ----------
if want people; then
  step "Background-check + security-training evidence (CC1.4)"
  mkdir -p "evidence/people"
  cat > "evidence/people/README.md" <<EOF
# People evidence (CC1.4)

One folder per person: \`evidence/people/<login>/\`, containing:
- \`background-check.md\` — provider, date, result (or attestation of why waived for founders)
- \`training-$(date -u +%Y).md\` — security-training completion + date

The ritual-policies interview generates the training + attestation issues; file the
signed results here. Background checks are a manual procurement step (auditors sample them).
EOF
  ok "scaffolded evidence/people/ — file each person's background check + training here"
  info "run the training+attestation interview:  gh workflow run shadow-agent.yml -f ritual=ritual-policies"
fi

# ---------- vendor DPAs (point at ritual + trust-page deep links) ----------
if want vendors; then
  step "Vendor DPAs / subprocessor SOC 2 reports (CC9.2, P6.4)"
  info "the vendor ritual auto-detects vendors and walks each one:"
  info "  gh workflow run shadow-agent.yml -f ritual=ritual-vendors"
  info "common trust/DPA pages to collect reports from:"
  link "GCP:    https://cloud.google.com/security/compliance/soc-2"
  link "GitHub: https://github.com/security"
  link "Anthropic: https://trust.anthropic.com"
fi

echo
if [ "$IS_ORG" = 0 ]; then
  info "org-2fa / org-base-perms skipped: $REPO_OWNER is a personal account, not an org."
  info "Personal-account MFA is per-user (github.com/settings/security). Move to a GitHub"
  info "Organization to get enforceable org-wide 2FA — an auditor prefers the org model."
fi
step "atonement pass complete"
info "re-check anytime:  ./judgment.sh --skip-pipeline   (the board shows what turned green)"
