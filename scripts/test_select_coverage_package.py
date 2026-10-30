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
                "delay-message",
                pathlib.Path.cwd() / "src/delay-message",
            ),
            (
                "mqtt-broker",
                pathlib.Path.cwd() / "src/mqtt-broker",
            ),
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
            ["Cargo.toml", "src/not-a-workspace-package/generated.rs"], self.packages
        )

        self.assertIsNone(selected)

    def test_skips_integration_only_package(self):
        selected = selector.select_packages(["tests/Cargo.toml"], self.packages)

        self.assertEqual(selected, [])

    def test_ignores_integration_only_paths_when_unit_packages_changed(self):
        selected = selector.select_packages(
            [
                "src/delay-message/src/delay.rs",
                "src/mqtt-broker/src/mqtt/publish.rs",
                "tests/tests/mqtt/protocol/lastwill_message_test.rs",
                "tests/tests/mqtt/protocol/common.rs",
            ],
            self.packages,
        )

        self.assertEqual(selected, ["delay-message", "mqtt-broker"])

    def test_skips_integration_only_changes(self):
        selected = selector.select_packages(
            [
                "tests/tests/mqtt/protocol/lastwill_message_test.rs",
                "tests/tests/mqtt/protocol/common.rs",
            ],
            self.packages,
        )

        self.assertEqual(selected, [])

    def test_skips_when_only_non_code_files_changed(self):
        selected = selector.select_packages(
            ["docs/en/Observability/Infrastructure-Metrics.md"], self.packages
        )

        self.assertEqual(selected, [])

    def test_ignores_coverage_control_files_with_package_changes(self):
        selected = selector.select_packages(
            [
                "scripts/select-coverage-package.py",
                "src/common/system-info/src/lib.rs",
            ],
            self.packages,
        )

        self.assertEqual(selected, ["system-info"])

    def test_ignores_config_and_script_changes_with_package_changes(self):
        selected = selector.select_packages(
            [
                "scripts/cluster.sh",
                "config/server.toml.template",
                "src/common/system-info/src/lib.rs",
            ],
            self.packages,
        )

        self.assertEqual(selected, ["system-info"])

    def test_skips_when_only_config_and_script_files_change(self):
        selected = selector.select_packages(
            ["scripts/cluster.sh", "config/server.toml.template"], self.packages
        )

        self.assertEqual(selected, [])

    def test_current_delay_message_pr_scope(self):
        changed_paths = [
            "Cargo.toml",
            "Cargo.lock",
            ".github/workflows/security-audit.yml",
            ".github/workflows/coverage-unit.yml",
            "config/certs/ca.pem",
            "config/certs/cert.pem",
            "config/certs/key.pem",
            "config/server.toml.template",
            "docs/en/Configuration/BROKER.md",
            "docs/en/RobustMQ-MQTT/DelayMessage.md",
            "docs/zh/Configuration/BROKER.md",
            "docs/zh/RobustMQ-MQTT/DelayMessage.md",
            "scripts/cluster.sh",
            "scripts/ig-test.sh",
            "scripts/select-coverage-package.py",
            "scripts/test_select_coverage_package.py",
            "src/delay-message/src/delay.rs",
            "src/delay-message/src/manager.rs",
            "src/delay-message/src/pop.rs",
            "src/delay-message/src/recover.rs",
            "src/mqtt-broker/src/core/delay_message.rs",
            "src/mqtt-broker/src/core/error.rs",
            "src/mqtt-broker/src/core/offline_message.rs",
            "src/mqtt-broker/src/mqtt/publish.rs",
            "tests/tests/mqtt/protocol/common.rs",
            "tests/tests/mqtt/protocol/lastwill_message_test.rs",
        ]

        selected = selector.select_packages(changed_paths, self.packages)

        self.assertEqual(selected, ["delay-message", "mqtt-broker"])

    def test_uses_workspace_fallback_for_workspace_manifest_changes(self):
        for path in (
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "makefile",
        ):
            with self.subTest(path=path):
                selected = selector.select_packages([path], self.packages)
                self.assertIsNone(selected)

    def test_package_changes_take_precedence_over_workspace_files(self):
        selected = selector.select_packages(
            ["Cargo.toml", "Cargo.lock", "src/common/system-info/src/lib.rs"],
            self.packages,
        )

        self.assertEqual(selected, ["system-info"])

    def test_known_package_change_with_unmapped_path_uses_workspace_scope(self):
        selected = selector.select_packages(
            [
                "src/common/system-info/src/lib.rs",
                "src/generated-workspace-file.rs",
            ],
            self.packages,
        )

        self.assertIsNone(selected)

    def test_workspace_packages_are_all_mappable(self):
        package_roots = selector.workspace_packages()
        self.assertGreater(len(package_roots), 0)
        for package_name, package_root in package_roots:
            relative_root = package_root.relative_to(pathlib.Path.cwd())
            selected = selector.select_packages(
                [str(relative_root / "src/lib.rs")], package_roots
            )
            if package_name in selector.INTEGRATION_ONLY:
                self.assertEqual(selected, [])
            else:
                self.assertEqual(selected, [package_name])

    def test_uses_workspace_fallback_for_unmapped_source_paths(self):
        selected = selector.select_packages(
            ["src/not-a-workspace-package/generated.rs"], self.packages
        )

        self.assertIsNone(selected)


if __name__ == "__main__":
    unittest.main()
