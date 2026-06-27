use common::{transaction::NSSATransaction, HashType};
use lp0002_private_multisig_core::ProposalAction;
use lp0002_private_multisig_host::{
    decode_any_journal_bytes, fixture_guest_input, threshold_proof_image_id_bytes,
    verify_and_execute_program_id_bytes,
};
use nssa::program::Program;
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

fn parse_hex32(s: &str) -> anyhow::Result<[u8; 32]> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)?;
    bytes.as_slice().try_into().map_err(|_| {
        anyhow::anyhow!(
            "expected a 32-byte (64 hex char) value, got {} bytes",
            bytes.len()
        )
    })
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
    // rc3 replaced the `(&ProgramId, &PdaSeed)` From impl with `for_public_pda`.
    AccountId::for_public_pda(program_id, &seed)
}

fn usage() -> ! {
    eprintln!(
        "Usage: lp0002-submit-localnet [--submit] [--no-poll] [--tx-hash HEX] [--program-id HEX] [--image-id HEX] [ARTIFACT_DIR]"
    );
    eprintln!("Default mode builds transaction evidence only. --submit sends via WalletCore::from_env() to whichever sequencer NSSA_WALLET_HOME_DIR points at (localnet or public testnet).");
    eprintln!("--tx-hash HEX queries inclusion evidence for an already-submitted transaction without resubmitting.");
    eprintln!("--program-id HEX overrides the wrapper program id (defaults to the embedded verify_and_execute image id; use the id returned by `wallet deploy-program` for public-testnet deploys).");
    eprintln!("--image-id HEX overrides the threshold-proof image id field (defaults to the embedded value; pass the recorded inner-proof image id when submitting against a different sequencer build).");
    eprintln!("--signer BASE58 funds/signs the public tx with a wallet-held account (required for the public testnet, which rejects nonce-less/unsigned public txs; account_ids are unchanged).");
    eprintln!("--deploy-program PATH deploys the program ELF (ProgramDeployment) to the configured sequencer and exits; run this once before --submit so the public testnet has the wrapper registered (a Public tx invoking an unregistered program is dropped).");
    std::process::exit(2);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut submit = false;
    let mut poll = true;
    let mut tx_hash_to_query: Option<HashType> = None;
    let mut artifact_dir: Option<PathBuf> = None;
    let mut program_id_override: Option<[u8; 32]> = None;
    let mut image_id_override: Option<[u8; 32]> = None;
    let mut signer_override: Option<String> = None;
    let mut deploy_program_path: Option<PathBuf> = None;
    let mut from_block: u64 = 1;
    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--submit" => submit = true,
            "--no-poll" => poll = false,
            "--deploy-program" => {
                let Some(p) = args.next() else { usage() };
                deploy_program_path = Some(PathBuf::from(p));
            }
            "--from-block" => {
                let Some(n) = args.next() else { usage() };
                from_block = n
                    .parse()
                    .map_err(|_| anyhow::anyhow!("invalid --from-block value: {n}"))?;
            }
            "--tx-hash" => {
                let Some(hash) = args.next() else { usage() };
                tx_hash_to_query = Some(HashType::from_str(&hash)?);
            }
            "--program-id" => {
                let Some(hex_str) = args.next() else { usage() };
                program_id_override = Some(parse_hex32(&hex_str)?);
            }
            "--image-id" => {
                let Some(hex_str) = args.next() else { usage() };
                image_id_override = Some(parse_hex32(&hex_str)?);
            }
            "--signer" => {
                let Some(s) = args.next() else { usage() };
                signer_override = Some(s);
            }
            "--help" | "-h" => usage(),
            other if other.starts_with('-') => usage(),
            other => artifact_dir = Some(PathBuf::from(other)),
        }
    }
    let dir = artifact_dir.unwrap_or_else(|| PathBuf::from("target/lp0002-risc0-fixture"));

    // Optional standalone step: deploy a program ELF (ProgramDeployment) to the
    // configured sequencer, then exit. Mirrors LP-0013's proven rc3 deploy path:
    // the deploy tx carries only the ELF bytes (no signer, no accounts, no gas);
    // a fresh deploy is included in a block, a re-deploy is skipped by the
    // sequencer (ProgramAlreadyExists). The public testnet drops a Public execute
    // tx whose program is not registered, so this must run before --submit there.
    if let Some(elf_path) = deploy_program_path.as_ref() {
        if !submit {
            eprintln!("--deploy-program requires --submit (it sends a real ProgramDeployment tx)");
            std::process::exit(2);
        }
        let elf = std::fs::read(elf_path)?;
        let program =
            Program::new(elf.clone()).map_err(|e| anyhow::anyhow!("parse program ELF: {e:?}"))?;
        let computed_id = program.id();
        // Honor --program-id so the check works under RISC0_SKIP_BUILD (which zeroes
        // the embedded image-id constant). program.id() is computed from the ELF bytes
        // at runtime and is unaffected by SKIP_BUILD, so it is always the real id.
        let expected_image_bytes =
            program_id_override.unwrap_or_else(verify_and_execute_program_id_bytes);
        let expected_id = program_id_from_image_id(expected_image_bytes);
        let id_bytes = |id: &ProgramId| -> String {
            hex::encode(id.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>())
        };
        let computed_hex = id_bytes(&computed_id);
        let expected_hex = id_bytes(&expected_id);
        let ids_match = computed_id == expected_id;
        println!(
            "deploy: elf={} bytes sha256={}",
            elf.len(),
            sha256_hex(&elf)
        );
        println!("deploy: computed_program_id={computed_hex}");
        println!("deploy: expected_program_id={expected_hex}");
        println!("deploy: program_id_matches_execute={ids_match}");
        if !ids_match {
            eprintln!("WARNING: deployed ELF's content-addressed program id does NOT match the execute tx's referenced program id; the execute will not resolve this program.");
        }
        let wallet = WalletCore::from_env()?;
        let deploy_tx = nssa::ProgramDeploymentTransaction::new(
            nssa::program_deployment_transaction::Message::new(elf.clone()),
        );
        let mut deploy_evidence = json!({
            "lane": "risc0-heavy-lane-program-deployment",
            "elf_path": elf_path.display().to_string(),
            "elf_len": elf.len(),
            "elf_sha256": sha256_hex(&elf),
            "computed_program_id": computed_hex,
            "expected_program_id": expected_hex,
            "program_id_matches_execute": ids_match,
        });
        match wallet
            .sequencer_client
            .send_transaction(NSSATransaction::ProgramDeployment(deploy_tx))
            .await
        {
            Ok(h) => {
                deploy_evidence["status"] = json!("submitted");
                deploy_evidence["tx_hash"] = json!(h.to_string());
                println!("deploy: submitted tx_hash={h}");
                if poll {
                    let poller = wallet::poller::TxPoller::new(
                        wallet.config(),
                        wallet.sequencer_client.clone(),
                    );
                    match poller.poll_tx(h).await {
                        Ok(_) => {
                            deploy_evidence["confirmed"] = json!(true);
                            println!("deploy: confirmed tx_hash={h}");
                        }
                        Err(err) => {
                            deploy_evidence["confirmed"] = json!(false);
                            deploy_evidence["poll_error"] = json!(err.to_string());
                            println!("deploy: poll_error={err} (lag, or program already deployed)");
                        }
                    }
                }
            }
            Err(e) => {
                deploy_evidence["status"] = json!("submit_error");
                deploy_evidence["submit_error"] = json!(format!("{e:?}"));
                println!("deploy: submit error (program may already be deployed): {e:?}");
            }
        }
        std::fs::create_dir_all(&dir).ok();
        let deploy_path = dir.join("nssa-deploy-evidence.json");
        std::fs::write(&deploy_path, serde_json::to_vec_pretty(&deploy_evidence)?)?;
        println!("deploy_evidence={}", deploy_path.display());
        return Ok(());
    }

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
    let wrapper_image_bytes =
        program_id_override.unwrap_or_else(verify_and_execute_program_id_bytes);
    let image_id = image_id_override.unwrap_or_else(threshold_proof_image_id_bytes);
    let program_id = program_id_from_image_id(wrapper_image_bytes);
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

    // Construct the wallet first so a signed public tx can fetch the signer's nonce.
    let wallet = if submit || tx_hash_to_query.is_some() {
        Some(WalletCore::from_env()?)
    } else {
        None
    };

    // Optional funded signer. Localnet accepted nonce-less, witness-less public txs,
    // but the public LEZ testnet sequencer drops them; it requires a signed, nonce'd
    // public tx for inclusion. account_ids is unchanged (the wrapper program still
    // sees [multisig_state, proposal]); the signer only adds a witness + nonce, whose
    // account is tracked separately via the witness set, not the program account list.
    let signer_account = signer_override
        .as_deref()
        .map(AccountId::from_str)
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid --signer account id: {e:?}"))?;
    let (nonces, signing_keys): (Vec<_>, Vec<&nssa::PrivateKey>) =
        if let (true, Some(signer)) = (submit, signer_account) {
            let w = wallet.as_ref().expect("wallet constructed for submit");
            let nonces = w.get_accounts_nonces(vec![signer]).await?;
            let key = w.get_account_public_signing_key(signer).ok_or_else(|| {
                anyhow::anyhow!("no signing key for --signer {signer} in NSSA_WALLET_HOME_DIR")
            })?;
            (nonces, vec![key])
        } else {
            (vec![], vec![])
        };
    let signer_count = signing_keys.len();
    let nonce_count = nonces.len();
    let signer_account_hex = signer_account.map(|s| hex::encode(s.value()));
    let message = Message::new_preserialized(
        program_id,
        account_ids.clone(),
        nonces,
        instruction_data.clone(),
    );
    let witness_set = WitnessSet::for_message(&message, &signing_keys);
    let tx = PublicTransaction::new(message, witness_set);

    let mut evidence = json!({
        "status": if submit { "built_pending_submit" } else { "built_dry_run" },
        "lane": "risc0-heavy-lane-file-backed-nssa-submit",
        "risc0_dev_mode": "0",
        "program_id": hex::encode(wrapper_image_bytes),
        "threshold_proof_image_id": hex::encode(image_id),
        "instruction": "verify_and_execute_bytes",
        "instruction_index": instruction_index,
        "account_ids": account_ids.iter().map(|id| hex::encode(id.value())).collect::<Vec<_>>(),
        "signer_count": signer_count,
        "nonce_count": nonce_count,
        "signer_account": signer_account_hex,
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
            .get_block_range(from_block, last_block)
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
    println!("program_id={}", hex::encode(wrapper_image_bytes));
    println!("threshold_proof_image_id={}", hex::encode(image_id));
    println!("accounts={}", account_ids.len());
    println!("instruction_data_len={}", instruction_bytes.len());
    println!("instruction_data_sha256={}", sha256_hex(&instruction_bytes));
    println!("evidence={}", path.display());
    Ok(())
}
