use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use risc0_build::{
    embed_methods, embed_methods_with_options, DockerOptionsBuilder, GuestOptionsBuilder,
};
use sha2::{Digest, Sha256};

const CANONICAL_THRESHOLD_RELATIVE_PATH: &str = "guest/elf/threshold_proof.bin";
const CANONICAL_THRESHOLD_BYTES: usize = 402_972;
const CANONICAL_THRESHOLD_SHA256: &str =
    "ff03aac0e9f900cef8c3011b76d6dd64b9d79ac9377731e1d892fe7cbc245d0f";
const CANONICAL_THRESHOLD_IMAGE_ID: &str =
    "e57b90a0fd306a8a6258f4fe7c045d056cf70142408e31a020817aaaba23bac4";
const CANONICAL_THRESHOLD_SOURCE_TREE_SHA256: &str =
    "4f76c12f178f298e2129cc8409b9411f0281f591a393808bcee993f94a3bd1cc";

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn collect_rust_sources(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read tracked Rust source directory") {
        let path = entry.expect("read tracked Rust source entry").path();
        if path.is_dir() {
            collect_rust_sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn tracked_threshold_sources(repo_root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        repo_root.join("Cargo.lock"),
        repo_root.join("Cargo.toml"),
        repo_root.join("core/Cargo.toml"),
        repo_root.join("methods/guest/Cargo.toml"),
        repo_root.join("methods/guest/src/bin/threshold_proof.rs"),
    ];
    collect_rust_sources(&repo_root.join("core/src"), &mut files);
    files.sort_by_key(|path| {
        path.strip_prefix(repo_root)
            .expect("tracked source stays under repository root")
            .to_string_lossy()
            .replace('\\', "/")
    });
    files.dedup();
    files
}

fn source_tree_sha256(repo_root: &Path, files: &[PathBuf]) -> String {
    let mut hasher = Sha256::new();
    for path in files {
        let relative = path
            .strip_prefix(repo_root)
            .expect("tracked source stays under repository root")
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(path).unwrap_or_else(|error| {
            panic!("read tracked threshold source {}: {error}", path.display())
        });
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
        println!("cargo:rerun-if-changed={}", path.display());
    }
    hex(hasher.finalize())
}

fn emit_canonical_threshold_constants() {
    let methods_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let repo_root = methods_dir
        .parent()
        .expect("methods crate lives below repository root");
    let binary_path = methods_dir.join(CANONICAL_THRESHOLD_RELATIVE_PATH);
    let binary = fs::read(&binary_path)
        .unwrap_or_else(|error| panic!("read canonical threshold ProgramBinary: {error}"));
    println!("cargo:rerun-if-changed={}", binary_path.display());

    assert_eq!(
        CANONICAL_THRESHOLD_BYTES,
        binary.len(),
        "canonical threshold ProgramBinary length changed"
    );
    assert_eq!(
        CANONICAL_THRESHOLD_SHA256,
        hex(Sha256::digest(&binary)),
        "canonical threshold ProgramBinary SHA-256 changed"
    );

    let image_id = risc0_binfmt::compute_image_id(&binary)
        .expect("compute canonical threshold ProgramBinary ImageID");
    assert_eq!(
        CANONICAL_THRESHOLD_IMAGE_ID,
        image_id.to_string(),
        "canonical threshold ProgramBinary ImageID changed"
    );

    let tracked_sources = tracked_threshold_sources(repo_root);
    assert_eq!(
        CANONICAL_THRESHOLD_SOURCE_TREE_SHA256,
        source_tree_sha256(repo_root, &tracked_sources),
        "tracked threshold source changed; regenerate and re-attest the canonical ProgramBinary"
    );

    let canonical_path = binary_path
        .canonicalize()
        .expect("canonicalize threshold ProgramBinary path");
    let canonical_path = canonical_path.to_string_lossy();
    let generated = format!(
        "pub const THRESHOLD_PROOF_ELF: &[u8] = include_bytes!({canonical_path:?});\n\
         pub const THRESHOLD_PROOF_PATH: &str = {canonical_path:?};\n\
         pub const THRESHOLD_PROOF_ID: [u32; 8] = {:?};\n",
        image_id.as_words()
    );
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(out_dir.join("canonical_threshold.rs"), generated)
        .expect("write canonical threshold constants");
}

/// Build LP-0002 guests while always embedding one hash-pinned threshold ProgramBinary.
///
/// Native wrapper builds remain suitable for the standalone-sequencer E2E. The threshold
/// prover input is the committed ProgramBinary above, so its ImageID remains identical on
/// macOS and Linux. `RISC0_USE_DOCKER=1` remains available for reproducible wrapper builds.
fn main() {
    emit_canonical_threshold_constants();

    if env::var_os("RISC0_USE_DOCKER").is_some() {
        let docker = DockerOptionsBuilder::default()
            .root_dir("..")
            .build()
            .expect("build DockerOptions");
        let guest = GuestOptionsBuilder::default()
            .use_docker(docker)
            .build()
            .expect("build GuestOptions");
        let mut opts = HashMap::new();
        opts.insert("lp0002-private-multisig-guest", guest);
        embed_methods_with_options(opts);
    } else {
        embed_methods();
    }
}
