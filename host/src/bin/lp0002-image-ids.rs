//! Print the LP-0002 guest ImageIDs embedded in this build.
//!
//! Built with `RISC0_USE_DOCKER=1`, these are the reproducible, content-addressed
//! ids an evaluator regenerates via `cargo risczero build` — the inner
//! threshold-proof image id and the on-chain wrapper program id (ProgramId == ImageID).
use lp0002_private_multisig_host::{threshold_proof_image_id_hex, verify_and_execute_program_id_hex};

fn main() {
    println!("threshold_proof_image_id={}", threshold_proof_image_id_hex());
    println!(
        "verify_and_execute_program_id={}",
        verify_and_execute_program_id_hex()
    );
}
