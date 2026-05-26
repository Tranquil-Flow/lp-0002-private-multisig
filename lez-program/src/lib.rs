//! LEZ-shaped execution wrapper for LP-0002 private multisig.
//!
//! This crate is deliberately runtime-small: it contains the serializable account
//! state and instruction boundary a Logos/LEZ program needs, but leaves the
//! concrete RISC0 receipt verification backend behind [`ReceiptVerifier`]. That
//! lets unit tests exercise replay/account semantics without compiling the heavy
//! prover, while the localnet adapter can plug in the real RISC0 image verifier.

use borsh::{BorshDeserialize, BorshSerialize};
use lp0002_private_multisig_core::{
    Digest32, MultisigConfig, MultisigId, ProofError, Proposal, ProposalAction, ProposalId,
    ThresholdJournal, ThresholdProof,
};
use lp0002_private_multisig_verifier::{ExecutionReceipt, VerifierError, VerifierProgram};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct MultisigState {
    pub create_key: Digest32,
    pub multisig_id: MultisigId,
    pub threshold: u16,
    pub member_count: u16,
    pub member_root: Digest32,
    pub bump: u8,
}

impl MultisigState {
    pub fn from_config(create_key: Digest32, config: &MultisigConfig, bump: u8) -> Self {
        Self {
            create_key,
            multisig_id: config.multisig_id,
            threshold: config.threshold,
            member_count: config.member_count,
            member_root: config.member_root,
            bump,
        }
    }

    pub fn to_config(&self) -> MultisigConfig {
        MultisigConfig {
            multisig_id: self.multisig_id,
            threshold: self.threshold,
            member_count: self.member_count,
            member_root: self.member_root,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ProposalState {
    pub proposal_id: ProposalId,
    pub action_hash: Digest32,
    pub description: String,
    pub approval_count: u16,
    pub nullifiers: Vec<Digest32>,
    pub executed: bool,
    pub bump: u8,
}

impl ProposalState {
    pub fn from_proposal(proposal: &Proposal, bump: u8) -> Self {
        Self {
            proposal_id: proposal.id,
            action_hash: proposal.action_hash,
            description: proposal.description.clone(),
            approval_count: 0,
            nullifiers: Vec::new(),
            executed: false,
            bump,
        }
    }

    pub fn to_proposal(&self) -> Proposal {
        Proposal {
            id: self.proposal_id,
            action_hash: self.action_hash,
            description: self.description.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ExecuteProposalInstruction {
    pub create_key: Digest32,
    pub proposal_id: ProposalId,
    pub risc0_image_id: Digest32,
    /// Borsh-encoded or runtime-native RISC0 receipt bytes. The verifier backend
    /// decides how to parse these bytes.
    pub receipt_bytes: Vec<u8>,
    /// Public journal decoded from the RISC0 receipt. Keeping this typed at the
    /// LEZ boundary mirrors the SPEL IDL and avoids exposing private witnesses.
    pub proof_journal: ThresholdJournal,
    pub action: ProposalAction,
}

impl ExecuteProposalInstruction {
    pub fn to_borsh(&self) -> Result<Vec<u8>, LezProgramError> {
        borsh::to_vec(self).map_err(|err| LezProgramError::Codec(err.to_string()))
    }

    pub fn from_borsh(bytes: &[u8]) -> Result<Self, LezProgramError> {
        Self::try_from_slice(bytes).map_err(|err| LezProgramError::Codec(err.to_string()))
    }
}

pub trait ReceiptVerifier {
    fn verify_receipt(
        &self,
        image_id: Digest32,
        receipt_bytes: &[u8],
        journal: &ThresholdJournal,
    ) -> Result<(), LezProgramError>;
}

pub fn execute_proposal<V: ReceiptVerifier>(
    multisig_state: &MultisigState,
    proposal_state: &mut ProposalState,
    instruction: ExecuteProposalInstruction,
    verifier: &V,
) -> Result<ExecutionReceipt, LezProgramError> {
    if instruction.create_key != multisig_state.create_key {
        return Err(LezProgramError::CreateKeyMismatch);
    }
    if instruction.proposal_id != proposal_state.proposal_id {
        return Err(LezProgramError::ProposalAccountMismatch);
    }
    if proposal_state.executed {
        return Err(LezProgramError::ProposalAlreadyExecuted);
    }
    if instruction.receipt_bytes.is_empty() {
        return Err(LezProgramError::EmptyReceipt);
    }

    verifier.verify_receipt(
        instruction.risc0_image_id,
        &instruction.receipt_bytes,
        &instruction.proof_journal,
    )?;

    let config = multisig_state.to_config();
    let proposal = proposal_state.to_proposal();
    let proof = ThresholdProof {
        journal: instruction.proof_journal.clone(),
        // The real soundness check is performed by `ReceiptVerifier`. The core
        // verifier still wants a non-zero seal for safe-lane compatibility.
        receipt_seal: lp0002_private_multisig_core::hash_chunks(&[
            b"lp0002:lez-risc0-receipt",
            &instruction.receipt_bytes,
        ]),
    };

    let mut gate = VerifierProgram::default();
    let receipt = gate
        .execute_if_threshold_met(&config, &proposal, &proof, instruction.action)
        .map_err(LezProgramError::Verifier)?;

    proposal_state.executed = true;
    proposal_state.approval_count = proof.journal.approval_count;
    proposal_state.nullifiers = proof.journal.nullifiers.clone();

    Ok(receipt)
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LezProgramError {
    #[error("instruction create_key does not match multisig account")]
    CreateKeyMismatch,
    #[error("instruction proposal_id does not match proposal account")]
    ProposalAccountMismatch,
    #[error("proposal already executed")]
    ProposalAlreadyExecuted,
    #[error("receipt bytes are empty")]
    EmptyReceipt,
    #[error("RISC0 image id mismatch")]
    Risc0ImageIdMismatch,
    #[error("RISC0 receipt verification failed: {0}")]
    ReceiptVerification(String),
    #[error("invalid threshold proof: {0}")]
    Proof(ProofError),
    #[error("verifier gate failed: {0}")]
    Verifier(VerifierError),
    #[error("codec error: {0}")]
    Codec(String),
}

impl From<ProofError> for LezProgramError {
    fn from(value: ProofError) -> Self {
        Self::Proof(value)
    }
}

#[derive(Debug, Clone)]
pub struct ExpectedImageVerifier {
    pub expected_image_id: Digest32,
}

impl ReceiptVerifier for ExpectedImageVerifier {
    fn verify_receipt(
        &self,
        image_id: Digest32,
        receipt_bytes: &[u8],
        journal: &ThresholdJournal,
    ) -> Result<(), LezProgramError> {
        if image_id != self.expected_image_id {
            return Err(LezProgramError::Risc0ImageIdMismatch);
        }
        if receipt_bytes.is_empty() {
            return Err(LezProgramError::EmptyReceipt);
        }
        if journal.domain != "lp0002.private-multisig.threshold.v1" {
            return Err(LezProgramError::ReceiptVerification(
                "unexpected LP-0002 journal domain".into(),
            ));
        }
        Ok(())
    }
}
