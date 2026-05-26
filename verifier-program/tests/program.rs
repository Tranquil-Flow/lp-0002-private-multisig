use lp0002_private_multisig_core::*;
use lp0002_private_multisig_verifier::*;

fn fixture() -> (MultisigConfig, Vec<MemberSecret>, Proposal, ProposalAction) {
    let members = vec![
        MemberSecret::from_seed(b"member alice shielded secret"),
        MemberSecret::from_seed(b"member boris shielded secret"),
        MemberSecret::from_seed(b"member cyra shielded secret"),
    ];
    let config = MultisigConfig::new("treasury-demo", 2, &members).unwrap();
    let proposal = Proposal::new("proposal-set-limit", "set daily spend limit to 100 LOGOS");
    let action = ProposalAction::SetParameter {
        key: "daily_limit".into(),
        value: "100".into(),
    };
    (config, members, proposal, action)
}

#[test]
fn verifier_executes_a_threshold_gated_action_once() {
    let (config, members, proposal, action) = fixture();
    let proof = prove_threshold(&config, &proposal, [&members[0], &members[2]]).unwrap();
    let mut program = VerifierProgram::default();

    let receipt = program
        .execute_if_threshold_met(&config, &proposal, &proof, action.clone())
        .unwrap();
    assert_eq!(receipt.proposal_id, proposal.id);
    assert_eq!(receipt.action, action);
    assert_eq!(receipt.approval_count, 2);

    let err = program
        .execute_if_threshold_met(&config, &proposal, &proof, action)
        .unwrap_err();
    assert_eq!(err, VerifierError::ProposalAlreadyExecuted);
}

#[test]
fn verifier_rejects_a_receipt_for_the_wrong_multisig_root() {
    let (config, members, proposal, action) = fixture();
    let proof = prove_threshold(&config, &proposal, [&members[0], &members[1]]).unwrap();
    let wrong_members = vec![
        MemberSecret::from_seed(b"different alice"),
        MemberSecret::from_seed(b"different boris"),
        MemberSecret::from_seed(b"different cyra"),
    ];
    let mut wrong_config = MultisigConfig::new("treasury-demo", 2, &wrong_members).unwrap();
    wrong_config.multisig_id = config.multisig_id;
    let mut program = VerifierProgram::default();

    let err = program
        .execute_if_threshold_met(&wrong_config, &proposal, &proof, action)
        .unwrap_err();
    assert_eq!(
        err,
        VerifierError::InvalidProof(ProofError::MultisigRootMismatch)
    );
}

// ── edge-case & integration tests ──────────────────────────────────────

#[test]
fn test_verifier_rejects_proof_with_wrong_threshold() {
    // Create 3-of-5 config
    let seeds: Vec<&[u8]> = vec![
        b"voter-a-seed",
        b"voter-b-seed",
        b"voter-c-seed",
        b"voter-d-seed",
        b"voter-e-seed",
    ];
    let members: Vec<MemberSecret> = seeds.iter().map(|s| MemberSecret::from_seed(s)).collect();
    let config = MultisigConfig::new("five-member-treasury", 3, &members).unwrap();
    let proposal = Proposal::new("threshold-test", "set limit to 200");
    let action = ProposalAction::SetParameter {
        key: "daily_limit".into(),
        value: "200".into(),
    };

    // Provide exactly 3 approvals (meets threshold) and prove
    let proof =
        prove_threshold(&config, &proposal, [&members[0], &members[1], &members[2]]).unwrap();

    // Verify with correct config works
    let mut program = VerifierProgram::default();
    program
        .execute_if_threshold_met(&config, &proposal, &proof, action.clone())
        .unwrap();

    // Now construct a config where threshold is raised to 4
    // (same members, same label produces different multisig_id because threshold
    //  is part of the multisig_id hash, so we force the old multisig_id + root)
    let mut wrong_config = MultisigConfig::new("five-member-treasury", 4, &members).unwrap();
    wrong_config.multisig_id = config.multisig_id;
    wrong_config.member_root = config.member_root;
    wrong_config.member_count = config.member_count;

    // The verifier should reject: threshold mismatch
    let mut program2 = VerifierProgram::default();
    let err = program2
        .execute_if_threshold_met(&wrong_config, &proposal, &proof, action)
        .unwrap_err();
    assert_eq!(
        err,
        VerifierError::InvalidProof(ProofError::ThresholdMismatch)
    );
}

#[test]
fn test_verifier_handles_multiple_distinct_proposals() {
    let (config, members, _, _action) = fixture();

    // Proposal 1
    let proposal_1 = Proposal::new("grant-1", "transfer 10 LOGOS to A");
    let action_1 = ProposalAction::Transfer {
        to: "A".into(),
        amount: 10,
        denom: "LOGOS".into(),
    };
    let proof_1 = prove_threshold(&config, &proposal_1, [&members[0], &members[1]]).unwrap();

    // Proposal 2
    let proposal_2 = Proposal::new("grant-2", "transfer 20 LOGOS to B");
    let action_2 = ProposalAction::Transfer {
        to: "B".into(),
        amount: 20,
        denom: "LOGOS".into(),
    };
    let proof_2 = prove_threshold(&config, &proposal_2, [&members[1], &members[2]]).unwrap();

    let mut program = VerifierProgram::default();

    // Execute proposal 1
    let receipt_1 = program
        .execute_if_threshold_met(&config, &proposal_1, &proof_1, action_1)
        .unwrap();
    assert_eq!(receipt_1.proposal_id, proposal_1.id);
    assert_eq!(receipt_1.approval_count, 2);

    // Execute proposal 2 — should also succeed (different proposal ID)
    let receipt_2 = program
        .execute_if_threshold_met(&config, &proposal_2, &proof_2, action_2)
        .unwrap();
    assert_eq!(receipt_2.proposal_id, proposal_2.id);
    assert_eq!(receipt_2.approval_count, 2);

    // Replay proposal 1 should now fail
    let err = program
        .execute_if_threshold_met(
            &config,
            &proposal_1,
            &proof_1,
            ProposalAction::Transfer {
                to: "A".into(),
                amount: 10,
                denom: "LOGOS".into(),
            },
        )
        .unwrap_err();
    assert_eq!(err, VerifierError::ProposalAlreadyExecuted);
}

#[test]
fn test_verifier_nullifier_uniqueness_in_journal() {
    let (config, members, proposal, _action) = fixture();

    let proof = prove_threshold(&config, &proposal, [&members[0], &members[2]]).unwrap();

    assert_eq!(proof.journal.nullifiers.len(), 2);
    assert_ne!(
        proof.journal.nullifiers[0], proof.journal.nullifiers[1],
        "nullifiers for different members must be distinct"
    );
}
