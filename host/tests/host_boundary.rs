use lp0002_private_multisig_core::prove_threshold_guest;
use lp0002_private_multisig_host::{
    build_guest_input_bytes, decode_threshold_journal_bytes, fixture_guest_input,
    threshold_proof_image_id_bytes, Risc0ReceiptVerifier,
};
use lp0002_private_multisig_lez_program::{LezProgramError, ReceiptVerifier};

#[test]
fn fixture_guest_input_matches_shared_relation() {
    let input = fixture_guest_input();
    let proof = prove_threshold_guest(&input).unwrap();
    let journal_bytes = borsh::to_vec(&proof.journal).unwrap();
    let decoded = decode_threshold_journal_bytes(&journal_bytes).unwrap();
    assert_eq!(decoded, proof.journal);
}

#[test]
fn guest_input_boundary_is_borsh_bytes() {
    let input = fixture_guest_input();
    let bytes = build_guest_input_bytes(&input).unwrap();
    let decoded: lp0002_private_multisig_core::ThresholdGuestInput =
        borsh::from_slice(&bytes).unwrap();
    assert_eq!(decoded, input);
}

#[test]
fn risc0_receipt_verifier_rejects_wrong_image_before_decoding_receipt() {
    let input = fixture_guest_input();
    let proof = prove_threshold_guest(&input).unwrap();
    let mut wrong_image = threshold_proof_image_id_bytes();
    wrong_image[0] ^= 0xFF;

    let err = Risc0ReceiptVerifier
        .verify_receipt(wrong_image, b"not a real receipt", &proof.journal)
        .expect_err("wrong image id is rejected first");

    assert_eq!(err, LezProgramError::Risc0ImageIdMismatch);
}

#[test]
fn risc0_receipt_verifier_rejects_malformed_receipt_for_expected_image() {
    let input = fixture_guest_input();
    let proof = prove_threshold_guest(&input).unwrap();

    let err = Risc0ReceiptVerifier
        .verify_receipt(
            threshold_proof_image_id_bytes(),
            b"not a real receipt",
            &proof.journal,
        )
        .expect_err("malformed receipt rejected");

    assert!(matches!(err, LezProgramError::ReceiptVerification(_)));
}
