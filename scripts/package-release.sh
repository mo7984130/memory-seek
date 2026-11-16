#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

cd "$ROOT"

version="${1:?usage: package-release.sh <version>}"
target="${TARGET:-x86_64-unknown-linux-gnu}"

bash "$SCRIPT_DIR/build-server.sh" --release --locked

cargo build --release --locked -p "$CLI_PACKAGE"

rm -rf dist
mkdir -p dist

package() {
    local name="$1" binary="$2"
    local stage
    stage="$(mktemp -d)"
    cp "target/release/${binary}" "${stage}/"
    cp LICENSE-MIT LICENSE-APACHE "${stage}/"
    tar -czf "dist/${name}.tar.gz" -C "${stage}" .
    rm -rf "${stage}"
}

package "memory-seek-${WEB_SERVER_BIN}-v${version}-${target}" "$WEB_SERVER_BIN"
package "memory-seek-${CLI_BIN}-v${version}-${target}" "$CLI_BIN"

cp LICENSE-MIT LICENSE-APACHE dist/
( cd dist && sha256sum *.tar.gz > SHA256SUMS )
