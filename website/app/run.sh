#!/usr/bin/env bash
# Build, seed, and serve the compliance shadow one-pager.
#   ./run.sh                 # serve on :8300 with defaults
#   PORT=9000 ORG=acme ./run.sh
set -euo pipefail
cd "$(dirname "$0")"

PORT="${PORT:-8300}"
ORG="${ORG:-${SHADOW_ORG:-pilot project}}"
DB="${DB:-shadow.db}"

cargo build --release

# Seed/refresh from the markdown corpus (upserts; never clobbers live status)
./target/release/shadow seed \
  --criteria ../../criteria \
  --procedures ../../procedures/PROCEDURES.md \
  --db "$DB"

exec env SHADOW_ORG="$ORG" ./target/release/shadow serve --db "$DB" --port "$PORT"
