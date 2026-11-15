#!/usr/bin/env bash
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# 经过 tee 管道时 cargo 默认关闭颜色，这里强制保留
export CARGO_TERM_COLOR=always

depth=(--depth 2)

usage() {
    cat <<'EOF'
用法: scripts/check-features.sh [--depth N | --full]
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --depth)
            [[ $# -ge 2 ]] || { echo "--depth 需要参数" >&2; exit 2; }
            depth=(--depth "$2")
            shift 2
            ;;
        --full)
            depth=()
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "未知参数: $1" >&2
            usage
            exit 2
            ;;
    esac
done

log="$(mktemp)"
fail=0
cleanup() { [[ $fail -eq 0 ]] && rm -f "$log"; }
trap cleanup EXIT

# 去掉 ANSI 转义，仅用于文本判断；终端输出仍然保留颜色
strip_ansi() {
    sed -E 's/\x1B\[[0-9;]*[mK]//g'
}

die() {
    local reason="$1"
    fail=1
    trap - EXIT

    echo
    echo "========== 汇总 ==========" >&2
    echo "FAIL: $reason" >&2

    local warnings
    warnings="$(strip_ansi < "$log" | grep -E '^warning(:|\[)' || true)"
    if [[ -n "$warnings" ]]; then
        echo "发现的 warning（去重）:" >&2
        printf '%s\n' "$warnings" | sort -u >&2
    fi

    echo "完整日志: $log" >&2
    exit 1
}

run() {
    "$@" 2>&1 | tee -a "$log"
    local status=${PIPESTATUS[0]}
    if [[ $status -ne 0 ]]; then
        die "命令失败: $*"
    fi

    local warnings
    warnings="$(strip_ansi < "$log" | grep -E '^warning(:|\[)' || true)"
    if [[ -n "$warnings" ]]; then
        die "输出中出现 warning"
    fi
}

# 1. feature 名拼写检查
if grep -rnE '\b(deafult|defualt|defalut)\b' \
    --include=Cargo.toml --exclude-dir=target --exclude-dir=.git .; then
    die "Cargo.toml 中疑似 feature 名拼写错误"
fi

# 2. feature 矩阵（失败立即停止）
run cargo hack check \
    --workspace \
    --all-targets \
    --feature-powerset \
    "${depth[@]}"

# 3. 分层断言
for pkg in auth user; do
    tree_log="$(mktemp)"
    if ! cargo tree -p "$pkg" --no-default-features -e normal --prefix none \
        >"$tree_log" 2>&1; then
        cat "$tree_log" >&2
        rm -f "$tree_log"
        die "cargo tree -p $pkg 执行失败"
    fi

    if grep -qE '^(axum|common-web) ' "$tree_log"; then
        grep -E '^(axum|common-web) ' "$tree_log" >&2
        rm -f "$tree_log"
        die "$pkg 在 --no-default-features 下仍依赖 web 层"
    fi

    tree_warnings="$(strip_ansi < "$tree_log" | grep -E '^warning(:|\[)' || true)"
    if [[ -n "$tree_warnings" ]]; then
        rm -f "$tree_log"
        die "cargo tree -p $pkg 输出中出现 warning"
    fi

    rm -f "$tree_log"
done

# 4. 全 feature（depth 2 不覆盖 3 个以上 feature 同时开启）
run cargo check --workspace --all-features

echo "全部通过"
