#!/usr/bin/env python3
"""Generate reproducible LP-0002 LEZ payload/proof cost evidence.

The current public/local LEZ tooling used by this repo does not expose a stable
per-transaction compute-unit counter through lgs/NSSA query output. This script
therefore records the measurable submission-cost fields that are available now
(payload bytes, account metas, receipt size, journal size, hashes, inclusion
status) and marks CU as unavailable rather than inventing a number.
"""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "target" / "lp0002-risc0-fixture-new"
OUT = ROOT / "submission" / "LEZ_COST_BENCHMARKS.json"

def load(name: str) -> dict:
    path = FIXTURE / name
    if not path.exists():
        raise SystemExit(f"missing fixture evidence: {path}")
    return json.loads(path.read_text())

spel = load("spel-adapter-evidence.json")
dry = load("nssa-submit-dry-run.json")
submit = load("nssa-submit-evidence.json")
lez = load("lez-execution.json")

bench = {
    "schema": "lp0002.lez-cost-benchmarks.v1",
    "network": submit.get("network", "localnet"),
    "status": submit.get("status"),
    "confirmed": submit.get("confirmed"),
    "program_id": submit.get("program_id"),
    "tx_hash": submit.get("tx_hash"),
    "included_block_id": submit.get("included_block_id"),
    "included_tx_index": submit.get("included_tx_index"),
    "instruction": spel.get("instruction"),
    "accounts": len(dry.get("account_ids", [])),
    "instruction_data_len_bytes": dry.get("instruction_data_len"),
    "instruction_data_sha256": dry.get("instruction_data_sha256"),
    "receipt_bytes_len": spel.get("receipt_bytes_len"),
    "receipt_bytes_sha256": spel.get("receipt_bytes_sha256"),
    "proof_journal_bytes_sha256": spel.get("proof_journal_bytes_sha256"),
    "receipt_journal_commitment": spel.get("receipt_journal_commitment"),
    "threshold_proof_image_id": spel.get("threshold_proof_image_id"),
    "approval_count": lez.get("approval_count") or lez.get("journal", {}).get("approval_count"),
    "nullifier_count": spel.get("public_journal", {}).get("nullifier_count") or lez.get("proposal_state_nullifier_count"),
    "cu_metering": {
        "available": False,
        "reason": "The lgs/NSSA localnet query output available in this environment did not expose a stable per-transaction compute-unit counter. Payload/account/receipt sizes are recorded as reproducible cost evidence; replace this field with runtime CU once public LEZ devnet/testnet exposes it."
    }
}
OUT.write_text(json.dumps(bench, indent=2) + "\n")
print(f"wrote {OUT}")
print(json.dumps(bench, indent=2))
