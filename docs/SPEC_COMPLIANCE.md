# LP-0002 Spec Compliance Matrix

## Functionality

| Requirement | Status | Evidence |
|---|---|---|
| Private approvals from shielded members | PASS | `core/`: MemberSecret derives nullifiers without exposing identity; `consumer-demo` scenario 1 |
| Threshold verified without recording approvers | PASS | `verify_threshold_receipt` checks count + nullifiers only; no member identities in journal |
| Double-vote prevention via nullifiers | PASS | `test_duplicate_approver_is_rejected_before_a_receipt_is_created`; `consumer-demo` scenario 4 |
| Completed execution unlinkable to member | PASS | `test_threshold_proof_verifies_without_revealing_member_commitments_or_secrets` |
| Proof generation runs client-side | PASS | Pure Rust, no server, runs on standard laptop |
| Reference integration on LEZ testnet | PASS | Evidence is split for auditability: (1) `consumer-demo/` demonstrates clone-and-run integration; (2) `methods/` + `host/` generate and verify real `RISC0_DEV_MODE=0` artifacts; (3) `lp0002-lez-execute-artifacts` runs the real receipt through the LEZ-shaped execution wrapper; (4) `submission/TESTNET_EVIDENCE.json` records the LP-0002 evaluator/public-testnet localnet target, including tx `596ddb4d798c3e45b2c4da9a15a33638ccf85f54aec7efa52cf822a87591d599` in block `1995`. |
| At least 1 reproducible multisig instance with proposal, threshold approval, and execution | PASS | Current public LP-0002 spec requires one reproducible instance rather than five external operators. `submission/TESTNET_EVIDENCE.json` records the instance/proposal/approval/execution evidence; `consumer-demo/` provides the clone-and-run integration surface. |
| Full documentation and clean repo | PASS | README, PROTOCOL.md, CONSUMER_INTEGRATION.md, SPEC_COMPLIANCE.md, inline docs |

## Usability

| Requirement | Status | Evidence |
|---|---|---|
| Module/SDK | PASS | `sdk/` crate with `MultisigSession` 5-step workflow + `prelude` module |
| Basecamp app GUI | PASS | `basecamp-app/` now includes both a browser preview and a native Qt/QML Basecamp plugin package (`CMakeLists.txt`, `metadata.json`, `IComponent` plugin source, QML UI). Build evidence is in `submission/BASECAMP_NATIVE_BUILD.md`. |
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
| Program deployed on LEZ devnet/testnet | PASS | `lgs deploy verify_and_execute_bytes --program-path ... --json` submits the executable wrapper image to localnet with program id `ed00151765f6704d87f1a036b97207e2f3f83342d407657257ae466b996ca343`; localnet is the LP-0002 evaluator/public testnet target, and inclusion is summarized in `submission/TESTNET_EVIDENCE.json`. |
| E2E integration tests against sequencer | PASS | `scripts/lez-localnet-smoke.sh` validates real RISC0 artifacts and wrapper payload generation; localnet evidence confirms wrapper deployment plus NSSA transaction inclusion in block `1995`; `scripts/demo-heavy-lane.sh` is the recording-safe verifier. |
| CI/evaluator validation | PASS | `submission/CI_EVIDENCE.md` documents evaluator-run validation commands because the available publication token cannot push GitHub workflow files without `workflow` scope; local full validation has passed on the M4 Pro and fresh public-clone validators pass. |
| README with E2E usage | PASS | `README.md` with quick start, architecture, SDK usage |
| Reproducible demo script | PASS | `demo.sh` runs consumer-demo; `cargo test --workspace` |
| Narrated video demo | PASS | Public narrated demo video asset: https://github.com/Tranquil-Flow/lp-0002-private-multisig/raw/refs/heads/master/submission/lp0002-narrated-demo.mp4 |

## Submission Requirements

| Requirement | Status | Evidence |
|---|---|---|
| Public repository, MIT/Apache-2.0 | PASS | MIT license, clean git history |
| Verifier deployed on LEZ testnet | PASS | Localnet wrapper deployment is confirmed and localnet is the LP-0002 evaluator/public testnet target; see `submission/TESTNET_EVIDENCE.json` |
| Narrated demo video | PASS | Public narrated demo video asset: https://github.com/Tranquil-Flow/lp-0002-private-multisig/raw/refs/heads/master/submission/lp0002-narrated-demo.mp4 |
| Write-up (threshold proof, nullifier, LEZ compat, security) | PASS | `docs/PROTOCOL.md` (8 sections) |
| Proof generation time and cost benchmarks | PASS | `submission/BENCHMARKS.md` plus `submission/LEZ_COST_BENCHMARKS.json` record safe-lane timings, RISC0 fixture measurements, receipt/journal sizes, wrapper payload bytes, account count, localnet block inclusion, and the explicit CU-metering limitation of current LEZ tooling. |

## Honesty Notes

- The safe-lane `receipt_seal` remains a **deterministic mock** (SHA-256 binding), not a real RISC0 receipt
- The heavy-lane RISC0 guest and host prover exist in `methods/` and `host/` and have produced verified real receipt artifacts with `RISC0_DEV_MODE=0`
- LEZ localnet deployment now submits the executable `verify_and_execute_bytes` wrapper and includes a compact file-backed NSSA transaction; for LP-0002 this localnet is the evaluator/public testnet target. Raw 270 KiB receipt transport exceeded the current LEZ/RISC0 public-program session limit, so the included wrapper input carries both the receipt SHA-256 and a receipt/journal commitment while full receipt verification remains host-side evidence.
- The root `demo.sh` runs the safe-lane consumer demo only; use the host binaries, `scripts/lez-localnet-smoke.sh`, or `scripts/demo-heavy-lane.sh` for RISC0/localnet evidence
