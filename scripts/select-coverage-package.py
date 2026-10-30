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

"""Select safe package-scoped coverage targets from changed repository paths."""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys


INTEGRATION_ONLY = {"robustmq-test", "grpc-clients"}


def workspace_packages() -> list[tuple[str, pathlib.Path]]:
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        check=True,
        capture_output=True,
        text=True,
    )
    packages = json.loads(metadata.stdout)["packages"]
    return [
        (package["name"], pathlib.Path(package["manifest_path"]).parent.resolve())
        for package in packages
    ]


def select_packages(
    changed_paths: list[str], packages: list[tuple[str, pathlib.Path]]
) -> list[str] | None:
    """Return affected unit-test packages, None for workspace, or [] to skip."""
    selected: set[str] = set()
    for changed_path in changed_paths:
        path = pathlib.Path(changed_path)
        # Workspace tests are selected from Rust crate changes only. Docs,
        # configs, scripts, workflows, root manifests, and integration tests
        # live outside src/ and must not expand a run to the whole workspace.
        if not path.parts or path.parts[0] != "src":
            continue

        matches = [
            (name, root)
            for name, root in packages
            if path.is_relative_to(root.relative_to(pathlib.Path.cwd()))
        ]
        if matches:
            # Pick the deepest package root in case a workspace has nested crates.
            package, _ = max(matches, key=lambda match: len(match[1].parts))
            if package not in INTEGRATION_ONLY:
                selected.add(package)
            continue

        # A source path outside every known Cargo package is ambiguous. Keep
        # the conservative workspace fallback for this exceptional case.
        return None

    # A package-specific source change is the narrowest useful unit-coverage
    # scope even when Cargo manifests or the lockfile also changed.
    if selected:
        return sorted(selected)
    return []


def parse_changed_paths(data: bytes) -> list[str]:
    """Decode git's NUL-delimited paths without locale-dependent quoting."""
    return [path.decode("utf-8") for path in data.split(b"\0") if path]


def main() -> int:
    changed_paths = parse_changed_paths(sys.stdin.buffer.read())
    packages = select_packages(changed_paths, workspace_packages())
    if packages is None:
        print("workspace")
    elif packages:
        print(",".join(packages))
    else:
        print("skip")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
