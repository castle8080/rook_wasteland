#!/usr/bin/env python3
"""
Build script for rw_sixzee.

Usage:
    python make.py <target>

Targets:
    build   Debug WASM build (trunk build)
    test    Run unit tests (cargo test) + WASM tests (wasm-pack test --headless --firefox)
    dist    Release WASM build (trunk build --release)
    lint    Run clippy for the WASM target (zero warnings enforced)
    help    Show this message
"""

import sys
import subprocess
import shutil
from pathlib import Path

ROOT = Path(__file__).parent


def _run(*cmd):
    subprocess.run(cmd, cwd=ROOT, check=True)


def build():
    _run("trunk", "build")


def test():
    _run("cargo", "test")
    # Pass --features wasm-test so the library's #[wasm_bindgen(start)] fn
    # is excluded; otherwise its `main` export conflicts with the test
    # harness's own `main` and wasm-ld discards both (see doc/lessons.md L5).
    _run("wasm-pack", "test", "--headless", "--firefox", "--", "--features", "wasm-test")


def dist():
    _run("trunk", "build", "--release")


def e2e():
    # shutil.which is required for cross-platform compatibility: on Windows,
    # `npx` resolves to `npx.CMD` (a batch script) which subprocess cannot
    # invoke without the full path.  On Linux/macOS shutil.which returns the
    # normal binary path; the "npx" fallback works there too if npm is on PATH.
    npx = shutil.which("npx") or "npx"
    _run(npx, "playwright", "test")


def lint():
    _run("cargo", "clippy", "--target", "wasm32-unknown-unknown", "--", "-D", "warnings")


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
