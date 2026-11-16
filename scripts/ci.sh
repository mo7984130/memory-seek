#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

cd "$ROOT"

if ! command -v codebook-lsp >/dev/null 2>&1; then
    echo "请先安装 codebook-lsp@$CODEBOOK_VERSION" >&2
    exit 1
fi

if ! command -v cargo-hack >/dev/null 2>&1; then
    echo "请先安装 cargo-hack@$CARGO_HACK_VERSION" >&2
    exit 1
fi

cargo fmt --all -- --check
codebook-lsp lint --unique .
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features --no-fail-fast
bash "$SCRIPT_DIR/check-features.sh"
