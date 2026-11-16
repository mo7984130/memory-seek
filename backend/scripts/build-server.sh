#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

cd "$ROOT"

exec cargo build \
    -p "$WEB_SERVER_PACKAGE" \
    --features "$WEB_SERVER_FEATURES" \
    "$@"
