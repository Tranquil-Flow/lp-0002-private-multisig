# Consumer Integration Guide

## Overview

The LP-0002 SDK is designed for easy integration into any application that needs
privacy-preserving threshold governance. This document describes how third-party
developers can use the LP-0002 crates.

## Prerequisites

- Rust 1.70+ (edition 2021)
- No RISC0/Docker/network required for safe-lane integration

## Adding the Dependency

```toml
[dependencies]
lp0002-private-multisig-sdk = { path = "../lp-0002-private-multisig/sdk" }
# External projects can use a git dependency after public release; crates.io publication is pending.
```

Or use the core + verifier crates directly for fine-grained control:

```toml
[dependencies]
lp0002-private-multisig-core = { path = "../lp-0002-private-multisig/core" }
lp0002-private-multisig-verifier = { path = "../lp-0002-private-multisig/verifier-program" }
```

## Integration Patterns

### Pattern 1: Full SDK (Recommended)

Use `MultisigSession` for the simplest integration:

```rust
use lp0002_private_multisig_sdk::prelude::*;

let mut session = MultisigSession::new("my-multisig", 2, vec![
    b"member-1-key".as_slice(),
    b"member-2-key".as_slice(),
    b"member-3-key".as_slice(),
])?;

session.create_proposal("action-1", "description of proposed action");
session.approve(0)?;
session.approve(1)?;
let proof = session.prove()?;
let receipt = session.verify_and_execute(ProposalAction::Transfer {
    to: "logos1recipient".into(),
    amount: 100,
    denom: "LOGOS".into(),
})?;
```

### Pattern 2: Raw Core/Verifier (Advanced)

For applications that need custom approval collection or proof handling:

```rust
use lp0002_private_multisig_core::*;
use lp0002_private_multisig_verifier::*;

let members = vec![
    MemberSecret::from_seed(b"member-key"),
];
let config = MultisigConfig::new("1-of-1", 1, &members)?;
let proposal = Proposal::new("action", "description");
let proof = prove_threshold(&config, &proposal, &members)?;

let mut verifier = VerifierProgram::default();
let receipt = verifier.execute_if_threshold_met(
    &config, &proposal, &proof,
    ProposalAction::Custom {
        description: "custom action".into(),
        payload_hash: [0u8; 32],
    },
)?;
```

### Pattern 3: Resumable Partial Approvals

For applications where approvals arrive over time:

```rust
let mut acc = ApprovalAccumulator::new(config.multisig_id, proposal.id);

// Collect approvals as they arrive (possibly across sessions)
acc.add_member_approval(&members[0])?;

// Serialize and persist for later
let state = serde_json::to_string(&acc)?;
// ... store state to disk ...

// Later, restore and continue
let mut acc: ApprovalAccumulator = serde_json::from_str(&state)?;
acc.add_member_approval(&members[1])?;

// When threshold met, generate proof
if acc.is_threshold_met(config.threshold) {
    let proof = prove_threshold(&config, &proposal, &approved_members)?;
}
```

## Consumer Demo

The `consumer-demo/` crate demonstrates all three patterns across 5 scenarios:

1. 2-of-3 treasury transfer (SDK pattern)
2. 3-of-5 governance parameter change (non-sequential approvers)
3. Resumable partial approvals (serialization roundtrip)
4. Error paths (double-vote, insufficient approvals, invalid index)
5. Replay protection (double-execution rejection)

Run it: `cargo run -p lp0002-consumer-demo`

## Reference Integration Evidence

The current public LP-0002 specification requires a reproducible reference
integration plus evidence for at least one multisig instance, proposal,
threshold approval, and execution. It no longer requires five external multisig
operators. The `consumer-demo/` crate is the clone-and-run integration surface;
the RISC0/localnet evidence path is documented in `submission/EVALUATOR_GUIDE.md`
and `submission/TESTNET_EVIDENCE.json`.
