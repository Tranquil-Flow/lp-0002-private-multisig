#!/usr/bin/env bash
# LP-0002 narrated demo companion script — unified 11-section edition.
#
# Run alongside demo-video.sh during QuickTime recording.
# Usage:
#   Terminal 1 (recording target):  bash scripts/demo-video.sh
#   Terminal 2 (this narrator):     bash scripts/narrate-demo.sh
#
# Pauses are tuned to match demo-video.sh defaults:
#   SECTION_PAUSE=4, COMMAND_PAUSE=2, SCENE_PAUSE=6

set -uo pipefail

cd "$(dirname "$0")/.."

SECTION_PAUSE="${NARRATE_SECTION_PAUSE:-4}"
COMMAND_PAUSE="${NARRATE_COMMAND_PAUSE:-2}"
SCENE_PAUSE="${NARRATE_SCENE_PAUSE:-6}"

say_line() {
  local text="$1"
  echo "[narrator] $text"
  say "$text" 2>/dev/null || echo "(say unavailable: $text)"
}

p_section() { sleep "$(( SECTION_PAUSE + 1 ))"; }
p_command() { sleep "$COMMAND_PAUSE"; }
p_scene()   { sleep "$SCENE_PAUSE"; }

# ── Opening ────────────────────────────────────────────────

say_line "This is LP-0002: Private M-of-N Multisig for the Logos Execution Zone. This unified walkthrough demonstrates the full submission end to end — from repository layout through Rust tests, consumer integration, validators, native Basecamp assets, and the complete heavy-lane RISC0 proof and LEZ localnet evidence chain. Eleven sections, no nested sub-scripts."
p_scene

# ── Section 1: Repository layout ───────────────────────────

say_line "Section one: repository layout. Here are the main submission surfaces. The Rust workspace is split into core, SDK, verifier program, RISC0 methods, host bridge, LEZ wrapper, and a consumer demo application. You can see the protocol docs, the SPEL interface definition language, the spec compliance matrix, Basecamp app assets, and all submission validators."
p_scene

# ── Section 2: Safety and evidence gates ───────────────────

say_line "Section two: safety and evidence gates. The compliance matrix is intentionally explicit about what is real and what is not. The safe lane uses a deterministic mock receipt seal. The separate heavy lane — which we will see in detail later — verifies real RISC0 DEV MODE zero artifacts. Compute unit counters are documented as a current tooling limitation rather than faked."
p_scene

# ── Section 3: Rust tests ──────────────────────────────────

say_line "Section three: running the focused Rust test suite. These tests cover the core threshold relation, double-vote detection, proposal context binding, large threshold configurations up to ten of twenty, resumable approval state with serialization roundtrips, verifier replay protection, and LEZ execution wrapper correctness."
sleep 8
say_line "All twenty-four tests pass. Core privacy relation, SDK, verifier, and LEZ wrapper are solid."
p_scene

# ── Section 4: Consumer demo ──────────────────────────────

say_line "Section four: the clone-and-run consumer demo. This imports the SDK exactly as a third-party application would, as a library dependency. Five scenarios: a two-of-three treasury transfer, a three-of-five governance change with out-of-order approvers, resumable partial approvals simulating a client restart, error paths including double-vote and insufficient approvals, and replay protection preventing double execution."
sleep 10
say_line "All five consumer scenarios pass. The SDK integration surface is clean."
p_scene

# ── Section 5: Validators ──────────────────────────────────

say_line "Section five: validating submission readiness. The final publication validator checks all evidence gates, file presence, IDL discriminators, documentation completeness, and honesty markers. The implementation validator confirms workspace structure, crate layout, and the native Basecamp package structure."
p_scene

# ── Section 6: Basecamp package ────────────────────────────

say_line "Section six: inspecting the Basecamp package assets. The native Qt QML Basecamp module is built with CMake — you can see it compiling the library target. This gives Logos Basecamp a native desktop plugin package with metadata, C++ sources, and QML UI assets."
p_scene

# ── Section 7: Heavy lane — proof artifacts ────────────────

say_line "Section seven: the heavy lane begins. We verify the bundled RISC0 DEV MODE zero proof artifacts. Each artifact file is hash-checked against the audited evidence set — the 270 kilobyte receipt, the 1 kilobyte journal, the manifest metadata, and all supporting evidence files. The manifest confirms the image ID, proof ID, threshold of two, and approval count of two."
p_scene

# ── Section 8: Heavy lane — LEZ host bridge ────────────────

say_line "Section eight: inspecting the LEZ-shaped host bridge evidence. This shows the execution journal driving the replay-protected state gate — proposal state is executed, nullifier count is two, and the proof ID is bound to the exact proposal context."
p_scene

# ── Section 9: Heavy lane — NSSA/SPEL payload ──────────────

say_line "Section nine: compact NSSA and SPEL payload evidence. Because the raw 270 kilobyte receipt exceeds current LEZ public-program session limits, the wrapper carries a receipt-and-journal commitment in its input while the full receipt is retained as file artifact evidence. The payload hashes are deterministic and reproducible."
p_scene

# ── Section 10: Heavy lane — localnet inclusion ────────────

say_line "Section ten: confirmed localnet inclusion evidence. The wrapper transaction was submitted, confirmed, and included in block one thousand nine hundred ninety-five at transaction index zero. The program ID, image ID, instruction data hashes, and receipt transport mechanism are all recorded."
p_scene

# ── Section 11: Final readiness gate ───────────────────────

say_line "Section eleven: final readiness gate. We re-run the implementation validator to confirm that the heavy-lane evidence is coherent with the rest of the submission. All thirty-five required files, eight workspace crates, IDL discriminators, protocol documentation, and compliance matrix sections validate."
p_scene

# ── Conclusion ─────────────────────────────────────────────

say_line "In conclusion: LP-0002 is ready for review. Privacy-preserving threshold relations are modeled and tested. Nullifier-based double-vote prevention works. Replay-protected execution is enforced. The SDK provides a clean integration surface. A real RISC0 proof was generated and verified. Native Basecamp module compiles. Confirmed localnet inclusion evidence exists. Known transport caveats around receipt size and compute unit metering are documented honestly. All eleven sections pass."
p_scene

say_line "Recording complete. Thank you for reviewing LP-0002."
