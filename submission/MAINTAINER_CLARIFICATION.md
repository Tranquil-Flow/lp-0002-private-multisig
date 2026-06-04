# LP-0002 Adoption-Criteria Update

The current public LP-0002 prize text requires at least one reproducible multisig
instance with a proposal submitted, threshold approved, and executed. It no
longer requires five distinct multisig instances operated by parties outside the
submitting team.

This repository therefore provides two complementary evaluator surfaces:

- `consumer-demo/`: a clone-and-run integration app showing how a consumer uses
  the SDK for threshold-gated actions, resumable approvals, errors, and replay
  protection.
- `submission/TESTNET_EVIDENCE.json`: structured evidence for the multisig
  instance, proposal, approvals/nullifiers, and wrapper execution transaction
  deployed and executed on the public LEZ testnet (https://testnet.lez.logos.co/) —
  deploy tx `82516880...` in block `39547`, execute tx `cb8bfd5a...` in block
  `39548`, re-verifiable via `getTransaction` against the public sequencer.
