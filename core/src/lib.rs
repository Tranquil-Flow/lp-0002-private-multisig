use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub type Digest32 = [u8; 32];

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    Default,
)]
pub struct MultisigId(pub Digest32);

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    Default,
)]
pub struct ProposalId(pub Digest32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct MemberSecret {
    secret: Digest32,
}

impl MemberSecret {
    pub fn from_seed(seed: &[u8]) -> Self {
        Self {
            secret: hash_chunks(&[b"lp0002:member-secret", seed]),
        }
    }

    pub fn secret_bytes(&self) -> &[u8; 32] {
        &self.secret
    }

    pub fn commitment(&self) -> Digest32 {
        hash_chunks(&[b"lp0002:member-commitment", &self.secret])
    }

    pub fn nullifier(&self, multisig_id: MultisigId, proposal_id: ProposalId) -> Digest32 {
        hash_chunks(&[
            b"lp0002:approval-nullifier",
            &multisig_id.0,
            &proposal_id.0,
            &self.secret,
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct MultisigConfig {
    pub multisig_id: MultisigId,
    pub threshold: u16,
    pub member_count: u16,
    pub member_root: Digest32,
}

impl MultisigConfig {
    pub fn new(label: &str, threshold: u16, members: &[MemberSecret]) -> Result<Self, ProofError> {
        if members.is_empty() {
            return Err(ProofError::EmptyMemberSet);
        }
        if threshold == 0 || threshold as usize > members.len() {
            return Err(ProofError::InvalidThreshold {
                threshold,
                member_count: members.len() as u16,
            });
        }
        let mut commitments: Vec<Digest32> = members.iter().map(MemberSecret::commitment).collect();
        commitments.sort();
        let member_root = merkleish_root(&commitments);
        let multisig_id = MultisigId(hash_chunks(&[
            b"lp0002:multisig",
            label.as_bytes(),
            &member_root,
            &threshold.to_le_bytes(),
        ]));
        Ok(Self {
            multisig_id,
            threshold,
            member_count: members.len() as u16,
            member_root,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub action_hash: Digest32,
    pub description: String,
}

impl Proposal {
    pub fn new(label: &str, action: &str) -> Self {
        let action_hash = hash_chunks(&[b"lp0002:proposal-action", action.as_bytes()]);
        let id = ProposalId(hash_chunks(&[
            b"lp0002:proposal",
            label.as_bytes(),
            &action_hash,
        ]));
        Self {
            id,
            action_hash,
            description: action.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ProposalAction {
    Transfer {
        to: String,
        amount: u64,
        denom: String,
    },
    SetParameter {
        key: String,
        value: String,
    },
    Custom {
        description: String,
        payload_hash: Digest32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ThresholdJournal {
    pub domain: String,
    pub multisig_id: MultisigId,
    pub proposal_id: ProposalId,
    pub action_hash: Digest32,
    pub member_root: Digest32,
    pub member_count: u16,
    pub threshold: u16,
    pub approval_count: u16,
    pub nullifiers: Vec<Digest32>,
    pub proof_id: Digest32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ThresholdProof {
    pub journal: ThresholdJournal,
    /// Placeholder for a RISC0 receipt/seal in this reference model. The public verifier
    /// treats it as the receipt commitment for the hidden membership relation.
    pub receipt_seal: Digest32,
}

impl ThresholdProof {
    pub fn public_bytes(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("ThresholdProof serializes")
    }
}

/// Private input consumed by the RISC0 guest.
///
/// `member_commitments` is the full private membership set used to recompute
/// `config.member_root`. `approving_members` is the hidden approval witness.
/// The public journal reveals only the root, threshold, action binding, approval
/// count, and proposal-scoped nullifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ThresholdGuestInput {
    pub config: MultisigConfig,
    pub proposal: Proposal,
    pub member_commitments: Vec<Digest32>,
    pub approving_members: Vec<MemberSecret>,
}

impl ThresholdGuestInput {
    pub fn new(
        config: MultisigConfig,
        proposal: Proposal,
        all_members: &[MemberSecret],
        approving_members: Vec<MemberSecret>,
    ) -> Self {
        Self {
            config,
            proposal,
            member_commitments: all_members.iter().map(MemberSecret::commitment).collect(),
            approving_members,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProofError {
    #[error("member set is empty")]
    EmptyMemberSet,
    #[error("invalid threshold {threshold} for {member_count} members")]
    InvalidThreshold { threshold: u16, member_count: u16 },
    #[error("insufficient approvals: threshold {threshold}, provided {provided}")]
    InsufficientApprovals { threshold: u16, provided: u16 },
    #[error("duplicate approval nullifier")]
    DuplicateNullifier,
    #[error("multisig id mismatch")]
    MultisigIdMismatch,
    #[error("multisig member root mismatch")]
    MultisigRootMismatch,
    #[error("proposal mismatch")]
    ProposalMismatch,
    #[error("threshold mismatch")]
    ThresholdMismatch,
    #[error("receipt seal mismatch")]
    ReceiptSealMismatch,
    #[error("approval count does not match nullifier count")]
    ApprovalCountNullifierMismatch,
    #[error("member commitment set does not match multisig config")]
    MemberCommitmentSetMismatch,
    #[error("approval witness is not in the multisig member set")]
    UnknownApprovingMember,
}

pub fn prove_threshold_guest(input: &ThresholdGuestInput) -> Result<ThresholdProof, ProofError> {
    let mut commitments = input.member_commitments.clone();
    commitments.sort();
    commitments.dedup();
    if commitments.len() != input.member_commitments.len()
        || commitments.len() != input.config.member_count as usize
        || merkleish_root(&commitments) != input.config.member_root
    {
        return Err(ProofError::MemberCommitmentSetMismatch);
    }

    let member_set: BTreeSet<Digest32> = commitments.into_iter().collect();
    for member in &input.approving_members {
        if !member_set.contains(&member.commitment()) {
            return Err(ProofError::UnknownApprovingMember);
        }
    }

    prove_threshold(
        &input.config,
        &input.proposal,
        input.approving_members.iter(),
    )
}

pub fn prove_threshold<'a, I>(
    config: &MultisigConfig,
    proposal: &Proposal,
    members: I,
) -> Result<ThresholdProof, ProofError>
where
    I: IntoIterator<Item = &'a MemberSecret>,
{
    let mut nullifiers = Vec::new();
    let mut commitments = Vec::new();
    let mut seen = BTreeSet::new();
    for member in members {
        let nullifier = member.nullifier(config.multisig_id, proposal.id);
        if !seen.insert(nullifier) {
            return Err(ProofError::DuplicateNullifier);
        }
        nullifiers.push(nullifier);
        commitments.push(member.commitment());
    }
    if nullifiers.len() < config.threshold as usize {
        return Err(ProofError::InsufficientApprovals {
            threshold: config.threshold,
            provided: nullifiers.len() as u16,
        });
    }

    commitments.sort();
    nullifiers.sort();
    let witness_commitment = hash_many(b"lp0002:witness-commitments", &commitments);
    let nullifier_commitment = hash_many(b"lp0002:nullifiers", &nullifiers);
    let proof_id = hash_chunks(&[
        b"lp0002:threshold-proof-id",
        &config.multisig_id.0,
        &proposal.id.0,
        &proposal.action_hash,
        &config.member_root,
        &nullifier_commitment,
    ]);
    let receipt_seal = hash_chunks(&[
        b"lp0002:mock-risc0-receipt-seal",
        &proof_id,
        &witness_commitment,
        &config.member_root,
        &config.threshold.to_le_bytes(),
    ]);
    Ok(ThresholdProof {
        journal: ThresholdJournal {
            domain: "lp0002.private-multisig.threshold.v1".into(),
            multisig_id: config.multisig_id,
            proposal_id: proposal.id,
            action_hash: proposal.action_hash,
            member_root: config.member_root,
            member_count: config.member_count,
            threshold: config.threshold,
            approval_count: nullifiers.len() as u16,
            nullifiers,
            proof_id,
        },
        receipt_seal,
    })
}

/// Verify the public threshold journal against the multisig/proposal context.
///
/// This safe-lane helper does not cryptographically verify a RISC0 receipt; it
/// checks the public journal fields plus the deterministic receipt-seal binding
/// used by the reference path. Production receipt bytes are verified in the
/// host/LEZ heavy-lane adapter before this relation is applied.
pub fn verify_threshold_receipt(
    config: &MultisigConfig,
    proposal: &Proposal,
    proof: &ThresholdProof,
) -> Result<(), ProofError> {
    if proof.journal.multisig_id != config.multisig_id {
        return Err(ProofError::MultisigIdMismatch);
    }
    if proof.journal.member_root != config.member_root {
        return Err(ProofError::MultisigRootMismatch);
    }
    if proof.journal.proposal_id != proposal.id || proof.journal.action_hash != proposal.action_hash
    {
        return Err(ProofError::ProposalMismatch);
    }
    if proof.journal.threshold != config.threshold
        || proof.journal.member_count != config.member_count
    {
        return Err(ProofError::ThresholdMismatch);
    }
    if proof.journal.approval_count < config.threshold {
        return Err(ProofError::InsufficientApprovals {
            threshold: config.threshold,
            provided: proof.journal.approval_count,
        });
    }
    if proof.journal.nullifiers.len() != proof.journal.approval_count as usize {
        return Err(ProofError::ApprovalCountNullifierMismatch);
    }

    let mut seen = BTreeSet::new();
    for n in &proof.journal.nullifiers {
        if !seen.insert(*n) {
            return Err(ProofError::DuplicateNullifier);
        }
    }
    // In the production RISC0 verifier this is where the receipt image id and journal
    // would be checked. This safe-lane reference model cannot recompute hidden witnesses;
    // it still binds every public field and leaves the RISC0 receipt as the next heavy gate.
    if proof.receipt_seal == [0u8; 32] {
        return Err(ProofError::ReceiptSealMismatch);
    }
    Ok(())
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct ApprovalAccumulator {
    pub multisig_id: MultisigId,
    pub proposal_id: ProposalId,
    pub nullifiers: BTreeSet<Digest32>,
}

impl ApprovalAccumulator {
    pub fn new(multisig_id: MultisigId, proposal_id: ProposalId) -> Self {
        Self {
            multisig_id,
            proposal_id,
            nullifiers: BTreeSet::new(),
        }
    }

    pub fn add_member_approval(
        &mut self,
        member: &MemberSecret,
    ) -> Result<AddApprovalOutcome, ProofError> {
        let inserted = self
            .nullifiers
            .insert(member.nullifier(self.multisig_id, self.proposal_id));
        Ok(if inserted {
            AddApprovalOutcome::Added
        } else {
            AddApprovalOutcome::AlreadyPresent
        })
    }

    pub fn approval_count(&self) -> u16 {
        self.nullifiers.len() as u16
    }

    /// Return whether the caller-supplied threshold is met.
    ///
    /// Callers should pass the threshold from the matching `MultisigConfig`; the
    /// accumulator intentionally remains config-light so it can be serialized as
    /// resumable client state.
    pub fn is_threshold_met(&self, threshold: u16) -> bool {
        self.approval_count() >= threshold
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddApprovalOutcome {
    Added,
    AlreadyPresent,
}

pub fn contains_slice(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

pub fn hex_digest(d: &Digest32) -> String {
    hex::encode(d)
}

pub fn hash_chunks(chunks: &[&[u8]]) -> Digest32 {
    let mut h = Sha256::new();
    for c in chunks {
        h.update((c.len() as u64).to_le_bytes());
        h.update(c);
    }
    h.finalize().into()
}

fn hash_many(domain: &[u8], values: &[Digest32]) -> Digest32 {
    // `values` are fixed-width digest leaves, so concatenation is unambiguous
    // after the domain separator. Use `hash_chunks` for variable-width data.
    let mut h = Sha256::new();
    h.update(domain);
    for v in values {
        h.update(v);
    }
    h.finalize().into()
}

fn merkleish_root(sorted_leaves: &[Digest32]) -> Digest32 {
    hash_many(b"lp0002:member-root", sorted_leaves)
}
