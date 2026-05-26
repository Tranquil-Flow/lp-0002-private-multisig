#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

cat <<'EOF'
LP-0002 Private M-of-N Multisig demo
Mode: safe-lane reference implementation
Note: this demo uses the deterministic mock receipt path. Real RISC0_DEV_MODE=0 proof artifacts and compact LEZ localnet wrapper
      inclusion evidence exist separately; run scripts/demo-heavy-lane.sh.
EOF

echo
cargo run -p lp0002-consumer-demo
