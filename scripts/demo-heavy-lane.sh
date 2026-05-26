#!/usr/bin/env bash
set -euo pipefail

# LP-0002 heavy-lane evidence demo for QuickTime recording.
# Usage:
#   bash scripts/demo-heavy-lane.sh
#   SECTION_PAUSE=0 COMMAND_PAUSE=0 SCENE_PAUSE=0 bash scripts/demo-heavy-lane.sh  # dry run
#   bash scripts/demo-heavy-lane.sh --live-submit  # deploy wrapper and submit a fresh localnet tx

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="/opt/homebrew/bin:$HOME/bin:$HOME/.cargo/bin:/Applications/Docker.app/Contents/Resources/bin:$PATH"
export TERM="${TERM:-xterm-256color}"
export RISC0_DEV_MODE=0

ARTIFACT_DIR="${ARTIFACT_DIR:-target/lp0002-risc0-fixture-new}"
LIVE_SUBMIT=0
if [[ "${1:-}" == "--live-submit" ]]; then
  LIVE_SUBMIT=1
elif [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
Usage: bash scripts/demo-heavy-lane.sh [--live-submit]

Default mode is recording-safe: it verifies existing RISC0_DEV_MODE=0 artifacts,
builds the compact wrapper payload, and prints confirmed localnet evidence.

--live-submit additionally deploys verify_and_execute_bytes to localnet and sends
a fresh NSSA transaction. This requires lgs localnet, Docker/RISC0 tooling, and
NSSA_WALLET_HOME_DIR=.scaffold/wallet.
USAGE
  exit 0
elif [[ -n "${1:-}" ]]; then
  echo "ERROR: unknown argument: $1" >&2
  exit 2
fi

SECTION_PAUSE="${SECTION_PAUSE:-3}"
COMMAND_PAUSE="${COMMAND_PAUSE:-1}"
SCENE_PAUSE="${SCENE_PAUSE:-5}"

BOLD='\033[1m'
DIM='\033[2m'
GREEN='\033[32m'
CYAN='\033[36m'
YELLOW='\033[33m'
RESET='\033[0m'

pause() { sleep "${1:-$COMMAND_PAUSE}"; }
section() {
  echo
  echo -e "${BOLD}${CYAN}════════════════════════════════════════════════════════════${RESET}"
  echo -e "${BOLD}${CYAN}  $1${RESET}"
  echo -e "${BOLD}${CYAN}════════════════════════════════════════════════════════════${RESET}"
  echo
  pause "$SECTION_PAUSE"
}
step() { echo -e "${BOLD}▸ $1${RESET}"; }
result() { echo -e "  ${GREEN}✓ $1${RESET}"; }
note() { echo -e "  ${YELLOW}Note:${RESET} $1"; }
run_cmd() {
  echo -e "${DIM}$ $*${RESET}"
  "$@"
  echo
  pause "$COMMAND_PAUSE"
}

require_file() {
  if [[ ! -s "$1" ]]; then
    echo "ERROR: required artifact missing: $1" >&2
    echo "Run: RISC0_DEV_MODE=0 cargo run -p lp0002-private-multisig-host --bin lp0002-prove-fixture -- $ARTIFACT_DIR" >&2
    exit 1
  fi
}

section "LP-0002 heavy lane: RISC0 proof + LEZ wrapper evidence"
cat <<EOF
This demo shows the heavy-lane evidence for LP-0002:
  • real RISC0_DEV_MODE=0 proof artifacts are verified host-side
  • the executable verify_and_execute_bytes wrapper surface is built
  • the NSSA/LEZ compact wrapper payload is reproducible
  • confirmed localnet inclusion evidence is printed

Honesty note: current LEZ/RISC0 public-program sessions cannot carry the raw
270 KiB receipt inside the public wrapper input. The wrapper therefore carries
a receipt/journal commitment, while the full receipt remains retained and
verified as file artifact evidence.
EOF
pause "$SCENE_PAUSE"

section "1. Verify required proof artifacts"
require_file "$ARTIFACT_DIR/receipt.borsh"
require_file "$ARTIFACT_DIR/journal.borsh"
require_file "$ARTIFACT_DIR/manifest.txt"
run_cmd cargo run -p lp0002-private-multisig-host --bin lp0002-verify-artifacts -- "$ARTIFACT_DIR"
result "RISC0 receipt artifacts verify against the expected threshold proof image"
pause "$SCENE_PAUSE"

section "2. Execute the LEZ-shaped host bridge"
run_cmd cargo run -p lp0002-private-multisig-host --bin lp0002-lez-execute-artifacts -- "$ARTIFACT_DIR"
python3 - <<'PY'
import json
from pathlib import Path
p = Path('target/lp0002-risc0-fixture-new/lez-execution.json')
if p.exists():
    data = json.loads(p.read_text())
    for k in ['status', 'proposal_state_executed', 'proposal_state_nullifier_count', 'proof_id']:
        if k in data:
            print(f'{k}: {data[k]}')
PY
result "The verified journal drives the replay-protected LEZ-shaped execution gate"
pause "$SCENE_PAUSE"

section "3. Build compact NSSA/SPEL payload evidence"
run_cmd cargo run -p lp0002-private-multisig-host --bin lp0002-spel-adapter-evidence -- "$ARTIFACT_DIR"
run_cmd cargo run -p lp0002-private-multisig-host --bin lp0002-submit-localnet -- "$ARTIFACT_DIR"
python3 - <<'PY'
import json
from pathlib import Path
for rel in ['spel-adapter-evidence.json', 'nssa-submit-dry-run.json']:
    p = Path('target/lp0002-risc0-fixture-new') / rel
    data = json.loads(p.read_text())
    print(f'{rel}:')
    for k in ['program_id', 'threshold_proof_image_id', 'instruction_data_len', 'instruction_data_sha256', 'receipt_bytes_len', 'receipt_bytes_sha256', 'proof_journal_bytes_sha256', 'receipt_journal_commitment', 'receipt_transport']:
        print(f'  {k}: {data.get(k)}')
PY
result "Payload hash and receipt/journal commitment is deterministic and reproducible"
pause "$SCENE_PAUSE"

if [[ "$LIVE_SUBMIT" == "1" ]]; then
  section "4. Live localnet submit"
  if ! command -v lgs >/dev/null 2>&1; then
    echo "ERROR: lgs not found; cannot live-submit" >&2
    exit 1
  fi
  run_cmd lgs localnet status
  WRAPPER_BIN="target/riscv-guest/lp0002-private-multisig-methods/lp0002-private-multisig-guest/riscv32im-risc0-zkvm-elf/release/verify_and_execute_bytes.bin"
  require_file "$WRAPPER_BIN"
  run_cmd lgs deploy verify_and_execute_bytes --program-path "$WRAPPER_BIN" --json
  NSSA_WALLET_HOME_DIR=.scaffold/wallet run_cmd target/debug/lp0002-submit-localnet --submit --no-poll "$ARTIFACT_DIR"
  TX_HASH="$(python3 - <<'PY'
import json
print(json.load(open('target/lp0002-risc0-fixture-new/nssa-submit-evidence.json'))['tx_hash'])
PY
)"
  sleep 20
  NSSA_WALLET_HOME_DIR=.scaffold/wallet run_cmd target/debug/lp0002-submit-localnet --tx-hash "$TX_HASH" "$ARTIFACT_DIR"
else
  section "4. Confirmed localnet inclusion evidence"
  note "Default mode prints the recorded confirmed localnet evidence, avoiding duplicate txs during recording."
fi

python3 - <<'PY'
import json
from pathlib import Path
p = Path('target/lp0002-risc0-fixture-new/nssa-submit-evidence.json')
if not p.exists():
    raise SystemExit('missing target/lp0002-risc0-fixture-new/nssa-submit-evidence.json')
data = json.loads(p.read_text())
for k in ['status', 'confirmed', 'program_id', 'threshold_proof_image_id', 'tx_hash', 'included_block_id', 'included_tx_index', 'instruction_data_len', 'instruction_data_sha256', 'receipt_transport']:
    print(f'{k}: {data.get(k)}')
if data.get('status') != 'confirmed':
    raise SystemExit('localnet evidence is not confirmed')
PY
result "Wrapper transaction inclusion evidence is confirmed"
pause "$SCENE_PAUSE"

section "5. Final readiness gate"
run_cmd python3 scripts/validate-submission-readiness.py
result "LP-0002 heavy-lane evidence is documented and the base submission still validates"

echo
result "Heavy-lane demo complete"
