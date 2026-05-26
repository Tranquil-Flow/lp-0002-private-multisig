use lp0002_private_multisig_core::{
    prove_threshold_guest, Digest32, MemberSecret, MultisigConfig, Proposal, ProposalAction,
    ThresholdGuestInput,
};
use lp0002_private_multisig_lez_program::{
    execute_proposal, ExecuteProposalInstruction, ExpectedImageVerifier, LezProgramError,
    MultisigState, ProposalState,
};

fn fixture() -> (
    MultisigState,
    ProposalState,
    ExecuteProposalInstruction,
    ExpectedImageVerifier,
) {
    let members = vec![
        MemberSecret::from_seed(b"lez alice shielded secret"),
        MemberSecret::from_seed(b"lez boris shielded secret"),
        MemberSecret::from_seed(b"lez cyra shielded secret"),
    ];
    let config = MultisigConfig::new("lez-wrapper-treasury", 2, &members).unwrap();
    let proposal = Proposal::new("lez-wrapper-proposal", "pay grant recipient 42 LOGOS");
    let input = ThresholdGuestInput::new(
        config.clone(),
        proposal.clone(),
        &members,
        vec![members[0].clone(), members[2].clone()],
    );
    let proof = prove_threshold_guest(&input).unwrap();
    let image_id: Digest32 = [0x42; 32];
    let instruction = ExecuteProposalInstruction {
        create_key: [0xA5; 32],
        proposal_id: proposal.id,
        risc0_image_id: image_id,
        receipt_bytes: b"fake-risc0-receipt-bytes-for-unit-test".to_vec(),
        proof_journal: proof.journal,
        action: ProposalAction::Transfer {
            to: "logos1grantrecipient".into(),
            amount: 42,
            denom: "LOGOS".into(),
        },
    };
    let multisig_state = MultisigState::from_config(instruction.create_key, &config, 255);
    let proposal_state = ProposalState::from_proposal(&proposal, 254);
    let verifier = ExpectedImageVerifier {
        expected_image_id: image_id,
    };
    (multisig_state, proposal_state, instruction, verifier)
}

#[test]
fn execute_proposal_updates_state_and_returns_receipt() {
    let (multisig_state, mut proposal_state, instruction, verifier) = fixture();

    let receipt = execute_proposal(&multisig_state, &mut proposal_state, instruction, &verifier)
        .expect("threshold execution succeeds");

    assert!(proposal_state.executed);
    assert_eq!(proposal_state.approval_count, 2);
    assert_eq!(proposal_state.nullifiers.len(), 2);
    assert_eq!(receipt.approval_count, 2);
    assert_eq!(receipt.proposal_id, proposal_state.proposal_id);
}

#[test]
fn execute_proposal_rejects_replay() {
    let (multisig_state, mut proposal_state, instruction, verifier) = fixture();

    execute_proposal(
        &multisig_state,
        &mut proposal_state,
        instruction.clone(),
        &verifier,
    )
    .unwrap();
    let err = execute_proposal(&multisig_state, &mut proposal_state, instruction, &verifier)
        .expect_err("second execution is replay");

    assert_eq!(err, LezProgramError::ProposalAlreadyExecuted);
}

#[test]
fn execute_proposal_rejects_wrong_image_id() {
    let (multisig_state, mut proposal_state, mut instruction, verifier) = fixture();
    instruction.risc0_image_id = [0x99; 32];

    let err = execute_proposal(&multisig_state, &mut proposal_state, instruction, &verifier)
        .expect_err("image mismatch rejected");

    assert_eq!(err, LezProgramError::Risc0ImageIdMismatch);
    assert!(!proposal_state.executed);
}

#[test]
fn execute_instruction_borsh_round_trips_for_lez_payload() {
    let (_, _, instruction, _) = fixture();

    let bytes = instruction.to_borsh().unwrap();
    let decoded = ExecuteProposalInstruction::from_borsh(&bytes).unwrap();

    assert_eq!(decoded, instruction);
}
