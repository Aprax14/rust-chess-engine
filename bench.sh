#!/usr/bin/env bash
set -euo pipefail

RESULTS_FILE="benches/results.md"
BASELINE="latest"

cargo build --release --quiet

COMMIT=$(git rev-parse --short HEAD)
DATE=$(date -u '+%Y-%m-%d %H:%M UTC')
CPU=$(lscpu | awk -F': +' '/Model name/ { print $2; exit }')
CORES=$(nproc)
RAM=$(free -h | awk '/^Mem:/ { print $2 }')
OS=$(uname -sr)

{
    echo "## Benchmark results"
    echo ""
    echo "**Commit:** \`$COMMIT\`  "
    echo "**Date:** $DATE  "
    echo "**CPU:** $CPU ($CORES cores)  "
    echo "**RAM:** $RAM  "
    echo "**OS:** $OS"
    echo ""
    echo '```'
    cargo bench --bench chess -- --save-baseline "$BASELINE" 2>/dev/null
    echo '```'
} > "$RESULTS_FILE"

echo "Results saved to $RESULTS_FILE"
