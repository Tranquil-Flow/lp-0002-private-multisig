# Draft λPrize PR: Solution LP-0002 — Private M-of-N Multisig

Do not open this PR until Evi explicitly approves opening the upstream Logos PR.
`scripts/final-publication-check.py` currently passes for the repository-side package.

## Repository

https://github.com/Tranquil-Flow/lp-0002-private-multisig

## Demo video

HUMAN_RECORDED_DEMO_REQUIRED

## Public LEZ testnet/evaluator evidence

The LP-0002 evaluator/public-testnet target is the lgs/NSSA LEZ localnet, per
maintainer/user clarification. Structured evidence is attached in
`submission/TESTNET_EVIDENCE.json`, including:

- network interpretation
- verifier/wrapper program ID
- multisig instance identifier
- proposal transaction evidence
- shielded approval/nullifier evidence
- confirmed execution transaction
- block/confirmation references

Confirmed execution tx:

`596ddb4d798c3e45b2c4da9a15a33638ccf85f54aec7efa52cf822a87591d599`

Included block: `1995`.

## Notes for reviewers

The root `demo.sh` is the fast safe-lane consumer demonstration. The heavy-lane
RISC0/localnet evidence is in `host/`, `scripts/demo-heavy-lane.sh`,
`submission/TESTNET_EVIDENCE.json`, and `submission/BENCHMARKS.md`.

The submission is careful about the boundary: the full RISC0 receipt is verified
host-side and persisted as file-backed evidence; the included LEZ wrapper
transaction carries compact receipt/journal commitments because raw receipt bytes
exceed the current public-program session transport limit.
