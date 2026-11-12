#!/usr/bin/env python3
"""Draws the app's icon and writes it in every size the apps and packages
need: an indigo square with rounded corners and two cream bars, the Roman
II. Pure Python, no imaging library.

Usage: tool/icon.py
"""

import pathlib
import struct
import zlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
INDIGO = (57, 73, 171)
CREAM = (250, 246, 235)


def pixel(x, y, size):
    """The colour at (x, y) of an icon `size` wide, with its alpha."""
    u, v = (x + 0.5) / size, (y + 0.5) / size
    radius = 0.22
    # Outside the rounded corners.
    cx = min(max(u, radius), 1 - radius)
    cy = min(max(v, radius), 1 - radius)
    if (u - cx) ** 2 + (v - cy) ** 2 > radius**2:
        return (0, 0, 0, 0)
    serif = 0.24 <= u <= 0.76 and (0.24 <= v <= 0.31 or 0.69 <= v <= 0.76)
    bars = 0.24 <= v <= 0.76 and (0.33 <= u <= 0.44 or 0.56 <= u <= 0.67)
    return (*(CREAM if serif or bars else INDIGO), 255)


def png(size):
    rows = b"".join(
        b"\x00" + b"".join(bytes(pixel(x, y, size)) for x in range(size))
        for y in range(size)
    )

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(rows, 9))
        + chunk(b"IEND", b"")
    )


def write(path, size):
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(png(size))
    print(f"{path} ({size} px)")


write("desktop/packaging/icon/icon.png", 1024)
write("desktop/packaging/icon/icon_256.png", 256)
for name, size in [("mdpi", 48), ("hdpi", 72), ("xhdpi", 96), ("xxhdpi", 144), ("xxxhdpi", 192)]:
    write(f"android/app/src/main/res/mipmap-{name}/ic_launcher.png", size)
