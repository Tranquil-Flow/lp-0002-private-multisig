# LP-0002 Validation Evidence

This repository is published with an evaluator-run validation bundle rather than
a GitHub Actions workflow because the publishing OAuth token available in this
automation environment lacks the GitHub `workflow` scope. The commands are kept
plain and reproducible so reviewers can run them directly.

Required validation commands:

```bash
cargo test --workspace
python3 scripts/validate-submission-readiness.py
bash scripts/demo-heavy-lane.sh --live-submit
python3 scripts/final-publication-check.py
```

Latest verified local results before publication:

- `RISC0_SKIP_BUILD=1 cargo test --workspace` passed on the M4 Pro.
- `bash scripts/demo-heavy-lane.sh --live-submit` passed and produced confirmed
  public LEZ testnet evidence in block 39548.
- `python3 scripts/final-publication-check.py` must pass after the human-recorded
  narrated demo video (https://youtu.be/Wssfp_rkC54) and TESTNET_EVIDENCE.json are attached. The previous
  generated/TTS draft video is not sufficient final evidence.
- `RISC0_SKIP_BUILD=1 python3 scripts/validate-submission-readiness.py --skip-exec` passed.
