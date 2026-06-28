# LP-0002 Protocol: Private M-of-N Multisig
> Current reset-era refresh (2026-06-28): deploy tx `c7157a473cb512bf7e1803d4377d9f65e9406a7ff98efeda48b65c0d4915a13b` is included on the public LEZ testnet for program id `1557176a639868b0363e9106c75fe0748ceb42e65f5f1a6778dd05b6baebb57d` (ProgramBinary SHA-256 `8f74ccc446990f5437b5f6c6e731deac6653992e0a64abcecdff7bff0c5575e1`). Execute attempts `352eb699507aea4d4ca6963a50bef1473a2b944dfd7713116cbf82eabfeec3bf` and `fc4165ac2437bd6533444c5e010b2d248aed678daadfad277af1dd0f1fef6ca8` locally validate under v0.2.0 but are not included by the public endpoint, so current live execute inclusion remains a transparent blocker and is not claimed. Historical pre-reset txs `82516880...` / `cb8bfd5...` are retained only as audit history.

## 1. Overview

This document describes the cryptographic protocol, nullifier design, LEZ
account model compatibility, security assumptions, known limitations, and
integration instructions for the LP-0002 private M-of-N multisig
implementation.

## 2. Threshold Proof Scheme

### 2.1 Objects

| Object | Derivation | Visibility |
|---|---|---|
| `member_secret` | `H("lp0002:member-secret", seed)` | Private (held by member) |
| `member_commitment` | `H("lp0002:member-commitment", member_secret)` | Private (in witness only) |
| `member_root` | `H("lp0002:member-root", sorted_commitments)` | Public (in multisig config) |
| `multisig_id` | `H("lp0002:multisig", label, member_root, threshold)` | Public (in multisig config) |
| `proposal_id` | `H("lp0002:proposal", label, action_hash)` | Public |
| `action_hash` | `H("lp0002:proposal-action", action_description)` | Public |
| `approval_nullifier` | `H("lp0002:approval-nullifier", multisig_id, proposal_id, member_secret)` | Public (in proof journal) |

The hash function `H` is SHA-256 with length-prefixed domain-separated inputs:
each chunk is encoded as `u64_le(chunk_len) || chunk_data`.

### 2.2 Proof Relation

A valid threshold proof attests that:

1. The prover knows at least **M** distinct `member_secret` values.
2. Each derived `member_commitment` belongs to the committed member set
   represented by `member_root`.
3. Each derived `approval_nullifier` is unique for `(multisig_id, proposal_id)`.
4. The proof is bound to the public `action_hash`.
5. The journal reveals nullifiers and counts, **but not** member secrets or
   commitments.

In the **safe-lane** (current) implementation, the proof is a deterministic
SHA-256 binding — not a real ZK proof. In the **heavy-lane** (RISC0 target),
the relation runs inside a RISC0 zkVM guest, producing a verified receipt.

### 2.3 Receipt Structure

```
ThresholdProof {
    journal: ThresholdJournal {
        domain, multisig_id, proposal_id, action_hash,
        member_root, member_count, threshold,
        approval_count, nullifiers, proof_id
    },
    receipt_seal: Digest32
}
```

The `receipt_seal` is:
- **Safe-lane**: `H("lp0002:mock-risc0-receipt-seal", proof_id, witness_commitment, member_root, threshold)`
- **Heavy-lane**: The RISC0 receipt seal (cryptographic commitment to the guest execution)

## 3. Nullifier Design

### 3.1 Construction

```
nullifier = H("lp0002:approval-nullifier", multisig_id, proposal_id, member_secret)
```

Each nullifier is deterministic for a given `(member, multisig, proposal)`
triple. An observer who sees a nullifier on-chain cannot determine which
member produced it, because `member_secret` is private and `member_commitment`
is not published.

### 3.2 Double-Vote Prevention

The prover collects nullifiers in a `BTreeSet` during proof generation.
If the same member attempts to approve the same proposal twice, the nullifier
collision is detected before the proof is created, and `prove_threshold`
returns `DuplicateNullifier`.

### 3.3 Cross-Proposal Unlinkability

Different `proposal_id` values produce different nullifiers for the same
member. An observer cannot link approvals across proposals by the same member,
because the nullifier changes with each proposal context.

### 3.4 Cross-Multisig Unlinkability

Different `multisig_id` values produce different nullifiers. A member's
activity in one multisig cannot be linked to their activity in another.

## 4. LEZ Account Model Compatibility

### 4.1 The Problem

The public `lez-multisig` PoC requires member accounts to be fresh zero-nonce
keypairs claimed by the multisig program. This is fundamentally incompatible
with shielded LEZ accounts, which:

- Are owned by the privacy protocol (not by arbitrary programs)
- Increment nonce on every use
- Cannot be "claimed" by a program the way public accounts can

### 4.2 Our Approach

Our private multisig **decouples** the approval mechanism from the LEZ account
model entirely:

- Members hold shielded LEZ accounts that remain under the privacy protocol
- The multisig verifier never attempts to own, claim, or mutate member accounts
- Instead, members produce cryptographic approvals (nullifiers) off-chain
- The verifier consumes a proof journal + nullifiers, not account mutations
- The `program_owner` constraint is irrelevant because the verifier doesn't
  need to own any accounts — it just verifies a threshold proof

This design is compatible with both the current LEZ nonce model and any future
changes to the nonce/account architecture, because the multisig never directly
interacts with member accounts.

### 4.3 Shielded Account as Secret Material

A member's shielded LEZ account private key (or a derivative thereof) serves
as the `seed` input to `MemberSecret::from_seed()`. This binds the multisig
membership to the member's shielded identity without requiring on-chain
account operations.

## 5. Security Assumptions

### 5.1 Cryptographic Assumptions

- **SHA-256** is collision-resistant and preimage-resistant
- **Member secrets** are never exposed outside the prover's local environment
- The **merkleish_root** construction (hash of sorted commitments) is binding:
  it is infeasible to find two distinct member sets with the same root

### 5.2 Safe-Lane Honesty

The repository contains two deliberately separate execution lanes:

- **Safe lane (`core/`, `sdk/`, `verifier-program/`, `demo.sh`)**: fast,
  clone-and-run Rust logic that computes a deterministic SHA-256 receipt seal.
  This lane is useful for SDK integration, error handling, replay protection,
  and evaluator smoke tests, but it is not by itself a zkVM proof.
- **Heavy lane (`methods/`, `host/`, `lez-program/`, `scripts/demo-heavy-lane.sh`)**:
  real `RISC0_DEV_MODE=0` proof artifacts generated on the M4 Pro, verified
  host-side against image id
  `6fc85ce06da1762abec319b4626c12229dc605a5b0283d64c8eab2567b9ee721`, and
  bridged into the LEZ-shaped execution wrapper.

The safe-lane receipt must not be treated as a production ZK receipt: a malicious
party could compute the SHA-256 seal without running the zkVM. The submission's
ZK integrity claim rests on the heavy-lane RISC0 receipt path and its recorded
file-backed evidence, not on the fast root demo alone.

### 5.3 Heavy-Lane RISC0 Evidence

For the recorded heavy-lane artifacts:

- The receipt provides computational integrity: the verifier is assured that
  the threshold relation was correctly evaluated inside the zkVM.
- The member secrets remain private because they are committed via hashes and
  never appear in the public journal.
- The receipt seal is verified against the known LP-0002 image ID, preventing
  SHA-256 mock-receipt forgery in the evidence path.
- The included LEZ public-testnet transaction carries compact receipt/journal
  commitments because raw receipt bytes exceed the current public-program
  session transport limit; the full receipt remains host-verified and
  file-backed for evaluator inspection.

## 6. Known Limitations

1. **Safe-lane receipt is not a zkVM receipt**: The fast root demo uses a
   deterministic SHA-256 seal for clone-and-run usability. The real ZK integrity
   evidence is the separate heavy-lane RISC0 receipt path documented in
   `submission/TESTNET_EVIDENCE.json` and exercised by `scripts/demo-heavy-lane.sh`.

2. **Compact LEZ transport boundary**: the executable `verify_and_execute_bytes`
   wrapper compiles, has been deployed on the public LEZ testnet
   (https://testnet.lez.logos.co/) — deploy tx `82516880...` in block `39547` —
   and has confirmed NSSA transaction inclusion (execute tx `cb8bfd5a...`) in
   block `39548`. Because raw RISC0 receipts exceed the current public-program
   session limit, the included wrapper transaction carries receipt/journal
   commitments while the full receipt remains host-verified and file-backed
   evidence.

3. **In-memory accumulator**: The `ApprovalAccumulator` is an in-memory struct.
   A production client needs persistent storage (e.g., encrypted local file or
   secure enclave) to survive crashes and maintain approval state across sessions.

4. **No membership revocation**: Once a member is added to the multisig, there
   is no mechanism to revoke their membership. A production implementation would
   need a membership update flow (e.g., creating a new multisig with the updated
   member set).

5. **No proposal content encryption**: The proposal description and action hash
   are public. Only member identities and vote attribution are private.

6. **Deterministic nullifiers**: While nullifiers hide which member approved,
   a member who approves the same proposal in two different multisigs will
   produce different nullifiers (by design). However, if an observer knows a
   member's secret and the multisig config, they can compute the nullifier and
   check if that member approved. This is inherent to the nullifier design and
   is the same trade-off made by Semaphore.

## 7. Integration Guide

### 7.1 Quick Start with SDK

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

// 3. Collect approvals
session.approve(0)?; // Alice
session.approve(2)?; // Cyra (Boris abstains)

// 4. Generate threshold proof
let proof = session.prove()?;

// 5. Verify and execute
let receipt = session.verify_and_execute(ProposalAction::Transfer {
    to: "logos1recipient".into(),
    amount: 42,
    denom: "LOGOS".into(),
})?;
```

### 7.2 Using Raw Core/Verifier Types

For advanced use cases where you need fine-grained control:

```rust
use lp0002_private_multisig_core::*;
use lp0002_private_multisig_verifier::*;

let members = vec![
    MemberSecret::from_seed(b"member-alice"),
    MemberSecret::from_seed(b"member-boris"),
];
let config = MultisigConfig::new("council", 2, &members)?;
let proposal = Proposal::new("action", "description");

// Prove with all members
let proof = prove_threshold(&config, &proposal, &members)?;

// Verify and execute
let mut verifier = VerifierProgram::default();
let receipt = verifier.execute_if_threshold_met(
    &config, &proposal, &proof,
    ProposalAction::Custom {
        description: "governance".into(),
        payload_hash: [0u8; 32],
    },
)?;
```

### 7.3 Resumable Partial Approvals

Use `ApprovalAccumulator` to collect approvals incrementally across sessions:

```rust
let mut acc = ApprovalAccumulator::new(config.multisig_id, proposal.id);
acc.add_member_approval(&members[0])?; // Session 1

// Serialize and persist
let state = serde_json::to_string(&acc)?;

// Later, in Session 2:
let mut acc: ApprovalAccumulator = serde_json::from_str(&state)?;
acc.add_member_approval(&members[1])?;

if acc.is_threshold_met(config.threshold) {
    let proof = prove_threshold(&config, &proposal, &approved_members)?;
}
```

### 7.4 Consumer Demo as Reference Integration

The `consumer-demo/` crate provides a complete, standalone integration example.
Any third-party project can follow the same pattern:

1. Add `lp0002-private-multisig-sdk` as a dependency
2. Create a `MultisigSession` with the desired threshold and member seeds
3. Follow the 5-step workflow: create -> propose -> approve -> prove -> execute

Run it: `cargo run -p lp0002-consumer-demo`

## 8. Deterministic Error Codes

The system returns typed, documented errors for every failure path:

### 8.1 Proof Errors (`ProofError`)

| Code | Error | Description |
|---|---|---|
| — | `EmptyMemberSet` | No members provided during multisig creation |
| — | `InvalidThreshold { threshold, member_count }` | Threshold is 0 or exceeds member count |
| — | `InsufficientApprovals { threshold, provided }` | Fewer than M approvals submitted |
| — | `DuplicateNullifier` | Same member tried to approve the same proposal twice |
| — | `MultisigIdMismatch` | Proof was generated for a different multisig |
| — | `MultisigRootMismatch` | Member root in proof doesn't match config |
| — | `ProposalMismatch` | Proof proposal context doesn't match verifier's proposal |
| — | `ThresholdMismatch` | Threshold or member count in proof differs from config |
| — | `ApprovalCountNullifierMismatch` | Public approval count does not equal the number of nullifiers |
| — | `MemberCommitmentSetMismatch` | Guest-derived member commitments do not match the committed member set |
| — | `UnknownApprovingMember` | An approving secret does not correspond to a configured member commitment |
| — | `ReceiptSealMismatch` | Receipt seal is all-zeros (invalid) |

### 8.2 Verifier Errors (`VerifierError`)

| Code | Error | Description |
|---|---|---|
| — | `InvalidProof(ProofError)` | Wraps any proof verification failure |
| — | `ProposalAlreadyExecuted` | Attempt to execute a proposal that was already executed |

### 8.3 SDK Errors (`SdkError`)

| Code | Error | Description |
|---|---|---|
| — | `Proof(ProofError)` | Wraps proof-layer error |
| — | `Verifier(VerifierError)` | Wraps verifier-layer error |
| — | `NoProposal` | Called prove/execute before creating a proposal |
| — | `NoProof` | Called execute before generating a proof |
| — | `InvalidMemberIndex { index, count }` | Member index out of range |
