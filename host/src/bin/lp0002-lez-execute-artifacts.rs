use lp0002_private_multisig_core::ProposalAction;
use lp0002_private_multisig_host::{
    decode_any_journal_bytes, fixture_guest_input, threshold_proof_image_id_bytes,
    threshold_proof_image_id_hex, Risc0ReceiptVerifier,
};
use lp0002_private_multisig_lez_program::{
    execute_proposal, ExecuteProposalInstruction, MultisigState, ProposalState,
};
use serde_json::json;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/lp0002-risc0-fixture"));

    let receipt_bytes = std::fs::read(dir.join("receipt.borsh"))?;
    let journal_bytes = std::fs::read(dir.join("journal.borsh"))?;
    let journal = decode_any_journal_bytes(&journal_bytes).map_err(anyhow::Error::msg)?;
    let input = fixture_guest_input();

    let create_key = lp0002_private_multisig_core::hash_chunks(&[
        b"lp0002:lez-execute-artifacts:create-key",
        &input.config.multisig_id.0,
    ]);
    let multisig_state = MultisigState::from_config(create_key, &input.config, 255);
    let mut proposal_state = ProposalState::from_proposal(&input.proposal, 254);
    let instruction = ExecuteProposalInstruction {
        create_key,
        proposal_id: input.proposal.id,
        risc0_image_id: threshold_proof_image_id_bytes(),
        receipt_bytes,
        proof_journal: journal.clone(),
        action: ProposalAction::Custom {
            description: input.proposal.description.clone(),
            payload_hash: input.proposal.action_hash,
        },
    };

    let execution = execute_proposal(
        &multisig_state,
        &mut proposal_state,
        instruction,
        &Risc0ReceiptVerifier,
    )?;

    let evidence = json!({
        "status": "executed",
        "lane": "risc0-heavy-lane-lez-wrapper",
        "risc0_dev_mode": "0",
        "image_id": threshold_proof_image_id_hex(),
        "multisig_id": hex::encode(execution.multisig_id.0),
        "proposal_id": hex::encode(execution.proposal_id.0),
        "approval_count": execution.approval_count,
        "proof_id": hex::encode(execution.proof_id),
        "proposal_state_executed": proposal_state.executed,
        "proposal_state_nullifier_count": proposal_state.nullifiers.len(),
        "receipt_path": dir.join("receipt.borsh").display().to_string(),
        "journal_path": dir.join("journal.borsh").display().to_string(),
        "note": "This is the concrete RISC0-to-LEZ wrapper execution path. It verifies the RISC0 receipt and then mutates the LEZ-shaped proposal account state; live sequencer transaction evidence is a separate final gate."
    });
    let evidence_path = dir.join("lez-execution.json");
    std::fs::write(&evidence_path, serde_json::to_vec_pretty(&evidence)?)?;

    println!("LP-0002 LEZ execution wrapper accepted RISC0 receipt");
    println!("image_id={}", threshold_proof_image_id_hex());
    println!("approval_count={}", execution.approval_count);
    println!("proof_id={}", hex::encode(execution.proof_id));
    println!("evidence={}", evidence_path.display());
    Ok(())
}
