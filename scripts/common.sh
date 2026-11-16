# scripts/common.sh
# shellcheck shell=bash
# 只定义公共常量，不改变 shell 选项，不执行副作用。

COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$COMMON_DIR/.." && pwd)"

COMPOSE_FILE="${COMPOSE_FILE:-$ROOT/tests/run/docker-compose.yml}"
TEST_CONFIG="${MEMORY_SEEK_CONFIG_PATH:-$ROOT/tests/run/docker-config.yml}"
E2E_LOG_DIR="${E2E_LOG_DIR:-$ROOT/target/e2e}"
E2E_BASE_URL="${E2E_BASE_URL:-http://127.0.0.1:7984}"

WEB_SERVER_PACKAGE="${WEB_SERVER_PACKAGE:-web-server}"
WEB_SERVER_FEATURES="${WEB_SERVER_FEATURES:-prod}"
WEB_SERVER_BIN="${WEB_SERVER_BIN:-web-server}"

E2E_PACKAGE="${E2E_PACKAGE:-e2e}"

CLI_PACKAGE="${CLI_PACKAGE:-cli}"
CLI_BIN="${CLI_BIN:-cli}"

CARGO_HACK_VERSION="${CARGO_HACK_VERSION:-0.6.45}"
CODEBOOK_VERSION="${CODEBOOK_VERSION:-0.4.0}"
