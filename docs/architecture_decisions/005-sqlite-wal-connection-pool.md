# Architecture Decision Record 005: SQLite WAL Mode and Multi-Connection Pool

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-06

---

## Context and Problem Statement

The desktop application processes large cataloguing tasks (scanning folders, generating previews, performing bulk AI and stitching backfills) concurrently while the user browses, searches, and tags designs.

Single-connection SQLite handles blocked the UI thread during large batch transactions. Furthermore, setting write pragmas (such as journal mode) on read-only SQLite verification connections produced `(code: 3850) disk I/O error` on Windows. Synchronous startup integrity checks (e.g. `PRAGMA quick_check(1)`) also scanned full B-trees on 7GB+ databases, causing UI freezes.

---

## Decision Drivers

* **Concurrent Reads & Writes:** Background batch inserts and backfills must not block responsive UI search and pagination queries.
* **Instant Verification (<1ms):** Startup database verification must be constant-time regardless of catalogue size.
* **Pragma Lifecycle Safety:** Read-only connections must never attempt write pragmas; WAL mode and synchronous pragmas must be applied cleanly on write-capable pools.

---

## Decision Outcome

1. **WAL Mode & Connection Pooling:** The primary `SqlitePool` is configured with `SqliteJournalMode::Wal`, `SqliteSynchronous::Normal`, a busy timeout of 5000ms, and up to 10 connections (`database/connection.rs`).
2. **Fast Constant-Time Verification:** Schema checks verify `PRAGMA schema_version` and probe key tables with `SELECT 1 FROM settings LIMIT 1; SELECT 1 FROM designs LIMIT 1;`, completing in <1ms without full table scans.
3. **Managed State Lifecycle:** SQLite connections are managed via Tauri managed state (`tauri::State`), with `AppState` owning the pool holder and coordinators.
