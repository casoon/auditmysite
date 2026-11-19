"""Decision logic of the tag run's mutation coverage check; no network, no git."""

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    "check_mutation_coverage", Path(__file__).with_name("check-mutation-coverage.py")
)
coverage = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(coverage)


def jobs(n, conclusions=None, skip=()):
    out = [{"name": "Check & Test", "conclusion": "success"}]
    for k in range(1, n + 1):
        if k not in skip:
            out.append({"name": f"Detector mutation check {k}/{n}",
                        "conclusion": (conclusions or {}).get(k, "success")})
    return out


class CompleteAndGreenTests(unittest.TestCase):
    def test_all_shards_green(self):
        self.assertTrue(coverage.complete_and_green(jobs(8)))

    def test_called_workflow_prefix_is_accepted(self):
        prefixed = [{"name": f"Quality Gates (full CI) / {j['name']}", "conclusion": j["conclusion"]}
                    for j in jobs(4)]
        self.assertTrue(coverage.complete_and_green(prefixed))

    def test_a_failed_or_skipped_shard_is_not_coverage(self):
        self.assertFalse(coverage.complete_and_green(jobs(8, {3: "failure"})))
        self.assertFalse(coverage.complete_and_green(jobs(8, {3: "skipped"})))
        self.assertFalse(coverage.complete_and_green(jobs(8, {3: "cancelled"})))

    def test_a_missing_shard_is_not_coverage(self):
        self.assertFalse(coverage.complete_and_green(jobs(8, skip=(5,))))

    def test_a_run_without_mutation_jobs_is_not_coverage(self):
        self.assertFalse(coverage.complete_and_green([{"name": "Check & Test", "conclusion": "success"}]))

    def test_mixed_shard_counts_are_not_coverage(self):
        self.assertFalse(coverage.complete_and_green(jobs(4) + jobs(8)[1:]))


class AllowedChangesTests(unittest.TestCase):
    def test_changelog_only_is_covered(self):
        self.assertTrue(coverage.only_allowed(["CHANGELOG.md"]))
        self.assertTrue(coverage.only_allowed([]))

    def test_any_code_or_manifest_change_is_not(self):
        for path in ("src/wcag/rules/x.rs", "Cargo.toml", "Cargo.lock",
                     "tests/fixtures/rule_contracts/a.toml", "scripts/mutation-check.py",
                     ".github/workflows/ci.yml", "docs/guides/development.md"):
            self.assertFalse(coverage.only_allowed(["CHANGELOG.md", path]), path)


if __name__ == "__main__":
    unittest.main()
