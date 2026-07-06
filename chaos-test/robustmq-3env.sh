#!/usr/bin/env bash
# Copyright 2023 RobustMQ Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${GREEN}[3env]${NC} $*"; }
warn() { echo -e "${YELLOW}[warn]${NC}  $*" >&2; }
error() {
    echo -e "${RED}[error]${NC} $*" >&2
    exit 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
STATE_DIR="${SCRIPT_DIR}/.robustmq-3env"
STATE_FILE="${STATE_DIR}/state.tsv"
PACKAGE_FILE="${STATE_DIR}/package.path"
MANAGED_MARKER=".robustmq-3env-managed"
CONFIG_SOURCE_DIR="${ROBUSTMQ_3ENV_CONFIG_DIR:-${SCRIPT_DIR}/config/robustmq-3env}"

NODES=("node1" "node2" "node3")
RUNTIME_ROOTS=("/home/node1/robustmq" "/home/node2/robustmq" "/home/node3/robustmq")
START_ROLLBACK_ACTIVE=false

usage() {
    cat <<EOF
Usage:
  bash chaos-test/robustmq-3env.sh <command>

Commands:
  build     Build/package RobustMQ once, or use ROBUSTMQ_3ENV_PACKAGE
  prepare   Create/reuse node1-node3 users and prepare /home/nodeX/robustmq
  start     Start three RobustMQ nodes as node1-node3
  status    Show process state recorded by this script
  stop      Stop only processes started by this script
  restart   Stop then start
  clean     Remove generated runtime directories and state, but keep users
  all       Run build, prepare, start, status
  help      Show this help

Environment:
  ROBUSTMQ_3ENV_PACKAGE=/path/file.tar.gz  Use an existing robustmq-*.tar.gz package
  ROBUSTMQ_3ENV_BUILD_FRONTEND=true  Pass --with-frontend to scripts/build.sh
  ROBUSTMQ_3ENV_OVERWRITE=true  Allow prepare to overwrite unmarked runtime roots
  ROBUSTMQ_3ENV_CONFIG_DIR=/path/dir  Use config dir with node1-node3/server.toml and logger.toml
  ROBUSTMQ_3ENV_READY_TIMEOUT=90  Seconds to wait for each node to become ready
EOF
}

require_root() {
    if [ "$(id -u)" -ne 0 ]; then
        error "This command must run as root because it manages users and /home/nodeX runtime directories."
    fi
}

ensure_state_dir() {
    mkdir -p "$STATE_DIR"
}

package_path() {
    if [ -n "${ROBUSTMQ_3ENV_PACKAGE:-}" ]; then
        printf '%s\n' "$ROBUSTMQ_3ENV_PACKAGE"
        return 0
    fi
    if [ -f "$PACKAGE_FILE" ]; then
        cat "$PACKAGE_FILE"
        return 0
    fi
    discover_package
}

build_package() {
    ensure_state_dir

    if [ -n "${ROBUSTMQ_3ENV_PACKAGE:-}" ]; then
        [ -f "$ROBUSTMQ_3ENV_PACKAGE" ] || error "ROBUSTMQ_3ENV_PACKAGE does not exist: $ROBUSTMQ_3ENV_PACKAGE"
        case "$ROBUSTMQ_3ENV_PACKAGE" in
            *.tar.gz) ;;
            *) error "ROBUSTMQ_3ENV_PACKAGE must point to a .tar.gz package: $ROBUSTMQ_3ENV_PACKAGE" ;;
        esac
        printf '%s\n' "$ROBUSTMQ_3ENV_PACKAGE" > "$PACKAGE_FILE"
        info "Using existing package: $ROBUSTMQ_3ENV_PACKAGE"
        return 0
    fi

    local args=()
    if [ "${ROBUSTMQ_3ENV_BUILD_FRONTEND:-false}" = "true" ]; then
        args+=(--with-frontend)
    fi

    info "Building RobustMQ package with scripts/build.sh ${args[*]:-}"
    (cd "$PROJECT_ROOT" && bash scripts/build.sh "${args[@]}")

    local package
    package="$(discover_package)"
    printf '%s\n' "$package" > "$PACKAGE_FILE"
    info "Package ready: $package"
}

discover_package() {
    local package
    package="$(find "$PROJECT_ROOT/build" -maxdepth 1 -type f -name 'robustmq-*.tar.gz' -printf '%T@ %p\n' 2>/dev/null | sort -nr | awk 'NR == 1 {print $2}')"
    [ -n "$package" ] || error "No build/robustmq-*.tar.gz package found. Run: bash chaos-test/robustmq-3env.sh build"
    printf '%s\n' "$package"
}

ensure_user_exists() {
    local user="$1"

    if id "$user" >/dev/null 2>&1; then
        info "User exists: $user"
        return 0
    fi

    info "Creating user: $user"
    useradd --create-home --shell /bin/bash "$user"
}

prepare_node_root() {
    local user="$1"
    local root="$2"
    local home_dir

    home_dir="$(dirname "$root")"
    if [ -e "$root" ]; then
        refuse_if_runtime_root_running "$root"

        if [ -f "$root/$MANAGED_MARKER" ]; then
            rm -rf "$root"
        elif [ "${ROBUSTMQ_3ENV_OVERWRITE:-false}" = "true" ]; then
            warn "Overwriting unmarked runtime root because ROBUSTMQ_3ENV_OVERWRITE=true: $root"
            rm -rf "$root"
        elif [ -z "$(find "$root" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]; then
            rm -rf "$root"
        else
            error "$root already exists and is not marked as managed by this script. Move it away or set ROBUSTMQ_3ENV_OVERWRITE=true explicitly."
        fi
    fi

    mkdir -p "$home_dir" "$root"
    touch "$root/$MANAGED_MARKER"
    chown -R "$user:$user" "$root"
}

extract_package_to_node() {
    local package="$1"
    local user="$2"
    local root="$3"

    tar -xzf "$package" -C "$root" --strip-components=1

    [ -x "$root/libs/broker-server" ] || error "broker-server not found or not executable under $root/libs"
    touch "$root/$MANAGED_MARKER"
    chown -R "$user:$user" "$root"
}

source_config_path() {
    local index="$1"
    local file="$2"

    printf '%s/%s/%s\n' "$CONFIG_SOURCE_DIR" "${NODES[$index]}" "$file"
}

runtime_server_config_path() {
    local index="$1"

    printf '%s/config/server.toml\n' "${RUNTIME_ROOTS[$index]}"
}

runtime_logger_config_path() {
    local index="$1"

    printf '%s/config/logger.toml\n' "${RUNTIME_ROOTS[$index]}"
}

require_source_configs() {
    local index="$1"
    local server_config logger_config

    server_config="$(source_config_path "$index" server.toml)"
    logger_config="$(source_config_path "$index" logger.toml)"

    [ -f "$server_config" ] || error "Missing source config: $server_config"
    [ -f "$logger_config" ] || error "Missing source config: $logger_config"
}

require_runtime_configs() {
    local index="$1"
    local server_config logger_config

    server_config="$(runtime_server_config_path "$index")"
    logger_config="$(runtime_logger_config_path "$index")"

    [ -f "$server_config" ] || error "Missing runtime config: $server_config. Run prepare before start."
    [ -f "$logger_config" ] || error "Missing runtime config: $logger_config. Run prepare before start."
}

copy_node_config() {
    local index="$1"
    local root="${RUNTIME_ROOTS[$index]}"

    require_source_configs "$index"
    cp "$(source_config_path "$index" server.toml)" "$root/config/server.toml"
    cp "$(source_config_path "$index" logger.toml)" "$root/config/logger.toml"
}

toml_scalar() {
    local file="$1"
    local section="$2"
    local key="$3"

    awk -v target_section="$section" -v target_key="$key" '
        function trim(value) {
            gsub(/^[ \t\r\n]+|[ \t\r\n]+$/, "", value)
            return value
        }

        /^[ \t]*($|#)/ {
            next
        }

        /^[ \t]*\[/ {
            current = $0
            sub(/^[ \t]*\[/, "", current)
            sub(/\][ \t]*$/, "", current)
            current = trim(current)
            next
        }

        {
            if (target_section == "" && current != "") {
                next
            }
            if (target_section != "" && current != target_section) {
                next
            }

            split($0, parts, "=")
            if (trim(parts[1]) != target_key) {
                next
            }

            sub(/^[^=]*=/, "", $0)
            value = trim($0)
            sub(/[ \t]*#.*/, "", value)
            value = trim(value)
            sub(/^"/, "", value)
            sub(/"$/, "", value)
            print value
            found = 1
            exit
        }

        END {
            if (!found) {
                exit 1
            }
        }
    ' "$file"
}

broker_id_for_node() {
    local index="$1"

    toml_scalar "$(runtime_server_config_path "$index")" "" broker_id
}

server_port_for_node() {
    local index="$1"
    local key="$2"

    toml_scalar "$(runtime_server_config_path "$index")" "" "$key"
}

section_port_for_node() {
    local index="$1"
    local section="$2"
    local key="$3"

    toml_scalar "$(runtime_server_config_path "$index")" "$section" "$key"
}

tokio_console_port_for_node() {
    local index="$1"
    local bind

    bind="$(toml_scalar "$(runtime_logger_config_path "$index")" tokio_console bind)"
    printf '%s\n' "${bind##*:}"
}

ports_for_node() {
    local index="$1"
    local grpc http storage mqtt mqtt_tls mqtt_ws mqtt_wss mqtt_quic
    local nats nats_tls nats_ws nats_wss kafka amqp tokio_console

    require_runtime_configs "$index"
    grpc="$(server_port_for_node "$index" grpc_port)"
    http="$(server_port_for_node "$index" http_port)"
    storage="$(section_port_for_node "$index" storage_runtime tcp_port)"
    mqtt="$(section_port_for_node "$index" mqtt_server tcp_port)"
    mqtt_tls="$(section_port_for_node "$index" mqtt_server tls_port)"
    mqtt_ws="$(section_port_for_node "$index" mqtt_server websocket_port)"
    mqtt_wss="$(section_port_for_node "$index" mqtt_server websockets_port)"
    mqtt_quic="$(section_port_for_node "$index" mqtt_server quic_port)"
    nats="$(section_port_for_node "$index" nats_runtime tcp_port)"
    nats_tls="$(section_port_for_node "$index" nats_runtime tls_port)"
    nats_ws="$(section_port_for_node "$index" nats_runtime ws_port)"
    nats_wss="$(section_port_for_node "$index" nats_runtime wss_port)"
    kafka="$(section_port_for_node "$index" kafka_runtime tcp_port)"
    amqp="$(section_port_for_node "$index" amqp_runtime tcp_port)"
    tokio_console="$(tokio_console_port_for_node "$index")"

    printf '%s\n' \
        "$grpc" "$http" "$storage" \
        "$mqtt" "$mqtt_tls" "$mqtt_ws" "$mqtt_wss" "$mqtt_quic" \
        "$nats" "$nats_tls" "$nats_ws" "$nats_wss" \
        "$kafka" "$amqp" "$tokio_console"
}

state_ports_for_node() {
    local index="$1"
    local grpc http storage mqtt nats kafka amqp tokio_console

    require_runtime_configs "$index"
    grpc="$(server_port_for_node "$index" grpc_port)"
    http="$(server_port_for_node "$index" http_port)"
    storage="$(section_port_for_node "$index" storage_runtime tcp_port)"
    mqtt="$(section_port_for_node "$index" mqtt_server tcp_port)"
    nats="$(section_port_for_node "$index" nats_runtime tcp_port)"
    kafka="$(section_port_for_node "$index" kafka_runtime tcp_port)"
    amqp="$(section_port_for_node "$index" amqp_runtime tcp_port)"
    tokio_console="$(tokio_console_port_for_node "$index")"

    printf 'grpc=%s,http=%s,storage=%s,mqtt=%s,nats=%s,kafka=%s,amqp=%s,tokio_console=%s\n' \
        "$grpc" "$http" "$storage" "$mqtt" "$nats" "$kafka" "$amqp" "$tokio_console"
}

prepare_node() {
    local index="$1"
    local user="${NODES[$index]}"
    local root="${RUNTIME_ROOTS[$index]}"
    local package="$2"

    ensure_user_exists "$user"
    prepare_node_root "$user" "$root"
    extract_package_to_node "$package" "$user" "$root"
    copy_node_config "$index"

    local broker_id
    broker_id="$(broker_id_for_node "$index")"

    mkdir -p \
        "$root/data/broker-${broker_id}/logs" \
        "$root/data/broker-${broker_id}/data" \
        "$root/data/broker-${broker_id}/engine"
    chown -R "$user:$user" "$root"
    info "Prepared $user at $root"
}

prepare() {
    require_root
    ensure_state_dir

    local package
    package="$(package_path)"
    [ -f "$package" ] || error "Package does not exist: $package"

    local i
    for i in "${!NODES[@]}"; do
        prepare_node "$i" "$package"
    done
}

all_ports() {
    local i
    for i in "${!NODES[@]}"; do
        ports_for_node "$i"
    done
}

validate_runtime_configs() {
    local i

    for i in "${!NODES[@]}"; do
        ports_for_node "$i" >/dev/null
        broker_id_for_node "$i" >/dev/null
    done
}

port_is_listening() {
    local port="$1"

    if command -v ss >/dev/null 2>&1; then
        ss -H -ltn "sport = :$port" 2>/dev/null | grep -q .
        return $?
    fi
    if command -v lsof >/dev/null 2>&1; then
        lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
        return $?
    fi
    if command -v netstat >/dev/null 2>&1; then
        netstat -ltn 2>/dev/null | grep -Eq "[.:]$port[[:space:]]"
        return $?
    fi
    if command -v nc >/dev/null 2>&1; then
        nc -z 127.0.0.1 "$port" >/dev/null 2>&1
        return $?
    fi

    error "Cannot check ports because none of ss, lsof, netstat, or nc is available."
}

preflight_ports() {
    local conflicts=()
    local port

    for port in $(all_ports | sort -n | uniq); do
        if port_is_listening "$port"; then
            conflicts+=("$port")
        fi
    done

    if [ "${#conflicts[@]}" -gt 0 ]; then
        error "Port(s) already in use: ${conflicts[*]}"
    fi
}

node_index() {
    local node="$1"
    local i

    for i in "${!NODES[@]}"; do
        if [ "${NODES[$i]}" = "$node" ]; then
            printf '%s\n' "$i"
            return 0
        fi
    done

    return 1
}

cmdline_matches_config() {
    local pid="$1"
    local config="$2"

    [ -r "/proc/$pid/cmdline" ] || return 1
    tr '\0' ' ' < "/proc/$pid/cmdline" | grep -F -- "$config" >/dev/null 2>&1
}

pid_running() {
    local pid="$1"
    [ -n "$pid" ] && kill -0 "$pid" >/dev/null 2>&1
}

node_running_for_config() {
    local config="$1"
    local pid

    if [ -f "$STATE_FILE" ]; then
        while IFS=$'\t' read -r _node _user pid recorded_config _ports; do
            [ "$recorded_config" = "$config" ] || continue
            if pid_running "$pid" && cmdline_matches_config "$pid" "$config"; then
                return 0
            fi
        done < "$STATE_FILE"
    fi

    ps -eo pid=,args= | grep -F -- "broker-server" | grep -F -- "$config" | grep -v grep >/dev/null 2>&1
}

refuse_if_runtime_root_running() {
    local root="$1"
    local config="$root/config/server.toml"

    if node_running_for_config "$config"; then
        error "$root has a running broker-server for config $config. Stop it before overwriting or cleaning this runtime root."
    fi
}

assert_not_running() {
    local i
    for i in "${!NODES[@]}"; do
        local config="${RUNTIME_ROOTS[$i]}/config/server.toml"
        if node_running_for_config "$config"; then
            error "${NODES[$i]} is already running for config $config"
        fi
    done
}

launch_node() {
    local index="$1"
    local user="${NODES[$index]}"
    local root="${RUNTIME_ROOTS[$index]}"
    local config="$root/config/server.toml"
    local broker_id stdout_log pid_file

    [ -x "$root/libs/broker-server" ] || error "Missing broker-server binary: $root/libs/broker-server"
    [ -f "$config" ] || error "Missing config: $config"
    broker_id="$(broker_id_for_node "$index")"
    stdout_log="$root/data/broker-${broker_id}/logs/process.log"
    pid_file="$root/data/broker-${broker_id}/broker.pid"

    command -v setsid >/dev/null 2>&1 || error "setsid is required to start broker-server outside the parent process group."

    runuser -u "$user" -- bash -c "cd '$root' || exit 1; nohup setsid ./libs/broker-server --conf '$config' </dev/null >> '$stdout_log' 2>&1 & echo \$! > '$pid_file'; disown" >/dev/null

    local pid
    pid="$(cat "$pid_file")"
    sleep 1
    if ! pid_running "$pid"; then
        error "Failed to start ${NODES[$index]}; check $stdout_log"
    fi
    printf '%s\n' "$pid"
}

node_process_log_path() {
    local index="$1"
    local root="${RUNTIME_ROOTS[$index]}"
    local broker_id

    broker_id="$(broker_id_for_node "$index")"
    printf '%s/data/broker-%s/logs/process.log\n' "$root" "$broker_id"
}

node_process_log_size() {
    local index="$1"
    local log_file

    log_file="$(node_process_log_path "$index")"
    if [ -f "$log_file" ]; then
        stat -c '%s' "$log_file"
    else
        printf '0\n'
    fi
}

print_node_log_tail() {
    local index="$1"
    local log_file

    log_file="$(node_process_log_path "$index")"
    warn "Last log lines for ${NODES[$index]}: $log_file"
    if [ -f "$log_file" ]; then
        tail -n 80 "$log_file" >&2 || true
    else
        warn "Log file not found: $log_file"
    fi
}

detect_startup_fatal() {
    local index="$1"
    local log_offset="${2:-0}"
    local log_file

    log_file="$(node_process_log_path "$index")"
    [ -f "$log_file" ] || return 1

    tail -c +"$((log_offset + 1))" "$log_file" 2>/dev/null | grep -E \
        "NodeCallManager global sender is not initialized|Failed to initialize inner topics|Timeout waiting for topic|thread '.*' panicked|panicked at|Address already in use|Permission denied" \
        >/dev/null 2>&1
}

wait_for_port() {
    local port="$1"
    local label="$2"
    local max_wait="${ROBUSTMQ_3ENV_READY_TIMEOUT:-90}"
    local elapsed=0

    while [ "$elapsed" -lt "$max_wait" ]; do
        if port_is_listening "$port"; then
            return 0
        fi
        sleep 2
        elapsed=$((elapsed + 2))
    done

    error "Timed out waiting for $label on port $port after ${max_wait}s"
}

wait_for_node_ready() {
    local index="$1"
    local pid="$2"
    local log_offset="$3"
    local node="${NODES[$index]}"
    local max_wait="${ROBUSTMQ_3ENV_READY_TIMEOUT:-90}"
    local elapsed=0
    local grpc_port http_port mqtt_port

    grpc_port="$(server_port_for_node "$index" grpc_port)"
    http_port="$(server_port_for_node "$index" http_port)"
    mqtt_port="$(section_port_for_node "$index" mqtt_server tcp_port)"

    while [ "$elapsed" -lt "$max_wait" ]; do
        if ! pid_running "$pid"; then
            print_node_log_tail "$index"
            error "$node exited before becoming ready; config=${RUNTIME_ROOTS[$index]}/config/server.toml"
        fi

        if detect_startup_fatal "$index" "$log_offset"; then
            print_node_log_tail "$index"
            error "$node reported a fatal startup error; config=${RUNTIME_ROOTS[$index]}/config/server.toml"
        fi

        if port_is_listening "$grpc_port" \
            && port_is_listening "$http_port" \
            && port_is_listening "$mqtt_port"; then
            info "$node ready: grpc=$grpc_port http=$http_port mqtt=$mqtt_port"
            return 0
        fi

        sleep 2
        elapsed=$((elapsed + 2))
    done

    print_node_log_tail "$index"
    error "Timed out waiting for $node readiness after ${max_wait}s; grpc=$grpc_port http=$http_port mqtt=$mqtt_port config=${RUNTIME_ROOTS[$index]}/config/server.toml"
}

port_state() {
    local port="$1"

    if port_is_listening "$port"; then
        printf 'listening'
    else
        printf 'closed'
    fi
}

state_port_value() {
    local ports="$1"
    local key="$2"

    tr ',' '\n' <<< "$ports" | awk -F= -v target="$key" '$1 == target {print $2; found = 1; exit} END {if (!found) exit 1}'
}

write_state_header() {
    ensure_state_dir
    : > "$STATE_FILE"
}

append_state() {
    local index="$1"
    local pid="$2"
    local ports

    ports="$(state_ports_for_node "$index")"
    printf '%s\t%s\t%s\t%s\t%s\n' \
        "${NODES[$index]}" \
        "${NODES[$index]}" \
        "$pid" \
        "${RUNTIME_ROOTS[$index]}/config/server.toml" \
        "$ports" >> "$STATE_FILE"
}

start_rollback_on_exit() {
    local exit_code=$?

    trap - EXIT
    if [ "${START_ROLLBACK_ACTIVE:-false}" = "true" ] && [ -s "$STATE_FILE" ]; then
        warn "Start failed; stopping nodes launched by this attempt"
        START_ROLLBACK_ACTIVE=false
        stop || true
    fi

    exit "$exit_code"
}

start() {
    require_root
    assert_not_running
    validate_runtime_configs
    preflight_ports
    write_state_header

    START_ROLLBACK_ACTIVE=true
    trap start_rollback_on_exit EXIT

    local i
    for i in "${!NODES[@]}"; do
        local pid
        local log_offset
        log_offset="$(node_process_log_size "$i")"
        pid="$(launch_node "$i")"
        append_state "$i" "$pid"
        info "Started ${NODES[$i]} pid=$pid"
        wait_for_node_ready "$i" "$pid" "$log_offset"
    done

    START_ROLLBACK_ACTIVE=false
    trap - EXIT
}

status() {
    if [ ! -f "$STATE_FILE" ]; then
        warn "No state file found: $STATE_FILE"
        return 1
    fi

    local node user pid config ports state index mqtt_state http_state mqtt_port http_port
    while IFS=$'\t' read -r node user pid config ports; do
        [ -n "$node" ] || continue
        if pid_running "$pid" && cmdline_matches_config "$pid" "$config"; then
            state="running"
        else
            state="stopped"
        fi

        if index="$(node_index "$node")"; then
            mqtt_port="$(state_port_value "$ports" mqtt || true)"
            http_port="$(state_port_value "$ports" http || true)"
            if [ -n "$mqtt_port" ]; then
                mqtt_state="$(port_state "$mqtt_port")"
            else
                mqtt_state="unknown"
            fi
            if [ -n "$http_port" ]; then
                http_state="$(port_state "$http_port")"
            else
                http_state="unknown"
            fi
        else
            mqtt_state="unknown"
            http_state="unknown"
        fi

        printf '%s\t%s\tpid=%s\t%s\tconfig=%s\t%s\tmqtt_status=%s\thttp_status=%s\n' \
            "$node" "$user" "$pid" "$state" "$config" "$ports" "$mqtt_state" "$http_state"
    done < "$STATE_FILE"

    return 0
}

stop() {
    require_root

    if [ ! -f "$STATE_FILE" ]; then
        warn "No state file found: $STATE_FILE"
        return 0
    fi

    local node user pid config ports
    while IFS=$'\t' read -r node user pid config ports; do
        [ -n "$node" ] || continue
        if pid_running "$pid" && cmdline_matches_config "$pid" "$config"; then
            info "Stopping $node pid=$pid"
            kill "$pid" >/dev/null 2>&1 || true
        else
            warn "Skip $node pid=$pid because it is not running or config path does not match"
        fi
    done < "$STATE_FILE"

    sleep 2

    while IFS=$'\t' read -r node user pid config ports; do
        [ -n "$node" ] || continue
        if pid_running "$pid" && cmdline_matches_config "$pid" "$config"; then
            warn "Force stopping $node pid=$pid"
            kill -9 "$pid" >/dev/null 2>&1 || true
        fi
    done < "$STATE_FILE"
}

clean() {
    require_root
    stop || true

    local root
    for root in "${RUNTIME_ROOTS[@]}"; do
        case "$root" in
            /home/node1/robustmq|/home/node2/robustmq|/home/node3/robustmq)
                if [ -f "$root/$MANAGED_MARKER" ]; then
                    refuse_if_runtime_root_running "$root"
                    rm -rf "$root"
                    info "Removed $root"
                elif [ -e "$root" ]; then
                    warn "Skip unmarked runtime root: $root"
                else
                    info "Runtime root already absent: $root"
                fi
                ;;
            *)
                error "Refusing to remove unexpected runtime root: $root"
                ;;
        esac
    done

    rm -rf "$STATE_DIR"
    info "Removed $STATE_DIR"
}

restart() {
    stop || true
    start
}

all() {
    build_package
    prepare
    start
    status
}

main() {
    local cmd="${1:-help}"
    case "$cmd" in
        build) build_package ;;
        prepare) prepare ;;
        start) start ;;
        status) status ;;
        stop) stop ;;
        restart) restart ;;
        clean) clean ;;
        all) all ;;
        help|-h|--help) usage ;;
        *) usage; error "Unknown command: $cmd" ;;
    esac
}

main "$@"
