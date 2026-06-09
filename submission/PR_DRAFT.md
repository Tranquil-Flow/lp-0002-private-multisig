# Draft λPrize PR: Solution LP-0002 — Private M-of-N Multisig

Do not open this PR until Evi explicitly approves opening the upstream Logos PR.
`scripts/final-publication-check.py` should return GO after the fresh human-recorded narrated demo URL has been inserted.

## Repository

https://github.com/Tranquil-Flow/lp-0002-private-multisig

## Demo video

https://youtu.be/Wssfp_rkC54

## Public LEZ testnet evidence

LP-0002 is deployed and executed on the public LEZ testnet
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
