#!/usr/bin/env bash
# testimony.sh — the evidence, given under oath. Run from inside a
# shadow-installed repo (genesis.sh installs this file at the repo root).
#
#   ./testimony.sh --since 2026-01-01                 attest the window → attestation-*.md/.json
#   ./testimony.sh --since 2026-01-01 --until 2026-06-30
#   ./testimony.sh --since 2026-01-01 --file          also commit the report to compliance-archives
#
# This is what the CPA's fieldwork does, run against GitHub's own records:
# population from three independent sources (merged-PR API, git history,
# archive records), completeness reconciliation, then 100% attribute testing —
# authorized (ticket PRECEDES the change), independently reviewed, gated
# (no bypass), documented, via staging. Exit 0 = no exceptions; exit 1 = the
# honest exception list.
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

SINCE=""; UNTIL=""; PUSH=""
while [ $# -gt 0 ]; do
  case "$1" in
    --since) SINCE="$2"; shift ;;
    --until) UNTIL="$2"; shift ;;
    --file) PUSH=1 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done
[ -n "$SINCE" ] || { echo "usage: ./testimony.sh --since YYYY-MM-DD [--until YYYY-MM-DD] [--file]" >&2; exit 2; }
[ -d .shadow/ci ] || { echo "not a shadow-installed repo (.shadow/ci missing)" >&2; exit 1; }

cargo build --release --manifest-path .shadow/ci/Cargo.toml >/dev/null 2>&1
SINCE="$SINCE" ${UNTIL:+UNTIL="$UNTIL"} ${PUSH:+ARCHIVES_PUSH=1} \
  .shadow/ci/target/release/shadow-ci attest
