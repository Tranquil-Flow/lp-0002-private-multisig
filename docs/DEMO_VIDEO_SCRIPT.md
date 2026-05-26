# LP-0002 Demo Video Narration Guide

Use QuickTime Player -> File -> New Screen Recording. Open Terminal in this repo,
increase font size to ~18pt, then run:

```bash
bash scripts/demo-video.sh
```

For a fast dry-run before recording:

```bash
SECTION_PAUSE=0 COMMAND_PAUSE=0 SCENE_PAUSE=0 bash scripts/demo-video.sh
```

## Recording stance

Be precise: `scripts/demo-video.sh` is the **safe-lane consumer demo**. The
heavy-lane evidence is now available separately via `scripts/demo-heavy-lane.sh`,
which verifies real `RISC0_DEV_MODE=0` artifacts, builds compact wrapper payload
evidence, and prints confirmed localnet inclusion. Formal public testnet/evaluator
repetition and CU counters remain publication gates.

## Suggested narration

### Opening

"This is LP-0002, private M-of-N multisig for the Logos Execution Zone. The goal
is to let shielded members approve proposals without exposing which members
approved, while still enforcing threshold approval and replay protection. This
video demonstrates the safe-lane reference implementation and the consumer demo
that Mart1n said is acceptable in place of the external-instance burden."

### Section 1 — Repository layout

"Here are the submission surfaces: the Rust workspace, protocol docs, spec
compliance matrix, SPEL IDL, Basecamp app assets, benchmark notes, and the
validator. The workspace is split into core, verifier-program, SDK, and a
consumer-demo application."

### Section 2 — Safety and honesty gates

"The compliance matrix is intentionally explicit about what is done and what is
not. The safe-lane uses a deterministic mock receipt. The separate heavy-lane
demo verifies real RISC0 artifacts and confirmed compact localnet inclusion, but
public testnet repetition and formal CU counters are still final publication gates.
I don't want to overclaim this as production-final ZK."

### Section 3 — Tests

"The test suite covers the core threshold relation, double-vote detection,
proposal/context binding, large threshold configurations, resumable approval
state, and verifier replay protection."

### Section 4 — Consumer demo

"This is the most important evaluator surface. It imports the SDK exactly as a
third-party application would. It runs five scenarios: a 2-of-3 treasury
transfer, a 3-of-5 governance parameter change with non-sequential approvers,
resumable partial approvals across a simulated restart, error paths for duplicate
and insufficient approvals, and replay protection."

### Section 5 — Validator

"The readiness validator checks more than file presence: it validates JSON,
verifies SPEL discriminators, checks the browser GUI JavaScript syntax, enforces
honesty gates in the docs, and runs tests plus the demo."

### Section 6 — Basecamp app

"The current GUI is a browser-facing walkthrough with local JavaScript crypto.
It is not the on-chain verifier; it's an integration and visualization surface
for the private multisig flow."

### Closing

"This submission is ready for review as a thorough implementation: the protocol
is modeled, tested, documented, and easy to integrate. For the heavy lane, run
`scripts/demo-heavy-lane.sh` to show real RISC0 artifact verification, compact
wrapper payload evidence, and confirmed localnet inclusion. The final remaining
gates are public testnet/evaluator repetition, the narrated video URL, and any
formal CU counter exposed by the target runtime."

## Important phrases to avoid

Do **not** say:

- "The safe-lane demo generates a real RISC0 proof" — it does not; use `scripts/demo-heavy-lane.sh` for RISC0 artifact evidence.
- "This is deployed on public LEZ testnet" — current evidence is localnet.
- "These are measured LEZ gas/CU costs" — current evidence is payload/hash/inclusion plus sequencer execution time, not a formal CU meter.

Safe phrasing:

- "safe-lane reference implementation"
- "deterministic mock receipt"
- "RISC0/LEZ heavy-lane localnet evidence available"
- "clone-and-run consumer demo accepted by maintainer clarification"
