#!/usr/bin/env python3
"""Contract tests for the reviewer-requested current-LEZ CI lane."""

from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
LEZ_TAG = "v0.2.4"
LEZ_COMMIT = "47eba256479f6f785acbd138834340703cd03401"
CIRCUITS_SHA256 = "db9d76f50f3536ac112067f2083d8aef3fab8024d1ef2f7a259b37490bc35eac"
THRESHOLD_IMAGE_ID = "e57b90a0fd306a8a6258f4fe7c045d056cf70142408e31a020817aaaba23bac4"
THRESHOLD_PROGRAM_SHA256 = "ff03aac0e9f900cef8c3011b76d6dd64b9d79ac9377731e1d892fe7cbc245d0f"


class CurrentLezContractTest(unittest.TestCase):
    def test_all_lez_dependencies_pin_current_stable_release(self) -> None:
        scaffold = (ROOT / "scaffold.toml").read_text()
        self.assertRegex(scaffold, rf'(?m)^pin = "{LEZ_COMMIT}"$')

        for relative in ("host/Cargo.toml", "methods/guest/Cargo.toml"):
            manifest = (ROOT / relative).read_text()
            stale_tags = re.findall(r'tag\s*=\s*"([^"]+)"', manifest)
            self.assertTrue(stale_tags, f"{relative} has no pinned LEZ tag")
            self.assertEqual(
                {LEZ_TAG},
                set(stale_tags),
                f"{relative} contains stale LEZ tags: {stale_tags}",
            )

    def test_ci_runs_real_sequencer_deploy_execute_and_inclusion_gate(self) -> None:
        script_path = ROOT / "scripts/lez-sequencer-e2e.sh"
        self.assertTrue(script_path.is_file(), "missing real sequencer E2E script")
        script = script_path.read_text()
        for token in (
            "RISC0_DEV_MODE=0",
            "--untracked-files=no",
            "--features standalone",
            "--metrics-address",
            "verify_and_execute_bytes.bin",
            "--deploy-program",
            "refusing deployment: supplied program id",
            "--submit",
            "--tx-hash",
            '"confirmed"',
            "included_block",
        ):
            self.assertIn(token, script, f"E2E script missing {token!r}")

        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        self.assertIn("test-sequencer-e2e:", workflow)
        self.assertIn("scripts/lez-sequencer-e2e.sh", workflow)
        self.assertIn(LEZ_COMMIT, workflow)
        self.assertRegex(workflow, r'RISC0_DEV_MODE:\s*["\']?0["\']?')
        self.assertEqual(2, workflow.count(CIRCUITS_SHA256))

    def test_ci_pins_reproducible_risc0_toolchain(self) -> None:
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        for token in (
            "cargo install rzup --version 0.5.1 --locked",
            "rzup install cargo-risczero 3.0.5",
            "rzup install r0vm 3.0.5",
            "rzup install rust 1.94.1",
        ):
            self.assertIn(token, workflow, f"CI missing reproducibility guard {token!r}")
        self.assertNotRegex(workflow, r"(?m)^\s*rzup install\s*$")
        self.assertNotIn("--skip wrapper_threshold_image_id", workflow)
        self.assertIn(
            "wrapper_threshold_image_id_matches_canonical_threshold_guest",
            (ROOT / "scripts/lez-sequencer-e2e.sh").read_text(),
        )
        self.assertEqual(
            2,
            workflow.count("libpcsclite-dev"),
            "both CI jobs compile PCSC-dependent LEZ workspace crates",
        )

    def test_canonical_threshold_program_is_hash_pinned(self) -> None:
        binary_path = ROOT / "methods/guest/elf/threshold_proof.bin"
        manifest_path = ROOT / "methods/guest/elf/threshold_proof.manifest.json"
        self.assertTrue(binary_path.is_file(), "missing canonical threshold ProgramBinary")
        self.assertTrue(manifest_path.is_file(), "missing canonical threshold manifest")

        manifest = json.loads(manifest_path.read_text())
        self.assertEqual(THRESHOLD_IMAGE_ID, manifest["image_id"])
        self.assertEqual(THRESHOLD_PROGRAM_SHA256, manifest["program_binary_sha256"])
        self.assertEqual(
            THRESHOLD_PROGRAM_SHA256,
            hashlib.sha256(binary_path.read_bytes()).hexdigest(),
        )

        public = json.loads(
            (ROOT / "submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json").read_text()
        )
        self.assertEqual(THRESHOLD_IMAGE_ID, public["proof"]["threshold_proof_image_id"])

        methods_lib = (ROOT / "methods/src/lib.rs").read_text()
        self.assertIn("canonical_threshold.rs", methods_lib)
        build_script = (ROOT / "methods/build.rs").read_text()
        for guard in (THRESHOLD_IMAGE_ID, THRESHOLD_PROGRAM_SHA256, "source_tree_sha256"):
            self.assertIn(guard, build_script)

    def test_cost_benchmark_generator_matches_current_public_evidence(self) -> None:
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts/benchmark-lez-costs.py"), "--check"],
            cwd=ROOT,
            text=True,
            capture_output=True,
        )
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def test_demo_scripts_separate_archived_and_current_evidence(self) -> None:
        demo_heavy = (ROOT / "scripts/demo-heavy-lane.sh").read_text()
        demo_video = (ROOT / "scripts/demo-video.sh").read_text()
        render_demo = (ROOT / "scripts/render-narrated-demo.py").read_text()
        narration = (ROOT / "scripts/narrate-demo.sh").read_text()
        legacy_smoke = (ROOT / "scripts/lez-localnet-smoke.sh").read_text()

        for text in (demo_heavy, demo_video):
            self.assertIn("archived", text.lower())
            self.assertIn("submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json", text)
            self.assertNotIn("Wrapper transaction confirmed in block 39548", text)

        self.assertIn("scripts/lez-sequencer-e2e.sh", demo_heavy)
        self.assertIn("--current-e2e", demo_heavy)
        self.assertIn("LEZ_V024_PUBLIC_E2E_EVIDENCE.json", render_demo)
        self.assertNotIn("cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697", render_demo)
        self.assertIn("block two thousand three hundred seventy-four", narration)
        self.assertIn("--current-e2e", legacy_smoke)
        self.assertNotIn("--live-submit", legacy_smoke)


if __name__ == "__main__":
    unittest.main()
