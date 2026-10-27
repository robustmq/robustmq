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

import importlib.util
import pathlib
import unittest


SCRIPT = pathlib.Path(__file__).with_name("select-coverage-package.py")
SPEC = importlib.util.spec_from_file_location("select_coverage_package", SCRIPT)
selector = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(selector)


class SelectCoveragePackagesTest(unittest.TestCase):
    def setUp(self):
        self.packages = [
            (
                "common-metrics",
                pathlib.Path.cwd() / "src/common/metrics",
            ),
            (
                "system-info",
                pathlib.Path.cwd() / "src/common/system-info",
            ),
            ("robustmq-test", pathlib.Path.cwd() / "tests"),
        ]

    def test_selects_single_package(self):
        selected = selector.select_packages(
            ["src/common/system-info/src/lib.rs"], self.packages
        )

        self.assertEqual(selected, ["system-info"])

    def test_selects_multiple_packages(self):
        selected = selector.select_packages(
            [
                "src/common/system-info/src/lib.rs",
                "src/common/metrics/src/broker.rs",
            ],
            self.packages,
        )

        self.assertEqual(selected, ["common-metrics", "system-info"])

    def test_ignores_docs_and_dashboard_when_selecting_packages(self):
        selected = selector.select_packages(
            [
                "docs/en/Observability/Infrastructure-Metrics.md",
                "docs/zh/Observability/基础设施指标.md",
                "grafana/robustmq-broker.json",
                "src/common/system-info/src/lib.rs",
            ],
            self.packages,
        )

        self.assertEqual(selected, ["system-info"])

    def test_parses_nul_delimited_non_ascii_paths(self):
        paths = selector.parse_changed_paths(
            "docs/zh/Observability/基础设施指标.md\0"
            "src/common/system-info/src/lib.rs\0".encode()
        )

        self.assertEqual(
            paths,
            [
                "docs/zh/Observability/基础设施指标.md",
                "src/common/system-info/src/lib.rs",
            ],
        )

    def test_uses_workspace_fallback_for_unmapped_paths(self):
        selected = selector.select_packages(
            ["Cargo.toml", "src/common/system-info/src/lib.rs"], self.packages
        )

        self.assertIsNone(selected)

    def test_uses_workspace_fallback_for_integration_only_package(self):
        selected = selector.select_packages(["tests/Cargo.toml"], self.packages)

        self.assertIsNone(selected)

    def test_uses_workspace_fallback_when_only_non_code_files_changed(self):
        selected = selector.select_packages(
            ["docs/en/Observability/Infrastructure-Metrics.md"], self.packages
        )

        self.assertIsNone(selected)


if __name__ == "__main__":
    unittest.main()
