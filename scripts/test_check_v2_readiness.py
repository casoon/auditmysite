"""Offline regression tests for the release gate; no GitHub mutations."""

import copy
import importlib.util
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from contextlib import redirect_stderr

DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("v2_readiness", DIRECTORY / "check-v2-readiness.py")
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


class ReleaseGateTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((DIRECTORY / "v2-readiness.json").read_text())
        numbers = [r["issue"] for r in self.manifest["requirements"]] + [self.manifest["tracker"]]
        self.issues = [{"number": n, "state": "closed", "state_reason": "completed",
                        "milestone": {"number": self.manifest["milestone"]}} for n in numbers]

    def test_complete_prerequisites_pass_for_final_and_prerelease(self):
        for version in ["2.0.0", "v2.0.0-rc.1", "3.0.0"]:
            self.assertEqual(gate.check(version, self.manifest, lambda _: self.issues), [])

    def test_every_required_issue_independently_blocks(self):
        for index in range(len(self.issues)):
            for state, reason in [("open", None), ("closed", "not_planned"), ("closed", "duplicate")]:
                issues = copy.deepcopy(self.issues)
                issues[index].update(state=state, state_reason=reason)
                self.assertTrue(gate.blockers(self.manifest, issues))

    def test_missing_or_moved_requirements_block(self):
        self.assertTrue(gate.blockers(self.manifest, self.issues[1:]))
        self.issues[0]["milestone"]["number"] += 1
        self.assertTrue(gate.blockers(self.manifest, self.issues))

    def test_new_milestone_issue_is_not_silently_excluded(self):
        self.issues.append({"number": 999999, "state": "open"})
        self.assertTrue(gate.blockers(self.manifest, self.issues))

    def test_deleted_or_duplicated_audit_requirement_is_invalid(self):
        for requirements in [self.manifest["requirements"][:-1], [self.manifest["requirements"][0]] * 26]:
            manifest = dict(self.manifest, requirements=requirements)
            with self.assertRaises(ValueError):
                gate.blockers(manifest, self.issues)

    def test_one_x_does_not_query_github(self):
        self.assertEqual(gate.check("1.8.1", self.manifest, lambda _: self.fail("unexpected API request")), [])
        with self.assertRaises(ValueError):
            gate.check("unknown", self.manifest)

    def test_api_error_and_invalid_response_fail_closed(self):
        for error in [TimeoutError("timeout"), ValueError("invalid JSON")]:
            with patch.object(gate, "urlopen", side_effect=error), patch("sys.argv", ["gate", "--version", "2.0.0"]), redirect_stderr(io.StringIO()):
                self.assertEqual(gate.main(), 1)
        response = io.BytesIO(b'{"message":"not an issue list"}')
        with patch.object(gate, "urlopen", return_value=response), self.assertRaises(ValueError):
            gate.fetch_issues(self.manifest)


if __name__ == "__main__":
    unittest.main()
