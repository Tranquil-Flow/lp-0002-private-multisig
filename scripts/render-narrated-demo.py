#!/usr/bin/env python3
"""Render a compact narrated LP-0002 demo video asset.

This is not a substitute for a human QuickTime walkthrough, but it creates an
accessible public video asset for reviewers: slides + macOS TTS narration that
summarize the implementation, proof path, localnet/evaluator evidence, and how
to reproduce it.
"""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "submission" / "lp0002-narrated-demo.mp4"
WORK = ROOT / "target" / "lp0002-demo-video"
SLIDES = WORK / "slides"
NARRATION = WORK / "narration.txt"
AUDIO = WORK / "narration.aiff"
SWIFT = ROOT / "scripts" / "render_slideshow_with_audio.swift"

VIDEO_URL = "https://github.com/Tranquil-Flow/lp-0002-private-multisig/raw/refs/heads/master/submission/lp0002-narrated-demo.mp4"

SLIDE_DATA = [
    (
        "LP-0002: Private M-of-N Multisig",
        [
            "Shielded members approve proposals without revealing which members signed.",
            "Public journal exposes only proposal binding, threshold count, nullifiers, and proof id.",
            "Repository: github.com/Tranquil-Flow/lp-0002-private-multisig",
        ],
    ),
    (
        "Architecture",
        [
            "Rust workspace: core, SDK, consumer demo, verifier, RISC0 methods, host bridge, LEZ wrapper.",
            "Consumer demo imports the SDK as a library and runs five realistic integration scenarios.",
            "Native Qt/QML Basecamp module is included alongside the browser preview.",
        ],
    ),
    (
        "Privacy relation",
        [
            "Member commitments and secrets remain private witness data.",
            "Nullifiers bind member, multisig, and proposal to prevent double voting and cross-proposal replay.",
            "The verifier rejects wrong roots, wrong thresholds, malformed receipts, and replayed execution.",
        ],
    ),
    (
        "Heavy lane proof",
        [
            "RISC0_DEV_MODE=0 proof artifacts are generated and verified host-side.",
            "Image id: 026e95199ae495d946f7632d721823def2756584332c771a64207114311d4f01",
            "Proof id: 9e6492e73d1e8382abfa0e94e91842100b9041516857f215fcad7276cbad8b11",
        ],
    ),
    (
        "LEZ localnet / evaluator testnet evidence",
        [
            "The Logos evaluator testnet target for this prize is the LEZ localnet fixture.",
            "Confirmed wrapper tx: 596ddb4d798c3e45b2c4da9a15a33638ccf85f54aec7efa52cf822a87591d599",
            "Included in block 1995 at transaction index 0.",
        ],
    ),
    (
        "Honesty boundary",
        [
            "Current LEZ public-program sessions cannot carry the raw 270 KiB RISC0 receipt as input.",
            "The localnet wrapper carries a receipt/journal commitment; the full receipt is retained as file evidence.",
            "This limitation is documented instead of hidden or guessed around.",
        ],
    ),
    (
        "Reproduce",
        [
            "Run: cargo test --workspace",
            "Run: python3 scripts/validate-submission-readiness.py",
            "Run: bash scripts/demo-heavy-lane.sh --live-submit for fresh localnet evidence.",
        ],
    ),
]

NARRATION_TEXT = """
LP zero zero zero two implements a private M of N multisig for Logos Execution Zone.
Members can approve a proposal without revealing which shielded accounts participated.
The public journal exposes proposal binding, threshold count, nullifiers, and a proof id, while member secrets and commitments remain private witness data.

The repository is a Rust workspace with core threshold logic, a high level SDK, a consumer demo, a verifier program, RISC Zero methods, a host bridge, and a LEZ shaped wrapper.
The consumer demo imports the SDK as a library and exercises five realistic integration scenarios, matching the maintainer accepted external use requirement.

The heavy lane generates and verifies real RISC Zero proof artifacts with RISC0 DEV MODE set to zero.
The current proof image id is zero two six e nine five one nine, ending in four f zero one.
The proof id is nine e six four nine two e seven, ending in eight b one one.

For LP zero zero zero two, the evaluator testnet target is the LEZ localnet fixture.
The wrapper transaction was confirmed on localnet with hash five nine six d d b four d, ending in nine one d five nine nine.
It was included in block nineteen ninety five at transaction index zero.

One limitation is documented honestly. The current LEZ public program session limit cannot carry the raw two hundred seventy kilobyte RISC Zero receipt inside the public wrapper input.
The wrapper therefore carries a receipt and journal commitment, while the full receipt remains retained and verified as file backed evidence.

To reproduce the submission, clone the repository, run cargo test workspace, run the readiness validator, and run the heavy lane demo script with live submit for fresh localnet evidence.
""".strip()


def font(size: int, bold: bool = False):
    candidates = [
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf" if bold else "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial.ttf",
    ]
    for c in candidates:
        if c and Path(c).exists():
            try:
                return ImageFont.truetype(c, size=size)
            except Exception:
                pass
    return ImageFont.load_default()


def wrap(draw: ImageDraw.ImageDraw, text: str, fnt, max_width: int) -> list[str]:
    words = text.split()
    lines: list[str] = []
    cur = ""
    for w in words:
        test = (cur + " " + w).strip()
        if draw.textbbox((0, 0), test, font=fnt)[2] <= max_width:
            cur = test
        else:
            if cur:
                lines.append(cur)
            cur = w
    if cur:
        lines.append(cur)
    return lines


def make_slides() -> None:
    if WORK.exists():
        shutil.rmtree(WORK)
    SLIDES.mkdir(parents=True)
    title_font = font(50, True)
    body_font = font(30)
    small_font = font(22)
    for idx, (title, bullets) in enumerate(SLIDE_DATA):
        img = Image.new("RGB", (1280, 720), (8, 14, 30))
        draw = ImageDraw.Draw(img)
        # soft moon gradient
        for y in range(720):
            shade = int(20 + y * 0.04)
            draw.line((0, y, 1280, y), fill=(8, 14 + shade // 3, 30 + shade))
        draw.ellipse((1040, 60, 1180, 200), fill=(210, 225, 255), outline=(150, 170, 230), width=3)
        draw.text((70, 70), title, font=title_font, fill=(232, 239, 255))
        y = 180
        for bullet in bullets:
            lines = wrap(draw, bullet, body_font, 1040)
            draw.text((90, y), "•", font=body_font, fill=(132, 210, 255))
            for line in lines:
                draw.text((130, y), line, font=body_font, fill=(230, 234, 244))
                y += 42
            y += 24
        draw.text((70, 650), f"LP-0002 narrated demo · slide {idx+1}/{len(SLIDE_DATA)}", font=small_font, fill=(150, 164, 190))
        img.save(SLIDES / f"slide_{idx:03d}.png")
    NARRATION.write_text(NARRATION_TEXT + "\n")


def run(cmd: list[str]) -> None:
    print("$", " ".join(cmd))
    subprocess.run(cmd, cwd=ROOT, check=True)


def main() -> int:
    if not SWIFT.exists():
        print(f"missing Swift renderer: {SWIFT}", file=sys.stderr)
        return 1
    make_slides()
    if not shutil.which("say"):
        print("macOS say command is required", file=sys.stderr)
        return 1
    run(["say", "-o", str(AUDIO), "-f", str(NARRATION)])
    OUT.parent.mkdir(exist_ok=True)
    run(["swift", str(SWIFT), str(SLIDES), str(AUDIO), str(OUT)])
    size = OUT.stat().st_size
    print(json.dumps({"video": str(OUT), "bytes": size, "url": VIDEO_URL}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
