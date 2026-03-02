#!/usr/bin/env python3
"""
Rebuild public/poems/poems_index.json by scanning all poem JSON files
under public/poems/authors/.

Usage:
    python3 scripts/build_poems_index.py

Run this any time you add, rename, or remove poem files.
The output is sorted by author then title for a stable, diffable index.
"""

import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent
POEMS_DIR = REPO_ROOT / "public" / "poems"
AUTHORS_DIR = POEMS_DIR / "authors"
INDEX_PATH = POEMS_DIR / "poems_index.json"


def build_index() -> list[dict]:
    if not AUTHORS_DIR.exists():
        print(f"ERROR: authors directory not found: {AUTHORS_DIR}", file=sys.stderr)
        sys.exit(1)

    entries = []
    errors = 0

    for poem_file in sorted(AUTHORS_DIR.rglob("*.json")):
        try:
            data = json.loads(poem_file.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            print(f"  SKIP (invalid JSON): {poem_file.relative_to(POEMS_DIR)} — {e}", file=sys.stderr)
            errors += 1
            continue

        poem_id = data.get("id")
        title = data.get("title")
        author = data.get("author")

        if not all([poem_id, title, author]):
            print(f"  SKIP (missing id/title/author): {poem_file.relative_to(POEMS_DIR)}", file=sys.stderr)
            errors += 1
            continue

        # Build site-root path from file location relative to public/
        rel = poem_file.relative_to(REPO_ROOT / "public")
        site_path = "/" + str(rel).replace("\\", "/")

        entries.append({
            "id": poem_id,
            "path": site_path,
            "title": title,
            "author": author,
        })

    # Stable sort: author then title
    entries.sort(key=lambda e: (e["author"].lower(), e["title"].lower()))
    return entries, errors


def main():
    print(f"Scanning: {AUTHORS_DIR}")
    entries, errors = build_index()

    index = {
        "version": 1,
        "poems": entries,
    }

    INDEX_PATH.write_text(
        json.dumps(index, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    print(f"Written:  {INDEX_PATH}")
    print(f"Poems:    {len(entries)}")
    if errors:
        print(f"Skipped:  {errors} files with errors (see stderr)", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
