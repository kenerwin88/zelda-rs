#!/usr/bin/env python3
"""Promote an oracle-observed boot image to a runtime PNG asset.

The input is a Snes9x RGBA frame. Black is made transparent so the produced
image is a normal direct-color material, not a captured framebuffer overlay.
"""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


NINTENDO_PRESENTS_RECT = (96, 104, 152, 120)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("oracle_frame", type=Path)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("assets/boot/nintendo_presents.png"),
    )
    args = parser.parse_args()

    source = Image.open(args.oracle_frame).convert("RGBA")
    image = source.crop(NINTENDO_PRESENTS_RECT)
    pixels = image.load()
    for y in range(image.height):
        for x in range(image.width):
            red, green, blue, _alpha = pixels[x, y]
            pixels[x, y] = (red, green, blue, 255 if (red, green, blue) != (0, 0, 0) else 0)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    image.save(args.output)


if __name__ == "__main__":
    main()
