use lp0002_private_multisig_host::{
    fixture_guest_input, prove_to_dir, threshold_proof_image_id_hex,
};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let output_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/lp0002-risc0-fixture"));
    let input = fixture_guest_input();
    let artifacts = prove_to_dir(&input, &output_dir).map_err(anyhow::Error::msg)?;
    println!("LP-0002 RISC0 proof generated");
    println!("image_id={}", threshold_proof_image_id_hex());
    println!("receipt={}", artifacts.receipt_borsh.display());
    println!("journal={}", artifacts.journal_borsh.display());
    println!("manifest={}", artifacts.manifest_txt.display());
    Ok(())
}
