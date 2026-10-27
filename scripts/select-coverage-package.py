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
SELECTION_CONTROL_FILES = {
    ".github/workflows/coverage-unit.yml",
    "scripts/coverage-unit.sh",
    "scripts/select-coverage-package.py",
    "scripts/test_select_coverage_package.py",
}
NON_CODE_PREFIXES = ("docs/", "grafana/")


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
    """Return affected unit-test packages, or None for workspace fallback."""
    relevant_paths = [
        path for path in changed_paths if path not in SELECTION_CONTROL_FILES
    ]
    relevant_paths = [
        path for path in relevant_paths if not path.startswith(NON_CODE_PREFIXES)
    ]
    if not relevant_paths:
        return None

    selected: set[str] = set()
    for changed_path in relevant_paths:
        path = pathlib.Path(changed_path)
        matches = [
            name
            for name, root in packages
            if path.is_relative_to(root.relative_to(pathlib.Path.cwd()))
        ]
        if len(matches) != 1:
            return None
        selected.add(matches[0])

    if selected & INTEGRATION_ONLY:
        return None
    return sorted(selected)


def parse_changed_paths(data: bytes) -> list[str]:
    """Decode git's NUL-delimited paths without locale-dependent quoting."""
    return [path.decode("utf-8") for path in data.split(b"\0") if path]


def main() -> int:
    changed_paths = parse_changed_paths(sys.stdin.buffer.read())
    packages = select_packages(changed_paths, workspace_packages())
    if packages:
        print(",".join(packages))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
