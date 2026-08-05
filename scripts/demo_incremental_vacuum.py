"""Demonstrate incremental_vacuum on a COPY of the live database.

This script:
  1. Copies `data/Database/EmbroideryCatalogue.db` to a temp file.
  2. Inserts ~3,000 rows (~2 KiB each) into a scratch table on the copy,
     then deletes them — simulating a bulk delete that frees pages.
  3. Runs `PRAGMA incremental_vacuum(256)` repeatedly (same stepping logic
     as src/services/compaction.rs) until the freelist is empty.
  4. Reports page_count / freelist_count / file size at each stage, proving
     that reclaimed space is returned to the operating system.

The live database is never touched — only a disposable copy is modified.
"""

import os
import shutil
import sqlite3
import sys
import tempfile

SOURCE = r"data\Database\EmbroideryCatalogue.db"
STEP = 256
MAX_STEPS = 1000  # run until the freelist is fully reclaimed

FILLER = "x" * 2048  # ~2 KiB payload => roughly one 4 KiB page per row


def stats(path):
    conn = sqlite3.connect(path)
    try:
        page_count = conn.execute("PRAGMA page_count").fetchone()[0]
        freelist = conn.execute("PRAGMA freelist_count").fetchone()[0]
    finally:
        conn.close()
    size = os.path.getsize(path)
    return page_count, freelist, size


def main():
    if not os.path.exists(SOURCE):
        print("ERROR: {} not found".format(SOURCE))
        sys.exit(1)

    tmp_dir = tempfile.mkdtemp(prefix="embroidery-vacuum-demo-")
    copy_path = os.path.join(tmp_dir, "demo_copy.db")
    shutil.copy2(SOURCE, copy_path)

    print("Working on a disposable copy: {}".format(copy_path))
    print("Live database is NOT modified.\n")

    pc0, fl0, sz0 = stats(copy_path)
    print("Stage 0 — initial (auto_vacuum=INCREMENTAL copy):")
    print("  page_count={}  freelist_count={}  size={:,} bytes".format(pc0, fl0, sz0))

    conn = sqlite3.connect(copy_path)
    conn.execute("PRAGMA auto_vacuum = INCREMENTAL")
    conn.execute(
        "CREATE TABLE _vacuum_demo (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)"
    )

    print("\nInserting 3,000 rows of ~2 KiB...")
    conn.executemany(
        "INSERT INTO _vacuum_demo (payload) VALUES (?)",
        [("row-{}-{}".format(i, FILLER),) for i in range(3000)],
    )
    conn.commit()
    inserted_pc, _, inserted_sz = stats(copy_path)
    print("  after insert: page_count={}  size={:,} bytes".format(inserted_pc, inserted_sz))

    print("Deleting all 3,000 rows (freelist grows)...")
    conn.execute("DELETE FROM _vacuum_demo")
    conn.commit()
    conn.close()

    pc1, fl1, sz1 = stats(copy_path)
    print("  after delete: freelist_count={}  page_count={}  size={:,} bytes".format(fl1, pc1, sz1))

    print("\nRunning PRAGMA incremental_vacuum({}) in steps...".format(STEP))
    conn = sqlite3.connect(copy_path)
    steps = 0
    before_free = fl1
    while True:
        remaining = conn.execute("PRAGMA freelist_count").fetchone()[0]
        if remaining == 0:
            break
        conn.execute("PRAGMA incremental_vacuum({})".format(min(remaining, STEP)))
        steps += 1
        if steps >= MAX_STEPS:
            break
    conn.close()

    pc2, fl2, sz2 = stats(copy_path)
    reclaimed = before_free - fl2
    print("  steps={}  reclaimed_pages={}".format(steps, reclaimed))
    print("  after: freelist_count={}  page_count={}  size={:,} bytes".format(fl2, pc2, sz2))

    print("\nSummary:")
    print("  initial file size:  {:,} bytes".format(sz0))
    print("  size after delete:  {:,} bytes (+{:,})".format(sz1, sz1 - sz0))
    print("  size after vacuum:  {:,} bytes (-{:,} reclaimed)".format(sz2, sz2 - sz1))
    print("  freelist before:    {}".format(before_free))
    print("  freelist after:     {}".format(fl2))

    os.remove(copy_path)
    os.rmdir(tmp_dir)
    print("\nDisposable copy removed. Live database untouched.")


if __name__ == "__main__":
    main()