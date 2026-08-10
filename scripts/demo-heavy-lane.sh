#!/usr/bin/env bash
set -euo pipefail

# LP-0002 heavy-lane evidence demo for QuickTime recording.
# Usage:
#   bash scripts/demo-heavy-lane.sh
#   SECTION_PAUSE=0 COMMAND_PAUSE=0 SCENE_PAUSE=0 bash scripts/demo-heavy-lane.sh  # dry run
#   bash scripts/demo-heavy-lane.sh --current-e2e  # run the current standalone LEZ v0.2.4 E2E

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="/opt/homebrew/bin:$HOME/bin:$HOME/.cargo/bin:/Applications/Docker.app/Contents/Resources/bin:$PATH"
export TERM="${TERM:-xterm-256color}"
export RISC0_DEV_MODE=0
HEAVY_CARGO_VERIFY="${HEAVY_CARGO_VERIFY:-0}"

ARTIFACT_DIR="${ARTIFACT_DIR:-target/lp0002-risc0-fixture-new}"
BUNDLED_ARTIFACT_DIR="submission/proof-artifacts/lp0002-risc0-fixture-new"
CURRENT_E2E=0
if [[ "${1:-}" == "--current-e2e" || "${1:-}" == "--live-submit" ]]; then
  CURRENT_E2E=1
elif [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
Usage: bash scripts/demo-heavy-lane.sh [--current-e2e]

Default mode is recording-safe: it hash-validates the explicitly archived
pre-reset RISC0_DEV_MODE=0 evidence, then prints current LEZ v0.2.4 public
deployment/execution evidence from submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json.

--current-e2e runs scripts/lez-sequencer-e2e.sh, which generates a fresh proof,
deploys the current ProgramBinary, executes it, and checks canonical inclusion.
It requires LEZ_SOURCE_DIR to name the exact v0.2.4 source checkout.
--live-submit remains a compatibility alias for --current-e2e.
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

section "LP-0002 heavy lane: archived proof + current LEZ v0.2.4 evidence"
cat <<EOF
This demo shows the heavy-lane evidence for LP-0002:
  • bundled pre-reset RISC0_DEV_MODE=0 artifacts are hash-checked as history
  • their old image id is never presented as the current wrapper image id
  • current LEZ v0.2.4 standalone evidence is reproducible with --current-e2e
  • current deploy/execute block coordinates and explorer links are printed

Honesty note: current LEZ/RISC0 public-program sessions cannot carry the raw
270 KiB receipt inside the public wrapper input. The wrapper therefore carries
a receipt/journal commitment, while the full receipt remains retained and
verified as file artifact evidence.
EOF
pause "$SCENE_PAUSE"

section "1. Verify archived pre-reset proof artifacts"
materialize_bundled_artifacts
require_file "$ARTIFACT_DIR/receipt.borsh"
require_file "$ARTIFACT_DIR/journal.borsh"
require_file "$ARTIFACT_DIR/manifest.txt"
python3 - <<'PY'
import hashlib, json, os
from pathlib import Path
artifact_dir = Path(os.environ['ARTIFACT_DIR'])
expected = {
    'manifest.txt': '2d205c1d0ef91e7523d8dc68678dc68a2804875faa57e4b4c65b84f47daba946',
    'receipt.borsh': '6e4979983c996ca4154d7eeedb59444105b99d984a69a223ab58d429811b89a7',
    'journal.borsh': 'a8fe85f8d63f948409941b585cbe9244c2d0ae45082bf635173f753037ad4d8e',
    'lez-execution.json': '8938d0cfc197ad5579e88c8ef1430ae89a6636fc07f59f586fc08574370326bd',
    'spel-adapter-evidence.json': '01b7145e70635b7cb036774e8174f24496c3c2951b86c2747ab154405be42adf',
    'nssa-submit-dry-run.json': '5a13e14dc557a4a662fba850f1dd3355aa8a62235ce19dda39e8e96d2f1972e3',
    'nssa-submit-evidence.json': '9b638b0cc6fcd790c139ac115505a09ee79d730ee74166a4896031742b4d7aa5',
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
print('archived manifest confirms its historical RISC0_DEV_MODE=0 image/proof ids and 2-of-3 threshold')
PY
if [[ "$HEAVY_CARGO_VERIFY" == "1" ]]; then
  echo "ERROR: HEAVY_CARGO_VERIFY cannot apply the current wrapper verifier to the archived old-image receipt." >&2
  echo "Run with --current-e2e to generate and verify a fresh current-image proof." >&2
  exit 2
else
  note "Archived receipt pins image 6fc85ce0...; current wrapper pins e57b90a0...."
  result "Archived RISC0_DEV_MODE=0 artifact hashes and manifest metadata match their audited historical evidence set"
fi
pause "$SCENE_PAUSE"

section "2. Inspect archived LEZ-shaped host bridge evidence"
note "Using bundled pre-reset lez-execution.json as historical evidence only."
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
result "The archived journal records the historical replay-protected LEZ-shaped execution gate"
pause "$SCENE_PAUSE"

section "3. Inspect archived NSSA/SPEL payload evidence"
note "These filenames and payloads are retained for pre-v0.2.4 audit history only."
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
result "Archived payload hash and receipt/journal commitment are deterministic and reproducible"
pause "$SCENE_PAUSE"

if [[ "$CURRENT_E2E" == "1" ]]; then
  section "4. Run current LEZ v0.2.4 standalone E2E"
  run_cmd bash scripts/lez-sequencer-e2e.sh
else
  section "4. Current LEZ v0.2.4 public-testnet evidence"
  note "Default mode prints current committed evidence without creating duplicate transactions."
fi

python3 - <<'PY'
import json
from pathlib import Path
p = Path('submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json')
if not p.exists():
    raise SystemExit(f'missing {p}')
data = json.loads(p.read_text())
if data.get('status') != 'confirmed_rpc_and_explorer_visible':
    raise SystemExit('current public-testnet evidence is not RPC-confirmed and explorer-visible')
if data.get('lez', {}).get('release') != 'v0.2.4' or data.get('lez', {}).get('risc0_dev_mode') != '0':
    raise SystemExit('current evidence must pin LEZ v0.2.4 with RISC0_DEV_MODE=0')
print(f"release: {data['lez']['release']}")
print(f"commit: {data['lez']['commit']}")
print(f"threshold_proof_image_id: {data['proof']['threshold_proof_image_id']}")
for label in ('deployment', 'execution'):
    tx = data[label]
    print(f"{label}_tx_hash: {tx['tx_hash']}")
    print(f"{label}_included_block_id: {tx['included_block_id']}")
    print(f"{label}_explorer_url: {tx['explorer_url']}")
PY
result "Current v0.2.4 deployment and proof-backed execution are RPC-confirmed and explorer-visible"
pause "$SCENE_PAUSE"

section "5. Final readiness gate"
run_cmd env RISC0_SKIP_BUILD=1 python3 scripts/validate-submission-readiness.py --skip-exec
result "LP-0002 heavy-lane evidence is documented and the base submission still validates"

echo
result "Heavy-lane demo complete"
