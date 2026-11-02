"""Exercise quick-CI selection against real Git histories, without network."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("quick-ci-scope.sh").resolve()


class QuickScopeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git("init", "-q")
        self.git("config", "user.email", "ci-test@example.invalid")
        self.git("config", "user.name", "CI scope test")
        self.change("README.md", "base")
        self.base = self.commit()

    def git(self, *args):
        return subprocess.check_output(
            ["git", *args], cwd=self.root, text=True, stderr=subprocess.DEVNULL
        ).strip()

    def change(self, name, value):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value)

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-qm", "fixture")
        return self.git("rev-parse", "HEAD")

    def scope(self, base=None, event="push"):
        return subprocess.check_output(
            ["bash", str(SCRIPT)], cwd=self.root, text=True,
            env={**os.environ, "BASE_SHA": base or self.base,
                 "HEAD_SHA": self.git("rev-parse", "HEAD"), "EVENT_NAME": event},
        ).strip()

    def test_prose_and_site_only(self):
        self.change("README.md", "edited")
        self.change("docs/guides/nested/a page.md", "guide")
        self.change("site/src/page.astro", "site")
        self.commit()
        self.assertEqual(self.scope(), "rust=false")

    def test_build_and_test_inputs_are_not_prose(self):
        for path in ["docs/OUTPUT_CONTRACT.md", "docs/report.schema.json",
                     "docs/calibration.md", "docs/PARITY_CONTRACT.jsonc",
                     "src/main.rs", "Cargo.lock", "rust-toolchain.toml",
                     ".github/workflows/ci.yml", "tests/fixtures/page.html"]:
            with self.subTest(path=path):
                self.change(path, "input")
                self.commit()
                self.assertEqual(self.scope(), "rust=true")
                self.git("reset", "--hard", self.base)

    def test_all_commits_in_push_are_considered(self):
        self.change("src/main.rs", "code")
        self.commit()
        self.change("README.md", "latest prose")
        self.commit()
        self.assertEqual(self.scope(), "rust=true")

    def test_pull_request_uses_merge_base(self):
        self.change("README.md", "PR prose")
        head = self.commit()
        self.git("checkout", "-q", self.base)
        self.change("src/main.rs", "unrelated base advance")
        base = self.commit()
        self.git("checkout", "-q", head)
        self.assertEqual(self.scope(base, "pull_request"), "rust=false")

    def test_rename_from_build_input_to_prose_still_runs(self):
        self.change("src/contract.rs", "contract")
        self.base = self.commit()
        (self.root / "docs/guides").mkdir(parents=True)
        self.git("mv", "src/contract.rs", "docs/guides/contract.md")
        self.commit()
        self.assertEqual(self.scope(), "rust=true")

    def test_unknown_or_zero_base_runs_checks(self):
        for base in ["0" * 40, "f" * 40]:
            self.assertEqual(self.scope(base), "rust=true")


if __name__ == "__main__":
    unittest.main()
