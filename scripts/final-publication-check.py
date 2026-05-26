#!/usr/bin/env python3
"""Strict LP-0002 final-publication gate.

This script is intentionally harsher than validate-submission-readiness.py. It
answers one question only: is this repository ready to be submitted to the public
lambda-prize repo as a real candidate, not merely a local implementation demo?

It must fail until the external/public evidence exists.
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]
errors: list[str] = []
warnings: list[str] = []


def read(path: str) -> str:
    p = root / path
    if not p.exists():
        errors.append(f"missing required publication file: {path}")
        return ""
    return p.read_text(errors="replace")


def git_output(*args: str) -> str:
    try:
        return subprocess.check_output(["git", *args], cwd=root, text=True, stderr=subprocess.STDOUT).strip()
    except subprocess.CalledProcessError as exc:
        return ""

# 1. Public clone URL.
origin = git_output("remote", "get-url", "origin")
if not origin:
    errors.append("no git origin remote configured; final submission needs a public clone URL")
elif not re.search(r"github\.com[:/].+/.+", origin):
    errors.append(f"origin remote is not a GitHub-style public URL: {origin}")

module = {}
try:
    module = json.loads(read("module.json"))
except json.JSONDecodeError as exc:
    errors.append(f"module.json is invalid JSON: {exc}")
repo = str(module.get("repository", ""))
if not re.match(r"https://github\.com/.+/.+", repo):
    errors.append("module.json repository must be a public https://github.com/... URL, not a placeholder")

solution = read("solutions/LP-0002.md")
for phrase in ["pending public release", "TBD", "to be recorded", "placeholder"]:
    if phrase.lower() in solution.lower():
        errors.append(f"solutions/LP-0002.md still contains publication placeholder phrase: {phrase}")
required_solution_sections = [
    "## Summary",
    "## Repository",
    "## Approach",
    "## Success Criteria Checklist",
    "### Functionality",
    "### Usability",
    "### Reliability",
    "### Performance",
    "### Supportability",
    "## FURPS Self-Assessment",
    "## Terms & Conditions",
]
for section in required_solution_sections:
    if section not in solution:
        errors.append(f"solutions/LP-0002.md missing upstream validator section: {section}")
if not re.search(r"^\s*-\s+\[[xX]\]", solution, re.MULTILINE):
    errors.append("solutions/LP-0002.md must include checked success-criteria checklist items")
if not re.search(r"^\*\*Submitted by:\*\*.+[A-Za-z]", solution, re.MULTILINE):
    errors.append("solutions/LP-0002.md must include a non-empty Submitted by field")
if not re.search(r"Terms (&|and) Conditions|TERMS\.md", solution, re.I):
    errors.append("solutions/LP-0002.md must acknowledge Terms & Conditions")
if not re.search(r"https://(?:www\.)?(youtube\.com|youtu\.be|vimeo\.com|loom\.com)/\S+|https://github\.com/[^\s)]+\.(?:mp4|mov|m4v)", solution):
    errors.append("solutions/LP-0002.md must include an accessible narrated demo video URL (YouTube/Vimeo/Loom or public GitHub video asset)")
if not re.search(r"https://github\.com/.+/.+", solution):
    errors.append("solutions/LP-0002.md must include the public implementation repository URL")

# 2. LEZ testnet evidence. For LP-0002, the evaluator/public testnet target is lgs/NSSA localnet.
text_surface = "\n".join([
    read("docs/PROTOCOL.md"),
    read("docs/SPEC_COMPLIANCE.md"),
    read("submission/BENCHMARKS.md"),
    read("docs/HEAVY_LANE_ROADMAP.md"),
    solution,
])
if "localnet" in text_surface.lower() and "public testnet" not in text_surface.lower():
    warnings.append("docs mention localnet but not public testnet; check claim surface")
if re.search(r"LEZ public testnet/evaluator deployment|LEZ testnet deployment", json.dumps(module)):
    errors.append("module.json still lists LEZ testnet deployment as pending")
if "Public testnet deployment remains" in text_surface or "public testnet/evaluator deployment remains" in text_surface or "No public LEZ testnet deployment yet" in text_surface:
    errors.append("docs still state public LEZ testnet deployment remains pending")

# Require a structured evidence file so reviewers can audit all public tx ids.
testnet_evidence = root / "submission/TESTNET_EVIDENCE.json"
if not testnet_evidence.exists():
    errors.append("missing submission/TESTNET_EVIDENCE.json with public LEZ testnet program/multisig/proposal/execute tx evidence")
else:
    try:
        ev = json.loads(testnet_evidence.read_text())
        required = ["network", "verifier_program_id", "multisig_instance", "proposal_tx", "approval_txs", "execute_tx"]
        for key in required:
            if not ev.get(key):
                errors.append(f"TESTNET_EVIDENCE.json missing {key}")
        if str(ev.get("network", "")).lower() in {"localnet", "localhost"}:
            note = str(ev.get("network_interpretation", "")).lower()
            if "public testnet" not in note and "evaluator" not in note:
                errors.append("TESTNET_EVIDENCE.json uses localnet but does not record the LP-0002 evaluator/public-testnet interpretation")
        execute = ev.get("execute_tx", {})
        if isinstance(execute, dict):
            if not execute.get("confirmed") or not execute.get("tx_hash"):
                errors.append("TESTNET_EVIDENCE.json execute_tx must record a confirmed localnet transaction hash")
        if ev.get("status") != "confirmed" or ev.get("confirmed") is not True:
            errors.append("TESTNET_EVIDENCE.json must record confirmed localnet/evaluator inclusion evidence")
    except json.JSONDecodeError as exc:
        errors.append(f"TESTNET_EVIDENCE.json invalid JSON: {exc}")

# 3. Basecamp: static HTML is not enough after LP-0005 rejection.
basecamp = root / "basecamp-app"
if not (basecamp / "CMakeLists.txt").exists():
    errors.append("Basecamp app is still static/web-only: missing basecamp-app/CMakeLists.txt for a loadable native module")
if not list(basecamp.glob("**/*.qml")):
    errors.append("Basecamp app missing QML UI files required for Logos app/Basecamp loading")
if not list(basecamp.glob("**/*.cpp")):
    errors.append("Basecamp app missing native plugin .cpp source")
if not (basecamp / "metadata.json").exists() and not (basecamp / "manifest.json").exists():
    errors.append("Basecamp app missing metadata.json/manifest.json load metadata")
if not (root / "submission" / "BASECAMP_NATIVE_BUILD.md").exists():
    errors.append("missing submission/BASECAMP_NATIVE_BUILD.md with native Basecamp build evidence")

# 4. Validation/CI signal.
# The automation token available for publishing this repository may not have the
# GitHub `workflow` scope, so the final gate accepts either an actual workflow or
# an explicit evaluator-side validation evidence note. The commands remain the
# same in both cases.
workflow_dir = root / ".github" / "workflows"
workflows = list(workflow_dir.glob("*.yml")) + list(workflow_dir.glob("*.yaml"))
ci_files = workflows + [p for p in [root / ".gitlab-ci.yml", root / "Jenkinsfile", root / ".circleci" / "config.yml"] if p.exists()]
validation_surface = ""
if not ci_files:
    errors.append("missing CI config required by upstream lambda-prize validator (.github/workflows, .gitlab-ci.yml, Jenkinsfile, or .circleci/config.yml)")
    validation_surface = read("submission/CI_EVIDENCE.md")
else:
    validation_surface = "\n".join(p.read_text(errors="replace") for p in ci_files if p.is_file())
for token in ["cargo test --workspace", "validate-submission-readiness.py"]:
    if token not in validation_surface:
        errors.append(f"validation evidence missing local validation token: {token}")
if "lez-localnet-smoke.sh" not in validation_surface and "demo-heavy-lane.sh" not in validation_surface:
    errors.append("validation evidence lacks LEZ/RISC0 evidence smoke command")

evaluator_guide = read("submission/EVALUATOR_GUIDE.md")
for token in ["./demo.sh", "final-publication-check.py", "demo-heavy-lane.sh", "TESTNET_EVIDENCE.json", "submission/proof-artifacts"]:
    if token not in evaluator_guide:
        errors.append(f"submission/EVALUATOR_GUIDE.md missing evaluator command/evidence token: {token}")

proof_artifacts = root / "submission" / "proof-artifacts" / "lp0002-risc0-fixture-new"
for rel in [
    "receipt.borsh",
    "journal.borsh",
    "manifest.txt",
    "lez-execution.json",
    "spel-adapter-evidence.json",
    "nssa-submit-dry-run.json",
    "nssa-submit-evidence.json",
]:
    p = proof_artifacts / rel
    if not p.exists() or p.stat().st_size == 0:
        errors.append(f"missing bundled heavy-lane proof artifact: {p.relative_to(root)}")

# 4b. Claim-surface sanity: do not ship stale blocker/TODO language in evaluator docs.
claim_surface = "\n".join([
    text_surface,
    read("submission/FINAL_PUBLICATION_AUDIT.md"),
    read("submission/PR_DRAFT.md"),
    evaluator_guide,
])
for phrase in [
    "TODO: replace with public GitHub URL",
    "TODO: attach",
    "DEMO_PENDING",
    "Narrated demo video | PENDING",
    "REPLACED-BY-CLARIFICATION",
    "not yet implemented",
    "Full public completion still requires repeating this on public testnet",
]:
    if phrase.lower() in claim_surface.lower():
        errors.append(f"stale publication phrase remains in evaluator-facing docs: {phrase}")

# 5. Benchmarks and CU/cost evidence.
bench_text = read("submission/BENCHMARKS.md")
if not re.search(r"CU|compute unit", bench_text, re.I):
    errors.append("submission/BENCHMARKS.md must discuss LEZ compute-unit cost")
lez_cost_path = root / "submission" / "LEZ_COST_BENCHMARKS.json"
if not lez_cost_path.exists():
    errors.append("missing submission/LEZ_COST_BENCHMARKS.json machine-readable LEZ cost evidence")
else:
    try:
        cost = json.loads(lez_cost_path.read_text())
        for key in ["instruction_data_len_bytes", "accounts", "receipt_bytes_len", "tx_hash", "cu_metering"]:
            if key not in cost:
                errors.append(f"LEZ_COST_BENCHMARKS.json missing {key}")
        cu = cost.get("cu_metering", {})
        if cu.get("available") is False and not cu.get("reason"):
            errors.append("LEZ_COST_BENCHMARKS.json says CU metering unavailable but gives no reason")
    except json.JSONDecodeError as exc:
        errors.append(f"LEZ_COST_BENCHMARKS.json invalid JSON: {exc}")

# 6. License: current spec says MIT OR Apache-2.0.
license_text = read("LICENSE")
if "MIT License" not in license_text and "Apache License" not in license_text:
    errors.append("LICENSE must contain MIT or Apache-2.0 text")

if errors:
    print("NO-GO: LP-0002 is not ready for real public submission")
    for err in errors:
        print(f"FAIL: {err}")
    for warn in warnings:
        print(f"WARN: {warn}")
    sys.exit(1)

print("GO: LP-0002 final-publication gate passed")
