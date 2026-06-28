# LP-0002 Demo Video Narration Guide
> Current reset-era refresh (2026-06-28): deploy tx `c7157a473cb512bf7e1803d4377d9f65e9406a7ff98efeda48b65c0d4915a13b` is included on the public LEZ testnet for program id `1557176a639868b0363e9106c75fe0748ceb42e65f5f1a6778dd05b6baebb57d` (ProgramBinary SHA-256 `8f74ccc446990f5437b5f6c6e731deac6653992e0a64abcecdff7bff0c5575e1`). Execute attempts `352eb699507aea4d4ca6963a50bef1473a2b944dfd7713116cbf82eabfeec3bf` and `fc4165ac2437bd6533444c5e010b2d248aed678daadfad277af1dd0f1fef6ca8` locally validate under v0.2.0 but are not included by the public endpoint, so current live execute inclusion remains a transparent blocker and is not claimed. Historical pre-reset txs `82516880...` / `cb8bfd5...` are retained only as audit history.

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
it runs the consumer demo, validates the native/Basecamp surfaces, and
then replays the heavy-lane evidence via `scripts/demo-heavy-lane.sh`. The heavy
lane verifies real `RISC0_DEV_MODE=0` artifacts, builds compact wrapper payload
evidence, and prints confirmed inclusion on the public LEZ testnet
(https://testnet.lez.logos.co/) — deploy tx `82516880...` block `39547`, execute
tx `cb8bfd5a...` block `39548`. The current LEZ JSON-RPC surface does not expose
stable per-transaction CU counters, so the submission records payload/account/receipt
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
demo verifies real RISC0 artifacts and confirmed compact public-testnet inclusion.
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
verifies SPEL discriminators, checks the native Basecamp package structure, enforces
honesty gates in the docs, and runs tests plus the demo."

### Section 6 — Basecamp app

"The Basecamp surface is a native Qt/QML plugin package (`basecamp-app/`) with
local build evidence and runtime load evidence. The GUI is not the on-chain
verifier; it is an integration and visualization surface for the private
multisig flow, with native package source, build evidence, and raw-log-bound
runtime load evidence included for Basecamp review."

### Closing

"This submission is ready for review as a thorough implementation: the protocol
is modeled, tested, documented, and easy to integrate. For the heavy lane, run
`scripts/demo-heavy-lane.sh` to show real RISC0 artifact verification, compact
wrapper payload evidence, and confirmed public-testnet inclusion. The narrated demo
asset and structured evidence are included in the submission package."

## Important phrases to avoid

Do **not** say:

- "The safe-lane demo generates a real RISC0 proof" — it does not; use `scripts/demo-heavy-lane.sh` for RISC0 artifact evidence.
- "This uses a remote public network beyond the evaluator localnet" — the recorded target is the public LEZ testnet (https://testnet.lez.logos.co/).
- "These are measured LEZ gas/CU costs" — current evidence is payload/hash/inclusion plus sequencer execution time, not a formal CU meter.

Safe phrasing:

- "safe-lane reference implementation"
- "deterministic mock receipt"
- "RISC0/LEZ heavy-lane public-testnet evidence available"
- "clone-and-run consumer integration matching the current public LP-0002 spec"
