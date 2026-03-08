#!/usr/bin/env python3
"""
Build script for rw_sixzee.

Usage:
    python make.py <target>

Targets:
    build   Debug WASM build (worker build -> trunk build)
    serve   Pre-build worker then start trunk serve (use instead of bare trunk serve)
    test    Run unit tests (cargo test) + WASM tests (wasm-pack test --headless --firefox)
    dist    Release WASM build (worker build -> trunk build --release)
    lint    Run clippy for the WASM target (zero warnings enforced)
    worker  Build only the grandma worker WASM binary (debug)
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

    Outputs grandma_worker_core.js and grandma_worker_core_bg.wasm into the
    SOURCE assets/ directory (not dist/).  Trunk's [[copy-dir]] then picks
    them up during its staging step so they survive the atomic distribution
    swap.  Call this BEFORE trunk build/serve, not after.
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

    # Write into source assets/ so Trunk's [[copy-dir]] stages these files
    # alongside grandma_worker.js and grandma_quotes.json.  They are
    # gitignored (see .gitignore) since they are build artifacts.
    assets_dir = ROOT / "assets"
    assets_dir.mkdir(parents=True, exist_ok=True)
    _run(
        "wasm-bindgen",
        str(wasm_path),
        "--out-dir", str(assets_dir),
        "--target", "no-modules",
        "--no-typescript",
        "--out-name", "grandma_worker_core",
    )
    print("[grandma worker] built -> assets/grandma_worker_core.js + .wasm")


def worker():
    """Build the grandma worker WASM binary (debug) into assets/."""
    _build_worker(release=False)


def build():
    # Build the worker INTO source assets/ first so Trunk's [[copy-dir]]
    # stages it during the distribution swap (see doc/lessons.md L17).
    _build_worker()
    _run("trunk", "build")


def serve():
    """Pre-build the grandma worker then start trunk serve.

    Always use this instead of bare `trunk serve` — Trunk's post_build hooks
    run before the atomic distribution swap and cannot reliably place files
    into dist/.  Pre-building the worker into assets/ ensures [[copy-dir]]
    includes it in every subsequent staged build.
    """
    _build_worker()
    _run("trunk", "serve", "--no-autoreload")


def test():
    _run("cargo", "test")
    # Pass --features wasm-test so the library's #[wasm_bindgen(start)] fn
    # is excluded; otherwise its `main` export conflicts with the test
    # harness's own `main` and wasm-ld discards both (see doc/lessons.md L5).
    _run("wasm-pack", "test", "--headless", "--firefox", "--", "--features", "wasm-test")


def dist():
    # Same pre-build pattern: worker into assets/ before trunk stages.
    _build_worker(release=True)
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
