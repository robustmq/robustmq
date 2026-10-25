#!/usr/bin/env python3
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

"""Exercise CLI MQTT publish/subscribe with a pseudo-terminal subscriber."""

import os
import pty
import select
import signal
import subprocess
import sys
import time
from pathlib import Path


def fail(message: str, output: bytes = b"") -> None:
    if output:
        print(output.decode("utf-8", errors="replace"), file=sys.stderr)
    raise SystemExit(message)


def read_until(fd: int, output: bytearray, needle: bytes, timeout: float) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if needle in output:
            return True
        ready, _, _ = select.select([fd], [], [], min(0.2, deadline - time.monotonic()))
        if not ready:
            continue
        try:
            chunk = os.read(fd, 4096)
        except OSError:
            return False
        if not chunk:
            return False
        output.extend(chunk)
    return needle in output


def read_until_exit(pid: int, fd: int, output: bytearray, timeout: float) -> int:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.2)
        if ready:
            try:
                chunk = os.read(fd, 4096)
            except OSError:
                chunk = b""
            output.extend(chunk)
        child, status = os.waitpid(pid, os.WNOHANG)
        if child == pid:
            return os.waitstatus_to_exitcode(status)
    os.kill(pid, signal.SIGTERM)
    _, status = os.waitpid(pid, 0)
    fail("Subscriber did not exit after Ctrl-C", bytes(output))
    return os.waitstatus_to_exitcode(status)


def run_publish(
    cli: str,
    admin: str,
    mqtt: str,
    topic: str,
    qos: int,
    payload: str,
    log_path: Path,
    retained: bool = False,
) -> None:
    command = [
        cli,
        "mqtt",
        "--server",
        admin,
        "--mqtt-server",
        mqtt,
        "publish",
        "--username",
        "admin",
        "--password",
        "robustmq",
        "--topic",
        topic,
        "--qos",
        str(qos),
    ]
    if retained:
        command.append("--retained")
    result = subprocess.run(
        command,
        input=f"{payload}\n",
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    log_path.with_suffix(".stdout").write_text(result.stdout)
    log_path.with_suffix(".stderr").write_text(result.stderr)
    if result.returncode != 0:
        fail(
            f"Publish command exited with {result.returncode}: {command!r}\n{result.stderr}"
        )


def run_seed_subscriber(
    cli: str, admin: str, mqtt: str, topic: str, ready_path: Path, output_path: Path
) -> None:
    """Keep one real subscriber connected while read-only commands are sampled."""
    command = [
        cli,
        "mqtt",
        "--server",
        admin,
        "--mqtt-server",
        mqtt,
        "subscribe",
        "--username",
        "admin",
        "--password",
        "robustmq",
        "--topic",
        topic,
        "--qos",
        "1",
    ]
    pid, fd = pty.fork()
    if pid == 0:
        os.execv(cli, command)

    output = bytearray()
    if not read_until(fd, output, b"subscribe success", 20):
        os.kill(pid, signal.SIGTERM)
        read_until_exit(pid, fd, output, 5)
        fail("Seed subscriber failed to report readiness", bytes(output))

    ready_path.touch()

    def stop_child(_signum: int, _frame: object) -> None:
        try:
            os.kill(pid, signal.SIGINT)
        except ProcessLookupError:
            pass

    signal.signal(signal.SIGINT, stop_child)
    signal.signal(signal.SIGTERM, stop_child)
    status = read_until_exit(pid, fd, output, 3600)
    os.close(fd)
    output_path.write_bytes(output)
    if status != 0:
        fail(f"Seed subscriber exited with status {status}", bytes(output))


def main() -> None:
    if len(sys.argv) >= 2 and sys.argv[1] == "seed-subscriber":
        if len(sys.argv) != 8:
            fail(
                "Usage: mqtt_flow.py seed-subscriber CLI ADMIN MQTT TOPIC READY OUTPUT"
            )
        _, _, cli, admin, mqtt, topic, ready_path, output_path = sys.argv
        run_seed_subscriber(
            cli, admin, mqtt, topic, Path(ready_path), Path(output_path)
        )
        return

    cli, admin, mqtt, topic, run_id, log_dir = sys.argv[1:]
    logs = Path(log_dir)
    payloads = [
        f"retained-{run_id}",
        f"message-one-{run_id}",
        f"message-two-{run_id}",
        f"qos2-{run_id}",
    ]

    command = [
        cli,
        "mqtt",
        "--server",
        admin,
        "--mqtt-server",
        mqtt,
        "subscribe",
        "--username",
        "admin",
        "--password",
        "robustmq",
        "--topic",
        topic,
        "--qos",
        "1",
    ]
    pid, fd = pty.fork()
    if pid == 0:
        os.execv(cli, command)

    output = bytearray()
    if not read_until(fd, output, b"subscribe success", 20):
        os.kill(pid, signal.SIGTERM)
        read_until_exit(pid, fd, output, 5)
        fail("Subscriber failed to report readiness", bytes(output))

    run_publish(
        cli,
        admin,
        mqtt,
        topic,
        1,
        payloads[0],
        logs / "publish-retained",
        retained=True,
    )
    run_publish(cli, admin, mqtt, topic, 1, payloads[1], logs / "publish-one")
    run_publish(cli, admin, mqtt, topic, 1, payloads[2], logs / "publish-two")
    run_publish(cli, admin, mqtt, topic, 2, payloads[3], logs / "publish-qos2")

    if not read_until(fd, output, f"payload: {payloads[-1]}".encode(), 20):
        os.kill(pid, signal.SIGINT)
        read_until_exit(pid, fd, output, 5)
        fail("Subscriber did not receive all expected MQTT messages", bytes(output))

    os.kill(pid, signal.SIGINT)
    status = read_until_exit(pid, fd, output, 10)
    os.close(fd)
    (logs / "subscriber.stdout").write_bytes(output)
    (logs / "subscriber.stderr").write_bytes(b"")
    if status != 0:
        fail(f"Subscriber exited with status {status}", bytes(output))

    text = output.decode("utf-8", errors="replace")
    for payload in payloads:
        count = text.count(f"payload: {payload}")
        if count != 1:
            fail(f"Expected one delivery for {payload!r}, got {count}", bytes(output))

    print(
        "[mqtt] retained delivery, QoS 1/2 publish, exact-once messages, and Ctrl-C passed"
    )


if __name__ == "__main__":
    main()
