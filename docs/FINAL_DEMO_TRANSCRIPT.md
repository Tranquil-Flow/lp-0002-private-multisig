# LP-0002 final demo read-aloud transcript

Read this while running `bash scripts/record-final-video.sh` and showing the visible Basecamp window on the M4.

## Opening

This is the final LP-0002 private M-of-N multisig resubmission demo. The submission implements a privacy-preserving threshold multisig relation, SDK integration surface, LEZ-shaped execution evidence, and a native Basecamp package.

The previous blocker was Basecamp runtime evidence. The repository now includes `.lgx` install evidence and a real Basecamp runtime-host launch log. This fresh video supplies the remaining visible activation walkthrough.

## Final gate

I am showing the final publication gate first. Before this video URL is inserted, the gate should fail only because the human-recorded narrated demo video URL is missing. After this recording is uploaded and the URL is inserted, this gate should pass.

## Implementation and SDK

Now I run the focused Rust tests and the consumer demo. These show the privacy relation, SDK, verifier, and third-party integration path. The consumer demo imports the SDK like an external application and exercises the intended integration surface.

## Proof and LEZ evidence

Next I show the bundled RISC0_DEV_MODE=0 proof artifact manifest and the LEZ/NSSA public-testnet evidence. The full receipt and journal are file-backed and hash-checked. The LEZ wrapper carries compact receipt and journal commitments because current transport cannot carry the raw receipt payload directly.

This limitation is documented honestly; the submission does not invent unavailable compute-unit counters or claim unsupported transport behavior.

## Basecamp evidence

Now I show the native Basecamp package validation and the runtime-host evidence. The evidence records the LP-0002 `.lgx` package, the loaded component identifier `lp0002_private_multisig`, and a real M4 Basecamp launch log.

The evidence distinguishes package/runtime-host launch from visual component activation. This video is the visual activation evidence.

## Visible activation

Here I switch to the visible LogosBasecamp window and show the LP-0002 private multisig module active in the Basecamp UI. This is the final piece that could not be proven by a log alone.

## Closing

LP-0002 is ready for resubmission once this video URL is inserted into `solutions/LP-0002.md` and `python3 scripts/final-publication-check.py` passes.
