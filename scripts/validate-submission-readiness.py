#!/usr/bin/env python3
"""Validate LP-0002 local implementation readiness.

This is intentionally stricter than a file-presence checklist: it parses JSON,
validates SPEL discriminators, checks public-claim honesty gates, verifies the
native Basecamp package structure, and by default runs the Rust test suite plus the
clone-and-run consumer demo.

Important: this is NOT the final publication gate. A real LP-0002 submission
also needs a public repo URL, narrated video URL, LEZ public testnet evidence,
formal CU evidence or maintainer-accepted replacement, CI green on the public
default branch, and a real Logos Basecamp-loadable app. Use
`scripts/final-publication-check.py` for that stricter NO-GO/GO gate.

Use `--skip-exec` only when a fast structural check is needed; final local
readiness should run without it.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parents[1]

parser = argparse.ArgumentParser()
parser.add_argument("--skip-exec", action="store_true", help="skip cargo test and demo.sh execution")
args = parser.parse_args()

required_files = [
    "README.md",
    "LICENSE",
    "demo.sh",
    "module.json",
    "Cargo.toml",
    "docs/PROTOCOL.md",
    "docs/CONSUMER_INTEGRATION.md",
    "docs/SPEC_COMPLIANCE.md",
    "docs/DEMO_VIDEO_SCRIPT.md",
    "docs/HEAVY_LANE_ROADMAP.md",
    "scaffold.toml",
    "scripts/demo-video.sh",
    "scripts/lez-localnet-smoke.sh",
    "scripts/run-risc0-fixture-m4.sh",
    "interfaces/lp0002.spel",
    "interfaces/lp0002.idl.json",
    # Static HTML browser preview removed in favor of full native Qt/QML package
    "basecamp-app/qml/Lp0002PrivateMultisig.qml",
    "solutions/LP-0002.md",
    "submission/BENCHMARKS.md",
    "submission/MAINTAINER_CLARIFICATION.md",
    "submission/FINAL_PUBLICATION_AUDIT.md",
    "submission/PR_DRAFT.md",
    "submission/EVALUATOR_GUIDE.md",
    "submission/BASECAMP_NATIVE_BUILD.md",
    "submission/LEZ_COST_BENCHMARKS.json",
    "scripts/final-publication-check.py",
    "scripts/benchmark-lez-costs.py",
    "scripts/validate-basecamp-native.sh",
    "core/src/lib.rs",
    "verifier-program/src/lib.rs",
    "sdk/src/lib.rs",
    "consumer-demo/src/main.rs",
    "consumer-demo/examples/bench.rs",
]
required_crates = [
    "core",
    "verifier-program",
    "sdk",
    "consumer-demo",
    "methods",
    "methods/guest",
    "host",
    "lez-program",
]
errors: list[str] = []
notes: list[str] = []


def run(cmd: list[str], *, env: dict[str, str] | None = None) -> None:
    printable = " ".join(cmd)
    print(f"CHECK: {printable}")
    merged_env = os.environ.copy()
    path_entries = [str(Path.home() / ".cargo" / "bin"), str(Path.home() / "bin")]
    for candidate in [
        "/root/.cargo/bin",
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/Applications/Docker.app/Contents/Resources/bin",
    ]:
        if Path(candidate).exists():
            path_entries.append(candidate)
    path_entries.append(merged_env.get("PATH", ""))
    merged_env["PATH"] = os.pathsep.join(path_entries)
    if env:
        merged_env.update(env)
    result = subprocess.run(cmd, cwd=root, env=merged_env, text=True, capture_output=True)
    if result.returncode != 0:
        errors.append(
            f"Command failed ({printable}) with exit {result.returncode}\n"
            f"stdout:\n{result.stdout[-4000:]}\n"
            f"stderr:\n{result.stderr[-4000:]}"
        )
    else:
        if result.stdout.strip():
            notes.append(f"{printable}: ok")


# Required file/crate checks.
missing = [p for p in required_files if not (root / p).exists()]
if missing:
    errors.append(f"Missing required files: {missing}")

for crate in required_crates:
    if not (root / crate / "Cargo.toml").exists():
        errors.append(f"Missing crate: {crate}/Cargo.toml")

# JSON and IDL checks.
idl = None
module = None
for json_file in ["interfaces/lp0002.idl.json", "module.json"]:
    try:
        parsed = json.loads((root / json_file).read_text())
        if json_file.endswith("idl.json"):
            idl = parsed
        else:
            module = parsed
    except (json.JSONDecodeError, FileNotFoundError) as exc:
        errors.append(f"{json_file}: {exc}")

if idl:
    instructions = idl.get("instructions", [])
    if len(instructions) < 5:
        errors.append(f"interfaces/lp0002.idl.json: expected at least 5 instructions, got {len(instructions)}")
    if not idl.get("errors"):
        errors.append("interfaces/lp0002.idl.json: no errors defined")
    seen_errors = set()
    for err in idl.get("errors", []):
        code = err.get("code")
        if code in seen_errors:
            errors.append(f"Duplicate IDL error code: {code}")
        seen_errors.add(code)
    for instr in instructions:
        name = instr.get("name")
        disc = instr.get("discriminator", [])
        expected = list(hashlib.sha256(f"global:{name}".encode()).digest()[:8])
        if disc != expected:
            errors.append(f"IDL discriminator mismatch for {name}: got {disc}, expected {expected}")

    spel = (root / "interfaces/lp0002.spel").read_text()
    for instr in instructions:
        name = instr.get("name")
        if f"instruction {name}" not in spel:
            errors.append(f"interfaces/lp0002.spel missing instruction: {name}")

if module:
    if module.get("safe_lane") is not True:
        errors.append("module.json must explicitly mark safe_lane=true")
    if module.get("heavy_lane") is not True:
        errors.append("module.json must explicitly mark heavy_lane=true once RISC0/public-testnet evidence is claimed")
    if module.get("repository") == "TBD":
        errors.append("module.json repository must not be bare TBD")
    completed = module.get("heavy_lane_completed", [])
    pending = module.get("heavy_lane_pending", [])
    if "RISC0 zkVM guest program" not in completed:
        errors.append("module.json must list RISC0 zkVM guest program as completed")
    testnet_evidence = root / "submission" / "TESTNET_EVIDENCE.json"
    if not testnet_evidence.exists():
        if not pending or not any("testnet" in item.lower() for item in pending):
            errors.append("module.json must list public LEZ devnet/testnet deployment as pending until public evidence exists")
    else:
        try:
            ev = json.loads(testnet_evidence.read_text())
            if str(ev.get("network", "")).lower() not in {"testnet", "public-testnet", "lez-testnet"}:
                errors.append("TESTNET_EVIDENCE.json network must be the public LEZ testnet (https://testnet.lez.logos.co/), not localnet")
        except json.JSONDecodeError as exc:
            errors.append(f"submission/TESTNET_EVIDENCE.json: {exc}")

# Documentation surface checks.
compliance = (root / "docs/SPEC_COMPLIANCE.md").read_text()
required_sections = ["Functionality", "Usability", "Reliability", "Performance", "Supportability"]
for section in required_sections:
    if section not in compliance:
        errors.append(f"SPEC_COMPLIANCE.md missing section: {section}")

protocol = (root / "docs/PROTOCOL.md").read_text()
required_protocol = [
    "Threshold Proof Scheme",
    "Nullifier Design",
    "LEZ Account Model",
    "Security Assumptions",
    "Known Limitations",
    "Integration Guide",
    "Error Codes",
]
for section in required_protocol:
    if section not in protocol:
        errors.append(f"PROTOCOL.md missing section: {section}")

for phrase in ["deterministic mock", "public LEZ testnet", "safe-lane"]:
    if phrase not in compliance and phrase not in protocol:
        errors.append(f"Missing honesty gate: {phrase!r}")

claim_surface = "\n".join([
    compliance,
    protocol,
    (root / "README.md").read_text(),
    (root / "submission/PR_DRAFT.md").read_text(),
    (root / "submission/EVALUATOR_GUIDE.md").read_text(),
])
for forbidden in [
    "PASS-SAFE-LANE",
    "RISC0_DEV_MODE=1 semantics",
    "demo.sh runs RISC0_DEV_MODE=0",
    "TODO: replace with public GitHub URL",
    "TODO: attach",
    "DEMO_PENDING",
    "Narrated demo video | PENDING",
    "REPLACED-BY-CLARIFICATION",
    "not yet implemented",
    "Full public completion still requires repeating this on public testnet",
]:
    if forbidden.lower() in claim_surface.lower():
        errors.append(f"Evaluator-facing docs contain stale/ambiguous phrase: {forbidden}")

clarification = (root / "submission/MAINTAINER_CLARIFICATION.md").read_text()
if "one reproducible multisig" not in clarification.lower() or "consumer-demo" not in clarification.lower():
    errors.append("MAINTAINER_CLARIFICATION.md must describe the current one-instance criterion and consumer-demo evidence")

# Basecamp app checks.
# Static HTML browser preview removed; native Qt/QML package is the primary surface.
# Static files are intentionally absent from this validator.
qml_main = root / "basecamp-app" / "qml" / "Lp0002PrivateMultisig.qml"
if qml_main.exists():
    qml_text = qml_main.read_text()
    if "lp0002" not in qml_text.lower():
        notes.append("basecamp-app/qml/Lp0002PrivateMultisig.qml may not reference lp0002")
else:
    notes.append("basecamp-app/qml/Lp0002PrivateMultisig.qml not found; native QML UI is expected for evaluator review")

# Executable gates.
if not args.skip_exec:
    run(["cargo", "test", "--workspace"])
    run(["bash", "demo.sh"])
else:
    notes.append("--skip-exec supplied; cargo test and demo.sh were skipped")

if errors:
    for err in errors:
        print(f"FAIL: {err}")
    sys.exit(1)

print("PASS: LP-0002 local implementation readiness validator")
print(f"  - {len(required_files)} required files")
print(f"  - {len(required_crates)} workspace crates")
print(f"  - IDL with {len(idl.get('instructions', [])) if idl else 0} instructions and verified discriminators")
print(f"  - Protocol doc with {len(required_protocol)} required sections")
print(f"  - Compliance matrix with {len(required_sections)} sections")
print("  - Basecamp native QML structure validated")
print("  - Honesty gates present")
print("  - NOTE: final publication gates are checked separately by scripts/final-publication-check.py")
if args.skip_exec:
    print("  - Execution gates skipped by --skip-exec")
else:
    print("  - cargo test --workspace passed")
    print("  - demo.sh passed")
