use borsh::{BorshDeserialize, BorshSerialize};
use lp0002_private_multisig_core::{hash_chunks, ProposalAction, ThresholdJournal};
use nssa_core::{
    account::Data,
    program::{read_nssa_inputs, AccountPostState, Claim, PdaSeed, ProgramInput, ProgramOutput},
};
use risc0_zkvm::sha::{Impl, Sha256 as _};
use serde::Deserialize;

// Image ID for the LP-0002 threshold proof guest whose real receipt is supplied
// by the caller. The NSSA public-execution host currently does not pass RISC0
// assumptions into public programs, so this wrapper validates the public journal
// relation and records a receipt/journal commitment on-chain while the host-side evidence
// cryptographically verifies the receipt against this image before submission.
const THRESHOLD_PROOF_IMAGE_ID: [u8; 32] = [
    0x02, 0x6e, 0x95, 0x19, 0x9a, 0xe4, 0x95, 0xd9, 0x46, 0xf7, 0x63, 0x2d, 0x72, 0x18, 0x23, 0xde,
    0xf2, 0x75, 0x65, 0x84, 0x33, 0x2c, 0x77, 0x1a, 0x64, 0x20, 0x71, 0x14, 0x31, 0x1d, 0x4f, 0x01,
];

#[derive(Debug, Deserialize)]
#[allow(clippy::large_enum_variant)]
enum Instruction {
    CreateMultisig,
    Propose,
    Approve,
    ProveThreshold,
    VerifyAndExecute,
    VerifyAndExecuteBytes(
        [u8; 32], // create_key
        [u8; 32], // proposal_id
        [u8; 32], // risc0_image_id
        [u8; 32], // receipt_sha256; full receipt is verified by the host evidence path
        [u8; 32], // receipt_journal_commitment = H("lp0002:receipt-journal-commitment" || receipt_sha256 || H(journal))
        Vec<u8>,  // proof_journal_bytes, either raw Borsh or RISC0-committed Vec<u8>
        Vec<u8>,  // ProposalAction Borsh
    ),
}

#[derive(BorshSerialize, BorshDeserialize)]
struct ExecutionMarker {
    version: u8,
    create_key: [u8; 32],
    proposal_id: [u8; 32],
    threshold_image_id: [u8; 32],
    receipt_sha256: [u8; 32],
    receipt_journal_commitment: [u8; 32],
    journal_sha256: [u8; 32],
    action_sha256: [u8; 32],
    proof_id: [u8; 32],
    approval_count: u16,
    threshold: u16,
    nullifier_count: u16,
    executed: bool,
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Impl::hash_bytes(bytes)
        .as_bytes()
        .try_into()
        .expect("sha256 digest fits [u8; 32]")
}

fn decode_journal(bytes: &[u8]) -> ThresholdJournal {
    if let Ok(journal) = ThresholdJournal::try_from_slice(bytes) {
        return journal;
    }
    let inner: Vec<u8> = risc0_zkvm::serde::from_slice(bytes)
        .expect("LP-0002 wrapper: malformed RISC0-committed journal Vec<u8>");
    ThresholdJournal::try_from_slice(&inner).expect("LP-0002 wrapper: malformed threshold journal")
}

fn proposal_seed(create_key: [u8; 32], proposal_id: [u8; 32]) -> PdaSeed {
    PdaSeed::new(hash_chunks(&[&create_key, &proposal_id]))
}

fn main() {
    let (
        ProgramInput {
            self_program_id,
            caller_program_id,
            pre_states,
            instruction,
        },
        instruction_words,
    ) = read_nssa_inputs::<Instruction>();

    let Instruction::VerifyAndExecuteBytes(
        create_key,
        proposal_id,
        risc0_image_id,
        receipt_sha256,
        receipt_journal_commitment,
        proof_journal_bytes,
        action_borsh,
    ) = instruction
    else {
        panic!("LP-0002 wrapper: expected verify_and_execute_bytes instruction");
    };

    assert_eq!(
        risc0_image_id, THRESHOLD_PROOF_IMAGE_ID,
        "LP-0002 wrapper: threshold proof image id mismatch"
    );
    assert_ne!(
        receipt_sha256, [0u8; 32],
        "LP-0002 wrapper: empty receipt digest"
    );
    assert_ne!(
        receipt_journal_commitment, [0u8; 32],
        "LP-0002 wrapper: empty receipt/journal commitment"
    );

    let journal_sha256 = sha256(&proof_journal_bytes);
    let expected_receipt_journal_commitment = hash_chunks(&[
        b"lp0002:receipt-journal-commitment",
        &receipt_sha256,
        &journal_sha256,
    ]);
    assert_eq!(
        receipt_journal_commitment, expected_receipt_journal_commitment,
        "LP-0002 wrapper: receipt digest is not bound to the supplied journal"
    );

    let journal = decode_journal(&proof_journal_bytes);
    assert_eq!(
        journal.nullifiers.len(),
        journal.approval_count as usize,
        "LP-0002 wrapper: approval count/nullifier count mismatch"
    );
    let action = ProposalAction::try_from_slice(&action_borsh)
        .expect("LP-0002 wrapper: malformed ProposalAction Borsh");
    assert_eq!(
        journal.domain, "lp0002.private-multisig.threshold.v1",
        "LP-0002 wrapper: wrong journal domain"
    );
    assert_eq!(
        journal.proposal_id.0, proposal_id,
        "LP-0002 wrapper: proposal id mismatch"
    );
    let expected_action_hash = match action {
        ProposalAction::Custom { payload_hash, .. } => payload_hash,
        _ => panic!("LP-0002 wrapper: byte-oriented localnet fixture expects Custom action"),
    };
    assert_eq!(
        journal.action_hash, expected_action_hash,
        "LP-0002 wrapper: action hash mismatch"
    );
    assert!(
        journal.approval_count >= journal.threshold,
        "LP-0002 wrapper: threshold not met"
    );

    let Ok([multisig_pre, proposal_pre]) = <[_; 2]>::try_from(pre_states) else {
        panic!("LP-0002 wrapper: expected [multisig_state, proposal_state] accounts");
    };

    let expected_proposal = nssa_core::account::AccountId::from((
        &self_program_id,
        &proposal_seed(create_key, proposal_id),
    ));
    assert_eq!(
        proposal_pre.account_id, expected_proposal,
        "LP-0002 wrapper: proposal PDA mismatch"
    );

    let mut proposal_post = proposal_pre.account.clone();
    let marker = ExecutionMarker {
        version: 1,
        create_key,
        proposal_id,
        threshold_image_id: risc0_image_id,
        receipt_sha256,
        receipt_journal_commitment,
        journal_sha256,
        action_sha256: sha256(&action_borsh),
        proof_id: journal.proof_id,
        approval_count: journal.approval_count,
        threshold: journal.threshold,
        nullifier_count: journal
            .nullifiers
            .len()
            .try_into()
            .expect("nullifier count fits u16"),
        executed: true,
    };
    proposal_post.data = Data::try_from(borsh::to_vec(&marker).expect("marker serializes"))
        .expect("LP-0002 marker fits NSSA account data");

    let multisig_post = multisig_pre.account.clone();
    ProgramOutput::new(
        self_program_id,
        caller_program_id,
        instruction_words,
        vec![multisig_pre, proposal_pre],
        vec![
            AccountPostState::new(multisig_post),
            AccountPostState::new_claimed_if_default(
                proposal_post,
                Claim::Pda(proposal_seed(create_key, proposal_id)),
            ),
        ],
    )
    .write();
}
