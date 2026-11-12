#!/usr/bin/env python3
"""Writes THIRD-PARTY-NOTICES.md: every third-party component the apps ship,
with its licence. The Rust crates come from `cargo tree` over what the
desktop app and the phone's library link on any platform, the Dart packages from
pubspec.lock, their licences from the packages' own LICENSE files in the
pub cache.

Usage: tool/notices.py
"""

import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent.parent
OWN = ("iibrain-",)


def crates():
    out = subprocess.run(
        [
            "cargo", "tree", "--locked", "--target", "all", "-p", "iibrain-gui", "-p", "iibrain-ffi",
            "-e", "normal,build", "--prefix", "none", "--format", "{p}|{l}",
        ],
        cwd=ROOT / "desktop", check=True, capture_output=True, text=True,
    ).stdout
    found = {}
    for line in out.splitlines():
        package, _, licence = line.partition("|")
        parts = package.split()
        if len(parts) < 2 or parts[0].startswith(OWN):
            continue
        licence = licence.replace("(*)", "").strip()
        found[(parts[0], parts[1].lstrip("v"))] = licence or "see the crate"
    return sorted(found.items())


def dart_licence(name, version):
    cache = pathlib.Path.home() / ".pub-cache/hosted/pub.dev" / f"{name}-{version}"
    for file in ("LICENSE", "LICENSE.md", "LICENSE.txt"):
        path = cache / file
        if path.exists():
            text = path.read_text(errors="replace")
            if "Apache License" in text:
                return "Apache-2.0"
            if "MIT License" in text or "Permission is hereby granted" in text:
                return "MIT"
            if "Redistribution and use" in text:
                return "BSD-3-Clause"
    return "see the package"


def packages():
    lock = (ROOT / "pubspec.lock").read_text()
    found = []
    for name, body in re.findall(r"^  (\w+):\n((?:    .*\n)+)", lock, re.M):
        if '"direct dev"' in body or "sdk: flutter" in body:
            continue
        version = re.search(r'version: "([^"]+)"', body)
        if version:
            found.append((name, version.group(1)))
    return sorted(found)


def table(header, rows):
    widths = [max(len(str(row[i])) for row in [header, *rows]) for i in range(len(header))]
    line = lambda row: "| " + " | ".join(str(c).ljust(w) for c, w in zip(row, widths)) + " |"
    rule = "|" + "|".join("-" * (w + 2) for w in widths) + "|"
    return "\n".join([line(header), rule, *(line(row) for row in rows)])


rust = [(name, version, licence) for (name, version), licence in crates()]
dart = [(name, version, dart_licence(name, version)) for name, version in packages()]

text = f"""# Third-party notices

IIbrain is dual-licensed under Apache-2.0 or MIT. It ships the components
below under their own licences. This file is written by `tool/notices.py`.

## Models

The text recognition reads with the PP-OCRv6 Tiny text detection and text
recognition models of the PaddlePaddle Authors (PaddleOCR), Apache-2.0,
converted to the RTen format by the RustO project. They are built into both
apps; see `desktop/crates/iibrain-core/models/rusto/NOTICE`.

## Rust crates

In the desktop app and in the phone app's core.

{table(("Crate", "Version", "Licence"), rust)}

## Dart and Flutter packages

In the phone app, beside the Flutter framework and engine (BSD-3-Clause).

{table(("Package", "Version", "Licence"), dart)}
"""
(ROOT / "THIRD-PARTY-NOTICES.md").write_text(text)
print(f"{len(rust)} crates, {len(dart)} packages")
