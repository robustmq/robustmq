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
NON_CODE_PREFIXES = ("docs/", "grafana/", "config/certs/")
IGNORED_FILES = {
    ".github/workflows/coverage-unit.yml",
    ".github/workflows/security-audit.yml",
    "config/server.toml.template",
    "scripts/cluster.sh",
    "scripts/coverage-unit.sh",
    "scripts/ig-test.sh",
    "scripts/select-coverage-package.py",
    "scripts/test_select_coverage_package.py",
}
WORKSPACE_FILES = {
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "Makefile",
    "makefile",
}


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
    relevant_paths = [
        path
        for path in changed_paths
        if not path.startswith(NON_CODE_PREFIXES) and path not in IGNORED_FILES
    ]
    if not relevant_paths:
        return []

    selected: set[str] = set()
    has_workspace_file = False
    for changed_path in relevant_paths:
        path = pathlib.Path(changed_path)
        matches = [
            name
            for name, root in packages
            if path.is_relative_to(root.relative_to(pathlib.Path.cwd()))
        ]
        if len(matches) != 1:
            if changed_path in WORKSPACE_FILES:
                has_workspace_file = True
                continue
            return None
        package = matches[0]
        if package not in INTEGRATION_ONLY:
            selected.add(package)

    # A package-specific source change is the narrowest useful unit-coverage
    # scope even when Cargo manifests or the lockfile also changed.
    if selected:
        return sorted(selected)
    if has_workspace_file:
        return None
    return sorted(selected)


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
