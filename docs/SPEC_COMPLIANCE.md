# LP-0002 Spec Compliance Matrix

## Functionality

| Requirement | Status | Evidence |
|---|---|---|
| Private approvals from shielded members | PASS | `core/`: MemberSecret derives nullifiers without exposing identity; `consumer-demo` scenario 1 |
| Threshold verified without recording approvers | PASS | `verify_threshold_receipt` checks count + nullifiers only; no member identities in journal |
| Double-vote prevention via nullifiers | PASS | `test_duplicate_approver_is_rejected_before_a_receipt_is_created`; `consumer-demo` scenario 4 |
| Completed execution unlinkable to member | PASS | `test_threshold_proof_verifies_without_revealing_member_commitments_or_secrets` |
| Proof generation runs client-side | PASS | Pure Rust, no server, runs on standard laptop |
| Reference integration on LEZ testnet | PASS | Evidence is split for auditability: (1) `consumer-demo/` demonstrates clone-and-run integration; (2) `methods/` + `host/` generate and verify real `RISC0_DEV_MODE=0` artifacts; (3) `lp0002-lez-execute-artifacts` runs the real receipt through the LEZ-shaped execution wrapper; (4) `submission/TESTNET_EVIDENCE.json` records the public LEZ testnet (https://testnet.lez.logos.co/), including tx `cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697` in block `39548`. |
| At least 1 reproducible multisig instance with proposal, threshold approval, and execution | PASS | Current public LP-0002 spec requires one reproducible instance rather than five external operators. `submission/TESTNET_EVIDENCE.json` records the instance/proposal/approval/execution evidence; `consumer-demo/` provides the clone-and-run integration surface. |
| Full documentation and clean repo | PASS | README, PROTOCOL.md, CONSUMER_INTEGRATION.md, SPEC_COMPLIANCE.md, inline docs |

## Usability

| Requirement | Status | Evidence |
|---|---|---|
| Module/SDK | PASS | `sdk/` crate with `MultisigSession` 5-step workflow + `prelude` module |
| Basecamp app GUI | PASS | `basecamp-app/` provides the native Qt/QML Basecamp plugin package (`CMakeLists.txt`, `metadata.json`, `IComponent` plugin source, QML UI). Local build/package evidence is in `submission/BASECAMP_NATIVE_BUILD.md`; real LogosBasecamp runtime launch/load evidence with raw-log SHA-256 binding is attached in `submission/BASECAMP_RUNTIME_LOAD_EVIDENCE.json`; the fresh narrated video shows the final visual walkthrough. |
| SPEL IDL | PASS | `interfaces/lp0002.idl.json` with typed and byte-oriented execute surfaces (`verify_and_execute`, `verify_and_execute_bytes`), RISC0 receipt-byte boundary, types, errors; discriminators computed |

## Reliability

| Requirement | Status | Evidence |
|---|---|---|
| Graceful error handling | PASS | Typed errors: `ProofError`, `VerifierError`, `SdkError` with clear messages |
| Partial approvals preserved and resumable | PASS | `ApprovalAccumulator` serializable via serde; `consumer-demo` scenario 3 |
| Deterministic documented error codes | PASS | 12 ProofError variants + 2 VerifierError variants + 5 SdkError variants; documented in `docs/PROTOCOL.md` section 8 |

## Performance

| Requirement | Status | Evidence |
|---|---|---|
| CU cost documented | PASS | Safe-lane benchmarks plus real RISC0 2-of-3 fixture measurements and serialized NSSA payload metrics in `submission/BENCHMARKS.md`; `submission/LEZ_COST_BENCHMARKS.json` records payload size/hash, account count, receipt size, block inclusion, and the explicit current limitation that LEZ v0.2.0-rc1 exposes block inclusion but not stable per-transaction CU counters. |

## Supportability

| Requirement | Status | Evidence |
|---|---|---|
| Program deployed on LEZ devnet/testnet | PASS | The reproducible `verify_and_execute_bytes` wrapper ELF (`cargo risczero build --manifest-path methods/guest/Cargo.toml`) was deployed to the public LEZ testnet (https://testnet.lez.logos.co/) via the submit binary's `ProgramDeployment` path with program id `974939edb6fc9cffd97929dd830a0d75bfc7a09b08c2f3fc87da940aadc0c130`; deploy tx `82516880f60c2076d78b28ad7b147ac0b05ed247b7bc33a27ac8f68b1d809c56` confirmed in block `39547`, and inclusion is summarized in `submission/TESTNET_EVIDENCE.json`. |
| E2E integration tests against sequencer | PASS | `scripts/lez-localnet-smoke.sh` validates real RISC0 artifacts and wrapper payload generation; public-testnet evidence confirms wrapper deployment plus NSSA transaction inclusion in block `39548`; `scripts/demo-heavy-lane.sh` is the recording-safe verifier. |
| CI/evaluator validation | PASS | `submission/CI_EVIDENCE.md` documents evaluator-run validation commands because the available publication token cannot push GitHub workflow files without `workflow` scope; local full validation has passed on the M4 Pro and fresh public-clone validators pass. |
| README with E2E usage | PASS | `README.md` with quick start, architecture, SDK usage |
| Reproducible demo script | PASS | `demo.sh` runs consumer-demo; `cargo test --workspace` |
| Narrated video demo | COMPLETE | Fresh human-recorded narrated walkthrough: https://youtu.be/Wssfp_rkC54 |

## Submission Requirements

| Requirement | Status | Evidence |
|---|---|---|
| Public repository, MIT/Apache-2.0 | PASS | MIT license, clean git history |
| Verifier deployed on LEZ testnet | PASS | Public-testnet wrapper deployment is confirmed and the public LEZ testnet is the LP-0002 deployment target; see `submission/TESTNET_EVIDENCE.json` |
| Narrated demo video | COMPLETE | Fresh human-recorded narrated walkthrough: https://youtu.be/Wssfp_rkC54 |
| Write-up (threshold proof, nullifier, LEZ compat, security) | PASS | `docs/PROTOCOL.md` (8 sections) |
| Proof generation time and cost benchmarks | PASS | `submission/BENCHMARKS.md` plus `submission/LEZ_COST_BENCHMARKS.json` record safe-lane timings, RISC0 fixture measurements, receipt/journal sizes, wrapper payload bytes, account count, public-testnet block inclusion, and the explicit CU-metering limitation of current LEZ tooling. |

## Honesty Notes

- The safe-lane `receipt_seal` remains a **deterministic mock** (SHA-256 binding), not a real RISC0 receipt
- The heavy-lane RISC0 guest and host prover exist in `methods/` and `host/` and have produced verified real receipt artifacts with `RISC0_DEV_MODE=0`
- The executable `verify_and_execute_bytes` wrapper is deployed and executed on the public LEZ testnet (https://testnet.lez.logos.co/): deploy tx `82516880f60c2076d78b28ad7b147ac0b05ed247b7bc33a27ac8f68b1d809c56` in block `39547`, execute tx `cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697` in block `39548`. Raw 270 KiB receipt transport exceeded the current LEZ/RISC0 public-program session limit, so the included wrapper input carries both the receipt SHA-256 and a receipt/journal commitment while full receipt verification remains host-side evidence.
- Native Qt/QML source and local build evidence are not the same as final Basecamp runtime load evidence. LP-0002 therefore keeps both evidence surfaces: local native build evidence in `submission/BASECAMP_NATIVE_BUILD.md` and raw-log-bound runtime/load evidence in `submission/BASECAMP_RUNTIME_LOAD_EVIDENCE.json`.
- The root `demo.sh` runs the consumer demo; use `scripts/demo-heavy-lane.sh` for RISC0/public-testnet evidence and `scripts/demo-video.sh` for the combined recording walkthrough
