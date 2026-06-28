#!/usr/bin/env python3
"""Fail-closed read-only verifier for LP-0002 current public-testnet evidence.

LP-0002 currently has fresh reset-era deploy evidence, but public execute
inclusion remains blocked. This verifier checks exactly that claim surface:
current deploy must be present, execute-attempt hashes must remain absent, and
local v0.2.0 validation evidence must be retained. It must be updated before
claiming current live execute inclusion.
"""
from __future__ import annotations

import json
import sys
import urllib.request
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
RPC_URL = "https://testnet.lez.logos.co/"
EXPECTED_DEPLOY_TX = "c7157a473cb512bf7e1803d4377d9f65e9406a7ff98efeda48b65c0d4915a13b"
EXPECTED_EXECUTE_ATTEMPTS = [
    "352eb699507aea4d4ca6963a50bef1473a2b944dfd7713116cbf82eabfeec3bf",
    "fc4165ac2437bd6533444c5e010b2d248aed678daadfad277af1dd0f1fef6ca8",
]


def rpc_raw(method: str, params: list[Any]) -> dict[str, Any]:
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
    req = urllib.request.Request(RPC_URL, data=body, headers={"content-type": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read())


def tx_lookup(tx_hash: str) -> Any:
    errors: list[dict[str, Any]] = []
    # Current testnet accepts camelCase. Keep snake_case fallback only for older
    # evaluator nodes; do not treat method absence as transaction absence.
    for method in ("getTransaction", "get_transaction_by_hash"):
        data = rpc_raw(method, [tx_hash])
        if "error" in data:
            errors.append({method: data["error"]})
            continue
        result = data.get("result")
        if isinstance(result, dict) and "transaction" in result:
            return result.get("transaction")
        return result
    raise SystemExit(f"transaction lookup RPC methods unavailable: {errors}")


def main() -> None:
    evidence_path = ROOT / "submission" / "TESTNET_EVIDENCE.json"
    evidence = json.loads(evidence_path.read_text())
    refresh = evidence.get("current_testnet_refresh_2026_06_28", {})
    if refresh.get("local_v020_validation", {}).get("status") != "passed":
        raise SystemExit("missing passed local v0.2.0 validation evidence")

    deploy = tx_lookup(EXPECTED_DEPLOY_TX)
    if deploy is None or (isinstance(deploy, str) and len(deploy) < 20):
        raise SystemExit(f"current deploy transaction not found: {EXPECTED_DEPLOY_TX}")
    print(f"current_deploy_tx_found={EXPECTED_DEPLOY_TX}")

    absent: list[str] = []
    present: list[str] = []
    for tx_hash in EXPECTED_EXECUTE_ATTEMPTS:
        result = tx_lookup(tx_hash)
        if result is None or (isinstance(result, str) and len(result) < 20):
            absent.append(tx_hash)
        else:
            present.append(tx_hash)
    if present:
        raise SystemExit(
            "execute attempt unexpectedly became included; update evidence before claiming: "
            + ",".join(present)
        )
    print("current_execute_attempts_not_included=" + ",".join(absent))
    print("LP-0002 current claim verified: deploy present; execute inclusion remains blocked and not overclaimed")


if __name__ == "__main__":
    main()
