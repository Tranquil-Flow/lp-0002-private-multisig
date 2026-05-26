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

Expected final line:

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

The heavy lane consists of:

- real `RISC0_DEV_MODE=0` proof artifacts generated and verified by `host/`
- `lp0002-lez-execute-artifacts` running the real receipt through the
  LEZ-shaped execution wrapper
- compact NSSA transaction transport carrying receipt/journal commitments
- confirmed LEZ localnet/evaluator inclusion

Structured evidence:

- `submission/TESTNET_EVIDENCE.json`
- `submission/LEZ_COST_BENCHMARKS.json`
- `submission/BENCHMARKS.md`

Important boundary: current LEZ public-program sessions cannot transport the raw
~270 KiB receipt inside one transaction. The included wrapper transaction carries
receipt and journal commitments; the full receipt is host-verified and retained
as file-backed evidence.

## 5. Narrated demo

Public video asset:

https://github.com/Tranquil-Flow/lp-0002-private-multisig/raw/refs/heads/master/submission/lp0002-narrated-demo.mp4

## 6. Basecamp package

The submission includes a native Qt/QML Basecamp plugin package, not only static
HTML:

```bash
bash scripts/validate-basecamp-native.sh
```

Build evidence is recorded in:

- `submission/BASECAMP_NATIVE_BUILD.md`

## 7. Upstream PR note

The repository-side package is ready. The upstream Logos PR is intentionally not
opened by automation; Evi will decide when to submit it.
