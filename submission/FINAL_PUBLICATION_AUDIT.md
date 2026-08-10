# LP-0002 Final Publication Audit
> Repository evidence refresh (2026-08-10): exact LEZ v0.2.4 commit `47eba256...` is current. Deploy tx `b15dc325...` is confirmed in block `2373`; proof-backed execute tx `4a985d6f...` is confirmed in block `2374`, index `0`; both direct explorer pages render. `scripts/lez-sequencer-e2e.sh` independently proves the non-dev-mode standalone path. Earlier reset/pre-reset records remain audit history only.

Status: **SUBMISSION-READY AFTER FINAL VALIDATION. Fresh human-recorded narrated demo video is attached: https://youtu.be/Wssfp_rkC54. Do not open the upstream Logos PR without explicit approval.**

This file exists because the local implementation validator is not enough. The
LP-0005 rejection showed that local demos, static HTML, and partial localnet
substitutes can look polished while still missing hard public submission gates.
For LP-0002, `scripts/final-publication-check.py` remains the publication guard;
it now passes with the current v0.2.4 evidence and video URL.

## Current publication gates

| Gate | Current state | Required before submission |
|---|---|---|
| Public repository | PASS: public repo published at https://github.com/Tranquil-Flow/lp-0002-private-multisig | Keep public branch synchronized with this publication package |
| Narrated video | PASS: fresh human-recorded narrated demo attached | https://youtu.be/Wssfp_rkC54 |
| LEZ public testnet/devnet verifier | PASS: current v0.2.4 deploy/execute evidence and direct explorer links are recorded in `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json` | Keep evidence JSON synchronized if rerun |
| Current public testnet multisig instance | PASS: deploy tx `b15dc325...` is visible in block `2373`; execute tx `4a985d6f...` is visible in block `2374`, index `0` | Preserve current RPC and explorer evidence |
| Basecamp app | PASS for non-video readiness: native Qt/QML plugin package builds locally, runtime launch evidence is attached in `submission/BASECAMP_RUNTIME_LOAD_EVIDENCE.json`, and raw log hashes verify. Visual component activation is covered by the fresh narrated video. | Keep runtime evidence/log hashes synchronized; keep the fresh video URL in public submission docs |
| CI/evaluator validation | PASS: GitHub Actions contains a real standalone v0.2.4 sequencer E2E, and evaluator-run validation outputs are documented in `submission/CI_EVIDENCE.md` | Keep the exact source/toolchain pins and fail-closed contract test |
| CU/on-chain cost | Machine-readable payload/account/receipt cost evidence exists; current LEZ tooling exposes no stable CU counter | Keep `submission/LEZ_COST_BENCHMARKS.json`; replace `cu_metering.available=false` with real CU values if public devnet exposes them |

## What is already strong

- Real `RISC0_DEV_MODE=0` proof artifacts have been generated and verified.
- The current `verify_and_execute_bytes` ProgramBinary was deployed and executed
  on public LEZ v0.2.4: deploy tx `b15dc325...` in block `2373`, execute tx
  `4a985d6f...` in block `2374`, index `0`. Earlier reset-era hashes remain
  explicitly historical.
- SDK, consumer demo, SPEL IDL, protocol docs, native/QML Basecamp package, Basecamp runtime launch evidence, cost evidence, and local validators are substantial.
- Claims now distinguish safe-lane mock receipt, host-side receipt verification,
  compact wrapper commitment, and confirmed public-testnet inclusion evidence.

## Required command before any submission PR

```bash
python3 scripts/final-publication-check.py
```

A failing result means **do not submit**.
