# LP-0002 Evaluator Guide

This guide is the shortest reproducible path for evaluating the LP-0002 private
M-of-N multisig submission from a fresh clone.

Repository:

https://github.com/Tranquil-Flow/lp-0002-private-multisig

## 1. Fast clone-and-run path

```bash
git clone https://github.com/Tranquil-Flow/lp-0002-private-multisig.git
cd lp-0002-private-multisig
./demo.sh
```

Expected signal:

- five consumer scenarios run
- shielded member approvals reach threshold
- wrong-context and replay paths are rejected
- the demo states that this root path is the safe-lane mock-receipt path

## 2. Local implementation validation

```bash
RISC0_SKIP_BUILD=1 python3 scripts/validate-submission-readiness.py --skip-exec
```

Expected final line:

```text
PASS: LP-0002 local implementation readiness validator
```

When the RISC0 toolchain and enough memory are available, run without
`--skip-exec` and run the full workspace tests:

```bash
cargo test --workspace
```

## 3. Final publication gate

```bash
python3 scripts/final-publication-check.py
```

The gate recognizes fresh v0.2.4 public evidence and both visible direct links:

```text
GO: LP-0002 final-publication gate passed
```

This gate checks public repository metadata, narrated demo URL, structured
LEZ/evaluator evidence, Basecamp native-package evidence, validation evidence,
benchmarks/cost evidence, and license presence.

## 4. Heavy-lane RISC0 and LEZ evidence

Recording-safe evidence walkthrough:

```bash
bash scripts/demo-heavy-lane.sh
```

Exact current-version standalone E2E, where a LEZ v0.2.4 checkout is available:

```bash
LEZ_SOURCE_DIR=/path/to/logos-execution-zone-v0.2.4 scripts/lez-sequencer-e2e.sh
```

This sets `RISC0_DEV_MODE=0`, starts an isolated standalone sequencer, generates a
fresh proof, deploys the generated ProgramBinary, executes it, and fails unless
canonical block/hash/index inclusion is returned.

A fresh clone contains the audited proof artifacts under
`submission/proof-artifacts/lp0002-risc0-fixture-new/`. The script copies them to
`target/lp0002-risc0-fixture-new/` when the target directory is absent, checks
artifact hashes and manifest freshness metadata, and prints the compact historical
LEZ/NSSA evidence. Current v0.2.4 submission evidence uses LEE. Set
`HEAVY_CARGO_VERIFY=1` on a machine with the Rust/RISC0 host deps to
also run the Rust receipt verifier and regenerate derived evidence.

The heavy lane consists of:

- real `RISC0_DEV_MODE=0` proof artifacts generated and verified by `host/`
- `lp0002-lez-execute-artifacts` running the real receipt through the
  LEZ-shaped execution wrapper
- compact transaction transport carrying receipt/journal commitments
- fresh current-v0.2.4 public deploy/execute confirmation in blocks `2373`/`2374`
- direct explorer routes that now render after an observed indexing delay of about 59 minutes

Structured evidence:

- `submission/LEZ_V024_LOCAL_E2E_EVIDENCE.json`
- `submission/LEZ_V024_PUBLIC_E2E_EVIDENCE.json`
- `submission/TESTNET_EVIDENCE.json`
- `submission/LEZ_COST_BENCHMARKS.json`
- `submission/BENCHMARKS.md`

Important boundary: current LEZ public-program sessions cannot transport the raw
~270 KiB receipt inside one transaction. The included wrapper transaction carries
receipt and journal commitments; the full receipt is host-verified and retained
as file-backed evidence.

## 5. Narrated demo

Fresh human-recorded narrated demo video: https://youtu.be/Wssfp_rkC54

Note: any generated/TTS draft video is not a substitute for the upstream requirement that the builder narrates an end-to-end demo.

## 6. Basecamp package

The submission includes a native Qt/QML Basecamp plugin package, not only static
HTML:

```bash
bash scripts/validate-basecamp-native.sh
```

Build evidence is recorded in:

- `submission/BASECAMP_NATIVE_BUILD.md`

## 7. Upstream PR note

The repository-side implementation and fresh public sequencer/explorer evidence
are ready, with the narrated demo attached: https://youtu.be/Wssfp_rkC54. The
upstream Logos PR must not be opened until Evi explicitly approves. Evi will
decide when to submit it.
