use lp0002_private_multisig_core::{
    verify_threshold_receipt, MultisigConfig, ProofError, Proposal, ProposalAction, ProposalId,
    ThresholdProof,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierProgram {
    executed: BTreeSet<ProposalId>,
}

impl VerifierProgram {
    pub fn execute_if_threshold_met(
        &mut self,
        config: &MultisigConfig,
        proposal: &Proposal,
        proof: &ThresholdProof,
        action: ProposalAction,
    ) -> Result<ExecutionReceipt, VerifierError> {
        verify_threshold_receipt(config, proposal, proof).map_err(VerifierError::InvalidProof)?;
        if !self.executed.insert(proposal.id) {
            return Err(VerifierError::ProposalAlreadyExecuted);
        }
        Ok(ExecutionReceipt {
            proposal_id: proposal.id,
            multisig_id: config.multisig_id,
            action,
            approval_count: proof.journal.approval_count,
            proof_id: proof.journal.proof_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub proposal_id: ProposalId,
    pub multisig_id: lp0002_private_multisig_core::MultisigId,
    pub action: ProposalAction,
    pub approval_count: u16,
    pub proof_id: lp0002_private_multisig_core::Digest32,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VerifierError {
    #[error("invalid private multisig proof: {0}")]
    InvalidProof(ProofError),
    #[error("proposal already executed")]
    ProposalAlreadyExecuted,
}
