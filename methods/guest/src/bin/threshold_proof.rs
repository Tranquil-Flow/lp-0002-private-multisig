use borsh::BorshDeserialize;
use lp0002_private_multisig_core::{prove_threshold_guest, ThresholdGuestInput};
use risc0_zkvm::guest::env;

fn main() {
    // Byte-oriented boundary for SPEL/LEZ compatibility: the host writes a Borsh
    // ThresholdGuestInput as a Vec<u8>, and the guest commits a Borsh
    // ThresholdJournal as public output. The private member set and signer
    // secrets never leave the RISC0 witness.
    let input_bytes: Vec<u8> = env::read();
    let input = ThresholdGuestInput::try_from_slice(&input_bytes)
        .expect("LP-0002 guest: malformed Borsh ThresholdGuestInput");
    let proof = prove_threshold_guest(&input).expect("LP-0002 guest: invalid threshold witness");
    let journal_bytes =
        borsh::to_vec(&proof.journal).expect("LP-0002 guest: threshold journal serializes");

    env::commit(&journal_bytes);
}
