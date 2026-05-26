use lp0002_private_multisig_host::{
    fixture_guest_input, threshold_proof_image_id_hex, verify_receipt_and_journal_bytes,
};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/lp0002-risc0-fixture"));
    let receipt_bytes = std::fs::read(dir.join("receipt.borsh"))?;
    let journal_bytes = std::fs::read(dir.join("journal.borsh"))?;
    let input = fixture_guest_input();
    let journal = verify_receipt_and_journal_bytes(
        &receipt_bytes,
        &journal_bytes,
        &input.config,
        &input.proposal,
    )
    .map_err(anyhow::Error::msg)?;
    println!("LP-0002 RISC0 proof verified");
    println!("image_id={}", threshold_proof_image_id_hex());
    println!("threshold={}", journal.threshold);
    println!("approval_count={}", journal.approval_count);
    println!("proof_id={}", hex::encode(journal.proof_id));
    Ok(())
}
