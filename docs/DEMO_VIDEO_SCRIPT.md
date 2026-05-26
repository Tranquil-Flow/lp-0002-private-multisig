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

Be precise: `scripts/demo-video.sh` is the **combined recording walkthrough**:
it runs the safe-lane consumer demo, validates the native/Basecamp surfaces, and
then replays the heavy-lane evidence via `scripts/demo-heavy-lane.sh`. The heavy
lane verifies real `RISC0_DEV_MODE=0` artifacts, builds compact wrapper payload
evidence, and prints confirmed localnet inclusion for the LP-0002 evaluator /
public-testnet target. The current LEZ JSON-RPC surface does not expose stable
per-transaction CU counters, so the submission records payload/account/receipt
metrics plus an explicit CU-metering limitation.

## Suggested narration

### Opening

"This is LP-0002, private M-of-N multisig for the Logos Execution Zone. The goal
is to let shielded members approve proposals without exposing which members
approved, while still enforcing threshold approval and replay protection. This
video demonstrates the safe-lane reference implementation and the clone-and-run
consumer integration. The current public LP-0002 spec asks for reproducible
evidence for one multisig instance, not five external operators."

### Section 1 — Repository layout

"Here are the submission surfaces: the Rust workspace, protocol docs, spec
compliance matrix, SPEL IDL, Basecamp app assets, benchmark notes, and the
validator. The workspace is split into core, verifier-program, SDK, and a
consumer-demo application."

### Section 2 — Safety and honesty gates

"The compliance matrix is intentionally explicit about what is done and what is
not. The safe-lane uses a deterministic mock receipt. The separate heavy-lane
demo verifies real RISC0 artifacts and confirmed compact localnet inclusion.
Because this LEZ toolchain does not expose stable per-transaction CU counters,
the submission records the available metrics and the limitation rather than
inventing a cost number."

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

"The Basecamp surface includes a native Qt/QML plugin package plus a browser
preview. The GUI is not the on-chain verifier; it is an integration and
visualization surface for the private multisig flow, with native package source
and build evidence included for Basecamp review."

### Closing

"This submission is ready for review as a thorough implementation: the protocol
is modeled, tested, documented, and easy to integrate. For the heavy lane, run
`scripts/demo-heavy-lane.sh` to show real RISC0 artifact verification, compact
wrapper payload evidence, and confirmed localnet inclusion. The narrated demo
asset and structured evidence are included in the submission package."

## Important phrases to avoid

Do **not** say:

- "The safe-lane demo generates a real RISC0 proof" — it does not; use `scripts/demo-heavy-lane.sh` for RISC0 artifact evidence.
- "This uses a remote public network beyond the evaluator localnet" — the recorded target is the LP-0002 evaluator/public-testnet localnet.
- "These are measured LEZ gas/CU costs" — current evidence is payload/hash/inclusion plus sequencer execution time, not a formal CU meter.

Safe phrasing:

- "safe-lane reference implementation"
- "deterministic mock receipt"
- "RISC0/LEZ heavy-lane localnet evidence available"
- "clone-and-run consumer integration matching the current public LP-0002 spec"
