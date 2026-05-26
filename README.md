# LP-0002: Private M-of-N Multisig

A privacy-preserving M-of-N multisig primitive for the Logos Execution Zone
(LEZ), where members hold shielded accounts, proposals and approvals leave no
public trace of who voted, and on-chain state reveals only that a threshold was
met — not which members approved.

## Status

This repository now contains both the **safe-lane** reference implementation and
the first heavy-lane RISC0 implementation. The safe-lane SDK, consumer demo, and
replay-protected verifier gate remain fast and clone-and-run. The heavy lane adds
a RISC0 guest method plus host prover that produces real `RISC0_DEV_MODE=0`
proof artifacts sealing the private membership relation, plus a LEZ-shaped
`execute_proposal` transaction wrapper crate. The heavy lane now includes
localnet wrapper deployment and confirmed compact NSSA transaction inclusion;
for LP-0002, this LEZ localnet is the evaluator/public-testnet target. Formal
per-transaction CU counters are recorded as unavailable in current LEZ tooling
rather than invented; see `submission/LEZ_COST_BENCHMARKS.json`.

## Quick Start

```bash
# Run the consumer integration demo (5 scenarios)
./demo.sh

# Run safe-lane tests locally
cargo test -p lp0002-private-multisig-core -p lp0002-private-multisig-sdk -p lp0002-private-multisig-verifier --tests

# Run RISC0 host/method tests where cargo-risczero is installed
cargo test -p lp0002-private-multisig-host --tests

# Run LEZ transaction wrapper tests
cargo test -p lp0002-private-multisig-lez-program --tests

# Run benchmarks
cargo run --example bench --release

# Validate submission files
python3 scripts/validate-submission-readiness.py

# Generate and verify real RISC0 proof artifacts (requires cargo-risczero)
RISC0_DEV_MODE=0 cargo run -p lp0002-private-multisig-host --bin lp0002-prove-fixture -- target/lp0002-risc0-fixture
cargo run -p lp0002-private-multisig-host --bin lp0002-verify-artifacts -- target/lp0002-risc0-fixture

# Smoke-check LEZ localnet plus RISC0 artifacts
scripts/lez-localnet-smoke.sh

# Recording-safe heavy-lane evidence demo
bash scripts/demo-heavy-lane.sh
```

## Architecture

```
lp-0002-private-multisig/
├── core/                    # Cryptographic primitives and threshold relation
│   └── src/lib.rs           # MemberSecret, MultisigConfig, Proposal,
│                            # ApprovalAccumulator, prove_threshold,
│                            # verify_threshold_receipt
│   └── tests/               # 14 core tests
├── verifier-program/        # LEZ-shaped execution gate
│   └── src/lib.rs           # VerifierProgram: verify + execute-once
│   └── tests/               # 5 verifier tests
├── sdk/                     # High-level integration SDK
│   └── src/lib.rs           # MultisigSession: 5-step workflow
├── methods/                 # RISC0 heavy-lane method crate
│   └── guest/               # threshold_proof guest program
├── host/                    # RISC0 prover/verifier artifact adapter
│   └── src/bin/             # lp0002-prove-fixture, lp0002-verify-artifacts
├── lez-program/             # LEZ-shaped execute_proposal wrapper
│   └── src/lib.rs           # accounts, instruction payload, replay state
├── consumer-demo/           # Standalone integration demo
│   └── src/main.rs          # 5 scenarios: transfer, governance,
│                            #   resume, errors, replay
│   └── examples/bench.rs    # Benchmark harness
├── basecamp-app/            # Browser preview plus native Qt/QML Basecamp plugin package
│   ├── index.html           # 5-step interactive walkthrough
│   ├── app.js               # Full SHA-256 crypto in JavaScript
│   └── styles.css           # Dark theme
├── interfaces/              # SPEL IDL and interface definitions
│   ├── lp0002.idl.json      # SPEL program IDL (6 instructions)
│   └── lp0002.spel          # Human-readable interface
├── docs/                    # Documentation
│   ├── PROTOCOL.md          # Full protocol specification
│   ├── CONSUMER_INTEGRATION.md  # Integration guide
│   └── SPEC_COMPLIANCE.md   # Spec compliance matrix
├── submission/              # Submission materials
│   ├── BENCHMARKS.md        # Performance benchmarks
│   └── MAINTAINER_CLARIFICATION.md  # Consumer-demo clarification
├── solutions/LP-0002.md     # Solution description
├── scaffold.toml             # Logos scaffold / LEZ localnet config
├── scripts/                 # Tooling
│   ├── demo-heavy-lane.sh
│   ├── lez-localnet-smoke.sh
│   └── validate-submission-readiness.py
└── demo.sh                  # One-command demo runner
```

## SDK Usage

```rust
use lp0002_private_multisig_sdk::prelude::*;

// 1. Create a 2-of-3 private multisig
let mut session = MultisigSession::new(
    "treasury",
    2,
    vec![
        b"alice-shielded-key".as_slice(),
        b"boris-shielded-key".as_slice(),
        b"cyra-shielded-key".as_slice(),
    ],
)?;

// 2. Define a proposal
session.create_proposal("grant-42", "transfer 42 LOGOS to recipient");

// 3. Collect approvals (Alice and Cyra approve; Boris abstains)
session.approve(0)?;
session.approve(2)?;

// 4. Generate threshold proof
let proof = session.prove()?;

// 5. Verify and execute the threshold-gated action
let receipt = session.verify_and_execute(ProposalAction::Transfer {
    to: "logos1recipient".into(),
    amount: 42,
    denom: "LOGOS".into(),
})?;
```

## Privacy Model

The public journal exposes only:

- Multisig ID and member root
- Proposal ID and public action hash
- Approval nullifiers (deterministic per member+proposal, but unlinkable)
- Approval count and threshold
- Proof ID / receipt seal

It intentionally does **not** expose:

- Raw shielded account secrets
- Member commitments
- Which specific members approved

## Nullifier Design

```
nullifier = H("lp0002:approval-nullifier", multisig_id, proposal_id, member_secret)
```

- **Double-vote prevention**: Same member + proposal = same nullifier (deduped)
- **Cross-proposal unlinkability**: Different proposal = different nullifier
- **Cross-multisig unlinkability**: Different multisig = different nullifier
- **Observer privacy**: Knowing the nullifier doesn't reveal which member produced it

## LEZ Account Compatibility

The public `lez-multisig` PoC requires fresh zero-nonce keypairs claimed by the
program — incompatible with shielded accounts. Our design avoids this:

- Members keep shielded accounts under the privacy protocol
- The multisig verifier only consumes proof journals + nullifiers
- No program ownership of member accounts required
- No nonce constraint violations

## Benchmarks

| Config | Config::new | Prove | Verify | Execute | Proof Size |
|--------|-------------|-------|--------|---------|------------|
| 2-of-3 | 1.4 μs | 4.8 μs | 29 ns | 75 ns | 306 B |
| 3-of-5 | 2.1 μs | 3.4 μs | 53 ns | 92 ns | 338 B |
| 5-of-10 | 3.4 μs | 5.0 μs | 84 ns | 125 ns | 402 B |
| 10-of-20 | 6.6 μs | 8.6 μs | 173 ns | 216 ns | 562 B |
| 25-of-50 | 15.6 μs | 20.0 μs | 461 ns | 527 ns | 1042 B |

RISC0 heavy-lane proof generation is wired through `host/` and queued for
measurement on the M4 Pro. The LEZ `execute_proposal` account/instruction
wrapper is implemented in `lez-program/`; the executable `verify_and_execute_bytes` wrapper has confirmed localnet inclusion evidence via compact receipt/journal-commitment transport. Cost evidence is recorded in `submission/LEZ_COST_BENCHMARKS.json`; the current LEZ JSON-RPC surface exposes inclusion and payload/account metrics but not per-transaction CU counters, so that limitation is explicit rather than estimated.

## Test Coverage

**30 tests** across the workspace when RISC0 host and LEZ-wrapper tests are included:

- 14 core tests: threshold proof, guest relation, nullifiers, privacy, context
  binding, accumulator serialization, large configurations, edge cases
- 5 verifier tests: execution gate, replay protection, root mismatch,
  threshold mismatch, nullifier uniqueness
- 2 RISC0 host boundary tests: Borsh guest input and journal decoding
- 4 LEZ wrapper tests: Borsh payload, image-id check, state update, replay rejection
- 1 SDK doc test: prelude compilation

## Consumer Demo Scenarios

The consumer demo runs 5 scenarios demonstrating real-world integration:

1. **2-of-3 Treasury Transfer** — SDK high-level API
2. **3-of-5 Governance Parameter Change** — non-sequential approvers
3. **Resumable Partial Approvals** — client restart with serialization
4. **Error Paths** — double-vote, insufficient approvals, invalid index
5. **Replay Protection** — double-execution rejection

## License

MIT
