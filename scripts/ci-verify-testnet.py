#!/usr/bin/env python3
"""Fail-closed read-only verifier for LP-0002 public-testnet evidence.

The recorded deploy/execute hashes are historical public LEZ testnet evidence.
This script re-queries the current public endpoint and exits non-zero if a
Logos-side reset makes those hashes unavailable, preventing accidental
current-live overclaims.
"""
from __future__ import annotations

import json
import urllib.request
from typing import Any

RPC_URL = "https://testnet.lez.logos.co/"
EXPECTED_DEPLOY_TX = "82516880f60c2076d78b28ad7b147ac0b05ed247b7bc33a27ac8f68b1d809c56"
EXPECTED_EXECUTE_TX = "cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697"


def rpc_raw(method: str, params: list[Any]) -> dict[str, Any]:
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
    req = urllib.request.Request(RPC_URL, data=body, headers={"content-type": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read())


def tx_lookup(tx_hash: str) -> Any:
    errors: list[dict[str, Any]] = []
    for method in ("get_transaction_by_hash", "getTransaction"):
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
    for label, tx_hash in (("deploy", EXPECTED_DEPLOY_TX), ("execute", EXPECTED_EXECUTE_TX)):
        result = tx_lookup(tx_hash)
        if result is None or (isinstance(result, str) and len(result) < 20):
            raise SystemExit(
                f"{label} transaction not found on current public testnet "
                f"(historical/reset evidence only): {tx_hash}"
            )
        print(f"{label}_tx_found={tx_hash}")
    print("LP-0002 public-testnet evidence is current-live on this endpoint")


if __name__ == "__main__":
    main()
