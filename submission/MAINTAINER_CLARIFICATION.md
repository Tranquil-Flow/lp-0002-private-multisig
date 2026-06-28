# LP-0002 Adoption-Criteria Update
> Current reset-era refresh (2026-06-28): deploy tx `c7157a473cb512bf7e1803d4377d9f65e9406a7ff98efeda48b65c0d4915a13b` is included on the public LEZ testnet for program id `1557176a639868b0363e9106c75fe0748ceb42e65f5f1a6778dd05b6baebb57d` (ProgramBinary SHA-256 `8f74ccc446990f5437b5f6c6e731deac6653992e0a64abcecdff7bff0c5575e1`). Execute attempts `352eb699507aea4d4ca6963a50bef1473a2b944dfd7713116cbf82eabfeec3bf` and `fc4165ac2437bd6533444c5e010b2d248aed678daadfad277af1dd0f1fef6ca8` locally validate under v0.2.0 but are not included by the public endpoint, so current live execute inclusion remains a transparent blocker and is not claimed. Historical pre-reset txs `82516880...` / `cb8bfd5...` are retained only as audit history.

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
  deployed and executed on the public LEZ testnet before the June 2026 reset (https://testnet.lez.logos.co/) —
  deploy tx `82516880...` in block `39547`, execute tx `cb8bfd5a...` in block
  `39548`, re-verifiable via `getTransaction` against the public sequencer.
