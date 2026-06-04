//! LP-0002 Private Multisig SDK
//!
//! A high-level integration crate for building threshold-gated, privacy-preserving
//! multisig workflows. Wraps the core proving/verification logic and the verifier
//! program behind a single [`MultisigSession`] that guides developers through the
//! complete lifecycle: multisig creation, proposal formation, member approval
//! collection, threshold proof generation, and on-chain execution.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use lp0002_private_multisig_sdk::prelude::*;
//!
//! let mut session = MultisigSession::new(
//!     "treasury",
//!     2,
//!     vec![
//!         b"alice-seed".as_slice(),
//!         b"boris-seed".as_slice(),
//!         b"cyra-seed".as_slice(),
//!     ],
//! )?;
//! session.create_proposal("grant-42", "transfer 42 LOGOS");
//! session.approve(0)?;
//! session.approve(2)?;
//! let _proof = session.prove()?;
//! let _receipt = session.verify_and_execute(ProposalAction::Transfer {
//!     to: "recipient".into(),
//!     amount: 42,
//!     denom: "LOGOS".into(),
//! })?;
//! # Ok::<(), SdkError>(())
//! ```

// ---------------------------------------------------------------------------
// Re-exports — expose every public symbol from the core and verifier crates
// so that downstream consumers only need a single dependency.
// ---------------------------------------------------------------------------
pub use lp0002_private_multisig_core::{
    contains_slice, hash_chunks, hex_digest, prove_threshold, verify_threshold_receipt,
    AddApprovalOutcome, ApprovalAccumulator, Digest32, MemberSecret, MultisigConfig, MultisigId,
    ProofError, Proposal, ProposalAction, ProposalId, ThresholdGuestInput, ThresholdJournal,
    ThresholdProof,
};

pub use lp0002_private_multisig_verifier::{ExecutionReceipt, VerifierError, VerifierProgram};

// ---------------------------------------------------------------------------
// SDK error type
// ---------------------------------------------------------------------------

/// Errors that can occur during a [`MultisigSession`] workflow.
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    /// A proof-level error from the core crate (e.g. insufficient approvals,
    /// duplicate nullifier, threshold mismatch).
    #[error("proof error: {0}")]
    Proof(#[from] ProofError),

    /// A verifier-level error (e.g. invalid proof, proposal already executed).
    #[error("verifier error: {0}")]
    Verifier(#[from] VerifierError),

    /// No proposal has been created yet — call [`MultisigSession::create_proposal`] first.
    #[error("no proposal has been created yet; call create_proposal() first")]
    NoProposal,

    /// No proof has been generated yet — call [`MultisigSession::prove`] first.
    #[error("no proof has been generated yet; call prove() first")]
    NoProof,

    /// The provided member index is out of range.
    #[error("invalid member index {index}: session has {count} members")]
    InvalidMemberIndex { index: usize, count: usize },
}

// ---------------------------------------------------------------------------
// MultisigSession — orchestrated workflow
// ---------------------------------------------------------------------------

/// A high-level session that orchestrates the full private-multisig lifecycle.
///
/// `MultisigSession` holds the multisig configuration, member secrets, the
/// current proposal, an approval accumulator, and a verifier instance.  Callers
/// progress through the workflow by calling methods in order:
///
/// 1. [`MultisigSession::new`] — create the multisig config and member set from seeds.
/// 2. [`MultisigSession::create_proposal`] — define the action to be authorized.
/// 3. [`MultisigSession::approve`] — collect member approvals (repeat until
///    threshold is met).
/// 4. [`MultisigSession::prove`] — produce a zero-knowledge-style threshold
///    proof suitable for public verification.
/// 5. [`MultisigSession::verify_and_execute`] — verify the proof and execute
///    the threshold-gated action, returning an [`ExecutionReceipt`].
///
/// The verifier tracks executed proposals internally, preventing double-spend
/// / replay of the same proposal ID.
pub struct MultisigSession {
    /// The multisig configuration (threshold, member root, multisig ID).
    config: MultisigConfig,
    /// All member secrets, retained so they can be passed to the prover.
    members: Vec<MemberSecret>,
    /// The current proposal (set by `create_proposal`).
    proposal: Option<Proposal>,
    /// Approval accumulator keyed to the current proposal.
    accumulator: Option<ApprovalAccumulator>,
    /// Indices of members that have approved (so we can look up their secrets
    /// during proving).
    approved_indices: Vec<usize>,
    /// The most recently generated threshold proof (set by `prove`).
    proof: Option<ThresholdProof>,
    /// The verifier program that gates execution and prevents double-spend.
    verifier: VerifierProgram,
}

impl MultisigSession {
    /// Create a new multisig session.
    ///
    /// `label` is a human-readable name for the multisig.  `threshold` is the
    /// minimum number of approvals required (must be ≥ 1 and ≤ the number of
    /// members).  `member_seeds` is a list of byte slices, one per member,
    /// from which deterministic member secrets are derived via
    /// [`MemberSecret::from_seed`].
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::Proof`] if the configuration is invalid (e.g. empty
    /// member set, threshold out of range).
    pub fn new(label: &str, threshold: u16, member_seeds: Vec<&[u8]>) -> Result<Self, SdkError> {
        let members: Vec<MemberSecret> = member_seeds
            .iter()
            .map(|seed| MemberSecret::from_seed(seed))
            .collect();
        let config = MultisigConfig::new(label, threshold, &members)?;
        Ok(Self {
            config,
            members,
            proposal: None,
            accumulator: None,
            approved_indices: Vec::new(),
            proof: None,
            verifier: VerifierProgram::default(),
        })
    }

    /// Return a reference to the multisig configuration.
    pub fn config(&self) -> &MultisigConfig {
        &self.config
    }

    /// Return the member secrets slice (useful for inspecting derived secrets).
    pub fn members(&self) -> &[MemberSecret] {
        &self.members
    }

    /// Create a new proposal and reset the approval state.
    ///
    /// `label` is a short proposal identifier (e.g. `"grant-42"`).
    /// `action` is a human-readable description of the proposed action (e.g.
    /// `"transfer 42 LOGOS to recipient"`).
    ///
    /// After calling this method the session is ready to accept approvals via
    /// [`approve`](Self::approve).
    pub fn create_proposal(&mut self, label: &str, action: &str) -> &Proposal {
        let proposal = Proposal::new(label, action);
        self.accumulator = Some(ApprovalAccumulator::new(
            self.config.multisig_id,
            proposal.id,
        ));
        self.approved_indices.clear();
        self.proof = None;
        self.proposal = Some(proposal);
        // Safety: we just set it above
        self.proposal
            .as_ref()
            .expect("proposal must be created before proving")
    }

    /// Return a reference to the current proposal, if one has been created.
    pub fn proposal(&self) -> Option<&Proposal> {
        self.proposal.as_ref()
    }

    /// Record an approval from the member at the given index.
    ///
    /// The member index corresponds to the order of seeds passed to
    /// [`MultisigSession::new`].  Each member may approve at most once per
    /// proposal; duplicate approvals are silently accepted but return
    /// [`AddApprovalOutcome::AlreadyPresent`].
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::NoProposal`] if no proposal has been created yet.
    /// Returns [`SdkError::InvalidMemberIndex`] if the index is out of range.
    pub fn approve(&mut self, member_index: usize) -> Result<AddApprovalOutcome, SdkError> {
        let accumulator = self.accumulator.as_mut().ok_or(SdkError::NoProposal)?;
        let member = self
            .members
            .get(member_index)
            .ok_or(SdkError::InvalidMemberIndex {
                index: member_index,
                count: self.members.len(),
            })?;
        let outcome = accumulator.add_member_approval(member)?;
        if outcome == AddApprovalOutcome::Added {
            self.approved_indices.push(member_index);
        }
        Ok(outcome)
    }

    /// Return the current number of unique approvals collected.
    pub fn approval_count(&self) -> u16 {
        self.accumulator
            .as_ref()
            .map(|a| a.approval_count())
            .unwrap_or(0)
    }

    /// Return `true` when enough approvals have been collected to meet the
    /// configured threshold.
    pub fn is_threshold_met(&self) -> bool {
        self.accumulator
            .as_ref()
            .is_some_and(|a| a.is_threshold_met(self.config.threshold))
    }

    /// Generate a threshold proof from the collected approvals.
    ///
    /// Consumes the approval accumulator state and produces a
    /// [`ThresholdProof`] that can be publicly verified without revealing
    /// which specific members approved (only their nullifiers are made public).
    ///
    /// The proof is stored internally and can be consumed by
    /// [`verify_and_execute`](Self::verify_and_execute).
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::NoProposal`] if no proposal has been created.
    /// Returns [`SdkError::Proof`] if the threshold has not been met or if
    /// duplicate nullifiers are detected.
    pub fn prove(&mut self) -> Result<&ThresholdProof, SdkError> {
        let proposal = self.proposal.as_ref().ok_or(SdkError::NoProposal)?;
        let approving_members: Vec<&MemberSecret> = self
            .approved_indices
            .iter()
            .map(|&i| &self.members[i])
            .collect();
        let proof = prove_threshold(&self.config, proposal, approving_members)?;
        self.proof = Some(proof);
        Ok(self
            .proof
            .as_ref()
            .expect("proof must be created before execution"))
    }

    /// Verify the most recent proof and execute the threshold-gated action.
    ///
    /// `action` is the concrete action to execute (must match the proposal
    /// description used in [`create_proposal`](Self::create_proposal)).
    ///
    /// Returns an [`ExecutionReceipt`] that records the execution.  The
    /// internal verifier tracks executed proposals, so calling this method
    /// twice with the same proposal will fail with
    /// [`VerifierError::ProposalAlreadyExecuted`].
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::NoProposal`] if no proposal has been created.
    /// Returns [`SdkError::NoProof`] if no proof has been generated yet.
    /// Returns [`SdkError::Verifier`] if the proof is invalid or the proposal
    /// was already executed.
    pub fn verify_and_execute(
        &mut self,
        action: ProposalAction,
    ) -> Result<ExecutionReceipt, SdkError> {
        let proposal = self.proposal.as_ref().ok_or(SdkError::NoProposal)?;
        let proof = self.proof.as_ref().ok_or(SdkError::NoProof)?;
        let receipt =
            self.verifier
                .execute_if_threshold_met(&self.config, proposal, proof, action)?;
        Ok(receipt)
    }
}

// ---------------------------------------------------------------------------
// Prelude — one import for everything a downstream consumer needs
// ---------------------------------------------------------------------------

/// Convenience module that re-exports every public type, function, and error
/// from this SDK as well as the underlying core and verifier crates.
///
/// ```rust,no_run
/// use lp0002_private_multisig_sdk::prelude::*;
/// ```
pub mod prelude {
    pub use super::*;
}
