# LP-0002 Final Publication Audit

Status: **NOT YET SUBMISSION-READY. `scripts/final-publication-check.py` returns NO-GO pending a fresh human-recorded narrated demo video (the prior video is historical only and must not be used as final evidence). Do not open the upstream Logos PR without explicit approval.**

This file exists because the local implementation validator is not enough. The
LP-0005 rejection showed that local demos, static HTML, and partial localnet
substitutes can look polished while still missing hard public submission gates.
For LP-0002, do not submit until `scripts/final-publication-check.py` passes.

## Current hard blockers

| Gate | Current state | Required before submission |
|---|---|---|
| Public repository | PASS: public repo published at https://github.com/Tranquil-Flow/lp-0002-private-multisig | Keep public branch synchronized with this publication package |
| Narrated video | PENDING: fresh human-recorded narrated demo not yet recorded; the prior video is historical only and must not be used as final evidence | Record builder-narrated architecture/decisions/M-of-N approval/execution with terminal output proving RISC0_DEV_MODE=0, then insert the URL |
| LEZ public testnet/devnet verifier | PASS: the public LEZ testnet is the deployment target; wrapper tx evidence is recorded in `submission/TESTNET_EVIDENCE.json` | Keep evidence JSON synchronized if rerun |
| Public testnet multisig instance | PASS: deployed and executed on the public LEZ testnet (https://testnet.lez.logos.co/) — deploy tx `82516880...` block `39547`, execute tx `cb8bfd5a...` block `39548` | Keep transaction hashes and block ids in final write-up |
| Basecamp app | PASS for non-video readiness: native Qt/QML plugin package builds locally, runtime launch evidence is attached in `submission/BASECAMP_RUNTIME_LOAD_EVIDENCE.json`, and raw log hashes verify. Visual component activation remains part of the fresh video step. | Keep runtime evidence/log hashes synchronized; record the fresh video before final submission |
| CI/evaluator validation | PASS: evaluator-run validation commands and outputs are documented in `submission/CI_EVIDENCE.md` | If a workflow-scope token is later available, add GitHub Actions without changing the evidence semantics |
| CU/on-chain cost | Machine-readable payload/account/receipt cost evidence exists; current LEZ tooling exposes no stable CU counter | Keep `submission/LEZ_COST_BENCHMARKS.json`; replace `cu_metering.available=false` with real CU values if public devnet exposes them |

## What is already strong

- Real `RISC0_DEV_MODE=0` proof artifacts have been generated and verified.
- The `verify_and_execute_bytes` wrapper is deployed and executed on the public LEZ testnet (https://testnet.lez.logos.co/): deploy tx `82516880...` confirmed in block `39547`, execute tx `cb8bfd5a...` confirmed in block `39548`. On LEZ a transaction is included only if its program execution succeeds, so the confirmed execute tx is a successful on-chain run.
- SDK, consumer demo, SPEL IDL, protocol docs, native/QML Basecamp package, Basecamp runtime launch evidence, cost evidence, and local validators are substantial.
- Claims now distinguish safe-lane mock receipt, host-side receipt verification,
  compact wrapper commitment, and confirmed public-testnet inclusion evidence.

## Required command before any submission PR

```bash
python3 scripts/final-publication-check.py
```

A failing result means **do not submit**.
