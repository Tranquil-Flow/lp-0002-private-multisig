# Benchmarks
> Repository evidence refresh (2026-08-10): exact LEZ v0.2.4 commit `47eba256...` is current. Deploy tx `b15dc325...` is confirmed in block `2373`; proof-backed execute tx `4a985d6f...` is confirmed in block `2374`, index `0`; both direct explorer pages render. `scripts/lez-sequencer-e2e.sh` independently proves the non-dev-mode standalone path. Earlier reset/pre-reset records remain audit history only.

LP-0002 Private Multisig — safe-lane benchmarks, with RISC0 heavy-lane and LEZ payload measurements below.

All measurements taken on the reference container, release build (`--release`),
1000 iterations averaged per operation. Uses SHA-256 mock receipt (no RISC0
heavy-lane proving). Run the safe-lane table via `cargo run --example bench --release` from the
workspace root. Heavy-lane artifact commands are listed below.

## Operation Timings

| Config    | Config::new | prove_threshold | verify_receipt | execute_if_met | proof_bytes |
|-----------|-------------|-----------------|----------------|----------------|-------------|
| 2-of-3    | 1.4 μs      | 4.8 μs          | 29 ns          | 75 ns          | 306 B       |
| 3-of-5    | 2.1 μs      | 3.4 μs          | 53 ns          | 92 ns          | 338 B       |
| 5-of-10   | 3.4 μs      | 5.0 μs          | 84 ns          | 125 ns         | 402 B       |
| 10-of-20  | 6.6 μs      | 8.6 μs          | 173 ns         | 216 ns         | 562 B       |
| 25-of-50  | 15.6 μs     | 20.0 μs         | 461 ns         | 527 ns         | 1042 B      |

### Key Observations

- **Config::new** scales roughly linearly with member count (O(n) for commitment
  derivation + sort).
- **prove_threshold** scales with the number of _approving_ members (threshold),
  not total members. The 25-of-50 case (25 approvals) takes ~20 μs.
- **verify_threshold_receipt** and **execute_if_threshold_met** are O(n) in
  nullifier count (checking for duplicates via BTreeSet).
- **execute_if_threshold_met** includes a full verify pass + duplicate-execution
  guard (BTreeSet insert), so it is the most complete consumer-side operation.

## Proof Sizes

Proof sizes are measured via `proof.public_bytes()` which uses Borsh
serialization of the full `ThresholdProof` struct, including the
`ThresholdJournal` (with all nullifiers) and the 32-byte receipt seal.

Proof size grows with the number of approving members because nullifiers
(32 bytes each) are included in the journal:

| Config   | Approvals | Proof Size |
|----------|-----------|------------|
| 2-of-3   | 2         | 306 B      |
| 3-of-5   | 3         | 338 B      |
| 5-of-10  | 5         | 402 B      |
| 10-of-20 | 10        | 562 B      |
| 25-of-50 | 25        | 1042 B     |

### Proof Size Breakdown

Fixed overhead (headers, IDs, hashes, counts): ~242 bytes.
Variable component: 32 bytes per nullifier (32 × approval_count).

Total formula: `242 + 32 × approval_count` bytes.

For a 25-of-50 multisig, that is ~1 KB — well within acceptable on-chain
payload limits.

## Historical RISC0 Heavy-Lane Measurements (superseded 2026-08-10)

The safe-lane table above still uses a **mock SHA-256 receipt seal**. The heavy
lane had real RISC0 artifacts generated with `RISC0_DEV_MODE=0` on the M4 Pro.
These identifiers document the pre-v0.2.4 run; current identifiers appear below.

Measured fixture run (`target/lp0002-risc0-fixture-new/manifest.txt`):

| Metric | Value |
|---|---:|
| RISC0 image id | `6fc85ce06da1762abec319b4626c12229dc605a5b0283d64c8eab2567b9ee721` |
| Proof id | `9e6492e73d1e8382abfa0e94e91842100b9041516857f215fcad7276cbad8b11` |
| Threshold fixture | 2-of-3 |
| Approval count | 2 |
| Prove + verify queue job duration | 71 s |
| Initial Rust/RISC0 host build time | 36.13 s |
| Receipt artifact size | 264 KiB |
| Journal artifact size | 1.1 KiB |
| `risc0_dev_mode` | `0` |

Privacy note: member secrets, selected approver commitments, and the full
membership set remain private witness data; the public journal exposes only the
threshold relation outputs needed by the verifier.


## Historical Public-Testnet Deployment Evidence (pre-reset)

The reproducible `verify_and_execute_bytes` wrapper ELF (`cargo risczero build --manifest-path methods/guest/Cargo.toml`, Docker builder `risczero/risc0-guest-builder:r0.1.88.0`; on LEZ ProgramId == ImageID) was deployed to the public LEZ testnet (https://testnet.lez.logos.co/) via the submit binary's `ProgramDeployment` path.

Deployment surface captured on M4 Pro:

```json
{"program":"verify_and_execute_bytes","program_id":"974939edb6fc9cffd97929dd830a0d75bfc7a09b08c2f3fc87da940aadc0c130","deploy_tx":"82516880f60c2076d78b28ad7b147ac0b05ed247b7bc33a27ac8f68b1d809c56","included_block_id":39547,"status":"confirmed"}
```

This records the historical pre-reset `ProgramDeployment` (deploy tx
`82516880...` in block `39547`). Current v0.2.4 deploy/execute evidence appears
below. The only unavailable metric is a formal per-transaction CU/cycle counter,
because the target network surface used here does not expose one.


## Historical SPEL/NSSA Adapter Payload Evidence

The historical heavy-lane wrapper has a concrete byte-oriented SPEL instruction surface,
`verify_and_execute_bytes`, plus a host evidence builder. Raw receipt transport
was attempted first and exceeded the current public-program session limit, so the
confirmed path sends a compact receipt/journal commitment and retains the full
receipt as file-backed evidence.

Measured artifact: `target/lp0002-risc0-fixture-new/spel-adapter-evidence.json`.

| Metric | Value |
|---|---:|
| Instruction | `verify_and_execute_bytes` |
| Instruction index | 5 |
| Receipt bytes retained off-input | 270,334 B |
| Public journal bytes | 1,100 B |
| Action Borsh bytes | 110 B |
| Serialized instruction words | 1,373 u32 words |
| Serialized instruction data length | 5,492 B |
| Instruction data SHA-256 | `e1dc304173c1f27542b0017e167eb709f47e6bc907888968e9efaf0cd655f3c0` |
| Receipt SHA-256 | `6e4979983c996ca4154d7eeedb59444105b99d984a69a223ab58d429811b89a7` |
| Journal SHA-256 | `a8fe85f8d63f948409941b585cbe9244c2d0ae45082bf635173f753037ad4d8e` |
| Receipt/journal commitment | `be58410de0e0f71642f82f287c39c7f70acb8820cb7468e50927bfd91ee4c850` |

## Current LEE Submitter Evidence

Measured current evidence: `submission/LEZ_V024_LOCAL_E2E_EVIDENCE.json` and `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json`.

The native file-backed submitter constructs a current `LeeTransaction::Public` through the public sequencer RPC client, verifies the real receipt host-side, and sends the compact commitment-bound wrapper payload:

| Metric | Value |
|---|---:|
| Instruction | `verify_and_execute_bytes` |
| Instruction payload | 5,492 B / 1,373 u32 words |
| Instruction data SHA-256 | `19944c558baa9cfc327fb5d4c078d6097b85cbf929943ec4eec9a82a28bc5170` |
| Receipt SHA-256 | `212970331ca67e2557899b13f77fcf43855db2362b123c0c2dc3b88797bfd22d` |
| Receipt/journal commitment | `52948db7bc8b75cd54d7825afe0d3e64f8426cb94879c00852903b581a1c623f` |
| Confirmed public-testnet tx hash | `4a985d6f1e0d8384e2774e4c62f4e1c714f7542d4647e0b0e4cfd4f08ae0e242` |
| Inclusion status | Confirmed in block `2374`, transaction index `0` |

LEZ v0.2.4 JSON-RPC exposes transaction/block inclusion but not per-transaction compute-unit counters. Successful public-testnet inclusion is captured for the compact wrapper path; the missing formal CU counter is recorded as an explicit target-runtime limitation rather than estimated.

## LEZ Compute-Unit Costs

LEZ (Logos Execution Zone) per-transaction compute-unit pricing/counters are not
currently exposed by the v0.2.4 JSON-RPC surface used here. The compact wrapper
path therefore records block inclusion, payload size/hash, and receipt/journal
commitment. If a later public devnet/testnet exposes a stable CU/cycle counter,
replace the machine-readable
`cu_metering.available=false` note in `submission/LEZ_COST_BENCHMARKS.json` with
the chain-native value.

## Reproducing

```bash
cd lp-0002-private-multisig
cargo run --example bench --release
RISC0_DEV_MODE=0 cargo run -p lp0002-private-multisig-host --bin lp0002-prove-fixture -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-verify-artifacts -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-lez-execute-artifacts -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-spel-adapter-evidence -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-submit-localnet -- target/lp0002-risc0-fixture
bash scripts/demo-heavy-lane.sh
LEZ_SOURCE_DIR=/path/to/logos-execution-zone-v0.2.4 scripts/lez-sequencer-e2e.sh
```

The benchmark source is at `consumer-demo/examples/bench.rs`.

## RISC0-to-LEZ wrapper evidence

The heavy-lane host verifies the real RISC0 receipt, constructs the current LEE payload, and deploys the generated ProgramBinary. `scripts/lez-sequencer-e2e.sh` starts exact LEZ v0.2.4 standalone mode and fails unless deployment, execution, and canonical inclusion all succeed. The same final artifacts are confirmed on public testnet in blocks `2373`/`2374`. CU metering is recorded as unavailable in LEZ tooling.

## Wrapper Public-Testnet Inclusion Evidence

The final `verify_and_execute_bytes` ProgramBinary was deployed and executed on current public LEZ testnet with a fresh `RISC0_DEV_MODE=0` proof. The compact path sends a receipt/journal commitment in wrapper input and retains the full receipt as file-backed evidence.

| Metric | Value |
|---|---:|
| Wrapper program id | `75efdee2321a1e15055bd5d03c1ea1a5665bf3e633683c9fb044a9e1f5941f79` |
| Threshold proof image id | `e57b90a0fd306a8a6258f4fe7c045d056cf70142408e31a020817aaaba23bac4` |
| Deploy tx hash | `b15dc325d3ec4332a3f925b27383d932ed7d7605df50f962640149a583a059c8` (block `2373`) |
| Confirmed execute tx hash | `4a985d6f1e0d8384e2774e4c62f4e1c714f7542d4647e0b0e4cfd4f08ae0e242` |
| Included block id | `2374` |
| Included tx index | `0` |
| Instruction payload | `5,492 bytes` / `1,373 u32 words` |
| Instruction SHA-256 | `19944c558baa9cfc327fb5d4c078d6097b85cbf929943ec4eec9a82a28bc5170` |
| Receipt bytes retained off-input | `270,334 bytes`, SHA-256 `212970331ca67e2557899b13f77fcf43855db2362b123c0c2dc3b88797bfd22d` |

Current on-chain evidence is recorded in `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json`. Sequencer RPC confirms both transactions and both direct explorer pages now render. LEZ v0.2.4 does not expose per-transaction CU counters over JSON-RPC, so this is inclusion + payload evidence rather than a formal CU meter.


## Machine-readable LEZ Cost Evidence

`submission/LEZ_COST_BENCHMARKS.json` is generated from current public v0.2.4
evidence by `python3 scripts/benchmark-lez-costs.py`; `--check` fails on drift. It
records account count, instruction payload bytes, RISC0 receipt size, journal
hash, receipt/journal commitment, public-testnet inclusion block/tx, and an
explicit `cu_metering.available=false` marker because current LEZ JSON-RPC does
not expose a stable per-transaction compute-unit counter.
