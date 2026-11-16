#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"
cd "$ROOT"

keep_stack=0
no_clean=0
skip_build=0
down_only=0
server_pid=""

log() { printf '\n==> %s\n' "$*"; }
die() { printf '\nERROR: %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'EOF'
用法: scripts/e2e.sh [选项]

选项:
  --keep        结束后保留依赖容器和 web-server
  --down        停止 web-server 并执行 compose down -v
  --no-clean    启动前不执行 down -v（保留数据库，迭代更快）
  --no-build    跳过 web-server 构建
  -h, --help    显示帮助

环境变量:
  MEMORY_SEEK_CONFIG_PATH  默认 tests/run/docker-config.yml
  E2E_BASE_URL             默认 http://127.0.0.1:7984
  E2E_ENGINE               强制指定 docker 或 podman
  E2E_LOG_DIR              默认 target/e2e
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --keep) keep_stack=1 ;;
        --no-clean) no_clean=1 ;;
        --no-build) skip_build=1 ;;
        --down) down_only=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "未知参数: $1" ;;
    esac
    shift
done

# ---------- 引擎检测 ----------

if [[ -n "${E2E_ENGINE:-}" ]]; then
    engine="$E2E_ENGINE"
elif command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    engine="docker"
elif command -v podman >/dev/null 2>&1 && podman compose version >/dev/null 2>&1; then
    engine="podman"
else
    die "未找到可用的 docker compose 或 podman compose"
fi

"$engine" compose version >/dev/null 2>&1 || die "$engine compose 不可用"
compose_cmd=("$engine" compose)

compose() {
    "${compose_cmd[@]}" -f "$COMPOSE_FILE" "$@"
}

# ---------- 工具函数 ----------

port_in_use() {
    (echo > "/dev/tcp/127.0.0.1/$1") 2>/dev/null
}

wait_tcp() {
    local port="$1" timeout="$2"
    local deadline=$((SECONDS + timeout))
    until (echo > "/dev/tcp/127.0.0.1/$port") 2>/dev/null; do
        (( SECONDS < deadline )) || die "等待端口 $port 超时"
        sleep 1
    done
}

compose_host_port() {
    local service="$1" container_port="$2" out
    out="$(compose port "$service" "$container_port")"
    [[ -n "$out" ]] || die "无法获取 $service:$container_port 的宿主端口"
    printf '%s\n' "${out##*:}"
}

compose_container_id() {
    local pattern="$1" ids id name
    ids="$(compose ps -aq 2>/dev/null || true)"
    [[ -n "$ids" ]] || ids="$(compose ps -q 2>/dev/null || true)"

    while read -r id; do
        [[ -n "$id" ]] || continue
        name="$("$engine" inspect -f '{{.Name}}' "$id" 2>/dev/null | sed 's#^/##')"
        if [[ "$name" == *"$pattern"* ]]; then
            printf '%s\n' "$id"
            return 0
        fi
    done <<< "$ids"

    return 1
}

wait_until() {
    local desc="$1" timeout="$2"
    shift 2

    local deadline=$((SECONDS + timeout))
    until "$@" >/dev/null 2>&1; do
        (( SECONDS < deadline )) || die "等待${desc}超时"
        sleep 1
    done
}

stop_server() {
    local pid="${server_pid:-}"

    if [[ -z "$pid" && -f "$E2E_LOG_DIR/server.pid" ]]; then
        pid="$(cat "$E2E_LOG_DIR/server.pid")"
    fi

    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        for _ in {1..20}; do
            kill -0 "$pid" 2>/dev/null || break
            sleep 0.1
        done
        kill -9 "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    fi

    rm -f "$E2E_LOG_DIR/server.pid"
}

wait_rustfs_init() {
    local deadline=$((SECONDS + 180))
    local cid status code

    cid="$(compose_container_id rustfs-init)" || die "找不到 rustfs-init 容器"

    while (( SECONDS < deadline )); do
        status="$("$engine" inspect -f '{{.State.Status}}' "$cid" 2>/dev/null || true)"

        if [[ "$status" == "exited" ]]; then
            code="$("$engine" inspect -f '{{.State.ExitCode}}' "$cid" 2>/dev/null || true)"
            if [[ "$code" == "0" ]]; then
                return 0
            fi
            compose logs --no-color rustfs-init >&2 || true
            die "rustfs-init 退出码 $code"
        fi

        sleep 1
    done

    compose logs --no-color rustfs-init >&2 || true
    die "等待 rustfs-init 超时"
}

cleanup() {
    local rc=$?
    trap - EXIT INT TERM

    if (( keep_stack == 0 )); then
        stop_server
    fi

    compose logs --no-color > "$E2E_LOG_DIR/compose.log" 2>&1 || true

    if (( keep_stack == 0 )); then
        compose down -v --remove-orphans || true
    fi

    if (( rc != 0 )); then
        echo "---- web-server.log ----" >&2
        tail -n 80 "$E2E_LOG_DIR/web-server.log" 2>/dev/null || true
        echo "---- e2e.log ----" >&2
        tail -n 80 "$E2E_LOG_DIR/e2e.log" 2>/dev/null || true
        echo "---- compose.log ----" >&2
        tail -n 80 "$E2E_LOG_DIR/compose.log" 2>/dev/null || true
        echo "日志目录: $E2E_LOG_DIR" >&2
    fi

    exit "$rc"
}

# ---------- 预检查 ----------

[[ -f "$COMPOSE_FILE" ]] || die "找不到 compose 文件: $COMPOSE_FILE"
[[ -f "$TEST_CONFIG" ]] || die "找不到配置文件: $TEST_CONFIG"
command -v curl >/dev/null 2>&1 || die "需要 curl"

mkdir -p "$E2E_LOG_DIR"

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

log "配置: $TEST_CONFIG"
log "Compose: $engine compose -f $COMPOSE_FILE"
log "e2e 地址: $E2E_BASE_URL"

# ---------- --down ----------

if (( down_only )); then
    log "停止 web-server 和依赖容器"
    stop_server
    compose down -v --remove-orphans || true
    exit 0
fi

# ---------- 端口检查 ----------

BASE_PORT="${E2E_BASE_URL##*:}"
BASE_PORT="${BASE_PORT%%/*}"
if ! [[ "$BASE_PORT" =~ ^[0-9]+$ ]]; then
    BASE_PORT=80
fi

if port_in_use "$BASE_PORT"; then
    die "端口 $BASE_PORT 已被占用。先执行 scripts/e2e.sh --down，或停止占用该端口的进程"
fi

# ---------- 启动依赖 ----------

if (( no_clean == 0 )); then
    log "清理旧依赖环境"
    compose down -v --remove-orphans || true
fi

log "启动依赖容器"
compose up -d

wait_tcp "$(compose_host_port postgres 5432)" 90
wait_tcp "$(compose_host_port valkey 6379)" 60
wait_tcp "$(compose_host_port mailhog 1025)" 60
wait_tcp "$(compose_host_port rustfs 9000)" 90

log "等待 Postgres 就绪"
wait_until "Postgres 就绪" 90 \
    compose exec -T postgres pg_isready -U memory_seek_test -d memory_seek_test

log "等待 Valkey 就绪"
wait_until "Valkey 就绪" 60 \
    compose exec -T valkey valkey-cli ping

log "等待 RustFS 初始化"
wait_rustfs_init

# ---------- 构建并启动 web-server ----------

if (( skip_build == 0 )); then
    log "构建 web-server"
    bash "$SCRIPT_DIR/build-server.sh" --locked
fi

log "启动 web-server"
MEMORY_SEEK_CONFIG_PATH="$TEST_CONFIG" \
    nohup "$ROOT/target/debug/$WEB_SERVER_BIN" \
    > "$E2E_LOG_DIR/web-server.log" 2>&1 &
server_pid=$!
echo "$server_pid" > "$E2E_LOG_DIR/server.pid"

log "等待 web-server 健康检查"
wait_until "web-server /health" 90 curl -fsS "$E2E_BASE_URL/health"

# ---------- 运行 e2e ----------

log "运行 e2e"
set +e
MEMORY_SEEK_CONFIG_PATH="$TEST_CONFIG" \
    E2E_BASE_URL="$E2E_BASE_URL" \
    cargo run --locked -p "$E2E_PACKAGE" 2>&1 | tee "$E2E_LOG_DIR/e2e.log"
rc=${PIPESTATUS[0]}
set -e

log "e2e 退出码: $rc"
log "web-server 日志: $E2E_LOG_DIR/web-server.log"
log "e2e 日志: $E2E_LOG_DIR/e2e.log"

exit "$rc"
