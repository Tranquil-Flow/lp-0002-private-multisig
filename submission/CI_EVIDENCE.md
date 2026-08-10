# LP-0002 Validation Evidence

This repository contains both a GitHub Actions workflow and plain evaluator-run
commands. The dedicated `test-sequencer-e2e` job pins LEZ v0.2.4, sets
`RISC0_DEV_MODE=0`, builds the standalone sequencer and current LP-0002 guests,
then fails closed unless deploy, execute, and canonical block inclusion all pass.

Required validation commands:

```bash
cargo test --workspace
python3 scripts/validate-submission-readiness.py
LEZ_SOURCE_DIR=/path/to/logos-execution-zone-v0.2.4 scripts/lez-sequencer-e2e.sh
python3 scripts/final-publication-check.py
```

Latest verified local results:

- `RISC0_SKIP_BUILD=1 cargo test --workspace` passed on the M4 Pro.
- `scripts/lez-sequencer-e2e.sh` passed against LEZ v0.2.4 with a fresh
  `RISC0_DEV_MODE=0` proof, confirmed wrapper deployment, confirmed execution,
  and canonical transaction/block lookup. Exact IDs, file hashes, and block
  locations are recorded in `submission/LEZ_V024_LOCAL_E2E_EVIDENCE.json`.
- Public testnet recovered during verification. Fresh current-v0.2.4 deploy and
  execute transactions are sequencer-RPC confirmed in blocks `2373` and `2374`;
  `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json` records both direct explorer
  routes. Both pages now render; observed index delay was about 59 minutes.
- `python3 scripts/final-publication-check.py` passes with the human-recorded
  narrated demo (https://youtu.be/Wssfp_rkC54), current v0.2.4 public evidence,
  and visible explorer links. Historical `TESTNET_EVIDENCE.json` remains audit-only.
- `RISC0_SKIP_BUILD=1 python3 scripts/validate-submission-readiness.py --skip-exec` passed.
