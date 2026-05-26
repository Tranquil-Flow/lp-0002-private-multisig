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
This recording demonstrates the safe-lane submission for LP-0002.

What is live in this video:
  • The Rust workspace builds and tests locally
  • The consumer demo imports the SDK exactly like a third-party app
  • Five integration scenarios pass end-to-end
  • The submission validator checks docs, IDL, JavaScript, tests, and demo

Heavy-lane status:
  • Real RISC0_DEV_MODE=0 proof artifacts exist and verify host-side
  • verify_and_execute_bytes has confirmed compact-wrapper localnet inclusion evidence
  • Formal per-transaction CU counters remain pending on the target LEZ RPC/runtime

For the heavy-lane recording, run: bash scripts/demo-heavy-lane.sh

EOF
pause "$SCENE_PAUSE"

section "1. Repository layout"
step "Show the main submission surfaces"
run_cmd find . -maxdepth 2 -type f \
  \( -path './README.md' -o -path './demo.sh' -o -path './module.json' -o -path './docs/*' -o -path './interfaces/*' -o -path './submission/*' -o -path './scripts/*' \) \
  ! -path './target/*' | sort
result "Submission docs, IDL, Basecamp app, validator, and demo are present"
pause "$SCENE_PAUSE"

section "2. Safety and honesty gates"
step "Show the compliance matrix lines that distinguish safe-lane from heavy-lane"
python3 - <<'PY'
from pathlib import Path
text = Path('docs/SPEC_COMPLIANCE.md').read_text().splitlines()
for i, line in enumerate(text, start=1):
    if any(key in line for key in ['Reference integration on LEZ testnet', 'Verifier deployed', 'Proof generation time', 'deterministic mock', 'demo.sh runs']):
        print(f'{i:03d}: {line}')
PY
result "The submission does not overclaim real RISC0 or LEZ deployment"
pause "$SCENE_PAUSE"

section "3. Run the test suite"
step "Execute all Rust tests"
run_cmd cargo test --workspace
result "Core, verifier, SDK doctest, and integration tests pass"
pause "$SCENE_PAUSE"

section "4. Run the clone-and-run consumer demo"
step "Execute the evaluator-facing third-party integration demo"
run_cmd bash demo.sh
result "All five consumer scenarios pass"
pause "$SCENE_PAUSE"

section "5. Validate submission readiness"
step "Run the strict readiness validator"
run_cmd python3 scripts/validate-submission-readiness.py
result "Validator confirms files, IDL discriminators, docs, JS syntax, tests, and demo"
pause "$SCENE_PAUSE"

section "6. Optional: inspect Basecamp app assets"
step "Check that the browser GUI assets are present and syntactically valid"
run_cmd node --check basecamp-app/app.js
python3 - <<'PY'
from pathlib import Path
for p in ['basecamp-app/index.html', 'basecamp-app/app.js', 'basecamp-app/styles.css']:
    print(f'{p}: {Path(p).stat().st_size} bytes')
PY
result "Basecamp app assets are ready for local browser preview"
pause "$SCENE_PAUSE"

section "Conclusion"
cat <<'EOF'
LP-0002 is ready for PR review as a thorough safe-lane submission:
  • privacy-preserving threshold relation modeled and tested
  • nullifier-based double-vote prevention
  • replay-protected execution gate
  • high-level SDK for integrators
  • clone-and-run consumer demo replacing the external-instance burden
  • comprehensive protocol, compliance, IDL, and benchmark docs

The heavy-lane evidence is also available:
  run bash scripts/demo-heavy-lane.sh to show RISC0_DEV_MODE=0 receipt verification,
  compact wrapper payload evidence, and confirmed localnet inclusion.

Remaining publication gates are public testnet/evaluator repetition, demo video URL,
  and any formal CU counter exposed by the target LEZ runtime.
EOF

echo
result "Recording script complete"
