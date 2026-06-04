# Bugs Filed — LP-0002 Private M-of-N Multisig

Per λPrize submission guidance, this file lists upstream Logos issues encountered while building LP-0002.

## Filed upstream

- **logos-blockchain/logos-blockchain-circuits#33** (CLOSED 2026-05-27 by maintainer) — README missing install path documentation for downstream tools (`~/.logos-blockchain-circuits` / `LOGOS_BLOCKCHAIN_CIRCUITS`). Hits any builder running `cargo install --git logos-co/spel` or compiling code that transitively depends on `logos-blockchain-pol`. Maintainer closed noting the concern belongs against `logos-blockchain-node` (the actual consumer), and confirmed the team is migrating from binary releases to library-based circuits, which addresses the underlying first-run install panic. https://github.com/logos-blockchain/logos-blockchain-circuits/issues/33

## Worked around (candidates for upstream filing)

### 1. `cargo install cargo-risczero` fails on macOS without full Xcode

`cargo install cargo-risczero` hits a panic in `risc0-build-kernel` trying to compile Metal kernels (`xcrun: error: unable to find utility "metal"`). Apple's CLI tools don't include `metal`; only full Xcode does.

**Workaround:** download the prebuilt binaries from the official Risc0 release (`v3.0.5`, `cargo-risczero-aarch64-apple-darwin.tgz`) and place them in `~/.cargo/bin/`.

**Severity:** low — workaround exists, but the first-time failure mode is unhelpful.

### 2. RISC0 receipt size exceeds LEZ public-program session transport limit

The raw ~270 KiB RISC0 receipt is larger than the current LEZ public-program session limit, so we cannot ship the full receipt inside the on-chain wrapper transaction. The full receipt is verified host-side and persisted as file-backed evidence; the LEZ wrapper carries compact receipt/journal commitments only.

**Documented in submission as a known limitation rather than a code workaround.** See `docs/SPEC_COMPLIANCE.md` "Known Limitations" section.

**Severity:** moderate (architectural) — affects any λPrize whose receipt would exceed LEZ session limits.

## Devnet target

Per fryorcraken on Logos Discord (2026-05-11): "devnet == localnet. We don't have a public testnet for lez (yet)." **This is now superseded:** a public LEZ testnet (https://testnet.lez.logos.co/, real consensus, `RISC0_DEV_MODE=0`) subsequently became available, and LP-0002 is now deployed and executed there — deploy tx `82516880f60c2076d78b28ad7b147ac0b05ed247b7bc33a27ac8f68b1d809c56` in block `39547`, execute tx `cb8bfd5afca3c88a99b12b42a6875bcc2cad419d394da0e39d8ca463ee376697` in block `39548`. The historical quote is retained for context only.
