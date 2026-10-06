# Architecture Decision Record 004: Recovery Mode Side-Effect Gating

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-06

---

## Context and Problem Statement

When the configured data root or SQLite database is missing (e.g., disconnected removable drive, missing folder), the application enters Database Recovery Mode.

In earlier versions, the recovery mode prevented database queries but auxiliary startup hooks continued running. These background tasks inadvertently called `create_dir_all` or initialized logging at the missing path, creating phantom empty folders (`Database/`, `MachineEmbroideryDesigns/`, `logs/`) on disconnected drive letters or creating partial directories before the user could choose a recovery action.

---

## Decision Drivers

* **Single Source of Truth:** The determination that a database or data root is missing/invalid must be computed once and gated everywhere.
* **Zero Side-Effects on Invalid State:** Entering recovery mode must freeze or redirect *every* filesystem side effect touching that path (folder creation, log writes, thumbnail caching, background backfills).
* **Safe State Restoration:** Recovery operations must never overwrite existing files; corrupted files are renamed aside (`<file>.corrupt-<timestamp>`) rather than deleted.

---

## Decision Outcome

1. **Gated Initialization:** All startup routines (logging, background scan workers, thumbnail cache, database migrations) check `paths::database_recovery_mode(&AppPaths)` before touching the data root.
2. **Negative Assertions in Tests:** Test suites explicitly assert that entering recovery mode does not create directories at stale paths.
3. **No Phantom Writes:** When in recovery mode, the frontend router redirects to `/recovery` and backend commands return structured `IpcError::RecoveryMode` responses.
