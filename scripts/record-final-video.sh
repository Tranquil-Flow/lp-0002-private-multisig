#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export TERM="${TERM:-xterm-256color}"
BOLD='[1m'; DIM='[2m'; GREEN='[32m'; CYAN='[36m'; YELLOW='[33m'; RESET='[0m'
SECTION_PAUSE="${SECTION_PAUSE:-4}"
COMMAND_PAUSE="${COMMAND_PAUSE:-1}"
SCENE_PAUSE="${SCENE_PAUSE:-5}"
RUN_COMMANDS="${RUN_COMMANDS:-1}"
pause(){ sleep "${1:-$SECTION_PAUSE}"; }
scene(){ printf '
%b════════════════════════════════════════════════════════════%b
' "$BOLD$CYAN" "$RESET"; printf '%b  %s%b
' "$BOLD$CYAN" "$1" "$RESET"; printf '%b════════════════════════════════════════════════════════════%b

' "$BOLD$CYAN" "$RESET"; pause "$SECTION_PAUSE"; }
say(){ printf '%s
' "$*"; }
cmd(){ printf '%b$ %s%b
' "$DIM" "$*" "$RESET"; if [[ "$RUN_COMMANDS" == "1" ]]; then bash -lc "$*"; else printf '  [dry display only]
'; fi; printf '
'; pause "$COMMAND_PAUSE"; }
soft(){ printf '%b$ %s%b
' "$DIM" "$*" "$RESET"; if [[ "$RUN_COMMANDS" == "1" ]]; then bash -lc "$*" || true; else printf '  [dry display only]
'; fi; printf '
'; pause "$COMMAND_PAUSE"; }

clear || true
scene "LP-0002 final resubmission demo — private M-of-N multisig"
say "This recording demonstrates the final LP-0002 package. The repository now has proof, SDK, public LEZ testnet, and Basecamp runtime-host evidence."
say "The remaining requirement is this fresh human video showing the visible Basecamp activation path."
pause "$SCENE_PAUSE"
scene "1. Repository and final gate"
cmd "git log -1 --oneline"
soft "python3 scripts/final-publication-check.py"
say "Before the video URL is inserted, the final gate should fail only on the missing narrated video URL."
pause "$SCENE_PAUSE"
scene "2. Implementation and consumer integration"
cmd "env RISC0_SKIP_BUILD=1 cargo test -p lp0002-private-multisig-core -p lp0002-private-multisig-sdk -p lp0002-private-multisig-verifier --tests"
cmd "bash demo.sh"
scene "3. Proof and LEZ evidence"
cmd "python3 - <<'PY'
import json, pathlib
p=pathlib.Path('submission/proof-artifacts/lp0002-risc0-fixture-new/manifest.txt')
print(p.read_text())
PY"
cmd "python3 - <<'PY'
import json
from pathlib import Path
for rel in ['submission/proof-artifacts/lp0002-risc0-fixture-new/lez-execution.json','submission/proof-artifacts/lp0002-risc0-fixture-new/nssa-submit-evidence.json']:
    data=json.loads(Path(rel).read_text())
    print(rel)
    for k in ['status','confirmed','program_id','tx_hash','included_block_id','proof_id']:
        if k in data: print(f'  {k}: {data[k]}')
PY"
scene "4. Basecamp package and runtime-host evidence"
cmd "bash scripts/validate-basecamp-native.sh"
cmd "python3 - <<'PY'
import json
from pathlib import Path
for rel in ['submission/basecamp-install-evidence.json','submission/BASECAMP_RUNTIME_LOAD_EVIDENCE.json']:
    p=Path(rel)
    data=json.loads(p.read_text())
    print(rel)
    for k in ['status','loaded_component_id','loaded_artifact','runtime_launch_evidence','component_activation_evidence','component_activation_video_required','lgx_sha256']:
        if k in data: print(f'  {k}: {data[k]}')
PY"
scene "5. Visible Basecamp activation"
say "Now show the actual LogosBasecamp UI or launch flow on the M4 desktop. Open/activate the LP-0002 private multisig module so the evaluator can see the package is not only installed, but visible in Basecamp."
say "If recording from Terminal only, pause here and switch to the visible Basecamp window."
pause 12
scene "6. Closing"
say "LP-0002 is ready for resubmission once this video URL is inserted and final-publication-check.py passes."
printf '\n%bLP-0002 final video script complete.%b\n' "$GREEN" "$RESET"
