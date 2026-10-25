# MQTT CLI integration test

`run.sh` executes the functional MQTT command matrix against a fresh,
single-node broker. Before the read-only matrix, it creates representative
users, ACLs, blacklist and rewrite rules, a local-file connector, a schema and
binding, an auto-subscribe rule, a topic, a seed message, and a live subscriber.
This keeps the documented list responses populated while the same run still
verifies deletion and cleanup. It invokes the built `cli-command` binary as an
external process, uses unique resource names for every run, and removes only
the resources and broker process it created. `mqtt_flow.py` provides the
pseudo-terminal handling required by the interactive publish/subscribe CLI.

Performance, throughput, latency, and benchmark behavior are intentionally out
of scope.

## Prerequisites

- Rust toolchain required by `rust-toolchain.toml` and `cargo`.
- `curl`, `jq`, `rg`, and `python3` available on `PATH`.
- A Unix-like environment with signals, temporary directories, and TCP sockets.

The script builds `target/debug/cli-command` and `target/debug/broker-server`
when either binary is missing. Existing binaries can be supplied explicitly.

## Usage

From the repository root:

```bash
./scripts/test/cli/mqtt/run.sh
```

Optional binary paths and log retention:

```bash
./scripts/test/cli/mqtt/run.sh \
  --cli ./target/debug/cli-command \
  --broker ./target/debug/broker-server \
  --keep-logs
```

The script allocates free per-run ports for the Admin API, gRPC, MQTT, storage,
NATS, Kafka, and AMQP listeners. The broker configuration and data are written
under a temporary directory. On success that directory is removed unless
`--keep-logs` is set. On failure it is retained and its path is printed.

## Coverage

The read-only matrix runs every action in table and JSON mode:

- `overview`
- `session list`
- `subscribes list`
- `user list`
- `acl list`
- `blacklist list`
- `client list`
- `flapping-detect`
- `slow-subscribe list`
- `system-alarm list`
- `topic list`
- `topic-rewrite list`
- `connector list`
- `schema list`
- `schema list-bind`
- `auto-subscribe list`

Stateful workflows cover create, list/assert, delete, and list/assert-absent
for users, ACLs, blacklists, topic rewrite rules, local-file connectors,
schemas, schema bindings, and auto-subscribe rules. Duplicate creation and an
invalid QoS are expected to fail with a non-zero status. An unavailable Admin
API is also checked.

The MQTT section uses a pseudo-terminal for `subscribe`, publishes one
retained payload plus two QoS 1 payloads and one QoS 2 payload, asserts exactly
one delivery of each, and stops the subscriber with `SIGINT`. It also checks
EOF-based publish completion and invalid MQTT credentials.

## Verified Output Contract

Command-by-command examples and verified outputs for every MQTT CLI group are
documented in `docs/en/RobustMQ-Command/CLI_MQTT.md` under sections 3.1–3.10.
This section records the normalized mutation, interactive, and error contracts
used by the reusable harness.
The examples below are normalized from a passing run of `run.sh` on
2026-10-20. Replace values in angle brackets with values from the current run;
the field names, message text, and exit-status rules are the contract.

Successful mutations return exit status `0` and a short confirmation on
stdout:

```text
$ robust-ctl mqtt user create --username <run-user> --password <run-password>
Created successfully!
```

List commands in table mode print a human-readable heading and rows:

```text
user list result:
+----------+--------------+
| username | is_superuser |
+==========+==============+
| admin    | true         |
+----------+--------------+
```

JSON mode prints only a JSON document on stdout. Paginated lists use `data` and
`total_count`:

```json
{
  "data": [
    {
      "tenant": "default",
      "username": "admin",
      "is_superuser": true,
      "create_time": 1792462675
    }
  ],
  "total_count": 1
}
```

Resource-specific JSON rows preserve their API fields. For example, an ACL
created by the test contains `tenant`, `name`, `resource_type`,
`resource_name`, `topic`, `action`, and `permission`:

```json
{
  "data": [
    {
      "tenant": "default",
      "name": "<run-id>-acl",
      "desc": "",
      "resource_type": "ClientId",
      "resource_name": "<run-id>-client",
      "topic": "<run-id>/messages",
      "ip": "",
      "action": "All",
      "permission": "Allow"
    }
  ],
  "total_count": 1
}
```

Schema binding returns grouped rows. The `--tenant` argument is required so
the list query observes the same tenant as the bind operation:

```json
{
  "data": [
    { "data_type": "resource", "data": ["<run-id>-schema"] },
    { "data_type": "schema", "data": ["<run-id>/schema-topic"] }
  ],
  "total_count": 2
}
```

Interactive MQTT output is line-oriented. A passing flow includes one retained
payload, two QoS 1 payloads, one QoS 2 payload, and a clean Ctrl-C termination:

```text
able to connect: "127.0.0.1:<mqtt-port>"
subscribe success
payload: retained-<run-id>
payload: message-one-<run-id>
payload: message-two-<run-id>
payload: qos2-<run-id>
 Ctrl+C detected,  Please press ENTER to end the program.
End of input stream.
```

Expected failures return a non-zero status and preserve a diagnostic on
stderr. Examples from the passing run:

```text
# duplicate user: exit status 1
Exception:Server returned error: code=100, message=... User [<run-id>-user] already exist

# invalid QoS: exit status 1
Exception:Server returned error: code=400, message=... QoS must be 0, 1 or 2

# unavailable Admin API: exit status 1
Exception:Request failed: error sending request for url (http://127.0.0.1:1/api/mqtt/overview)
```

The banner is diagnostic output and may appear on stderr; scripts must parse
JSON only from stdout. The harness validates these contracts with `jq`, exact
exit statuses, and payload-count assertions.

A complete passing invocation ends with status `0` and this summary:

```text
$ ./scripts/test/cli/mqtt/run.sh
[mqtt] retained delivery, QoS 1/2 publish, exact-once messages, and Ctrl-C passed
All MQTT CLI integration checks passed.
```

The latest populated verified run executed 72 CLI invocations: 34 table/JSON
read-only checks against seeded resources, stateful resource checks, MQTT
publish/subscribe checks, and endpoint or credential checks. Duplicate user creation, duplicate auto-subscribe
creation, invalid QoS, unavailable Admin API, and invalid MQTT credentials are
expected non-zero cases asserted by the passing run.

## Cleanup

Cleanup runs from an `EXIT`, `INT`, or `TERM` trap. It stops the subscriber,
deletes resources in reverse dependency order, and terminates only the broker
PID started by this invocation. It does not use `pkill`, `killall`, or port-wide
process cleanup. A cleanup failure makes an otherwise passing run fail.
