# ADR 011: Cancellation vs. Stopping Semantics and Transactional Boundaries

## Status
Accepted

## Context
Across desktop applications handling long-running background tasks (such as batch metadata extraction, bulk import, database restore, directory sync, and storage migration), user-initiated interruptions typically fall into two fundamentally different categories:

1. **Atomic Rollback (Cancel):** The user wishes to abort an uncommitted action or in-flight process entirely, requiring the system to discard partial state, clean up temporary or partial artifacts, and return the environment to its prior pristine state.
2. **Progressive Halt (Stop):** The user wishes to pause or terminate an iterative, multi-step batch process without losing progress, requiring the system to halt immediately after the current step while strictly committing and retaining all successfully processed items up to that moment.

Historically, UI copy and backend flags across the catalogue used the term "Cancel" ambiguously for both scenarios (e.g. labelling unmatched file batch imports and design folder syncs as "Cancel" even though imported records and copied files were retained). This created confusion around whether data was rolled back or committed.

## Decision

We establish a strict, system-wide architectural distinction between **Cancel** and **Stop** across the UI, Tauri IPC bridge, and backend task coordinator.

### 1. Semantic Definitions & Behavioral Contracts

- **Cancel (Atomic Rollback):**
  - **Definition:** The operation is abandoned; any work in progress is completely rolled back or discarded.
  - **Persistence:** Zero changes made by the cancelled operation persist in SQLite or on disk.
  - **Applications:**
    - Modal / form dialog dismissals without saving.
    - Setup wizard abandonment prior to execution.
    - Database storage migration (`cancel_catalogue_storage_migration` removes the partial destination tree and retains the existing data root).
    - Database backup creation (`request_cancel_backup` deletes partial `.zip` backup archives).
  - **UI Label:** `"Cancel"`.

- **Stop (Progressive Halt):**
  - **Definition:** The operation is halted immediately after the active unit of work completes.
  - **Persistence:** All records and files processed up to the stop point remain valid, committed, and retained in SQLite and on disk.
  - **Applications:**
    - Bulk design importing (`request_stop_bulk_import`).
    - Unified backfill & stitch tag processing (`stop_unified_backfill`).
    - Unmatched files reconciliation (`request_stop_restore`).
    - Design files synchronization / restore (`request_stop_restore`).
  - **UI Label:** `"Stop"` (active state: `"Stopping..."`).

### 2. IPC & Task Coordinator Conventions

1. Task coordinator flags in `src/state.rs` must explicitly reflect the transactional semantic:
   - `*_stop_requested: AtomicBool` for progressive halt loops.
   - `*_cancel_requested: AtomicBool` for rollback loops.
2. IPC command names must use `request_stop_*` for progressive operations and `request_cancel_*` for rollback operations.
3. Status events and toasts must report accurate feedback (e.g. `"Import stopped — 12 file(s) imported."` rather than `"Import cancelled"`).

## Consequences

- **Positive:** Clear, unambiguous user expectations regarding data persistence during background task interruptions.
- **Positive:** Type and name consistency across backend services, IPC routes, and frontend views.
- **Maintenance:** All future background processing features and confirmation modals must adhere to this contract.
