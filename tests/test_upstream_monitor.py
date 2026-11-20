"""
Offline tests for upstream SHA observation. Never fetch live WhatsApp state.
Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].
"""
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("upstream_checker", ROOT / "scripts" / "check_whatsapp_upstream.py")
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


class OfficialUpstreamTrackingTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((ROOT / "config" / "upstream-sources.json").read_text())

    def test_observed_sources_are_exact_and_unique(self):
        entries = checker.validate_sources(self.manifest)
        self.assertEqual({x["name"] for x in entries}, {"whatsmeow", "baileys", "whatsapp-rust"})
        self.assertTrue(all(len(x["observed_sha"]) == 40 for x in entries))

    def test_detects_official_revision_drift_without_adopting_it(self):
        baselines = {row["name"]: row["observed_sha"] for row in self.manifest["sources"]}
        original = checker.report(self.manifest, baselines)
        self.assertEqual(original["updates_detected"], 0)
        current = dict(baselines)
        current["whatsmeow"] = "f" * 40
        changed = checker.report(self.manifest, current)
        self.assertEqual(changed["updates_detected"], 1)
        source = next(x for x in changed["results"] if x["name"] == "whatsmeow")
        self.assertTrue(source["requires_manual_review"])
        self.assertIn(source["baseline"], source["compare_url"])
        self.assertIn(source["latest"], source["compare_url"])

    def test_fails_closed_for_invalid_manifest(self):
        modified = json.loads(json.dumps(self.manifest))
        modified["sources"][0]["repository"] = "example.com/attacker/evil"
        with self.assertRaises(ValueError):
            checker.validate_sources(modified)
        current = {row["name"]: row["observed_sha"] for row in self.manifest["sources"]}
        current["whatsmeow"] = "latest"
        with self.assertRaises(ValueError):
            checker.report(self.manifest, current)


if __name__ == "__main__":
    unittest.main()
