# LP-0002 Heavy-Lane Roadmap: RISC0 + LEZ
> Current reset-era refresh (2026-06-28): deploy tx `c7157a473cb512bf7e1803d4377d9f65e9406a7ff98efeda48b65c0d4915a13b` is included on the public LEZ testnet for program id `1557176a639868b0363e9106c75fe0748ceb42e65f5f1a6778dd05b6baebb57d` (ProgramBinary SHA-256 `8f74ccc446990f5437b5f6c6e731deac6653992e0a64abcecdff7bff0c5575e1`). Execute attempts `352eb699507aea4d4ca6963a50bef1473a2b944dfd7713116cbf82eabfeec3bf` and `fc4165ac2437bd6533444c5e010b2d248aed678daadfad277af1dd0f1fef6ca8` locally validate under v0.2.0 but are not included by the public endpoint, so current live execute inclusion remains a transparent blocker and is not claimed. Historical pre-reset txs `82516880...` / `cb8bfd5...` are retained only as audit history.

The current LP-0002 repository contains both the fast safe-lane reference
implementation and a completed first heavy-lane evidence path. The safe lane nails
the relation, nullifier model, replay gate, SDK, consumer demo, and documentation.
The heavy lane adds real RISC0 `DEV_MODE=0` artifacts, a LEZ-shaped execution
wrapper, and confirmed public-testnet transaction inclusion.

This document records the completed heavy-lane path, the precise trust boundary,
and the optional target-runtime metrics that depend on future LEZ tooling.

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
- Image id: `6fc85ce06da1762abec319b4626c12229dc605a5b0283d64c8eab2567b9ee721`.
- Proof id: `9e6492e73d1e8382abfa0e94e91842100b9041516857f215fcad7276cbad8b11`.
- Artifacts: `target/lp0002-risc0-fixture-new/{receipt.borsh,journal.borsh,manifest.txt}`.
- LEZ wrapper evidence: `target/lp0002-risc0-fixture-new/lez-execution.json` with `status: executed`.
- File-backed NSSA submitter evidence: `lp0002-submit-localnet` constructs a public transaction from receipt/journal/action files and can submit/query via `NSSA_WALLET_HOME_DIR=.scaffold/wallet`. The executable `verify_and_execute_bytes` wrapper image was deployed on the public LEZ testnet before reset and tx `cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697` was included historically; current re-query now returns null in block `39548`. Raw receipt transport exceeded the current public-program session limit, so the included wrapper input carries the receipt/journal commitment while retaining the full receipt as host-side proof evidence.
- SPEL/NSSA adapter payload evidence: `target/lp0002-risc0-fixture-new/spel-adapter-evidence.json` with `status: spel_adapter_payload_built`, instruction `verify_and_execute_bytes`, instruction payload length `5,492` bytes, instruction data SHA-256 `e1dc304173c1f27542b0017e167eb709f47e6bc907888968e9efaf0cd655f3c0`, and receipt/journal commitment `be58410de0e0f71642f82f287c39c7f70acb8820cb7468e50927bfd91ee4c850`.
- `scaffold.toml` migrated to logos-scaffold schema `0.2.0` and `lgs doctor` reaches `22 PASS, 0 WARN, 0 FAIL` with localnet live on port 3040.
- The reproducible wrapper ELF (`cargo risczero build --manifest-path methods/guest/Cargo.toml`, Docker builder `risczero/risc0-guest-builder:r0.1.88.0`; on LEZ ProgramId == ImageID) was deployed to the public LEZ testnet via the submit binary's `ProgramDeployment` path: deploy tx `82516880f60c2076d78b28ad7b147ac0b05ed247b7bc33a27ac8f68b1d809c56` confirmed in block `39547`, `program_id: 974939edb6fc9cffd97929dd830a0d75bfc7a09b08c2f3fc87da940aadc0c130`.
- The earlier adapter gap is resolved for current tooling by compact transport:
  the native wallet/NSSA submitter sends receipt and journal commitments while
  the full receipt remains file-backed, host-verified evidence. For LP-0002, the
  wrapper has historical pre-reset public LEZ testnet deploy/execute evidence
  (https://testnet.lez.logos.co/): execute tx
  `cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697` in block
  `39548`. On LEZ a transaction is included in a block only if its program
  execution succeeds, so the confirmed execute tx is a successful on-chain run.

Dependent on future target-network tooling:

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

Implemented evidence path:

```rust
receipt.verify(LP0002_THRESHOLD_GUEST_ID)?;
let journal = decode_receipt_journal(receipt.journal.bytes())?;
verify_public_journal(config, proposal, &journal)?;
```

`host::Risc0ReceiptVerifier` performs the concrete receipt/image check and
journal binding. `lez-program::execute_proposal` then reuses the safe-lane public-
journal checks against active account state so the heavy-lane receipt remains
bound to the proposal, action, replay state, and wrapper transport commitments.

### 4. LEZ program / scaffold integration

`verifier-program/` is the pure Rust execution gate; `lez-program/` wraps it in
LEZ-shaped account state and Borsh instruction payloads. The recorded public-testnet
path uses the executable `verify_and_execute_bytes` wrapper program plus a native
file-backed NSSA submitter. For current LEZ public-program session limits, the
submitted transaction carries compact receipt/journal commitments while the full
receipt remains host-verified and file-backed evidence.

The wrapper/evidence flow:

- reads instruction args and account-style state through the LEZ-shaped boundary,
- verifies the real receipt host-side before preparing compact transport,
- uses `lez-program::execute_proposal` for replay/journal/account checks,
- writes executed proposal state in the wrapper evidence,
- records the included public-testnet transaction hash and block.

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

# submit/query the recorded public-testnet wrapper transaction when localnet is live
NSSA_WALLET_HOME_DIR=.scaffold/wallet cargo run -p lp0002-private-multisig-host --bin lp0002-submit-localnet -- \
  --evidence-dir target/lp0002-risc0-fixture \
  --program-id 974939edb6fc9cffd97929dd830a0d75bfc7a09b08c2f3fc87da940aadc0c130 \
  --query cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697
```

### 5. Benchmarks and target-runtime metric boundary

Recorded in `submission/BENCHMARKS.md` and
`submission/LEZ_COST_BENCHMARKS.json`:

- safe-lane operation timings across multiple M-of-N sizes,
- RISC0 image/proof identifiers and receipt/journal sizes,
- wrapper instruction payload length and hash,
- account count and confirmed public-testnet block inclusion,
- explicit `cu_metering.available=false` reason for the current LEZ toolchain.

The only unavailable metric is formal per-transaction CU; the current target LEZ
runtime/RPC surface does not expose stable counters.

## Operational notes

The heavy-lane proof and public-testnet evidence have already been generated on the M4
Pro for this submission. Re-running them from scratch still requires the M4 Pro
or an equivalent machine because the full path needs:

- `cargo-risczero` and RISC0 Rust target,
- Docker Desktop in PATH and running,
- `lgs` / Logos scaffold setup,
- `spel` tooling,
- a funded LEZ testnet wallet and credentials,
- enough memory/time for real proving.

## Reproducibility scope

The fast evaluator path is `./demo.sh` plus the validators. The heavy-lane path
is reproducible on the M4 Pro toolchain; lightweight containers can inspect and
verify the recorded evidence but may not have Docker/RISC0/localnet capacity to
regenerate every proof artifact from scratch.
