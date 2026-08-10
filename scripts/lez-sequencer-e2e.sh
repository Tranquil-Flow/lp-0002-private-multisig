#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEZ_SOURCE_DIR="${LEZ_SOURCE_DIR:?set LEZ_SOURCE_DIR to a logos-execution-zone v0.2.4 checkout}"
PROJECT_TARGET_DIR="${PROJECT_TARGET_DIR:-${CARGO_TARGET_DIR:-$ROOT/target}}"
LEZ_TARGET_DIR="${LEZ_TARGET_DIR:-$LEZ_SOURCE_DIR/target}"
export RISC0_DEV_MODE=0

if [[ "$(git -C "$LEZ_SOURCE_DIR" rev-parse HEAD)" != "47eba256479f6f785acbd138834340703cd03401" ]]; then
  echo "ERROR: LEZ_SOURCE_DIR must be pinned to v0.2.4 commit 47eba256479f6f785acbd138834340703cd03401" >&2
  exit 1
fi
if [[ -n "$(git -C "$LEZ_SOURCE_DIR" status --porcelain --untracked-files=no)" ]]; then
  echo "ERROR: LEZ_SOURCE_DIR has tracked modifications; exact v0.2.4 source is required" >&2
  exit 1
fi

PORT="${LEZ_E2E_PORT:-$(python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}"
METRICS_PORT="${LEZ_E2E_METRICS_PORT:-$(python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}"
while [[ "$METRICS_PORT" == "$PORT" ]]; do
  METRICS_PORT="$(python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)"
done
RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/lp0002-lez-e2e.XXXXXX")"
PROOF_DIR="$RUN_DIR/proof"
SEQUENCER_HOME="$RUN_DIR/sequencer-home"
SEQUENCER_LOG="$RUN_DIR/sequencer.log"
SEQUENCER_PID=""

cleanup() {
  status=$?
  trap - EXIT
  if [[ -n "$SEQUENCER_PID" ]] && kill -0 "$SEQUENCER_PID" 2>/dev/null; then
    kill "$SEQUENCER_PID" 2>/dev/null || true
    wait "$SEQUENCER_PID" 2>/dev/null || true
  fi
  if [[ "$status" -ne 0 && -f "$SEQUENCER_LOG" ]]; then
    echo "== Sequencer log (last 200 lines) ==" >&2
    python3 - "$SEQUENCER_LOG" <<'PY' >&2
from collections import deque
from pathlib import Path
import sys
for line in deque(Path(sys.argv[1]).open(errors="replace"), maxlen=200):
    print(line, end="")
PY
  fi
  if [[ "${KEEP_LEZ_E2E_ARTIFACTS:-0}" == "1" ]]; then
    echo "E2E artifacts retained at $RUN_DIR"
  else
    rm -rf "$RUN_DIR"
  fi
  exit "$status"
}
trap cleanup EXIT

mkdir -p "$PROOF_DIR" "$SEQUENCER_HOME"

printf '== Build LP-0002 RISC0 guests and E2E clients ==\n'
(
  cd "$ROOT"
  CARGO_TARGET_DIR="$PROJECT_TARGET_DIR" cargo build --locked \
    -p lp0002-private-multisig-host \
    --bin lp0002-prove-fixture \
    --bin lp0002-submit-localnet
  CARGO_TARGET_DIR="$PROJECT_TARGET_DIR" cargo test --locked \
    -p lp0002-private-multisig-host \
    --test host_boundary \
    wrapper_threshold_image_id_matches_canonical_threshold_guest \
    -- --exact
)

printf '== Build LEZ v0.2.4 standalone sequencer ==\n'
(
  cd "$LEZ_SOURCE_DIR"
  CARGO_TARGET_DIR="$LEZ_TARGET_DIR" cargo build --locked --features standalone -p sequencer_service
)

PROVER="$PROJECT_TARGET_DIR/debug/lp0002-prove-fixture"
SUBMIT="$PROJECT_TARGET_DIR/debug/lp0002-submit-localnet"
PROGRAM="$PROJECT_TARGET_DIR/riscv-guest/lp0002-private-multisig-methods/lp0002-private-multisig-guest/riscv32im-risc0-zkvm-elf/release/verify_and_execute_bytes.bin"
SEQUENCER="$LEZ_TARGET_DIR/debug/sequencer_service"
SEQUENCER_CONFIG="$LEZ_SOURCE_DIR/lez/sequencer/service/configs/debug/sequencer_config.json"

for executable in "$PROVER" "$SUBMIT" "$SEQUENCER"; do
  [[ -x "$executable" ]] || { echo "ERROR: missing executable $executable" >&2; exit 1; }
done
[[ -s "$PROGRAM" ]] || { echo "ERROR: missing generated RISC0 program $PROGRAM" >&2; exit 1; }

printf '== Generate fresh RISC0_DEV_MODE=0 proof ==\n'
"$PROVER" "$PROOF_DIR"

printf '== Start isolated LEZ v0.2.4 sequencer ==\n'
RUST_LOG="${RUST_LOG:-info}" "$SEQUENCER" "$SEQUENCER_CONFIG" \
  --port "$PORT" \
  --listen-address 127.0.0.1 \
  --home "$SEQUENCER_HOME" \
  --metrics-address "127.0.0.1:$METRICS_PORT" \
  >"$SEQUENCER_LOG" 2>&1 &
SEQUENCER_PID=$!

RPC_URL="http://127.0.0.1:$PORT"
healthy=0
for _ in $(seq 1 120); do
  if ! kill -0 "$SEQUENCER_PID" 2>/dev/null; then
    echo "ERROR: sequencer exited before becoming healthy" >&2
    exit 1
  fi
  if response="$(curl --fail --silent --show-error --max-time 2 \
      -H 'content-type: application/json' \
      --data '{"jsonrpc":"2.0","id":1,"method":"checkHealth","params":[]}' \
      "$RPC_URL" 2>/dev/null)" && [[ "$response" == *'"result":null'* ]]; then
    healthy=1
    break
  fi
  sleep 1
done
[[ "$healthy" == "1" ]] || { echo "ERROR: sequencer health timeout" >&2; exit 1; }

printf '== Reject mismatched caller-supplied program id before deployment ==\n'
MISMATCH_DIR="$RUN_DIR/program-id-mismatch"
if "$SUBMIT" --submit --sequencer-url "$RPC_URL" \
  --program-id 0000000000000000000000000000000000000000000000000000000000000000 \
  --deploy-program "$PROGRAM" "$MISMATCH_DIR" >"$RUN_DIR/program-id-mismatch.log" 2>&1; then
  echo "ERROR: mismatched caller-supplied program id was accepted" >&2
  exit 1
fi
if ! grep -q "refusing deployment: supplied program id" "$RUN_DIR/program-id-mismatch.log"; then
  echo "ERROR: mismatched program id did not fail at the expected pre-submit guard" >&2
  exit 1
fi
if [[ -e "$MISMATCH_DIR/nssa-deploy-evidence.json" ]]; then
  echo "ERROR: mismatched program id produced deployment evidence" >&2
  exit 1
fi

printf '== Deploy generated wrapper program ==\n'
"$SUBMIT" --submit --sequencer-url "$RPC_URL" --deploy-program "$PROGRAM" "$PROOF_DIR"
python3 - "$PROOF_DIR/nssa-deploy-evidence.json" <<'PY'
import json
from pathlib import Path
import sys
payload = json.loads(Path(sys.argv[1]).read_text())
assert payload.get("program_id_matches_execute") is True, payload
assert payload.get("confirmed") is True, payload
assert isinstance(payload.get("included_block_id"), int), payload
print("deployment confirmed", payload["tx_hash"], "block", payload["included_block_id"])
PY

printf '== Execute LP-0002 proof-backed transaction ==\n'
"$SUBMIT" --submit --sequencer-url "$RPC_URL" "$PROOF_DIR"
TX_HASH="$(python3 - "$PROOF_DIR/nssa-submit-evidence.json" <<'PY'
import json
from pathlib import Path
import sys
payload = json.loads(Path(sys.argv[1]).read_text())
assert payload.get("risc0_dev_mode") == "0", payload
assert payload.get("confirmed") is True, payload
assert isinstance(payload.get("included_block_id"), int), payload
print(payload["tx_hash"])
PY
)"

printf '== Query canonical transaction/block inclusion ==\n'
"$SUBMIT" --sequencer-url "$RPC_URL" --tx-hash "$TX_HASH" --from-block 1 "$PROOF_DIR"
python3 - "$PROOF_DIR/nssa-submit-evidence.json" <<'PY'
import json
from pathlib import Path
import sys
payload = json.loads(Path(sys.argv[1]).read_text())
assert payload.get("status") == "confirmed", payload
assert payload.get("confirmed") is True, payload
assert isinstance(payload.get("included_block_id"), int), payload
assert isinstance(payload.get("included_block_hash"), str) and payload["included_block_hash"], payload
assert isinstance(payload.get("included_tx_index"), int), payload
print(
    "PASS: LEZ v0.2.4 RISC0_DEV_MODE=0 deploy/execute/inclusion",
    payload["tx_hash"],
    "block",
    payload["included_block_id"],
    "index",
    payload["included_tx_index"],
)
PY
