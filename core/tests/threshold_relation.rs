use lp0002_private_multisig_core::*;
use serde_json;

fn fixture() -> (MultisigConfig, Vec<MemberSecret>, Proposal) {
    let members = vec![
        MemberSecret::from_seed(b"member alice shielded secret"),
        MemberSecret::from_seed(b"member boris shielded secret"),
        MemberSecret::from_seed(b"member cyra shielded secret"),
    ];
    let config = MultisigConfig::new("treasury-demo", 2, &members).unwrap();
    let proposal = Proposal::new(
        "proposal-pay-grant",
        "transfer 42 LOGOS to public grant recipient",
    );
    (config, members, proposal)
}

#[test]
fn threshold_proof_verifies_without_revealing_member_commitments_or_secrets() {
    let (config, members, proposal) = fixture();
    let proof = prove_threshold(&config, &proposal, [&members[0], &members[2]]).unwrap();

    verify_threshold_receipt(&config, &proposal, &proof).unwrap();
    assert_eq!(proof.journal.approval_count, 2);
    assert_eq!(proof.journal.threshold, 2);
    assert_eq!(proof.journal.member_count, 3);

    let public_bytes = proof.public_bytes();
    for member in &members {
        assert!(
            !contains_slice(&public_bytes, member.secret_bytes()),
            "journal leaked raw shielded member secret"
        );
        assert!(
            !contains_slice(&public_bytes, &member.commitment()),
            "journal leaked shielded member commitment"
        );
    }
}

#[test]
fn tampered_approval_count_must_match_nullifier_count() {
    let (config, members, proposal) = fixture();
    let mut proof = prove_threshold(&config, &proposal, [&members[0], &members[1]]).unwrap();
    proof.journal.approval_count += 1;

    let err = verify_threshold_receipt(&config, &proposal, &proof).unwrap_err();
    assert_eq!(err, ProofError::ApprovalCountNullifierMismatch);
}

#[test]
fn duplicate_approver_is_rejected_before_a_receipt_is_created() {
    let (config, members, proposal) = fixture();
    let err = prove_threshold(&config, &proposal, [&members[1], &members[1]]).unwrap_err();
    assert_eq!(err, ProofError::DuplicateNullifier);
}

#[test]
fn fewer_than_threshold_approvals_do_not_produce_a_receipt() {
    let (config, members, proposal) = fixture();
    let err = prove_threshold(&config, &proposal, [&members[0]]).unwrap_err();
    assert_eq!(
        err,
        ProofError::InsufficientApprovals {
            threshold: 2,
            provided: 1
        }
    );
}

#[test]
fn receipt_is_bound_to_the_exact_proposal_context() {
    let (config, members, proposal) = fixture();
    let proof = prove_threshold(&config, &proposal, [&members[0], &members[1]]).unwrap();
    let tampered = Proposal::new("proposal-pay-attacker", "transfer 42 LOGOS to attacker");
    let err = verify_threshold_receipt(&config, &tampered, &proof).unwrap_err();
    assert_eq!(err, ProofError::ProposalMismatch);
}

#[test]
fn partial_approval_set_is_resumable_and_deduplicated() {
    let (config, members, proposal) = fixture();
    let mut acc = ApprovalAccumulator::new(config.multisig_id, proposal.id);
    assert_eq!(acc.approval_count(), 0);
    assert_eq!(
        acc.add_member_approval(&members[0]).unwrap(),
        AddApprovalOutcome::Added
    );
    assert_eq!(
        acc.add_member_approval(&members[0]).unwrap(),
        AddApprovalOutcome::AlreadyPresent
    );
    assert_eq!(acc.approval_count(), 1);
    assert!(!acc.is_threshold_met(config.threshold));
    assert_eq!(
        acc.add_member_approval(&members[2]).unwrap(),
        AddApprovalOutcome::Added
    );
    assert!(acc.is_threshold_met(config.threshold));
}

// ── edge-case & integration tests ──────────────────────────────────────

#[test]
fn test_large_threshold_10_of_20() {
    // Create 20 members, threshold 10
    let seeds: Vec<Vec<u8>> = (0..20)
        .map(|i| format!("member-{:02}-shielded-secret", i).into_bytes())
        .collect();
    let members: Vec<MemberSecret> = seeds.iter().map(|s| MemberSecret::from_seed(s)).collect();
    let config = MultisigConfig::new("large-treasury", 10, &members).unwrap();

    let proposal = Proposal::new("mega-transfer", "transfer 5000 LOGOS to bridge contract");

    // 10 non-sequential members approve (0,2,4,6,8,10,12,14,16,18)
    let approvers: Vec<&MemberSecret> = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18]
        .iter()
        .map(|&i| &members[i])
        .collect();

    let proof = prove_threshold(&config, &proposal, approvers).unwrap();
    verify_threshold_receipt(&config, &proposal, &proof).unwrap();

    assert_eq!(proof.journal.nullifiers.len(), 10);
    assert_eq!(proof.journal.approval_count, 10);
    assert_eq!(proof.journal.threshold, 10);
    assert_eq!(proof.journal.member_count, 20);

    // Verify all 10 nullifiers are unique
    let mut seen = std::collections::BTreeSet::new();
    for n in &proof.journal.nullifiers {
        assert!(seen.insert(*n), "duplicate nullifier in journal");
    }
}

#[test]
fn test_all_members_approve_1_of_1() {
    let members = vec![MemberSecret::from_seed(b"sole member secret")];
    let config = MultisigConfig::new("solo-treasury", 1, &members).unwrap();
    let proposal = Proposal::new("solo-action", "single signer transfer");

    let proof = prove_threshold(&config, &proposal, [&members[0]]).unwrap();
    verify_threshold_receipt(&config, &proposal, &proof).unwrap();

    assert_eq!(proof.journal.approval_count, 1);
    assert_eq!(proof.journal.threshold, 1);
    assert_eq!(proof.journal.member_count, 1);
    assert_eq!(proof.journal.nullifiers.len(), 1);
}

#[test]
fn test_maximum_threshold_equals_member_count() {
    let members = vec![
        MemberSecret::from_seed(b"triple a secret"),
        MemberSecret::from_seed(b"triple b secret"),
        MemberSecret::from_seed(b"triple c secret"),
    ];
    let config = MultisigConfig::new("unanimous-treasury", 3, &members).unwrap();
    let proposal = Proposal::new("unanimous-action", "requires all three signers");

    let proof =
        prove_threshold(&config, &proposal, [&members[0], &members[1], &members[2]]).unwrap();
    verify_threshold_receipt(&config, &proposal, &proof).unwrap();

    assert_eq!(proof.journal.approval_count, 3);
    assert_eq!(proof.journal.threshold, 3);
    assert_eq!(proof.journal.member_count, 3);

    // With threshold == member_count, missing even one should fail
    let err = prove_threshold(&config, &proposal, [&members[0], &members[1]]).unwrap_err();
    assert_eq!(
        err,
        ProofError::InsufficientApprovals {
            threshold: 3,
            provided: 2,
        }
    );
}

#[test]
fn test_wrong_multisig_cannot_verify_proof() {
    // Multisig A
    let members_a = vec![
        MemberSecret::from_seed(b"alpha member 1"),
        MemberSecret::from_seed(b"alpha member 2"),
        MemberSecret::from_seed(b"alpha member 3"),
    ];
    let config_a = MultisigConfig::new("alpha-treasury", 2, &members_a).unwrap();

    // Multisig B — same threshold, different members, different label
    let members_b = vec![
        MemberSecret::from_seed(b"bravo member 1"),
        MemberSecret::from_seed(b"bravo member 2"),
        MemberSecret::from_seed(b"bravo member 3"),
    ];
    let config_b = MultisigConfig::new("bravo-treasury", 2, &members_b).unwrap();

    // Prove on multisig A
    let proposal = Proposal::new("test-proposal", "do something");
    let proof = prove_threshold(&config_a, &proposal, [&members_a[0], &members_a[1]]).unwrap();

    // Verify against A's config works
    verify_threshold_receipt(&config_a, &proposal, &proof).unwrap();

    // Verify against B's config — reject (MultisigIdMismatch)
    let err = verify_threshold_receipt(&config_b, &proposal, &proof).unwrap_err();
    assert_eq!(err, ProofError::MultisigIdMismatch);
}

#[test]
fn test_proposal_binding_prevents_cross_proposal_replay() {
    let (config, members, _proposal) = fixture();

    // Two different proposals for the same multisig
    let proposal_1 = Proposal::new("grant-42", "transfer 42 LOGOS to recipient A");
    let proposal_2 = Proposal::new("grant-99", "transfer 99 LOGOS to recipient B");

    let proof = prove_threshold(&config, &proposal_1, [&members[0], &members[1]]).unwrap();

    verify_threshold_receipt(&config, &proposal_1, &proof).unwrap();

    let err = verify_threshold_receipt(&config, &proposal_2, &proof).unwrap_err();
    assert_eq!(err, ProofError::ProposalMismatch);
}

#[test]
fn test_accumulator_serialization_roundtrip() {
    let (config, members, proposal) = fixture();

    // Create accumulator and add 2 approvals
    let mut acc = ApprovalAccumulator::new(config.multisig_id, proposal.id);
    assert_eq!(
        acc.add_member_approval(&members[0]).unwrap(),
        AddApprovalOutcome::Added
    );
    assert_eq!(
        acc.add_member_approval(&members[1]).unwrap(),
        AddApprovalOutcome::Added
    );
    assert_eq!(acc.approval_count(), 2);

    // Serialize to JSON
    let json = serde_json::to_string(&acc).unwrap();

    // Deserialize
    let mut deserialized: ApprovalAccumulator = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.approval_count(), 2);
    assert_eq!(deserialized.multisig_id, acc.multisig_id);
    assert_eq!(deserialized.proposal_id, acc.proposal_id);

    // Add another approval after deserialization
    assert_eq!(
        deserialized.add_member_approval(&members[2]).unwrap(),
        AddApprovalOutcome::Added
    );
    assert_eq!(deserialized.approval_count(), 3);

    // Verify that re-adding member[0] is still detected as duplicate
    assert_eq!(
        deserialized.add_member_approval(&members[0]).unwrap(),
        AddApprovalOutcome::AlreadyPresent
    );
    assert_eq!(deserialized.approval_count(), 3);
}
