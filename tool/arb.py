#!/usr/bin/env python3
"""Adds or replaces strings in the ARB files, English and German in step.

Usage: tool/arb.py [--desktop] < strings.json

The input is a JSON list of objects: {"key", "en", "de", "description",
"placeholders": {"name": "String" | "int"}}. "placeholders" is optional.
The phone's files (lib/l10n) are shared with the desktop; --desktop writes
the strings only the desktop needs (desktop/l10n).
"""

import json
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
desktop = "--desktop" in sys.argv
base = root / ("desktop/l10n" if desktop else "lib/l10n")
prefix = "desktop" if desktop else "app"


def load(locale):
    return json.loads((base / f"{prefix}_{locale}.arb").read_text(encoding="utf-8"))


def save(locale, arb):
    text = json.dumps(arb, ensure_ascii=False, indent=2) + "\n"
    (base / f"{prefix}_{locale}.arb").write_text(text, encoding="utf-8")


en, de = load("en"), load("de")
for item in json.load(sys.stdin):
    key = item["key"]
    en[key] = item["en"]
    meta = {"description": item["description"]}
    if item.get("placeholders"):
        meta["placeholders"] = {n: {"type": t} for n, t in item["placeholders"].items()}
    en["@" + key] = meta
    de[key] = item["de"]
save("en", en)
save("de", de)
