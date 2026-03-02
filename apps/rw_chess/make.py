#!/usr/bin/env python3
"""
Build script for rw_chess.

Usage:
    python make.py <target>

Targets:
    build   Debug build (trunk build)
    test    Run unit tests (cargo test)
    dist    Release build (trunk build --release)
    help    Show this message
"""

import sys
import subprocess
from pathlib import Path

ROOT = Path(__file__).parent


def _run(*cmd):
    subprocess.run(cmd, cwd=ROOT, check=True)


def build():
    _run("trunk", "build")


def test():
    _run("cargo", "test")


def dist():
    _run("trunk", "build", "--release")


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
