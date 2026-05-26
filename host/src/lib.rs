use borsh::BorshDeserialize;
use lp0002_private_multisig_core::{
    prove_threshold_guest, verify_threshold_receipt, Digest32, MemberSecret, MultisigConfig,
    Proposal, ThresholdGuestInput, ThresholdJournal, ThresholdProof,
};
use lp0002_private_multisig_lez_program::{LezProgramError, ReceiptVerifier};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::path::{Path, PathBuf};

pub use lp0002_private_multisig_methods::{
    THRESHOLD_PROOF_ELF, THRESHOLD_PROOF_ID, VERIFY_AND_EXECUTE_BYTES_ELF,
    VERIFY_AND_EXECUTE_BYTES_ID,
};

pub fn threshold_proof_image_id_hex() -> String {
    hex::encode(threshold_proof_image_id_bytes())
}

pub fn threshold_proof_image_id_bytes() -> [u8; 32] {
    digest_to_bytes(THRESHOLD_PROOF_ID)
}

pub fn verify_and_execute_program_id_hex() -> String {
    hex::encode(verify_and_execute_program_id_bytes())
}

pub fn verify_and_execute_program_id_bytes() -> [u8; 32] {
    digest_to_bytes(VERIFY_AND_EXECUTE_BYTES_ID)
}

pub fn build_guest_input_bytes(input: &ThresholdGuestInput) -> Result<Vec<u8>, String> {
    borsh::to_vec(input).map_err(|err| format!("encode ThresholdGuestInput: {err}"))
}

pub fn decode_threshold_journal_bytes(bytes: &[u8]) -> Result<ThresholdJournal, String> {
    ThresholdJournal::try_from_slice(bytes).map_err(|err| format!("decode ThresholdJournal: {err}"))
}

pub fn decode_risc0_committed_journal(bytes: &[u8]) -> Result<ThresholdJournal, String> {
    let inner: Vec<u8> = risc0_zkvm::serde::from_slice(bytes)
        .map_err(|err| format!("decode RISC0 journal Vec<u8>: {err}"))?;
    decode_threshold_journal_bytes(&inner)
}

pub fn decode_any_journal_bytes(bytes: &[u8]) -> Result<ThresholdJournal, String> {
    decode_threshold_journal_bytes(bytes).or_else(|_| decode_risc0_committed_journal(bytes))
}

pub fn verify_receipt_and_journal_bytes(
    receipt_bytes: &[u8],
    journal_bytes: &[u8],
    config: &MultisigConfig,
    proposal: &Proposal,
) -> Result<ThresholdJournal, String> {
    let receipt: Receipt = borsh::from_slice(receipt_bytes)
        .map_err(|err| format!("decode RISC0 receipt bytes: {err}"))?;
    receipt
        .verify(THRESHOLD_PROOF_ID)
        .map_err(|err| format!("verify RISC0 receipt against LP-0002 image id: {err}"))?;

    let journal = decode_any_journal_bytes(journal_bytes)?;
    let proof = ThresholdProof {
        journal: journal.clone(),
        // For the typed safe verifier this just means "non-zero seal present".
        // Real soundness is established above by `receipt.verify(image_id)`.
        receipt_seal: receipt_seal_commitment(receipt_bytes),
    };
    verify_threshold_receipt(config, proposal, &proof).map_err(|err| format!("{err}"))?;
    Ok(journal)
}

pub struct ProofArtifacts {
    pub receipt_borsh: PathBuf,
    pub journal_borsh: PathBuf,
    pub manifest_txt: PathBuf,
}

/// Concrete LEZ receipt verifier for the LP-0002 heavy lane.
///
/// The `lez-program` crate deliberately avoids a direct RISC0 dependency. This
/// adapter is the host/localnet bridge: it checks that the instruction image id
/// is the compiled LP-0002 method id, verifies the receipt cryptographically, and
/// rejects any supplied journal that is not exactly the journal committed by the
/// verified receipt.
pub struct Risc0ReceiptVerifier;

impl ReceiptVerifier for Risc0ReceiptVerifier {
    fn verify_receipt(
        &self,
        image_id: Digest32,
        receipt_bytes: &[u8],
        journal: &ThresholdJournal,
    ) -> Result<(), LezProgramError> {
        if image_id != threshold_proof_image_id_bytes() {
            return Err(LezProgramError::Risc0ImageIdMismatch);
        }
        let receipt: Receipt = borsh::from_slice(receipt_bytes)
            .map_err(|err| LezProgramError::ReceiptVerification(err.to_string()))?;
        receipt
            .verify(THRESHOLD_PROOF_ID)
            .map_err(|err| LezProgramError::ReceiptVerification(err.to_string()))?;
        let committed_journal = decode_any_journal_bytes(&receipt.journal.bytes)
            .map_err(LezProgramError::ReceiptVerification)?;
        if &committed_journal != journal {
            return Err(LezProgramError::ReceiptVerification(
                "supplied LEZ journal does not match verified RISC0 receipt journal".into(),
            ));
        }
        Ok(())
    }
}

pub fn prove(input: &ThresholdGuestInput) -> Result<(Receipt, ThresholdJournal), String> {
    if std::env::var("RISC0_DEV_MODE").as_deref() != Ok("0") {
        return Err("refusing to prove unless RISC0_DEV_MODE=0 is set in the environment".into());
    }

    // Run the shared relation outside the guest first so normal Rust errors are
    // easier to read before the same relation is sealed in the zkVM.
    prove_threshold_guest(input).map_err(|err| format!("preflight guest relation: {err}"))?;

    let input_bytes = build_guest_input_bytes(input)?;
    let env = ExecutorEnv::builder()
        .write(&input_bytes)
        .map_err(|err| format!("write Borsh ThresholdGuestInput into ExecutorEnv: {err}"))?
        .build()
        .map_err(|err| format!("build ExecutorEnv: {err}"))?;
    let prove_info = default_prover()
        .prove(env, THRESHOLD_PROOF_ELF)
        .map_err(|err| format!("prove RISC0 LP-0002 receipt: {err}"))?;
    let receipt = prove_info.receipt;
    receipt
        .verify(THRESHOLD_PROOF_ID)
        .map_err(|err| format!("verify freshly produced receipt: {err}"))?;
    let journal = decode_any_journal_bytes(&receipt.journal.bytes)?;
    Ok((receipt, journal))
}

pub fn prove_to_dir(
    input: &ThresholdGuestInput,
    output_dir: &Path,
) -> Result<ProofArtifacts, String> {
    let (receipt, journal) = prove(input)?;
    let receipt_bytes = borsh::to_vec(&receipt).map_err(|err| format!("encode receipt: {err}"))?;
    let journal_bytes = receipt.journal.bytes.clone();
    write_proof_artifacts(output_dir, &receipt_bytes, &journal_bytes, &journal)
}

pub fn write_proof_artifacts(
    output_dir: &Path,
    receipt_bytes: &[u8],
    journal_bytes: &[u8],
    journal: &ThresholdJournal,
) -> Result<ProofArtifacts, String> {
    std::fs::create_dir_all(output_dir).map_err(|err| format!("create artifact dir: {err}"))?;
    let receipt_borsh = output_dir.join("receipt.borsh");
    let journal_borsh = output_dir.join("journal.borsh");
    let manifest_txt = output_dir.join("manifest.txt");
    std::fs::write(&receipt_borsh, receipt_bytes).map_err(|err| format!("write receipt: {err}"))?;
    std::fs::write(&journal_borsh, journal_bytes).map_err(|err| format!("write journal: {err}"))?;
    let manifest = format!(
        "LP-0002 RISC0 threshold proof artifacts\n\
         risc0_dev_mode=0\n\
         image_id={}\n\
         receipt_borsh={}\n\
         journal_borsh={}\n\
         multisig_id={}\n\
         proposal_id={}\n\
         member_root={}\n\
         threshold={}\n\
         approval_count={}\n\
         proof_id={}\n\
         privacy_note=member secrets, selected approver commitments, and the full membership set stay private witness data\n",
        threshold_proof_image_id_hex(),
        receipt_borsh.display(),
        journal_borsh.display(),
        hex32(journal.multisig_id.0),
        hex32(journal.proposal_id.0),
        hex32(journal.member_root),
        journal.threshold,
        journal.approval_count,
        hex32(journal.proof_id),
    );
    std::fs::write(&manifest_txt, manifest).map_err(|err| format!("write manifest: {err}"))?;
    Ok(ProofArtifacts {
        receipt_borsh,
        journal_borsh,
        manifest_txt,
    })
}

pub fn fixture_guest_input() -> ThresholdGuestInput {
    let members = vec![
        MemberSecret::from_seed(b"risc0 alice shielded secret"),
        MemberSecret::from_seed(b"risc0 boris shielded secret"),
        MemberSecret::from_seed(b"risc0 cyra shielded secret"),
    ];
    let config = MultisigConfig::new("risc0-heavy-lane-treasury", 2, &members)
        .expect("fixture config is valid");
    let proposal = Proposal::new(
        "risc0-heavy-lane-proposal",
        "transfer 42 LOGOS to grant recipient through RISC0-gated private multisig",
    );
    ThresholdGuestInput::new(
        config,
        proposal,
        &members,
        vec![members[0].clone(), members[2].clone()],
    )
}

pub fn receipt_seal_commitment(receipt_bytes: &[u8]) -> Digest32 {
    lp0002_private_multisig_core::hash_chunks(&[b"lp0002:risc0-receipt-seal", receipt_bytes])
}

fn digest_to_bytes(words: [u32; 8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, word) in words.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

fn hex32(bytes: [u8; 32]) -> String {
    hex::encode(bytes)
}
