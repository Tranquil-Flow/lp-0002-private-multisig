use lp0002_private_multisig_core::ProposalAction;
use lp0002_private_multisig_host::{
    decode_any_journal_bytes, fixture_guest_input, threshold_proof_image_id_bytes,
    threshold_proof_image_id_hex, verify_and_execute_program_id_hex,
};
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleVariant};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

struct VerifyAndExecuteBytesInstruction<'a> {
    variant_index: u32,
    fields: &'a [Field<'a>],
}

enum Field<'a> {
    Bytes(&'a [u8]),
    Bytes32([u8; 32]),
}

impl Serialize for Field<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            // IDL `Vec<u8>` fields are variable-length sequences.
            Field::Bytes(bytes) => {
                let mut seq = serializer.serialize_seq(Some(bytes.len()))?;
                for byte in *bytes {
                    seq.serialize_element(byte)?;
                }
                seq.end()
            }
            // IDL `[u8; 32]` fields are fixed-size tuples, matching spel-cli's
            // `serialize_to_risc0` adapter for array types.
            Field::Bytes32(bytes) => {
                let mut tuple = serializer.serialize_tuple(bytes.len())?;
                for byte in bytes {
                    tuple.serialize_element(byte)?;
                }
                tuple.end()
            }
        }
    }
}

impl Serialize for VerifyAndExecuteBytesInstruction<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut variant =
            serializer.serialize_tuple_variant("", self.variant_index, "", self.fields.len())?;
        for field in self.fields {
            variant.serialize_field(field)?;
        }
        variant.end()
    }
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(sha256_bytes(bytes))
}

fn u32_words_to_le_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

fn bytes_to_csv_preview(bytes: &[u8], max: usize) -> String {
    bytes
        .iter()
        .take(max)
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

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
    let action = ProposalAction::Custom {
        description: input.proposal.description.clone(),
        payload_hash: input.proposal.action_hash,
    };
    let action_borsh = borsh::to_vec(&action)?;

    // This mirrors the newly-added SPEL-compatible `verify_and_execute_bytes`
    // interface. The typed `verify_and_execute` remains in the public IDL, but
    // current spel-cli cannot parse/serialize arbitrary Defined structs/enums
    // from CLI strings, while it can represent byte-oriented args.
    // Variant index is the instruction index in interfaces/lp0002.idl.json.
    let variant_index = 5u32;
    let image_id = threshold_proof_image_id_bytes();
    let receipt_sha256 = sha256_bytes(&receipt_bytes);
    let journal_sha256 = sha256_bytes(&journal_bytes);
    let receipt_journal_commitment = lp0002_private_multisig_core::hash_chunks(&[
        b"lp0002:receipt-journal-commitment",
        &receipt_sha256,
        &journal_sha256,
    ]);
    let fields = [
        Field::Bytes32(create_key),
        Field::Bytes32(input.proposal.id.0),
        Field::Bytes32(image_id),
        Field::Bytes32(receipt_sha256),
        Field::Bytes32(receipt_journal_commitment),
        Field::Bytes(&journal_bytes),
        Field::Bytes(&action_borsh),
    ];
    let adapter = VerifyAndExecuteBytesInstruction {
        variant_index,
        fields: &fields,
    };
    let instruction_words = risc0_zkvm::serde::to_vec(&adapter)?;
    let instruction_bytes = u32_words_to_le_bytes(&instruction_words);

    let evidence = json!({
        "status": "spel_adapter_payload_built",
        "lane": "risc0-heavy-lane-spel-nssa-adapter",
        "risc0_dev_mode": "0",
        "program_id": verify_and_execute_program_id_hex(),
        "threshold_proof_image_id": threshold_proof_image_id_hex(),
        "instruction": "verify_and_execute_bytes",
        "instruction_index": variant_index,
        "multisig_state_account_seed_hex": hex::encode(create_key),
        "proposal_id": hex::encode(input.proposal.id.0),
        "risc0_image_id": threshold_proof_image_id_hex(),
        "receipt_bytes_len": receipt_bytes.len(),
        "receipt_bytes_sha256": sha256_hex(&receipt_bytes),
        "proof_journal_bytes_sha256": sha256_hex(&journal_bytes),
        "receipt_journal_commitment": hex::encode(receipt_journal_commitment),
        "receipt_transport": "receipt-journal-commitment-in-wrapper-input; full receipt retained in artifact evidence because raw receipt bytes exceed the current LEZ/RISC0 public-program session limit",
        "proof_journal_bytes_len": journal_bytes.len(),
        "action_borsh_len": action_borsh.len(),
        "action_borsh_sha256": sha256_hex(&action_borsh),
        "instruction_u32_words": instruction_words.len(),
        "instruction_data_len": instruction_bytes.len(),
        "instruction_data_sha256": sha256_hex(&instruction_bytes),
        "public_journal": {
            "domain": journal.domain,
            "multisig_id": hex::encode(journal.multisig_id.0),
            "proposal_id": hex::encode(journal.proposal_id.0),
            "action_hash": hex::encode(journal.action_hash),
            "member_root": hex::encode(journal.member_root),
            "member_count": journal.member_count,
            "threshold": journal.threshold,
            "approval_count": journal.approval_count,
            "nullifier_count": journal.nullifiers.len(),
            "proof_id": hex::encode(journal.proof_id)
        },
        "spel_cli_limit_note": "The full receipt is too large for a reliable single shell argv on macOS; this artifact is the exact byte-oriented payload a native NSSA/wallet adapter should submit from files, not by pasting Vec<u8> CSV into spel-cli.",
        "csv_previews": {
            "receipt_bytes_first_16": bytes_to_csv_preview(&receipt_bytes, 16),
            "proof_journal_bytes_first_16": bytes_to_csv_preview(&journal_bytes, 16),
            "action_borsh_first_16": bytes_to_csv_preview(&action_borsh, 16)
        }
    });

    let evidence_path = dir.join("spel-adapter-evidence.json");
    std::fs::write(&evidence_path, serde_json::to_vec_pretty(&evidence)?)?;
    println!("LP-0002 SPEL/NSSA adapter payload evidence written");
    println!("instruction=verify_and_execute_bytes");
    println!("program_id={}", verify_and_execute_program_id_hex());
    println!(
        "threshold_proof_image_id={}",
        threshold_proof_image_id_hex()
    );
    println!("instruction_data_len={}", instruction_bytes.len());
    println!("instruction_data_sha256={}", sha256_hex(&instruction_bytes));
    println!("evidence={}", evidence_path.display());
    Ok(())
}
