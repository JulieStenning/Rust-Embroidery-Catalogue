"""One-off conversion: enable incremental auto-vacuum on a SQLite database.

Usage:
    python scripts/convert_auto_vacuum.py <path-to-db>

The script:
  1. Records the table list + row counts before conversion.
  2. Runs `PRAGMA auto_vacuum = INCREMENTAL;` followed by a full `VACUUM;`
     (required to convert the file structure of an existing database).
  3. Verifies the resulting `auto_vacuum` mode is 2 (INCREMENTAL) and that
     the table list and row counts are identical to the before state.
"""

import os
import sqlite3
import sys


def table_counts(path):
    conn = sqlite3.connect(path)
    try:
        tables = [
            row[0]
            for row in conn.execute(
                "SELECT name FROM sqlite_master WHERE type='table' "
                "AND name NOT LIKE 'sqlite_%' ORDER BY name"
            )
        ]
        counts = {}
        for table in tables:
            counts[table] = conn.execute(
                'SELECT COUNT(*) FROM "{}"'.format(table)
            ).fetchone()[0]
        return counts
    finally:
        conn.close()


def main():
    if len(sys.argv) != 2:
        print("Usage: python convert_auto_vacuum.py <path-to-db>")
        sys.exit(2)

    path = sys.argv[1]
    if not os.path.exists(path):
        print("ERROR: database not found: {}".format(path))
        sys.exit(1)

    before = table_counts(path)
    before_size = os.path.getsize(path)
    print("Before:")
    print("  tables: {}".format(len(before)))
    print("  rows:   {}".format(sum(before.values())))
    print("  size:   {} bytes".format(before_size))

    conn = sqlite3.connect(path)
    try:
        mode_before = conn.execute("PRAGMA auto_vacuum").fetchone()[0]
        print("  auto_vacuum: {}".format(mode_before))

        conn.execute("PRAGMA auto_vacuum = INCREMENTAL")
        conn.execute("VACUUM")
        conn.commit()
    finally:
        conn.close()

    after = table_counts(path)
    after_size = os.path.getsize(path)

    conn = sqlite3.connect(path)
    try:
        mode_after = conn.execute("PRAGMA auto_vacuum").fetchone()[0]
        journal_mode = conn.execute("PRAGMA journal_mode").fetchone()[0]
        freelist = conn.execute("PRAGMA freelist_count").fetchone()[0]
    finally:
        conn.close()

    print("After:")
    print("  auto_vacuum: {}".format(mode_after))
    print("  journal_mode: {}".format(journal_mode))
    print("  freelist_count: {}".format(freelist))
    print("  tables: {}".format(len(after)))
    print("  rows:   {}".format(sum(after.values())))
    print("  size:   {} bytes".format(after_size))

    checks = []
    checks.append(("auto_vacuum == 2 (INCREMENTAL)", mode_after == 2))
    checks.append(("table list unchanged", list(before) == list(after)))
    checks.append(("row counts unchanged", before == after))
    checks.append(
        ("integrity check", sqlite_integrity_ok(path))
    )

    failed = False
    for label, ok in checks:
        status = "OK" if ok else "FAILED"
        print("  [{}] {}".format(status, label))
        if not ok:
            failed = True

    if failed:
        print("CONVERSION FAILED — database left as-is? Check backup.")
        sys.exit(1)

    print("CONVERSION OK: {} -> {}".format(path, after_size))


def sqlite_integrity_ok(path):
    conn = sqlite3.connect(path)
    try:
        result = conn.execute("PRAGMA integrity_check").fetchone()[0]
        return result == "ok"
    finally:
        conn.close()


if __name__ == "__main__":
    main()