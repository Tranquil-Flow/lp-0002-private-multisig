use std::collections::HashMap;

use risc0_build::{
    embed_methods, embed_methods_with_options, DockerOptionsBuilder, GuestOptionsBuilder,
};

/// Build the LP-0002 guests.
///
/// Default (CI / fast iteration): a native `embed_methods()` build — no Docker,
/// keeping the per-push CI gate toolchain-light.
///
/// Reproducible evidence build: set `RISC0_USE_DOCKER=1`. This embeds the guests
/// built inside the pinned `risczero/risc0-guest-builder` Docker image, so the
/// resulting `*_ID` constants are the deterministic, content-addressed ImageIDs an
/// evaluator reproduces with `cargo risczero build`. Used to generate the on-chain
/// program id, the threshold-proof receipt fixture, and all verification evidence.
fn main() {
    if std::env::var_os("RISC0_USE_DOCKER").is_some() {
        // root_dir is the Docker build context; build.rs runs in `methods/`, so the
        // repo root (one level up) carries the guest crate and its `../core` path dep.
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
