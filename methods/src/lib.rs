#[allow(dead_code)]
mod native_generated {
    include!(concat!(env!("OUT_DIR"), "/methods.rs"));
}

pub use native_generated::{
    VERIFY_AND_EXECUTE_BYTES_ELF, VERIFY_AND_EXECUTE_BYTES_ID, VERIFY_AND_EXECUTE_BYTES_PATH,
};

// Threshold proof generation must use the same content-addressed ProgramBinary on every
// host. build.rs verifies its SHA-256, RISC0 ImageID, and tracked source-tree commitment
// before emitting these constants.
include!(concat!(env!("OUT_DIR"), "/canonical_threshold.rs"));
