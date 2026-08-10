# Draft λPrize PR: Solution LP-0002 — Private M-of-N Multisig
> Current LEZ refresh (2026-08-10): LEZ v0.2.4 standalone deploy/execute/inclusion now runs in CI with `RISC0_DEV_MODE=0`. Fresh public testnet deploy tx `b15dc325…` is confirmed in block `2373`; execute tx `4a985d6f…` is confirmed in block `2374`. Both direct explorer routes now render.

Do not open this PR until Evi explicitly approves opening the upstream Logos PR.
`scripts/final-publication-check.py` returns GO with current evidence and the human-recorded narrated demo URL.

## Repository

https://github.com/Tranquil-Flow/lp-0002-private-multisig

## Demo video

https://youtu.be/Wssfp_rkC54

## Public LEZ testnet evidence

Current v0.2.4 public re-submission:

- deploy: https://explorer.testnet.lez.logos.co/transaction/b15dc325d3ec4332a3f925b27383d932ed7d7605df50f962640149a583a059c8 — block `2373`
- execute: https://explorer.testnet.lez.logos.co/transaction/4a985d6f1e0d8384e2774e4c62f4e1c714f7542d4647e0b0e4cfd4f08ae0e242 — block `2374`, tx index `0`

Sequencer RPC confirms both and both direct explorer pages now render. Structured
current evidence is in `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json`, including:

- network interpretation
- verifier/wrapper program ID
- multisig instance identifier
- proposal transaction evidence
- shielded approval/nullifier evidence
- confirmed execution transaction
- block/confirmation references

Historical pre-reset evidence remains in `submission/TESTNET_EVIDENCE.json` only
as audit history.

## Notes for reviewers

The root `demo.sh` is the fast consumer demonstration. The heavy-lane
RISC0/public-testnet evidence is in `host/`, `scripts/lez-sequencer-e2e.sh`,
`submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json`, and `submission/BENCHMARKS.md`.

The submission is careful about the boundary: the full RISC0 receipt is verified
host-side and persisted as file-backed evidence; the included LEZ wrapper
transaction carries compact receipt/journal commitments because raw receipt bytes
exceed the current public-program session transport limit.
