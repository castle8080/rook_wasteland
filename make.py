#!/usr/bin/env python3
"""
Top-level build orchestrator for Rook Wasteland.

Delegates to each app's own make.py, then assembles the combined static tree.

Usage:
    python make.py <target>

Targets:
    build   Debug build for all apps
    test    Run tests for all apps
    dist    Release build all apps and assemble into dist/
    clean   Remove the top-level dist/ directory
    help    Show this message
"""

import sys
import subprocess
import shutil
from pathlib import Path

ROOT = Path(__file__).parent

# Any apps/ subdirectory containing a make.py is treated as a buildable app.
def _apps():
    return sorted(p.parent for p in (ROOT / "apps").glob("*/make.py"))


def _delegate(target):
    for app_dir in _apps():
        print(f"\n==> {app_dir.name}: {target}")
        subprocess.run([sys.executable, "make.py", target], cwd=app_dir, check=True)


def build():
    _delegate("build")


def test():
    _delegate("test")


def dist():
    _delegate("dist")

    out = ROOT / "dist"
    out.mkdir(exist_ok=True)

    for app_dir in _apps():
        src = app_dir / "dist"
        if app_dir.name == "rw_index":
            for item in src.iterdir():
                dst_item = out / item.name
                if dst_item.exists():
                    shutil.rmtree(dst_item) if dst_item.is_dir() else dst_item.unlink()
                (shutil.copytree if item.is_dir() else shutil.copy2)(item, dst_item)
            print(f"  copied {app_dir.name}/dist  →  dist/")
        else:
            dst = out / app_dir.name
            if dst.exists():
                shutil.rmtree(dst)
            shutil.copytree(src, dst)
            print(f"  copied {app_dir.name}/dist  →  dist/{app_dir.name}/")


def clean():
    out = ROOT / "dist"
    if out.exists():
        shutil.rmtree(out)
        print(f"  removed {out}")


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
