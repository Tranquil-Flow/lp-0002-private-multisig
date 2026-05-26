# LP-0002 Heavy-Lane Roadmap: RISC0 + LEZ

The current LP-0002 repository contains both the fast safe-lane reference
implementation and a completed first heavy-lane evidence path. The safe lane nails
the relation, nullifier model, replay gate, SDK, consumer demo, and documentation.
The heavy lane adds real RISC0 `DEV_MODE=0` artifacts, a LEZ-shaped execution
wrapper, and confirmed localnet/evaluator transaction inclusion.

This document records the completed heavy-lane path, the precise trust boundary,
and the remaining nice-to-have evidence that depends on future LEZ tooling.

## Status update

Completed in this increment:

- `core::ThresholdGuestInput` and `prove_threshold_guest` validate the private
  member commitment set against the public root and reject outsider approvers.
- `methods/guest/src/bin/threshold_proof.rs` seals the threshold relation in
  RISC0 and commits Borsh `ThresholdJournal` bytes.
- `host/` provides `lp0002-prove-fixture` and `lp0002-verify-artifacts` for real
  `RISC0_DEV_MODE=0` receipts.
- `scaffold.toml` and `scripts/lez-localnet-smoke.sh` establish the LEZ localnet
  smoke path.
- `lez-program/` implements the LEZ-shaped `execute_proposal` account state,
  Borsh instruction payload, pluggable RISC0 receipt-verifier boundary, replay
  guard, and proposal-state mutation tests.
- `host::Risc0ReceiptVerifier` is the concrete bridge from RISC0 receipt bytes to
  the LEZ wrapper boundary: it checks the compiled image id, verifies the receipt,
  and rejects any supplied journal that does not exactly match the receipt journal.
- `lp0002-lez-execute-artifacts` runs verified RISC0 artifacts through
  `lez-program::execute_proposal` and writes `lez-execution.json` evidence.
- `verify_and_execute_bytes` is now exposed in the SPEL IDL as a byte-oriented
  transaction surface for the current CLI/tooling, and
  `lp0002-spel-adapter-evidence` builds the exact serialized NSSA instruction
  payload from the real receipt, journal, and action bytes.


### Current heavy-lane evidence (2026-05-25)

- Real `RISC0_DEV_MODE=0` fixture proof generated and verified on M4 Pro.
- Image id: `026e95199ae495d946f7632d721823def2756584332c771a64207114311d4f01`.
- Proof id: `9e6492e73d1e8382abfa0e94e91842100b9041516857f215fcad7276cbad8b11`.
- Artifacts: `target/lp0002-risc0-fixture-new/{receipt.borsh,journal.borsh,manifest.txt}`.
- LEZ wrapper evidence: `target/lp0002-risc0-fixture-new/lez-execution.json` with `status: executed`.
- File-backed NSSA submitter evidence: `lp0002-submit-localnet` constructs a public transaction from receipt/journal/action files and can submit/query via `NSSA_WALLET_HOME_DIR=.scaffold/wallet`. The executable `verify_and_execute_bytes` wrapper image was deployed on localnet and tx `596ddb4d798c3e45b2c4da9a15a33638ccf85f54aec7efa52cf822a87591d599` was included in block `1995`. Raw receipt transport exceeded the current public-program session limit, so the included wrapper input carries the receipt/journal commitment while retaining the full receipt as host-side proof evidence.
- SPEL/NSSA adapter payload evidence: `target/lp0002-risc0-fixture-new/spel-adapter-evidence.json` with `status: spel_adapter_payload_built`, instruction `verify_and_execute_bytes`, instruction payload length `5,492` bytes, instruction data SHA-256 `4a04669d3d183d659353f72a7fa0ca7adc61d41ca07b8d7de2642f861d96a677`, and receipt/journal commitment `68141a959293adaaebffb41be3969ecccf30e43947e0008ed10726b8e03444e7`.
- `scaffold.toml` migrated to logos-scaffold schema `0.2.0` and `lgs doctor` reaches `22 PASS, 0 WARN, 0 FAIL` with localnet live on port 3040.
- `lgs deploy verify_and_execute_bytes --program-path ... --json` submits the executable wrapper image to localnet: `status: submitted`, `program_id: ed00151765f6704d87f1a036b97207e2f3f83342d407657257ae466b996ca343`.
- The earlier adapter gap is resolved for current tooling by compact transport:
  the native wallet/NSSA submitter sends receipt and journal commitments while
  the full receipt remains file-backed, host-verified evidence. For LP-0002, the
  LEZ localnet target is the evaluator/public-testnet target.

Still dependent on future target-network tooling:

- Formal per-transaction CU measurements once exposed by the target LEZ runtime/RPC surface.


## Trust boundary

Safe-lane verifier behavior:

- `prove_threshold` computes a deterministic `ThresholdJournal` and mock
  `receipt_seal`.
- `verify_threshold_receipt` checks public context binding, threshold counts,
  duplicate nullifiers, and that the seal is non-zero.
- It cannot prove the nullifiers came from real hidden members. That integrity
  property is provided by the heavy-lane RISC0 receipt path.

Therefore the root safe-lane demo must not be narrated as:

- a real RISC0 proof,
- an on-chain LEZ deployment,
- measured LEZ compute-unit evidence,
- or a production ZK verifier by itself.

## Required heavy-lane components

### 1. RISC0 guest method

Implemented as:

```text
methods/
  Cargo.toml
  build.rs
  guest/
    Cargo.toml
    src/bin/threshold_proof.rs
```

Guest private input:

- approving member secrets,
- optional membership witness data if the member-set representation is upgraded
  from sorted commitments to a Merkle tree.

Guest public/context input:

- `MultisigConfig`,
- `Proposal`,
- expected threshold,
- expected member root.

Guest journal output:

- exactly the current `ThresholdJournal` fields.

Guest relation:

1. Derive commitments from private member secrets.
2. Prove each commitment belongs to the committed member set.
3. Derive nullifiers for `(multisig_id, proposal_id, member_secret)`.
4. Reject duplicate nullifiers.
5. Reject fewer than threshold approvals.
6. Commit the public `ThresholdJournal`.

### 2. Host prover

Implemented in `host/`. The host adapter:

1. Serializes the private and public inputs.
2. Runs the RISC0 executor/prover with `RISC0_DEV_MODE=0`.
3. Decodes the journal bytes into `ThresholdJournal`.
4. Persists receipt artifacts for demo/readiness:
   - image id,
   - receipt file,
   - journal JSON,
   - proof timestamp,
   - dev-mode flag.

### 3. Verifier integration

Replace the mock seal check with real receipt verification:

```rust
receipt.verify(LP0002_THRESHOLD_GUEST_ID)?;
let journal = decode_receipt_journal(receipt.journal.bytes())?;
verify_public_journal(config, proposal, &journal)?;
```

Keep the existing safe-lane public-journal checks; they are still useful after
RISC0 because they bind the receipt to the active on-chain state.

### 4. LEZ program / scaffold integration

`verifier-program/` is the pure Rust execution gate; `lez-program/` now wraps it
in LEZ-shaped account state and Borsh instruction payloads. A deployable scaffold
adapter still needs to connect that wrapper to NSSA runtime account IO and submit
transactions that:

- read accounts and instruction args via LEZ/NSSA input APIs,
- call the concrete RISC0 receipt verifier for the LP-0002 image id,
- use `lez-program::execute_proposal` for replay/journal/account checks,
- write executed proposal state,
- emit an execution receipt/event.

Current workflow on the M4 Pro:

```bash
cd ~/Projects/logos-basecamp/lp-0002-private-multisig
export PATH="$HOME/.cargo/bin:/Applications/Docker.app/Contents/Resources/bin:$PATH"

# build and verify real RISC0 guest/prover artifacts
RISC0_DEV_MODE=0 cargo run -p lp0002-private-multisig-host --bin lp0002-prove-fixture -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-verify-artifacts -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-lez-execute-artifacts -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-spel-adapter-evidence -- target/lp0002-risc0-fixture

# smoke-check scaffold and localnet
scripts/lez-localnet-smoke.sh

# next increment: submit this same instruction as a real NSSA transaction through the deployed wrapper
```

### 5. Real benchmarks

Update `submission/BENCHMARKS.md` with measured values:

- RISC0 guest build time,
- RISC0 proof generation time for at least 2-of-3, 3-of-5, 10-of-20,
- receipt size,
- LEZ verification compute units,
- end-to-end transaction latency.

## Environment requirements

This work should run on the M4 Pro, not the lightweight container, because it
requires:

- `cargo-risczero` and RISC0 Rust target,
- Docker Desktop in PATH and running,
- `lgs` / Logos scaffold setup,
- `spel` tooling,
- LEZ localnet/testnet wallet and credentials,
- enough memory/time for real proving.

## Estimated effort

If the M4 Pro toolchain is healthy:

- RISC0 guest + host prover: 2-4 hours
- LEZ scaffold/deployment wrapper: 3-6 hours
- localnet/testnet debugging and benchmarks: 2-5 hours
- demo-video script update: 1 hour

Total: roughly one focused day, with most risk in LEZ program/scaffold glue rather
than in the threshold relation itself.
