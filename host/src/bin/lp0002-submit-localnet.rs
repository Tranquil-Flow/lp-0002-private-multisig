use common::{transaction::NSSATransaction, HashType};
use lp0002_private_multisig_core::ProposalAction;
use lp0002_private_multisig_host::{
    decode_any_journal_bytes, fixture_guest_input, threshold_proof_image_id_bytes,
    threshold_proof_image_id_hex, verify_and_execute_program_id_bytes,
    verify_and_execute_program_id_hex,
};
use nssa::public_transaction::{Message, WitnessSet};
use nssa::{AccountId, PublicTransaction};
use nssa_core::program::{PdaSeed, ProgramId};
use sequencer_service_rpc::RpcClient as _;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleVariant};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{path::PathBuf, str::FromStr};
use wallet::WalletCore;

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
            Field::Bytes(bytes) => {
                let mut seq = serializer.serialize_seq(Some(bytes.len()))?;
                for byte in *bytes {
                    seq.serialize_element(byte)?;
                }
                seq.end()
            }
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

fn program_id_from_image_id(image_id: [u8; 32]) -> ProgramId {
    let mut out = [0u32; 8];
    for (idx, chunk) in image_id.chunks_exact(4).enumerate() {
        out[idx] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    out
}

fn proposal_pda(program_id: &ProgramId, create_key: [u8; 32], proposal_id: [u8; 32]) -> AccountId {
    let seed = PdaSeed::new(lp0002_private_multisig_core::hash_chunks(&[
        &create_key,
        &proposal_id,
    ]));
    AccountId::from((program_id, &seed))
}

fn usage() -> ! {
    eprintln!(
        "Usage: lp0002-submit-localnet [--submit] [--no-poll] [--tx-hash HEX] [ARTIFACT_DIR]"
    );
    eprintln!("Default mode builds transaction evidence only. --submit sends to localnet via WalletCore::from_env().");
    eprintln!("--tx-hash HEX queries inclusion evidence for an already-submitted transaction without resubmitting.");
    std::process::exit(2);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut submit = false;
    let mut poll = true;
    let mut tx_hash_to_query: Option<HashType> = None;
    let mut artifact_dir: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--submit" => submit = true,
            "--no-poll" => poll = false,
            "--tx-hash" => {
                let Some(hash) = args.next() else { usage() };
                tx_hash_to_query = Some(HashType::from_str(&hash)?);
            }
            "--help" | "-h" => usage(),
            other if other.starts_with('-') => usage(),
            other => artifact_dir = Some(PathBuf::from(other)),
        }
    }
    let dir = artifact_dir.unwrap_or_else(|| PathBuf::from("target/lp0002-risc0-fixture"));

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
    let image_id = threshold_proof_image_id_bytes();
    let program_id = program_id_from_image_id(verify_and_execute_program_id_bytes());
    let multisig_state = AccountId::new(create_key);
    let proposal = proposal_pda(&program_id, create_key, input.proposal.id.0);

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
    let instruction_index = 5u32;
    let instruction = VerifyAndExecuteBytesInstruction {
        variant_index: instruction_index,
        fields: &fields,
    };
    let instruction_data = risc0_zkvm::serde::to_vec(&instruction)?;
    let instruction_bytes: Vec<u8> = instruction_data
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect();
    let account_ids = vec![multisig_state, proposal];

    // The IDL marks no signer accounts for this wrapper path; mirror `spel` public tx behavior.
    let nonces = vec![];
    let message = Message::new_preserialized(
        program_id,
        account_ids.clone(),
        nonces,
        instruction_data.clone(),
    );
    let witness_set = WitnessSet::for_message(&message, &[]);
    let tx = PublicTransaction::new(message, witness_set);

    let mut evidence = json!({
        "status": if submit { "built_pending_submit" } else { "built_dry_run" },
        "lane": "risc0-heavy-lane-file-backed-nssa-submit",
        "risc0_dev_mode": "0",
        "program_id": verify_and_execute_program_id_hex(),
        "threshold_proof_image_id": threshold_proof_image_id_hex(),
        "instruction": "verify_and_execute_bytes",
        "instruction_index": instruction_index,
        "account_ids": account_ids.iter().map(|id| hex::encode(id.value())).collect::<Vec<_>>(),
        "signer_count": 0,
        "nonce_count": 0,
        "receipt_bytes_len": receipt_bytes.len(),
        "proof_journal_bytes_len": journal_bytes.len(),
        "action_borsh_len": action_borsh.len(),
        "instruction_u32_words": instruction_data.len(),
        "instruction_data_len": instruction_bytes.len(),
        "instruction_data_sha256": sha256_hex(&instruction_bytes),
        "receipt_bytes_sha256": sha256_hex(&receipt_bytes),
        "proof_journal_bytes_sha256": sha256_hex(&journal_bytes),
        "receipt_journal_commitment": hex::encode(receipt_journal_commitment),
        "receipt_transport": "receipt-journal-commitment-in-wrapper-input; full receipt retained in artifact evidence because raw receipt bytes exceed the current LEZ/RISC0 public-program session limit",
        "public_journal": {
            "domain": journal.domain,
            "proposal_id": hex::encode(journal.proposal_id.0),
            "approval_count": journal.approval_count,
            "threshold": journal.threshold,
            "proof_id": hex::encode(journal.proof_id),
        }
    });

    let wallet = if submit || tx_hash_to_query.is_some() {
        Some(WalletCore::from_env()?)
    } else {
        None
    };

    if let Some(tx_hash) = tx_hash_to_query {
        let wallet = wallet.as_ref().expect("wallet constructed for query");
        let last_block = wallet.sequencer_client.get_last_block_id().await?;
        let tx_found = wallet
            .sequencer_client
            .get_transaction(tx_hash)
            .await?
            .is_some();
        let mut included_block_id = None;
        let mut included_block_hash = None;
        let mut included_tx_index = None;
        let blocks = wallet
            .sequencer_client
            .get_block_range(1, last_block)
            .await?;
        'outer: for block in blocks {
            for (idx, candidate) in block.body.transactions.iter().enumerate() {
                if candidate.hash() == tx_hash {
                    included_block_id = Some(block.header.block_id);
                    included_block_hash = Some(block.header.hash.to_string());
                    included_tx_index = Some(idx);
                    break 'outer;
                }
            }
        }
        evidence["status"] = json!(if tx_found {
            "confirmed"
        } else {
            "submitted_not_included"
        });
        evidence["tx_hash"] = json!(tx_hash.to_string());
        evidence["confirmed"] = json!(tx_found);
        evidence["last_block_id"] = json!(last_block);
        evidence["included_block_id"] = json!(included_block_id);
        evidence["included_block_hash"] = json!(included_block_hash);
        evidence["included_tx_index"] = json!(included_tx_index);
        evidence["compute_unit_note"] = json!("LEZ v0.2.0-rc1 JSON-RPC exposes transaction/block inclusion but not per-transaction compute-unit counters; this evidence records canonical payload size/hash and block inclusion when available.");
        if !tx_found {
            evidence["submit_attempt_note"] = json!("Transaction was accepted by sendTransaction but not included. Check sequencer logs for ProgramExecutionFailed details; the wrapper program id and threshold proof image id are recorded separately in this evidence.");
        }
    } else if submit {
        let wallet = wallet.as_ref().expect("wallet constructed for submit");
        let tx_hash = wallet
            .sequencer_client
            .send_transaction(NSSATransaction::Public(tx))
            .await?;
        evidence["status"] = json!("submitted");
        evidence["tx_hash"] = json!(tx_hash.to_string());
        println!("submitted tx_hash={tx_hash}");
        if poll {
            let poller =
                wallet::poller::TxPoller::new(wallet.config(), wallet.sequencer_client.clone());
            match poller.poll_tx(tx_hash).await {
                Ok(_) => {
                    evidence["confirmed"] = json!(true);
                    println!("confirmed tx_hash={tx_hash}");
                }
                Err(err) => {
                    evidence["confirmed"] = json!(false);
                    evidence["poll_error"] = json!(err.to_string());
                    println!("poll_error={err}");
                }
            }
        }
    } else {
        println!("dry_run=true");
    }

    let path = dir.join(if submit || tx_hash_to_query.is_some() {
        "nssa-submit-evidence.json"
    } else {
        "nssa-submit-dry-run.json"
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&evidence)?)?;
    println!("program_id={}", verify_and_execute_program_id_hex());
    println!(
        "threshold_proof_image_id={}",
        threshold_proof_image_id_hex()
    );
    println!("accounts={}", account_ids.len());
    println!("instruction_data_len={}", instruction_bytes.len());
    println!("instruction_data_sha256={}", sha256_hex(&instruction_bytes));
    println!("evidence={}", path.display());
    Ok(())
}
