# LP-0002 Spec Compliance Matrix
> Current LEZ refresh (2026-08-10): LEZ v0.2.4 (`47eba256…`) standalone deploy/execute/inclusion is verified locally with a fresh `RISC0_DEV_MODE=0` proof and wired into `.github/workflows/ci.yml`. Public testnet re-submission is also sequencer-RPC confirmed: deploy block `2373`, execute block `2374`. Both direct explorer routes now render after an observed indexing delay of about 59 minutes; see `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json`.

## Functionality

| Requirement | Status | Evidence |
|---|---|---|
| Private approvals from shielded members | PASS | `core/`: MemberSecret derives nullifiers without exposing identity; `consumer-demo` scenario 1 |
| Threshold verified without recording approvers | PASS | `verify_threshold_receipt` checks count + nullifiers only; no member identities in journal |
| Double-vote prevention via nullifiers | PASS | `test_duplicate_approver_is_rejected_before_a_receipt_is_created`; `consumer-demo` scenario 4 |
| Completed execution unlinkable to member | PASS | `test_threshold_proof_verifies_without_revealing_member_commitments_or_secrets` |
| Proof generation runs client-side | PASS | Pure Rust, no server, runs on standard laptop |
| Reference integration on LEZ testnet | PASS | Evidence is split for auditability: (1) `consumer-demo/` demonstrates clone-and-run integration; (2) `methods/` + `host/` generate and verify real `RISC0_DEV_MODE=0` artifacts; (3) `scripts/lez-sequencer-e2e.sh` proves the current standalone path; (4) `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json` records fresh public deploy tx `b15dc325…` in block `2373` and execute tx `4a985d6f…` in block `2374`. |
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
| CU cost documented | PASS | Safe-lane benchmarks plus real RISC0 2-of-3 fixture measurements and serialized transaction payload metrics in `submission/BENCHMARKS.md`; `submission/LEZ_COST_BENCHMARKS.json` records payload size/hash, account count, receipt size, block inclusion, and the explicit limitation that LEZ v0.2.4 exposes inclusion but not stable per-transaction CU counters. |

## Supportability

| Requirement | Status | Evidence |
|---|---|---|
| Program deployed on LEZ devnet/testnet | PASS | Current v0.2.4 ProgramBinary id `75efdee2…` is confirmed on public testnet via deploy tx [`b15dc325…`](https://explorer.testnet.lez.logos.co/transaction/b15dc325d3ec4332a3f925b27383d932ed7d7605df50f962640149a583a059c8) in block `2373`; execute tx [`4a985d6f…`](https://explorer.testnet.lez.logos.co/transaction/4a985d6f1e0d8384e2774e4c62f4e1c714f7542d4647e0b0e4cfd4f08ae0e242) is confirmed in block `2374`. Both direct explorer pages render. |
| E2E integration tests against sequencer | PASS | `scripts/lez-sequencer-e2e.sh` builds exact LEZ v0.2.4 standalone mode, generates a fresh non-dev RISC0 proof, deploys the generated ProgramBinary, submits execution, and queries canonical block/hash/index inclusion; `submission/LEZ_V024_LOCAL_E2E_EVIDENCE.json` records the latest run. |
| CI/evaluator validation | PASS | `.github/workflows/ci.yml` contains a dedicated `test-sequencer-e2e` job with `RISC0_DEV_MODE=0`; `scripts/test_ci_sequencer_contract.py` prevents version/workflow drift. See `submission/CI_EVIDENCE.md`. |
| README with E2E usage | PASS | `README.md` with quick start, architecture, SDK usage |
| Reproducible demo script | PASS | `demo.sh` runs consumer-demo; `cargo test --workspace` |
| Narrated video demo | COMPLETE | Fresh human-recorded narrated walkthrough: https://youtu.be/Wssfp_rkC54 |

## Submission Requirements

| Requirement | Status | Evidence |
|---|---|---|
| Public repository, MIT/Apache-2.0 | PASS | MIT license, clean git history |
| Verifier deployed on LEZ testnet | PASS | Fresh current-v0.2.4 public deploy/execute evidence is in `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json`; historical pre-reset evidence remains separately in `submission/TESTNET_EVIDENCE.json`. |
| Narrated demo video | COMPLETE | Fresh human-recorded narrated walkthrough: https://youtu.be/Wssfp_rkC54 |
| Write-up (threshold proof, nullifier, LEZ compat, security) | PASS | `docs/PROTOCOL.md` (8 sections) |
| Proof generation time and cost benchmarks | PASS | `submission/BENCHMARKS.md` plus `submission/LEZ_COST_BENCHMARKS.json` record safe-lane timings, RISC0 fixture measurements, receipt/journal sizes, wrapper payload bytes, account count, public-testnet block inclusion, and the explicit CU-metering limitation of current LEZ tooling. |

## Honesty Notes

- The safe-lane `receipt_seal` remains a **deterministic mock** (SHA-256 binding), not a real RISC0 receipt
- The heavy-lane RISC0 guest and host prover exist in `methods/` and `host/`; the current LEZ v0.2.4 standalone E2E generates and consumes a fresh real receipt with `RISC0_DEV_MODE=0`
- Fresh current-v0.2.4 public transactions are canonical by sequencer RPC in blocks `2373`/`2374`, and both direct explorer routes now render
- The executable `verify_and_execute_bytes` wrapper was deployed and executed on the public LEZ testnet before the June 2026 reset (https://testnet.lez.logos.co/): deploy tx `82516880f60c2076d78b28ad7b147ac0b05ed247b7bc33a27ac8f68b1d809c56` in block `39547`, execute tx `cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697` in block `39548`. Raw 270 KiB receipt transport exceeded the current LEZ/RISC0 public-program session limit, so the included wrapper input carries both the receipt SHA-256 and a receipt/journal commitment while full receipt verification remains host-side evidence.
- Native Qt/QML source and local build evidence are not the same as final Basecamp runtime load evidence. LP-0002 therefore keeps both evidence surfaces: local native build evidence in `submission/BASECAMP_NATIVE_BUILD.md` and raw-log-bound runtime/load evidence in `submission/BASECAMP_RUNTIME_LOAD_EVIDENCE.json`.
- The root `demo.sh` runs the consumer demo; use `scripts/demo-heavy-lane.sh` for RISC0/public-testnet evidence and `scripts/demo-video.sh` for the combined recording walkthrough
