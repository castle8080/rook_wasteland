#!/usr/bin/env python3
"""
Build script for rw_index.

Usage:
    python make.py <target>

Targets:
    build   No-op (nothing to compile)
    test    No-op (no tests)
    dist    Copy static assets to dist/
    help    Show this message
"""

import sys
import shutil
from pathlib import Path

ROOT = Path(__file__).parent
ASSETS = ["index.html", "apps.json"]


def build():
    print("rw_index: nothing to build.")


def test():
    print("rw_index: nothing to test.")


def dist():
    out = ROOT / "dist"
    out.mkdir(exist_ok=True)
    for name in ASSETS:
        shutil.copy2(ROOT / name, out / name)
        print(f"  copied {name}  →  dist/{name}")


def help():
    print(__doc__)


if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "help"
    fn = globals().get(target)
    if not callable(fn) or target.startswith("_"):
        available = [k for k, v in globals().items() if callable(v) and not k.startswith("_")]
        print(f"Unknown target: '{target}'. Available: {', '.join(sorted(available))}")
        sys.exit(1)
    fn()
