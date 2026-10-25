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

set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
CLI_BIN="$ROOT_DIR/target/debug/cli-command"
BROKER_BIN="$ROOT_DIR/target/debug/broker-server"
KEEP_LOGS=false
RUN_DIR=""
BROKER_PID=""
SUBSCRIBER_PID=""
CLEANUP_STATUS=0
CLEANED=false
COMMAND_NUMBER=0
LAST_STDOUT=""
LAST_STDERR=""
USER_CREATED=false
ACL_CREATED=false
BLACKLIST_CREATED=false
REWRITE_CREATED=false
CONNECTOR_CREATED=false
SCHEMA_CREATED=false
SCHEMA_BOUND=false
AUTO_SUBSCRIBE_CREATED=false
SEED_SUBSCRIBER_READY=""

usage() {
    cat <<'USAGE'
Usage: scripts/test/cli/mqtt/run.sh [options]

Options:
  --cli PATH       cli-command binary (default: target/debug/cli-command)
  --broker PATH    broker-server binary (default: target/debug/broker-server)
  --keep-logs      preserve the temporary run directory after success
  -h, --help       show this help
USAGE
}

while (($# > 0)); do
    case "$1" in
        --cli)
            [[ $# -ge 2 ]] || { echo "--cli requires a path" >&2; exit 2; }
            CLI_BIN="$2"; shift 2 ;;
        --broker)
            [[ $# -ge 2 ]] || { echo "--broker requires a path" >&2; exit 2; }
            BROKER_BIN="$2"; shift 2 ;;
        --keep-logs) KEEP_LOGS=true; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

absolute_path() {
    case "$1" in
        /*) printf '%s\n' "$1" ;;
        *) printf '%s/%s\n' "$ROOT_DIR" "$1" ;;
    esac
}

CLI_BIN="$(absolute_path "$CLI_BIN")"
BROKER_BIN="$(absolute_path "$BROKER_BIN")"

require_command() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "Required command not found: $1" >&2
        exit 2
    }
}

for command_name in curl jq python3 rg; do
    require_command "$command_name"
done

if [[ ! -x "$CLI_BIN" || ! -x "$BROKER_BIN" ]]; then
    require_command cargo
    echo "Building CLI and broker binaries..."
    (cd "$ROOT_DIR" && cargo build --package cmd --bin cli-command --bin broker-server)
fi
[[ -x "$CLI_BIN" ]] || { echo "CLI binary is not executable: $CLI_BIN" >&2; exit 2; }
[[ -x "$BROKER_BIN" ]] || { echo "Broker binary is not executable: $BROKER_BIN" >&2; exit 2; }

RUN_ID="cli-mqtt-$(date +%Y%m%d%H%M%S)-$$-$(python3 -c 'import secrets; print(secrets.token_hex(4))')"
RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/robustmq-cli-mqtt.XXXXXX")"
LOG_DIR="$RUN_DIR/logs"
CONFIG_FILE="$RUN_DIR/server.toml"
LOGGER_CONFIG="$RUN_DIR/logger.toml"
mkdir -p "$LOG_DIR"

find_free_port() {
    python3 - <<'PY'
import socket
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
}

GRPC_PORT="$(find_free_port)"
HTTP_PORT="$(find_free_port)"
MQTT_PORT="$(find_free_port)"
MQTT_TLS_PORT="$(find_free_port)"
MQTT_WS_PORT="$(find_free_port)"
MQTT_WSS_PORT="$(find_free_port)"
MQTT_QUIC_PORT="$(find_free_port)"
STORAGE_PORT="$(find_free_port)"
NATS_PORT="$(find_free_port)"
KAFKA_PORT="$(find_free_port)"
AMQP_PORT="$(find_free_port)"
ADMIN_ADDR="127.0.0.1:$HTTP_PORT"
MQTT_ADDR="127.0.0.1:$MQTT_PORT"

TENANT=default
USER_NAME="${RUN_ID}-user"
USER_PASSWORD="${RUN_ID}-password"
ACL_NAME="${RUN_ID}-acl"
BLACKLIST_NAME="${RUN_ID}-blacklist"
REWRITE_NAME="${RUN_ID}-rewrite"
CONNECTOR_NAME="${RUN_ID}-connector"
SCHEMA_NAME="${RUN_ID}-schema"
AUTO_SUBSCRIBE_NAME="${RUN_ID}-auto"
CLIENT_RESOURCE="${RUN_ID}-client"
SOURCE_TOPIC="${RUN_ID}/source"
DEST_TOPIC="${RUN_ID}/destination"
SCHEMA_TOPIC="${RUN_ID}/schema-topic"
MQTT_TOPIC="${RUN_ID}/messages"
CONNECTOR_FILE="$RUN_DIR/connector.jsonl"

sed "s#\./data/logs#$RUN_DIR/logs/logger#g" "$ROOT_DIR/config/logger.toml" > "$LOGGER_CONFIG"

cat > "$CONFIG_FILE" <<EOF
cluster_name = "$RUN_ID"
broker_id = 1
broker_ip = "127.0.0.1"
roles = ["meta", "broker", "engine"]
grpc_port = $GRPC_PORT
http_port = $HTTP_PORT
meta_addrs = { 1 = "127.0.0.1:$GRPC_PORT" }
data_path = "$RUN_DIR/data"

[log]
log_config = "$LOGGER_CONFIG"
log_path = "$RUN_DIR/logs/broker"

[storage_runtime]
tcp_port = $STORAGE_PORT
data_path = ["$RUN_DIR/engine"]
io_thread_num = 2

[mqtt_runtime]
broker_worker_threads = 4

[mqtt_runtime.auth]
default_user = "admin"
default_password = "robustmq"
durable_sessions_enable = false
secret_free_login = false
is_self_protection_status = false

[mqtt_runtime.server]
tcp_port = $MQTT_PORT
tls_port = $MQTT_TLS_PORT
websocket_port = $MQTT_WS_PORT
websockets_port = $MQTT_WSS_PORT
quic_port = $MQTT_QUIC_PORT

[nats_runtime]
tcp_port = $NATS_PORT

[kafka_runtime]
tcp_port = $KAFKA_PORT

[amqp_runtime]
tcp_port = $AMQP_PORT
EOF

show_log() {
    local label="$1" stream="$2" file="$3"
    if [[ -s "$file" ]]; then
        printf '\n[%s %s]\n' "$label" "$stream" >&2
        sed -n '1,120p' "$file" >&2 || true
    fi
}

stop_subscriber() {
    if [[ -n "$SUBSCRIBER_PID" ]] && kill -0 "$SUBSCRIBER_PID" 2>/dev/null; then
        kill -INT "$SUBSCRIBER_PID" 2>/dev/null || true
        for _ in {1..30}; do
            kill -0 "$SUBSCRIBER_PID" 2>/dev/null || break
            sleep 0.2
        done
        kill -TERM "$SUBSCRIBER_PID" 2>/dev/null || true
        wait "$SUBSCRIBER_PID" 2>/dev/null || true
    fi
    SUBSCRIBER_PID=""
}

start_seed_subscriber() {
    SEED_SUBSCRIBER_READY="$RUN_DIR/seed-subscriber.ready"
    python3 "$SCRIPT_DIR/mqtt_flow.py" seed-subscriber "$CLI_BIN" "$ADMIN_ADDR" \
        "$MQTT_ADDR" "$MQTT_TOPIC" "$SEED_SUBSCRIBER_READY" \
        "$LOG_DIR/seed-subscriber.stdout" >"$LOG_DIR/seed-subscriber.runner.log" 2>&1 &
    SUBSCRIBER_PID=$!
    for _ in {1..100}; do
        [[ -e "$SEED_SUBSCRIBER_READY" ]] && return 0
        if ! kill -0 "$SUBSCRIBER_PID" 2>/dev/null; then
            echo "Seed subscriber exited before readiness." >&2
            show_log seed-subscriber runner "$LOG_DIR/seed-subscriber.runner.log"
            return 1
        fi
        sleep 0.2
    done
    echo "Seed subscriber did not become ready." >&2
    show_log seed-subscriber runner "$LOG_DIR/seed-subscriber.runner.log"
    return 1
}

cleanup_cli() {
    local label="$1"; shift
    local file="$LOG_DIR/cleanup-${label}.log"
    "$CLI_BIN" mqtt --server "$ADMIN_ADDR" --mqtt-server "$MQTT_ADDR" "$@" >"$file" 2>&1 || {
        CLEANUP_STATUS=1
        echo "Cleanup failed for $label; see $file" >&2
    }
}

cleanup() {
    local status=$?
    [[ "$CLEANED" == true ]] && exit "$status"
    CLEANED=true
    trap - EXIT INT TERM
    set +e
    stop_subscriber
    if [[ -n "$BROKER_PID" ]] && kill -0 "$BROKER_PID" 2>/dev/null; then
        [[ "$AUTO_SUBSCRIBE_CREATED" == true ]] && cleanup_cli auto-subscribe-delete auto-subscribe delete --tenant "$TENANT" --name "$AUTO_SUBSCRIBE_NAME"
        [[ "$SCHEMA_BOUND" == true ]] && cleanup_cli schema-unbind schema unbind --tenant "$TENANT" --schema-name "$SCHEMA_NAME" --resource-name "$SCHEMA_TOPIC"
        [[ "$SCHEMA_CREATED" == true ]] && cleanup_cli schema-delete schema delete --tenant "$TENANT" --schema-name "$SCHEMA_NAME"
        [[ "$CONNECTOR_CREATED" == true ]] && cleanup_cli connector-delete connector delete --tenant "$TENANT" --connector-name "$CONNECTOR_NAME"
        [[ "$REWRITE_CREATED" == true ]] && cleanup_cli rewrite-delete topic-rewrite delete --tenant "$TENANT" --name "$REWRITE_NAME"
        [[ "$BLACKLIST_CREATED" == true ]] && cleanup_cli blacklist-delete blacklist delete --tenant "$TENANT" --name "$BLACKLIST_NAME"
        [[ "$ACL_CREATED" == true ]] && cleanup_cli acl-delete acl delete --tenant "$TENANT" --name "$ACL_NAME"
        [[ "$USER_CREATED" == true ]] && cleanup_cli user-delete user delete --username "$USER_NAME"
        kill -TERM "$BROKER_PID" 2>/dev/null || true
        for _ in {1..50}; do
            kill -0 "$BROKER_PID" 2>/dev/null || break
            sleep 0.2
        done
        kill -KILL "$BROKER_PID" 2>/dev/null || true
        wait "$BROKER_PID" 2>/dev/null || true
    fi
    (( status == 0 && CLEANUP_STATUS != 0 )) && status=1
    if [[ "$KEEP_LOGS" == true || $status -ne 0 ]]; then
        echo "CLI MQTT test logs: $RUN_DIR" >&2
    else
        rm -rf "$RUN_DIR"
    fi
    exit "$status"
}

trap cleanup EXIT INT TERM

echo "Run ID: $RUN_ID"
echo "Admin API: $ADMIN_ADDR"
echo "MQTT TCP: $MQTT_ADDR"
echo "Logs: $RUN_DIR"
echo "Starting isolated broker..."
(cd "$ROOT_DIR" && exec "$BROKER_BIN" --conf "$CONFIG_FILE") >"$LOG_DIR/broker.stdout" 2>"$LOG_DIR/broker.stderr" &
BROKER_PID=$!

ready=false
for _ in {1..120}; do
    if ! kill -0 "$BROKER_PID" 2>/dev/null; then
        echo "Broker exited before readiness." >&2
        show_log broker stderr "$LOG_DIR/broker.stderr"
        exit 1
    fi
    if curl -fsS "http://$ADMIN_ADDR/health/ready" >/dev/null 2>&1; then
        ready=true
        break
    fi
    sleep 0.5
done
[[ "$ready" == true ]] || { echo "Broker did not become ready." >&2; show_log broker stderr "$LOG_DIR/broker.stderr"; exit 1; }

run_command() {
    local label="$1" expected_status="$2" server="$3" mqtt_server="$4"
    shift 4
    COMMAND_NUMBER=$((COMMAND_NUMBER + 1))
    local safe_label prefix actual_status
    safe_label="$(printf '%s' "$label" | tr -c '[:alnum:]_.-' '_')"
    prefix="$LOG_DIR/$(printf '%03d' "$COMMAND_NUMBER")-${safe_label}"
    LAST_STDOUT="$prefix.stdout"
    LAST_STDERR="$prefix.stderr"
    set +e
    "$CLI_BIN" mqtt --server "$server" --mqtt-server "$mqtt_server" "$@" >"$LAST_STDOUT" 2>"$LAST_STDERR"
    actual_status=$?
    set -e
    printf '[%03d] %-42s status=%d\n' "$COMMAND_NUMBER" "$label" "$actual_status"
    if (( actual_status != expected_status )); then
        echo "Expected status $expected_status for $label, got $actual_status" >&2
        show_log "$label" stdout "$LAST_STDOUT"
        show_log "$label" stderr "$LAST_STDERR"
        return 1
    fi
}

run_cli() {
    local label="$1" expected_status="$2"
    shift 2
    run_command "$label" "$expected_status" "$ADMIN_ADDR" "$MQTT_ADDR" "$@"
}

run_cli_json() {
    local label="$1"; shift
    run_cli "$label (json)" 0 --output json "$@"
    jq empty "$LAST_STDOUT" >/dev/null || { echo "Invalid JSON output for $label" >&2; show_log "$label" stdout "$LAST_STDOUT"; return 1; }
}

run_cli_table() {
    local label="$1"; shift
    run_cli "$label (table)" 0 "$@"
}

assert_stdout_contains() {
    rg -F -- "$1" "$LAST_STDOUT" >/dev/null || { echo "Expected '$1' in $LAST_STDOUT" >&2; show_log assertion stdout "$LAST_STDOUT"; return 1; }
}

assert_stdout_absent() {
    if rg -F -- "$1" "$LAST_STDOUT" >/dev/null; then
        echo "Did not expect '$1' in $LAST_STDOUT" >&2
        show_log assertion stdout "$LAST_STDOUT"
        return 1
    fi
}

assert_json_contains() {
    jq -e --arg expected "$1" '.. | strings | select(. == $expected)' "$LAST_STDOUT" >/dev/null || {
        echo "Expected JSON value '$1' in $LAST_STDOUT" >&2
        show_log assertion stdout "$LAST_STDOUT"
        return 1
    }
}

run_cli_json_until_contains() {
    local label="$1" expected="$2"
    shift 2
    for _ in {1..40}; do
        run_cli_json "$label" "$@"
        if jq -e --arg expected "$expected" '.. | strings | select(. == $expected)' "$LAST_STDOUT" >/dev/null; then
            return 0
        fi
        sleep 0.25
    done
    echo "Timed out waiting for JSON value '$expected' in $label" >&2
    show_log "$label" stdout "$LAST_STDOUT"
    return 1
}

echo "Creating resources for populated read-only command examples..."
start_seed_subscriber
run_cli seed-publish 0 publish --username admin --password robustmq --topic "$MQTT_TOPIC" --qos 1 <<< "seed-payload-$RUN_ID"

run_cli user-create 0 user create --username "$USER_NAME" --password "$USER_PASSWORD"
USER_CREATED=true
run_cli acl-create 0 acl create --tenant "$TENANT" --name "$ACL_NAME" --resource-type ClientId --resource-name "$CLIENT_RESOURCE" --topic "$MQTT_TOPIC" --action All --permission Allow
ACL_CREATED=true
BLACKLIST_END_TIME="$(( $(date +%s) + 3600 ))"
run_cli blacklist-create 0 blacklist create --tenant "$TENANT" --name "$BLACKLIST_NAME" --blacklist-type ClientId --resource-name "$CLIENT_RESOURCE" --end-time "$BLACKLIST_END_TIME" --desc "$RUN_ID blacklist"
BLACKLIST_CREATED=true
run_cli rewrite-create 0 topic-rewrite create --tenant "$TENANT" --name "$REWRITE_NAME" --action All --source-topic "$SOURCE_TOPIC" --dest-topic "$DEST_TOPIC" --regex '^.*$' --desc "$RUN_ID rewrite"
REWRITE_CREATED=true
CONNECTOR_CONFIG="{\"local_file_path\":\"$CONNECTOR_FILE\",\"rotation_strategy\":\"none\"}"
run_cli connector-create 0 connector create --tenant "$TENANT" --connector-name "$CONNECTOR_NAME" --connector-type file --config "$CONNECTOR_CONFIG" --topic-name "$MQTT_TOPIC"
CONNECTOR_CREATED=true
SCHEMA_DOCUMENT='{"type":"object","properties":{"value":{"type":"string"}}}'
run_cli schema-create 0 schema create --tenant "$TENANT" --schema-name "$SCHEMA_NAME" --schema-type json --schema "$SCHEMA_DOCUMENT" --desc "$RUN_ID schema"
SCHEMA_CREATED=true
run_cli schema-bind 0 schema bind --tenant "$TENANT" --schema-name "$SCHEMA_NAME" --resource-name "$SCHEMA_TOPIC"
SCHEMA_BOUND=true
run_cli auto-subscribe-create 0 auto-subscribe create --tenant "$TENANT" --name "$AUTO_SUBSCRIBE_NAME" --topic "$MQTT_TOPIC" --qos 1
AUTO_SUBSCRIBE_CREATED=true

echo "Running read-only MQTT command matrix..."
run_cli_table overview overview; run_cli_json overview overview
run_cli_table session-list session list; run_cli_json session-list session list
run_cli_table subscribes-list subscribes list; run_cli_json subscribes-list subscribes list
run_cli_table user-list user list; run_cli_json user-list user list
run_cli_table acl-list acl list; run_cli_json acl-list acl list
run_cli_table blacklist-list blacklist list; run_cli_json blacklist-list blacklist list
run_cli_table client-list client list; run_cli_json client-list client list
run_cli_table flapping-detect flapping-detect; run_cli_json flapping-detect flapping-detect
run_cli_table slow-subscribe-list slow-subscribe list; run_cli_json slow-subscribe-list slow-subscribe list
run_cli_table system-alarm-list system-alarm list; run_cli_json system-alarm-list system-alarm list
run_cli_table topic-list topic list; run_cli_json topic-list topic list
run_cli_table session-list-page-2 --page 2 --limit 50 session list
run_cli_json client-list-page-1 --page 1 --limit 20 client list
run_cli_table topic-rewrite-list topic-rewrite list; run_cli_json topic-rewrite-list topic-rewrite list
run_cli_table connector-list connector list --connector-name "$CONNECTOR_NAME"
run_cli_json connector-list connector list --connector-name "$CONNECTOR_NAME"
run_cli_table schema-list schema list --schema-name "$SCHEMA_NAME"
run_cli_json schema-list schema list --schema-name "$SCHEMA_NAME"
run_cli_table schema-list-bind schema list-bind --tenant "$TENANT" --schema-name "$SCHEMA_NAME" --resource-name "$SCHEMA_TOPIC"
run_cli_json schema-list-bind schema list-bind --tenant "$TENANT" --schema-name "$SCHEMA_NAME" --resource-name "$SCHEMA_TOPIC"
run_cli_table auto-subscribe-list auto-subscribe list; run_cli_json auto-subscribe-list auto-subscribe list

echo "Running user workflow..."
run_cli user-create-duplicate 1 user create --username "$USER_NAME" --password "$USER_PASSWORD"
run_cli_table user-list-created user list; assert_stdout_contains "$USER_NAME"
run_cli_json user-list-created user list; assert_json_contains "$USER_NAME"
run_cli user-delete 0 user delete --username "$USER_NAME"
USER_CREATED=false
run_cli_json user-list-deleted user list; assert_stdout_absent "$USER_NAME"

echo "Running ACL workflow..."
run_cli_json acl-list-created acl list; assert_json_contains "$ACL_NAME"
run_cli acl-delete 0 acl delete --tenant "$TENANT" --name "$ACL_NAME"
ACL_CREATED=false
run_cli_json acl-list-deleted acl list; assert_stdout_absent "$ACL_NAME"

echo "Running blacklist workflow..."
run_cli_json blacklist-list-created blacklist list; assert_json_contains "$BLACKLIST_NAME"
run_cli blacklist-delete 0 blacklist delete --tenant "$TENANT" --name "$BLACKLIST_NAME"
BLACKLIST_CREATED=false
run_cli_json blacklist-list-deleted blacklist list; assert_stdout_absent "$BLACKLIST_NAME"

echo "Running topic rewrite workflow..."
run_cli_json rewrite-list-created topic-rewrite list; assert_json_contains "$REWRITE_NAME"
run_cli rewrite-delete 0 topic-rewrite delete --tenant "$TENANT" --name "$REWRITE_NAME"
REWRITE_CREATED=false
run_cli_json rewrite-list-deleted topic-rewrite list; assert_stdout_absent "$REWRITE_NAME"

echo "Running connector workflow..."
run_cli_json connector-list-created connector list --connector-name "$CONNECTOR_NAME"; assert_json_contains "$CONNECTOR_NAME"
run_cli connector-delete 0 connector delete --tenant "$TENANT" --connector-name "$CONNECTOR_NAME"
CONNECTOR_CREATED=false
run_cli_json connector-list-deleted connector list --connector-name "$CONNECTOR_NAME"; assert_stdout_absent "$CONNECTOR_NAME"

echo "Running schema workflow..."
run_cli_json schema-list-created schema list --schema-name "$SCHEMA_NAME"; assert_json_contains "$SCHEMA_NAME"
run_cli_json_until_contains schema-bind-list "$SCHEMA_TOPIC" schema list-bind --tenant "$TENANT" --schema-name "$SCHEMA_NAME" --resource-name "$SCHEMA_TOPIC"
run_cli schema-unbind 0 schema unbind --tenant "$TENANT" --schema-name "$SCHEMA_NAME" --resource-name "$SCHEMA_TOPIC"
SCHEMA_BOUND=false
run_cli schema-delete 0 schema delete --tenant "$TENANT" --schema-name "$SCHEMA_NAME"
SCHEMA_CREATED=false
run_cli_json schema-list-deleted schema list --schema-name "$SCHEMA_NAME"; assert_stdout_absent "$SCHEMA_NAME"

echo "Running auto-subscribe workflow..."
run_cli_json auto-subscribe-list-created auto-subscribe list; assert_json_contains "$AUTO_SUBSCRIBE_NAME"
run_cli auto-subscribe-duplicate 1 auto-subscribe create --tenant "$TENANT" --name "$AUTO_SUBSCRIBE_NAME" --topic "$MQTT_TOPIC" --qos 1
run_cli auto-subscribe-delete 0 auto-subscribe delete --tenant "$TENANT" --name "$AUTO_SUBSCRIBE_NAME"
AUTO_SUBSCRIBE_CREATED=false
run_cli_json auto-subscribe-list-deleted auto-subscribe list; assert_stdout_absent "$AUTO_SUBSCRIBE_NAME"

echo "Running invalid-input and endpoint checks..."
run_cli invalid-qos 1 auto-subscribe create --tenant "$TENANT" --name "${RUN_ID}-invalid-qos" --topic "$MQTT_TOPIC" --qos 9
run_command unavailable-admin 1 "127.0.0.1:1" "$MQTT_ADDR" overview

echo "Running MQTT publish/subscribe checks..."
python3 "$SCRIPT_DIR/mqtt_flow.py" "$CLI_BIN" "$ADMIN_ADDR" "$MQTT_ADDR" "$MQTT_TOPIC" "$RUN_ID" "$LOG_DIR"
run_command invalid-mqtt-credentials 1 "$ADMIN_ADDR" "$MQTT_ADDR" publish --username "${RUN_ID}-missing" --password invalid --topic "$MQTT_TOPIC" --qos 1 < /dev/null

echo "All MQTT CLI integration checks passed."
