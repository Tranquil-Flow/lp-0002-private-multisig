# Draft λPrize PR: Solution LP-0002 — Private M-of-N Multisig
> Current reset-era refresh (2026-06-28): deploy tx `c7157a473cb512bf7e1803d4377d9f65e9406a7ff98efeda48b65c0d4915a13b` is included on the public LEZ testnet for program id `1557176a639868b0363e9106c75fe0748ceb42e65f5f1a6778dd05b6baebb57d` (ProgramBinary SHA-256 `8f74ccc446990f5437b5f6c6e731deac6653992e0a64abcecdff7bff0c5575e1`). Execute attempts `352eb699507aea4d4ca6963a50bef1473a2b944dfd7713116cbf82eabfeec3bf` and `fc4165ac2437bd6533444c5e010b2d248aed678daadfad277af1dd0f1fef6ca8` locally validate under v0.2.0 but are not included by the public endpoint, so current live execute inclusion remains a transparent blocker and is not claimed. Historical pre-reset txs `82516880...` / `cb8bfd5...` are retained only as audit history.

Do not open this PR until Evi explicitly approves opening the upstream Logos PR.
`scripts/final-publication-check.py` should return GO after the fresh human-recorded narrated demo URL has been inserted.

## Repository

https://github.com/Tranquil-Flow/lp-0002-private-multisig

## Demo video

https://youtu.be/Wssfp_rkC54

## Public LEZ testnet evidence

LP-0002 has historical pre-reset public LEZ testnet deploy/execute evidence
(https://testnet.lez.logos.co/): deploy tx
`82516880f60c2076d78b28ad7b147ac0b05ed247b7bc33a27ac8f68b1d809c56` in block
`39547`, execute tx
`cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697` in block
`39548`. Structured evidence is attached in
`submission/TESTNET_EVIDENCE.json`, including:

- network interpretation
- verifier/wrapper program ID
- multisig instance identifier
- proposal transaction evidence
- shielded approval/nullifier evidence
- confirmed execution transaction
- block/confirmation references

Confirmed execution tx:

`cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697`

Included block: `39548`.

## Notes for reviewers

The root `demo.sh` is the fast consumer demonstration. The heavy-lane
RISC0/public-testnet evidence is in `host/`, `scripts/demo-heavy-lane.sh`,
`submission/TESTNET_EVIDENCE.json`, and `submission/BENCHMARKS.md`.

The submission is careful about the boundary: the full RISC0 receipt is verified
host-side and persisted as file-backed evidence; the included LEZ wrapper
transaction carries compact receipt/journal commitments because raw receipt bytes
exceed the current public-program session transport limit.
