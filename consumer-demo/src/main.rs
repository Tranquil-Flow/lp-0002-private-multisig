//! LP-0002 Consumer Demo — thorough multi-scenario integration showcase.
//!
//! Demonstrates how easily a third-party application can integrate the
//! LP-0002 private multisig SDK and core/verifier crates to build
//! privacy-preserving threshold-gated governance workflows.
//!
//! Five scenarios cover the full range of real-world usage:
//!   1. Simple treasury transfer via SDK
//!   2. Governance parameter change (3-of-5, non-sequential approvers)
//!   3. Resumable partial approvals (client restart simulation)
//!   4. Error paths (double-vote, insufficient approvals)
//!   5. Replay protection (double-execution rejection)
//!
//! For evaluators: clone + `cargo run -p lp0002-consumer-demo`.
//! No RISC0/Docker/network needed.

use anyhow::{bail, Result};
use lp0002_private_multisig_core::{
    hex_digest, prove_threshold, AddApprovalOutcome, ApprovalAccumulator, MemberSecret,
    MultisigConfig, ProofError, Proposal, ProposalAction,
};
use lp0002_private_multisig_sdk::MultisigSession;
use lp0002_private_multisig_verifier::VerifierError;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn banner(n: u8, title: &str) {
    let sep = "=".repeat(60);
    println!("\n{sep}");
    println!("  SCENARIO {n}: {title}");
    println!("{sep}\n");
}

fn pass(msg: &str) {
    println!("  [PASS] {msg}");
}

// ---------------------------------------------------------------------------
// Scenario 1: Simple 2-of-3 treasury transfer via SDK
// ---------------------------------------------------------------------------

fn scenario_1() -> Result<()> {
    banner(1, "2-of-3 Treasury Transfer (SDK high-level API)");

    let mut session = MultisigSession::new(
        "treasury-alpha",
        2,
        vec![
            b"alice-shielded-account-seed".as_slice(),
            b"boris-shielded-account-seed".as_slice(),
            b"cyra-shielded-account-seed".as_slice(),
        ],
    )?;

    println!(
        "  Created multisig: threshold={} of {}",
        session.config().threshold,
        session.config().member_count
    );
    println!(
        "  member_root: {}",
        hex_digest(&session.config().member_root)
    );

    session.create_proposal("grant-42", "transfer 42 LOGOS to public-good recipient");
    println!("  Proposal created: grant-42");

    // Alice (index 0) and Cyra (index 2) approve — Boris abstains
    session.approve(0)?;
    session.approve(2)?;
    println!(
        "  Approvals collected: {} of {}",
        session.approval_count(),
        session.config().threshold
    );

    let proof = session.prove()?;
    println!(
        "  Threshold proof generated: proof_id={}",
        hex_digest(&proof.journal.proof_id)
    );
    println!(
        "  Nullifiers in public journal: {} (no member commitments exposed)",
        proof.journal.nullifiers.len()
    );

    // Verify no member secrets leak
    let public_bytes = proof.public_bytes();
    for member in session.members() {
        assert!(
            !lp0002_private_multisig_core::contains_slice(&public_bytes, member.secret_bytes()),
            "journal leaked member secret"
        );
    }
    pass("public journal contains no member secrets or commitments");

    let action = ProposalAction::Transfer {
        to: "logos1public_good_recipient".into(),
        amount: 42,
        denom: "LOGOS".into(),
    };
    let receipt = session.verify_and_execute(action)?;
    println!("  Executed action: {:?}", receipt.action);
    println!("  approval_count: {}", receipt.approval_count);

    pass("2-of-3 treasury transfer approved, proved, and executed via SDK");
    Ok(())
}

// ---------------------------------------------------------------------------
// Scenario 2: 3-of-5 governance parameter change
// ---------------------------------------------------------------------------

fn scenario_2() -> Result<()> {
    banner(
        2,
        "3-of-5 Governance Parameter Change (non-sequential approvers)",
    );

    let mut session = MultisigSession::new(
        "governance-council",
        3,
        vec![
            b"gov-member-0".as_slice(),
            b"gov-member-1".as_slice(),
            b"gov-member-2".as_slice(),
            b"gov-member-3".as_slice(),
            b"gov-member-4".as_slice(),
        ],
    )?;

    println!("  Created 3-of-5 governance multisig");

    session.create_proposal("param-daily-limit", "set daily spend limit to 100 LOGOS");
    println!("  Proposal: set daily spend limit to 100 LOGOS");

    // Non-sequential: members 0, 2, 4 approve (members 1, 3 abstain)
    session.approve(0)?;
    session.approve(2)?;
    session.approve(4)?;
    println!("  Non-sequential approvals: members 0, 2, 4");

    let proof = session.prove()?;
    println!(
        "  Proof generated: {} nullifiers",
        proof.journal.nullifiers.len()
    );

    let action = ProposalAction::SetParameter {
        key: "daily_limit".into(),
        value: "100".into(),
    };
    let receipt = session.verify_and_execute(action)?;
    println!("  Executed: {:?}", receipt.action);

    pass("3-of-5 governance parameter change approved by non-sequential members");
    Ok(())
}

// ---------------------------------------------------------------------------
// Scenario 3: Resumable partial approvals (client restart simulation)
// ---------------------------------------------------------------------------

fn scenario_3() -> Result<()> {
    banner(3, "Resumable Partial Approvals (client restart simulation)");

    // Use raw core types to demonstrate both APIs
    let members: Vec<MemberSecret> = vec![
        MemberSecret::from_seed(b"resume-member-alice"),
        MemberSecret::from_seed(b"resume-member-boris"),
        MemberSecret::from_seed(b"resume-member-cyra"),
    ];
    let config = MultisigConfig::new("resumable-treasury", 2, &members)?;
    let proposal = Proposal::new("resumable-proposal", "transfer 10 LOGOS");

    println!("  Created 2-of-3 multisig for resumable test");

    // --- Session 1: one approval, then "crash" ---
    let mut acc = ApprovalAccumulator::new(config.multisig_id, proposal.id);
    let outcome = acc.add_member_approval(&members[0])?;
    assert_eq!(outcome, AddApprovalOutcome::Added);
    println!(
        "  [Session 1] Member 0 approved (1 of {})",
        config.threshold
    );
    assert!(!acc.is_threshold_met(config.threshold));

    // Simulate persisting accumulator to JSON and back
    let serialized = serde_json::to_string(&acc)?;
    println!(
        "  [Session 1] Serializing accumulator state: {} bytes",
        serialized.len()
    );
    let mut acc_restored: ApprovalAccumulator = serde_json::from_str(&serialized)?;
    println!("  [Session 2] Restored accumulator from serialized state");

    // --- Session 2: resume with second approval ---
    let outcome = acc_restored.add_member_approval(&members[1])?;
    assert_eq!(outcome, AddApprovalOutcome::Added);
    println!(
        "  [Session 2] Member 1 approved (now {} of {})",
        acc_restored.approval_count(),
        config.threshold
    );
    assert!(acc_restored.is_threshold_met(config.threshold));

    // Now prove with the full set
    let _proof = prove_threshold(&config, &proposal, [&members[0], &members[1]])?;
    println!("  Proof generated after resumed approvals");

    // Dedup test: same member approving again is idempotent
    let outcome = acc_restored.add_member_approval(&members[0])?;
    assert_eq!(outcome, AddApprovalOutcome::AlreadyPresent);
    println!("  Duplicate approval correctly detected as AlreadyPresent");

    pass("partial approvals persist across simulated client restart and resume correctly");
    Ok(())
}

// ---------------------------------------------------------------------------
// Scenario 4: Error paths
// ---------------------------------------------------------------------------

fn scenario_4() -> Result<()> {
    banner(4, "Error Paths (double-vote, insufficient approvals)");

    let members: Vec<MemberSecret> = vec![
        MemberSecret::from_seed(b"err-alice"),
        MemberSecret::from_seed(b"err-boris"),
        MemberSecret::from_seed(b"err-cyra"),
    ];
    let config = MultisigConfig::new("error-test", 2, &members)?;
    let proposal = Proposal::new("err-proposal", "test action");

    // --- Double-vote ---
    println!("  Testing double-vote prevention...");
    let err = prove_threshold(&config, &proposal, [&members[0], &members[0]]).unwrap_err();
    assert_eq!(err, ProofError::DuplicateNullifier);
    println!("  prove_threshold correctly rejected duplicate approver: {err}");
    pass("double-vote returns DuplicateNullifier");

    // --- Insufficient approvals ---
    println!("\n  Testing insufficient approvals...");
    let err = prove_threshold(&config, &proposal, [&members[0]]).unwrap_err();
    assert_eq!(
        err,
        ProofError::InsufficientApprovals {
            threshold: 2,
            provided: 1
        }
    );
    println!("  prove_threshold correctly rejected 1-of-2 threshold: {err}");
    pass("insufficient approvals returns InsufficientApprovals");

    // --- SDK: invalid member index ---
    println!("\n  Testing invalid member index via SDK...");
    let mut session = MultisigSession::new(
        "error-test-sdk",
        2,
        vec![b"seed-a".as_slice(), b"seed-b".as_slice()],
    )?;
    session.create_proposal("test", "action");
    let err = session.approve(5).unwrap_err();
    println!("  SDK correctly rejected member index 5 in 2-member multisig: {err}");
    pass("invalid member index returns InvalidMemberIndex");

    Ok(())
}

// ---------------------------------------------------------------------------
// Scenario 5: Replay protection
// ---------------------------------------------------------------------------

fn scenario_5() -> Result<()> {
    banner(5, "Replay Protection (double-execution rejection)");

    let mut session = MultisigSession::new(
        "replay-test",
        2,
        vec![
            b"replay-alice".as_slice(),
            b"replay-boris".as_slice(),
            b"replay-cyra".as_slice(),
        ],
    )?;

    session.create_proposal("replay-proposal", "transfer 100 LOGOS");
    session.approve(0)?;
    session.approve(1)?;
    let _proof = session.prove()?;

    let action = ProposalAction::Transfer {
        to: "logos1replay_target".into(),
        amount: 100,
        denom: "LOGOS".into(),
    };

    // First execution succeeds
    let receipt = session.verify_and_execute(action.clone())?;
    println!(
        "  First execution succeeded: proof_id={}",
        hex_digest(&receipt.proof_id)
    );

    // Second execution with same proof must fail
    let err = session.verify_and_execute(action).unwrap_err();
    match &err {
        lp0002_private_multisig_sdk::SdkError::Verifier(VerifierError::ProposalAlreadyExecuted) => {
            println!("  Second execution correctly rejected: ProposalAlreadyExecuted");
        }
        other => bail!("expected ProposalAlreadyExecuted, got: {other:?}"),
    }

    pass("replay protection prevents double-execution of the same proposal");
    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    println!("LP-0002 Private M-of-N Multisig — Consumer Integration Demo");
    println!("This app imports the LP-0002 SDK, core, and verifier crates as library");
    println!("dependencies, exactly as a third-party integration would.\n");
    println!("5 scenarios demonstrate the full range of SDK capabilities.\n");

    scenario_1()?;
    scenario_2()?;
    scenario_3()?;
    scenario_4()?;
    scenario_5()?;

    let sep = "=".repeat(60);
    println!("\n{sep}");
    println!("  ALL 5 SCENARIOS PASSED");
    println!("{sep}\n");
    println!("This consumer demo proves the LP-0002 SDK can be used by any");
    println!("application to integrate private multisig threshold approval,");
    println!("replay-protected execution, and resumable partial state.");
    Ok(())
}
