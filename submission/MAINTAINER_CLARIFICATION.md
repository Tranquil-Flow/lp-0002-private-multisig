# LP-0002 Adoption-Criteria Update

The current public LP-0002 prize text requires at least one reproducible multisig
instance with a proposal submitted, threshold approved, and executed. It no
longer requires five distinct multisig instances operated by parties outside the
submitting team.

This repository therefore provides two complementary evaluator surfaces:

- `consumer-demo/`: a clone-and-run integration app showing how a consumer uses
  the SDK for threshold-gated actions, resumable approvals, errors, and replay
  protection.
- `submission/TESTNET_EVIDENCE.json`: structured evidence for the recorded
  localnet/evaluator multisig instance, proposal, approvals/nullifiers, wrapper
  execution transaction, and block inclusion.
