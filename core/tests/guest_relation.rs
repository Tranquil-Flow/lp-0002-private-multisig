use lp0002_private_multisig_core::*;

fn fixture() -> (MultisigConfig, Vec<MemberSecret>, Proposal) {
    let members = vec![
        MemberSecret::from_seed(b"guest alice shielded secret"),
        MemberSecret::from_seed(b"guest boris shielded secret"),
        MemberSecret::from_seed(b"guest cyra shielded secret"),
    ];
    let config = MultisigConfig::new("guest-treasury", 2, &members).unwrap();
    let proposal = Proposal::new("guest-proposal", "rotate treasury guardian set");
    (config, members, proposal)
}

#[test]
fn guest_relation_accepts_hidden_members_in_private_set() {
    let (config, members, proposal) = fixture();
    let input = ThresholdGuestInput::new(
        config.clone(),
        proposal.clone(),
        &members,
        vec![members[0].clone(), members[2].clone()],
    );

    let proof = prove_threshold_guest(&input).unwrap();

    verify_threshold_receipt(&config, &proposal, &proof).unwrap();
    assert_eq!(proof.journal.member_root, config.member_root);
    assert_eq!(proof.journal.approval_count, 2);
}

#[test]
fn guest_relation_rejects_member_set_that_does_not_match_public_root() {
    let (config, members, proposal) = fixture();
    let outsider = MemberSecret::from_seed(b"outsider not in multisig");
    let input = ThresholdGuestInput {
        config,
        proposal,
        member_commitments: vec![members[0].commitment(), outsider.commitment()],
        approving_members: vec![members[0].clone(), members[1].clone()],
    };

    let err = prove_threshold_guest(&input).unwrap_err();
    assert_eq!(err, ProofError::MemberCommitmentSetMismatch);
}

#[test]
fn guest_relation_rejects_approver_outside_private_member_set() {
    let (config, members, proposal) = fixture();
    let outsider = MemberSecret::from_seed(b"outsider not in multisig");
    let input = ThresholdGuestInput::new(
        config,
        proposal,
        &members,
        vec![members[0].clone(), outsider],
    );

    let err = prove_threshold_guest(&input).unwrap_err();
    assert_eq!(err, ProofError::UnknownApprovingMember);
}
