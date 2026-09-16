# Frontend API & IPC Adapter Layer

This directory contains the client-side IPC (Inter-Process Communication) bridge between the Svelte frontend and the Tauri Rust backend.

## Architectural Design

The frontend never invokes raw Tauri commands directly in view components. Instead, all communication passes through typed domain adapters built on top of a centralized IPC transport layer.

```
Svelte Components / Views / Stores
               │
               ▼
   Domain Adapters (*Adapter.ts)
               │
               ▼
        ipcClient.ts (invokeLoose)
               │
   ┌───────────┴───────────┐
   ▼                       ▼
Tauri IPC Backend    E2E Mock Stubs (__E2E_IPC_STUBS__)
```

## Core Modules

### 1. Central IPC Client (`ipcClient.ts`)

- Wraps Tauri's `@tauri-apps/api/core` `invoke()` method.
- **E2E Test Interceptor:** Supports `window.__E2E_IPC_STUBS__` to inject synthetic responses during Playwright / Vitest tests without requiring a compiled Rust binary.
- Provides consistent Promise rejection and error forwarding.

### 2. Backward-Compatible Barrel (`commandAdapter.ts`)

- Re-exports all domain adapters from a single location for ergonomic imports:
  ```typescript
  import { getDesigns, saveSettings, startBackfill } from "$lib/api/commandAdapter";
  ```

### 3. Domain Adapters

Each domain adapter corresponds to a backend route module in `src/routes/`:

- **`designsAdapter.ts`**: Querying catalogue designs, search filtering, tag queries, design detail inspection, stitch rendering data, and metadata editing.
- **`projectsAdapter.ts`**: Project CRUD, adding/removing designs from projects, and project hierarchy management.
- **`importAdapter.ts`**: Folder picker invocations, directory scanning, and bulk import execution.
- **`tagsAdapter.ts`**: Tag definitions, tag groups, automated tag rules, and bulk tag assignments.
- **`batchOperationsAdapter.ts`**: Starting, monitoring, and cancelling long-running tasks (thumbnail generation, AI metadata backfill).
- **`settingsAdapter.ts`**: Application settings, theme preferences, library root directories, and API key management.
- **`adminAdapter.ts`**: Database health diagnostics, compaction, vacuuming, and stats.
- **`backupAdapter.ts`**: Database backup creation, archive inspection, and database restoration.
- **`orphansAdapter.ts`**: Detection and cleanup of unlinked files and stale thumbnail cache entries.

## Error Handling Pattern

Domain adapters normalize backend errors (`AppError`) into standard JavaScript exceptions with user-actionable messages. Views can catch these errors and dispatch notifications via `toastStore`.
