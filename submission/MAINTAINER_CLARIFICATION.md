# LP-0002 Adoption-Criteria Update
> Repository evidence refresh (2026-08-10): exact LEZ v0.2.4 commit `47eba256...` is current. Deploy tx `b15dc325...` is confirmed in block `2373`; proof-backed execute tx `4a985d6f...` is confirmed in block `2374`, index `0`; both direct explorer pages render. `scripts/lez-sequencer-e2e.sh` independently proves the non-dev-mode standalone path. Earlier reset/pre-reset records remain audit history only.

The current public LP-0002 prize text requires at least one reproducible multisig
instance with a proposal submitted, threshold approved, and executed. It no
longer requires five distinct multisig instances operated by parties outside the
submitting team.

This repository therefore provides two complementary evaluator surfaces:

- `consumer-demo/`: a clone-and-run integration app showing how a consumer uses
  the SDK for threshold-gated actions, resumable approvals, errors, and replay
  protection.
- `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json`: current v0.2.4 public deploy
  and execution evidence, including direct explorer links and canonical
  block/hash/index coordinates.
- `scripts/lez-sequencer-e2e.sh`: source-reproducible standalone-sequencer lane
  that generates a fresh proof with `RISC0_DEV_MODE=0`, deploys, executes, and
  verifies canonical inclusion.
- `submission/TESTNET_EVIDENCE.json`: pre-reset audit history only; it is not the
  current reviewer evidence.
