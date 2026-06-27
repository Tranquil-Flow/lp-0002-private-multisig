# LP-0002 Final Publication Audit

Status: **SUBMISSION-READY AFTER FINAL VALIDATION. Fresh human-recorded narrated demo video is attached: https://youtu.be/Wssfp_rkC54. Do not open the upstream Logos PR without explicit approval.**

This file exists because the local implementation validator is not enough. The
LP-0005 rejection showed that local demos, static HTML, and partial localnet
substitutes can look polished while still missing hard public submission gates.
For LP-0002, do not submit until `scripts/final-publication-check.py` passes after this video URL insertion.

## Current publication gates

| Gate | Current state | Required before submission |
|---|---|---|
| Public repository | PASS: public repo published at https://github.com/Tranquil-Flow/lp-0002-private-multisig | Keep public branch synchronized with this publication package |
| Narrated video | PASS: fresh human-recorded narrated demo attached | https://youtu.be/Wssfp_rkC54 |
| LEZ public testnet/devnet verifier | PASS: the public LEZ testnet is the deployment target; wrapper tx evidence is recorded in `submission/TESTNET_EVIDENCE.json` | Keep evidence JSON synchronized if rerun |
| Current public testnet multisig instance | BLOCKED: pre-reset deploy tx `82516880...` and execute tx `cb8bfd5a...` now re-query as null on the current public testnet | Redeploy/re-execute before claiming current-live evidence |
| Basecamp app | PASS for non-video readiness: native Qt/QML plugin package builds locally, runtime launch evidence is attached in `submission/BASECAMP_RUNTIME_LOAD_EVIDENCE.json`, and raw log hashes verify. Visual component activation is covered by the fresh narrated video. | Keep runtime evidence/log hashes synchronized; keep the fresh video URL in public submission docs |
| CI/evaluator validation | PASS: evaluator-run validation commands and outputs are documented in `submission/CI_EVIDENCE.md` | If a workflow-scope token is later available, add GitHub Actions without changing the evidence semantics |
| CU/on-chain cost | Machine-readable payload/account/receipt cost evidence exists; current LEZ tooling exposes no stable CU counter | Keep `submission/LEZ_COST_BENCHMARKS.json`; replace `cu_metering.available=false` with real CU values if public devnet exposes them |

## What is already strong

- Real `RISC0_DEV_MODE=0` proof artifacts have been generated and verified.
- The `verify_and_execute_bytes` wrapper was deployed and executed on the public LEZ testnet before the June 2026 reset: deploy tx `82516880...` confirmed in block `39547`, execute tx `cb8bfd5a...` confirmed in block `39548`. Those hashes now re-query as null on the current endpoint, so this is historical evidence only until redeploy/re-execute refreshes current-live proof.
- SDK, consumer demo, SPEL IDL, protocol docs, native/QML Basecamp package, Basecamp runtime launch evidence, cost evidence, and local validators are substantial.
- Claims now distinguish safe-lane mock receipt, host-side receipt verification,
  compact wrapper commitment, and confirmed public-testnet inclusion evidence.

## Required command before any submission PR

```bash
python3 scripts/final-publication-check.py
```

A failing result means **do not submit**.
