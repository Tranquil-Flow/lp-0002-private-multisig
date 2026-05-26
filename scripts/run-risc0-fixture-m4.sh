#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="/opt/homebrew/bin:$HOME/bin:$HOME/.cargo/bin:/Applications/Docker.app/Contents/Resources/bin:$PATH"
export RISC0_DEV_MODE=0

cargo run -p lp0002-private-multisig-host --bin lp0002-prove-fixture -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-verify-artifacts -- target/lp0002-risc0-fixture
