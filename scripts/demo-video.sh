#!/usr/bin/env bash
set -euo pipefail

# LP-0002 demo video script for QuickTime recording.
# Usage:
#   bash scripts/demo-video.sh
#   SECTION_PAUSE=0 COMMAND_PAUSE=0 SCENE_PAUSE=0 bash scripts/demo-video.sh  # dry run

cd "$(dirname "$0")/.."
export PATH="/opt/homebrew/bin:$HOME/bin:/root/.cargo/bin:$HOME/.cargo/bin:$PATH"
export TERM="${TERM:-xterm-256color}"

SECTION_PAUSE="${SECTION_PAUSE:-4}"
COMMAND_PAUSE="${COMMAND_PAUSE:-2}"
SCENE_PAUSE="${SCENE_PAUSE:-6}"

BOLD='\033[1m'
DIM='\033[2m'
GREEN='\033[32m'
CYAN='\033[36m'
YELLOW='\033[33m'
RED='\033[31m'
RESET='\033[0m'

pause() { sleep "${1:-$COMMAND_PAUSE}"; }
clear_screen() { printf '\033[2J\033[H'; }
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

clear_screen
section "LP-0002: Private M-of-N Multisig"
cat <<'EOF'
This recording demonstrates the final LP-0002 submission package.

What is live in this video:
  • The consumer demo imports the SDK exactly like a third-party app
  • Five integration scenarios pass end-to-end
  • The submission validator checks docs, IDL, JavaScript, and evidence gates
  • Bundled RISC0_DEV_MODE=0 proof artifacts are hash-checked
  • Confirmed compact LEZ/NSSA localnet inclusion evidence is shown

Honesty note:
  • The current LEZ/RISC0 public-program transport cannot carry the raw 270 KiB
    RISC0 receipt inside wrapper input, so the wrapper carries compact
    receipt/journal commitments while the full receipt remains host-verified
    and file-backed as artifact evidence.
  • The available LEZ tooling does not expose stable per-transaction CU counters;
    this submission records that limitation rather than inventing numbers.

EOF
pause "$SCENE_PAUSE"

section "1. Repository layout"
step "Show the main submission surfaces"
run_cmd find . -maxdepth 2 -type f \
  \( -path './README.md' -o -path './demo.sh' -o -path './module.json' -o -path './docs/*' -o -path './interfaces/*' -o -path './submission/*' -o -path './scripts/*' \) \
  ! -path './target/*' | sort
result "Submission docs, IDL, Basecamp app, validator, and demo are present"
pause "$SCENE_PAUSE"

section "2. Safety and evidence gates"
step "Show the compliance matrix lines that distinguish full proof artifacts, compact transport, and CU limits"
python3 - <<'PY'
from pathlib import Path
text = Path('docs/SPEC_COMPLIANCE.md').read_text().splitlines()
keys = [
    'Reference integration on LEZ testnet',
    'Verifier deployed',
    'Proof generation time',
    'receipt/journal',
    'CU',
    'deterministic mock',
]
for i, line in enumerate(text, start=1):
    if any(key in line for key in keys):
        print(f'{i:03d}: {line}')
PY
result "The submission separates clone-and-run behavior, real proof artifacts, compact localnet transport, and unavailable CU counters"
pause "$SCENE_PAUSE"

section "3. Run focused Rust tests"
step "Execute the fast evaluator-safe Rust test subset"
run_cmd env RISC0_SKIP_BUILD=1 cargo test -p lp0002-private-multisig-core -p lp0002-private-multisig-sdk -p lp0002-private-multisig-verifier --tests
run_cmd env RISC0_SKIP_BUILD=1 cargo test -p lp0002-private-multisig-lez-program --tests
result "Core privacy relation, SDK, verifier, and LEZ wrapper tests pass"
pause "$SCENE_PAUSE"

section "4. Run the clone-and-run consumer demo"
step "Execute the evaluator-facing third-party integration demo"
run_cmd bash demo.sh
result "All five consumer scenarios pass"
pause "$SCENE_PAUSE"

section "5. Validate submission readiness"
step "Run the strict final publication validator"
run_cmd python3 scripts/final-publication-check.py
step "Run the local implementation validator in evaluator-safe mode"
run_cmd env RISC0_SKIP_BUILD=1 python3 scripts/validate-submission-readiness.py --skip-exec
result "Validators confirm files, IDL discriminators, docs, JS syntax, evidence, and publication gates"
pause "$SCENE_PAUSE"

section "6. Inspect Basecamp package assets"
step "Check browser-preview JavaScript and native Basecamp package sources"
run_cmd node --check basecamp-app/app.js
run_cmd bash scripts/validate-basecamp-native.sh
python3 - <<'PY'
from pathlib import Path
for p in ['basecamp-app/index.html', 'basecamp-app/app.js', 'basecamp-app/CMakeLists.txt', 'basecamp-app/metadata.json', 'basecamp-app/qml/Lp0002PrivateMultisig.qml']:
    print(f'{p}: {Path(p).stat().st_size} bytes')
PY
result "Basecamp preview and native Qt/QML package surfaces are present"
pause "$SCENE_PAUSE"

section "7. Show heavy-lane RISC0 and LEZ localnet evidence"
step "Replay the recording-safe heavy-lane evidence script"
run_cmd env SECTION_PAUSE=0 COMMAND_PAUSE=0 SCENE_PAUSE=0 bash scripts/demo-heavy-lane.sh
result "RISC0 proof artifacts, compact wrapper evidence, and confirmed localnet inclusion are shown"
pause "$SCENE_PAUSE"

section "Conclusion"
cat <<'EOF'
LP-0002 is ready for PR review as a polished submission package:
  • privacy-preserving threshold relation modeled and tested
  • nullifier-based double-vote prevention
  • replay-protected execution gate
  • high-level SDK for integrators
  • clone-and-run consumer demo replacing the external-instance burden
  • native Qt/QML Basecamp package source and validation
  • real RISC0_DEV_MODE=0 proof artifacts retained as reproducible evidence
  • confirmed compact LEZ/NSSA localnet wrapper inclusion evidence

Known transport/runtime caveats are documented rather than hidden:
  • wrapper input carries compact receipt/journal commitments, not the raw receipt
  • current LEZ tooling does not expose stable per-transaction CU counters

Everything needed for review is linked from README.md, submission/EVALUATOR_GUIDE.md,
solutions/LP-0002.md, and the root demo scripts.
EOF

echo
result "Recording script complete"
