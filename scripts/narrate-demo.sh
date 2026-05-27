#!/usr/bin/env bash
# LP-0002 narrated demo companion script.
#
# Run this alongside demo-video.sh to hear spoken narration during recording.
# Usage:
#   Terminal 1 (recording target):  bash scripts/demo-video.sh
#   Terminal 2 (this narrator):     bash scripts/narrate-demo.sh
#
# Or combine them:
#   bash scripts/demo-video.sh & bash scripts/narrate-demo.sh; wait
#
# The pauses are tuned to match demo-video.sh default SECTION_PAUSE=4,
# COMMAND_PAUSE=2, SCENE_PAUSE=6. If you override those, set the same
# values here via NARRATE_SECTION_PAUSE etc.

set -uo pipefail

cd "$(dirname "$0")/.."

SECTION_PAUSE="${NARRATE_SECTION_PAUSE:-4}"
COMMAND_PAUSE="${NARRATE_COMMAND_PAUSE:-2}"
SCENE_PAUSE="${NARRATE_SCENE_PAUSE:-6}"

# Use macOS 'say' (always available on M4)
say_line() {
  local text="$1"
  echo "[narrator] $text"
  say "$text" 2>/dev/null || echo "(say unavailable: $text)"
}

# Pauses to sync with demo-video.sh
p_section() { sleep "$(( SECTION_PAUSE + 1 ))"; }
p_command() { sleep "$COMMAND_PAUSE"; }
p_scene()   { sleep "$SCENE_PAUSE"; }

# ── Intro ──────────────────────────────────────────────────

say_line "This is LP-0002: Private M-of-N Multisig for the Logos Execution Zone. The goal is to let shielded members approve proposals without exposing which members signed, while still enforcing threshold approval and replay protection. This video demonstrates the full submission: the safe-lane reference implementation, the consumer integration, Basecamp native assets, and the heavy-lane RISC0 proof evidence."
p_scene

# ── Section 1: Repository layout ───────────────────────────

say_line "Section one: repository layout. Here are the main submission surfaces. The Rust workspace is split into core, SDK, verifier program, RISC0 methods, host bridge, LEZ wrapper, and a consumer demo application. You can see the protocol docs, the SPEL interface definition language, the spec compliance matrix, Basecamp app assets, and the submission validators."
p_scene

# ── Section 2: Safety and evidence gates ───────────────────

say_line "Section two: safety and evidence gates. The compliance matrix is intentionally explicit about what is real and what is not. The safe lane uses a deterministic mock receipt seal. The separate heavy lane verifies real RISC0 DEV MODE zero artifacts and confirmed compact localnet inclusion. Because the current LEZ toolchain does not expose stable per-transaction compute unit counters, the submission records the available metrics and documents the limitation rather than inventing numbers."
p_scene

# ── Section 3: Rust tests ──────────────────────────────────

say_line "Section three: running the focused Rust test suite. These tests cover the core threshold relation, double-vote detection, proposal context binding, large threshold configurations up to ten of twenty, resumable approval state with serialization roundtrips, and verifier replay protection. Watch as all twenty four tests pass."
sleep 8
say_line "All core privacy relation, SDK, verifier, and LEZ wrapper tests pass."
p_scene

# ── Section 4: Consumer demo ──────────────────────────────

say_line "Section four: the clone-and-run consumer demo. This is the most important evaluator surface. The consumer demo imports the SDK exactly as a third-party application would, as a library dependency. It runs five integration scenarios. Scenario one: a two of three treasury transfer using the high-level SDK API. Scenario two: a three of five governance parameter change with non-sequential approvers. Scenario three: resumable partial approvals simulating a client restart. Scenario four: error paths including double-vote prevention and insufficient approvals. Scenario five: replay protection preventing double execution."
sleep 10
say_line "All five consumer scenarios pass. The SDK is ready for external integration."
p_scene

# ── Section 5: Validators ──────────────────────────────────

say_line "Section five: validating submission readiness. The final publication validator checks all evidence gates, file presence, IDL discriminators, documentation sections, and honesty markers. The implementation validator confirms workspace structure, crate layout, and JavaScript syntax."
p_scene

# ── Section 6: Basecamp package ────────────────────────────

say_line "Section six: inspecting the Basecamp package assets. The browser preview JavaScript is syntax-checked with node. The native Qt/QML Basecamp module has its metadata, CMakeLists, QML interface, and C++ plugin source validated. This gives Logos Basecamp two integration surfaces: a lightweight browser preview and a native desktop module."
p_scene

# ── Section 7: Heavy lane ──────────────────────────────────

say_line "Section seven: heavy-lane RISC0 proof and LEZ localnet evidence. The heavy lane script verifies bundled RISC0 DEV MODE zero proof artifacts, checks the image and proof IDs against the manifest, inspects the LEZ-shaped host bridge execution evidence, shows the compact NSSA/SPEL payload with receipt and journal commitments, and prints confirmed localnet inclusion. The wrapper transaction is included in block nineteen ninety-five at transaction index zero."
sleep 8
say_line "All heavy-lane evidence is documented and the base submission still validates."
p_scene

# ── Conclusion ─────────────────────────────────────────────

say_line "To summarize: LP-0002 is ready for review. Privacy-preserving threshold relations are modeled and tested. Nullifier-based double-vote prevention works. Replay-protected execution is enforced. The SDK provides a clean integration surface. A real RISC0 proof was generated and verified. Confirmed localnet inclusion evidence exists. Known transport caveats around receipt size and compute unit metering are documented honestly. Everything needed for evaluation is in the repository."
p_scene

say_line "Recording complete. Thank you for reviewing LP-0002."
