#!/usr/bin/env bash
set -euo pipefail

version="${1:?usage: package-release.sh <version>}"
target="${TARGET:-x86_64-unknown-linux-gnu}"

cargo build --release --locked -p web-server --features auth
cargo build --release --locked -p cli

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

package "memory-seek-web-server-v${version}-${target}" web-server
package "memory-seek-cli-v${version}-${target}" cli

cp LICENSE-MIT LICENSE-APACHE dist/
( cd dist && sha256sum *.tar.gz > SHA256SUMS )
