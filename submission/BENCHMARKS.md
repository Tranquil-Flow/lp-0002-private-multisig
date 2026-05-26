# Benchmarks

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

## RISC0 Heavy-Lane Measurements

The safe-lane table above still uses a **mock SHA-256 receipt seal**. The heavy
lane now has real RISC0 artifacts generated with `RISC0_DEV_MODE=0` on the M4 Pro.

Measured fixture run (`target/lp0002-risc0-fixture-new/manifest.txt`):

| Metric | Value |
|---|---:|
| RISC0 image id | `026e95199ae495d946f7632d721823def2756584332c771a64207114311d4f01` |
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


## Localnet Deployment Evidence

`lgs doctor` is green against the migrated lgs 0.2.0 scaffold (`22 PASS, 0 WARN, 0 FAIL`) with the local sequencer reachable on `127.0.0.1:3040`.

Deployment surface captured on M4 Pro:

```json
{"program":"verify_and_execute_bytes","program_id":"ed00151765f6704d87f1a036b97207e2f3f83342d407657257ae466b996ca343","status":"submitted"}
```

This proves the scaffold/localnet deployment lane is usable for the executable wrapper image. The wrapper section below records confirmed compact NSSA transaction inclusion. The remaining benchmark gap is only a formal per-transaction CU/cycle counter if exposed by the target network.


## SPEL/NSSA Adapter Payload Evidence

The heavy-lane wrapper now has a concrete byte-oriented SPEL instruction surface,
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
| Instruction data SHA-256 | `4a04669d3d183d659353f72a7fa0ca7adc61d41ca07b8d7de2642f861d96a677` |
| Receipt SHA-256 | `8142fe9e92d144541d579521940ee873f09d15fb60aad4eb45f3c369fe3177ff` |
| Journal SHA-256 | `a8fe85f8d63f948409941b585cbe9244c2d0ae45082bf635173f753037ad4d8e` |
| Receipt/journal commitment | `68141a959293adaaebffb41be3969ecccf30e43947e0008ed10726b8e03444e7` |

## NSSA Submitter Evidence

Measured artifacts: `target/lp0002-risc0-fixture-new/nssa-submit-dry-run.json` and `target/lp0002-risc0-fixture-new/nssa-submit-evidence.json`.

The native file-backed submitter constructs a public `NSSATransaction::Public` directly from the real receipt, journal, and Borsh action files, avoids shell/argv limits, verifies the real receipt host-side before submission, and sends the compact commitment-bound wrapper payload. Dry-run metrics match the adapter payload exactly:

| Metric | Value |
|---|---:|
| Instruction | `verify_and_execute_bytes` |
| Instruction payload | 5,492 B / 1,373 u32 words |
| Instruction data SHA-256 | `4a04669d3d183d659353f72a7fa0ca7adc61d41ca07b8d7de2642f861d96a677` |
| Receipt SHA-256 | `8142fe9e92d144541d579521940ee873f09d15fb60aad4eb45f3c369fe3177ff` |
| Receipt/journal commitment | `68141a959293adaaebffb41be3969ecccf30e43947e0008ed10726b8e03444e7` |
| Confirmed localnet tx hash | `596ddb4d798c3e45b2c4da9a15a33638ccf85f54aec7efa52cf822a87591d599` |
| Inclusion status | Confirmed in block `1995`, transaction index `0` |

LEZ v0.2.0-rc1 JSON-RPC exposes transaction/block inclusion but not per-transaction compute-unit counters. Successful localnet inclusion is captured for the compact wrapper path; the missing formal CU counter is recorded as an explicit target-runtime limitation rather than estimated.

## LEZ Compute-Unit Costs

LEZ (Logos Execution Zone) per-transaction compute-unit pricing/counters are not
currently exposed by the v0.2.0-rc1 JSON-RPC surface used here. The compact wrapper
path therefore records block inclusion, payload size/hash, receipt/journal commitment, and the
sequencer RISC0 execution-time log line (`11.122875ms`). If a later public
devnet/testnet exposes a stable CU/cycle counter, replace the machine-readable
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
```

The benchmark source is at `consumer-demo/examples/bench.rs`.

## RISC0-to-LEZ wrapper evidence

The heavy-lane host includes `lp0002-lez-execute-artifacts`, which verifies the real RISC0 receipt with `host::Risc0ReceiptVerifier`, executes the resulting journal through `lez-program::execute_proposal`, and writes `target/lp0002-risc0-fixture-new/lez-execution.json`. The recorded wrapper evidence has `status: executed`, `proposal_state_executed: true`, and `proposal_state_nullifier_count: 2`. `spel-adapter-evidence.json` records the serialized byte payload for the NSSA/SPEL lane. The executable `verify_and_execute_bytes` wrapper is deployed on localnet and the compact native submitter has confirmed inclusion in block `1995`; formal CU counters remain pending target runtime support.

## Wrapper Localnet Inclusion Evidence

The executable `verify_and_execute_bytes` wrapper image was deployed on LEZ localnet and exercised with the real `RISC0_DEV_MODE=0` proof artifact set. Raw receipt transport was first attempted and rejected by the current public-program session limit (`Session limit exceeded: 33554432 >= 33554432`), so the successful compact path sends a receipt/journal commitment in the wrapper input and retains the full receipt as file-backed evidence.

| Metric | Value |
|---|---:|
| Wrapper program id | `ed00151765f6704d87f1a036b97207e2f3f83342d407657257ae466b996ca343` |
| Threshold proof image id | `026e95199ae495d946f7632d721823def2756584332c771a64207114311d4f01` |
| Confirmed tx hash | `596ddb4d798c3e45b2c4da9a15a33638ccf85f54aec7efa52cf822a87591d599` |
| Included block id | `1995` |
| Included tx index | `0` |
| Instruction payload | `5,492 bytes` / `1,373 u32 words` |
| Instruction SHA-256 | `4a04669d3d183d659353f72a7fa0ca7adc61d41ca07b8d7de2642f861d96a677` |
| Receipt bytes retained off-input | `270,334 bytes`, SHA-256 `8142fe9e92d144541d579521940ee873f09d15fb60aad4eb45f3c369fe3177ff` |
| Sequencer wrapper execution-time log | `11.122875ms` for the confirmed transaction block check |

Evidence files: `target/lp0002-risc0-fixture-new/localnet-wrapper-deploy-final.json`, `target/lp0002-risc0-fixture-new/nssa-submit-evidence.json`, and `.scaffold/logs/sequencer.log`. LEZ v0.2.0-rc1 does not expose per-transaction CU counters over JSON-RPC, so this is inclusion + payload + sequencer execution-time evidence rather than a formal CU meter.


## Machine-readable LEZ Cost Evidence

`submission/LEZ_COST_BENCHMARKS.json` is generated by `python3 scripts/benchmark-lez-costs.py` and records the reproducible wrapper cost surface: account count, instruction payload bytes, RISC0 receipt size, journal hash, receipt/journal commitment, localnet inclusion block/tx, and an explicit `cu_metering.available=false` marker because the current `lgs`/NSSA localnet query path does not expose a stable per-transaction compute-unit counter.
