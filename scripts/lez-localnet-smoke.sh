#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="/opt/homebrew/bin:$HOME/bin:$HOME/.cargo/bin:/Applications/Docker.app/Contents/Resources/bin:$PATH"
export RISC0_DEV_MODE=0

if ! command -v lgs >/dev/null 2>&1; then
  echo "ERROR: lgs is not installed or not on PATH" >&2
  exit 1
fi
if ! command -v spel >/dev/null 2>&1; then
  echo "ERROR: spel is not installed or not on PATH" >&2
  exit 1
fi

ARTIFACT_DIR="${1:-target/lp0002-risc0-fixture}"
RECEIPT="$ARTIFACT_DIR/receipt.borsh"
JOURNAL="$ARTIFACT_DIR/journal.borsh"
MANIFEST="$ARTIFACT_DIR/manifest.txt"

echo "== LP-0002 LEZ/localnet smoke harness =="
echo "root=$ROOT"
echo "artifact_dir=$ARTIFACT_DIR"
echo

echo "== Checking scaffold/tooling =="
lgs doctor || true
spel --help >/dev/null || true

echo
echo "== Building RISC0 host/method crates =="
cargo test -p lp0002-private-multisig-host --tests

echo
echo "== Ensuring real RISC0 proof artifacts exist =="
if [[ ! -s "$RECEIPT" || ! -s "$JOURNAL" ]]; then
  cargo run -p lp0002-private-multisig-host --bin lp0002-prove-fixture -- "$ARTIFACT_DIR"
elif ! cargo run -p lp0002-private-multisig-host --bin lp0002-verify-artifacts -- "$ARTIFACT_DIR"; then
  echo "Existing RISC0 fixture does not match the current image id; regenerating..."
  rm -rf "$ARTIFACT_DIR"
  cargo run -p lp0002-private-multisig-host --bin lp0002-prove-fixture -- "$ARTIFACT_DIR"
fi
cargo run -p lp0002-private-multisig-host --bin lp0002-verify-artifacts -- "$ARTIFACT_DIR"
cargo run -p lp0002-private-multisig-host --bin lp0002-lez-execute-artifacts -- "$ARTIFACT_DIR"
cargo run -p lp0002-private-multisig-host --bin lp0002-spel-adapter-evidence -- "$ARTIFACT_DIR"
cargo run -p lp0002-private-multisig-host --bin lp0002-submit-localnet -- "$ARTIFACT_DIR"

echo
echo "== Starting/checking LEZ localnet =="
lgs localnet start || true
lgs localnet status

echo
echo "== IDL and deployment surface checks =="
python3 interfaces/compute_discriminators.py >/tmp/lp0002-discriminators.txt
python3 scripts/validate-submission-readiness.py --skip-exec

# This legacy smoke harness validates host artifacts and the local scaffold
# surface. It does not replace the current LEZ v0.2.4 standalone-sequencer E2E.
# Use scripts/demo-heavy-lane.sh for a recording-safe archived/current evidence
# split, or scripts/demo-heavy-lane.sh --current-e2e for a fresh current proof,
# deployment, execution, and canonical inclusion check.

echo
echo "== Artifact manifest =="
cat "$MANIFEST"

echo
echo "PASS: current RISC0 proof artifacts verified, host wrapper execution accepted them, adapter/file-backed transaction evidence was built, and the legacy scaffold localnet is reachable."
echo "NEXT: for current standalone-sequencer inclusion evidence, run scripts/demo-heavy-lane.sh --current-e2e; CU metering remains unavailable until LEZ exposes stable counters."
