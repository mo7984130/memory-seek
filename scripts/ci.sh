#!/usr/bin/env bash
set -uo pipefail

cargo fmt --all -- --check
codebook-lsp lint --unique .
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features --no-fail-fast
./scripts/check-features.sh
