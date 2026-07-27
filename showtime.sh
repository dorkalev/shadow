#!/usr/bin/env bash
# showtime.sh — deploy the shadow, open the board, watch everything go green.
#
#   ./showtime.sh                      demo mode: simulated checks cascade green (~30s, $0)
#   ./showtime.sh --real               real mode: the claude verifier runs every criterion's
#                                      actual checks (gh/gcloud); failures open gh issues
#   ./showtime.sh --real --only CC6    real mode, criteria filtered by prefix
#   ./showtime.sh --firebase PROJECT   additionally deploy the static gauge to Firebase Hosting
#   ./showtime.sh --no-open            don't open the browser (CI/headless)
#
# Unified CI-style log on stdout; the micro board (auto-refreshing) mirrors it live.
set -euo pipefail
cd "$(dirname "$0")"

# ---------- args ----------
MODE=demo; ONLY=""; OPEN=1; FIREBASE=""; PORT="${PORT:-8399}"
while [ $# -gt 0 ]; do
  case "$1" in
    --real) MODE=real ;;
    --only) ONLY="$2"; shift ;;
    --firebase) FIREBASE="$2"; shift ;;
    --no-open) OPEN=0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

# ---------- CI-style log ----------
if [ -t 1 ]; then G=$'\033[32m'; R=$'\033[31m'; Y=$'\033[33m'; B=$'\033[1m'; D=$'\033[2m'; N=$'\033[0m'; else G=""; R=""; Y=""; B=""; D=""; N=""; fi
log()  { printf '%s[%s]%s %s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$*"; }
step() { printf '%s[%s]%s %s▶ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$B" "$*" "$N"; }
ok()   { printf '%s[%s]%s   %s✓ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$G" "$*" "$N"; }
warn() { printf '%s[%s]%s   %s⚠ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$Y" "$*" "$N"; }
bad()  { printf '%s[%s]%s   %s✗ %s%s\n' "$D" "$(date -u +%H:%M:%S)" "$N" "$R" "$*" "$N"; }

step "showtime — mode: $MODE"
for dep in cargo sqlite3 curl; do command -v "$dep" >/dev/null || { bad "$dep is required"; exit 1; }; done

# ---------- build + seed + serve ----------
step "building the gauge"
cargo build --release --manifest-path website/app/Cargo.toml >/dev/null 2>&1
SHADOW=website/app/target/release/shadow
DB="$(pwd)/showtime.db"; rm -f "$DB"
"$SHADOW" seed --criteria criteria --procedures procedures/PROCEDURES.md --db "$DB" >/dev/null
ok "built and seeded 61 criteria + 34 procedures"

step "starting the board on :$PORT"
# stop only OUR previous server via its pidfile — never pkill -f (which can
    # match unrelated processes whose argv contains the port)
    PIDFILE="/tmp/shadow-serve-$PORT.pid"
    [ -f "$PIDFILE" ] && kill "$(cat "$PIDFILE")" 2>/dev/null || true
SHADOW_ORG="${SHADOW_ORG:-showtime}" SHADOW_CRITERIA_DIR="$(pwd)/criteria" \
  nohup "$SHADOW" serve --db "$DB" --port "$PORT" >/tmp/shadow-showtime.log 2>&1 &
SERVER_PID=$!
  echo "$SERVER_PID" > "$PIDFILE"
for i in $(seq 1 40); do curl -sf -o /dev/null "http://localhost:$PORT/" && break; sleep 0.5; done
ok "live at http://localhost:$PORT/micro (pid $SERVER_PID)"

if [ "$OPEN" = 1 ]; then
  (command -v open >/dev/null && open "http://localhost:$PORT/micro") || \
  (command -v xdg-open >/dev/null && xdg-open "http://localhost:$PORT/micro") || true
fi

NOW() { date -u '+%Y-%m-%d %H:%M:%S'; }
ingest() { curl -sf -X POST -H 'content-type: application/json' -d "$1" "http://localhost:$PORT/ingest" >/dev/null; }

IDS=$(sqlite3 "$DB" "SELECT id FROM criteria WHERE in_scope=1 ${ONLY:+AND id LIKE '$ONLY%'} ORDER BY
      CASE substr(id,1,2) WHEN 'CC' THEN 0 ELSE 1 END, id")
TOTAL=$(printf '%s' "$IDS" | grep -c . || true)
[ "$TOTAL" -gt 0 ] || { bad "no criteria matched (bad --only prefix?)"; exit 1; }
GREEN=0; AMBER=0; RED=0; TICKETS=0
REPO=$(command -v gh >/dev/null && gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "")

# ---------- the run ----------
step "running $TOTAL criteria ($MODE)"
if [ "$MODE" = demo ]; then
  for id in $IDS; do
    ingest "{\"criteria\":[{\"id\":\"$id\",\"status\":\"verified\",\"credit\":1.0}],
             \"checks\":[{\"criterion\":\"$id\",\"name\":\"showtime demo check\",\"verdict\":\"pass\",
                          \"evidence\":\"simulated — run with --real for actual verification\",\"last_run\":\"$(NOW)\"}]}"
    ok "$id verified (simulated)"
    GREEN=$((GREEN+1)); sleep 0.12
  done
else
  command -v claude >/dev/null || { bad "--real needs the claude CLI"; exit 1; }
  warn "real mode: ~$TOTAL verifier calls against your actual gh/gcloud state — reds are honest findings"
  for id in $IDS; do
    step "$id — running its check table"
    PROMPT="You are the compliance shadow's single-criterion verifier. Criterion: $id. Read $(pwd)/criteria/$id.md and execute each row of its 'Automated shadow checks' table (skip MANUAL rows) using gh / gcloud / file checks. Scope: shadow/scope.json if present, else infer from 'gh repo view'. POST results with curl to http://localhost:$PORT/ingest as JSON with keys checks[] (criterion,name,verdict pass|fail|unknown,evidence,last_run UTC) and criteria[] (id,status verified|implemented|in_progress|failing,credit 1.0|0.6|0.25|0.0). unknown is never pass. Do NOT write a gauge entry. Be quick; no commentary."
    claude -p "$PROMPT" --allowedTools "Bash,Read,Glob,Grep" --max-turns 40 2>&1 | sed "s/^/$D          │ $N/" || true
    STATUS=$(sqlite3 "$DB" "SELECT status FROM criteria WHERE id='$id'")
    case "$STATUS" in
      verified) ok "$id verified"; GREEN=$((GREEN+1)) ;;
      implemented|in_progress) warn "$id $STATUS"; AMBER=$((AMBER+1)) ;;
      failing)
        bad "$id failing"
        RED=$((RED+1))
        if [ -n "$REPO" ]; then
          FAILS=$(sqlite3 "$DB" "SELECT group_concat(name, '; ') FROM checks WHERE criterion_id='$id' AND verdict='fail'")
          gh issue create --title "Shadow: $id failing — $FAILS" --label shadow \
            --body "Opened by showtime.sh. Failing checks: $FAILS. See criteria/$id.md for the fix; evidence is on the board at /criteria/$id." >/dev/null \
            && { ok "ticket opened for $id"; TICKETS=$((TICKETS+1)); } || warn "could not open ticket (labels/permissions?)"
        else
          warn "would open a ticket for $id (no gh repo in cwd)"
        fi ;;
      *) warn "$id unchanged (verifier did not conclude)" ;;
    esac
  done
fi

# gauge: recompute from the now-live criteria table and write the official reading
GAUGE=$(sqlite3 "$DB" "SELECT printf('%.1f', 100.0*SUM(weight*credit)/SUM(weight)) FROM criteria WHERE in_scope=1")
case "$GAUGE" in
  ''|*[!0-9.]*) warn "gauge not numeric ($GAUGE) — skipping gauge post" ;;
  *) ingest "{\"gauge\":{\"ts\":\"$(NOW)\",\"gauge\":$GAUGE}}" ;;
esac

# ---------- optional: ship the gauge to Firebase Hosting ----------
if [ -n "$FIREBASE" ]; then
  step "deploying the static gauge to Firebase Hosting ($FIREBASE)"
  warn "Firebase Hosting is PUBLIC — the board shows your compliance posture"
  SHADOW_ORG="${SHADOW_ORG:-showtime}" "$SHADOW" render --db "$DB" --out /tmp/shadow-dist >/dev/null
  cat > /tmp/shadow-dist-firebase.json <<EOF
{"hosting": {"public": "/tmp/shadow-dist", "ignore": []}}
EOF
  npx -y firebase-tools deploy --only hosting --project "$FIREBASE" --config /tmp/shadow-dist-firebase.json \
    && ok "hosted: https://$FIREBASE.web.app" || bad "firebase deploy failed (npx firebase-tools login?)"
fi

# ---------- summary ----------
echo
step "summary"
printf '  %s%s green%s · %s%s amber%s · %s%s red%s · gauge %s%s%%%s · tickets opened: %s\n' \
  "$G" "$GREEN" "$N" "$Y" "$AMBER" "$N" "$R" "$RED" "$N" "$B" "$GAUGE" "$N" "$TICKETS"
[ "$MODE" = demo ] && log "this was the simulation — ./showtime.sh --real runs the actual checks"
log "board is still live: http://localhost:$PORT/micro   (stop: kill $SERVER_PID)"
