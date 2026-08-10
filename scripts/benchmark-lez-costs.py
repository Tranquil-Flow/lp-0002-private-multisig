#!/usr/bin/env python3
"""Generate current LP-0002 LEZ payload/proof cost evidence.

LEZ v0.2.4 does not expose a stable per-transaction compute-unit counter.
Generate the measurable cost surface from the committed, reviewer-visible
v0.2.4 public evidence instead of historical NSSA fixtures.
"""
from __future__ import annotations

import argparse
import difflib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PUBLIC = ROOT / "submission" / "LEZ_V024_PUBLIC_E2E_EVIDENCE.json"
OUT = ROOT / "submission" / "LEZ_COST_BENCHMARKS.json"


def load_object(path: Path) -> dict:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise SystemExit(f"expected JSON object: {path}")
    return value


def require(mapping: dict, key: str, context: str):
    value = mapping.get(key)
    if value is None:
        raise SystemExit(f"missing {context}.{key} in {PUBLIC}")
    return value


def build_benchmark(public: dict) -> dict:
    if public.get("network_scope") != "public_testnet":
        raise SystemExit(f"{PUBLIC} is not public-testnet evidence")

    lez = require(public, "lez", "root")
    proof = require(public, "proof", "root")
    journal = require(public, "public_journal", "root")
    program = require(public, "program", "root")
    deploy = require(public, "deployment", "root")
    execute = require(public, "execution", "root")
    observation = require(public, "explorer_index_observation", "root")

    if deploy.get("confirmed_by_sequencer_rpc") is not True:
        raise SystemExit("deployment is not sequencer-RPC confirmed")
    if execute.get("confirmed_by_sequencer_rpc") is not True:
        raise SystemExit("execution is not sequencer-RPC confirmed")
    if deploy.get("explorer_status") != "visible":
        raise SystemExit("deployment explorer page is not visible")
    if execute.get("explorer_status") != "visible":
        raise SystemExit("execution explorer page is not visible")
    if deploy.get("program_id_matches_execute") is not True:
        raise SystemExit("deployed program id does not match execution")

    return {
        "schema": "lp0002.lez-cost-benchmarks.v2",
        "network": "testnet",
        "lez_release": require(lez, "release", "lez"),
        "lez_commit": require(lez, "commit", "lez"),
        "status": "confirmed",
        "confirmed": True,
        "risc0_dev_mode": require(lez, "risc0_dev_mode", "lez"),
        "program_id": require(program, "program_id", "program"),
        "program_binary_len_bytes": require(program, "program_binary_bytes", "program"),
        "program_binary_sha256": require(program, "program_binary_sha256", "program"),
        "tx_hash": require(execute, "tx_hash", "execution"),
        "included_block_id": require(execute, "included_block_id", "execution"),
        "included_block_hash": require(execute, "included_block_hash", "execution"),
        "included_tx_index": require(execute, "included_tx_index", "execution"),
        "instruction": require(execute, "instruction", "execution"),
        "accounts": require(execute, "account_count", "execution"),
        "instruction_data_len_bytes": require(execute, "instruction_data_bytes", "execution"),
        "instruction_data_sha256": require(execute, "instruction_data_sha256", "execution"),
        "receipt_bytes_len": require(proof, "receipt_bytes", "proof"),
        "receipt_bytes_sha256": require(proof, "receipt_sha256", "proof"),
        "proof_journal_bytes_sha256": require(proof, "journal_sha256", "proof"),
        "receipt_journal_commitment": require(
            execute, "receipt_journal_commitment", "execution"
        ),
        "threshold_proof_image_id": require(
            proof, "threshold_proof_image_id", "proof"
        ),
        "approval_count": require(journal, "approval_count", "public_journal"),
        "nullifier_count": require(journal, "nullifier_count", "public_journal"),
        "cu_metering": {
            "available": False,
            "reason": (
                "LEZ v0.2.4 JSON-RPC exposes canonical transaction/block inclusion "
                "but no stable per-transaction compute-unit counter. Payload, account, "
                "ProgramBinary, receipt, and journal sizes/hashes are recorded instead."
            ),
        },
        "current_testnet_refresh_2026_08_10": {
            "deploy_tx": require(deploy, "tx_hash", "deployment"),
            "deploy_block_id": require(deploy, "included_block_id", "deployment"),
            "deploy_confirmed_by_sequencer_rpc": True,
            "execute_tx": require(execute, "tx_hash", "execution"),
            "execute_block_id": require(execute, "included_block_id", "execution"),
            "execute_confirmed_by_sequencer_rpc": True,
            "explorer_status": "visible",
            "explorer_latest_observed_block_id": require(
                observation, "explorer_latest_block_id", "explorer_index_observation"
            ),
            "evidence": "submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json",
        },
    }


def render(value: dict) -> str:
    return json.dumps(value, indent=2) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail if committed cost evidence differs from current public evidence",
    )
    args = parser.parse_args()

    generated = build_benchmark(load_object(PUBLIC))
    expected = render(generated)
    if args.check:
        actual = OUT.read_text() if OUT.exists() else ""
        if actual != expected:
            print(
                "".join(
                    difflib.unified_diff(
                        actual.splitlines(keepends=True),
                        expected.splitlines(keepends=True),
                        fromfile=str(OUT),
                        tofile="generated-current-evidence",
                    )
                ),
                end="",
            )
            return 1
        print("PASS: LEZ cost evidence matches current public evidence")
        return 0

    OUT.write_text(expected)
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
