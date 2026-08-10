#!/usr/bin/env python3
"""Render a compact narrated LP-0002 demo video asset.

This is not a substitute for a human QuickTime walkthrough, but it creates an
accessible public video asset for reviewers: slides + macOS TTS narration that
summarize the implementation, proof path, public-testnet evidence, and how
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
CURRENT_EVIDENCE_PATH = ROOT / "submission" / "LEZ_V024_PUBLIC_E2E_EVIDENCE.json"
CURRENT_EVIDENCE = json.loads(CURRENT_EVIDENCE_PATH.read_text())
CURRENT_IMAGE_ID = CURRENT_EVIDENCE["proof"]["threshold_proof_image_id"]
CURRENT_EXECUTION = CURRENT_EVIDENCE["execution"]

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
            "Native Qt/QML Basecamp module is included as the Basecamp integration surface.",
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
            "Current standalone E2E generates and verifies a fresh RISC0_DEV_MODE=0 proof.",
            f"Current image id: {CURRENT_IMAGE_ID}",
            "Archived old-image proof artifacts remain explicitly labeled as historical evidence.",
        ],
    ),
    (
        "Public LEZ testnet evidence",
        [
            "The deployment target for this prize is the public LEZ testnet (https://testnet.lez.logos.co/).",
            f"Current proof-backed execute tx: {CURRENT_EXECUTION['tx_hash']}",
            f"Explorer-visible in block {CURRENT_EXECUTION['included_block_id']} at transaction index {CURRENT_EXECUTION['included_tx_index']}.",
        ],
    ),
    (
        "Honesty boundary",
        [
            "Current LEZ public-program sessions cannot carry the raw 270 KiB RISC0 receipt as input.",
            "The public-testnet wrapper carries a receipt/journal commitment; the full receipt is retained as file evidence.",
            "This limitation is documented instead of hidden or guessed around.",
        ],
    ),
    (
        "Reproduce",
        [
            "Run: cargo test --workspace",
            "Run: python3 scripts/validate-submission-readiness.py",
            "Run: bash scripts/demo-heavy-lane.sh --current-e2e for a fresh standalone proof and inclusion check.",
        ],
    ),
]

NARRATION_TEXT = """
LP zero zero zero two implements a private M of N multisig for Logos Execution Zone.
Members can approve a proposal without revealing which shielded accounts participated.
The public journal exposes proposal binding, threshold count, nullifiers, and a proof id, while member secrets and commitments remain private witness data.

The repository is a Rust workspace with core threshold logic, a high level SDK, a consumer demo, a verifier program, RISC Zero methods, a host bridge, and a LEZ shaped wrapper.
The consumer demo imports the SDK as a library and exercises five realistic integration scenarios, matching the maintainer accepted external use requirement.

The current heavy lane generates and verifies a fresh RISC Zero proof with RISC0 DEV MODE set to zero.
The current wrapper and receipt image identifier are checked in lockstep. Archived old-image proof artifacts are labeled as historical rather than presented as current.

For LP zero zero zero two, the deployment target is the public LEZ testnet.
The current proof-backed wrapper transaction begins four a nine eight five d six f and is explorer-visible in block two thousand three hundred seventy-four at transaction index zero.

One limitation is documented honestly. The current LEZ public program session limit cannot carry the raw two hundred seventy kilobyte RISC Zero receipt inside the public wrapper input.
The wrapper therefore carries a receipt and journal commitment, while the full receipt remains retained and verified as file backed evidence.

To reproduce the submission, clone the repository, run cargo test workspace, run the readiness validator, and run the heavy lane demo script with current e two e for a fresh standalone proof and canonical inclusion check.
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
