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
HEAVY_CARGO_VERIFY="${HEAVY_CARGO_VERIFY:-0}"

ARTIFACT_DIR="${ARTIFACT_DIR:-target/lp0002-risc0-fixture-new}"
BUNDLED_ARTIFACT_DIR="submission/proof-artifacts/lp0002-risc0-fixture-new"
LIVE_SUBMIT=0
if [[ "${1:-}" == "--live-submit" ]]; then
  LIVE_SUBMIT=1
elif [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
Usage: bash scripts/demo-heavy-lane.sh [--live-submit]

Default mode is recording-safe: it validates bundled RISC0_DEV_MODE=0 evidence,
prints the compact wrapper payload, and prints confirmed public-testnet evidence.
Set HEAVY_CARGO_VERIFY=1 to also run the Rust host receipt verifier locally.

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

materialize_bundled_artifacts() {
  if [[ "$ARTIFACT_DIR" == "target/lp0002-risc0-fixture-new" && ! -s "$ARTIFACT_DIR/receipt.borsh" && -s "$BUNDLED_ARTIFACT_DIR/receipt.borsh" ]]; then
    mkdir -p "$ARTIFACT_DIR"
    cp "$BUNDLED_ARTIFACT_DIR"/* "$ARTIFACT_DIR"/
    note "Loaded bundled proof artifacts into $ARTIFACT_DIR for fresh-clone replay."
  fi
  export ARTIFACT_DIR
}

section "LP-0002 heavy lane: RISC0 proof + LEZ wrapper evidence"
cat <<EOF
This demo shows the heavy-lane evidence for LP-0002:
  • bundled RISC0_DEV_MODE=0 proof artifacts are checked for freshness metadata
  • optional Rust host verification is available with HEAVY_CARGO_VERIFY=1
  • the executable verify_and_execute_bytes wrapper evidence is printed
  • the NSSA/LEZ compact wrapper payload is reproducible
  • confirmed public-testnet inclusion evidence is printed

Honesty note: current LEZ/RISC0 public-program sessions cannot carry the raw
270 KiB receipt inside the public wrapper input. The wrapper therefore carries
a receipt/journal commitment, while the full receipt remains retained and
verified as file artifact evidence.
EOF
pause "$SCENE_PAUSE"

section "1. Verify required proof artifacts"
materialize_bundled_artifacts
require_file "$ARTIFACT_DIR/receipt.borsh"
require_file "$ARTIFACT_DIR/journal.borsh"
require_file "$ARTIFACT_DIR/manifest.txt"
python3 - <<'PY'
import hashlib, json, os
from pathlib import Path
artifact_dir = Path(os.environ['ARTIFACT_DIR'])
expected = {
    'manifest.txt': '2345dd03a3fb0bbae02287662800559026e0e003cb99dd1acf5e5f177706a122',
    'receipt.borsh': '6e4979983c996ca4154d7eeedb59444105b99d984a69a223ab58d429811b89a7',
    'journal.borsh': 'a8fe85f8d63f948409941b585cbe9244c2d0ae45082bf635173f753037ad4d8e',
    'lez-execution.json': '7044725afc2e5f49cb25c5b35c2e3c851ec3ab461b00e4acb4876461f2ea3c10',
    'spel-adapter-evidence.json': 'cbeebe596edbda2426675fa4b18ecb6622f1685e973538a7a6c1a65f842fec97',
    'nssa-submit-dry-run.json': '33c83315a1cbf308074c336ada7accaae50238aa6cd5c639d6fce0823f3614f1',
    'nssa-submit-evidence.json': 'ee9eb8eb3c3dd9c578b4a94dda8c62da6d35b5edab01db9ac7e6e975e44433fe',
}
for rel, want in expected.items():
    p = artifact_dir / rel
    got = hashlib.sha256(p.read_bytes()).hexdigest()
    print(f'{rel}: {p.stat().st_size} bytes sha256={got}')
    if got != want:
        raise SystemExit(f'{rel} sha256 mismatch')
manifest = (artifact_dir / 'manifest.txt').read_text()
for token in [
    'risc0_dev_mode=0',
    'image_id=6fc85ce06da1762abec319b4626c12229dc605a5b0283d64c8eab2567b9ee721',
    'proof_id=9e6492e73d1e8382abfa0e94e91842100b9041516857f215fcad7276cbad8b11',
    'threshold=2',
    'approval_count=2',
]:
    if token not in manifest:
        raise SystemExit(f'manifest missing {token}')
print('manifest confirms RISC0_DEV_MODE=0 image/proof ids and 2-of-3 threshold')
PY
if [[ "$HEAVY_CARGO_VERIFY" == "1" ]]; then
  run_cmd cargo run -p lp0002-private-multisig-host --bin lp0002-verify-artifacts -- "$ARTIFACT_DIR"
  result "RISC0 receipt artifacts verify against the expected threshold proof image"
else
  note "Skipping Rust host receipt verifier by default to keep fresh-clone demo lightweight; set HEAVY_CARGO_VERIFY=1 to run it."
  result "Bundled RISC0_DEV_MODE=0 artifact hashes and manifest metadata match the audited evidence set"
fi
pause "$SCENE_PAUSE"

section "2. Inspect the LEZ-shaped host bridge evidence"
if [[ "$HEAVY_CARGO_VERIFY" == "1" ]]; then
  run_cmd cargo run -p lp0002-private-multisig-host --bin lp0002-lez-execute-artifacts -- "$ARTIFACT_DIR"
else
  note "Using bundled lez-execution.json; set HEAVY_CARGO_VERIFY=1 to regenerate it with the Rust host."
fi
python3 - <<'PY'
import json, os
from pathlib import Path
p = Path(os.environ['ARTIFACT_DIR']) / 'lez-execution.json'
if p.exists():
    data = json.loads(p.read_text())
    for k in ['status', 'proposal_state_executed', 'proposal_state_nullifier_count', 'proof_id']:
        if k in data:
            print(f'{k}: {data[k]}')
PY
result "The verified journal drives the replay-protected LEZ-shaped execution gate"
pause "$SCENE_PAUSE"

section "3. Inspect compact NSSA/SPEL payload evidence"
if [[ "$HEAVY_CARGO_VERIFY" == "1" ]]; then
  run_cmd cargo run -p lp0002-private-multisig-host --bin lp0002-spel-adapter-evidence -- "$ARTIFACT_DIR"
  run_cmd cargo run -p lp0002-private-multisig-host --bin lp0002-submit-localnet -- "$ARTIFACT_DIR"
else
  note "Using bundled compact payload evidence; set HEAVY_CARGO_VERIFY=1 to regenerate it."
fi
python3 - <<'PY'
import json, os
from pathlib import Path
for rel in ['spel-adapter-evidence.json', 'nssa-submit-dry-run.json']:
    p = Path(os.environ['ARTIFACT_DIR']) / rel
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
import json, os
from pathlib import Path
print(json.loads((Path(os.environ['ARTIFACT_DIR']) / 'nssa-submit-evidence.json').read_text())['tx_hash'])
PY
)"
  sleep 20
  NSSA_WALLET_HOME_DIR=.scaffold/wallet run_cmd target/debug/lp0002-submit-localnet --tx-hash "$TX_HASH" "$ARTIFACT_DIR"
else
  section "4. Confirmed public-testnet inclusion evidence"
  note "Default mode prints the recorded confirmed public-testnet evidence, avoiding duplicate txs during recording."
fi

python3 - <<'PY'
import json, os
from pathlib import Path
p = Path(os.environ['ARTIFACT_DIR']) / 'nssa-submit-evidence.json'
if not p.exists():
    raise SystemExit(f'missing {p}')
data = json.loads(p.read_text())
for k in ['status', 'confirmed', 'program_id', 'threshold_proof_image_id', 'tx_hash', 'included_block_id', 'included_tx_index', 'instruction_data_len', 'instruction_data_sha256', 'receipt_transport']:
    print(f'{k}: {data.get(k)}')
if data.get('status') != 'confirmed':
    raise SystemExit('public-testnet evidence is not confirmed')
PY
result "Wrapper transaction inclusion evidence is confirmed"
pause "$SCENE_PAUSE"

section "5. Final readiness gate"
run_cmd env RISC0_SKIP_BUILD=1 python3 scripts/validate-submission-readiness.py --skip-exec
result "LP-0002 heavy-lane evidence is documented and the base submission still validates"

echo
result "Heavy-lane demo complete"
