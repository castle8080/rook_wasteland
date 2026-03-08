#!/usr/bin/env python3
"""
Build script for rw_sixzee.

Usage:
    python make.py <target>

Targets:
    build   Debug WASM build (trunk build + worker build)
    test    Run unit tests (cargo test) + WASM tests (wasm-pack test --headless --firefox)
    dist    Release WASM build (trunk build --release + worker build --release)
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


def _build_worker(release: bool = False):
    """Build the grandma_worker WASM binary via cargo + wasm-bindgen.

    Produces dist/grandma_worker_core.js and dist/grandma_worker_core_bg.wasm.
    The JS loader assets/grandma_worker.js is copied by Trunk; these two files
    must also land in dist/ so the loader can importScripts + wasm_bindgen them.
    """
    target_dir = ROOT / "target" / "wasm32-unknown-unknown"
    profile = "release" if release else "debug"

    cargo_cmd = [
        "cargo", "build",
        "--target", "wasm32-unknown-unknown",
        "--features", "worker",
    ]
    if release:
        cargo_cmd.append("--release")
    _run(*cargo_cmd)

    wasm_path = target_dir / profile / "rw_sixzee.wasm"

    # wasm-bindgen produces a no-modules shim that works inside a Web Worker.
    # Output into dist/assets/ so it is co-located with the grandma_worker.js
    # loader that Trunk copies there from assets/.
    dist_dir = ROOT / "dist" / "assets"
    dist_dir.mkdir(parents=True, exist_ok=True)
    _run(
        "wasm-bindgen",
        str(wasm_path),
        "--out-dir", str(dist_dir),
        "--target", "no-modules",
        "--no-typescript",
        "--out-name", "grandma_worker_core",
    )
    print("[grandma worker] built -> dist/grandma_worker_core.js + .wasm")


def worker():
    """Build the grandma worker WASM binary (debug). Called by Trunk post_build hook."""
    _build_worker(release=False)


def build():
    # Trunk's post_build hook (see Trunk.toml) runs _build_worker after the
    # main app build, so no explicit call is needed here.
    _run("trunk", "build")


def test():
    _run("cargo", "test")
    # Pass --features wasm-test so the library's #[wasm_bindgen(start)] fn
    # is excluded; otherwise its `main` export conflicts with the test
    # harness's own `main` and wasm-ld discards both (see doc/lessons.md L5).
    _run("wasm-pack", "test", "--headless", "--firefox", "--", "--features", "wasm-test")


def dist():
    _run("trunk", "build", "--release")
    _build_worker(release=True)


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
